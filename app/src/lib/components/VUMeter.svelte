<script lang="ts">
  import { onDestroy, onMount } from 'svelte';

  // ============================================================================
  // TYPES
  // ============================================================================

  type MeterSize = 'xs' | 'sm' | 'md' | 'lg' | 'xl';
  type MeterOrientation = 'horizontal' | 'vertical';
  type MeterStyle = 'default' | 'segmented' | 'gradient' | 'vintage' | 'minimal';
  type MeterScale = 'linear' | 'logarithmic';

  interface MeterRange {
    min: number;
    max: number;
    label?: string;
    color: string;
  }

  // ============================================================================
  // PROPS
  // ============================================================================

  /** Current level (0-1 normalized, or dB if using scale='logarithmic') */
  export let level: number = 0;
  /** Peak level for peak hold indicator */
  export let peak: number | null = null;
  /** Stereo right channel level (when provided, meter becomes stereo) */
  export let levelRight: number | null = null;
  /** Stereo right channel peak */
  export let peakRight: number | null = null;
  /** Minimum value (in dB for logarithmic scale) */
  export let min: number = -60;
  /** Maximum value (in dB for logarithmic scale) */
  export let max: number = 6;
  /** Warning threshold (yellow zone starts) */
  export let warningLevel: number = -12;
  /** Danger threshold (red zone starts) */
  export let dangerLevel: number = -6;
  /** Clip threshold */
  export let clipLevel: number = 0;
  /** Size variant */
  export let size: MeterSize = 'md';
  /** Orientation */
  export let orientation: MeterOrientation = 'vertical';
  /** Visual style */
  export let style: MeterStyle = 'default';
  /** Scale type */
  export let scale: MeterScale = 'logarithmic';
  /** Show scale labels */
  export let showScale: boolean = true;
  /** Show peak indicator */
  export let showPeak: boolean = true;
  /** Peak hold time in ms (0 = infinite) */
  export let peakHoldTime: number = 2000;
  /** Peak decay rate (per second) */
  export let peakDecayRate: number = 20;
  /** Number of segments (for segmented style) */
  export let segments: number = 20;
  /** Show clip indicator */
  export let showClip: boolean = true;
  /** Auto-decay peak when not updating */
  export let autoPeakDecay: boolean = true;
  /** Label text */
  export let label: string = '';
  /** Custom ID */
  export let id: string = `meter-${Math.random().toString(36).substr(2, 9)}`;
  /** Custom color ranges */
  export let ranges: MeterRange[] = [];
  /** Smoothing factor (0-1, higher = more smoothing) */
  export let smoothing: number = 0.3;

  // ============================================================================
  // STATE
  // ============================================================================

  let smoothedLevel = 0;
  let smoothedLevelRight = 0;
  let internalPeak = 0;
  let internalPeakRight = 0;
  let peakHoldTimer: number | null = null;
  let peakHoldTimerRight: number | null = null;
  let isClipping = false;
  let isClippingRight = false;
  let lastUpdateTime = 0;
  let animationFrameId: number | null = null;

  // ============================================================================
  // COMPUTED VALUES
  // ============================================================================

  $: isStereo = levelRight !== null;
  $: isVertical = orientation === 'vertical';

  $: sizeConfig = {
    xs: { width: 6, height: 60, labelSize: 8 },
    sm: { width: 10, height: 100, labelSize: 9 },
    md: { width: 14, height: 140, labelSize: 10 },
    lg: { width: 20, height: 200, labelSize: 12 },
    xl: { width: 28, height: 280, labelSize: 14 },
  }[size];

  $: meterWidth = isVertical
    ? isStereo
      ? sizeConfig.width * 2 + 4
      : sizeConfig.width
    : sizeConfig.height;

  $: meterHeight = isVertical ? sizeConfig.height : sizeConfig.width;

  $: defaultRanges =
    ranges.length > 0
      ? ranges
      : [
          { min: min, max: warningLevel, color: 'var(--meter-green, #22c55e)' },
          { min: warningLevel, max: dangerLevel, color: 'var(--meter-yellow, #eab308)' },
          { min: dangerLevel, max: clipLevel, color: 'var(--meter-orange, #f97316)' },
          { min: clipLevel, max: max, color: 'var(--meter-red, #ef4444)' },
        ];

  $: scaleMarks = generateScaleMarks();

  // Normalized levels (0-1)
  $: normalizedLevel = normalizeValue(smoothedLevel);
  $: normalizedLevelRight = normalizeValue(smoothedLevelRight);
  $: normalizedPeak = normalizeValue(internalPeak);
  $: normalizedPeakRight = normalizeValue(internalPeakRight);

  // Percentage for CSS
  $: levelPercentage = normalizedLevel * 100;
  $: levelRightPercentage = normalizedLevelRight * 100;
  $: peakPercentage = normalizedPeak * 100;
  $: peakRightPercentage = normalizedPeakRight * 100;

  // ============================================================================
  // SCALE FUNCTIONS
  // ============================================================================

  function normalizeValue(value: number): number {
    if (scale === 'linear') {
      return Math.max(0, Math.min(1, (value - min) / (max - min)));
    }

    // Logarithmic (dB) scale
    const clampedValue = Math.max(min, Math.min(max, value));
    return (clampedValue - min) / (max - min);
  }

  function generateScaleMarks(): { value: number; label: string; position: number }[] {
    const marks: { value: number; label: string; position: number }[] = [];
    const steps = [0, -3, -6, -12, -18, -24, -36, -48, -60];

    for (const step of steps) {
      if (step >= min && step <= max) {
        marks.push({
          value: step,
          label: step === 0 ? '0' : step.toString(),
          position: normalizeValue(step) * 100,
        });
      }
    }

    return marks;
  }

  // ============================================================================
  // LEVEL PROCESSING
  // ============================================================================

  function updateLevels(): void {
    const now = performance.now();
    const deltaTime = (now - lastUpdateTime) / 1000;
    lastUpdateTime = now;

    // Smooth level transitions
    smoothedLevel += (level - smoothedLevel) * (1 - smoothing);
    if (levelRight !== null) {
      smoothedLevelRight += (levelRight - smoothedLevelRight) * (1 - smoothing);
    }

    // Update peaks
    if (smoothedLevel >= internalPeak) {
      internalPeak = smoothedLevel;
      isClipping = smoothedLevel >= clipLevel;
      resetPeakHoldTimer('left');
    } else if (autoPeakDecay && peakHoldTimer === null) {
      internalPeak = Math.max(smoothedLevel, internalPeak - peakDecayRate * deltaTime);
    }

    if (isStereo && smoothedLevelRight >= internalPeakRight) {
      internalPeakRight = smoothedLevelRight;
      isClippingRight = smoothedLevelRight >= clipLevel;
      resetPeakHoldTimer('right');
    } else if (isStereo && autoPeakDecay && peakHoldTimerRight === null) {
      internalPeakRight = Math.max(
        smoothedLevelRight,
        internalPeakRight - peakDecayRate * deltaTime
      );
    }

    animationFrameId = requestAnimationFrame(updateLevels);
  }

  function resetPeakHoldTimer(channel: 'left' | 'right'): void {
    if (peakHoldTime === 0) {
      return;
    }

    if (channel === 'left') {
      if (peakHoldTimer !== null) {
        clearTimeout(peakHoldTimer);
      }
      peakHoldTimer = window.setTimeout(() => {
        peakHoldTimer = null;
      }, peakHoldTime);
    } else {
      if (peakHoldTimerRight !== null) {
        clearTimeout(peakHoldTimerRight);
      }
      peakHoldTimerRight = window.setTimeout(() => {
        peakHoldTimerRight = null;
      }, peakHoldTime);
    }
  }

  // ============================================================================
  // PUBLIC METHODS
  // ============================================================================

  export function resetPeak(): void {
    internalPeak = smoothedLevel;
    internalPeakRight = smoothedLevelRight;
    isClipping = false;
    isClippingRight = false;
  }

  export function clearClip(): void {
    isClipping = false;
    isClippingRight = false;
  }

  // ============================================================================
  // REACTIVE UPDATES
  // ============================================================================

  // Use external peak values if provided
  $: if (peak !== null) {
    internalPeak = peak;
    isClipping = peak >= clipLevel;
  }

  $: if (peakRight !== null) {
    internalPeakRight = peakRight;
    isClippingRight = peakRight >= clipLevel;
  }

  // ============================================================================
  // LIFECYCLE
  // ============================================================================

  onMount(() => {
    lastUpdateTime = performance.now();
    animationFrameId = requestAnimationFrame(updateLevels);
  });

  onDestroy(() => {
    if (animationFrameId !== null) {
      cancelAnimationFrame(animationFrameId);
    }
    if (peakHoldTimer !== null) {
      clearTimeout(peakHoldTimer);
    }
    if (peakHoldTimerRight !== null) {
      clearTimeout(peakHoldTimerRight);
    }
  });

  // ============================================================================
  // SEGMENT RENDERING
  // ============================================================================

  function getSegmentColor(segmentIndex: number, totalSegments: number): string {
    const segmentPosition = segmentIndex / totalSegments;
    const segmentDb = min + segmentPosition * (max - min);

    for (let i = defaultRanges.length - 1; i >= 0; i--) {
      if (segmentDb >= defaultRanges[i].min) {
        return defaultRanges[i].color;
      }
    }
    return defaultRanges[0].color;
  }

  function isSegmentActive(segmentIndex: number, totalSegments: number, levelPct: number): boolean {
    const segmentThreshold = ((segmentIndex + 1) / totalSegments) * 100;
    return levelPct >= segmentThreshold;
  }
</script>

<div
  {id}
  class="vu-meter"
  class:vertical={isVertical}
  class:horizontal={!isVertical}
  class:stereo={isStereo}
  class:clipping={isClipping || isClippingRight}
  style="
    --meter-width: {meterWidth}px;
    --meter-height: {meterHeight}px;
    --channel-width: {sizeConfig.width}px;
    --label-size: {sizeConfig.labelSize}px;
  "
  role="meter"
  aria-valuemin={min}
  aria-valuemax={max}
  aria-valuenow={level}
  aria-label={label || 'Level meter'}
>
  {#if label}
    <span class="meter-label">{label}</span>
  {/if}

  <div class="meter-container">
    <!-- Scale labels (left/bottom) -->
    {#if showScale}
      <div class="scale-labels">
        {#each scaleMarks as mark (mark.value)}
          <span
            class="scale-mark"
            style={isVertical ? `bottom: ${mark.position}%` : `left: ${mark.position}%`}
          >
            {mark.label}
          </span>
        {/each}
      </div>
    {/if}

    <div class="meter-body meter-style-{style}">
      <!-- Left/Main Channel -->
      <div class="channel left">
        {#if style === 'segmented'}
          <div class="segments">
            {#each Array(segments) as _, i (i)}
              {@const segmentIndex = segments - 1 - i}
              <div
                class="segment"
                class:active={isSegmentActive(segmentIndex, segments, levelPercentage)}
                style="--segment-color: {getSegmentColor(segmentIndex, segments)}"
              />
            {/each}
          </div>
        {:else}
          <!-- Gradient/Default fill -->
          <div class="meter-track">
            <div class="meter-fill" style="--level: {levelPercentage}%">
              {#if style === 'gradient'}
                <div class="gradient-fill" />
              {/if}
            </div>
          </div>
        {/if}

        <!-- Peak indicator -->
        {#if showPeak}
          <div
            class="peak-indicator"
            class:clipping={isClipping}
            style={isVertical ? `bottom: ${peakPercentage}%` : `left: ${peakPercentage}%`}
          />
        {/if}
      </div>

      <!-- Right Channel (stereo) -->
      {#if isStereo}
        <div class="channel right">
          {#if style === 'segmented'}
            <div class="segments">
              {#each Array(segments) as _, i (i)}
                {@const segmentIndex = segments - 1 - i}
                <div
                  class="segment"
                  class:active={isSegmentActive(segmentIndex, segments, levelRightPercentage)}
                  style="--segment-color: {getSegmentColor(segmentIndex, segments)}"
                />
              {/each}
            </div>
          {:else}
            <div class="meter-track">
              <div class="meter-fill" style="--level: {levelRightPercentage}%">
                {#if style === 'gradient'}
                  <div class="gradient-fill" />
                {/if}
              </div>
            </div>
          {/if}

          {#if showPeak}
            <div
              class="peak-indicator"
              class:clipping={isClippingRight}
              style={isVertical
                ? `bottom: ${peakRightPercentage}%`
                : `left: ${peakRightPercentage}%`}
            />
          {/if}
        </div>
      {/if}

      <!-- Clip indicators -->
      {#if showClip}
        <div class="clip-indicators">
          <button
            class="clip-indicator"
            class:active={isClipping}
            on:click={() => (isClipping = false)}
            title="Click to clear clip"
            aria-label="Clear left channel clip indicator"
          />
          {#if isStereo}
            <button
              class="clip-indicator"
              class:active={isClippingRight}
              on:click={() => (isClippingRight = false)}
              title="Click to clear clip"
              aria-label="Clear right channel clip indicator"
            />
          {/if}
        </div>
      {/if}
    </div>

    <!-- Scale ticks (right/top) -->
    {#if showScale}
      <div class="scale-ticks">
        {#each scaleMarks as mark (mark.value)}
          <div
            class="tick"
            style={isVertical ? `bottom: ${mark.position}%` : `left: ${mark.position}%`}
          />
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .vu-meter {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-family: 'SF Mono', 'Monaco', monospace;
  }

  .meter-label {
    font-size: var(--label-size);
    color: var(--text-secondary, #9ca3af);
    text-align: center;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .meter-container {
    display: flex;
    gap: 4px;
  }

  .vertical .meter-container {
    flex-direction: row;
    height: var(--meter-height);
  }

  .horizontal .meter-container {
    flex-direction: column;
    width: var(--meter-width);
  }

  .scale-labels {
    position: relative;
    display: flex;
  }

  .vertical .scale-labels {
    width: 24px;
    flex-direction: column;
  }

  .horizontal .scale-labels {
    height: 16px;
    flex-direction: row;
  }

  .scale-mark {
    position: absolute;
    font-size: calc(var(--label-size) - 2px);
    color: var(--text-secondary, #9ca3af);
    transform: translateY(50%);
  }

  .vertical .scale-mark {
    right: 0;
    text-align: right;
  }

  .horizontal .scale-mark {
    transform: translateX(-50%);
    bottom: 0;
  }

  .scale-ticks {
    position: relative;
    display: flex;
  }

  .vertical .scale-ticks {
    width: 6px;
    flex-direction: column;
  }

  .horizontal .scale-ticks {
    height: 6px;
    flex-direction: row;
  }

  .tick {
    position: absolute;
    background: var(--text-secondary, #9ca3af);
  }

  .vertical .tick {
    width: 100%;
    height: 1px;
    transform: translateY(50%);
  }

  .horizontal .tick {
    width: 1px;
    height: 100%;
    transform: translateX(-50%);
  }

  .meter-body {
    position: relative;
    display: flex;
    gap: 2px;
    background: var(--meter-bg, #111827);
    border-radius: 4px;
    padding: 2px;
    overflow: hidden;
  }

  .vertical .meter-body {
    flex-direction: row;
    width: var(--meter-width);
    height: 100%;
  }

  .horizontal .meter-body {
    flex-direction: column;
    width: 100%;
    height: var(--channel-width);
  }

  .channel {
    position: relative;
    flex: 1;
    border-radius: 2px;
    overflow: hidden;
  }

  .vertical .channel {
    width: var(--channel-width);
  }

  .horizontal .channel {
    height: var(--channel-width);
  }

  /* Default & Gradient style */
  .meter-track {
    position: absolute;
    inset: 0;
    background: var(--meter-track-bg, #1f2937);
    border-radius: inherit;
  }

  .meter-fill {
    position: absolute;
    border-radius: inherit;
    background: linear-gradient(
      to top,
      var(--meter-green, #22c55e) 0%,
      var(--meter-green, #22c55e) 60%,
      var(--meter-yellow, #eab308) 75%,
      var(--meter-orange, #f97316) 85%,
      var(--meter-red, #ef4444) 100%
    );
    transition: none;
  }

  .vertical .meter-fill {
    bottom: 0;
    left: 0;
    right: 0;
    height: var(--level);
  }

  .horizontal .meter-fill {
    top: 0;
    bottom: 0;
    left: 0;
    width: var(--level);
    background: linear-gradient(
      to right,
      var(--meter-green, #22c55e) 0%,
      var(--meter-green, #22c55e) 60%,
      var(--meter-yellow, #eab308) 75%,
      var(--meter-orange, #f97316) 85%,
      var(--meter-red, #ef4444) 100%
    );
  }

  /* Gradient style overlay */
  .gradient-fill {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      to top,
      transparent 0%,
      rgba(255, 255, 255, 0.1) 50%,
      rgba(255, 255, 255, 0.2) 100%
    );
  }

  .horizontal .gradient-fill {
    background: linear-gradient(
      to right,
      transparent 0%,
      rgba(255, 255, 255, 0.1) 50%,
      rgba(255, 255, 255, 0.2) 100%
    );
  }

  /* Segmented style */
  .segments {
    display: flex;
    gap: 1px;
    height: 100%;
    width: 100%;
  }

  .vertical .segments {
    flex-direction: column;
  }

  .horizontal .segments {
    flex-direction: row;
  }

  .segment {
    flex: 1;
    background: var(--segment-color);
    opacity: 0.2;
    border-radius: 1px;
    transition: opacity 0.05s ease;
  }

  .segment.active {
    opacity: 1;
  }

  /* Peak indicator */
  .peak-indicator {
    position: absolute;
    background: white;
    box-shadow: 0 0 4px rgba(255, 255, 255, 0.5);
    transition: none;
  }

  .vertical .peak-indicator {
    left: 0;
    right: 0;
    height: 2px;
    transform: translateY(50%);
  }

  .horizontal .peak-indicator {
    top: 0;
    bottom: 0;
    width: 2px;
    transform: translateX(-50%);
  }

  .peak-indicator.clipping {
    background: var(--meter-red, #ef4444);
    box-shadow: 0 0 8px var(--meter-red, #ef4444);
  }

  /* Clip indicators */
  .clip-indicators {
    position: absolute;
    display: flex;
    gap: 2px;
  }

  .vertical .clip-indicators {
    top: 2px;
    left: 2px;
    right: 2px;
    flex-direction: row;
  }

  .horizontal .clip-indicators {
    top: 2px;
    right: 2px;
    bottom: 2px;
    flex-direction: column;
  }

  .clip-indicator {
    flex: 1;
    height: 6px;
    border: none;
    border-radius: 2px;
    background: var(--meter-track-bg, #1f2937);
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .horizontal .clip-indicator {
    width: 6px;
    height: auto;
  }

  .clip-indicator.active {
    background: var(--meter-red, #ef4444);
    animation: clip-flash 0.5s ease-in-out infinite alternate;
  }

  @keyframes clip-flash {
    from {
      opacity: 1;
    }
    to {
      opacity: 0.5;
    }
  }

  /* Style variants */
  .style-minimal .meter-body {
    background: transparent;
    border: 1px solid var(--border-color, #374151);
  }

  .style-vintage .meter-body {
    background: #2a2520;
    border: 2px solid #4a4540;
  }

  .style-vintage .meter-fill {
    background: linear-gradient(to top, #22c55e 0%, #22c55e 70%, #f59e0b 90%, #dc2626 100%);
  }

  /* High contrast mode */
  @media (prefers-contrast: high) {
    .meter-body {
      border: 2px solid currentColor;
    }

    .segment,
    .meter-fill {
      outline: 1px solid currentColor;
    }
  }

  /* Reduced motion */
  @media (prefers-reduced-motion: reduce) {
    .clip-indicator.active {
      animation: none;
    }
  }
</style>
