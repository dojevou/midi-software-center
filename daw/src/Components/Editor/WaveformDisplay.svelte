<script lang="ts">
  export let audioData: Float32Array = new Float32Array(0);
  export let width: number = 800;
  export let height: number = 100;

  let canvas: HTMLCanvasElement;

  function drawWaveform() {
    if (!canvas || audioData.length === 0) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear
    ctx.fillStyle = '#1a1a1a';
    ctx.fillRect(0, 0, width, height);

    // Draw waveform
    ctx.strokeStyle = '#4a9eff';
    ctx.lineWidth = 2;
    ctx.beginPath();

    const step = Math.ceil(audioData.length / width);
    const amp = height / 2;

    for (let i = 0; i < width; i++) {
      const index = i * step;
      const value = audioData[index] || 0;
      const y = amp + value * amp;

      if (i === 0) {
        ctx.moveTo(i, y);
      } else {
        ctx.lineTo(i, y);
      }
    }

    ctx.stroke();

    // Draw center line
    ctx.strokeStyle = '#3d3d3d';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, amp);
    ctx.lineTo(width, amp);
    ctx.stroke();
  }

  $: if (canvas && audioData.length > 0) {
    drawWaveform();
  }
</script>

<canvas
  bind:this={canvas}
  {width}
  {height}
  class="waveform-canvas"
></canvas>

<style>
  .waveform-canvas {
    display: block;
    width: 100%;
    background: #1a1a1a;
    border: 1px solid #3d3d3d;
    border-radius: 6px;
  }
</style>
