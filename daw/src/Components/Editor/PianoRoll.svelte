<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let fileId: number;

  interface MidiNote {
    pitch: number;
    velocity: number;
    start_tick: number;
    duration_ticks: number;
  }

  let canvas: HTMLCanvasElement;
  let notes: MidiNote[] = [];
  let loading = true;

  const noteHeight = 10;
  const pixelsPerTick = 0.05;

  async function loadNotes() {
    loading = true;
    try {
      const pattern = await invoke<{ events: any[] }>('load_midi_pattern', { fileId });
      notes = pattern.events
        .filter((e: any) => e.event_type === 'NoteOn')
        .map((e: any) => ({
          pitch: e.note,
          velocity: e.velocity,
          start_tick: e.tick,
          duration_ticks: 100 // Simplified, would need note-off matching
        }));

      drawNotes();
    } catch (error) {
      console.error('Failed to load MIDI notes:', error);
    } finally {
      loading = false;
    }
  }

  function drawNotes() {
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.fillStyle = '#1a1a1a';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Draw piano keys background
    for (let pitch = 0; pitch < 128; pitch++) {
      const y = (127 - pitch) * noteHeight;
      const isBlackKey = [1, 3, 6, 8, 10].includes(pitch % 12);
      ctx.fillStyle = isBlackKey ? '#2d2d2d' : '#252525';
      ctx.fillRect(0, y, canvas.width, noteHeight);

      // Draw grid lines
      ctx.strokeStyle = '#3d3d3d';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(canvas.width, y);
      ctx.stroke();
    }

    // Draw notes
    notes.forEach(note => {
      const x = note.start_tick * pixelsPerTick;
      const y = (127 - note.pitch) * noteHeight;
      const width = note.duration_ticks * pixelsPerTick;

      // Note color based on velocity
      const alpha = note.velocity / 127;
      ctx.fillStyle = `rgba(74, 158, 255, ${0.5 + alpha * 0.5})`;
      ctx.fillRect(x, y + 1, width, noteHeight - 2);

      // Note border
      ctx.strokeStyle = '#4a9eff';
      ctx.lineWidth = 1;
      ctx.strokeRect(x, y + 1, width, noteHeight - 2);
    });
  }

  onMount(() => {
    loadNotes();

    // Redraw on resize
    const resizeObserver = new ResizeObserver(() => {
      if (canvas) {
        canvas.width = canvas.offsetWidth;
        canvas.height = canvas.offsetHeight;
        drawNotes();
      }
    });

    if (canvas) {
      resizeObserver.observe(canvas);
    }

    return () => resizeObserver.disconnect();
  });
</script>

<div class="piano-roll">
  {#if loading}
    <div class="loading-overlay">
      <div class="spinner"></div>
      <p>Loading MIDI data...</p>
    </div>
  {/if}

  <canvas
    bind:this={canvas}
    class="note-canvas"
  ></canvas>
</div>

<style>
  .piano-roll {
    position: relative;
    width: 100%;
    height: 400px;
    background: #1a1a1a;
    border: 1px solid #3d3d3d;
    border-radius: 8px;
    overflow: auto;
  }

  .loading-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    background: rgba(26, 26, 26, 0.9);
    z-index: 10;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid #3d3d3d;
    border-top-color: #4a9eff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .loading-overlay p {
    margin: 0;
    color: #b0b0b0;
    font-size: 14px;
  }

  .note-canvas {
    display: block;
    width: 100%;
    height: 1280px; /* 128 notes * 10px */
    image-rendering: crisp-edges;
  }
</style>
