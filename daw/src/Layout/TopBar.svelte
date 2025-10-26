<script lang="ts">
  import MidiConnectionStatus from '../Components/Midi/MidiConnectionStatus.svelte';
</script>

<header class="top-bar">
  <div class="left-section">
    <div class="app-icon">🎵</div>
    <h1 class="app-title">MIDI Library DAW</h1>
  </div>

  <div class="center-section">
    <!-- Future: Add menu items here -->
  </div>

  <div class="right-section">
    <MidiConnectionStatus />
  </div>
</header>

<style>
  .top-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 60px;
    padding: 0 20px;
    background: #252525;
    border-bottom: 1px solid #3d3d3d;
  }

  .left-section {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .app-icon {
    font-size: 28px;
  }

  .app-title {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: #e0e0e0;
  }

  .center-section {
    flex: 1;
  }

  .right-section {
    display: flex;
    align-items: center;
    gap: 12px;
  }
</style>
