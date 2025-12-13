# 🖥️ Running on CPU-Only Systems (No GPU)

**Date:** 2025-11-10
**Issue:** White screen in Tauri app on systems without GPU
**Root Cause:** WebKit hardware acceleration fails with software rendering (llvmpipe)

---

## 🎯 Quick Solution

### **Option 1: Use Makefile (Recommended)**
```bash
make dev-cpu
```

### **Option 2: Use Launch Script**
```bash
cd app
./launch-cpu-only.sh
```

### **Option 3: Manual Launch**
```bash
cd app
WEBKIT_DISABLE_COMPOSITING_MODE=1 \
WEBKIT_DISABLE_DMABUF_RENDERER=1 \
LIBGL_ALWAYS_SOFTWARE=1 \
pnpm tauri dev
```

---

## 🔍 Diagnosis

### **Check if You Have a GPU:**
```bash
glxinfo | grep -i "opengl renderer"
```

**CPU-only output:**
```
OpenGL renderer string: llvmpipe (LLVM 20.1.2, 256 bits)
```

**GPU-enabled output:**
```
OpenGL renderer string: Mesa Intel(R) HD Graphics 620 (KBL GT2)
```

---

## 🛠️ Technical Details

### **Why WebKit Fails:**
- WebKit tries hardware acceleration by default
- On CPU-only systems, it falls back to software rendering
- The fallback uses DMA-BUF and compositing features
- These features don't work correctly with llvmpipe
- Result: White screen (HTML loads, JavaScript doesn't execute)

### **Environment Variables:**

| Variable | Purpose |
|----------|---------|
| `WEBKIT_DISABLE_COMPOSITING_MODE=1` | Disable WebKit compositing layers |
| `WEBKIT_DISABLE_DMABUF_RENDERER=1` | Disable DMA-BUF memory sharing |
| `LIBGL_ALWAYS_SOFTWARE=1` | Force software OpenGL rendering |
| `GALLIUM_DRIVER=llvmpipe` | Explicitly use llvmpipe driver |

---

## 📝 Permanent Configuration

### **For Development:**
Add to your shell profile (`~/.bashrc` or `~/.zshrc`):

```bash
# MIDI Software Center - CPU-only rendering
alias midi-dev='cd ~/projects/midi-software-center/app && \
  WEBKIT_DISABLE_COMPOSITING_MODE=1 \
  WEBKIT_DISABLE_DMABUF_RENDERER=1 \
  LIBGL_ALWAYS_SOFTWARE=1 \
  pnpm tauri dev'
```

### **For Production Builds:**
The environment variables only affect the dev webview. Production builds use the system's default rendering backend.

For production on CPU-only systems, users should set these variables before launching:

```bash
export WEBKIT_DISABLE_COMPOSITING_MODE=1
export WEBKIT_DISABLE_DMABUF_RENDERER=1
./midi-software-center
```

---

## ✅ Verification

### **1. Check Vite Server:**
```bash
curl -I http://localhost:5173/
# Should return: HTTP/1.1 200 OK
```

### **2. Check Backend:**
Look for in console output:
```
✅ Pipeline database connection established
✅ DAW database connection pool initialized
✅ MIDI manager initialized
✅ Sequencer engine initialized
✅ Application setup complete
```

### **3. Check Frontend:**
Look for Vite output:
```
VITE v5.4.21  ready in 1219 ms
➜  Local:   http://localhost:5173/
```

### **4. Check GUI:**
- Window should open (not white screen)
- Menu bar visible
- 4 windows displayed: DAW, Mixer, Database, Pipeline
- Dark theme applied

---

## 🚨 Still Not Working?

### **Check Browser Rendering:**
```bash
# Open in web browser to test Vite directly
xdg-open http://localhost:5173/
```

If the browser shows the GUI correctly, the issue is WebKit-specific.

### **Enable Debug Logging:**
```bash
cd app
WEBKIT_INSPECTOR=1 \
RUST_LOG=debug \
WEBKIT_DISABLE_COMPOSITING_MODE=1 \
WEBKIT_DISABLE_DMABUF_RENDERER=1 \
pnpm tauri dev
```

### **Check for JavaScript Errors:**
- Right-click in Tauri window → "Inspect Element" (if available)
- Or check browser console if testing via `xdg-open`

---

## 📚 Related Documents

- `WHITE-SCREEN-FIX-SOLUTION.md` - Original white screen diagnosis
- `WEBVIEW-DEBUG-GUIDE.md` - Complete troubleshooting guide
- `vite.config.ts` - Frontend configuration (line 16: `base: './'`)
- `app/src-tauri/tauri.conf.json` - Tauri configuration

---

## 🎓 Lessons Learned

1. **Always test on CPU-only systems** if distributing to diverse hardware
2. **WebKit != Chrome** - Different rendering backends have different requirements
3. **Software rendering needs explicit configuration** in Tauri/WebKit
4. **llvmpipe is common** on VMs, headless systems, and laptops without dedicated GPUs
5. **Environment variables are the solution** - no code changes needed

---

## 🚀 Future Improvements

- [ ] Auto-detect CPU-only systems and set variables automatically
- [ ] Add GPU detection to app startup script
- [ ] Create .desktop file with proper environment variables for production
- [ ] Add GPU info to debug/about panel in GUI

---

**Status:** ✅ **RESOLVED**
**Workaround:** Use `make dev-cpu` or set environment variables
**Permanent Fix:** Tauri team may improve software rendering support in future releases
