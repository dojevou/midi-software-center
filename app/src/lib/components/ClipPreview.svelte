<script lang="ts">
  /**
   * ClipPreview - MPC 3.0 Style Clip Content Visualization
   *
   * Displays a visual preview of clip contents:
   * - MIDI clips: Note blocks (piano roll mini-view)
   * - Drum clips: Hit markers
   * - Audio clips: Waveform visualization
   *
   * Uses canvas for efficient rendering of potentially many notes.
   */

  import { onMount, afterUpdate } from 'svelte';
  import type { SequencerTrackType } from '$lib/types';
  import { getTrackTypeColor, hexToRgba } from '$lib/utils/trackColors';

  // Props
  export let trackType: SequencerTrackType = 'midi';
  export let notes: Array<{
    startTime: number;
    duration: number;
    pitch: number;
    velocity: number;
  }> = [];
  export let clipDuration: number = 4; // In beats
  export let width: number = 200;
  export let height: number = 40;
  export let showVelocity: boolean = true;

  // Canvas reference
  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;

  // Computed values
  $: color = getTrackTypeColor(trackType);
  $: noteColor = hexToRgba(color, 0.9);
  $: gridColor = hexToRgba('#ffffff', 0.1);

  // Render on mount and updates
  onMount(() => {
    ctx = canvas.getContext('2d');
    render();
  });

  afterUpdate(() => {
    render();
  });

  function render() {
    if (!ctx || !canvas) {
      return;
    }

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    // Set canvas size for high DPI
    const dpr = window.devicePixelRatio || 1;
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    ctx.scale(dpr, dpr);

    // Draw based on track type
    if (trackType === 'drum') {
      drawDrumPattern();
    } else if (trackType === 'audio') {
      drawWaveform();
    } else {
      drawNotes();
    }
  }

  function drawNotes() {
    if (!ctx || notes.length === 0) {
      drawEmptyState();
      return;
    }

    // Find pitch range for scaling
    const pitches = notes.map((n) => n.pitch);
    const minPitch = Math.min(...pitches);
    const maxPitch = Math.max(...pitches);
    const pitchRange = Math.max(maxPitch - minPitch, 12); // At least one octave

    // Draw grid lines (optional background)
    drawGrid();

    // Draw notes
    const padding = 2;
    const availableHeight = height - padding * 2;
    const noteHeight = Math.max(2, Math.min(6, availableHeight / pitchRange));

    notes.forEach((note) => {
      const x = (note.startTime / clipDuration) * width;
      const noteWidth = Math.max(2, (note.duration / clipDuration) * width);
      const y =
        padding + ((maxPitch - note.pitch) / pitchRange) * (availableHeight - noteHeight);

      // Velocity affects opacity if showVelocity is true
      const opacity = showVelocity ? 0.4 + (note.velocity / 127) * 0.6 : 0.9;

      ctx!.fillStyle = hexToRgba(color, opacity);
      ctx!.fillRect(x, y, noteWidth - 1, noteHeight);
    });
  }

  function drawDrumPattern() {
    if (!ctx || notes.length === 0) {
      drawEmptyState();
      return;
    }

    // Group notes by approximate pitch (drum lanes)
    const lanes = new Map<number, typeof notes>();
    notes.forEach((note) => {
      // Quantize to drum lanes (kick, snare, hihat, etc.)
      const lane = Math.floor(note.pitch / 12); // Group by octave
      if (!lanes.has(lane)) {
        lanes.set(lane, []);
      }
      lanes.get(lane)!.push(note);
    });

    const laneCount = Math.max(lanes.size, 4);
    const laneHeight = height / laneCount;
    const hitRadius = Math.min(laneHeight / 2 - 2, 4);

    // Draw lane separators
    ctx.strokeStyle = gridColor;
    ctx.lineWidth = 0.5;
    for (let i = 1; i < laneCount; i++) {
      ctx.beginPath();
      ctx.moveTo(0, i * laneHeight);
      ctx.lineTo(width, i * laneHeight);
      ctx.stroke();
    }

    // Draw hits
    let laneIndex = 0;
    lanes.forEach((laneNotes) => {
      const y = laneIndex * laneHeight + laneHeight / 2;

      laneNotes.forEach((note) => {
        const x = (note.startTime / clipDuration) * width;
        const opacity = showVelocity ? 0.5 + (note.velocity / 127) * 0.5 : 0.9;

        ctx!.fillStyle = hexToRgba(color, opacity);
        ctx!.beginPath();
        ctx!.arc(x, y, hitRadius, 0, Math.PI * 2);
        ctx!.fill();
      });

      laneIndex++;
    });
  }

  function drawWaveform() {
    if (!ctx) {
      return;
    }

    // Simulated waveform (in real implementation, this would use actual audio data)
    // For now, draw a stylized waveform pattern
    ctx.fillStyle = hexToRgba(color, 0.6);

    const segments = Math.floor(width / 3);
    const centerY = height / 2;

    for (let i = 0; i < segments; i++) {
      const x = (i / segments) * width;
      const segmentWidth = width / segments - 1;

      // Generate pseudo-random heights based on position
      const seed = (i * 7919) % 100;
      const heightPercent = 0.3 + (seed / 100) * 0.5;
      const barHeight = height * heightPercent;

      ctx.fillRect(x, centerY - barHeight / 2, segmentWidth, barHeight);
    }
  }

  function drawGrid() {
    if (!ctx) {
      return;
    }

    ctx.strokeStyle = gridColor;
    ctx.lineWidth = 0.5;

    // Draw beat lines
    const beatsPerClip = Math.ceil(clipDuration);
    for (let i = 1; i < beatsPerClip; i++) {
      const x = (i / clipDuration) * width;
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, height);
      ctx.stroke();
    }
  }

  function drawEmptyState() {
    if (!ctx) {
      return;
    }

    // Draw subtle pattern indicating empty clip
    ctx.fillStyle = hexToRgba(color, 0.1);
    ctx.fillRect(0, 0, width, height);

    // Diagonal lines
    ctx.strokeStyle = hexToRgba(color, 0.2);
    ctx.lineWidth = 1;

    for (let i = -height; i < width + height; i += 12) {
      ctx.beginPath();
      ctx.moveTo(i, 0);
      ctx.lineTo(i + height, height);
      ctx.stroke();
    }
  }
</script>

<canvas
  bind:this={canvas}
  class="clip-preview"
  style:width="{width}px"
  style:height="{height}px"
  aria-label="Clip preview showing {notes.length} notes"
></canvas>

<style>
  .clip-preview {
    display: block;
    border-radius: 2px;
    pointer-events: none;
  }
</style>
