<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  // ============================================================================
  // TYPES
  // ============================================================================

  interface ToolbarItem {
    /** Unique identifier for the tool */
    id: string;
    /** Icon name (supports lucide icons, emoji, or custom SVG) */
    icon: string;
    /** Tooltip text shown on hover */
    tooltip: string;
    /** Optional keyboard shortcut to display in tooltip */
    shortcut?: string;
    /** Custom action to run instead of selection */
    action?: () => void;
    /** Marks this as a separator (other props ignored) */
    separator?: boolean;
    /** Disable the item */
    disabled?: boolean;
    /** Badge content (e.g., notification count) */
    badge?: string | number;
    /** Icon variant for toggle state */
    activeIcon?: string;
    /** ARIA label override */
    ariaLabel?: string;
  }

  interface ToolbarGroup {
    /** Group identifier */
    id: string;
    /** Optional group label */
    label?: string;
    /** Items in this group */
    items: ToolbarItem[];
    /** Add separator after group */
    separatorAfter?: boolean;
  }

  // ============================================================================
  // PROPS
  // ============================================================================

  /** Flat list of toolbar items (simple mode) */
  export let items: ToolbarItem[] = [];

  /** Grouped toolbar items (grouped mode, takes precedence over items) */
  export let groups: ToolbarGroup[] = [];

  /** Currently selected tool ID */
  export let selectedTool: string = '';

  /** Multiple selection mode (allows multiple active tools) */
  export let multiSelect: boolean = false;

  /** Set of selected tool IDs (for multiSelect mode) */
  export let selectedTools: Set<string> = new Set();

  /** Size variant */
  export let size: 'xs' | 'sm' | 'md' | 'lg' = 'md';

  /** Orientation */
  export let orientation: 'horizontal' | 'vertical' = 'horizontal';

  /** Show tooltips */
  export let showTooltips: boolean = true;

  /** Allow deselection by clicking selected item */
  export let allowDeselect: boolean = false;

  /** ARIA label for the toolbar */
  export let ariaLabel: string = 'Toolbar';

  /** Custom CSS class */
  let className: string = '';
  export { className as class };

  /** Disabled state for entire toolbar */
  export let disabled: boolean = false;

  // ============================================================================
  // EVENTS
  // ============================================================================

  const dispatch = createEventDispatcher<{
    toolSelect: { toolId: string; selected: boolean };
    action: { toolId: string };
  }>();

  // ============================================================================
  // CONSTANTS
  // ============================================================================

  const sizeConfig = {
    xs: { button: 'h-6 w-6', icon: 'w-3 h-3', padding: 'p-0.5', gap: 'gap-0.5', separator: 'h-4' },
    sm: { button: 'h-7 w-7', icon: 'w-4 h-4', padding: 'p-1', gap: 'gap-1', separator: 'h-5' },
    md: { button: 'h-8 w-8', icon: 'w-5 h-5', padding: 'p-1.5', gap: 'gap-1.5', separator: 'h-6' },
    lg: { button: 'h-10 w-10', icon: 'w-6 h-6', padding: 'p-2', gap: 'gap-2', separator: 'h-8' },
  };

  // Extended icon map with SVG icons (using inline SVG for common icons)
  const iconMap: Record<string, { svg: string } | { emoji: string }> = {
    // Selection & Edit
    select: { svg: '<path d="M3 3l7.07 16.97 2.51-7.39 7.39-2.51z"/><path d="M13 13l6 6"/>' },
    'mouse-pointer': { svg: '<path d="M3 3l7.07 16.97 2.51-7.39 7.39-2.51z"/>' },
    edit: {
      svg: '<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>',
    },
    pencil: {
      svg: '<line x1="18" y1="2" x2="22" y2="6"/><path d="M7.5 20.5 19 9l-4-4L3.5 16.5 2 22z"/>',
    },
    eraser: {
      svg: '<path d="m7 21-4.3-4.3c-1-1-1-2.5 0-3.4l9.6-9.6c1-1 2.5-1 3.4 0l5.6 5.6c1 1 1 2.5 0 3.4L13 21"/><path d="M22 21H7"/><path d="m5 11 9 9"/>',
    },
    scissors: {
      svg: '<circle cx="6" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><line x1="20" y1="4" x2="8.12" y2="15.88"/><line x1="14.47" y1="14.48" x2="20" y2="20"/><line x1="8.12" y1="8.12" x2="12" y2="12"/>',
    },

    // Transport
    play: { svg: '<polygon points="5 3 19 12 5 21 5 3"/>' },
    pause: {
      svg: '<rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/>',
    },
    stop: { svg: '<rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>' },
    'skip-back': {
      svg: '<polygon points="19 20 9 12 19 4 19 20"/><line x1="5" y1="19" x2="5" y2="5"/>',
    },
    'skip-forward': {
      svg: '<polygon points="5 4 15 12 5 20 5 4"/><line x1="19" y1="5" x2="19" y2="19"/>',
    },
    rewind: {
      svg: '<polygon points="11 19 2 12 11 5 11 19"/><polygon points="22 19 13 12 22 5 22 19"/>',
    },
    'fast-forward': {
      svg: '<polygon points="13 19 22 12 13 5 13 19"/><polygon points="2 19 11 12 2 5 2 19"/>',
    },
    record: { svg: '<circle cx="12" cy="12" r="10"/>' },
    loop: {
      svg: '<polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/>',
    },

    // File operations
    save: {
      svg: '<path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/>',
    },
    folder: {
      svg: '<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>',
    },
    'folder-open': {
      svg: '<path d="m6 14 1.45-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.55 6a2 2 0 0 1-1.94 1.5H4a2 2 0 0 1-2-2V5c0-1.1.9-2 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H18a2 2 0 0 1 2 2v2"/>',
    },
    file: {
      svg: '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/>',
    },

    // Edit operations
    undo: { svg: '<path d="M3 7v6h6"/><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"/>' },
    redo: { svg: '<path d="M21 7v6h-6"/><path d="M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6 2.3l3 2.7"/>' },
    copy: {
      svg: '<rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>',
    },
    paste: {
      svg: '<path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/><rect x="8" y="2" width="8" height="4" rx="1" ry="1"/>',
    },
    cut: {
      svg: '<circle cx="6" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><line x1="20" y1="4" x2="8.12" y2="15.88"/><line x1="14.47" y1="14.48" x2="20" y2="20"/><line x1="8.12" y1="8.12" x2="12" y2="12"/>',
    },
    delete: {
      svg: '<polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>',
    },
    trash: {
      svg: '<polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/>',
    },

    // View
    'zoom-in': {
      svg: '<circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/><line x1="11" y1="8" x2="11" y2="14"/><line x1="8" y1="11" x2="14" y2="11"/>',
    },
    'zoom-out': {
      svg: '<circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/><line x1="8" y1="11" x2="14" y2="11"/>',
    },
    maximize: {
      svg: '<path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3"/>',
    },
    minimize: {
      svg: '<path d="M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3"/>',
    },
    grid: {
      svg: '<rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/>',
    },

    // Settings & Info
    settings: {
      svg: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>',
    },
    help: {
      svg: '<circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><line x1="12" y1="17" x2="12.01" y2="17"/>',
    },
    info: {
      svg: '<circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/>',
    },

    // Navigation
    'chevron-up': { svg: '<polyline points="18 15 12 9 6 15"/>' },
    'chevron-down': { svg: '<polyline points="6 9 12 15 18 9"/>' },
    'chevron-left': { svg: '<polyline points="15 18 9 12 15 6"/>' },
    'chevron-right': { svg: '<polyline points="9 18 15 12 9 6"/>' },
    plus: { svg: '<line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>' },
    minus: { svg: '<line x1="5" y1="12" x2="19" y2="12"/>' },
    x: { svg: '<line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>' },
    check: { svg: '<polyline points="20 6 9 17 4 12"/>' },

    // MIDI/Audio specific
    music: {
      svg: '<path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/>',
    },
    volume: {
      svg: '<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"/>',
    },
    'volume-x': {
      svg: '<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><line x1="23" y1="9" x2="17" y2="15"/><line x1="17" y1="9" x2="23" y2="15"/>',
    },
    mic: {
      svg: '<path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/><path d="M19 10v2a7 7 0 0 1-14 0v-2"/><line x1="12" y1="19" x2="12" y2="23"/><line x1="8" y1="23" x2="16" y2="23"/>',
    },
    headphones: {
      svg: '<path d="M3 18v-6a9 9 0 0 1 18 0v6"/><path d="M21 19a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3zM3 19a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2v-3a2 2 0 0 0-2-2H3z"/>',
    },
    sliders: {
      svg: '<line x1="4" y1="21" x2="4" y2="14"/><line x1="4" y1="10" x2="4" y2="3"/><line x1="12" y1="21" x2="12" y2="12"/><line x1="12" y1="8" x2="12" y2="3"/><line x1="20" y1="21" x2="20" y2="16"/><line x1="20" y1="12" x2="20" y2="3"/><line x1="1" y1="14" x2="7" y2="14"/><line x1="9" y1="8" x2="15" y2="8"/><line x1="17" y1="16" x2="23" y2="16"/>',
    },
    activity: { svg: '<polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>' },
  };

  // ============================================================================
  // COMPUTED
  // ============================================================================

  $: config = sizeConfig[size];
  $: orientationClass = orientation === 'vertical' ? 'flex-col' : 'flex-row';
  $: separatorClass =
    orientation === 'vertical' ? `w-full h-px` : `h-${config.separator.replace('h-', '')} w-px`;

  // Flatten groups into items if groups are provided
  $: flatItems =
    groups.length > 0
      ? groups.flatMap((group, gi) => {
          const groupItems = group.items.map((item) => ({ ...item, groupId: group.id }));
          if (group.separatorAfter && gi < groups.length - 1) {
            return [...groupItems, { id: `sep-${group.id}`, separator: true } as ToolbarItem];
          }
          return groupItems;
        })
      : items;

  // ============================================================================
  // FUNCTIONS
  // ============================================================================

  function isSelected(itemId: string): boolean {
    if (multiSelect) {
      return selectedTools.has(itemId);
    }
    return selectedTool === itemId;
  }

  function handleItemClick(item: ToolbarItem): void {
    if (item.disabled || disabled) {
      return;
    }

    if (item.action) {
      item.action();
      dispatch('action', { toolId: item.id });
    } else {
      if (multiSelect) {
        const newSet = new Set(selectedTools);
        if (newSet.has(item.id)) {
          if (allowDeselect) {
            newSet.delete(item.id);
          }
        } else {
          newSet.add(item.id);
        }
        selectedTools = newSet;
        dispatch('toolSelect', { toolId: item.id, selected: newSet.has(item.id) });
      } else {
        if (selectedTool === item.id && allowDeselect) {
          selectedTool = '';
          dispatch('toolSelect', { toolId: item.id, selected: false });
        } else {
          selectedTool = item.id;
          dispatch('toolSelect', { toolId: item.id, selected: true });
        }
      }
    }
  }

  function handleKeyDown(event: KeyboardEvent, item: ToolbarItem): void {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      handleItemClick(item);
    }
  }

  function getTooltip(item: ToolbarItem): string {
    if (!showTooltips) {
      return '';
    }
    let tooltip = item.tooltip;
    if (item.shortcut) {
      tooltip += ` (${item.shortcut})`;
    }
    return tooltip;
  }

  function renderIcon(item: ToolbarItem): { type: 'svg' | 'emoji' | 'text'; content: string } {
    const iconName = isSelected(item.id) && item.activeIcon ? item.activeIcon : item.icon;
    const iconDef = iconMap[iconName];

    if (iconDef) {
      if ('svg' in iconDef) {
        return { type: 'svg', content: iconDef.svg };
      }
      return { type: 'emoji', content: iconDef.emoji };
    }

    // Check if it's an emoji (single character or emoji sequence)
    const emojiRegex = /^[\p{Emoji}]+$/u;
    if (emojiRegex.test(iconName)) {
      return { type: 'emoji', content: iconName };
    }

    // Fall back to text
    return { type: 'text', content: iconName };
  }
</script>

<div
  class="toolbar flex items-center {orientationClass} {config.padding} {config.gap} bg-gray-100 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 {className}"
  class:opacity-50={disabled}
  role="toolbar"
  aria-label={ariaLabel}
  aria-orientation={orientation}
  aria-disabled={disabled}
>
  {#each flatItems as item (item.id)}
    {#if item.separator}
      <div
        class="separator {separatorClass} bg-gray-300 dark:bg-gray-600"
        role="separator"
        aria-orientation={orientation === 'horizontal' ? 'vertical' : 'horizontal'}
      ></div>
    {:else}
      {@const selected = !item.action && isSelected(item.id)}
      {@const icon = renderIcon(item)}
      <button
        type="button"
        class="toolbar-button relative flex items-center justify-center rounded transition-all duration-150 {config.button}"
        class:bg-primary={selected}
        class:text-white={selected}
        class:bg-transparent={!selected}
        class:text-gray-700={!selected}
        class:dark:text-gray-300={!selected}
        class:hover:bg-gray-200={!selected && !item.disabled}
        class:dark:hover:bg-gray-700={!selected && !item.disabled}
        class:opacity-50={item.disabled}
        class:cursor-not-allowed={item.disabled || disabled}
        disabled={item.disabled || disabled}
        title={getTooltip(item)}
        on:click={() => handleItemClick(item)}
        on:keydown={(e) => handleKeyDown(e, item)}
        aria-pressed={selected}
        aria-label={item.ariaLabel || item.tooltip}
      >
        {#if icon.type === 'svg'}
          <svg
            class="{config.icon} fill-none stroke-current"
            viewBox="0 0 24 24"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            {@html icon.content}
          </svg>
        {:else if icon.type === 'emoji'}
          <span class="icon-emoji" aria-hidden="true">{icon.content}</span>
        {:else}
          <span class="icon-text text-xs font-medium" aria-hidden="true">{icon.content}</span>
        {/if}

        <!-- Badge -->
        {#if item.badge !== undefined && item.badge !== ''}
          <span
            class="badge absolute -top-1 -right-1 flex items-center justify-center min-w-[16px] h-4 px-1 text-xs font-bold text-white bg-red-500 rounded-full"
            aria-label="{item.badge} notifications"
          >
            {item.badge}
          </span>
        {/if}
      </button>
    {/if}
  {/each}
</div>

<style>
  .toolbar-button:focus-visible {
    outline: 2px solid var(--primary-color, #3b82f6);
    outline-offset: 2px;
  }

  .toolbar-button:active:not(:disabled) {
    transform: scale(0.95);
  }

  .icon-emoji {
    font-size: 1em;
    line-height: 1;
  }

  .badge {
    animation: badge-pop 0.2s ease-out;
  }

  @keyframes badge-pop {
    0% {
      transform: scale(0);
    }
    50% {
      transform: scale(1.2);
    }
    100% {
      transform: scale(1);
    }
  }

  /* Fill icon color for record button */
  .toolbar-button[aria-label*='Record'] svg {
    fill: currentColor;
    stroke: none;
  }

  .toolbar-button[aria-label*='Record'][aria-pressed='true'] svg {
    fill: #ef4444;
  }
</style>
