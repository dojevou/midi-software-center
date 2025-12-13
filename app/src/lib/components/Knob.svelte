<script lang="ts">
  import { createEventDispatcher, onDestroy, onMount } from 'svelte';

  // ============================================================================
  // TYPES
  // ============================================================================

  type KnobSize = 'xs' | 'sm' | 'md' | 'lg' | 'xl';
  type KnobStyle = 'default' | 'vintage' | 'minimal' | 'modern';
  type DragMode = 'vertical' | 'circular' | 'horizontal';

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
  /** Default/center value for double-click reset */
  export let defaultValue: number | null = null;
  /** Label text */
  export let label: string = '';
  /** Unit suffix (e.g., 'dB', '%', 'Hz') */
  export let unit: string = '';
  /** Size variant */
  export let size: KnobSize = 'md';
  /** Visual style */
  export let style: KnobStyle = 'default';
  /** Drag interaction mode */
  export let dragMode: DragMode = 'vertical';
  /** Sensitivity multiplier (higher = faster response) */
  export let sensitivity: number = 1;
  /** Show value display */
  export let showValue: boolean = true;
  /** Show tick marks */
  export let showTicks: boolean = false;
  /** Number of tick marks */
  export let tickCount: number = 11;
  /** Disabled state */
  export let disabled: boolean = false;
  /** Bipolar mode (center is zero, goes negative/positive) */
  export let bipolar: boolean = false;
  /** Custom value formatter */
  export let formatValue: ((value: number) => string) | null = null;
  /** Rotation range in degrees (default 270 = -135 to +135) */
  export let rotationRange: number = 270;
  /** Primary color (CSS color value) */
  export let color: string = 'var(--knob-color, #3b82f6)';
  /** Background color */
  export let bgColor: string = 'var(--knob-bg, #1f2937)';
  /** Track color */
  export let trackColor: string = 'var(--knob-track, #374151)';
  /** Fine control modifier (hold shift for fine adjustment) */
  export let fineControlMultiplier: number = 0.1;
  /** Custom ID for accessibility */
  export let id: string = `knob-${Math.random().toString(36).substr(2, 9)}`;

  // ============================================================================
  // STATE
  // ============================================================================

  let knobElement: HTMLDivElement;
  let isDragging = false;
  let startY = 0;
  let startX = 0;
  let startValue = 0;
  let startAngle = 0;
  let isFineControl = false;
  let isHovered = false;
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
  $: rotation = (normalizedValue - 0.5) * rotationRange;
  $: displayValue = formatValue ? formatValue(value) : formatDefaultValue(value);

  $: sizeConfig = {
    xs: { diameter: 32, fontSize: 9, labelSize: 8, strokeWidth: 3 },
    sm: { diameter: 44, fontSize: 10, labelSize: 9, strokeWidth: 4 },
    md: { diameter: 56, fontSize: 12, labelSize: 10, strokeWidth: 5 },
    lg: { diameter: 72, fontSize: 14, labelSize: 12, strokeWidth: 6 },
    xl: { diameter: 96, fontSize: 18, labelSize: 14, strokeWidth: 8 },
  }[size];

  $: tickPositions = showTicks
    ? Array.from({ length: tickCount }, (_, i) => {
        const normalized = i / (tickCount - 1);
        const angle = (normalized - 0.5) * rotationRange;
        return angle;
      })
    : [];

  // ============================================================================
  // VALUE FORMATTING
  // ============================================================================

  function formatDefaultValue(val: number): string {
    if (step >= 1) {
      return Math.round(val).toString();
    }
    const decimals = Math.max(0, -Math.floor(Math.log10(step)));
    return val.toFixed(Math.min(decimals, 3));
  }

  // ============================================================================
  // VALUE CLAMPING & STEPPING
  // ============================================================================

  function clampAndStep(val: number): number {
    // Apply step
    const stepped = Math.round(val / step) * step;
    // Clamp to range
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

  // ============================================================================
  // DRAG HANDLING
  // ============================================================================

  function handleMouseDown(event: MouseEvent): void {
    if (disabled) {
      return;
    }

    event.preventDefault();
    startDrag(event.clientX, event.clientY);

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  }

  function handleMouseMove(event: MouseEvent): void {
    if (!isDragging) {
      return;
    }
    updateDrag(event.clientX, event.clientY, event.shiftKey);
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
    startDrag(touch.clientX, touch.clientY);
  }

  function handleTouchMove(event: TouchEvent): void {
    if (!isDragging || touchId === null) {
      return;
    }

    const touch = Array.from(event.touches).find((t) => t.identifier === touchId);
    if (touch) {
      event.preventDefault();
      updateDrag(touch.clientX, touch.clientY, false);
    }
  }

  function handleTouchEnd(event: TouchEvent): void {
    const touch = Array.from(event.changedTouches).find((t) => t.identifier === touchId);
    if (touch) {
      endDrag();
      touchId = null;
    }
  }

  function startDrag(x: number, y: number): void {
    isDragging = true;
    startX = x;
    startY = y;
    startValue = value;

    if (dragMode === 'circular' && knobElement) {
      const rect = knobElement.getBoundingClientRect();
      const centerX = rect.left + rect.width / 2;
      const centerY = rect.top + rect.height / 2;
      startAngle = Math.atan2(y - centerY, x - centerX) * (180 / Math.PI);
    }

    dispatch('dragStart', { value });
  }

  function updateDrag(x: number, y: number, shiftKey: boolean): void {
    if (!isDragging) {
      return;
    }

    isFineControl = shiftKey;
    const multiplier = isFineControl ? fineControlMultiplier : 1;
    const range = max - min;
    let delta = 0;

    switch (dragMode) {
      case 'vertical':
        // Moving up increases value, down decreases
        const pixelDeltaV = startY - y;
        delta = (pixelDeltaV / 150) * range * sensitivity * multiplier;
        break;

      case 'horizontal':
        // Moving right increases value, left decreases
        const pixelDeltaH = x - startX;
        delta = (pixelDeltaH / 150) * range * sensitivity * multiplier;
        break;

      case 'circular':
        if (knobElement) {
          const rect = knobElement.getBoundingClientRect();
          const centerX = rect.left + rect.width / 2;
          const centerY = rect.top + rect.height / 2;
          const currentAngle = Math.atan2(y - centerY, x - centerX) * (180 / Math.PI);
          let angleDelta = currentAngle - startAngle;

          // Normalize angle delta
          if (angleDelta > 180) {
            angleDelta -= 360;
          }
          if (angleDelta < -180) {
            angleDelta += 360;
          }

          delta = (angleDelta / rotationRange) * range * sensitivity * multiplier;
          startAngle = currentAngle;
          startValue = value;
        }
        break;
    }

    if (dragMode !== 'circular') {
      setValue(startValue + delta);
    } else {
      setValue(value + delta);
    }
  }

  function endDrag(): void {
    if (isDragging) {
      isDragging = false;
      isFineControl = false;
      dispatch('dragEnd', { value });
    }
  }

  // ============================================================================
  // KEYBOARD & WHEEL HANDLING
  // ============================================================================

  function handleKeyDown(event: KeyboardEvent): void {
    if (disabled) {
      return;
    }

    const multiplier = event.shiftKey ? fineControlMultiplier : 1;
    const increment = step * multiplier;
    const largeIncrement = ((max - min) / 10) * multiplier;

    switch (event.key) {
      case 'ArrowUp':
      case 'ArrowRight':
        event.preventDefault();
        setValue(value + increment);
        break;
      case 'ArrowDown':
      case 'ArrowLeft':
        event.preventDefault();
        setValue(value - increment);
        break;
      case 'PageUp':
        event.preventDefault();
        setValue(value + largeIncrement);
        break;
      case 'PageDown':
        event.preventDefault();
        setValue(value - largeIncrement);
        break;
      case 'Home':
        event.preventDefault();
        setValue(min);
        break;
      case 'End':
        event.preventDefault();
        setValue(max);
        break;
    }
  }

  function handleWheel(event: WheelEvent): void {
    if (disabled || !isHovered) {
      return;
    }

    event.preventDefault();
    const multiplier = event.shiftKey ? fineControlMultiplier : 1;
    const delta = event.deltaY > 0 ? -step : step;
    setValue(value + delta * multiplier);
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
  // LIFECYCLE
  // ============================================================================

  onMount(() => {
    if (knobElement) {
      knobElement.addEventListener('wheel', handleWheel, { passive: false });
    }
  });

  onDestroy(() => {
    if (knobElement) {
      knobElement.removeEventListener('wheel', handleWheel);
    }
    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mouseup', handleMouseUp);
  });
</script>

<div
  class="knob-container"
  class:disabled
  class:dragging={isDragging}
  style="--knob-diameter: {sizeConfig.diameter}px; --font-size: {sizeConfig.fontSize}px; --label-size: {sizeConfig.labelSize}px;"
>
  {#if label}
    <label class="knob-label" for={id}>{label}</label>
  {/if}

  <div
    bind:this={knobElement}
    {id}
    class="knob"
    class:style-default={style === 'default'}
    class:style-vintage={style === 'vintage'}
    class:style-minimal={style === 'minimal'}
    class:style-modern={style === 'modern'}
    role="slider"
    aria-valuemin={min}
    aria-valuemax={max}
    aria-valuenow={value}
    aria-valuetext="{displayValue}{unit}"
    aria-label={label || 'Knob control'}
    aria-disabled={disabled}
    tabindex={disabled ? -1 : 0}
    on:mousedown={handleMouseDown}
    on:touchstart={handleTouchStart}
    on:touchmove={handleTouchMove}
    on:touchend={handleTouchEnd}
    on:touchcancel={handleTouchEnd}
    on:keydown={handleKeyDown}
    on:dblclick={handleDoubleClick}
    on:mouseenter={() => (isHovered = true)}
    on:mouseleave={() => (isHovered = false)}
  >
    <svg
      width={sizeConfig.diameter}
      height={sizeConfig.diameter}
      viewBox="0 0 100 100"
      class="knob-svg"
    >
      <!-- Background track -->
      <circle
        cx="50"
        cy="50"
        r="42"
        fill="none"
        stroke={trackColor}
        stroke-width={sizeConfig.strokeWidth}
        stroke-linecap="round"
        stroke-dasharray="{(rotationRange / 360) * 264} 264"
        transform="rotate({90 + (360 - rotationRange) / 2} 50 50)"
      />

      <!-- Value arc -->
      {#if bipolar}
        <!-- Bipolar: draw from center -->
        {@const centerNorm = (0 - min) / (max - min)}
        {@const arcStart = (centerNorm - 0.5) * rotationRange}
        {@const arcEnd = rotation}
        {@const arcLength = Math.abs(arcEnd - arcStart)}
        {@const arcOffset = Math.min(arcStart, arcEnd)}
        <circle
          cx="50"
          cy="50"
          r="42"
          fill="none"
          stroke={color}
          stroke-width={sizeConfig.strokeWidth}
          stroke-linecap="round"
          stroke-dasharray="{(arcLength / 360) * 264} 264"
          transform="rotate({90 + arcOffset} 50 50)"
          class="value-arc"
        />
      {:else}
        <!-- Unipolar: draw from start -->
        <circle
          cx="50"
          cy="50"
          r="42"
          fill="none"
          stroke={color}
          stroke-width={sizeConfig.strokeWidth}
          stroke-linecap="round"
          stroke-dasharray="{normalizedValue * (rotationRange / 360) * 264} 264"
          transform="rotate({90 + (360 - rotationRange) / 2} 50 50)"
          class="value-arc"
        />
      {/if}

      <!-- Tick marks -->
      {#if showTicks}
        {#each tickPositions as angle (angle)}
          <line
            x1="50"
            y1="8"
            x2="50"
            y2="14"
            stroke={trackColor}
            stroke-width="1"
            transform="rotate({angle} 50 50)"
          />
        {/each}
      {/if}

      <!-- Knob body -->
      <circle cx="50" cy="50" r="32" fill={bgColor} class="knob-body" />

      <!-- Inner highlight (for depth effect) -->
      {#if style === 'default' || style === 'vintage'}
        <circle
          cx="50"
          cy="50"
          r="28"
          fill="none"
          stroke="rgba(255,255,255,0.1)"
          stroke-width="1"
        />
      {/if}

      <!-- Indicator line -->
      <g transform="rotate({rotation} 50 50)">
        <line
          x1="50"
          y1="22"
          x2="50"
          y2="34"
          stroke={color}
          stroke-width="3"
          stroke-linecap="round"
          class="indicator"
        />
      </g>

      <!-- Center dot (for some styles) -->
      {#if style === 'vintage'}
        <circle cx="50" cy="50" r="4" fill={color} class="center-dot" />
      {/if}
    </svg>

    <!-- Focused ring -->
    <div class="focus-ring" />
  </div>

  {#if showValue}
    <div class="knob-value">
      <span class="value-text">{displayValue}</span>
      {#if unit}
        <span class="unit-text">{unit}</span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .knob-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    user-select: none;
    touch-action: none;
  }

  .knob-container.disabled {
    opacity: 0.5;
    pointer-events: none;
  }

  .knob-label {
    font-size: var(--label-size);
    color: var(--text-secondary, #9ca3af);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: calc(var(--knob-diameter) + 20px);
  }

  .knob {
    position: relative;
    width: var(--knob-diameter);
    height: var(--knob-diameter);
    cursor: grab;
    outline: none;
    border-radius: 50%;
  }

  .knob:focus .focus-ring {
    opacity: 1;
  }

  .knob.dragging {
    cursor: grabbing;
  }

  .knob-svg {
    display: block;
    filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.2));
  }

  .value-arc {
    transition: stroke 0.1s ease;
  }

  .knob-body {
    filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.3));
    transition: filter 0.1s ease;
  }

  .knob:hover .knob-body {
    filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.4));
  }

  .knob.dragging .knob-body {
    filter: drop-shadow(0 3px 6px rgba(0, 0, 0, 0.5));
  }

  .indicator {
    transition: stroke 0.1s ease;
  }

  .focus-ring {
    position: absolute;
    inset: -4px;
    border-radius: 50%;
    border: 2px solid var(--focus-color, #3b82f6);
    opacity: 0;
    transition: opacity 0.15s ease;
    pointer-events: none;
  }

  .knob-value {
    display: flex;
    align-items: baseline;
    gap: 2px;
    font-size: var(--font-size);
    font-family: 'SF Mono', 'Monaco', 'Inconsolata', monospace;
    color: var(--text-primary, #f3f4f6);
    min-width: calc(var(--knob-diameter) - 8px);
    justify-content: center;
  }

  .value-text {
    font-weight: 500;
  }

  .unit-text {
    font-size: 0.8em;
    color: var(--text-secondary, #9ca3af);
    font-weight: 400;
  }

  /* Style variants */
  .style-default .knob-body {
    fill: var(--knob-bg, #1f2937);
  }

  .style-vintage .knob-body {
    fill: linear-gradient(135deg, #3d3d3d 0%, #2a2a2a 50%, #1a1a1a 100%);
  }

  .style-minimal .knob-body {
    fill: transparent;
  }

  .style-minimal .knob-svg {
    filter: none;
  }

  .style-modern .knob-body {
    fill: var(--knob-bg, #1f2937);
  }

  .style-modern .indicator {
    stroke-width: 4;
  }

  /* Animations */
  @media (prefers-reduced-motion: no-preference) {
    .knob {
      transition: transform 0.1s ease;
    }

    .knob:active {
      transform: scale(0.98);
    }
  }

  /* High contrast mode */
  @media (prefers-contrast: high) {
    .knob-body {
      stroke: currentColor;
      stroke-width: 2;
    }

    .focus-ring {
      border-width: 3px;
    }
  }
</style>
