<script lang="ts">
  import { onMount } from 'svelte';
  import { uiActions } from '$lib/stores/uiStore';
  import { playbackActions } from '$lib/stores/playbackStore';
  import type { WindowId } from '$lib/types';

  type MenuKey = 'file' | 'edit' | 'view' | 'transport' | 'help';

  type Separator = { separator: true };

  type MenuItem = {
    label: string;
    shortcut?: string;
    action: () => void;
    disabled?: boolean;
  };

  type MenuEntry = MenuItem | Separator;

  let openMenu: MenuKey | null = null;

  const menuItems: Record<MenuKey, MenuEntry[]> = {
    file: [
      { label: 'New Project', shortcut: 'Ctrl+N', action: () => console.log('New Project') },
      { label: 'Open Project', shortcut: 'Ctrl+O', action: () => console.log('Open Project') },
      { label: 'Save Project', shortcut: 'Ctrl+S', action: () => console.log('Save Project') },
      { label: 'Save As...', shortcut: '', action: () => console.log('Save As') },
      { separator: true },
      { label: 'Import MIDI Files', shortcut: 'Ctrl+I', action: () => console.log('Import MIDI') },
      { label: 'Export MIDI', shortcut: 'Ctrl+E', action: () => console.log('Export MIDI') },
      { separator: true },
      { label: 'Exit', shortcut: 'Alt+F4', action: () => console.log('Exit') }
    ],
    edit: [
      { label: 'Undo', shortcut: 'Ctrl+Z', action: () => console.log('Undo'), disabled: true },
      { label: 'Redo', shortcut: 'Ctrl+Y', action: () => console.log('Redo'), disabled: true },
      { separator: true },
      { label: 'Cut', shortcut: 'Ctrl+X', action: () => console.log('Cut'), disabled: true },
      { label: 'Copy', shortcut: 'Ctrl+C', action: () => console.log('Copy'), disabled: true },
      { label: 'Paste', shortcut: 'Ctrl+V', action: () => console.log('Paste'), disabled: true },
      { label: 'Delete', shortcut: 'Del', action: () => console.log('Delete'), disabled: true },
      { separator: true },
      { label: 'Select All', shortcut: 'Ctrl+A', action: () => console.log('Select All'), disabled: true }
    ],
    view: [
      { label: 'Toggle DAW Window', shortcut: 'F1', action: () => uiActions.toggleWindow('daw' as WindowId) },
      { label: 'Toggle Mixer Window', shortcut: 'F2', action: () => uiActions.toggleWindow('mixer' as WindowId) },
      { label: 'Toggle Database Window', shortcut: 'F3', action: () => uiActions.toggleWindow('database' as WindowId) },
      { label: 'Toggle Pipeline Window', shortcut: 'F4', action: () => uiActions.toggleWindow('pipeline' as WindowId) },
      { separator: true },
      { label: 'Command Palette', shortcut: 'Ctrl+Shift+P', action: () => console.log('Command Palette') }
    ],
    transport: [
      { label: 'Play', shortcut: 'Space', action: () => playbackActions.play() },
      { label: 'Pause', shortcut: 'Space', action: () => playbackActions.pause() },
      { label: 'Stop', shortcut: 'Ctrl+Space', action: () => playbackActions.stop() },
      { separator: true },
      { label: 'Toggle Loop', shortcut: 'L', action: () => playbackActions.toggleLoop() },
      { label: 'Toggle Metronome', shortcut: 'M', action: () => playbackActions.toggleMetronome() }
    ],
    help: [
      { label: 'Documentation', shortcut: '', action: () => console.log('Documentation') },
      { label: 'Keyboard Shortcuts', shortcut: '', action: () => console.log('Shortcuts') },
      { separator: true },
      { label: 'Report Bug', shortcut: '', action: () => console.log('Report Bug') },
      { label: 'About MIDI Software Center', shortcut: '', action: () => console.log('About') }
    ]
  };

  function isSeparator(entry: MenuEntry): entry is Separator {
    return 'separator' in entry;
  }

  function getMenuItem(entry: MenuEntry): MenuItem {
    return entry as MenuItem;
  }

  function openDropdown(menu: string) {
    const key = menu as MenuKey;
    openMenu = openMenu === key ? null : key;
  }

  function closeDropdown() {
    openMenu = null;
  }

  function handleMenuAction(action: () => void) {
    action();
    closeDropdown();
  }

  // Click outside to close
  onMount(() => {
    function handleClickOutside(event: MouseEvent) {
      const target = event.target as HTMLElement;
      if (!target.closest('.menu-bar') && !target.closest('.dropdown-menu')) {
        closeDropdown();
      }
    }
    document.addEventListener('click', handleClickOutside);
    return () => document.removeEventListener('click', handleClickOutside);
  });
</script>

<div class="menu-bar dark:bg-menu dark:text-app-text flex space-x-0">
  {#each ['file', 'edit', 'view', 'transport', 'help'] as menuKey}
    <div class="menu-item relative">
      <button
        class="px-2 py-1 hover:dark:bg-primary-color/20 focus:outline-none"
        class:active={openMenu === menuKey}
        on:click={() => openDropdown(menuKey)}
      >
        {menuKey.charAt(0).toUpperCase() + menuKey.slice(1)}
      </button>
      {#if openMenu === menuKey}
        <div class="dropdown-menu dark:bg-menu dark:border-window-border absolute top-full left-0 mt-1 w-48 rounded shadow-lg z-50">
          {#each menuItems[menuKey] as item}
            {#if isSeparator(item)}
              <div class="border-t dark:border-window-border my-1"></div>
            {:else}
              {@const menuItem = getMenuItem(item)}
              <button
                class="w-full text-left px-4 py-2 hover:dark:bg-primary-color/20 disabled:dark:text-gray-500 disabled:cursor-not-allowed"
                class:disabled={menuItem.disabled ?? false}
                on:click={() => handleMenuAction(menuItem.action)}
              >
                <div class="flex justify-between">
                  <span>{menuItem.label}</span>
                  {#if menuItem.shortcut}
                    <span class="text-xs opacity-60">{menuItem.shortcut}</span>
                  {/if}
                </div>
              </button>
            {/if}
          {/each}
        </div>
      {/if}
    </div>
  {/each}
</div>

<style>
  .menu-bar {
    border-bottom: 1px solid var(--window-border);
    user-select: none;
  }

  .menu-item {
    position: relative;
  }

  .active {
    background-color: var(--primary-color);
    color: white;
  }

  .dropdown-menu {
    border: 1px solid var(--window-border);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }
</style>