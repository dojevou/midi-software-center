<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  
  const dispatch = createEventDispatcher();
  
  // Props
  export let scriptId: string | null = null;
  export let initialContent: string = '';
  
  // State
  let content: string = initialContent;
  let scriptName: string = 'Untitled Script';
  let scriptDescription: string = '';
  let scriptType: 'MidiProcessor' | 'Generator' | 'Automation' | 'Utility' | 'Action' = 'MidiProcessor';
  let isSaved: boolean = true;
  let isRunning: boolean = false;
  let consoleOutput: string[] = [];
  let error: string | null = null;
  
  // Editor settings
  let fontSize: number = 14;
  let showLineNumbers: boolean = true;
  let wordWrap: boolean = true;
  
  // Templates
  type TemplateName = 'midi_processor' | 'arpeggiator' | 'chord_trigger' | 'velocity_curve' | 'random_humanize';
  const templates: Record<TemplateName, string> = {
    'midi_processor': `-- MIDI Processor Script
-- Transforms MIDI events in real-time

function on_midi(channel, status, data1, data2)
    -- Pass through unchanged
    return {{channel, status, data1, data2}}
end
`,
    'arpeggiator': `-- Arpeggiator Script
-- Hold notes to arpeggiate

local held_notes = {}
local arp_index = 1

function on_midi(channel, status, data1, data2)
    local msg_type = bit32.band(status, 0xF0)
    
    if msg_type == 0x90 and data2 > 0 then
        -- Note on: add to held notes
        table.insert(held_notes, data1)
        table.sort(held_notes)
        return {}  -- Don't pass through
    elseif msg_type == 0x80 or (msg_type == 0x90 and data2 == 0) then
        -- Note off: remove from held notes
        for i, note in ipairs(held_notes) do
            if note == data1 then
                table.remove(held_notes, i)
                break
            end
        end
        return {}
    end
    
    return {{channel, status, data1, data2}}
end

function on_tick()
    if #held_notes == 0 then return end
    
    local note = held_notes[((arp_index - 1) % #held_notes) + 1]
    midi.note_on("default", 0, note, 100)
    timer.sleep_ms(50)
    midi.note_off("default", 0, note)
    
    arp_index = arp_index + 1
end
`,
    'chord_trigger': `-- Chord Trigger Script
-- Single notes trigger chords

local chord_type = "maj7"

function on_midi(channel, status, data1, data2)
    local msg_type = bit32.band(status, 0xF0)
    
    if msg_type == 0x90 and data2 > 0 then
        local chord = theory.chords[chord_type]
        local output = {}
        
        for _, interval in ipairs(chord) do
            local note = data1 + interval
            if note <= 127 then
                table.insert(output, {channel, 0x90, note, data2})
            end
        end
        
        return output
    elseif msg_type == 0x80 then
        local chord = theory.chords[chord_type]
        local output = {}
        
        for _, interval in ipairs(chord) do
            local note = data1 + interval
            if note <= 127 then
                table.insert(output, {channel, 0x80, note, 0})
            end
        end
        
        return output
    end
    
    return {{channel, status, data1, data2}}
end
`,
    'velocity_curve': `-- Velocity Curve Script
-- Apply custom velocity curves

local curve_type = "exponential"
local min_vel = 20
local max_vel = 127

function apply_curve(velocity)
    local normalized = velocity / 127
    local curved
    
    if curve_type == "exponential" then
        curved = normalized * normalized
    elseif curve_type == "logarithmic" then
        curved = math.sqrt(normalized)
    else
        curved = normalized
    end
    
    return math.floor(util.lerp(min_vel, max_vel, curved))
end

function on_midi(channel, status, data1, data2)
    local msg_type = bit32.band(status, 0xF0)
    
    if msg_type == 0x90 and data2 > 0 then
        local new_vel = apply_curve(data2)
        return {{channel, status, data1, new_vel}}
    end
    
    return {{channel, status, data1, data2}}
end
`,
    'random_humanize': `-- Random Humanize Script
-- Add timing and velocity variation

function on_midi(channel, status, data1, data2)
    local msg_type = bit32.band(status, 0xF0)
    
    if msg_type == 0x90 and data2 > 0 then
        -- Random velocity variation ±10%
        local variance = data2 * 0.1
        local new_vel = util.clamp(
            data2 + util.random(-variance, variance),
            1, 127
        )
        
        return {{channel, status, data1, math.floor(new_vel)}}
    end
    
    return {{channel, status, data1, data2}}
end
`
  };
  
  // Syntax highlighting keywords
  const keywords = ['function', 'local', 'if', 'then', 'else', 'elseif', 'end', 'for', 'while', 'do', 'return', 'and', 'or', 'not', 'in', 'nil', 'true', 'false', 'repeat', 'until', 'break'];
  const builtins = ['midi', 'theory', 'util', 'transport', 'params', 'timer', 'math', 'table', 'string', 'bit32'];
  
  $: {
    // Track unsaved changes
    if (content !== initialContent) {
      isSaved = false;
    }
  }
  
  function loadTemplate(name: string) {
    if (name && name in templates) {
      content = templates[name as TemplateName];
      isSaved = false;
    }
  }
  
  async function runScript() {
    isRunning = true;
    error = null;
    consoleOutput = [];
    
    try {
      await invoke('script_load', {
        id: scriptId || `temp_${Date.now()}`,
        name: scriptName,
        content,
        scriptType
      });
      
      consoleOutput = [...consoleOutput, '✓ Script loaded successfully'];
      dispatch('run', { content });
    } catch (e) {
      error = String(e);
      consoleOutput = [...consoleOutput, `✗ Error: ${e}`];
    } finally {
      isRunning = false;
    }
  }
  
  async function saveScript() {
    try {
      await invoke('script_save', {
        id: scriptId,
        name: scriptName,
        description: scriptDescription,
        content,
        scriptType
      });
      
      isSaved = true;
      consoleOutput = [...consoleOutput, '✓ Script saved'];
      dispatch('save', { id: scriptId, content });
    } catch (e) {
      error = String(e);
    }
  }
  
  function handleKeyDown(e: KeyboardEvent) {
    // Ctrl/Cmd + S to save
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault();
      saveScript();
    }
    
    // Ctrl/Cmd + Enter to run
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      runScript();
    }
    
    // Tab for indentation
    if (e.key === 'Tab') {
      e.preventDefault();
      const textarea = e.target as HTMLTextAreaElement;
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      
      content = content.substring(0, start) + '    ' + content.substring(end);
      
      // Move cursor after tab
      setTimeout(() => {
        textarea.selectionStart = textarea.selectionEnd = start + 4;
      }, 0);
    }
  }
  
  function getLineNumbers(): string[] {
    const lines = content.split('\n');
    return Array.from({ length: lines.length }, (_, i) => String(i + 1));
  }
  
  function clearConsole() {
    consoleOutput = [];
    error = null;
  }
</script>

<div class="script-editor">
  <!-- Toolbar -->
  <div class="toolbar">
    <div class="toolbar-section">
      <input 
        type="text" 
        class="script-name" 
        bind:value={scriptName}
        placeholder="Script Name"
      />
      
      <select bind:value={scriptType}>
        <option value="MidiProcessor">MIDI Processor</option>
        <option value="Generator">Generator</option>
        <option value="Automation">Automation</option>
        <option value="Utility">Utility</option>
        <option value="Action">Action</option>
      </select>
    </div>
    
    <div class="toolbar-section">
      <select on:change={(e) => loadTemplate(e.currentTarget.value)}>
        <option value="">Load Template...</option>
        <option value="midi_processor">MIDI Processor</option>
        <option value="arpeggiator">Arpeggiator</option>
        <option value="chord_trigger">Chord Trigger</option>
        <option value="velocity_curve">Velocity Curve</option>
        <option value="random_humanize">Humanize</option>
      </select>
    </div>
    
    <div class="toolbar-section">
      <button 
        class="run-btn" 
        on:click={runScript}
        disabled={isRunning}
      >
        {#if isRunning}
          <span class="spinner"></span>
        {:else}
          ▶
        {/if}
        Run
      </button>
      
      <button 
        class="save-btn"
        class:unsaved={!isSaved}
        on:click={saveScript}
      >
        💾 Save {!isSaved ? '*' : ''}
      </button>
    </div>
  </div>
  
  <!-- Editor -->
  <div class="editor-container">
    <div class="editor-wrapper">
      {#if showLineNumbers}
        <div class="line-numbers">
          {#each getLineNumbers() as num}
            <span>{num}</span>
          {/each}
        </div>
      {/if}
      
      <textarea
        class="code-editor"
        bind:value={content}
        on:keydown={handleKeyDown}
        spellcheck="false"
        style="font-size: {fontSize}px; white-space: {wordWrap ? 'pre-wrap' : 'pre'};"
        placeholder="-- Write your Lua script here..."
      ></textarea>
    </div>
  </div>
  
  <!-- Console -->
  <div class="console">
    <div class="console-header">
      <span>Console</span>
      <button class="clear-btn" on:click={clearConsole}>Clear</button>
    </div>
    
    <div class="console-output">
      {#each consoleOutput as line}
        <div class="console-line" class:error={line.startsWith('✗')}>
          {line}
        </div>
      {/each}
      
      {#if error}
        <div class="console-line error">
          Error: {error}
        </div>
      {/if}
    </div>
  </div>
  
  <!-- API Reference (collapsible) -->
  <details class="api-reference">
    <summary>API Reference</summary>
    
    <div class="api-content">
      <div class="api-section">
        <h4>MIDI</h4>
        <code>midi.note_on(device, channel, note, velocity)</code>
        <code>midi.note_off(device, channel, note)</code>
        <code>midi.cc(device, channel, controller, value)</code>
        <code>midi.program_change(device, channel, program)</code>
        <code>midi.pitch_bend(device, channel, value)</code>
      </div>
      
      <div class="api-section">
        <h4>Transport</h4>
        <code>transport.play()</code>
        <code>transport.stop()</code>
        <code>transport.set_tempo(bpm)</code>
        <code>transport.set_position(ticks)</code>
      </div>
      
      <div class="api-section">
        <h4>Theory</h4>
        <code>theory.scales.major, minor, dorian, etc.</code>
        <code>theory.chords.maj, min, maj7, min7, etc.</code>
        <code>theory.note_name(midi_note)</code>
        <code>theory.transpose(note, semitones)</code>
        <code>theory.in_scale(note, root, scale)</code>
      </div>
      
      <div class="api-section">
        <h4>Utility</h4>
        <code>util.random(min, max)</code>
        <code>util.clamp(value, min, max)</code>
        <code>util.lerp(a, b, t)</code>
        <code>util.map(val, in_min, in_max, out_min, out_max)</code>
        <code>util.log(message)</code>
      </div>
    </div>
  </details>
</div>

<style>
  .script-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary, #1e1e1e);
    color: var(--text-primary, #d4d4d4);
    font-family: 'Fira Code', 'Consolas', monospace;
  }
  
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: var(--bg-secondary, #252526);
    border-bottom: 1px solid var(--border-color, #3c3c3c);
  }
  
  .toolbar-section {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  
  .script-name {
    padding: 6px 12px;
    background: var(--bg-input, #3c3c3c);
    border: 1px solid var(--border-color, #5a5a5a);
    border-radius: 4px;
    color: var(--text-primary, #d4d4d4);
    font-size: 14px;
    width: 200px;
  }
  
  .toolbar select {
    padding: 6px 12px;
    background: var(--bg-input, #3c3c3c);
    border: 1px solid var(--border-color, #5a5a5a);
    border-radius: 4px;
    color: var(--text-primary, #d4d4d4);
    font-size: 13px;
  }
  
  .run-btn, .save-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 16px;
    border: none;
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
    transition: background 0.2s;
  }
  
  .run-btn {
    background: var(--success-color, #4caf50);
    color: white;
  }
  
  .run-btn:hover {
    background: var(--success-hover, #43a047);
  }
  
  .run-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .save-btn {
    background: var(--bg-button, #3c3c3c);
    color: var(--text-primary, #d4d4d4);
  }
  
  .save-btn.unsaved {
    background: var(--warning-bg, #ff9800);
    color: black;
  }
  
  .spinner {
    width: 12px;
    height: 12px;
    border: 2px solid rgba(255,255,255,0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  .editor-container {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  
  .editor-wrapper {
    flex: 1;
    display: flex;
    overflow: auto;
  }
  
  .line-numbers {
    display: flex;
    flex-direction: column;
    padding: 12px 8px;
    background: var(--bg-secondary, #252526);
    color: var(--text-secondary, #858585);
    font-size: 14px;
    line-height: 1.5;
    text-align: right;
    user-select: none;
    min-width: 40px;
  }
  
  .code-editor {
    flex: 1;
    padding: 12px;
    background: var(--bg-primary, #1e1e1e);
    border: none;
    outline: none;
    color: var(--text-primary, #d4d4d4);
    font-family: inherit;
    line-height: 1.5;
    resize: none;
    tab-size: 4;
  }
  
  .console {
    height: 150px;
    display: flex;
    flex-direction: column;
    border-top: 1px solid var(--border-color, #3c3c3c);
  }
  
  .console-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: var(--bg-secondary, #252526);
    font-size: 12px;
    font-weight: 600;
  }
  
  .clear-btn {
    padding: 2px 8px;
    font-size: 11px;
    background: transparent;
    border: 1px solid var(--border-color, #5a5a5a);
    color: var(--text-secondary, #858585);
    border-radius: 3px;
    cursor: pointer;
  }
  
  .console-output {
    flex: 1;
    overflow-y: auto;
    padding: 8px 12px;
    font-size: 12px;
    background: var(--bg-console, #1a1a1a);
  }
  
  .console-line {
    padding: 2px 0;
    white-space: pre-wrap;
    word-break: break-all;
  }
  
  .console-line.error {
    color: var(--error-color, #f44336);
  }
  
  .api-reference {
    border-top: 1px solid var(--border-color, #3c3c3c);
  }
  
  .api-reference summary {
    padding: 8px 12px;
    background: var(--bg-secondary, #252526);
    cursor: pointer;
    font-size: 12px;
    font-weight: 600;
  }
  
  .api-content {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 16px;
    padding: 12px;
    background: var(--bg-tertiary, #2d2d2d);
    max-height: 200px;
    overflow-y: auto;
  }
  
  .api-section h4 {
    margin: 0 0 8px 0;
    font-size: 12px;
    color: var(--accent-color, #569cd6);
  }
  
  .api-section code {
    display: block;
    padding: 2px 0;
    font-size: 11px;
    color: var(--text-secondary, #9cdcfe);
  }
</style>
