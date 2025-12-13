<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { writable } from 'svelte/store';
  // midiDeviceStore available for future device integration
  import { midiDeviceStore as _midiDeviceStore } from '$lib/stores/midiDeviceStore';

  // MIDI message types
  type MidiMessageType =
    | 'note-on'
    | 'note-off'
    | 'control-change'
    | 'program-change'
    | 'pitch-bend'
    | 'aftertouch'
    | 'channel-pressure'
    | 'sysex'
    | 'clock'
    | 'start'
    | 'stop'
    | 'continue'
    | 'active-sensing'
    | 'unknown';

  interface MidiMessage {
    id: number;
    timestamp: number;
    type: MidiMessageType;
    channel: number | null;
    data1: number | null;
    data2: number | null;
    raw: number[];
    source: 'input' | 'output';
    deviceName: string;
    formatted: string;
  }

  // Filter settings
  interface FilterSettings {
    noteOn: boolean;
    noteOff: boolean;
    controlChange: boolean;
    programChange: boolean;
    pitchBend: boolean;
    aftertouch: boolean;
    channelPressure: boolean;
    sysex: boolean;
    clock: boolean;
    transport: boolean;
    activeSensing: boolean;
    channels: boolean[];
    inputs: boolean;
    outputs: boolean;
  }

  // Display settings
  interface DisplaySettings {
    showTimestamp: boolean;
    showRawBytes: boolean;
    showDeviceName: boolean;
    autoScroll: boolean;
    maxMessages: number;
    hexDisplay: boolean;
  }

  // Message store
  const messages = writable<MidiMessage[]>([]);
  let messageId = 0;

  // Filter settings with defaults
  let filters: FilterSettings = {
    noteOn: true,
    noteOff: true,
    controlChange: true,
    programChange: true,
    pitchBend: true,
    aftertouch: true,
    channelPressure: true,
    sysex: true,
    clock: false, // Often very spammy
    transport: true,
    activeSensing: false, // Very spammy
    channels: Array(16).fill(true),
  inputs: true,
    outputs: true,
  };

  // Display settings with defaults
  let displaySettings: DisplaySettings = {
    showTimestamp: true,
    showRawBytes: false,
    showDeviceName: true,
    autoScroll: true,
    maxMessages: 1000,
    hexDisplay: true,
  };

  // UI state
  let isPaused = false;
  let showFilters = false;
  let showSettings = false;
  let messageContainer: HTMLElement;
  let selectedMessageId: number | null = null;

  // Note name lookup
  const noteNames = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

  function midiNoteToName(note: number): string {
    const octave = Math.floor(note / 12) - 1;
    const noteName = noteNames[note % 12];
    return `${noteName}${octave}`;
  }

  // CC name lookup (common controllers)
  const ccNames: Record<number, string> = {
    0: 'Bank Select',
    1: 'Mod Wheel',
    2: 'Breath',
    4: 'Foot',
    5: 'Portamento Time',
    6: 'Data Entry',
    7: 'Volume',
    8: 'Balance',
    10: 'Pan',
    11: 'Expression',
    64: 'Sustain',
    65: 'Portamento',
    66: 'Sostenuto',
    67: 'Soft Pedal',
    68: 'Legato',
    69: 'Hold 2',
    91: 'Reverb',
    93: 'Chorus',
    94: 'Detune',
    95: 'Phaser',
    120: 'All Sound Off',
    121: 'Reset Controllers',
    123: 'All Notes Off',
    124: 'Omni Off',
    125: 'Omni On',
    126: 'Mono On',
    127: 'Poly On',
  };

  function getCCName(cc: number): string {
    return ccNames[cc] || `CC${cc}`;
  }

  // Format MIDI message for display
  function formatMessage(type: MidiMessageType, channel: number | null, data1: number | null, data2: number | null): string {
    const ch = channel !== null ? `Ch${channel + 1}` : '';

    switch (type) {
      case 'note-on':
        return `${ch} Note On: ${midiNoteToName(data1!)} vel=${data2}`;
      case 'note-off':
        return `${ch} Note Off: ${midiNoteToName(data1!)} vel=${data2}`;
      case 'control-change':
        return `${ch} CC: ${getCCName(data1!)} = ${data2}`;
      case 'program-change':
        return `${ch} Program: ${data1! + 1}`;
      case 'pitch-bend':
        const bendValue = ((data2! << 7) | data1!) - 8192;
        return `${ch} Pitch Bend: ${bendValue}`;
      case 'aftertouch':
        return `${ch} Poly AT: ${midiNoteToName(data1!)} = ${data2}`;
      case 'channel-pressure':
        return `${ch} Channel Pressure: ${data1}`;
      case 'sysex':
        return 'SysEx Message';
      case 'clock':
        return 'Clock';
      case 'start':
        return 'Start';
      case 'stop':
        return 'Stop';
      case 'continue':
        return 'Continue';
      case 'active-sensing':
        return 'Active Sensing';
      default:
        return 'Unknown';
    }
  }

  // Parse MIDI status byte to get message type
  function parseMessageType(status: number): { type: MidiMessageType; channel: number | null } {
    // System messages (no channel)
    if (status >= 0xf0) {
      switch (status) {
        case 0xf0:
          return { type: 'sysex', channel: null };
        case 0xf8:
          return { type: 'clock', channel: null };
        case 0xfa:
          return { type: 'start', channel: null };
        case 0xfb:
          return { type: 'continue', channel: null };
        case 0xfc:
          return { type: 'stop', channel: null };
        case 0xfe:
          return { type: 'active-sensing', channel: null };
        default:
          return { type: 'unknown', channel: null };
      }
    }

    // Channel messages
    const messageType = status & 0xf0;
    const channel = status & 0x0f;

    switch (messageType) {
      case 0x90:
        return { type: 'note-on', channel };
      case 0x80:
        return { type: 'note-off', channel };
      case 0xb0:
        return { type: 'control-change', channel };
      case 0xc0:
        return { type: 'program-change', channel };
      case 0xe0:
        return { type: 'pitch-bend', channel };
      case 0xa0:
        return { type: 'aftertouch', channel };
      case 0xd0:
        return { type: 'channel-pressure', channel };
      default:
        return { type: 'unknown', channel };
    }
  }

  // Check if message passes filters
  function passesFilter(msg: MidiMessage): boolean {
    // Source filter
    if (msg.source === 'input' && !filters.inputs) {
      return false;
    }
    if (msg.source === 'output' && !filters.outputs) {
      return false;
    }

    // Channel filter
    if (msg.channel !== null && !filters.channels[msg.channel]) {
      return false;
    }

    // Type filter
    switch (msg.type) {
      case 'note-on':
        return filters.noteOn;
      case 'note-off':
        return filters.noteOff;
      case 'control-change':
        return filters.controlChange;
      case 'program-change':
        return filters.programChange;
      case 'pitch-bend':
        return filters.pitchBend;
      case 'aftertouch':
        return filters.aftertouch;
      case 'channel-pressure':
        return filters.channelPressure;
      case 'sysex':
        return filters.sysex;
      case 'clock':
        return filters.clock;
      case 'start':
      case 'stop':
      case 'continue':
        return filters.transport;
      case 'active-sensing':
        return filters.activeSensing;
      default:
        return true;
    }
  }

  // Add a new MIDI message
  function addMessage(raw: number[], source: 'input' | 'output', deviceName: string) {
    if (isPaused) {
      return;
    }
    if (raw.length === 0) {
      return;
    }

    const { type, channel } = parseMessageType(raw[0]);
    const data1 = raw.length > 1 ? raw[1] : null;
    const data2 = raw.length > 2 ? raw[2] : null;

    // Handle note-on with velocity 0 as note-off
    const actualType = type === 'note-on' && data2 === 0 ? 'note-off' : type;

    const msg: MidiMessage = {
      id: messageId++,
      timestamp: performance.now(),
      type: actualType,
      channel,
      data1,
      data2,
      raw,
      source,
      deviceName,
      formatted: formatMessage(actualType, channel, data1, data2),
    };

    if (!passesFilter(msg)) {
      return;
    }

    messages.update((msgs) => {
      const newMsgs = [...msgs, msg];
      // Trim to max messages
      if (newMsgs.length > displaySettings.maxMessages) {
        return newMsgs.slice(-displaySettings.maxMessages);
      }
      return newMsgs;
    });

    // Auto-scroll
    if (displaySettings.autoScroll && messageContainer) {
      requestAnimationFrame(() => {
        messageContainer.scrollTop = messageContainer.scrollHeight;
      });
    }
  }

  // Format timestamp
  function formatTimestamp(ts: number): string {
    const date = new Date(ts);
    const ms = date.getMilliseconds().toString().padStart(3, '0');
    const s = date.getSeconds().toString().padStart(2, '0');
    const m = date.getMinutes().toString().padStart(2, '0');
    const h = date.getHours().toString().padStart(2, '0');
    return `${h}:${m}:${s}.${ms}`;
  }

  // Format raw bytes
  function formatRawBytes(raw: number[]): string {
    if (displaySettings.hexDisplay) {
      return raw.map((b) => b.toString(16).padStart(2, '0').toUpperCase()).join(' ');
    }
    return raw.join(' ');
  }

  // Get CSS class for message type
  function getMessageTypeClass(type: MidiMessageType): string {
    switch (type) {
      case 'note-on':
        return 'msg-note-on';
      case 'note-off':
        return 'msg-note-off';
      case 'control-change':
        return 'msg-cc';
      case 'program-change':
        return 'msg-pc';
      case 'pitch-bend':
        return 'msg-pb';
      case 'aftertouch':
      case 'channel-pressure':
        return 'msg-at';
      case 'sysex':
        return 'msg-sysex';
      case 'clock':
      case 'start':
      case 'stop':
      case 'continue':
        return 'msg-transport';
      case 'active-sensing':
        return 'msg-system';
      default:
        return '';
    }
  }

  // Clear all messages
  function clearMessages() {
    messages.set([]);
    selectedMessageId = null;
  }

  // Toggle all channels
  function toggleAllChannels(enabled: boolean) {
    filters.channels = Array(16).fill(enabled);
  }

  // Demo mode - generate sample messages for testing
  let demoInterval: ReturnType<typeof setInterval> | null = null;
  let demoMode = false;

  function toggleDemoMode() {
    demoMode = !demoMode;
    if (demoMode) {
      demoInterval = setInterval(() => {
        const types = [
          [0x90, 60, 100], // Note on
          [0x80, 60, 0], // Note off
          [0xb0, 1, 64], // CC
          [0xc0, 5], // Program change
          [0xe0, 0, 64], // Pitch bend
        ];
        const type = types[Math.floor(Math.random() * types.length)];
        const channel = Math.floor(Math.random() * 16);
        type[0] = (type[0] & 0xf0) | channel;
        addMessage(type, Math.random() > 0.5 ? 'input' : 'output', 'Demo Device');
      }, 100);
    } else if (demoInterval) {
      clearInterval(demoInterval);
      demoInterval = null;
    }
  }

  // Export messages to file
  function exportMessages() {
    let currentMessages: MidiMessage[] = [];
    messages.subscribe((m) => (currentMessages = m))();

    const lines = currentMessages.map((msg) => {
      const ts = formatTimestamp(msg.timestamp);
      const raw = formatRawBytes(msg.raw);
      return `${ts}\t${msg.source}\t${msg.deviceName}\t${msg.formatted}\t[${raw}]`;
    });

    const content = lines.join('\n');
    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `midi-log-${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  }

  // Load settings from localStorage
  function loadSettings() {
    try {
      const savedFilters = localStorage.getItem('midi-monitor-filters');
      if (savedFilters) {
        filters = { ...filters, ...JSON.parse(savedFilters) };
      }
      const savedDisplay = localStorage.getItem('midi-monitor-display');
      if (savedDisplay) {
        displaySettings = { ...displaySettings, ...JSON.parse(savedDisplay) };
      }
    } catch (e) {
      console.warn('Failed to load MIDI Monitor settings:', e);
    }
  }

  // Save settings to localStorage
  function saveSettings() {
    try {
      localStorage.setItem('midi-monitor-filters', JSON.stringify(filters));
      localStorage.setItem('midi-monitor-display', JSON.stringify(displaySettings));
    } catch (e) {
      console.warn('Failed to save MIDI Monitor settings:', e);
    }
  }

  // Reactive save
  $: if (filters || displaySettings) {
    saveSettings();
  }

  onMount(() => {
    loadSettings();
  });

  onDestroy(() => {
    if (demoInterval) {
      clearInterval(demoInterval);
    }
  });

  // Get filtered messages
  $: filteredMessages = $messages;
</script>

<div class="midi-monitor">
  <!-- Toolbar -->
  <div class="toolbar">
    <div class="toolbar-group">
      <button class="btn" class:active={!isPaused} on:click={() => (isPaused = false)} title="Record">
        <span class="icon">&#9679;</span>
      </button>
      <button class="btn" class:active={isPaused} on:click={() => (isPaused = true)} title="Pause">
        <span class="icon">&#10074;&#10074;</span>
      </button>
      <button class="btn" on:click={clearMessages} title="Clear">
        <span class="icon">&#128465;</span>
      </button>
    </div>

    <div class="toolbar-group">
      <button class="btn" class:active={showFilters} on:click={() => (showFilters = !showFilters)}>
        Filters
      </button>
      <button class="btn" class:active={showSettings} on:click={() => (showSettings = !showSettings)}>
        Settings
      </button>
    </div>

    <div class="toolbar-group">
      <button class="btn" on:click={exportMessages} title="Export Log">
        Export
      </button>
      <button class="btn" class:active={demoMode} on:click={toggleDemoMode} title="Demo Mode">
        Demo
      </button>
    </div>

    <div class="toolbar-spacer"></div>

    <div class="status">
      {#if isPaused}
        <span class="status-paused">PAUSED</span>
      {:else}
        <span class="status-recording">RECORDING</span>
      {/if}
      <span class="message-count">{$messages.length} messages</span>
    </div>
  </div>

  <!-- Filter Panel -->
  {#if showFilters}
    <div class="panel filter-panel">
      <div class="panel-section">
        <h4>Message Types</h4>
        <div class="filter-grid">
          <label><input type="checkbox" bind:checked={filters.noteOn} /> Note On</label>
          <label><input type="checkbox" bind:checked={filters.noteOff} /> Note Off</label>
          <label><input type="checkbox" bind:checked={filters.controlChange} /> Control Change</label>
          <label><input type="checkbox" bind:checked={filters.programChange} /> Program Change</label>
          <label><input type="checkbox" bind:checked={filters.pitchBend} /> Pitch Bend</label>
          <label><input type="checkbox" bind:checked={filters.aftertouch} /> Aftertouch</label>
          <label><input type="checkbox" bind:checked={filters.channelPressure} /> Channel Pressure</label>
          <label><input type="checkbox" bind:checked={filters.sysex} /> SysEx</label>
          <label><input type="checkbox" bind:checked={filters.clock} /> Clock</label>
          <label><input type="checkbox" bind:checked={filters.transport} /> Transport</label>
          <label><input type="checkbox" bind:checked={filters.activeSensing} /> Active Sensing</label>
        </div>
      </div>

      <div class="panel-section">
        <h4>
          Channels
          <button class="btn-small" on:click={() => toggleAllChannels(true)}>All</button>
          <button class="btn-small" on:click={() => toggleAllChannels(false)}>None</button>
        </h4>
        <div class="channel-grid">
          {#each Array(16) as _, i (i)}
            <label class="channel-checkbox">
              <input type="checkbox" bind:checked={filters.channels[i]} />
              {i + 1}
            </label>
          {/each}
        </div>
      </div>

      <div class="panel-section">
        <h4>Source</h4>
        <div class="filter-row">
          <label><input type="checkbox" bind:checked={filters.inputs} /> Inputs</label>
          <label><input type="checkbox" bind:checked={filters.outputs} /> Outputs</label>
        </div>
      </div>
    </div>
  {/if}

  <!-- Settings Panel -->
  {#if showSettings}
    <div class="panel settings-panel">
      <div class="panel-section">
        <h4>Display Options</h4>
        <div class="settings-grid">
          <label><input type="checkbox" bind:checked={displaySettings.showTimestamp} /> Show Timestamp</label>
          <label><input type="checkbox" bind:checked={displaySettings.showRawBytes} /> Show Raw Bytes</label>
          <label><input type="checkbox" bind:checked={displaySettings.showDeviceName} /> Show Device Name</label>
          <label><input type="checkbox" bind:checked={displaySettings.autoScroll} /> Auto Scroll</label>
          <label><input type="checkbox" bind:checked={displaySettings.hexDisplay} /> Hex Display</label>
        </div>
      </div>

      <div class="panel-section">
        <h4>Buffer Size</h4>
        <div class="settings-row">
          <input
            type="range"
            min="100"
            max="10000"
            step="100"
            bind:value={displaySettings.maxMessages}
          />
          <span>{displaySettings.maxMessages} messages</span>
        </div>
      </div>
    </div>
  {/if}

  <!-- Message List -->
  <div class="message-list" bind:this={messageContainer}>
    {#if filteredMessages.length === 0}
      <div class="empty-state">
        <p>No MIDI messages received yet.</p>
        <p class="hint">Connect a MIDI device or enable Demo mode to see messages.</p>
      </div>
    {:else}
      {#each filteredMessages as msg (msg.id)}
        <div
          class="message-row {getMessageTypeClass(msg.type)}"
          class:selected={selectedMessageId === msg.id}
          on:click={() => (selectedMessageId = msg.id)}
          on:keydown={(e) => e.key === 'Enter' && (selectedMessageId = msg.id)}
          role="button"
          tabindex="0"
        >
          {#if displaySettings.showTimestamp}
            <span class="col-timestamp">{formatTimestamp(msg.timestamp)}</span>
          {/if}
          <span class="col-source" class:input={msg.source === 'input'} class:output={msg.source === 'output'}>
            {msg.source === 'input' ? 'IN' : 'OUT'}
          </span>
          {#if displaySettings.showDeviceName}
            <span class="col-device">{msg.deviceName}</span>
          {/if}
          <span class="col-message">{msg.formatted}</span>
          {#if displaySettings.showRawBytes}
            <span class="col-raw">[{formatRawBytes(msg.raw)}]</span>
          {/if}
        </div>
      {/each}
    {/if}
  </div>

  <!-- Detail Panel (shown when message selected) -->
  {#if selectedMessageId !== null}
    {@const selectedMsg = filteredMessages.find((m) => m.id === selectedMessageId)}
    {#if selectedMsg}
      <div class="detail-panel">
        <h4>Message Details</h4>
        <div class="detail-grid">
          <span class="detail-label">Type:</span>
          <span class="detail-value">{selectedMsg.type}</span>
          <span class="detail-label">Channel:</span>
          <span class="detail-value">{selectedMsg.channel !== null ? selectedMsg.channel + 1 : 'N/A'}</span>
          <span class="detail-label">Data 1:</span>
          <span class="detail-value">{selectedMsg.data1 ?? 'N/A'}</span>
          <span class="detail-label">Data 2:</span>
          <span class="detail-value">{selectedMsg.data2 ?? 'N/A'}</span>
          <span class="detail-label">Raw (Hex):</span>
          <span class="detail-value">{selectedMsg.raw.map((b) => b.toString(16).padStart(2, '0').toUpperCase()).join(' ')}</span>
          <span class="detail-label">Raw (Dec):</span>
          <span class="detail-value">{selectedMsg.raw.join(' ')}</span>
          <span class="detail-label">Source:</span>
          <span class="detail-value">{selectedMsg.source}</span>
          <span class="detail-label">Device:</span>
          <span class="detail-value">{selectedMsg.deviceName}</span>
        </div>
        <button class="btn btn-close-detail" on:click={() => (selectedMessageId = null)}>Close</button>
      </div>
    {/if}
  {/if}
</div>

<style>
  .midi-monitor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: var(--bg-primary, #1a1a1a);
    color: var(--text-primary, #e0e0e0);
    font-family: 'Consolas', 'Monaco', monospace;
    font-size: 12px;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background-color: var(--bg-secondary, #252525);
    border-bottom: 1px solid var(--border-color, #333);
  }

  .toolbar-group {
    display: flex;
    gap: 4px;
  }

  .toolbar-spacer {
    flex: 1;
  }

  .btn {
    padding: 6px 12px;
    background-color: var(--btn-bg, #333);
    border: 1px solid var(--border-color, #444);
    border-radius: 4px;
    color: var(--text-primary, #e0e0e0);
    cursor: pointer;
    font-size: 11px;
    transition: all 0.15s;
  }

  .btn:hover {
    background-color: var(--btn-hover, #3a3a3a);
  }

  .btn.active {
    background-color: var(--accent, #0066cc);
    border-color: var(--accent, #0066cc);
  }

  .btn .icon {
    font-size: 10px;
  }

  .btn-small {
    padding: 2px 6px;
    font-size: 10px;
    background-color: var(--btn-bg, #333);
    border: 1px solid var(--border-color, #444);
    border-radius: 3px;
    color: var(--text-secondary, #888);
    cursor: pointer;
    margin-left: 8px;
  }

  .btn-small:hover {
    background-color: var(--btn-hover, #3a3a3a);
  }

  .status {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 11px;
  }

  .status-recording {
    color: #ff4444;
    font-weight: bold;
  }

  .status-paused {
    color: #ffaa00;
    font-weight: bold;
  }

  .message-count {
    color: var(--text-secondary, #888);
  }

  .panel {
    padding: 12px;
    background-color: var(--bg-secondary, #252525);
    border-bottom: 1px solid var(--border-color, #333);
  }

  .panel-section {
    margin-bottom: 12px;
  }

  .panel-section:last-child {
    margin-bottom: 0;
  }

  .panel-section h4 {
    margin: 0 0 8px 0;
    font-size: 11px;
    color: var(--text-secondary, #888);
    text-transform: uppercase;
  }

  .filter-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 6px;
  }

  .filter-grid label {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
  }

  .filter-row {
    display: flex;
    gap: 16px;
  }

  .filter-row label {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
  }

  .channel-grid {
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    gap: 4px;
  }

  .channel-checkbox {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 2px;
    padding: 4px;
    background-color: var(--bg-tertiary, #2a2a2a);
    border-radius: 3px;
    font-size: 10px;
  }

  .settings-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
  }

  .settings-grid label {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
  }

  .settings-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .settings-row input[type='range'] {
    flex: 1;
  }

  .settings-row span {
    font-size: 11px;
    color: var(--text-secondary, #888);
    min-width: 100px;
  }

  .message-list {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .message-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 12px;
    border-bottom: 1px solid var(--border-color, #222);
    cursor: pointer;
    transition: background-color 0.1s;
  }

  .message-row:hover {
    background-color: var(--bg-hover, #2a2a2a);
  }

  .message-row.selected {
    background-color: var(--bg-selected, #333);
  }

  .col-timestamp {
    color: var(--text-secondary, #666);
    font-size: 10px;
    min-width: 80px;
  }

  .col-source {
    font-size: 10px;
    font-weight: bold;
    padding: 2px 6px;
    border-radius: 3px;
    min-width: 30px;
    text-align: center;
  }

  .col-source.input {
    background-color: #004400;
    color: #00cc00;
  }

  .col-source.output {
    background-color: #440044;
    color: #cc00cc;
  }

  .col-device {
    color: var(--text-secondary, #888);
    font-size: 10px;
    min-width: 100px;
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .col-message {
    flex: 1;
    color: var(--text-primary, #e0e0e0);
  }

  .col-raw {
    color: var(--text-secondary, #666);
    font-size: 10px;
  }

  /* Message type colors */
  .msg-note-on .col-message {
    color: #44ff44;
  }

  .msg-note-off .col-message {
    color: #88aa88;
  }

  .msg-cc .col-message {
    color: #ffaa44;
  }

  .msg-pc .col-message {
    color: #ff44ff;
  }

  .msg-pb .col-message {
    color: #44aaff;
  }

  .msg-at .col-message {
    color: #aaaa44;
  }

  .msg-sysex .col-message {
    color: #ff8888;
  }

  .msg-transport .col-message {
    color: #88ffff;
  }

  .msg-system .col-message {
    color: #888888;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary, #666);
  }

  .empty-state p {
    margin: 4px 0;
  }

  .empty-state .hint {
    font-size: 11px;
  }

  .detail-panel {
    padding: 12px;
    background-color: var(--bg-secondary, #252525);
    border-top: 1px solid var(--border-color, #333);
  }

  .detail-panel h4 {
    margin: 0 0 8px 0;
    font-size: 12px;
    color: var(--text-primary, #e0e0e0);
  }

  .detail-grid {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 4px 12px;
    font-size: 11px;
    margin-bottom: 8px;
  }

  .detail-label {
    color: var(--text-secondary, #888);
  }

  .detail-value {
    color: var(--text-primary, #e0e0e0);
    font-family: 'Consolas', 'Monaco', monospace;
  }

  .btn-close-detail {
    padding: 4px 12px;
    font-size: 10px;
  }

  /* Scrollbar styling */
  .message-list::-webkit-scrollbar {
    width: 8px;
  }

  .message-list::-webkit-scrollbar-track {
    background: var(--bg-primary, #1a1a1a);
  }

  .message-list::-webkit-scrollbar-thumb {
    background: var(--border-color, #444);
    border-radius: 4px;
  }

  .message-list::-webkit-scrollbar-thumb:hover {
    background: var(--text-secondary, #666);
  }
</style>
