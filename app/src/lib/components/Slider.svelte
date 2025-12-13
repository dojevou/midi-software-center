<script lang="ts">
  import { createEventDispatcher, onDestroy, onMount } from 'svelte';

  // ============================================================================
  // TYPES
  // ============================================================================

  type SliderSize = 'xs' | 'sm' | 'md' | 'lg' | 'xl';
  type SliderOrientation = 'horizontal' | 'vertical';
  type SliderStyle = 'default' | 'minimal' | 'fader' | 'range';

  interface SliderMark {
    value: number;
    label?: string;
  }

  // ============================================================================
  // PROPS
  // ============================================================================

  /** Current value */
  export let value: number = 0;
  /** Minimum value */
  export let min: number = 0;
  /** Maximum value */
  export let max: number = 100;
  /** Step increment */
  export let step: number = 1;
  /** Default value for double-click reset */
  export let defaultValue: number | null = null;
  /** Label text */
  export let label: string = '';
  /** Unit suffix */
  export let unit: string = '';
  /** Size variant */
  export let size: SliderSize = 'md';
  /** Visual style */
  export let style: SliderStyle = 'default';
  /** Orientation */
  export let orientation: SliderOrientation = 'horizontal';
  /** Show value display */
  export let showValue: boolean = true;
  /** Value display position */
  export let valuePosition: 'top' | 'bottom' | 'left' | 'right' | 'tooltip' = 'tooltip';
  /** Show tick marks */
  export let showMarks: boolean = false;
  /** Custom marks */
  export let marks: SliderMark[] = [];
  /** Disabled state */
  export let disabled: boolean = false;
  /** Bipolar mode (center is zero) */
  export let bipolar: boolean = false;
  /** Custom value formatter */
  export let formatValue: ((value: number) => string) | null = null;
  /** Primary color */
  export let color: string = 'var(--slider-color, #3b82f6)';
  /** Track color */
  export let trackColor: string = 'var(--slider-track, #374151)';
  /** Fine control multiplier */
  export let fineControlMultiplier: number = 0.1;
  /** Custom ID */
  export let id: string = `slider-${Math.random().toString(36).substr(2, 9)}`;
  /** Fill mode: 'from-start', 'from-center', 'none' */
  export let fillMode: 'from-start' | 'from-center' | 'none' = 'from-start';
  /** Show current value on thumb */
  export let showThumbValue: boolean = false;

  // ============================================================================
  // STATE
  // ============================================================================

  let trackElement: HTMLDivElement;
  let isDragging = false;
  let isFineControl = false;
  let isHovered = false;
  let showTooltip = false;
  let touchId: number | null = null;

  // ============================================================================
  // DISPATCHER
  // ============================================================================

  const dispatch = createEventDispatcher<{
    change: { value: number; previousValue: number };
    input: { value: number };
    dragStart: { value: number };
    dragEnd: { value: number };
  }>();

  // ============================================================================
  // COMPUTED VALUES
  // ============================================================================

  $: normalizedValue = (value - min) / (max - min);
  $: percentage = normalizedValue * 100;
  $: displayValue = formatValue ? formatValue(value) : formatDefaultValue(value);
  $: isVertical = orientation === 'vertical';

  $: sizeConfig = {
    xs: { trackSize: 4, thumbSize: 12, length: 80 },
    sm: { trackSize: 6, thumbSize: 16, length: 120 },
    md: { trackSize: 8, thumbSize: 20, length: 160 },
    lg: { trackSize: 10, thumbSize: 24, length: 200 },
    xl: { trackSize: 12, thumbSize: 32, length: 280 },
  }[size];

  $: computedMarks = marks.length > 0 ? marks : showMarks ? generateDefaultMarks() : [];

  $: fillStart = bipolar || fillMode === 'from-center' ? Math.min(50, percentage) : 0;

  $: fillEnd = bipolar || fillMode === 'from-center' ? Math.max(50, percentage) : percentage;

  $: fillWidth = fillEnd - fillStart;

  // ============================================================================
  // VALUE FORMATTING & MARKS
  // ============================================================================

  function formatDefaultValue(val: number): string {
    if (step >= 1) {
      return Math.round(val).toString();
    }
    const decimals = Math.max(0, -Math.floor(Math.log10(step)));
    return val.toFixed(Math.min(decimals, 3));
  }

  function generateDefaultMarks(): SliderMark[] {
    const count = 5;
    return Array.from({ length: count }, (_, i) => {
      const v = min + (i / (count - 1)) * (max - min);
      return { value: v, label: formatDefaultValue(v) };
    });
  }

  // ============================================================================
  // VALUE CLAMPING & STEPPING
  // ============================================================================

  function clampAndStep(val: number): number {
    const stepped = Math.round(val / step) * step;
    return Math.max(min, Math.min(max, stepped));
  }

  function setValue(newValue: number): void {
    const previousValue = value;
    const clampedValue = clampAndStep(newValue);

    if (clampedValue !== value) {
      value = clampedValue;
      dispatch('input', { value });
      dispatch('change', { value, previousValue });
    }
  }

  function getValueFromPosition(clientX: number, clientY: number): number {
    if (!trackElement) {
      return value;
    }

    const rect = trackElement.getBoundingClientRect();
    let normalizedPos: number;

    if (isVertical) {
      // Vertical: bottom is min, top is max
      normalizedPos = 1 - (clientY - rect.top) / rect.height;
    } else {
      // Horizontal: left is min, right is max
      normalizedPos = (clientX - rect.left) / rect.width;
    }

    normalizedPos = Math.max(0, Math.min(1, normalizedPos));
    return min + normalizedPos * (max - min);
  }

  // ============================================================================
  // DRAG HANDLING
  // ============================================================================

  function handleMouseDown(event: MouseEvent): void {
    if (disabled) {
      return;
    }

    event.preventDefault();
    startDrag();
    setValue(getValueFromPosition(event.clientX, event.clientY));

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  }

  function handleMouseMove(event: MouseEvent): void {
    if (!isDragging) {
      return;
    }

    isFineControl = event.shiftKey;

    if (isFineControl) {
      // Fine control: relative movement
      const delta = isVertical
        ? -event.movementY * fineControlMultiplier
        : event.movementX * fineControlMultiplier;
      const range = max - min;
      const valueDelta = (delta / 100) * range;
      setValue(value + valueDelta);
    } else {
      setValue(getValueFromPosition(event.clientX, event.clientY));
    }
  }

  function handleMouseUp(): void {
    endDrag();
    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mouseup', handleMouseUp);
  }

  function handleTouchStart(event: TouchEvent): void {
    if (disabled || touchId !== null) {
      return;
    }

    const touch = event.touches[0];
    touchId = touch.identifier;
    event.preventDefault();
    startDrag();
    setValue(getValueFromPosition(touch.clientX, touch.clientY));
  }

  function handleTouchMove(event: TouchEvent): void {
    if (!isDragging || touchId === null) {
      return;
    }

    const touch = Array.from(event.touches).find((t) => t.identifier === touchId);
    if (touch) {
      event.preventDefault();
      setValue(getValueFromPosition(touch.clientX, touch.clientY));
    }
  }

  function handleTouchEnd(event: TouchEvent): void {
    const touch = Array.from(event.changedTouches).find((t) => t.identifier === touchId);
    if (touch) {
      endDrag();
      touchId = null;
    }
  }

  function startDrag(): void {
    isDragging = true;
    showTooltip = true;
    dispatch('dragStart', { value });
  }

  function endDrag(): void {
    if (isDragging) {
      isDragging = false;
      isFineControl = false;
      showTooltip = false;
      dispatch('dragEnd', { value });
    }
  }

  // ============================================================================
  // KEYBOARD HANDLING
  // ============================================================================

  function handleKeyDown(event: KeyboardEvent): void {
    if (disabled) {
      return;
    }

    const multiplier = event.shiftKey ? fineControlMultiplier : 1;
    const increment = step * multiplier;
    const largeIncrement = ((max - min) / 10) * multiplier;

    let handled = true;

    switch (event.key) {
      case 'ArrowUp':
      case 'ArrowRight':
        setValue(value + increment);
        break;
      case 'ArrowDown':
      case 'ArrowLeft':
        setValue(value - increment);
        break;
      case 'PageUp':
        setValue(value + largeIncrement);
        break;
      case 'PageDown':
        setValue(value - largeIncrement);
        break;
      case 'Home':
        setValue(min);
        break;
      case 'End':
        setValue(max);
        break;
      default:
        handled = false;
    }

    if (handled) {
      event.preventDefault();
    }
  }

  function handleDoubleClick(): void {
    if (disabled) {
      return;
    }

    if (defaultValue !== null) {
      setValue(defaultValue);
    } else if (bipolar) {
      setValue(0);
    } else {
      setValue((max + min) / 2);
    }
  }

  // ============================================================================
  // UTILITY
  // ============================================================================

  function getMarkPosition(markValue: number): number {
    return ((markValue - min) / (max - min)) * 100;
  }
</script>

<div
  class="slider-container"
  class:vertical={isVertical}
  class:horizontal={!isVertical}
  class:disabled
  class:dragging={isDragging}
  style="
    --track-size: {sizeConfig.trackSize}px;
    --thumb-size: {sizeConfig.thumbSize}px;
    --length: {sizeConfig.length}px;
    --slider-color: {color};
    --slider-track: {trackColor};
  "
>
  {#if label && !isVertical}
    <label class="slider-label" for={id}>{label}</label>
  {/if}

  <div class="slider-wrapper">
    {#if showValue && valuePosition === 'left'}
      <div class="value-display left">
        <span class="value-text">{displayValue}</span>
        {#if unit}<span class="unit-text">{unit}</span>{/if}
      </div>
    {/if}

    {#if showValue && valuePosition === 'top' && !isVertical}
      <div class="value-display top">
        <span class="value-text">{displayValue}</span>
        {#if unit}<span class="unit-text">{unit}</span>{/if}
      </div>
    {/if}

    <div
      bind:this={trackElement}
      {id}
      class="slider-track"
      class:style-default={style === 'default'}
      class:style-minimal={style === 'minimal'}
      class:style-fader={style === 'fader'}
      class:style-range={style === 'range'}
      role="slider"
      aria-valuemin={min}
      aria-valuemax={max}
      aria-valuenow={value}
      aria-valuetext="{displayValue}{unit}"
      aria-label={label || 'Slider'}
      aria-orientation={orientation}
      aria-disabled={disabled}
      tabindex={disabled ? -1 : 0}
      on:mousedown={handleMouseDown}
      on:touchstart={handleTouchStart}
      on:touchmove={handleTouchMove}
      on:touchend={handleTouchEnd}
      on:touchcancel={handleTouchEnd}
      on:keydown={handleKeyDown}
      on:dblclick={handleDoubleClick}
      on:mouseenter={() => {
        isHovered = true;
        showTooltip = true;
      }}
      on:mouseleave={() => {
        isHovered = false;
        if (!isDragging) {
          showTooltip = false;
        }
      }}
    >
      <!-- Track background -->
      <div class="track-background" />

      <!-- Filled portion -->
      {#if fillMode !== 'none'}
        <div
          class="track-fill"
          style={isVertical
            ? `bottom: ${fillStart}%; height: ${fillWidth}%`
            : `left: ${fillStart}%; width: ${fillWidth}%`}
        />
      {/if}

      <!-- Marks -->
      {#if computedMarks.length > 0}
        <div class="marks-container">
          {#each computedMarks as mark (mark.value)}
            {@const pos = getMarkPosition(mark.value)}
            <div
              class="mark"
              class:active={mark.value <= value}
              style={isVertical ? `bottom: ${pos}%` : `left: ${pos}%`}
            >
              <div class="mark-tick" />
              {#if mark.label}
                <span class="mark-label">{mark.label}</span>
              {/if}
            </div>
          {/each}
        </div>
      {/if}

      <!-- Thumb -->
      <div class="thumb" style={isVertical ? `bottom: ${percentage}%` : `left: ${percentage}%`}>
        {#if showThumbValue}
          <span class="thumb-value">{displayValue}</span>
        {/if}

        <!-- Tooltip -->
        {#if showValue && valuePosition === 'tooltip' && showTooltip}
          <div class="tooltip">
            <span class="value-text">{displayValue}</span>
            {#if unit}<span class="unit-text">{unit}</span>{/if}
          </div>
        {/if}
      </div>

      <!-- Focus ring -->
      <div class="focus-ring" />
    </div>

    {#if showValue && valuePosition === 'right'}
      <div class="value-display right">
        <span class="value-text">{displayValue}</span>
        {#if unit}<span class="unit-text">{unit}</span>{/if}
      </div>
    {/if}

    {#if showValue && valuePosition === 'bottom' && !isVertical}
      <div class="value-display bottom">
        <span class="value-text">{displayValue}</span>
        {#if unit}<span class="unit-text">{unit}</span>{/if}
      </div>
    {/if}
  </div>

  {#if label && isVertical}
    <label class="slider-label vertical" for={id}>{label}</label>
  {/if}
</div>

<style>
  .slider-container {
    display: flex;
    gap: 8px;
    user-select: none;
    touch-action: none;
  }

  .slider-container.horizontal {
    flex-direction: column;
    width: var(--length);
  }

  .slider-container.vertical {
    flex-direction: column;
    align-items: center;
    height: var(--length);
  }

  .slider-container.disabled {
    opacity: 0.5;
    pointer-events: none;
  }

  .slider-label {
    font-size: 12px;
    color: var(--text-secondary, #9ca3af);
    font-weight: 500;
  }

  .slider-label.vertical {
    writing-mode: vertical-rl;
    text-orientation: mixed;
    transform: rotate(180deg);
  }

  .slider-wrapper {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .vertical .slider-wrapper {
    flex-direction: column;
    height: 100%;
  }

  .slider-track {
    position: relative;
    cursor: pointer;
    outline: none;
    border-radius: calc(var(--track-size) / 2);
  }

  .horizontal .slider-track {
    width: 100%;
    height: var(--track-size);
  }

  .vertical .slider-track {
    width: var(--track-size);
    height: 100%;
  }

  .track-background {
    position: absolute;
    inset: 0;
    background: var(--slider-track);
    border-radius: inherit;
  }

  .track-fill {
    position: absolute;
    background: var(--slider-color);
    border-radius: inherit;
    transition: background-color 0.15s ease;
  }

  .horizontal .track-fill {
    top: 0;
    bottom: 0;
  }

  .vertical .track-fill {
    left: 0;
    right: 0;
  }

  .thumb {
    position: absolute;
    width: var(--thumb-size);
    height: var(--thumb-size);
    background: var(--slider-color);
    border: 2px solid white;
    border-radius: 50%;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.3);
    transform: translate(-50%, -50%);
    transition:
      transform 0.1s ease,
      box-shadow 0.1s ease;
    z-index: 1;
  }

  .horizontal .thumb {
    top: 50%;
  }

  .vertical .thumb {
    left: 50%;
  }

  .slider-track:hover .thumb,
  .slider-container.dragging .thumb {
    transform: translate(-50%, -50%) scale(1.1);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  }

  .slider-track:focus .thumb {
    box-shadow:
      0 0 0 4px rgba(59, 130, 246, 0.3),
      0 2px 6px rgba(0, 0, 0, 0.3);
  }

  .thumb-value {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 9px;
    font-weight: 600;
    color: white;
    pointer-events: none;
  }

  .tooltip {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%);
    background: var(--tooltip-bg, #1f2937);
    color: var(--tooltip-text, #f3f4f6);
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 12px;
    white-space: nowrap;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    pointer-events: none;
    z-index: 10;
    display: flex;
    align-items: baseline;
    gap: 2px;
  }

  .vertical .tooltip {
    bottom: auto;
    left: calc(100% + 8px);
    transform: translateY(-50%);
  }

  .tooltip::after {
    content: '';
    position: absolute;
    top: 100%;
    left: 50%;
    transform: translateX(-50%);
    border: 4px solid transparent;
    border-top-color: var(--tooltip-bg, #1f2937);
  }

  .vertical .tooltip::after {
    top: 50%;
    left: auto;
    right: 100%;
    transform: translateY(-50%);
    border: 4px solid transparent;
    border-right-color: var(--tooltip-bg, #1f2937);
    border-top-color: transparent;
  }

  .marks-container {
    position: absolute;
    inset: 0;
  }

  .mark {
    position: absolute;
  }

  .horizontal .mark {
    transform: translateX(-50%);
    top: calc(100% + 4px);
  }

  .vertical .mark {
    transform: translateY(50%);
    left: calc(100% + 4px);
  }

  .mark-tick {
    width: 1px;
    height: 6px;
    background: var(--slider-track);
    transition: background-color 0.15s ease;
  }

  .vertical .mark-tick {
    width: 6px;
    height: 1px;
  }

  .mark.active .mark-tick {
    background: var(--slider-color);
  }

  .mark-label {
    display: block;
    margin-top: 2px;
    font-size: 10px;
    color: var(--text-secondary, #9ca3af);
    white-space: nowrap;
  }

  .vertical .mark-label {
    margin-top: 0;
    margin-left: 2px;
  }

  .value-display {
    display: flex;
    align-items: baseline;
    gap: 2px;
    font-size: 12px;
    font-family: 'SF Mono', 'Monaco', 'Inconsolata', monospace;
    color: var(--text-primary, #f3f4f6);
    min-width: 40px;
  }

  .value-display.top {
    margin-bottom: 4px;
    justify-content: center;
  }

  .value-display.bottom {
    margin-top: 4px;
    justify-content: center;
  }

  .value-display.left {
    justify-content: flex-end;
  }

  .value-display.right {
    justify-content: flex-start;
  }

  .value-text {
    font-weight: 500;
  }

  .unit-text {
    font-size: 0.9em;
    color: var(--text-secondary, #9ca3af);
  }

  .focus-ring {
    position: absolute;
    inset: -4px;
    border-radius: calc(var(--track-size) / 2 + 4px);
    border: 2px solid transparent;
    transition: border-color 0.15s ease;
    pointer-events: none;
  }

  .slider-track:focus-visible .focus-ring {
    border-color: var(--focus-color, #3b82f6);
  }

  /* Style variants */
  .style-fader .thumb {
    border-radius: 4px;
    width: calc(var(--thumb-size) * 1.5);
  }

  .style-minimal .track-background {
    background: transparent;
    border: 1px solid var(--slider-track);
  }

  .style-minimal .thumb {
    border: none;
    background: var(--slider-color);
  }

  .style-range .track-fill {
    background: linear-gradient(
      to right,
      var(--range-start, #22c55e),
      var(--range-mid, #eab308),
      var(--range-end, #ef4444)
    );
  }

  /* High contrast mode */
  @media (prefers-contrast: high) {
    .thumb {
      border-width: 3px;
    }

    .focus-ring {
      border-width: 3px;
    }
  }

  /* Reduced motion */
  @media (prefers-reduced-motion: reduce) {
    .thumb,
    .track-fill,
    .tooltip {
      transition: none;
    }
  }
</style>
