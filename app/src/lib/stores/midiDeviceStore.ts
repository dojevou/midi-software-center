import { derived, get, writable } from 'svelte/store';
import { safeInvoke } from '$lib/utils/tauri';
import { Commands } from '$lib/api/commands';

// ============================================================================
// TYPES
// ============================================================================

export interface MIDIDevice {
  id: string;
  name: string;
  manufacturer: string;
  type: 'input' | 'output' | 'both';
  state: 'connected' | 'disconnected';
  connection: 'open' | 'closed' | 'pending';
  version?: string;
  isConnected?: boolean;
}

export interface MIDIMessage {
  deviceId: string;
  timestamp: number;
  data: number[];
  type: MIDIMessageType;
  channel: number;
  note?: number;
  velocity?: number;
  controller?: number;
  value?: number;
  program?: number;
  pitchBend?: number;
}

export type MIDIMessageType =
  | 'note-on'
  | 'note-off'
  | 'control-change'
  | 'program-change'
  | 'pitch-bend'
  | 'aftertouch'
  | 'poly-aftertouch'
  | 'sysex'
  | 'clock'
  | 'start'
  | 'stop'
  | 'continue'
  | 'active-sensing'
  | 'reset'
  | 'unknown';

export interface MIDIMapping {
  id: string;
  deviceId: string;
  messageType: MIDIMessageType;
  channel: number;
  controller?: number;
  note?: number;
  action: string;
  parameter?: string;
  minValue?: number;
  maxValue?: number;
  invert?: boolean;
}

export interface MIDIDeviceSettings {
  inputEnabled: boolean;
  outputEnabled: boolean;
  clockSync: boolean;
  midiThrough: boolean;
  channelFilter: number[]; // Empty = all channels
  messageFilter: MIDIMessageType[]; // Empty = all messages
}

export interface MIDIDeviceState {
  devices: MIDIDevice[];
  activeInputs: string[];
  activeOutputs: string[];
  deviceSettings: Record<string, MIDIDeviceSettings>;
  mappings: MIDIMapping[];
  learningMode: boolean;
  learnTarget: string | null;
  lastMessage: MIDIMessage | null;
  messageHistory: MIDIMessage[];
  historyLimit: number;
  isScanning: boolean;
  error: string | null;
}

// ============================================================================
// DEFAULTS
// ============================================================================

const defaultDeviceSettings: MIDIDeviceSettings = {
  inputEnabled: true,
  outputEnabled: true,
  clockSync: false,
  midiThrough: false,
  channelFilter: [],
  messageFilter: [],
};

// ============================================================================
// STORE
// ============================================================================

const initialState: MIDIDeviceState = {
  devices: [],
  activeInputs: [],
  activeOutputs: [],
  deviceSettings: {},
  mappings: [],
  learningMode: false,
  learnTarget: null,
  lastMessage: null,
  messageHistory: [],
  historyLimit: 100,
  isScanning: false,
  error: null,
};

const { subscribe, set, update } = writable<MIDIDeviceState>(initialState);

export const midiDeviceStore = { subscribe };

// ============================================================================
// DERIVED STORES
// ============================================================================

export const inputDevices = derived(midiDeviceStore, ($store) =>
  $store.devices.filter((d) => d.type === 'input')
);

export const outputDevices = derived(midiDeviceStore, ($store) =>
  $store.devices.filter((d) => d.type === 'output')
);

export const connectedDevices = derived(midiDeviceStore, ($store) =>
  $store.devices.filter((d) => d.state === 'connected')
);

export const activeInputDevices = derived(midiDeviceStore, ($store) =>
  $store.devices.filter((d) => $store.activeInputs.includes(d.id))
);

export const activeOutputDevices = derived(midiDeviceStore, ($store) =>
  $store.devices.filter((d) => $store.activeOutputs.includes(d.id))
);

export const hasActiveInputs = derived(midiDeviceStore, ($store) => $store.activeInputs.length > 0);

export const hasActiveOutputs = derived(
  midiDeviceStore,
  ($store) => $store.activeOutputs.length > 0
);

export const isLearning = derived(midiDeviceStore, ($store) => $store.learningMode);

// ============================================================================
// MESSAGE PARSING
// ============================================================================

function parseMIDIMessage(deviceId: string, data: number[], timestamp: number): MIDIMessage {
  const statusByte = data[0];
  const messageType = statusByte & 0xf0;
  const channel = (statusByte & 0x0f) + 1;

  const message: MIDIMessage = {
    deviceId,
    timestamp,
    data,
    type: 'unknown',
    channel,
  };

  switch (messageType) {
    case 0x90: // Note On
      message.type = data[2] === 0 ? 'note-off' : 'note-on';
      message.note = data[1];
      message.velocity = data[2];
      break;
    case 0x80: // Note Off
      message.type = 'note-off';
      message.note = data[1];
      message.velocity = data[2];
      break;
    case 0xb0: // Control Change
      message.type = 'control-change';
      message.controller = data[1];
      message.value = data[2];
      break;
    case 0xc0: // Program Change
      message.type = 'program-change';
      message.program = data[1];
      break;
    case 0xe0: // Pitch Bend
      message.type = 'pitch-bend';
      message.pitchBend = (data[2] << 7) | data[1];
      break;
    case 0xd0: // Channel Aftertouch
      message.type = 'aftertouch';
      message.value = data[1];
      break;
    case 0xa0: // Poly Aftertouch
      message.type = 'poly-aftertouch';
      message.note = data[1];
      message.value = data[2];
      break;
    case 0xf0: // System messages
      if (statusByte === 0xf0) {
        message.type = 'sysex';
      } else if (statusByte === 0xf8) {
        message.type = 'clock';
      } else if (statusByte === 0xfa) {
        message.type = 'start';
      } else if (statusByte === 0xfb) {
        message.type = 'continue';
      } else if (statusByte === 0xfc) {
        message.type = 'stop';
      } else if (statusByte === 0xfe) {
        message.type = 'active-sensing';
      } else if (statusByte === 0xff) {
        message.type = 'reset';
      }
      break;
  }

  return message;
}

// ============================================================================
// ACTIONS
// ============================================================================

export const midiDeviceActions = {
  /**
   * Scan for available MIDI devices
   */
  async scanDevices(): Promise<MIDIDevice[]> {
    try {
      update((state) => ({ ...state, isScanning: true, error: null }));

      const devices = (await safeInvoke<MIDIDevice[]>(Commands.SCAN_MIDI_DEVICES)) || [];

      update((state) => {
        // Preserve settings for known devices
        const settings = { ...state.deviceSettings };
        devices.forEach((device) => {
          if (!settings[device.id]) {
            settings[device.id] = { ...defaultDeviceSettings };
          }
        });

        return {
          ...state,
          devices,
          deviceSettings: settings,
          isScanning: false,
        };
      });

      return devices;
    } catch (error) {
      console.error('Failed to scan MIDI devices:', error);
      update((state) => ({
        ...state,
        isScanning: false,
        error: String(error),
      }));
      return [];
    }
  },

  /**
   * Connect to a MIDI input device
   */
  async connectInput(deviceId: string): Promise<boolean> {
    try {
      await safeInvoke(Commands.CONNECT_MIDI_INPUT, { deviceId });

      update((state) => ({
        ...state,
        activeInputs: [...new Set([...state.activeInputs, deviceId])],
        devices: state.devices.map((d) =>
          d.id === deviceId ? { ...d, connection: 'open' as const } : d
        ),
      }));

      return true;
    } catch (error) {
      console.error('Failed to connect MIDI input:', error);
      update((state) => ({ ...state, error: String(error) }));
      return false;
    }
  },

  /**
   * Disconnect from a MIDI input device
   */
  async disconnectInput(deviceId: string): Promise<boolean> {
    try {
      await safeInvoke(Commands.DISCONNECT_MIDI_INPUT, { deviceId });

      update((state) => ({
        ...state,
        activeInputs: state.activeInputs.filter((id) => id !== deviceId),
        devices: state.devices.map((d) =>
          d.id === deviceId ? { ...d, connection: 'closed' as const } : d
        ),
      }));

      return true;
    } catch (error) {
      console.error('Failed to disconnect MIDI input:', error);
      update((state) => ({ ...state, error: String(error) }));
      return false;
    }
  },

  /**
   * Connect to a MIDI output device
   */
  async connectOutput(deviceId: string): Promise<boolean> {
    try {
      await safeInvoke(Commands.CONNECT_MIDI_OUTPUT, { deviceId });

      update((state) => ({
        ...state,
        activeOutputs: [...new Set([...state.activeOutputs, deviceId])],
        devices: state.devices.map((d) =>
          d.id === deviceId ? { ...d, connection: 'open' as const } : d
        ),
      }));

      return true;
    } catch (error) {
      console.error('Failed to connect MIDI output:', error);
      update((state) => ({ ...state, error: String(error) }));
      return false;
    }
  },

  /**
   * Disconnect from a MIDI output device
   */
  async disconnectOutput(deviceId: string): Promise<boolean> {
    try {
      await safeInvoke(Commands.DISCONNECT_MIDI_OUTPUT, { deviceId });

      update((state) => ({
        ...state,
        activeOutputs: state.activeOutputs.filter((id) => id !== deviceId),
        devices: state.devices.map((d) =>
          d.id === deviceId ? { ...d, connection: 'closed' as const } : d
        ),
      }));

      return true;
    } catch (error) {
      console.error('Failed to disconnect MIDI output:', error);
      update((state) => ({ ...state, error: String(error) }));
      return false;
    }
  },

  /**
   * Send MIDI message to output device(s)
   */
  async sendMessage(message: number[], deviceId?: string): Promise<void> {
    try {
      const state = get(midiDeviceStore);
      const targets = deviceId ? [deviceId] : state.activeOutputs;

      await Promise.all(
        targets.map((id) => safeInvoke(Commands.SEND_MIDI_MESSAGE, { deviceId: id, message }))
      );
    } catch (error) {
      console.error('Failed to send MIDI message:', error);
    }
  },

  /**
   * Send Note On message
   */
  async sendNoteOn(
    note: number,
    velocity: number = 100,
    channel: number = 1,
    deviceId?: string
  ): Promise<void> {
    const statusByte = 0x90 | ((channel - 1) & 0x0f);
    await this.sendMessage([statusByte, note & 0x7f, velocity & 0x7f], deviceId);
  },

  /**
   * Send Note Off message
   */
  async sendNoteOff(
    note: number,
    velocity: number = 0,
    channel: number = 1,
    deviceId?: string
  ): Promise<void> {
    const statusByte = 0x80 | ((channel - 1) & 0x0f);
    await this.sendMessage([statusByte, note & 0x7f, velocity & 0x7f], deviceId);
  },

  /**
   * Send Control Change message
   */
  async sendControlChange(
    controller: number,
    value: number,
    channel: number = 1,
    deviceId?: string
  ): Promise<void> {
    const statusByte = 0xb0 | ((channel - 1) & 0x0f);
    await this.sendMessage([statusByte, controller & 0x7f, value & 0x7f], deviceId);
  },

  /**
   * Send Program Change message
   */
  async sendProgramChange(program: number, channel: number = 1, deviceId?: string): Promise<void> {
    const statusByte = 0xc0 | ((channel - 1) & 0x0f);
    await this.sendMessage([statusByte, program & 0x7f], deviceId);
  },

  /**
   * Send Pitch Bend message
   */
  async sendPitchBend(
    value: number, // 0-16383, 8192 = center
    channel: number = 1,
    deviceId?: string
  ): Promise<void> {
    const statusByte = 0xe0 | ((channel - 1) & 0x0f);
    const lsb = value & 0x7f;
    const msb = (value >> 7) & 0x7f;
    await this.sendMessage([statusByte, lsb, msb], deviceId);
  },

  /**
   * Send All Notes Off (CC 123)
   */
  async sendAllNotesOff(channel: number = 1, deviceId?: string): Promise<void> {
    await this.sendControlChange(123, 0, channel, deviceId);
  },

  /**
   * Panic - Send All Notes Off on all channels
   */
  async panic(deviceId?: string): Promise<void> {
    for (let channel = 1; channel <= 16; channel++) {
      await this.sendAllNotesOff(channel, deviceId);
    }
  },

  // ============================================================================
  // MESSAGE HANDLING
  // ============================================================================

  /**
   * Process incoming MIDI message
   */
  handleIncomingMessage(deviceId: string, data: number[], timestamp: number): void {
    const message = parseMIDIMessage(deviceId, data, timestamp);
    const state = get(midiDeviceStore);

    // Check device settings for filtering
    const settings = state.deviceSettings[deviceId];
    if (settings) {
      if (!settings.inputEnabled) {
        return;
      }
      if (settings.channelFilter.length > 0 && !settings.channelFilter.includes(message.channel)) {
        return;
      }
      if (settings.messageFilter.length > 0 && !settings.messageFilter.includes(message.type)) {
        return;
      }
    }

    update((s) => {
      const history = [message, ...s.messageHistory].slice(0, s.historyLimit);

      // Handle MIDI learn mode
      if (s.learningMode && s.learnTarget) {
        // Learning will be handled by subscribers
      }

      return {
        ...s,
        lastMessage: message,
        messageHistory: history,
      };
    });

    // MIDI Thru - forward to outputs if enabled
    if (settings?.midiThrough) {
      const outputs = state.activeOutputs.filter((id) => id !== deviceId);
      outputs.forEach((outputId) => {
        this.sendMessage(data, outputId);
      });
    }
  },

  // ============================================================================
  // MIDI LEARN
  // ============================================================================

  /**
   * Start MIDI learn mode for a target parameter
   */
  startLearn(targetId: string): void {
    update((state) => ({
      ...state,
      learningMode: true,
      learnTarget: targetId,
    }));
  },

  /**
   * Stop MIDI learn mode
   */
  stopLearn(): void {
    update((state) => ({
      ...state,
      learningMode: false,
      learnTarget: null,
    }));
  },

  /**
   * Create mapping from learned message
   */
  createMapping(
    message: MIDIMessage,
    action: string,
    options: Partial<MIDIMapping> = {}
  ): MIDIMapping {
    const mapping: MIDIMapping = {
      id: `mapping-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      deviceId: message.deviceId,
      messageType: message.type,
      channel: message.channel,
      controller: message.controller,
      note: message.note,
      action,
      ...options,
    };

    update((state) => ({
      ...state,
      mappings: [...state.mappings, mapping],
      learningMode: false,
      learnTarget: null,
    }));

    return mapping;
  },

  /**
   * Remove a MIDI mapping
   */
  removeMapping(mappingId: string): void {
    update((state) => ({
      ...state,
      mappings: state.mappings.filter((m) => m.id !== mappingId),
    }));
  },

  /**
   * Clear all MIDI mappings
   */
  clearAllMappings(): void {
    update((state) => ({
      ...state,
      mappings: [],
    }));
  },

  /**
   * Find mappings that match a message
   */
  findMatchingMappings(message: MIDIMessage): MIDIMapping[] {
    const state = get(midiDeviceStore);
    return state.mappings.filter((m) => {
      if (m.deviceId !== message.deviceId && m.deviceId !== '*') {
        return false;
      }
      if (m.messageType !== message.type) {
        return false;
      }
      if (m.channel !== message.channel && m.channel !== 0) {
        return false;
      }
      if (m.controller !== undefined && m.controller !== message.controller) {
        return false;
      }
      if (m.note !== undefined && m.note !== message.note) {
        return false;
      }
      return true;
    });
  },

  // ============================================================================
  // DEVICE SETTINGS
  // ============================================================================

  /**
   * Update device settings
   */
  updateDeviceSettings(deviceId: string, settings: Partial<MIDIDeviceSettings>): void {
    update((state) => ({
      ...state,
      deviceSettings: {
        ...state.deviceSettings,
        [deviceId]: {
          ...(state.deviceSettings[deviceId] || defaultDeviceSettings),
          ...settings,
        },
      },
    }));
  },

  /**
   * Reset device settings to defaults
   */
  resetDeviceSettings(deviceId: string): void {
    update((state) => ({
      ...state,
      deviceSettings: {
        ...state.deviceSettings,
        [deviceId]: { ...defaultDeviceSettings },
      },
    }));
  },

  // ============================================================================
  // UTILITY
  // ============================================================================

  /**
   * Clear message history
   */
  clearHistory(): void {
    update((state) => ({
      ...state,
      messageHistory: [],
      lastMessage: null,
    }));
  },

  /**
   * Set history limit
   */
  setHistoryLimit(limit: number): void {
    update((state) => ({
      ...state,
      historyLimit: Math.max(0, limit),
      messageHistory: state.messageHistory.slice(0, limit),
    }));
  },

  /**
   * Clear error state
   */
  clearError(): void {
    update((state) => ({ ...state, error: null }));
  },

  /**
   * Reset store to initial state
   */
  reset(): void {
    set(initialState);
  },
};

// ============================================================================
// MIDI MESSAGE UTILITIES
// ============================================================================

export const midiUtils = {
  /**
   * Get note name from MIDI note number
   */
  noteName(note: number): string {
    const names = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
    const octave = Math.floor(note / 12) - 1;
    return `${names[note % 12]}${octave}`;
  },

  /**
   * Get MIDI note number from note name
   */
  noteNumber(name: string): number {
    const match = name.match(/^([A-G]#?)(-?\d+)$/);
    if (!match) {
      return -1;
    }

    const noteNames: Record<string, number> = {
      C: 0,
      'C#': 1,
      Db: 1,
      D: 2,
      'D#': 3,
      Eb: 3,
      E: 4,
      F: 5,
      'F#': 6,
      Gb: 6,
      G: 7,
      'G#': 8,
      Ab: 8,
      A: 9,
      'A#': 10,
      Bb: 10,
      B: 11,
    };

    const note = noteNames[match[1]];
    const octave = parseInt(match[2], 10);
    return (octave + 1) * 12 + note;
  },

  /**
   * Get controller name from CC number
   */
  ccName(cc: number): string {
    const names: Record<number, string> = {
      0: 'Bank Select MSB',
      1: 'Modulation',
      2: 'Breath',
      4: 'Foot',
      5: 'Portamento Time',
      6: 'Data Entry MSB',
      7: 'Volume',
      8: 'Balance',
      10: 'Pan',
      11: 'Expression',
      32: 'Bank Select LSB',
      64: 'Sustain',
      65: 'Portamento',
      66: 'Sostenuto',
      67: 'Soft Pedal',
      68: 'Legato',
      69: 'Hold 2',
      71: 'Resonance',
      72: 'Release',
      73: 'Attack',
      74: 'Cutoff',
      91: 'Reverb',
      93: 'Chorus',
      94: 'Detune',
      95: 'Phaser',
      120: 'All Sound Off',
      121: 'Reset Controllers',
      123: 'All Notes Off',
    };
    return names[cc] || `CC ${cc}`;
  },

  /**
   * Format velocity value as percentage
   */
  velocityPercent(velocity: number): string {
    return `${Math.round((velocity / 127) * 100)}%`;
  },

  /**
   * Format pitch bend value
   */
  formatPitchBend(value: number): string {
    const centered = value - 8192;
    const percent = Math.round((centered / 8191) * 100);
    return `${percent > 0 ? '+' : ''}${percent}%`;
  },
};
