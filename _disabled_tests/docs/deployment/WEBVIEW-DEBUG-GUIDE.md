# Tauri Webview White Screen - Debugging Guide

**Status:** App running, webview not executing JavaScript
**Date:** 2025-11-10

---

## ✅ **Confirmed Working**

```
✅ Tauri Window: OPEN (PID confirmed, visible in wmctrl)
✅ Rust Backend: 100% operational (all services initialized)
✅ Vite Dev Server: Running on http://localhost:5173
✅ HTML Served: Correct structure with <div id="app"></div>
✅ TypeScript: Compiles with 0 errors
✅ Database: PostgreSQL + Meilisearch connected
✅ Ports: 5173 (Vite), 5433 (PostgreSQL), 7700 (Meilisearch)
```

---

## ⚠️ **The Issue**

**Symptom:** Blank white screen in Tauri window

**Root Cause:** JavaScript module (main.ts) NOT executing in webview
- HTML loads ✅
- `<div id="app"></div>` rendered ✅
- But Svelte doesn't mount ❌

---

## 🔍 **Debugging Steps (In Order)**

### Step 1: Check Webview Console (CRITICAL)

**Option A: If DevTools Available**
```bash
# Try opening DevTools directly in Tauri window:
# 1. Click on the white Tauri window
# 2. Press F12
# 3. Or Right-click → Inspect Element
```

**Option B: Enable WebKit Inspector**
```bash
pkill -9 -f tauri
export WEBKIT_INSPECTOR=1
cd /home/dojevou/projects/midi-software-center/app
pnpm tauri dev
```

**What to Look For:**
- Console errors (red text)
- Module loading failures
- CSP violations
- CORS errors

---

### Step 2: Test in Regular Browser

```bash
# While pnpm tauri dev is running:
firefox http://localhost:5173

# If it works in Firefox but not Tauri:
# → Webview-specific issue (CSP, module loading, etc.)

# If it ALSO white screen in Firefox:
# → Frontend compilation issue
```

---

### Step 3: Check Console Logs

```bash
# Check if main.ts console.log statements appear:
tail -f /tmp/tauri-app.log | grep "Svelte"

# Expected output:
# 🚀 Starting Svelte app initialization
# 📦 Svelte App imported, mounting to #app
# ✅ Svelte app mounted successfully

# If missing → JavaScript not executing
```

---

### Step 4: Verify Module Loading

Check if TypeScript modules are being served:

```bash
# Test main.ts directly:
curl -s http://localhost:5173/src/main.ts | head -20

# Expected: TypeScript code transformed to JavaScript

# Test App.svelte:
curl -s http://localhost:5173/src/App.svelte | head -20

# Expected: Svelte component code
```

---

## 🛠️ **Potential Fixes**

### Fix 1: Clear All Caches

```bash
cd /home/dojevou/projects/midi-software-center/app

# Clear Vite cache
rm -rf node_modules/.vite

# Clear build artifacts
rm -rf dist
rm -rf src-tauri/target/debug/build

# Reinstall (if needed)
pnpm install --force

# Restart
pnpm tauri dev
```

---

### Fix 2: Disable CSP Temporarily

Edit `app/src-tauri/tauri.conf.json`:

```json
{
  "app": {
    "security": {
      "csp": null  // Already set - good for debugging
    }
  }
}
```

---

### Fix 3: Try Production Build

Sometimes dev mode has issues that production doesn't:

```bash
cd /home/dojevou/projects/midi-software-center/app
pnpm tauri build

# Run the built binary:
./src-tauri/target/release/midi-software-center
```

---

### Fix 4: Simplify App.svelte

Create absolute minimal version:

```bash
cat > /home/dojevou/projects/midi-software-center/app/src/App.svelte << 'EOF'
<h1 style="color: red; font-size: 50px; padding: 50px;">
  ✅ IT WORKS!
</h1>
EOF

# Restart Tauri
pkill -f tauri && pnpm tauri dev
```

If THIS works → Original App.svelte has a component loading issue
If THIS fails → Deeper webview/module issue

---

### Fix 5: Check Tauri Config Paths

Verify `app/src-tauri/tauri.conf.json`:

```json
{
  "build": {
    "devUrl": "http://localhost:5173",  // Must match Vite port
    "frontendDist": "../dist"            // Correct relative path
  }
}
```

---

## 📊 **Current Configuration**

**Tauri Config (`app/src-tauri/tauri.conf.json`):**
```json
{
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "app": {
    "security": {
      "csp": null  // CSP disabled for debugging ✅
    }
  }
}
```

**Main Entry (`app/src/main.ts`):**
```typescript
console.log('🚀 Starting Svelte app initialization');
import './app.css';
import App from './App.svelte';

const app = new App({
  target: document.getElementById('app')!,
});

export default app;
```

---

## 🔬 **Advanced Debugging**

### Check Webview Version

```bash
# Linux WebKit version:
ldd /usr/lib/x86_64-linux-gnu/webkit2gtk-4.0/libwebkit2gtkinjectedbundle.so 2>/dev/null || \
echo "WebKit2GTK info not available"
```

### Check Tauri Logs

```bash
# Full logs with timestamps:
RUST_LOG=trace pnpm tauri dev 2>&1 | tee /tmp/tauri-trace.log
```

### Network Tab (if DevTools available)

1. Open DevTools (F12)
2. Go to Network tab
3. Reload page (Ctrl+R)
4. Check for:
   - Failed requests (red)
   - 404 errors
   - CORS issues

---

## 📝 **Expected Behavior**

When working correctly, you should see:

**In Tauri Window:**
- Menu bar (File, Edit, View, etc.)
- Status bar at bottom
- Workspace with 4 windows:
  - DAW Window
  - Mixer Window
  - Database Window
  - Pipeline Window

**In Console (if accessible):**
```
🚀 Starting Svelte app initialization
📦 Svelte App imported, mounting to #app
✅ Svelte app mounted successfully
```

---

## 🐛 **Known Webview Issues**

### Issue 1: Module Type Mismatch
**Symptom:** `Unexpected token 'export'` error
**Fix:** Ensure Vite is serving ES modules correctly

### Issue 2: MIME Type Issues
**Symptom:** `MIME type ('text/html') is not executable`
**Fix:** Check Vite is serving `.ts` files as JavaScript

### Issue 3: Path Resolution
**Symptom:** `Cannot find module '@tauri-apps/api'`
**Fix:** Check node_modules and reinstall if needed

---

## ✅ **Success Indicators**

You'll know it's fixed when:

1. **Tauri window shows UI** (not blank white)
2. **Console logs appear** (main.ts logging statements)
3. **Components render** (menu bar, windows, status bar visible)
4. **No errors in webview console**

---

## 📞 **If Still Stuck**

**Collect this information:**

```bash
# 1. Webview console errors (F12 → Console tab)
# Screenshot or copy any red error messages

# 2. Network requests (F12 → Network tab)
# Check if main.ts and App.svelte load successfully

# 3. Full logs
cat /tmp/tauri-app.log > tauri-debug-$(date +%Y%m%d-%H%M%S).log

# 4. System info
uname -a
lsb_release -a 2>/dev/null || cat /etc/os-release
```

---

**Next Step:** Open Tauri window and press F12 to check webview console!
