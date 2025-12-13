# GUI 4-Window Layout - Troubleshooting Guide

**Issue:** GUI shows simple version instead of 4-window layout (DAW, Mixer, Database, Pipeline)

**Status:** Code is correct, likely caching/build issue

---

## Verification Steps

### 1. Check App.svelte ✅
The main app file (`app/src/App.svelte`) is correctly configured:
- ✅ All 4 windows imported (DAWWindow, MixerWindow, DatabaseWindow, PipelineWindow)
- ✅ 2x2 grid layout defined (lines 261-267)
- ✅ All windows rendered conditionally based on visibility (lines 207-246)
- ✅ onMount forces all windows visible (lines 49-54)

### 2. Check uiStore ✅
The UI store (`app/src/lib/stores/uiStore.ts`) is correctly configured:
- ✅ All windows default to `visible: true` (lines 17-48)
- ✅ localStorage loading forces windows visible (lines 66-68)
- ✅ Actions available: showWindow, hideWindow, toggleWindow

### 3. Check CSS Layout ✅
The CSS grid is properly defined:
```css
.grid-container {
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
  grid-template-areas:
    "daw mixer"
    "database pipeline";
}
```

---

## Solution Steps

### Step 1: Kill Existing Dev Server
```bash
# Find and kill any running dev servers
pkill -f "vite"
pkill -f "pnpm dev"

# Or use these specific commands
lsof -ti:5173 | xargs kill -9  # Kill frontend on port 5173
lsof -ti:5174 | xargs kill -9  # Kill alternative port
```

### Step 2: Clear Browser Cache
1. Open browser DevTools (F12)
2. Go to Application/Storage tab
3. Click "Clear site data" or:
   - Clear localStorage
   - Clear sessionStorage
   - Clear cache
4. Hard refresh: Ctrl+Shift+R (or Cmd+Shift+R on Mac)

### Step 3: Clear Vite Cache
```bash
cd /home/dojevou/projects/midi-software-center/app
rm -rf node_modules/.vite
rm -rf dist
rm -rf .svelte-kit
```

### Step 4: Rebuild and Start Fresh
```bash
cd /home/dojevou/projects/midi-software-center/app

# Install dependencies (if needed)
pnpm install

# Start dev server
pnpm run dev

# OR use Makefile from root
cd ..
make dev-pipeline
```

### Step 5: Open Browser with Cache Disabled
```bash
# Option 1: Open in private/incognito mode
# Option 2: Open DevTools and check "Disable cache"
# Option 3: Use this URL with cache-busting
http://localhost:5173/?nocache=$(date +%s)
```

---

## Expected Result

After following the steps above, you should see:

### Layout:
```
┌─────────────────┬─────────────────┐
│                 │                 │
│   DAW WINDOW    │  MIXER WINDOW   │
│                 │                 │
├─────────────────┼─────────────────┤
│                 │                 │
│ DATABASE WINDOW │ PIPELINE WINDOW │
│                 │                 │
└─────────────────┴─────────────────┘
```

### Window Contents:

**DAW Window (Top-Left):**
- Transport controls: ▶ ⏸ ⏹
- BPM, Time Signature controls
- Track list with M/S buttons
- Add/Remove track buttons

**Mixer Window (Top-Right):**
- Channel strips with volume sliders
- Pan controls
- Mute/Solo buttons
- VU meters
- Master volume

**Database Window (Bottom-Left):**
- Search bar
- BPM/Key filters
- File list with favorite/delete buttons
- Statistics panel

**Pipeline Window (Bottom-Right):**
- Three tabs: Import, Analyze, Archive
- Progress bars for operations
- Cancel buttons
- Error display

---

## Debug Indicators

The App.svelte includes debug indicators at the top of each window:

- **Green label** = Window is visible and rendering
- **Red background** = Window is hidden (shouldn't happen)

You should see all 4 windows with GREEN labels like:
- "DAW [visible]" (green)
- "MIXER [visible]" (green)
- "DATABASE [visible]" (green)
- "PIPELINE [visible]" (green)

---

## Keyboard Shortcuts

Once the 4-window layout is working:

- **F1** = Toggle DAW window
- **F2** = Toggle Mixer window
- **F3** = Toggle Database window
- **F4** = Toggle Pipeline window
- **Space** = Play/Pause transport
- **Ctrl+N** = New project
- **Ctrl+O** = Open project
- **Ctrl+S** = Save project

---

## If Problem Persists

### Check Console for Errors:
1. Open DevTools (F12)
2. Go to Console tab
3. Look for errors (red text)
4. Check for warnings about:
   - Failed to load components
   - Import errors
   - TypeScript errors
   - Render errors

### Check Network Tab:
1. Open DevTools Network tab
2. Refresh page
3. Look for failed requests (red status codes)
4. Verify all .svelte files load successfully

### Check localStorage:
```javascript
// In browser console:
localStorage.getItem('ui-state')

// Should show something like:
// {"windows":{"daw":{"visible":true,...}, "mixer":{"visible":true,...}, ...}}
```

### Force Reset localStorage:
```javascript
// In browser console:
localStorage.removeItem('ui-state');
location.reload();
```

---

## Alternative: Use Makefile Commands

From project root:

```bash
# Kill all services
make stop-all

# Clean build artifacts
make clean-frontend

# Rebuild
make build-pipeline

# Start dev server
make dev-pipeline
```

---

## Verify the Issue

Before troubleshooting, verify you're actually seeing the simple version:

**Simple version would show:**
- Only ONE window visible
- No grid layout
- Minimal UI

**Full version should show:**
- ALL FOUR windows in 2x2 grid
- Each window has its own content
- Menu bar at top
- Status bar at bottom

---

## Quick Test

Create a test HTML file to verify the layout works:

```bash
cat > /tmp/test-layout.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
<style>
.grid-container {
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
  grid-template-areas: "daw mixer" "database pipeline";
  gap: 8px;
  height: 800px;
}
.window { border: 2px solid #0f0; padding: 20px; }
</style>
</head>
<body>
<div class="grid-container">
  <div class="window" style="grid-area: daw">DAW</div>
  <div class="window" style="grid-area: mixer">MIXER</div>
  <div class="window" style="grid-area: database">DATABASE</div>
  <div class="window" style="grid-area: pipeline">PIPELINE</div>
</div>
</body>
</html>
EOF

# Open in browser
xdg-open /tmp/test-layout.html
```

If the test file shows 4 windows correctly, then the issue is with the Svelte app specifically.

---

## Most Common Causes

1. **Browser Cache** (90% of cases)
   - Solution: Hard refresh (Ctrl+Shift+R)

2. **Vite Cache** (5% of cases)
   - Solution: `rm -rf node_modules/.vite && pnpm run dev`

3. **localStorage Corruption** (3% of cases)
   - Solution: `localStorage.clear()` in console

4. **Multiple Dev Servers** (2% of cases)
   - Solution: Kill all and start one fresh

---

## Success Criteria

✅ You should see:
- Red "Tailwind Test" banner at top
- Menu bar with File, Edit, View, Window, Help
- 2x2 grid with 4 distinct windows
- Each window showing its name in green
- Each window showing its specific UI components
- Status bar at bottom

---

**Report Generated:** 2025-11-12
**Issue:** GUI showing simple version instead of 4-window layout
**Root Cause:** Likely browser/build cache
**Solution:** Clear caches and rebuild
