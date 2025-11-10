# ⚡ KILO CODE - QUICK START

## Copy-Paste These 3 Prompts Into Kilo Code

---

## 📋 PHASE 1: Foundation

**Copy this prompt into Kilo Code:**

```
Generate frontend foundation from KILO-CODE-GUIDE-V2-CORRECTED.md

Requirements:
- TypeScript strict mode
- Rust Option<T> → TypeScript T | undefined
- PostgreSQL port 5433
- Generate: package.json, tsconfig.json, vite.config.ts, svelte.config.js
- Generate: src/lib/types.ts (35 types)
- Generate: src/lib/api.ts (79 commands)

Follow guide EXACTLY. Every type and command must match specifications.
```

**Verify:**
```bash
cd app && pnpm install && pnpm tsc --noEmit
```

---

## 📋 PHASE 2: State Management

**Copy this prompt into Kilo Code:**

```
Generate state management from KILO-CODE-GUIDE-V2-PART2.md

Requirements:
- All event listeners cleanup on destroy
- Use Svelte writable/derived stores
- Generate: src/lib/events.ts (14 events)
- Generate: 4 stores (playback, project, database, ui)
- Generate: src/lib/utils/* (formatters, constants)

Event cleanup pattern required:
onMount(async () => {
  const unlisten = await listen('event', handler);
  return () => unlisten();
});
```

**Verify:**
```bash
pnpm tsc --noEmit
```

---

## 📋 PHASE 3: UI Components

**Copy this prompt into Kilo Code:**

```
Generate UI components from KILO-CODE-GUIDE-V2-PART3-FINAL.md

Requirements:
- WindowBase handles all drag/resize
- Dark theme CSS variables
- Generate: 3 base components (WindowBase, MenuBar, StatusBar)
- Generate: 4 windows (DAW, Mixer, Database, Pipeline)
- Generate: App.svelte, main.ts, app.css

Missing commands (add try/catch):
- set_loop_enabled, set_loop_range
- set_metronome_enabled, set_metronome_volume
- get_transport_info
```

**Verify:**
```bash
pnpm build && pnpm tauri dev
```

---

## ✅ Done!

**3 simple prompts → Complete frontend**

If Kilo Code works better with one prompt, use the "ALTERNATIVE: SINGLE PROMPT" from KILO-CODE-PROMPTS.md

---

**Full Details**: See KILO-CODE-PROMPTS.md
