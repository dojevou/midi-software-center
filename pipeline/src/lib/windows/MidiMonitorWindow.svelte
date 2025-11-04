<script lang="ts">
  /**
   * MidiMonitorWindow - Real-time MIDI event monitoring
   *
   * Task-O-Matic: Monitor MIDI events in real-time, filter, search, view details.
   */

  import { onMount, onDestroy } from 'svelte';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';

  // ============================================================================
  // Types
  // ============================================================================

  type MidiEventType =
    | 'NoteOn'
    | 'NoteOff'
    | 'ControlChange'
    | 'PitchBend'
    | 'ProgramChange'
    | 'Aftertouch'
    | 'ChannelPressure'
    | 'SysEx'
    | 'Clock'
    | 'Start'
    | 'Stop'
    | 'Continue'
    | 'Unknown';

  interface MidiEvent {
    id: number;
    timestamp: number; // milliseconds since start
    type: MidiEventType;
    channel: number; // 1-16
    data1: number; // 0-127
    data2: number; // 0-127
    description: string;
    rawBytes: number[];
  }

  // ============================================================================
  // Constants
  // ============================================================================

  const MAX_EVENTS = 1000;
  const NOTE_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

  const EVENT_TYPE_COLORS: Record<MidiEventType, string> = {
    NoteOn: '#4caf50',
    NoteOff: '#f44336',
    ControlChange: '#2196f3',
    PitchBend: '#9c27b0',
    ProgramChange: '#ff9800',
    Aftertouch: '#00bcd4',
    ChannelPressure: '#009688',
    SysEx: '#795548',
    Clock: '#607d8b',
    Start: '#8bc34a',
    Stop: '#e91e63',
    Continue: '#ffc107',
    Unknown: '#9e9e9e'
  };

  // ============================================================================
  // State
  // ============================================================================

  // Event list
  let events: MidiEvent[] = [];
  let eventIdCounter = 1;

  // Monitoring state
  let isMonitoring = false;
  let isPaused = false;
  let eventsPerSecond = 0;
  let startTime = 0;

  // UI state
  let searchQuery = '';
  let selectedEventId: number | null = null;
  let autoScroll = true;
  let showHex = false;

  // Filters
  let filterEventType: MidiEventType | 'all' = 'all';
  let filterChannel: number | 'all' = 'all';
  let filterNoteNumber = '';

  // Event listener
  let midiEventUnlisten: (() => void) | null = null;

  // Refs
  let eventListContainer: HTMLElement;

  // Stats tracking
  let eventCounter = 0;
  let lastStatsUpdate = 0;
  let statsInterval: number | null = null;

  // ============================================================================
  // Reactive
  // ============================================================================

  $: filteredEvents = filterEvents(events, searchQuery, filterEventType, filterChannel, filterNoteNumber);
  $: selectedEvent = events.find(e => e.id === selectedEventId) || null;

  // ============================================================================
  // Event Filtering
  // ============================================================================

  function filterEvents(
    evts: MidiEvent[],
    query: string,
    typeFilter: MidiEventType | 'all',
    channelFilter: number | 'all',
    noteFilter: string
  ): MidiEvent[] {
    return evts.filter(event => {
      // Type filter
      if (typeFilter !== 'all' && event.type !== typeFilter) {
        return false;
      }

      // Channel filter
      if (channelFilter !== 'all' && event.channel !== channelFilter) {
        return false;
      }

      // Note number filter
      if (noteFilter && event.type === 'NoteOn' || event.type === 'NoteOff') {
        const noteNum = parseInt(noteFilter);
        if (!isNaN(noteNum) && event.data1 !== noteNum) {
          return false;
        }
      }

      // Search query
      if (query) {
        const lowerQuery = query.toLowerCase();
        return (
          event.type.toLowerCase().includes(lowerQuery) ||
          event.description.toLowerCase().includes(lowerQuery) ||
          event.channel.toString().includes(lowerQuery)
        );
      }

      return true;
    });
  }

  // ============================================================================
  // Monitoring Control
  // ============================================================================

  async function startMonitoring(): Promise<void> {
    try {
      await invoke('start_midi_monitoring');
      isMonitoring = true;
      isPaused = false;
      startTime = Date.now();
      eventCounter = 0;
      lastStatsUpdate = Date.now();

      // Start stats update interval
      statsInterval = window.setInterval(updateStats, 1000);

      // Listen for MIDI events
      midiEventUnlisten = await listen('midi-event', (event) => {
        if (!isPaused) {
          handleMidiEvent(event.payload as any);
        }
      });
    } catch (err) {
      console.error('Failed to start monitoring:', err);
    }
  }

  async function stopMonitoring(): Promise<void> {
    try {
      await invoke('stop_midi_monitoring');
      isMonitoring = false;
      isPaused = false;

      if (midiEventUnlisten) {
        midiEventUnlisten();
        midiEventUnlisten = null;
      }

      if (statsInterval !== null) {
        clearInterval(statsInterval);
        statsInterval = null;
      }

      eventsPerSecond = 0;
    } catch (err) {
      console.error('Failed to stop monitoring:', err);
    }
  }

  function togglePause(): void {
    isPaused = !isPaused;
  }

  function clearEvents(): void {
    events = [];
    eventIdCounter = 1;
    selectedEventId = null;
    eventCounter = 0;
  }

  // ============================================================================
  // Event Handling
  // ============================================================================

  function handleMidiEvent(payload: {
    type: MidiEventType;
    channel: number;
    data1: number;
    data2: number;
    rawBytes: number[];
  }): void {
    const timestamp = Date.now() - startTime;

    const event: MidiEvent = {
      id: eventIdCounter++,
      timestamp,
      type: payload.type,
      channel: payload.channel,
      data1: payload.data1,
      data2: payload.data2,
      description: formatEventDescription(payload.type, payload.channel, payload.data1, payload.data2),
      rawBytes: payload.rawBytes
    };

    events = [...events, event];
    eventCounter++;

    // Limit event list size
    if (events.length > MAX_EVENTS) {
      events = events.slice(-MAX_EVENTS);
    }

    // Auto-scroll to bottom
    if (autoScroll && eventListContainer) {
      setTimeout(() => {
        eventListContainer.scrollTop = eventListContainer.scrollHeight;
      }, 0);
    }
  }

  function updateStats(): void {
    const now = Date.now();
    const elapsed = (now - lastStatsUpdate) / 1000; // seconds
    eventsPerSecond = Math.round(eventCounter / elapsed);
    eventCounter = 0;
    lastStatsUpdate = now;
  }

  // ============================================================================
  // Formatting
  // ============================================================================

  function formatEventDescription(
    type: MidiEventType,
    channel: number,
    data1: number,
    data2: number
  ): string {
    switch (type) {
      case 'NoteOn':
        return `Note On: ${midiNoteToName(data1)} (${data1}), Velocity ${data2}`;
      case 'NoteOff':
        return `Note Off: ${midiNoteToName(data1)} (${data1}), Velocity ${data2}`;
      case 'ControlChange':
        return `CC ${data1}: ${data2} (${getControllerName(data1)})`;
      case 'PitchBend':
        const pitchValue = (data2 << 7) | data1;
        return `Pitch Bend: ${pitchValue - 8192}`;
      case 'ProgramChange':
        return `Program Change: ${data1}`;
      case 'Aftertouch':
        return `Aftertouch: Note ${midiNoteToName(data1)} (${data1}), Pressure ${data2}`;
      case 'ChannelPressure':
        return `Channel Pressure: ${data1}`;
      case 'SysEx':
        return 'System Exclusive Message';
      case 'Clock':
        return 'MIDI Clock';
      case 'Start':
        return 'MIDI Start';
      case 'Stop':
        return 'MIDI Stop';
      case 'Continue':
        return 'MIDI Continue';
      default:
        return 'Unknown MIDI Event';
    }
  }

  function midiNoteToName(note: number): string {
    const octave = Math.floor(note / 12) - 1;
    const noteName = NOTE_NAMES[note % 12];
    return `${noteName}${octave}`;
  }

  function getControllerName(cc: number): string {
    const ccNames: Record<number, string> = {
      0: 'Bank Select',
      1: 'Modulation',
      7: 'Volume',
      10: 'Pan',
      11: 'Expression',
      64: 'Sustain',
      65: 'Portamento',
      66: 'Sostenuto',
      67: 'Soft Pedal',
      71: 'Resonance',
      72: 'Release Time',
      73: 'Attack Time',
      74: 'Brightness',
      91: 'Reverb',
      93: 'Chorus'
    };
    return ccNames[cc] || 'Unknown';
  }

  function formatTimestamp(ms: number): string {
    const totalSeconds = ms / 1000;
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = Math.floor(totalSeconds % 60);
    const millis = Math.floor((totalSeconds % 1) * 1000);
    return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}.${millis.toString().padStart(3, '0')}`;
  }

  function formatRawBytes(bytes: number[]): string {
    if (showHex) {
      return bytes.map(b => b.toString(16).toUpperCase().padStart(2, '0')).join(' ');
    } else {
      return bytes.join(' ');
    }
  }

  function getEventColor(type: MidiEventType): string {
    return EVENT_TYPE_COLORS[type] || EVENT_TYPE_COLORS.Unknown;
  }

  // ============================================================================
  // Event Selection
  // ============================================================================

  function selectEvent(eventId: number): void {
    selectedEventId = selectedEventId === eventId ? null : eventId;
  }

  // ============================================================================
  // Keyboard Shortcuts
  // ============================================================================

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'c' && (e.metaKey || e.ctrlKey)) {
      if (e.shiftKey) {
        // Cmd+Shift+C: Clear log
        clearEvents();
        e.preventDefault();
      }
    } else if (e.key === ' ' && !e.target || (e.target as HTMLElement).tagName !== 'INPUT') {
      // Space: Toggle pause
      togglePause();
      e.preventDefault();
    } else if (e.key === 'Escape') {
      // Escape: Deselect event
      selectedEventId = null;
    }
  }

  // ============================================================================
  // Lifecycle
  // ============================================================================

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    if (isMonitoring) {
      stopMonitoring();
    }
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

<WindowBase windowId="midi-monitor" title="MIDI Monitor">
  <div class="monitor-window">
    <!-- Header -->
    <div class="header">
      <div class="monitoring-controls">
        {#if !isMonitoring}
          <button class="btn btn-start" on:click={startMonitoring}>
            ▶ Start Monitoring
          </button>
        {:else}
          <button class="btn btn-stop" on:click={stopMonitoring}>
            ■ Stop Monitoring
          </button>
          <button class="btn btn-pause" on:click={togglePause}>
            {isPaused ? '▶ Resume' : '⏸ Pause'}
          </button>
        {/if}
        <button class="btn btn-clear" on:click={clearEvents}>
          Clear Log
        </button>
      </div>

      <div class="monitoring-status">
        {#if isMonitoring}
          <div class="recording-indicator" class:paused={isPaused}>
            <span class="indicator-dot"></span>
            <span class="indicator-text">
              {isPaused ? 'Paused' : 'Recording'}
            </span>
          </div>
          <div class="event-speed">
            {eventsPerSecond} events/sec
          </div>
        {/if}
      </div>
    </div>

    <!-- Filters Bar -->
    <div class="filters-bar">
      <div class="search-box">
        <input
          type="text"
          class="search-input"
          placeholder="Search events..."
          bind:value={searchQuery}
        />
      </div>

      <div class="filter-controls">
        <select class="filter-select" bind:value={filterEventType}>
          <option value="all">All Types</option>
          <option value="NoteOn">Note On</option>
          <option value="NoteOff">Note Off</option>
          <option value="ControlChange">Control Change</option>
          <option value="PitchBend">Pitch Bend</option>
          <option value="ProgramChange">Program Change</option>
          <option value="Aftertouch">Aftertouch</option>
          <option value="ChannelPressure">Channel Pressure</option>
          <option value="SysEx">SysEx</option>
          <option value="Clock">Clock</option>
          <option value="Start">Start</option>
          <option value="Stop">Stop</option>
          <option value="Continue">Continue</option>
        </select>

        <select class="filter-select" bind:value={filterChannel}>
          <option value="all">All Channels</option>
          {#each Array(16) as _, i}
            <option value={i + 1}>Channel {i + 1}</option>
          {/each}
        </select>

        <input
          type="text"
          class="filter-input"
          placeholder="Note #"
          bind:value={filterNoteNumber}
        />

        <label class="checkbox-label">
          <input type="checkbox" bind:checked={autoScroll} />
          <span>Auto-scroll</span>
        </label>

        <label class="checkbox-label">
          <input type="checkbox" bind:checked={showHex} />
          <span>Hex</span>
        </label>
      </div>
    </div>

    <!-- Event List -->
    <div class="event-list-container" bind:this={eventListContainer}>
      <table class="event-table">
        <thead>
          <tr>
            <th class="col-time">Time</th>
            <th class="col-type">Type</th>
            <th class="col-channel">Ch</th>
            <th class="col-data">Data1</th>
            <th class="col-data">Data2</th>
            <th class="col-description">Description</th>
          </tr>
        </thead>
        <tbody>
          {#if filteredEvents.length === 0}
            <tr class="empty-row">
              <td colspan="6">
                {#if !isMonitoring}
                  <div class="empty-state">
                    <p>Start monitoring to see MIDI events</p>
                  </div>
                {:else if isPaused}
                  <div class="empty-state">
                    <p>Monitoring paused</p>
                  </div>
                {:else}
                  <div class="empty-state">
                    <p>No events yet...</p>
                  </div>
                {/if}
              </td>
            </tr>
          {:else}
            {#each filteredEvents as event (event.id)}
              <tr
                class="event-row"
                class:selected={selectedEventId === event.id}
                on:click={() => selectEvent(event.id)}
              >
                <td class="col-time">{formatTimestamp(event.timestamp)}</td>
                <td
                  class="col-type"
                  style="color: {getEventColor(event.type)}"
                >
                  {event.type}
                </td>
                <td class="col-channel">{event.channel}</td>
                <td class="col-data">{event.data1}</td>
                <td class="col-data">{event.data2}</td>
                <td class="col-description">{event.description}</td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>

    <!-- Event Details Panel -->
    {#if selectedEvent}
      <div class="details-panel">
        <div class="details-header">
          <h4 class="details-title">Event Details</h4>
          <button class="close-btn" on:click={() => selectedEventId = null}>×</button>
        </div>

        <div class="details-content">
          <div class="detail-row">
            <span class="detail-label">Event ID:</span>
            <span class="detail-value">{selectedEvent.id}</span>
          </div>

          <div class="detail-row">
            <span class="detail-label">Timestamp:</span>
            <span class="detail-value">{formatTimestamp(selectedEvent.timestamp)}</span>
          </div>

          <div class="detail-row">
            <span class="detail-label">Type:</span>
            <span class="detail-value" style="color: {getEventColor(selectedEvent.type)}">
              {selectedEvent.type}
            </span>
          </div>

          <div class="detail-row">
            <span class="detail-label">Channel:</span>
            <span class="detail-value">{selectedEvent.channel}</span>
          </div>

          <div class="detail-row">
            <span class="detail-label">Data 1:</span>
            <span class="detail-value">{selectedEvent.data1}</span>
          </div>

          <div class="detail-row">
            <span class="detail-label">Data 2:</span>
            <span class="detail-value">{selectedEvent.data2}</span>
          </div>

          <div class="detail-row">
            <span class="detail-label">Description:</span>
            <span class="detail-value">{selectedEvent.description}</span>
          </div>

          <div class="detail-row">
            <span class="detail-label">Raw Bytes:</span>
            <span class="detail-value mono">{formatRawBytes(selectedEvent.rawBytes)}</span>
          </div>
        </div>
      </div>
    {/if}

    <!-- Footer Stats -->
    <div class="footer">
      <div class="footer-stats">
        <span class="stat-label">Total Events:</span>
        <span class="stat-value">{events.length}</span>
      </div>
      <div class="footer-stats">
        <span class="stat-label">Filtered:</span>
        <span class="stat-value">{filteredEvents.length}</span>
      </div>
      <div class="footer-stats">
        <span class="stat-label">Max Events:</span>
        <span class="stat-value">{MAX_EVENTS}</span>
      </div>
    </div>
  </div>
</WindowBase>

<style>
  .monitor-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--window-bg, #1e1e1e);
  }

  /* Header */
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: var(--header-bg, #2d2d2d);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .monitoring-controls {
    display: flex;
    gap: 8px;
  }

  .monitoring-status {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .recording-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .indicator-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #f44336;
    animation: blink 1s infinite;
  }

  .recording-indicator.paused .indicator-dot {
    background: #ff9800;
    animation: none;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }

  .indicator-text {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .event-speed {
    font-size: 13px;
    color: var(--text-secondary, #9e9e9e);
    font-family: monospace;
  }

  /* Buttons */
  .btn {
    padding: 6px 12px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    transition: all 0.15s ease;
  }

  .btn-start {
    background: var(--success-bg, #388e3c);
    color: white;
  }

  .btn-start:hover {
    background: var(--success-hover, #2e7d32);
  }

  .btn-stop {
    background: var(--error-bg, #d32f2f);
    color: white;
  }

  .btn-stop:hover {
    background: var(--error-hover, #c62828);
  }

  .btn-pause {
    background: var(--warning-bg, #f57c00);
    color: white;
  }

  .btn-pause:hover {
    background: var(--warning-hover, #ef6c00);
  }

  .btn-clear {
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
  }

  .btn-clear:hover {
    background: var(--button-hover, #4a4a4a);
  }

  /* Filters Bar */
  .filters-bar {
    display: flex;
    gap: 12px;
    padding: 12px 16px;
    background: var(--panel-bg, #252525);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .search-box {
    flex: 1;
  }

  .search-input {
    width: 100%;
    padding: 6px 12px;
    background: var(--input-bg, #1e1e1e);
    color: var(--text-primary, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    font-size: 13px;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--accent-color, #0078d4);
  }

  .filter-controls {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .filter-select,
  .filter-input {
    padding: 6px 10px;
    background: var(--input-bg, #1e1e1e);
    color: var(--text-primary, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    font-size: 13px;
  }

  .filter-select:focus,
  .filter-input:focus {
    outline: none;
    border-color: var(--accent-color, #0078d4);
  }

  .filter-input {
    width: 70px;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-secondary, #9e9e9e);
    cursor: pointer;
  }

  /* Event List */
  .event-list-container {
    flex: 1;
    overflow-y: auto;
    background: var(--window-bg, #1e1e1e);
  }

  .event-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
  }

  .event-table thead {
    position: sticky;
    top: 0;
    background: var(--header-bg, #2d2d2d);
    z-index: 1;
  }

  .event-table th {
    padding: 8px 12px;
    text-align: left;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .event-table td {
    padding: 6px 12px;
    border-bottom: 1px solid var(--border-color, #3e3e3e);
    color: var(--text-primary, #e0e0e0);
  }

  .event-row {
    cursor: pointer;
    transition: background 0.1s ease;
  }

  .event-row:hover {
    background: var(--row-hover, #2d2d2d);
  }

  .event-row.selected {
    background: var(--row-selected, #1a3a52);
  }

  .empty-row {
    height: 200px;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary, #9e9e9e);
    font-size: 14px;
  }

  .col-time {
    width: 100px;
    font-family: monospace;
  }

  .col-type {
    width: 140px;
    font-weight: 600;
  }

  .col-channel {
    width: 40px;
    text-align: center;
  }

  .col-data {
    width: 60px;
    text-align: center;
  }

  .col-description {
    flex: 1;
  }

  /* Details Panel */
  .details-panel {
    border-top: 1px solid var(--border-color, #3e3e3e);
    background: var(--panel-bg, #252525);
  }

  .details-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: var(--header-bg, #2d2d2d);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .details-title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .close-btn {
    padding: 4px 8px;
    background: transparent;
    color: var(--text-secondary, #9e9e9e);
    border: none;
    cursor: pointer;
    font-size: 20px;
    line-height: 1;
  }

  .close-btn:hover {
    color: var(--text-primary, #e0e0e0);
  }

  .details-content {
    padding: 12px 16px;
  }

  .detail-row {
    display: flex;
    justify-content: space-between;
    padding: 6px 0;
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .detail-row:last-child {
    border-bottom: none;
  }

  .detail-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary, #9e9e9e);
  }

  .detail-value {
    font-size: 12px;
    color: var(--text-primary, #e0e0e0);
  }

  .detail-value.mono {
    font-family: monospace;
  }

  /* Footer */
  .footer {
    display: flex;
    gap: 24px;
    padding: 8px 16px;
    background: var(--header-bg, #2d2d2d);
    border-top: 1px solid var(--border-color, #3e3e3e);
  }

  .footer-stats {
    display: flex;
    gap: 6px;
    font-size: 12px;
  }

  .stat-label {
    color: var(--text-secondary, #9e9e9e);
  }

  .stat-value {
    color: var(--text-primary, #e0e0e0);
    font-weight: 600;
  }
</style>
