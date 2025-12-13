<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { message, open } from '@tauri-apps/plugin-dialog';

  interface Theme {
    id: string;
    name: string;
    color: string;
  }

  interface AudioDevice {
    id: string;
    name: string;
  }

  // MidiDevice for UI - extends API fields with UI-friendly properties
  interface MidiDevice {
    id: string; // UI key (derived from name or port_number)
    name: string; // From API
    port_number: number; // From API
    is_available: boolean; // From API
    manufacturer?: string; // Optional UI field
  }

  interface Settings {
    // General
    language: string;
    defaultBpm: number;
    timeSignatureNumerator: number;
    timeSignatureDenominator: number;
    autoSave: boolean;
    autoSaveInterval: number;
    startupLoadLastProject: boolean;
    showWelcomeScreen: boolean;
    checkForUpdates: boolean;
    maxUndoHistory: number;

    // Audio (required - initialized with defaults)
    // Core fields align with AudioSettings from settingsStore.ts
    // Additional UI fields for extended configuration
    audio: {
      driverType: 'asio' | 'wasapi' | 'alsa' | 'coreaudio';
      driver: string; // UI-friendly driver selector (maps to driverType for API)
      outputDevice: string;
      inputDevice: string;
      sampleRate: number;
      bufferSize: number;
      inputChannels: number;
      outputChannels: number;
      channels: number; // UI-friendly channel count (maps to outputChannels for API)
    };

    // MIDI (required - initialized with defaults)
    // Core fields align with MIDISettings from settingsStore.ts
    // Additional UI fields for extended per-device configuration
    midi: {
      defaultInputDevice: string;
      defaultOutputDevice: string;
      midiThrough: boolean;
      recordVelocityCurve: 'linear' | 'soft' | 'hard';
      sendClock: boolean;
      receiveClock: boolean;
      syncMode: 'internal' | 'external' | 'auto';
      // UI-only fields for extended configuration
      inputDevices: Record<string, { enabled: boolean; channel: number }>;
      outputDevices: Record<string, { enabled: boolean; channel: number }>;
      clockSource: string; // Maps to syncMode for API
      thruMode: string; // Maps to midiThrough for API
      velocityCurve: string; // Maps to recordVelocityCurve for API
    };

    // Appearance
    theme: string;
    uiScale: number;
    alwaysOnTop: boolean;
    useNativeTitlebar: boolean;
    enableTransparency: boolean;
    autoHideMenuBar: boolean;
    enableAnimations: boolean;
    animationSpeed: string;
    customCSS: string;

    // Paths
    databasePath: string;
    audioPath: string;
    midiPath: string;
    projectsPath: string;
    tempPath: string;
    backupPath: string;

    // Advanced
    renderThreads: number;
    audioThreadPriority: string;
    cacheSize: number;

    // Shortcuts
    shortcutCategory: string;
    shortcuts: Record<string, Shortcut>;
  }

  interface Shortcut {
    id: string;
    description: string;
    key: string;
    alternative: string;
    category: string;
  }

  // State using Svelte 4 let bindings
  let activeTab = 'general';
  let settings: Settings = {
    language: 'en',
    defaultBpm: 120,
    timeSignatureNumerator: 4,
    timeSignatureDenominator: 4,
    autoSave: true,
    autoSaveInterval: 300,
    startupLoadLastProject: false,
    showWelcomeScreen: true,
    checkForUpdates: true,
    maxUndoHistory: 100,
    audio: {
      driverType: 'wasapi',
      driver: 'WASAPI', // UI-friendly name
      outputDevice: '',
      inputDevice: '',
      sampleRate: 44100,
      bufferSize: 512,
      inputChannels: 2,
      outputChannels: 2,
      channels: 2, // UI-friendly channel count
    },
    midi: {
      defaultInputDevice: '',
      defaultOutputDevice: '',
      midiThrough: true,
      recordVelocityCurve: 'linear',
      sendClock: false,
      receiveClock: false,
      syncMode: 'internal',
      // UI-only extended configuration
      inputDevices: {},
      outputDevices: {},
      clockSource: 'internal',
      thruMode: 'on',
      velocityCurve: 'linear',
    },
    theme: 'dark',
    uiScale: 1.0,
    alwaysOnTop: false,
    useNativeTitlebar: false,
    enableTransparency: false,
    autoHideMenuBar: false,
    enableAnimations: true,
    animationSpeed: 'normal',
    customCSS: '',
    databasePath: '',
    audioPath: '',
    midiPath: '',
    projectsPath: '',
    tempPath: '',
    backupPath: '',
    renderThreads: 4,
    audioThreadPriority: 'normal',
    cacheSize: 512,
    shortcutCategory: 'global',
    shortcuts: {},
  };
  let isLoading = true;
  let isSaving = false;
  let audioDevices: { inputs: AudioDevice[]; outputs: AudioDevice[] } = { inputs: [], outputs: [] };
  let midiDevices: MidiDevice[] = [];
  const themes: Theme[] = [
    { id: 'dark', name: 'Dark', color: '#1f2937' },
    { id: 'light', name: 'Light', color: '#f3f4f6' },
    { id: 'blue', name: 'Blue', color: '#1e40af' },
    { id: 'purple', name: 'Purple', color: '#7c3aed' },
  ];

  const tabs = [
    { id: 'general', name: 'General', icon: '⚙️' },
    { id: 'audio', name: 'Audio', icon: '🔊' },
    { id: 'midi', name: 'MIDI', icon: '🎹' },
    { id: 'appearance', name: 'Appearance', icon: '🎨' },
    { id: 'shortcuts', name: 'Shortcuts', icon: '⌨️' },
    { id: 'paths', name: 'Paths', icon: '📁' },
    { id: 'advanced', name: 'Advanced', icon: '⚡' },
  ];

  const defaultShortcuts: Shortcut[] = [
    { id: 'play', description: 'Play/Pause', key: 'Space', alternative: '', category: 'transport' },
    { id: 'stop', description: 'Stop', key: 'Enter', alternative: '', category: 'transport' },
    { id: 'record', description: 'Record', key: 'R', alternative: 'Ctrl+R', category: 'transport' },
    { id: 'undo', description: 'Undo', key: 'Ctrl+Z', alternative: '', category: 'editing' },
    {
      id: 'redo',
      description: 'Redo',
      key: 'Ctrl+Shift+Z',
      alternative: 'Ctrl+Y',
      category: 'editing',
    },
    { id: 'copy', description: 'Copy', key: 'Ctrl+C', alternative: '', category: 'editing' },
    { id: 'paste', description: 'Paste', key: 'Ctrl+V', alternative: '', category: 'editing' },
    { id: 'save', description: 'Save', key: 'Ctrl+S', alternative: '', category: 'global' },
    { id: 'open', description: 'Open', key: 'Ctrl+O', alternative: '', category: 'global' },
    { id: 'new', description: 'New', key: 'Ctrl+N', alternative: '', category: 'global' },
  ];

  onMount(async () => {
    await loadAllSettings();
  });

  async function loadAllSettings() {
    isLoading = true;
    try {
      const audioSettings = await api.settings.getAudioSettings();
      const devices = await api.audio.getDevices();
      const midi = await api.midi.listDevices();

      if (audioSettings) {
        // Merge API settings with UI-only fields
        settings = {
          ...settings,
          audio: {
            ...audioSettings,
            driver: audioSettings.driverType?.toUpperCase() || 'WASAPI',
            channels: audioSettings.outputChannels || 2,
          },
        };
      }
      audioDevices = devices || { inputs: [], outputs: [] };
      // Map API MidiDevice to UI MidiDevice (add id for keying)
      midiDevices = (midi || []).map((device) => ({
        ...device,
        id: device.name || String(device.port_number),
      }));

      // Load other settings from localStorage
      settings.theme = localStorage.getItem('theme') || 'dark';
      settings.language = localStorage.getItem('language') || 'en';
      settings.autoSave = localStorage.getItem('autoSave') !== 'false';
      settings.autoSaveInterval = parseInt(localStorage.getItem('autoSaveInterval') || '300');
      settings.uiScale = parseFloat(localStorage.getItem('uiScale') || '1.0');

      // Initialize shortcuts
      settings.shortcuts = {};
      defaultShortcuts.forEach((s) => {
        settings.shortcuts[s.id] = { ...s };
      });
    } catch (error) {
      console.error('Failed to load settings:', error);
    } finally {
      isLoading = false;
    }
  }

  async function saveSettings() {
    isSaving = true;
    try {
      // Save audio settings
      if (settings.audio) {
        await api.settings.setAudioSettings(settings.audio);
      }

      // Save MIDI settings
      if (settings.midi) {
        await api.settings.setMidiSettings(settings.midi);
      }

      // Save application settings to localStorage
      localStorage.setItem('theme', settings.theme);
      localStorage.setItem('language', settings.language);
      localStorage.setItem('autoSave', settings.autoSave.toString());
      localStorage.setItem('autoSaveInterval', settings.autoSaveInterval.toString());
      localStorage.setItem('uiScale', settings.uiScale.toString());

      // Apply theme
      document.documentElement.setAttribute('data-theme', settings.theme);

      await message('Settings saved successfully', { title: 'Success', kind: 'info' });
    } catch (error) {
      console.error('Failed to save settings:', error);
      await message(`Failed to save settings: ${error}`, { title: 'Error', kind: 'error' });
    } finally {
      isSaving = false;
    }
  }

  async function resetSettings() {
    if (!confirm('Reset all settings to defaults?')) {
      return;
    }

    try {
      await api.settings.resetSettings();
      await loadAllSettings();
      await message('Settings reset to defaults', { title: 'Success', kind: 'info' });
    } catch (error) {
      console.error('Failed to reset settings:', error);
    }
  }

  async function selectDatabasePath() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Database Location',
    });

    if (selected) {
      settings.databasePath = selected;
    }
  }

  async function selectPath(pathType: keyof Settings) {
    const selected = await open({
      directory: true,
      multiple: false,
      title: `Select ${pathType} Location`,
    });

    if (selected) {
      (settings as any)[pathType] = selected;
    }
  }

  async function testAudio() {
    try {
      await api.audio.testOutput();
      await message('Audio test successful', { title: 'Success', kind: 'info' });
    } catch (error) {
      await message(`Audio test failed: ${error}`, { title: 'Error', kind: 'error' });
    }
  }

  async function testMidi() {
    try {
      // Send middle C (note 60) on channel 1 with velocity 100
      await api.midi.sendTestNote(1, 60, 100);
      await message('MIDI test note sent', { title: 'Success', kind: 'info' });
    } catch (error) {
      await message(`MIDI test failed: ${error}`, { title: 'Error', kind: 'error' });
    }
  }

  function exportSettings() {
    const data = {
      version: '1.0',
      exported_at: new Date().toISOString(),
      settings,
    };

    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `midi_center_settings_${new Date().toISOString().split('T')[0]}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

  function importSettings(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) {
      return;
    }

    const reader = new FileReader();
    reader.onload = async (e) => {
      try {
        const imported = JSON.parse(e.target?.result as string);
        if (imported.version === '1.0') {
          settings = { ...settings, ...imported.settings };
          await message('Settings imported. Click Save to apply.', {
            title: 'Success',
            kind: 'info',
          });
        } else {
          throw new Error('Invalid settings file version');
        }
      } catch (error) {
        await message(`Failed to import settings: ${error}`, { title: 'Error', kind: 'error' });
      }
    };
    reader.readAsText(file);
  }

  function handleTabClick(tabId: string) {
    activeTab = tabId;
  }

  function getShortcutsForCategory(category: string): Shortcut[] {
    return defaultShortcuts.filter((s) => s.category === category);
  }

  function handleFileInputClick() {
    const input = document.querySelector('input[type="file"]') as HTMLInputElement;
    if (input) {
      input.click();
    }
  }

  function handleMidiDeviceToggle(deviceId: string, enabled: boolean, type: 'input' | 'output') {
    settings.midi = settings.midi || {
      inputDevices: {},
      outputDevices: {},
      sendClock: false,
      receiveClock: false,
      clockSource: 'internal',
      thruMode: 'off',
      velocityCurve: 'linear',
    };

    const devices = type === 'input' ? settings.midi.inputDevices : settings.midi.outputDevices;
    devices[deviceId] = {
      ...devices[deviceId],
      enabled,
      channel: devices[deviceId]?.channel || 0,
    };
  }

  // Reactive statements
  $: maxThreads = typeof navigator !== 'undefined' ? navigator.hardwareConcurrency || 8 : 8;
</script>

<div class="settings-window dark:bg-window dark:text-app-text h-full flex">
  <!-- Sidebar -->
  <div class="sidebar w-64 border-r dark:border-window-border flex flex-col">
    <div class="p-6 border-b dark:border-window-border">
      <h2 class="text-2xl dark:text-gray-200 mb-2">Settings</h2>
      <p class="text-sm dark:text-gray-400">Configure your MIDI Center</p>
    </div>

    <nav class="flex-1 p-4">
      <ul class="space-y-1">
        {#each tabs as tab (tab.id)}
          <li>
            <button
              on:click={() => handleTabClick(tab.id)}
              class="w-full text-left px-4 py-3 rounded-lg flex items-center gap-3 transition-colors"
              class:dark:bg-primary={activeTab === tab.id}
              class:dark:bg-transparent={activeTab !== tab.id}
              class:dark:text-white={activeTab === tab.id}
              class:dark:text-gray-300={activeTab !== tab.id}
              class:hover:dark:bg-window-subtle={activeTab !== tab.id}
            >
              <span class="text-lg">{tab.icon}</span>
              <span>{tab.name}</span>
            </button>
          </li>
        {/each}
      </ul>
    </nav>

    <div class="p-4 border-t dark:border-window-border">
      <button
        on:click={resetSettings}
        class="w-full px-4 py-3 dark:bg-secondary rounded-lg hover:opacity-80 mb-2"
      >
        Reset to Defaults
      </button>
      <button
        on:click={saveSettings}
        disabled={isSaving}
        class="w-full px-4 py-3 dark:bg-primary dark:text-white rounded-lg hover:opacity-80 disabled:opacity-50"
      >
        {isSaving ? 'Saving...' : 'Save Settings'}
      </button>
    </div>
  </div>

  <!-- Main Content -->
  <div class="main-content flex-1 overflow-auto">
    {#if isLoading}
      <div class="loading h-full flex items-center justify-center dark:text-gray-500">
        <div class="text-center">
          <div
            class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 dark:border-gray-400 mb-4"
          ></div>
          <div>Loading settings...</div>
        </div>
      </div>
    {:else}
      <div class="p-6">
        <!-- General Settings -->
        {#if activeTab === 'general'}
          <div class="space-y-6">
            <h3 class="text-xl dark:text-gray-200 mb-4">General Settings</h3>

            <div class="grid grid-cols-2 gap-6">
              <div class="space-y-4">
                <div>
                  <label for="language" class="block text-sm dark:text-gray-400 mb-2"
                    >Language</label
                  >
                  <select
                    id="language"
                    bind:value={settings.language}
                    class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                  >
                    <option value="en">English</option>
                    <option value="es">Español</option>
                    <option value="fr">Français</option>
                    <option value="de">Deutsch</option>
                    <option value="ja">日本語</option>
                    <option value="zh">中文</option>
                  </select>
                </div>

                <div>
                  <label for="defaultBpm" class="block text-sm dark:text-gray-400 mb-2"
                    >Default BPM</label
                  >
                  <input
                    id="defaultBpm"
                    type="number"
                    bind:value={settings.defaultBpm}
                    min="30"
                    max="300"
                    step="1"
                    class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                  />
                </div>

                <div>
                  <label class="block text-sm dark:text-gray-400 mb-2">Time Signature</label>
                  <div class="flex gap-2">
                    <select
                      bind:value={settings.timeSignatureNumerator}
                      class="flex-1 px-3 py-2 dark:bg-input dark:border-window-border rounded"
                    >
                      {#each [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12] as num (num)}
                        <option value={num}>{num}</option>
                      {/each}
                    </select>
                    <span class="flex items-center dark:text-gray-300">/</span>
                    <select
                      bind:value={settings.timeSignatureDenominator}
                      class="flex-1 px-3 py-2 dark:bg-input dark:border-window-border rounded"
                    >
                      <option value={4}>4</option>
                      <option value={8}>8</option>
                      <option value={16}>16</option>
                    </select>
                  </div>
                </div>
              </div>

              <div class="space-y-4">
                <div class="space-y-2">
                  <label class="flex items-center gap-2">
                    <input type="checkbox" bind:checked={settings.autoSave} class="rounded" />
                    <span class="text-sm dark:text-gray-400">Enable Auto-save</span>
                  </label>

                  {#if settings.autoSave}
                    <div class="ml-6">
                      <label for="autoSaveInterval" class="block text-sm dark:text-gray-400 mb-1"
                        >Auto-save Interval (seconds)</label
                      >
                      <input
                        id="autoSaveInterval"
                        type="number"
                        bind:value={settings.autoSaveInterval}
                        min="30"
                        max="3600"
                        step="30"
                        class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                      />
                    </div>
                  {/if}
                </div>

                <div class="space-y-2">
                  <label class="flex items-center gap-2">
                    <input
                      type="checkbox"
                      bind:checked={settings.startupLoadLastProject}
                      class="rounded"
                    />
                    <span class="text-sm dark:text-gray-400">Load last project on startup</span>
                  </label>

                  <label class="flex items-center gap-2">
                    <input
                      type="checkbox"
                      bind:checked={settings.showWelcomeScreen}
                      class="rounded"
                    />
                    <span class="text-sm dark:text-gray-400">Show welcome screen on startup</span>
                  </label>

                  <label class="flex items-center gap-2">
                    <input
                      type="checkbox"
                      bind:checked={settings.checkForUpdates}
                      class="rounded"
                    />
                    <span class="text-sm dark:text-gray-400">Check for updates automatically</span>
                  </label>
                </div>

                <div>
                  <label class="block text-sm dark:text-gray-400 mb-2">Maximum Undo History</label>
                  <input
                    type="range"
                    bind:value={settings.maxUndoHistory}
                    min="10"
                    max="1000"
                    step="10"
                    class="w-full"
                  />
                  <div class="text-xs dark:text-gray-500 text-center">
                    {settings.maxUndoHistory} steps
                  </div>
                </div>
              </div>
            </div>

            <!-- Import/Export -->
            <div class="border-t dark:border-window-border pt-6">
              <h4 class="text-lg dark:text-gray-200 mb-4">Import/Export Settings</h4>
              <div class="flex gap-4">
                <div class="flex-1">
                  <label class="block text-sm dark:text-gray-400 mb-2">Import Settings</label>
                  <div class="flex gap-2">
                    <input
                      type="file"
                      accept=".json"
                      on:change={importSettings}
                      class="flex-1 px-3 py-2 dark:bg-input dark:border-window-border rounded"
                    />
                    <button
                      on:click={handleFileInputClick}
                      class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
                    >
                      Browse
                    </button>
                  </div>
                </div>

                <div class="flex-1">
                  <label class="block text-sm dark:text-gray-400 mb-2">Export Settings</label>
                  <button
                    on:click={exportSettings}
                    class="w-full px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
                  >
                    Export to JSON
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Audio Settings -->
        {:else if activeTab === 'audio'}
          <div class="space-y-6">
            <h3 class="text-xl dark:text-gray-200 mb-4">Audio Settings</h3>

            <div class="grid grid-cols-2 gap-6">
              <div class="space-y-4">
                <div>
                  <label for="audioDriver" class="block text-sm dark:text-gray-400 mb-2"
                    >Audio Driver</label
                  >
                  <select
                    id="audioDriver"
                    bind:value={settings.audio.driver}
                    class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                  >
                    <option value="default">System Default</option>
                    <option value="asio">ASIO</option>
                    <option value="wasapi">WASAPI</option>
                    <option value="coreaudio">CoreAudio</option>
                    <option value="alsa">ALSA</option>
                    <option value="pulse">PulseAudio</option>
                    <option value="jack">JACK</option>
                  </select>
                </div>

                <div>
                  <label for="outputDevice" class="block text-sm dark:text-gray-400 mb-2"
                    >Output Device</label
                  >
                  <select
                    id="outputDevice"
                    bind:value={settings.audio.outputDevice}
                    class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                  >
                    <option value="">Default Output</option>
                    {#each audioDevices.outputs || [] as device (device.id)}
                      <option value={device.id}>{device.name}</option>
                    {/each}
                  </select>
                </div>

                <div>
                  <label for="inputDevice" class="block text-sm dark:text-gray-400 mb-2"
                    >Input Device</label
                  >
                  <select
                    id="inputDevice"
                    bind:value={settings.audio.inputDevice}
                    class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                  >
                    <option value="">No Input</option>
                    {#each audioDevices.inputs || [] as device (device.id)}
                      <option value={device.id}>{device.name}</option>
                    {/each}
                  </select>
                </div>
              </div>

              <div class="space-y-4">
                <div>
                  <label for="sampleRate" class="block text-sm dark:text-gray-400 mb-2"
                    >Sample Rate</label
                  >
                  <select
                    id="sampleRate"
                    bind:value={settings.audio.sampleRate}
                    class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                  >
                    <option value={44100}>44.1 kHz</option>
                    <option value={48000}>48 kHz</option>
                    <option value={88200}>88.2 kHz</option>
                    <option value={96000}>96 kHz</option>
                    <option value={176400}>176.4 kHz</option>
                    <option value={192000}>192 kHz</option>
                  </select>
                </div>

                <div>
                  <label for="bufferSize" class="block text-sm dark:text-gray-400 mb-2"
                    >Buffer Size</label
                  >
                  <select
                    id="bufferSize"
                    bind:value={settings.audio.bufferSize}
                    class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                  >
                    <option value={64}>64 samples (1.3ms @ 48kHz)</option>
                    <option value={128}>128 samples (2.6ms @ 48kHz)</option>
                    <option value={256}>256 samples (5.3ms @ 48kHz)</option>
                    <option value={512}>512 samples (10.6ms @ 48kHz)</option>
                    <option value={1024}>1024 samples (21.3ms @ 48kHz)</option>
                    <option value={2048}>2048 samples (42.6ms @ 48kHz)</option>
                  </select>
                  <p class="text-xs dark:text-gray-500 mt-1">
                    Lower = less latency, but higher CPU usage
                  </p>
                </div>

                <div>
                  <label for="channels" class="block text-sm dark:text-gray-400 mb-2"
                    >Number of Channels</label
                  >
                  <select
                    id="channels"
                    bind:value={settings.audio.channels}
                    class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                  >
                    <option value={1}>Mono</option>
                    <option value={2}>Stereo</option>
                    <option value={4}>Quad</option>
                    <option value={6}>5.1 Surround</option>
                    <option value={8}>7.1 Surround</option>
                  </select>
                </div>
              </div>
            </div>

            <div class="border-t dark:border-window-border pt-6">
              <div class="flex justify-between items-center">
                <div>
                  <h4 class="text-lg dark:text-gray-200 mb-2">Audio Test</h4>
                  <p class="text-sm dark:text-gray-400">Test your audio configuration</p>
                </div>
                <button
                  on:click={testAudio}
                  class="px-6 py-2 dark:bg-primary dark:text-white rounded hover:opacity-80"
                >
                  Test Audio
                </button>
              </div>
            </div>
          </div>

          <!-- MIDI Settings -->
        {:else if activeTab === 'midi'}
          <div class="space-y-6">
            <h3 class="text-xl dark:text-gray-200 mb-4">MIDI Settings</h3>

            <div class="space-y-4">
              <!-- MIDI Input Devices -->
              <div>
                <h4 class="text-lg dark:text-gray-300 mb-3">MIDI Input Devices</h4>
                <div class="space-y-2">
                  {#each midiDevices as device (device.id)}
                    <div
                      class="flex items-center justify-between p-3 dark:bg-window-subtle rounded border dark:border-window-border"
                    >
                      <div class="flex items-center gap-3">
                        <input
                          type="checkbox"
                          checked={settings.midi?.inputDevices?.[device.id]?.enabled || false}
                          on:change={(e) =>
                            handleMidiDeviceToggle(device.id, e.currentTarget.checked, 'input')}
                          class="rounded"
                        />
                        <div>
                          <div class="font-medium dark:text-gray-200">{device.name}</div>
                          {#if device.manufacturer}
                            <div class="text-xs dark:text-gray-400">{device.manufacturer}</div>
                          {/if}
                        </div>
                      </div>

                      {#if settings.midi?.inputDevices?.[device.id]?.enabled}
                        <div class="flex items-center gap-3">
                          <select
                            bind:value={settings.midi.inputDevices[device.id].channel}
                            class="px-2 py-1 text-sm dark:bg-input dark:border-window-border rounded"
                          >
                            <option value={0}>All Channels</option>
                            {#each Array.from({ length: 16 }, (_, i) => i + 1) as channel (channel)}
                              <option value={channel}>Ch {channel}</option>
                            {/each}
                          </select>
                          <button
                            on:click={testMidi}
                            class="px-3 py-1 text-sm dark:bg-secondary rounded hover:opacity-80"
                          >
                            Test
                          </button>
                        </div>
                      {/if}
                    </div>
                  {/each}
                  {#if midiDevices.length === 0}
                    <div class="text-center py-4 dark:text-gray-500">No MIDI devices found</div>
                  {/if}
                </div>
              </div>

              <!-- MIDI Clock -->
              <div class="border-t dark:border-window-border pt-6">
                <h4 class="text-lg dark:text-gray-300 mb-3">MIDI Clock & Sync</h4>
                <div class="grid grid-cols-2 gap-6">
                  <div class="space-y-4">
                    <label class="flex items-center gap-2">
                      <input
                        type="checkbox"
                        bind:checked={settings.midi.sendClock}
                        class="rounded"
                      />
                      <span class="text-sm dark:text-gray-400">Send MIDI Clock</span>
                    </label>

                    <label class="flex items-center gap-2">
                      <input
                        type="checkbox"
                        bind:checked={settings.midi.receiveClock}
                        class="rounded"
                      />
                      <span class="text-sm dark:text-gray-400">Receive MIDI Clock</span>
                    </label>

                    <div>
                      <label for="clockSource" class="block text-sm dark:text-gray-400 mb-2"
                        >Clock Source</label
                      >
                      <select
                        id="clockSource"
                        bind:value={settings.midi.clockSource}
                        class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                      >
                        <option value="internal">Internal</option>
                        <option value="midi">MIDI Clock</option>
                        <option value="mtc">MTC (MIDI Time Code)</option>
                      </select>
                    </div>
                  </div>

                  <div class="space-y-4">
                    <div>
                      <label for="thruMode" class="block text-sm dark:text-gray-400 mb-2"
                        >MIDI Thru</label
                      >
                      <select
                        id="thruMode"
                        bind:value={settings.midi.thruMode}
                        class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                      >
                        <option value="off">Off</option>
                        <option value="soft">Software Thru</option>
                        <option value="hard">Hardware Thru</option>
                      </select>
                    </div>

                    <div>
                      <label for="velocityCurve" class="block text-sm dark:text-gray-400 mb-2"
                        >Velocity Curve</label
                      >
                      <select
                        id="velocityCurve"
                        bind:value={settings.midi.velocityCurve}
                        class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                      >
                        <option value="linear">Linear</option>
                        <option value="logarithmic">Logarithmic</option>
                        <option value="exponential">Exponential</option>
                        <option value="custom">Custom</option>
                      </select>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Appearance Settings -->
        {:else if activeTab === 'appearance'}
          <div class="space-y-6">
            <h3 class="text-xl dark:text-gray-200 mb-4">Appearance</h3>

            <div class="grid grid-cols-2 gap-6">
              <div class="space-y-6">
                <!-- Theme Selection -->
                <div>
                  <h4 class="text-lg dark:text-gray-300 mb-3">Theme</h4>
                  <div class="grid grid-cols-2 gap-3">
                    {#each themes as theme (theme.id)}
                      <button
                        on:click={() => (settings.theme = theme.id)}
                        class="aspect-square rounded-lg border-2 flex flex-col items-center justify-center p-4 transition-all"
                        class:border-primary={settings.theme === theme.id}
                        class:dark:border-window-border={settings.theme !== theme.id}
                        style="background-color: {theme.color}"
                      >
                        <span class="text-lg mb-1">🎨</span>
                        <span
                          class="text-xs {settings.theme === theme.id
                            ? 'dark:text-white'
                            : 'dark:text-gray-300'}"
                        >
                          {theme.name}
                        </span>
                      </button>
                    {/each}
                  </div>
                </div>

                <!-- UI Scale -->
                <div>
                  <h4 class="text-lg dark:text-gray-300 mb-3">UI Scale</h4>
                  <div class="flex gap-2">
                    {#each [0.75, 0.9, 1.0, 1.1, 1.25] as scale (scale)}
                      <button
                        on:click={() => (settings.uiScale = scale)}
                        class="flex-1 py-2 rounded border text-center"
                        class:dark:bg-primary={settings.uiScale === scale}
                        class:dark:bg-secondary={settings.uiScale !== scale}
                        class:dark:border-primary={settings.uiScale === scale}
                        class:dark:border-window-border={settings.uiScale !== scale}
                      >
                        {scale === 1.0 ? 'Normal' : `${scale * 100}%`}
                      </button>
                    {/each}
                  </div>
                </div>
              </div>

              <div class="space-y-6">
                <!-- Window Preferences -->
                <div>
                  <h4 class="text-lg dark:text-gray-300 mb-3">Window Preferences</h4>
                  <div class="space-y-3">
                    <label class="flex items-center gap-2">
                      <input type="checkbox" bind:checked={settings.alwaysOnTop} class="rounded" />
                      <span class="text-sm dark:text-gray-400">Always on top</span>
                    </label>

                    <label class="flex items-center gap-2">
                      <input
                        type="checkbox"
                        bind:checked={settings.useNativeTitlebar}
                        class="rounded"
                      />
                      <span class="text-sm dark:text-gray-400">Use native title bar</span>
                    </label>

                    <label class="flex items-center gap-2">
                      <input
                        type="checkbox"
                        bind:checked={settings.enableTransparency}
                        class="rounded"
                      />
                      <span class="text-sm dark:text-gray-400">Enable window transparency</span>
                    </label>

                    <label class="flex items-center gap-2">
                      <input
                        type="checkbox"
                        bind:checked={settings.autoHideMenuBar}
                        class="rounded"
                      />
                      <span class="text-sm dark:text-gray-400">Auto-hide menu bar</span>
                    </label>
                  </div>
                </div>

                <!-- Animations -->
                <div>
                  <h4 class="text-lg dark:text-gray-300 mb-3">Animations</h4>
                  <div class="space-y-3">
                    <label class="flex items-center gap-2">
                      <input
                        type="checkbox"
                        bind:checked={settings.enableAnimations}
                        class="rounded"
                      />
                      <span class="text-sm dark:text-gray-400">Enable animations</span>
                    </label>

                    <div>
                      <label for="animationSpeed" class="block text-sm dark:text-gray-400 mb-2"
                        >Animation Speed</label
                      >
                      <select
                        id="animationSpeed"
                        bind:value={settings.animationSpeed}
                        class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                      >
                        <option value="instant">Instant</option>
                        <option value="fast">Fast</option>
                        <option value="normal">Normal</option>
                        <option value="slow">Slow</option>
                      </select>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Shortcuts Settings -->
        {:else if activeTab === 'shortcuts'}
          <div class="space-y-6">
            <h3 class="text-xl dark:text-gray-200 mb-4">Keyboard Shortcuts</h3>

            <div class="space-y-4">
              <!-- Category Tabs -->
              <div class="border-b dark:border-window-border">
                <div class="flex space-x-4">
                  {#each ['global', 'transport', 'editing', 'navigation', 'window'] as category (category)}
                    <button
                      on:click={() => (settings.shortcutCategory = category)}
                      class="px-4 py-2 border-b-2 -mb-px transition-colors"
                      class:dark:border-primary={settings.shortcutCategory === category}
                      class:dark:border-transparent={settings.shortcutCategory !== category}
                      class:dark:text-gray-300={settings.shortcutCategory !== category}
                    >
                      {category.charAt(0).toUpperCase() + category.slice(1)}
                    </button>
                  {/each}
                </div>
              </div>

              <!-- Shortcuts Table -->
              <div class="overflow-x-auto">
                <table class="w-full text-sm">
                  <thead>
                    <tr class="border-b dark:border-window-border">
                      <th class="p-3 text-left dark:text-gray-400">Action</th>
                      <th class="p-3 text-left dark:text-gray-400">Shortcut</th>
                      <th class="p-3 text-left dark:text-gray-400">Alternative</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each getShortcutsForCategory(settings.shortcutCategory) as shortcut (shortcut.id)}
                      <tr class="border-b dark:border-window-border hover:dark:bg-window-subtle">
                        <td class="p-3 dark:text-gray-300">{shortcut.description}</td>
                        <td class="p-3">
                          <input
                            type="text"
                            value={shortcut.key}
                            readonly
                            class="px-2 py-1 dark:bg-input dark:border-window-border rounded text-center min-w-32"
                          />
                        </td>
                        <td class="p-3">
                          <input
                            type="text"
                            value={shortcut.alternative}
                            readonly
                            class="px-2 py-1 dark:bg-input dark:border-window-border rounded text-center min-w-32"
                          />
                        </td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            </div>
          </div>

          <!-- Paths Settings -->
        {:else if activeTab === 'paths'}
          <div class="space-y-6">
            <h3 class="text-xl dark:text-gray-200 mb-4">File Paths</h3>

            <div class="space-y-4">
              <div>
                <label class="block text-sm dark:text-gray-400 mb-2">Database Location</label>
                <div class="flex gap-2">
                  <input
                    type="text"
                    bind:value={settings.databasePath}
                    readonly
                    class="flex-1 px-3 py-2 dark:bg-input dark:border-window-border rounded"
                  />
                  <button
                    on:click={selectDatabasePath}
                    class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
                  >
                    Browse
                  </button>
                </div>
              </div>

              <div>
                <label class="block text-sm dark:text-gray-400 mb-2">MIDI Files Location</label>
                <div class="flex gap-2">
                  <input
                    type="text"
                    bind:value={settings.midiPath}
                    readonly
                    class="flex-1 px-3 py-2 dark:bg-input dark:border-window-border rounded"
                  />
                  <button
                    on:click={() => selectPath('midiPath')}
                    class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
                  >
                    Browse
                  </button>
                </div>
              </div>

              <div>
                <label class="block text-sm dark:text-gray-400 mb-2">Projects Location</label>
                <div class="flex gap-2">
                  <input
                    type="text"
                    bind:value={settings.projectsPath}
                    readonly
                    class="flex-1 px-3 py-2 dark:bg-input dark:border-window-border rounded"
                  />
                  <button
                    on:click={() => selectPath('projectsPath')}
                    class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
                  >
                    Browse
                  </button>
                </div>
              </div>

              <div>
                <label class="block text-sm dark:text-gray-400 mb-2">Backup Location</label>
                <div class="flex gap-2">
                  <input
                    type="text"
                    bind:value={settings.backupPath}
                    readonly
                    class="flex-1 px-3 py-2 dark:bg-input dark:border-window-border rounded"
                  />
                  <button
                    on:click={() => selectPath('backupPath')}
                    class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
                  >
                    Browse
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Advanced Settings -->
        {:else if activeTab === 'advanced'}
          <div class="space-y-6">
            <h3 class="text-xl dark:text-gray-200 mb-4">Advanced Settings</h3>

            <div class="space-y-6">
              <div>
                <h4 class="text-lg dark:text-gray-300 mb-3">Performance</h4>
                <div class="grid grid-cols-2 gap-6">
                  <div class="space-y-4">
                    <div>
                      <label class="block text-sm dark:text-gray-400 mb-2">Render Threads</label>
                      <input
                        type="range"
                        bind:value={settings.renderThreads}
                        min="1"
                        max={maxThreads}
                        step="1"
                        class="w-full"
                      />
                      <div class="text-xs dark:text-gray-500 text-center">
                        {settings.renderThreads} thread{settings.renderThreads !== 1 ? 's' : ''} (max:
                        {maxThreads})
                      </div>
                    </div>

                    <div>
                      <label for="audioThreadPriority" class="block text-sm dark:text-gray-400 mb-2"
                        >Audio Thread Priority</label
                      >
                      <select
                        id="audioThreadPriority"
                        bind:value={settings.audioThreadPriority}
                        class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                      >
                        <option value="normal">Normal</option>
                        <option value="high">High</option>
                        <option value="time-critical">Time Critical</option>
                      </select>
                    </div>
                  </div>

                  <div class="space-y-4">
                    <div>
                      <label class="block text-sm dark:text-gray-400 mb-2"
                        >Memory Cache Size (MB)</label
                      >
                      <input
                        type="range"
                        bind:value={settings.cacheSize}
                        min="128"
                        max="4096"
                        step="128"
                        class="w-full"
                      />
                      <div class="text-xs dark:text-gray-500 text-center">
                        {settings.cacheSize} MB
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .settings-window {
    min-height: 600px;
  }

  .sidebar {
    background: linear-gradient(to bottom, var(--window-bg), var(--window-subtle));
  }

  input[type='range'] {
    accent-color: var(--primary-color);
  }
</style>
