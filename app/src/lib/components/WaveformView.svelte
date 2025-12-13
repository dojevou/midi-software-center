<script lang="ts">
  import { onDestroy, onMount } from 'svelte';

  export let audioData: number[] = [];
  export let currentTime: number = 0;
  export let duration: number = 0;
  export let playing: boolean = false;
  export let height: number = 100;
  export let showControls: boolean = false;
  export let color: string = '#3b82f6';

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;
  const animationFrame: number | null = null;

  $: progress = duration > 0 ? (currentTime / duration) * 100 : 0;

  onMount(() => {
    if (canvas) {
      ctx = canvas.getContext('2d');
      drawWaveform();
    }
  });

  onDestroy(() => {
    if (animationFrame) {
      cancelAnimationFrame(animationFrame);
    }
  });

  function drawWaveform() {
    if (!ctx || !canvas) {
      return;
    }

    const width = canvas.width;
    const canvasHeight = canvas.height;
    const midY = canvasHeight / 2;

    ctx.clearRect(0, 0, width, canvasHeight);

    // Draw background grid
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
    ctx.lineWidth = 1;
    for (let i = 0; i < width; i += 50) {
      ctx.beginPath();
      ctx.moveTo(i, 0);
      ctx.lineTo(i, canvasHeight);
      ctx.stroke();
    }

    // Draw center line
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.2)';
    ctx.beginPath();
    ctx.moveTo(0, midY);
    ctx.lineTo(width, midY);
    ctx.stroke();

    // Draw waveform
    if (audioData.length > 0) {
      ctx.strokeStyle = color;
      ctx.lineWidth = 1;
      ctx.beginPath();

      const step = Math.ceil(audioData.length / width);
      for (let i = 0; i < width; i++) {
        const dataIndex = Math.floor(i * step);
        const value = audioData[dataIndex] || 0;
        const y = midY - value * midY;

        if (i === 0) {
          ctx.moveTo(i, y);
        } else {
          ctx.lineTo(i, y);
        }
      }
      ctx.stroke();

      // Mirror waveform
      ctx.beginPath();
      for (let i = 0; i < width; i++) {
        const dataIndex = Math.floor(i * step);
        const value = audioData[dataIndex] || 0;
        const y = midY + value * midY;

        if (i === 0) {
          ctx.moveTo(i, y);
        } else {
          ctx.lineTo(i, y);
        }
      }
      ctx.stroke();
    }

    // Draw playhead
    if (duration > 0) {
      const playheadX = (currentTime / duration) * width;
      ctx.strokeStyle = '#ef4444';
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(playheadX, 0);
      ctx.lineTo(playheadX, canvasHeight);
      ctx.stroke();
    }
  }

  $: {
    audioData;
    currentTime;
    if (ctx) {
      drawWaveform();
    }
  }

  function handleCanvasClick(event: MouseEvent) {
    if (!canvas || duration <= 0) {
      return;
    }

    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const seekTime = (x / canvas.width) * duration;

    // Dispatch seek event
    canvas.dispatchEvent(
      new CustomEvent('seek', {
        detail: { time: seekTime },
        bubbles: true,
      })
    );
  }
</script>

<div class="waveform-view" style="height: {height}px">
  <canvas
    bind:this={canvas}
    width={800}
    {height}
    class="w-full h-full dark:bg-menu rounded cursor-pointer"
    on:click={handleCanvasClick}
  ></canvas>

  {#if showControls}
    <div class="controls flex items-center gap-4 mt-2">
      <div class="time text-xs dark:text-gray-400">
        {Math.floor(currentTime / 60)}:{(Math.floor(currentTime) % 60).toString().padStart(2, '0')} /
        {Math.floor(duration / 60)}:{(Math.floor(duration) % 60).toString().padStart(2, '0')}
      </div>

      <div class="progress-bar flex-1 h-1 dark:bg-gray-700 rounded-full overflow-hidden">
        <div
          class="progress h-full dark:bg-primary transition-all"
          style="width: {progress}%"
        ></div>
      </div>

      <div class="status text-xs dark:text-gray-400">
        {playing ? '▶ Playing' : '⏸ Paused'}
      </div>
    </div>
  {/if}
</div>

<style>
  .waveform-view {
    position: relative;
  }

  canvas {
    image-rendering: pixelated;
  }
</style>
