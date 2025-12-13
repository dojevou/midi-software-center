<script lang="ts">
  import { createEventDispatcher, onDestroy, onMount } from 'svelte';

  // ============================================================================
  // PROPS
  // ============================================================================

  /** Array of octave numbers to display (e.g., [3, 4, 5] shows C3-B5) */
  export let octaves: number[] = [3, 4, 5];

  /** MIDI notes to highlight (e.g., scale notes, chord tones) */
  export let highlightedNotes: number[] = [];

  /** Currently playing/active MIDI notes */
  export let activeNotes: number[] = [];

  /** Show note names on white keys */
  export let showNoteNames: boolean = true;

  /** Width of each white key in pixels */
  export let keyWidth: number = 24;

  /** Height of keys in pixels */
  export let keyHeight: number = 100;

  /** Enable computer keyboard input (QWERTY → piano) */
  export let enableKeyboardInput: boolean = true;

  /** Base octave for computer keyboard input */
  export let keyboardOctave: number = 4;

  /** Default velocity for keyboard input (0-1) */
  export let defaultVelocity: number = 0.8;

  /** Enable touch input for mobile devices */
  export let enableTouch: boolean = true;

  /** Show octave shift controls */
  export let showOctaveControls: boolean = true;

  /** Minimum octave for keyboard input */
  export let minOctave: number = 0;

  /** Maximum octave for keyboard input */
  export let maxOctave: number = 8;

  /** Enable velocity sensitivity (position-based velocity) */
  export let velocitySensitive: boolean = true;

  /** Custom highlight color (CSS color string) */
  export let highlightColor: string = '#3b82f6';

  /** Custom active color (CSS color string) */
  export let activeColor: string = '#60a5fa';

  /** Disabled state */
  export let disabled: boolean = false;

  // ============================================================================
  // EVENTS
  // ============================================================================

  const dispatch = createEventDispatcher<{
    noteOn: { note: number; velocity: number };
    noteOff: { note: number };
    octaveChange: { octave: number };
  }>();

  // ============================================================================
  // CONSTANTS
  // ============================================================================

  const whiteKeyPattern = [0, 2, 4, 5, 7, 9, 11]; // C, D, E, F, G, A, B
  const blackKeyPattern = [1, 3, 6, 8, 10]; // C#, D#, F#, G#, A#
  const noteNames = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

  // Computer keyboard → MIDI note offset mapping (QWERTY layout)
  // Lower row: Z-M = white keys, S-K = black keys (one octave)
  // Upper row: Q-U = white keys, 2-7 = black keys (one octave higher)
  const keyboardMapping: Record<string, number> = {
    // Lower octave (from keyboardOctave)
    z: 0, // C
    s: 1, // C#
    x: 2, // D
    d: 3, // D#
    c: 4, // E
    v: 5, // F
    g: 6, // F#
    b: 7, // G
    h: 8, // G#
    n: 9, // A
    j: 10, // A#
    m: 11, // B
    // Upper octave (keyboardOctave + 1)
    q: 12, // C
    '2': 13, // C#
    w: 14, // D
    '3': 15, // D#
    e: 16, // E
    r: 17, // F
    '5': 18, // F#
    t: 19, // G
    '6': 20, // G#
    y: 21, // A
    '7': 22, // A#
    u: 23, // B
    i: 24, // C (octave + 2)
  };

  // ============================================================================
  // STATE
  // ============================================================================

  let pressedKeys: Set<number> = new Set();
  let keyboardPressedKeys: Set<string> = new Set();
  let touchIdentifiers: Map<number, number> = new Map(); // touch.identifier → midiNote
  let containerElement: HTMLDivElement;

  // ============================================================================
  // COMPUTED
  // ============================================================================

  $: totalWhiteKeys = octaves.length * 7;
  $: totalWidth = totalWhiteKeys * keyWidth;
  $: currentKeyboardOctave = keyboardOctave;

  // ============================================================================
  // UTILITY FUNCTIONS
  // ============================================================================

  function isBlackKey(noteInOctave: number): boolean {
    return blackKeyPattern.includes(noteInOctave);
  }

  function getMidiNote(octave: number, noteInOctave: number): number {
    return (octave + 1) * 12 + noteInOctave;
  }

  function getNoteName(midiNote: number): string {
    const noteInOctave = midiNote % 12;
    const octave = Math.floor(midiNote / 12) - 1;
    return `${noteNames[noteInOctave]}${octave}`;
  }

  function isKeyActive(midiNote: number): boolean {
    return activeNotes.includes(midiNote) || pressedKeys.has(midiNote);
  }

  function isKeyHighlighted(midiNote: number): boolean {
    return highlightedNotes.includes(midiNote);
  }

  function getWhiteKeyIndex(octave: number, noteInOctave: number): number {
    const octaveOffset = octaves.indexOf(octave) * 7;
    return octaveOffset + whiteKeyPattern.indexOf(noteInOctave);
  }

  function getBlackKeyPosition(octave: number, noteInOctave: number): number {
    const octaveOffset = octaves.indexOf(octave) * 7;
    const blackKeyOffsets: Record<number, number> = {
      1: 0.7, // C#
      3: 1.7, // D#
      6: 3.7, // F#
      8: 4.7, // G#
      10: 5.7, // A#
    };
    return (octaveOffset + blackKeyOffsets[noteInOctave]) * keyWidth;
  }

  function calculateVelocity(event: MouseEvent | Touch, element: HTMLElement): number {
    if (!velocitySensitive) {
      return defaultVelocity;
    }
    const rect = element.getBoundingClientRect();
    const clientY = event.clientY;
    const y = clientY - rect.top;
    // Velocity increases as you click lower on the key (like a real piano)
    return Math.min(1, Math.max(0.1, y / rect.height));
  }

  // ============================================================================
  // NOTE HANDLERS
  // ============================================================================

  function triggerNoteOn(midiNote: number, velocity: number): void {
    if (disabled || pressedKeys.has(midiNote)) {
      return;
    }

    pressedKeys.add(midiNote);
    pressedKeys = new Set(pressedKeys);
    dispatch('noteOn', { note: midiNote, velocity });
  }

  function triggerNoteOff(midiNote: number): void {
    if (!pressedKeys.has(midiNote)) {
      return;
    }

    pressedKeys.delete(midiNote);
    pressedKeys = new Set(pressedKeys);
    dispatch('noteOff', { note: midiNote });
  }

  // ============================================================================
  // MOUSE HANDLERS
  // ============================================================================

  function handleMouseDown(midiNote: number, event: MouseEvent): void {
    if (disabled) {
      return;
    }
    event.preventDefault();
    const velocity = calculateVelocity(event, event.target as HTMLElement);
    triggerNoteOn(midiNote, velocity);
  }

  function handleMouseUp(midiNote: number): void {
    triggerNoteOff(midiNote);
  }

  function handleMouseLeave(midiNote: number): void {
    if (pressedKeys.has(midiNote)) {
      triggerNoteOff(midiNote);
    }
  }

  function handleMouseEnter(midiNote: number, event: MouseEvent): void {
    // Support dragging across keys while mouse is held down
    if (event.buttons === 1 && !disabled) {
      const velocity = calculateVelocity(event, event.target as HTMLElement);
      triggerNoteOn(midiNote, velocity);
    }
  }

  // ============================================================================
  // TOUCH HANDLERS
  // ============================================================================

  function handleTouchStart(event: TouchEvent): void {
    if (!enableTouch || disabled) {
      return;
    }
    event.preventDefault();

    for (const touch of Array.from(event.changedTouches)) {
      const element = document.elementFromPoint(touch.clientX, touch.clientY) as HTMLElement;
      const midiNote = element?.dataset?.midiNote;

      if (midiNote) {
        const note = parseInt(midiNote, 10);
        const velocity = calculateVelocity(touch, element);
        touchIdentifiers.set(touch.identifier, note);
        triggerNoteOn(note, velocity);
      }
    }
    touchIdentifiers = new Map(touchIdentifiers);
  }

  function handleTouchMove(event: TouchEvent): void {
    if (!enableTouch || disabled) {
      return;
    }
    event.preventDefault();

    for (const touch of Array.from(event.changedTouches)) {
      const currentNote = touchIdentifiers.get(touch.identifier);
      const element = document.elementFromPoint(touch.clientX, touch.clientY) as HTMLElement;
      const midiNoteStr = element?.dataset?.midiNote;

      if (midiNoteStr) {
        const newNote = parseInt(midiNoteStr, 10);

        if (currentNote !== undefined && currentNote !== newNote) {
          // Moved to a different key
          triggerNoteOff(currentNote);
          const velocity = calculateVelocity(touch, element);
          triggerNoteOn(newNote, velocity);
          touchIdentifiers.set(touch.identifier, newNote);
        }
      } else if (currentNote !== undefined) {
        // Moved off all keys
        triggerNoteOff(currentNote);
        touchIdentifiers.delete(touch.identifier);
      }
    }
    touchIdentifiers = new Map(touchIdentifiers);
  }

  function handleTouchEnd(event: TouchEvent): void {
    if (!enableTouch || disabled) {
      return;
    }
    event.preventDefault();

    for (const touch of Array.from(event.changedTouches)) {
      const midiNote = touchIdentifiers.get(touch.identifier);
      if (midiNote !== undefined) {
        triggerNoteOff(midiNote);
        touchIdentifiers.delete(touch.identifier);
      }
    }
    touchIdentifiers = new Map(touchIdentifiers);
  }

  // ============================================================================
  // KEYBOARD HANDLERS
  // ============================================================================

  function handleKeyboardDown(event: KeyboardEvent): void {
    if (!enableKeyboardInput || disabled) {
      return;
    }

    const key = event.key.toLowerCase();

    // Octave shift controls
    if (key === 'z' && event.shiftKey) {
      shiftOctaveDown();
      return;
    }
    if (key === 'x' && event.shiftKey) {
      shiftOctaveUp();
      return;
    }

    // Note input
    if (keyboardMapping[key] !== undefined && !keyboardPressedKeys.has(key)) {
      event.preventDefault();
      keyboardPressedKeys.add(key);
      keyboardPressedKeys = new Set(keyboardPressedKeys);

      const noteOffset = keyboardMapping[key];
      const midiNote = (currentKeyboardOctave + 1) * 12 + noteOffset;

      triggerNoteOn(midiNote, defaultVelocity);
    }
  }

  function handleKeyboardUp(event: KeyboardEvent): void {
    if (!enableKeyboardInput || disabled) {
      return;
    }

    const key = event.key.toLowerCase();

    if (keyboardMapping[key] !== undefined && keyboardPressedKeys.has(key)) {
      event.preventDefault();
      keyboardPressedKeys.delete(key);
      keyboardPressedKeys = new Set(keyboardPressedKeys);

      const noteOffset = keyboardMapping[key];
      const midiNote = (currentKeyboardOctave + 1) * 12 + noteOffset;

      triggerNoteOff(midiNote);
    }
  }

  // ============================================================================
  // OCTAVE CONTROLS
  // ============================================================================

  function shiftOctaveDown(): void {
    if (currentKeyboardOctave > minOctave) {
      keyboardOctave = currentKeyboardOctave - 1;
      dispatch('octaveChange', { octave: keyboardOctave });
    }
  }

  function shiftOctaveUp(): void {
    if (currentKeyboardOctave < maxOctave) {
      keyboardOctave = currentKeyboardOctave + 1;
      dispatch('octaveChange', { octave: keyboardOctave });
    }
  }

  // ============================================================================
  // LIFECYCLE
  // ============================================================================

  onMount(() => {
    if (enableKeyboardInput) {
      window.addEventListener('keydown', handleKeyboardDown);
      window.addEventListener('keyup', handleKeyboardUp);
    }
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeyboardDown);
    window.removeEventListener('keyup', handleKeyboardUp);

    // Release all pressed keys on destroy
    for (const note of pressedKeys) {
      dispatch('noteOff', { note });
    }
  });

  // Cleanup keyboard listeners when disabled changes
  $: if (!enableKeyboardInput) {
    // Release all keyboard-triggered notes
    for (const key of keyboardPressedKeys) {
      const noteOffset = keyboardMapping[key];
      if (noteOffset !== undefined) {
        const midiNote = (currentKeyboardOctave + 1) * 12 + noteOffset;
        triggerNoteOff(midiNote);
      }
    }
    keyboardPressedKeys = new Set();
  }
</script>

<div
  class="virtual-keyboard-container flex flex-col gap-2"
  class:opacity-50={disabled}
  class:pointer-events-none={disabled}
>
  <!-- Octave Controls -->
  {#if showOctaveControls && enableKeyboardInput}
    <div class="octave-controls flex items-center justify-center gap-4 text-sm">
      <button
        type="button"
        class="octave-btn px-3 py-1 rounded bg-gray-700 hover:bg-gray-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        on:click={shiftOctaveDown}
        disabled={currentKeyboardOctave <= minOctave || disabled}
        aria-label="Shift octave down"
      >
        <span class="text-lg">-</span>
      </button>
      <span class="octave-display font-mono text-gray-300 min-w-[80px] text-center">
        Octave: {currentKeyboardOctave}
      </span>
      <button
        type="button"
        class="octave-btn px-3 py-1 rounded bg-gray-700 hover:bg-gray-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        on:click={shiftOctaveUp}
        disabled={currentKeyboardOctave >= maxOctave || disabled}
        aria-label="Shift octave up"
      >
        <span class="text-lg">+</span>
      </button>
    </div>
  {/if}

  <!-- Keyboard -->
  <div
    bind:this={containerElement}
    class="virtual-keyboard relative select-none touch-none"
    style="width: {totalWidth}px; height: {keyHeight}px; --highlight-color: {highlightColor}; --active-color: {activeColor};"
    role="application"
    aria-label="Virtual Piano Keyboard"
    aria-describedby="keyboard-instructions"
    on:touchstart={handleTouchStart}
    on:touchmove={handleTouchMove}
    on:touchend={handleTouchEnd}
    on:touchcancel={handleTouchEnd}
  >
    <!-- Screen reader instructions -->
    <div id="keyboard-instructions" class="sr-only">
      Use Z through M keys for lower octave notes, Q through U for upper octave. Shift+Z to decrease
      octave, Shift+X to increase octave.
    </div>

    <!-- White keys -->
    {#each octaves as octave (octave)}
      {#each whiteKeyPattern as noteInOctave (noteInOctave)}
        {@const midiNote = getMidiNote(octave, noteInOctave)}
        {@const keyIndex = getWhiteKeyIndex(octave, noteInOctave)}
        {@const active = isKeyActive(midiNote)}
        {@const highlighted = isKeyHighlighted(midiNote)}
        <div
          class="white-key absolute top-0 border-r border-gray-300 dark:border-gray-600 cursor-pointer transition-all duration-75"
          class:key-active={active}
          class:key-highlighted={highlighted}
          style="left: {keyIndex * keyWidth}px; width: {keyWidth}px; height: {keyHeight}px;"
          data-midi-note={midiNote}
          on:mousedown={(e) => handleMouseDown(midiNote, e)}
          on:mouseup={() => handleMouseUp(midiNote)}
          on:mouseleave={() => handleMouseLeave(midiNote)}
          on:mouseenter={(e) => handleMouseEnter(midiNote, e)}
          role="button"
          tabindex={disabled ? -1 : 0}
          aria-label={getNoteName(midiNote)}
          aria-pressed={active}
          on:keydown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault();
              if (!active) {
                triggerNoteOn(midiNote, defaultVelocity);
              }
            }
          }}
          on:keyup={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault();
              triggerNoteOff(midiNote);
            }
          }}
        >
          {#if showNoteNames && noteInOctave === 0}
            <span
              class="note-label absolute bottom-1 left-1/2 -translate-x-1/2 text-xs text-gray-500 dark:text-gray-400 pointer-events-none"
            >
              C{octave}
            </span>
          {/if}
        </div>
      {/each}
    {/each}

    <!-- Black keys -->
    {#each octaves as octave (octave)}
      {#each blackKeyPattern as noteInOctave (noteInOctave)}
        {@const midiNote = getMidiNote(octave, noteInOctave)}
        {@const leftPos = getBlackKeyPosition(octave, noteInOctave)}
        {@const active = isKeyActive(midiNote)}
        {@const highlighted = isKeyHighlighted(midiNote)}
        <div
          class="black-key absolute top-0 cursor-pointer transition-all duration-75 z-10"
          class:key-active={active}
          class:key-highlighted={highlighted}
          style="left: {leftPos}px; width: {keyWidth * 0.6}px; height: {keyHeight * 0.65}px;"
          data-midi-note={midiNote}
          on:mousedown={(e) => handleMouseDown(midiNote, e)}
          on:mouseup={() => handleMouseUp(midiNote)}
          on:mouseleave={() => handleMouseLeave(midiNote)}
          on:mouseenter={(e) => handleMouseEnter(midiNote, e)}
          role="button"
          tabindex={disabled ? -1 : 0}
          aria-label={getNoteName(midiNote)}
          aria-pressed={active}
          on:keydown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault();
              if (!active) {
                triggerNoteOn(midiNote, defaultVelocity);
              }
            }
          }}
          on:keyup={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault();
              triggerNoteOff(midiNote);
            }
          }}
        ></div>
      {/each}
    {/each}
  </div>

  <!-- Keyboard shortcuts hint -->
  {#if enableKeyboardInput && showOctaveControls}
    <div class="keyboard-hint text-xs text-gray-500 text-center">
      <span>Keys: Z-M (lower) | Q-U (upper)</span>
      <span class="mx-2">|</span>
      <span>Shift+Z/X: Octave -/+</span>
    </div>
  {/if}
</div>

<style>
  .virtual-keyboard {
    background: linear-gradient(180deg, #e5e7eb 0%, #d1d5db 100%);
    border-radius: 0 0 4px 4px;
    box-shadow:
      0 4px 6px -1px rgba(0, 0, 0, 0.1),
      0 2px 4px -1px rgba(0, 0, 0, 0.06);
  }

  :global(.dark) .virtual-keyboard {
    background: linear-gradient(180deg, #374151 0%, #1f2937 100%);
  }

  .white-key {
    background: linear-gradient(180deg, #ffffff 0%, #f3f4f6 85%, #e5e7eb 100%);
    border-bottom-left-radius: 4px;
    border-bottom-right-radius: 4px;
    box-shadow: inset 0 -2px 4px rgba(0, 0, 0, 0.05);
  }

  .white-key:hover:not(.key-active) {
    background: linear-gradient(180deg, #f9fafb 0%, #e5e7eb 85%, #d1d5db 100%);
  }

  .white-key.key-active {
    background: linear-gradient(180deg, var(--active-color, #60a5fa) 0%, #3b82f6 100%);
    transform: translateY(2px);
    box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  .white-key.key-highlighted:not(.key-active) {
    box-shadow: inset 0 0 0 2px var(--highlight-color, #3b82f6);
  }

  .black-key {
    background: linear-gradient(180deg, #374151 0%, #1f2937 70%, #111827 100%);
    border-bottom-left-radius: 3px;
    border-bottom-right-radius: 3px;
    box-shadow:
      0 4px 6px rgba(0, 0, 0, 0.3),
      inset 0 -2px 4px rgba(255, 255, 255, 0.05);
  }

  .black-key:hover:not(.key-active) {
    background: linear-gradient(180deg, #4b5563 0%, #374151 70%, #1f2937 100%);
  }

  .black-key.key-active {
    background: linear-gradient(180deg, #1e40af 0%, #1e3a8a 100%);
    transform: translateY(1px);
    box-shadow:
      0 2px 4px rgba(0, 0, 0, 0.3),
      inset 0 2px 4px rgba(0, 0, 0, 0.3);
  }

  .black-key.key-highlighted:not(.key-active) {
    box-shadow:
      0 4px 6px rgba(0, 0, 0, 0.3),
      inset 0 0 0 2px var(--highlight-color, #3b82f6);
  }

  .white-key:focus-visible,
  .black-key:focus-visible {
    outline: 2px solid var(--highlight-color, #3b82f6);
    outline-offset: -2px;
    z-index: 20;
  }

  .octave-btn {
    font-weight: 600;
    min-width: 36px;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border-width: 0;
  }
</style>
