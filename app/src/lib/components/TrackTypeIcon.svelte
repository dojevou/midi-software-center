<script lang="ts">
  /**
   * TrackTypeIcon - MPC 3.0 Style Track Type Indicator
   *
   * Displays a color-coded icon indicating the track type
   * (MIDI, Drum, Audio, Bus) with optional tooltip.
   */

  import type { SequencerTrackType } from '$lib/types';
  import { TRACK_TYPE_ICONS, TRACK_TYPE_COLORS } from '$lib/types';
  import { getTrackTypeColor, getContrastingTextColor } from '$lib/utils/trackColors';

  // Props
  export let trackType: SequencerTrackType = 'midi';
  export let size: 'sm' | 'md' | 'lg' = 'md';
  export let showTooltip: boolean = true;
  export let showBackground: boolean = true;
  export let interactive: boolean = false;

  // Computed values
  $: icon = TRACK_TYPE_ICONS[trackType];
  $: color = getTrackTypeColor(trackType);
  $: textColor = getContrastingTextColor(color);
  $: tooltipText = getTooltipText(trackType);

  function getTooltipText(type: SequencerTrackType): string {
    switch (type) {
      case 'midi':
        return 'MIDI Track';
      case 'drum':
        return 'Drum Track';
      case 'audio':
        return 'Audio Track';
      case 'bus':
        return 'Bus Track';
      default:
        return 'Track';
    }
  }

  // Size classes
  const sizeClasses = {
    sm: 'track-type-icon--sm',
    md: 'track-type-icon--md',
    lg: 'track-type-icon--lg',
  };
</script>

<div
  class="track-type-icon {sizeClasses[size]}"
  class:track-type-icon--with-bg={showBackground}
  class:track-type-icon--interactive={interactive}
  style:--track-color={color}
  style:--track-text-color={textColor}
  title={showTooltip ? tooltipText : undefined}
  role={interactive ? 'button' : 'img'}
  aria-label={tooltipText}
  tabindex={interactive ? 0 : -1}
>
  <span class="track-type-icon__emoji">{icon}</span>
</div>

<style>
  .track-type-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    user-select: none;
    flex-shrink: 0;
  }

  .track-type-icon--with-bg {
    background-color: var(--track-color);
    color: var(--track-text-color);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  }

  .track-type-icon--interactive {
    cursor: pointer;
    transition:
      transform 0.1s ease,
      box-shadow 0.1s ease;
  }

  .track-type-icon--interactive:hover {
    transform: scale(1.1);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
  }

  .track-type-icon--interactive:active {
    transform: scale(0.95);
  }

  .track-type-icon--interactive:focus-visible {
    outline: 2px solid var(--track-color);
    outline-offset: 2px;
  }

  /* Size variants */
  .track-type-icon--sm {
    width: 20px;
    height: 20px;
    font-size: 12px;
  }

  .track-type-icon--md {
    width: 28px;
    height: 28px;
    font-size: 16px;
  }

  .track-type-icon--lg {
    width: 36px;
    height: 36px;
    font-size: 20px;
  }

  .track-type-icon__emoji {
    line-height: 1;
  }
</style>
