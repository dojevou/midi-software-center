# 🎯 EXACT KILO CODE PROMPTS

## Step-by-Step Generation Instructions

---

## PHASE 1: FOUNDATION (Part 1)

### Prompt for Kilo Code:

```
Generate the frontend foundation for a Tauri + Svelte + TypeScript MIDI application using the specifications in KILO-CODE-GUIDE-V2-CORRECTED.md.

CRITICAL REQUIREMENTS:
1. Follow the guide EXACTLY - every type, every field name, every command signature
2. Use TypeScript strict mode
3. Map Rust Option<T> to TypeScript T | undefined (NEVER | null)
4. Use snake_case for all JSON field names (serde rename)
5. All Tauri commands use snake_case (e.g., midi_list_devices)
6. Event names use kebab-case (e.g., playback-started)

GENERATE THESE FILES FROM SECTIONS 0-5:
- package.json (Section 3.1)
- tsconfig.json (Section 3.2) 
- vite.config.ts (Section 3.3)
- svelte.config.js (Section 3.4)
- src/lib/types.ts (Section 4 - all 35 types)
- src/lib/api.ts (Section 5 - all 79 commands)

VALIDATION:
- Verify all 35 types are defined
- Verify all 79 commands are implemented
- Ensure strict TypeScript compliance
- No 'any' types allowed

DATABASE CONNECTION:
- Use port 5433 (NOT 5432)
- DATABASE_URL=postgresql://midiuser:145278963@localhost:5433/midi_library
```

### Expected Output:
- 6 files generated
- ~800 lines of code
- All dependencies in package.json
- All types matching Rust backend
- All 79 API commands typed

### Verification:
```bash
cd app
pnpm install
pnpm tsc --noEmit  # Should pass with 0 errors
```

---

## PHASE 2: STATE MANAGEMENT (Part 2)

### Prompt for Kilo Code:

```
Generate the state management layer for the MIDI application using KILO-CODE-GUIDE-V2-PART2.md.

BUILD ON PHASE 1:
- Use the types from src/lib/types.ts
- Use the API from src/lib/api.ts

CRITICAL REQUIREMENTS:
1. All event listeners must cleanup on component destroy
2. Stores must be properly typed
3. Use Svelte's writable/derived for stores
4. Debounce search inputs (300ms)
5. Subscribe/unsubscribe pattern for memory safety

GENERATE THESE FILES FROM SECTIONS 6-8:
- src/lib/events.ts (Section 6 - 14 event listeners)
- src/lib/stores/playbackStore.ts (Section 7.1)
- src/lib/stores/projectStore.ts (Section 7.2)
- src/lib/stores/databaseStore.ts (Section 7.3)
- src/lib/stores/uiStore.ts (Section 7.4)
- src/lib/utils/formatters.ts (Section 8.1)
- src/lib/utils/constants.ts (Section 8.2)

VALIDATION:
- All event listeners have cleanup functions
- All stores properly typed
- No memory leaks from subscriptions

EVENT CLEANUP PATTERN:
Always use this pattern:
```typescript
onMount(async () => {
  const unlisten = await listen('event-name', handler);
  return () => { unlisten(); };
});
```
```

### Expected Output:
- 7 files generated
- ~600 lines of code
- 4 complete Svelte stores
- 14 event listeners with cleanup
- Utility functions

### Verification:
```bash
pnpm tsc --noEmit  # Should still pass
```

---

## PHASE 3: UI COMPONENTS (Part 3)

### Prompt for Kilo Code:

```
Generate the complete UI layer for the MIDI application using KILO-CODE-GUIDE-V2-PART3-FINAL.md.

BUILD ON PHASES 1 & 2:
- Import types from src/lib/types.ts
- Import API from src/lib/api.ts
- Import stores from src/lib/stores/*
- Import events from src/lib/events.ts

CRITICAL REQUIREMENTS:
1. WindowBase component handles ALL dragging/resizing logic
2. All windows use WindowBase as wrapper
3. Subscribe to stores using $ syntax
4. Cleanup all subscriptions in onDestroy
5. Follow dark theme CSS variables

GENERATE THESE FILES FROM SECTIONS 9-15:

BASE COMPONENTS (Section 9):
- src/lib/components/WindowBase.svelte
- src/lib/components/MenuBar.svelte
- src/lib/components/StatusBar.svelte

WINDOW COMPONENTS (Section 10):
- src/lib/windows/DAWWindow.svelte
- src/lib/windows/MixerWindow.svelte
- src/lib/windows/DatabaseWindow.svelte
- src/lib/windows/PipelineWindow.svelte

ROOT APPLICATION (Section 11):
- src/App.svelte
- src/main.ts

STYLES (Section 12):
- src/app.css (with all CSS variables)

WINDOW PATTERN:
All windows must follow this structure:
```svelte
<script lang="ts">
  import WindowBase from '$lib/components/WindowBase.svelte';
  // ... other imports
</script>

<WindowBase windowId="unique-id" title="Window Name" width={800} height={600}>
  <!-- Window content here -->
</WindowBase>
```

KNOWN LIMITATIONS:
These 5 backend commands don't exist yet - add try/catch or comment out:
- set_loop_enabled
- set_loop_range
- set_metronome_enabled
- set_metronome_volume
- get_transport_info
```

### Expected Output:
- 12 files generated
- ~1,800 lines of code
- Fully functional draggable windows
- Complete dark theme styling
- All components connected to stores

### Verification:
```bash
pnpm tsc --noEmit  # Should pass
pnpm build         # Should build successfully
pnpm tauri dev     # Should launch app
```

---

## COMPLETE WORKFLOW

### Full Command Sequence:

```bash
# PHASE 1: Foundation
# Give Kilo Code the Phase 1 prompt with KILO-CODE-GUIDE-V2-CORRECTED.md
cd app
pnpm install
pnpm tsc --noEmit  # Verify types

# PHASE 2: State Management  
# Give Kilo Code the Phase 2 prompt with KILO-CODE-GUIDE-V2-PART2.md
pnpm tsc --noEmit  # Verify still compiles

# PHASE 3: UI Components
# Give Kilo Code the Phase 3 prompt with KILO-CODE-GUIDE-V2-PART3-FINAL.md
pnpm build         # Full build test
pnpm tauri dev     # Launch app

# VALIDATION (from Part 3, Section 13-15)
./scripts/validate-types.ts      # Type validation
./scripts/validate-api.sh        # API call validation
./scripts/post-deploy-check.sh   # Health checks
```

---

## TROUBLESHOOTING

### If Kilo Code asks for clarification:

**Q: "Which port for PostgreSQL?"**
A: Port 5433 (NOT 5432)

**Q: "How to handle Option<T> types?"**
A: TypeScript T | undefined (NEVER T | null)

**Q: "What about the 5 missing commands?"**
A: Add try/catch wrappers or comment out for now. Implementation code is in QUANTUM-ANALYZER-VERIFICATION-REPORT.md

**Q: "Should I use any specific theme?"**
A: Yes, dark theme with CSS variables defined in Part 3, Section 12

**Q: "How to handle event listeners?"**
A: Always cleanup in onDestroy() - see Part 2, Section 6 for pattern

**Q: "Window management approach?"**
A: All windows use WindowBase component wrapper

---

## ALTERNATIVE: SINGLE PROMPT (All-in-One)

If Kilo Code can handle it, you can try this single comprehensive prompt:

```
Generate a complete Tauri + Svelte + TypeScript frontend for a MIDI application using these 3 guide files in sequence:

1. KILO-CODE-GUIDE-V2-CORRECTED.md (foundation)
2. KILO-CODE-GUIDE-V2-PART2.md (state management)
3. KILO-CODE-GUIDE-V2-PART3-FINAL.md (UI components)

CRITICAL REQUIREMENTS:
- TypeScript strict mode
- Rust Option<T> → TypeScript T | undefined (NOT null)
- snake_case for JSON fields and Tauri commands
- kebab-case for event names
- PostgreSQL port 5433
- Dark theme with CSS variables
- All event listeners must cleanup
- WindowBase wrapper for all windows

GENERATE ALL FILES from all 3 parts:
- Config files (package.json, tsconfig.json, etc.)
- Types (35 TypeScript types)
- API client (79 Tauri commands)
- Event listeners (14 events)
- Stores (4 Svelte stores)
- Components (12 UI files)
- Root app (App.svelte, main.ts)
- Styles (app.css)

KNOWN ISSUES:
5 backend commands don't exist yet - add try/catch:
- set_loop_enabled, set_loop_range
- set_metronome_enabled, set_metronome_volume  
- get_transport_info

Follow the guides EXACTLY. Every type, field name, and command signature must match the specifications.
```

---

## SUCCESS CRITERIA

After all 3 phases, you should have:

✅ **30 files generated**:
- 4 config files
- 1 types file (35 types)
- 1 API file (79 commands)
- 1 events file (14 listeners)
- 4 store files
- 2 utility files
- 3 base components
- 4 window components
- 2 root files (App.svelte, main.ts)
- 1 CSS file
- 7 test/validation scripts

✅ **Compilation**: `pnpm tsc --noEmit` passes with 0 errors

✅ **Build**: `pnpm build` completes successfully

✅ **Launch**: `pnpm tauri dev` starts app without errors

✅ **Functionality**:
- All 4 windows draggable and resizable
- Transport controls work (play/pause/stop)
- Database search works
- Stores update reactively
- No console errors

---

**Use Phase 1-3 prompts in sequence for best results.**

Each phase builds on the previous, and you can verify after each step.

**Created**: 2025-11-09
**Status**: Ready for Kilo Code
