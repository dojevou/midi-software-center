<script lang="ts">
  import { noteNameFromPitch } from '../../lib/trusty/notes';

  export let highlightedNote: number | undefined = undefined;
  export let selectedNotes: number[] = [];

  // MIDI pitch range for 88-key piano: A0 (21) to C8 (108)
  const MIN_PITCH = 21;
  const MAX_PITCH = 108;
  const TOTAL_KEYS = MAX_PITCH - MIN_PITCH + 1;

  // Pattern of white and black keys (repeat every 12 semitones)
  // 0=white, 1=black
  const keyPattern = [0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0]; // C, C#, D, D#, E, F, F#, G, G#, A, A#, B

  function getKeyType(pitch: number): 'white' | 'black' {
    const relativeNote = pitch % 12;
    return keyPattern[relativeNote] === 0 ? 'white' : 'black';
  }

  function getKeyName(pitch: number): string {
    return noteNameFromPitch(pitch);
  }

  function isKeyHighlighted(pitch: number): boolean {
    return pitch === highlightedNote;
  }

  function isKeySelected(pitch: number): boolean {
    return selectedNotes.includes(pitch);
  }

  function handleKeyClick(pitch: number) {
    dispatch('noteClick', pitch);
  }

  function handleKeyDoubleClick(pitch: number) {
    dispatch('noteDoubleClick', pitch);
  }

  import { createEventDispatcher } from 'svelte';
  const dispatch = createEventDispatcher<{
    noteClick: number;
    noteDoubleClick: number;
  }>();

  // Generate array of all pitches from MIN to MAX
  const keys = Array.from({ length: TOTAL_KEYS }, (_, i) => MIN_PITCH + i);
</script>

<div class="piano-keyboard">
  <div class="keys-container">
    {#each keys as pitch (pitch)}
      {@const keyType = getKeyType(pitch)}
      {@const keyName = getKeyName(pitch)}
      {@const isHighlighted = isKeyHighlighted(pitch)}
      {@const isSelected = isKeySelected(pitch)}

      <button
        class="key {keyType}"
        class:highlighted={isHighlighted}
        class:selected={isSelected}
        on:click={() => handleKeyClick(pitch)}
        on:dblclick={() => handleKeyDoubleClick(pitch)}
        aria-label="{keyName}"
        title={keyName}
      >
        {#if keyType === 'white'}
          <span class="key-label">{keyName}</span>
        {/if}
      </button>
    {/each}
  </div>
</div>

<style>
  .piano-keyboard {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    background-color: #1a1a1a;
    border: 1px solid #333;
    border-radius: 4px;
    padding: 8px;
    gap: 8px;
  }

  .keys-container {
    display: flex;
    gap: 0;
    justify-content: flex-start;
    align-items: flex-end;
    height: 100%;
    position: relative;
  }

  .key {
    cursor: pointer;
    border: 1px solid #333;
    border-radius: 4px;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    user-select: none;
    transition: all 0.1s ease;
    position: relative;
  }

  /* White keys */
  .key.white {
    flex: 0 0 auto;
    width: 40px;
    height: 120px;
    background-color: #f5f5f5;
    color: #000;
    border-color: #666;
  }

  .key.white:hover:not(.highlighted):not(.selected) {
    background-color: #e8e8e8;
  }

  .key.white.selected {
    background-color: #90ee90;
    box-shadow: inset 0 0 8px rgba(0, 150, 0, 0.4);
  }

  .key.white.highlighted {
    background-color: #ffd700;
    box-shadow: inset 0 0 12px rgba(255, 215, 0, 0.5), 0 0 12px rgba(255, 215, 0, 0.3);
  }

  .key.white.selected.highlighted {
    background-color: #ffed4e;
    box-shadow: inset 0 0 12px rgba(255, 215, 0, 0.6), 0 0 12px rgba(0, 150, 0, 0.3);
  }

  /* Black keys */
  .key.black {
    flex: 0 0 auto;
    width: 28px;
    height: 80px;
    background-color: #1a1a1a;
    color: #fff;
    border-color: #555;
    margin-left: -14px;
    margin-right: -14px;
    z-index: 10;
  }

  .key.black:hover:not(.highlighted):not(.selected) {
    background-color: #2a2a2a;
  }

  .key.black.selected {
    background-color: #006400;
    box-shadow: inset 0 0 8px rgba(0, 255, 0, 0.4), 0 0 12px rgba(0, 255, 0, 0.3);
  }

  .key.black.highlighted {
    background-color: #ff8c00;
    box-shadow: inset 0 0 12px rgba(255, 140, 0, 0.5), 0 0 12px rgba(255, 140, 0, 0.3);
  }

  .key.black.selected.highlighted {
    background-color: #ffa500;
    box-shadow: inset 0 0 12px rgba(255, 165, 0, 0.6), 0 0 12px rgba(255, 165, 0, 0.4);
  }

  /* Key label - only show on white keys */
  .key-label {
    font-size: 10px;
    font-weight: 600;
    margin-bottom: 6px;
    opacity: 0.7;
    transition: opacity 0.1s ease;
  }

  .key.white:hover .key-label {
    opacity: 1;
  }

  .key.white.selected .key-label,
  .key.white.highlighted .key-label {
    opacity: 1;
  }

  /* Focus outline for accessibility */
  .key:focus {
    outline: 2px solid #00aaff;
    outline-offset: -2px;
  }
</style>
