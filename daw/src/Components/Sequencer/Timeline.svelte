<script lang="ts">
  export let currentPosition: number;
  export let totalDuration: number;
  export let tempo: number;

  $: playheadPosition = totalDuration > 0
    ? (currentPosition / totalDuration) * 100
    : 0;

  function formatBar(seconds: number): string {
    const beatsPerBar = 4;
    const secondsPerBeat = 60 / tempo;
    const totalBeats = seconds / secondsPerBeat;
    const bar = Math.floor(totalBeats / beatsPerBar) + 1;
    const beat = Math.floor(totalBeats % beatsPerBar) + 1;
    return `${bar}:${beat}`;
  }

  // Generate markers every 4 beats
  function getMarkers(): number[] {
    const markers: number[] = [];
    const beatsPerBar = 4;
    const secondsPerBeat = 60 / tempo;
    const secondsPerBar = secondsPerBeat * beatsPerBar;

    for (let t = 0; t <= totalDuration; t += secondsPerBar) {
      markers.push(t);
    }

    return markers;
  }

  $: markers = getMarkers();
</script>

<div class="timeline">
  <div class="timeline-ruler">
    {#each markers as time}
      <div
        class="marker"
        style="left: {totalDuration > 0 ? (time / totalDuration) * 100 : 0}%"
      >
        <div class="marker-line"></div>
        <span class="marker-label">{formatBar(time)}</span>
      </div>
    {/each}
  </div>

  <div class="playhead" style="left: {playheadPosition}%"></div>
</div>

<style>
  .timeline {
    position: relative;
    height: 48px;
    background: #252525;
    border-bottom: 1px solid #3d3d3d;
    overflow: hidden;
  }

  .timeline-ruler {
    position: relative;
    height: 100%;
  }

  .marker {
    position: absolute;
    top: 0;
    height: 100%;
  }

  .marker-line {
    width: 1px;
    height: 12px;
    background: #4d4d4d;
  }

  .marker-label {
    display: block;
    margin-top: 2px;
    font-size: 10px;
    color: #808080;
    font-family: 'Courier New', monospace;
    user-select: none;
  }

  .playhead {
    position: absolute;
    top: 0;
    width: 2px;
    height: 100%;
    background: #4a9eff;
    box-shadow: 0 0 8px rgba(74, 158, 255, 0.6);
    z-index: 10;
    pointer-events: none;
    transition: left 0.1s linear;
  }

  .playhead::before {
    content: '';
    position: absolute;
    top: 0;
    left: -6px;
    width: 0;
    height: 0;
    border-left: 6px solid transparent;
    border-right: 6px solid transparent;
    border-top: 8px solid #4a9eff;
  }
</style>
