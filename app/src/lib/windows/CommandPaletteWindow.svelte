<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { api } from '$lib/api';

  interface Command {
    id: string;
    name: string;
    category: string;
    shortcut?: string;
    icon: string;
  }

  interface CommandHistoryEntry {
    command: string;
    timestamp: string;
  }

  // State using Svelte 4 let bindings
  let searchQuery = '';
  let filteredCommands: Command[] = [];
  let selectedIndex = 0;
  let showPalette = false;
  let categories: Record<string, Command[]> = {};
  let recentCommands: Command[] = [];
  let commandHistory: CommandHistoryEntry[] = [];

  const allCommands: Command[] = [
    // File Operations
    { id: 'new_project', name: 'New Project', category: 'File', shortcut: 'Ctrl+N', icon: '📄' },
    { id: 'open_project', name: 'Open Project', category: 'File', shortcut: 'Ctrl+O', icon: '📂' },
    { id: 'save_project', name: 'Save Project', category: 'File', shortcut: 'Ctrl+S', icon: '💾' },
    { id: 'save_as', name: 'Save As', category: 'File', shortcut: 'Ctrl+Shift+S', icon: '💾' },
    { id: 'import_file', name: 'Import File', category: 'File', shortcut: 'Ctrl+I', icon: '📥' },
    {
      id: 'export_project',
      name: 'Export Project',
      category: 'File',
      shortcut: 'Ctrl+E',
      icon: '📤',
    },

    // Edit Operations
    { id: 'undo', name: 'Undo', category: 'Edit', shortcut: 'Ctrl+Z', icon: '↩️' },
    { id: 'redo', name: 'Redo', category: 'Edit', shortcut: 'Ctrl+Y', icon: '↪️' },
    { id: 'cut', name: 'Cut', category: 'Edit', shortcut: 'Ctrl+X', icon: '✂️' },
    { id: 'copy', name: 'Copy', category: 'Edit', shortcut: 'Ctrl+C', icon: '📋' },
    { id: 'paste', name: 'Paste', category: 'Edit', shortcut: 'Ctrl+V', icon: '📝' },
    { id: 'delete', name: 'Delete', category: 'Edit', shortcut: 'Del', icon: '🗑️' },
    { id: 'select_all', name: 'Select All', category: 'Edit', shortcut: 'Ctrl+A', icon: '☑️' },
    { id: 'duplicate', name: 'Duplicate', category: 'Edit', shortcut: 'Ctrl+D', icon: '⧉' },

    // View Operations
    { id: 'toggle_mixer', name: 'Toggle Mixer', category: 'View', shortcut: 'F3', icon: '🎚️' },
    {
      id: 'toggle_piano_roll',
      name: 'Toggle Piano Roll',
      category: 'View',
      shortcut: 'F4',
      icon: '🎹',
    },
    { id: 'toggle_browser', name: 'Toggle Browser', category: 'View', shortcut: 'F5', icon: '📁' },
    {
      id: 'toggle_transport',
      name: 'Toggle Transport',
      category: 'View',
      shortcut: 'F6',
      icon: '⏯️',
    },
    { id: 'zoom_in', name: 'Zoom In', category: 'View', shortcut: 'Ctrl+=', icon: '🔍' },
    { id: 'zoom_out', name: 'Zoom Out', category: 'View', shortcut: 'Ctrl+-', icon: '🔍' },
    { id: 'zoom_reset', name: 'Reset Zoom', category: 'View', shortcut: 'Ctrl+0', icon: '🔍' },

    // Transport Operations
    { id: 'play', name: 'Play/Stop', category: 'Transport', shortcut: 'Space', icon: '⏯️' },
    { id: 'record', name: 'Record', category: 'Transport', shortcut: 'R', icon: '🔴' },
    { id: 'stop', name: 'Stop', category: 'Transport', shortcut: 'S', icon: '⏹️' },
    { id: 'rewind', name: 'Rewind', category: 'Transport', shortcut: 'Left', icon: '⏪' },
    { id: 'forward', name: 'Forward', category: 'Transport', shortcut: 'Right', icon: '⏩' },
    { id: 'loop', name: 'Toggle Loop', category: 'Transport', shortcut: 'L', icon: '🔁' },
    { id: 'metronome', name: 'Toggle Metronome', category: 'Transport', shortcut: 'M', icon: '🎵' },

    // Track Operations
    { id: 'add_track', name: 'Add Track', category: 'Track', shortcut: 'Ctrl+T', icon: '➕' },
    {
      id: 'delete_track',
      name: 'Delete Track',
      category: 'Track',
      shortcut: 'Ctrl+Shift+T',
      icon: '➖',
    },
    {
      id: 'duplicate_track',
      name: 'Duplicate Track',
      category: 'Track',
      shortcut: 'Ctrl+Alt+T',
      icon: '⧉',
    },
    { id: 'mute_track', name: 'Mute Track', category: 'Track', shortcut: 'M', icon: '🔇' },
    { id: 'solo_track', name: 'Solo Track', category: 'Track', shortcut: 'S', icon: '🔈' },

    // MIDI Operations
    { id: 'quantize', name: 'Quantize', category: 'MIDI', shortcut: 'Q', icon: '⏱️' },
    {
      id: 'transpose_up',
      name: 'Transpose Up',
      category: 'MIDI',
      shortcut: 'Shift+Up',
      icon: '⬆️',
    },
    {
      id: 'transpose_down',
      name: 'Transpose Down',
      category: 'MIDI',
      shortcut: 'Shift+Down',
      icon: '⬇️',
    },
    { id: 'velocity_up', name: 'Velocity Up', category: 'MIDI', shortcut: 'Ctrl+Up', icon: '🔺' },
    {
      id: 'velocity_down',
      name: 'Velocity Down',
      category: 'MIDI',
      shortcut: 'Ctrl+Down',
      icon: '🔻',
    },

    // Window Operations
    { id: 'minimize', name: 'Minimize Window', category: 'Window', shortcut: 'Ctrl+M', icon: '🗕' },
    { id: 'maximize', name: 'Maximize Window', category: 'Window', shortcut: 'F11', icon: '🗖' },
    { id: 'fullscreen', name: 'Toggle Fullscreen', category: 'Window', shortcut: 'F11', icon: '⛶' },
    { id: 'close_window', name: 'Close Window', category: 'Window', shortcut: 'Ctrl+W', icon: '✕' },

    // Settings
    {
      id: 'preferences',
      name: 'Preferences',
      category: 'Settings',
      shortcut: 'Ctrl+,',
      icon: '⚙️',
    },
    { id: 'keyboard_shortcuts', name: 'Keyboard Shortcuts', category: 'Settings', icon: '⌨️' },
    { id: 'audio_settings', name: 'Audio Settings', category: 'Settings', icon: '🔊' },
    { id: 'midi_settings', name: 'MIDI Settings', category: 'Settings', icon: '🎹' },
  ];

  onMount(() => {
    // Load recent commands from localStorage
    const savedRecent = localStorage.getItem('recentCommands');
    if (savedRecent) {
      recentCommands = JSON.parse(savedRecent);
    }

    const savedHistory = localStorage.getItem('commandHistory');
    if (savedHistory) {
      commandHistory = JSON.parse(savedHistory);
    }

    // Organize commands by category
    organizeCategories();

    // Set up keyboard shortcuts
    document.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    document.removeEventListener('keydown', handleKeyDown);
  });

  function organizeCategories() {
    const cats: Record<string, Command[]> = {};
    allCommands.forEach((cmd) => {
      if (!cats[cmd.category]) {
        cats[cmd.category] = [];
      }
      cats[cmd.category].push(cmd);
    });
    categories = cats;
  }

  function handleKeyDown(event: KeyboardEvent) {
    // Toggle palette with Ctrl+P or Ctrl+K
    if ((event.ctrlKey || event.metaKey) && (event.key === 'p' || event.key === 'k')) {
      event.preventDefault();
      showPalette = !showPalette;
      if (showPalette) {
        setTimeout(() => {
          const searchInput = document.querySelector('.search-input');
          if (searchInput instanceof HTMLInputElement) {
            searchInput.focus();
          }
        }, 0);
      }
      return;
    }

    // Handle palette navigation
    if (showPalette) {
      switch (event.key) {
        case 'Escape':
          event.preventDefault();
          showPalette = false;
          break;

        case 'ArrowUp':
          event.preventDefault();
          selectedIndex = Math.max(0, selectedIndex - 1);
          scrollToSelected();
          break;

        case 'ArrowDown':
          event.preventDefault();
          selectedIndex = Math.min(filteredCommands.length - 1, selectedIndex + 1);
          scrollToSelected();
          break;

        case 'Enter':
          event.preventDefault();
          if (filteredCommands[selectedIndex]) {
            void executeCommand(filteredCommands[selectedIndex]);
          }
          break;

        case 'Tab':
          event.preventDefault();
          // Cycle through categories
          break;
      }
    }
  }

  function scrollToSelected() {
    const selectedElement = document.querySelector('.command-item.selected');
    if (selectedElement) {
      selectedElement.scrollIntoView({
        block: 'nearest',
        behavior: 'smooth',
      });
    }
  }

  // Reactive statement to filter commands when search changes (Svelte 4 pattern)
  $: {
    if (!searchQuery.trim()) {
      // Show recent commands when search is empty
      filteredCommands = recentCommands.slice(0, 10);
    } else {
      // Filter commands based on search query
      const query = searchQuery.toLowerCase();
      filteredCommands = allCommands
        .filter(
          (cmd) =>
            cmd.name.toLowerCase().includes(query) ||
            cmd.category.toLowerCase().includes(query) ||
            (cmd.shortcut && cmd.shortcut.toLowerCase().includes(query))
        )
        .slice(0, 20);
    }
    selectedIndex = 0;
  }

  async function executeCommand(command: Command) {
    try {
      // Add to recent commands
      recentCommands = [command, ...recentCommands.filter((c) => c.id !== command.id)].slice(0, 10);

      // Save to localStorage
      localStorage.setItem('recentCommands', JSON.stringify(recentCommands));

      // Add to history
      commandHistory = [
        { command: command.name, timestamp: new Date().toISOString() },
        ...commandHistory,
      ].slice(0, 100);
      localStorage.setItem('commandHistory', JSON.stringify(commandHistory));

      // Execute command based on ID
      switch (command.id) {
        case 'new_project':
          await api.project.createNewProject();
          break;
        case 'open_project':
          await api.project.openProjectDialog();
          break;
        case 'save_project':
          await api.project.saveProject();
          break;
        case 'undo':
          await api.edit.undo();
          break;
        case 'redo':
          await api.edit.redo();
          break;
        case 'play':
          await api.transport.togglePlayback();
          break;
        case 'record':
          await api.transport.toggleRecord();
          break;
        // Add more command executions as needed
        default:
          console.log(`Executing command: ${command.name}`);
      }

      // Close palette
      showPalette = false;
      searchQuery = '';
    } catch (error) {
      console.error('Failed to execute command:', error);
    }
  }

  function clearHistory() {
    if (confirm('Clear command history?')) {
      recentCommands = [];
      commandHistory = [];
      localStorage.removeItem('recentCommands');
      localStorage.removeItem('commandHistory');
    }
  }

  function handleOverlayClick() {
    showPalette = false;
  }

  function handleCommandClick(command: Command) {
    void executeCommand(command);
  }

  function handleMouseEnterCommand(index: number) {
    selectedIndex = index;
  }

  function getCategoryStartIndex(categoryIndex: number): number {
    let startIndex = recentCommands.length;
    const categoryKeys = Object.keys(categories);
    for (let i = 0; i < categoryIndex; i++) {
      startIndex += categories[categoryKeys[i]].length;
    }
    return startIndex;
  }
</script>

<!-- Command Palette Modal -->
{#if showPalette}
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
  <div
    class="command-palette-overlay fixed inset-0 dark:bg-black dark:bg-opacity-50 flex items-start justify-center z-50 pt-20"
    on:click={handleOverlayClick}
    on:keydown={(e) => e.key === 'Escape' && (showPalette = false)}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      class="command-palette dark:bg-window rounded-lg shadow-2xl w-[600px] max-h-[70vh] overflow-hidden"
      on:click|stopPropagation
      on:keydown|stopPropagation
    >
      <!-- Search Input -->
      <div class="p-4 border-b dark:border-window-border">
        <div class="relative">
          <input
            type="text"
            bind:value={searchQuery}
            placeholder="Type a command or search..."
            class="search-input w-full px-4 py-3 pl-10 dark:bg-input dark:border-window-border rounded-lg text-lg dark:text-app-text focus:outline-none focus:ring-2 focus:ring-primary"
            autocomplete="off"
            spellcheck="false"
          />
          <div class="absolute left-3 top-3 text-gray-500">🔍</div>

          <!-- Shortcut hint -->
          <div class="absolute right-3 top-3 text-xs dark:text-gray-500">
            ↑↓ to navigate | Enter to execute | Esc to close
          </div>
        </div>
      </div>

      <!-- Command List -->
      <div class="command-list overflow-auto max-h-[60vh]">
        {#if !searchQuery.trim()}
          <!-- Recent Commands -->
          <div class="p-2">
            <div class="px-3 py-2 text-xs font-medium dark:text-gray-400">RECENT COMMANDS</div>
            {#each recentCommands as command, index (command.id)}
              <div
                class="command-item p-3 flex items-center gap-3 cursor-pointer rounded-lg mx-2"
                class:selected={selectedIndex === index}
                class:dark:bg-secondary={selectedIndex === index}
                class:hover:dark:bg-window-subtle={selectedIndex !== index}
                on:click={() => handleCommandClick(command)}
                on:keydown={(e) => e.key === 'Enter' && handleCommandClick(command)}
                on:mouseenter={() => handleMouseEnterCommand(index)}
                role="button"
                tabindex="0"
              >
                <div class="text-lg">{command.icon}</div>
                <div class="flex-1">
                  <div class="font-medium dark:text-gray-200">{command.name}</div>
                  <div class="text-xs dark:text-gray-400">{command.category}</div>
                </div>
                {#if command.shortcut}
                  <div class="shortcut px-2 py-1 text-xs dark:bg-menu rounded dark:text-gray-400">
                    {command.shortcut}
                  </div>
                {/if}
              </div>
            {/each}
          </div>

          <!-- All Commands by Category -->
          {#each Object.keys(categories) as category, catIdx (category)}
            <div class="category-section">
              <div
                class="px-3 py-2 text-xs font-medium dark:text-gray-400 sticky top-0 dark:bg-window"
              >
                {category.toUpperCase()}
              </div>
              {#each categories[category] as command, cmdIdx (command.id)}
                {@const globalIndex = getCategoryStartIndex(catIdx) + cmdIdx}
                <div
                  class="command-item p-3 flex items-center gap-3 cursor-pointer rounded-lg mx-2"
                  class:selected={selectedIndex === globalIndex}
                  class:dark:bg-secondary={selectedIndex === globalIndex}
                  class:hover:dark:bg-window-subtle={selectedIndex !== globalIndex}
                  on:click={() => handleCommandClick(command)}
                  on:keydown={(e) => e.key === 'Enter' && handleCommandClick(command)}
                  on:mouseenter={() => handleMouseEnterCommand(globalIndex)}
                  role="button"
                  tabindex="0"
                >
                  <div class="text-lg">{command.icon}</div>
                  <div class="flex-1">
                    <div class="font-medium dark:text-gray-200">{command.name}</div>
                    <div class="text-xs dark:text-gray-400">{command.category}</div>
                  </div>
                  {#if command.shortcut}
                    <div class="shortcut px-2 py-1 text-xs dark:bg-menu rounded dark:text-gray-400">
                      {command.shortcut}
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          {/each}
        {:else}
          <!-- Search Results -->
          <div class="p-2">
            <div class="px-3 py-2 text-xs font-medium dark:text-gray-400">
              {filteredCommands.length} COMMANDS FOUND
            </div>
            {#each filteredCommands as command, index (command.id)}
              <div
                class="command-item p-3 flex items-center gap-3 cursor-pointer rounded-lg mx-2"
                class:selected={selectedIndex === index}
                class:dark:bg-secondary={selectedIndex === index}
                class:hover:dark:bg-window-subtle={selectedIndex !== index}
                on:click={() => handleCommandClick(command)}
                on:keydown={(e) => e.key === 'Enter' && handleCommandClick(command)}
                on:mouseenter={() => handleMouseEnterCommand(index)}
                role="button"
                tabindex="0"
              >
                <div class="text-lg">{command.icon}</div>
                <div class="flex-1">
                  <div class="font-medium dark:text-gray-200">{command.name}</div>
                  <div class="text-xs dark:text-gray-400">{command.category}</div>
                </div>
                {#if command.shortcut}
                  <div class="shortcut px-2 py-1 text-xs dark:bg-menu rounded dark:text-gray-400">
                    {command.shortcut}
                  </div>
                {/if}
              </div>
            {:else}
              <div class="p-8 text-center dark:text-gray-500">
                <div class="text-4xl mb-4">🔍</div>
                <div class="text-lg mb-2">No commands found</div>
                <div class="text-sm">Try a different search term</div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div
        class="footer p-3 border-t dark:border-window-border flex justify-between text-xs dark:text-gray-500"
      >
        <div>
          {#if searchQuery.trim()}
            Showing {filteredCommands.length} of {allCommands.length} commands
          {:else}
            {allCommands.length} commands available
          {/if}
        </div>
        <button
          on:click={clearHistory}
          class="px-2 py-1 dark:bg-secondary rounded hover:opacity-80"
        >
          Clear History
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .command-palette-overlay {
    backdrop-filter: blur(8px);
    animation: fadeIn 0.2s ease-out;
  }

  .command-palette {
    animation: slideDown 0.2s ease-out;
  }

  .command-item {
    transition: background-color 0.15s;
  }

  .shortcut {
    font-family:
      'SF Mono', 'Monaco', 'Inconsolata', 'Fira Mono', 'Droid Sans Mono', 'Courier New', monospace;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
