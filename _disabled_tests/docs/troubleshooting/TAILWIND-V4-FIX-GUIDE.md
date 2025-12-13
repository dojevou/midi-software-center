# Tailwind CSS v4 Fix Guide - MIDI Software Center

**Issue**: White screen caused by Tailwind v4 not processing CSS correctly
**Impact**: 77 `dark:` classes across 8 components are not being compiled
**Environment**: Tauri 2.7 + Svelte 4.2 + Vite 5.0 + Tailwind v4.1.17

---

## 🔍 Phase 1: Diagnosis (5 minutes)

### Step 1.1: Check Current Tailwind Installation
```bash
cd /home/dojevou/projects/midi-software-center/app
pnpm list tailwindcss
```
**Expected**: `tailwindcss 4.1.17` (currently installed)

### Step 1.2: Verify PostCSS Configuration
```bash
cat postcss.config.js
```
**Current content** (should show):
```javascript
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
```

### Step 1.3: Check if Tailwind v4 Config File Exists
```bash
ls -la | grep tailwind
```
**Issue**: Tailwind v4 doesn't use `tailwind.config.js` by default - it uses `@theme` in CSS files

### Step 1.4: Start Dev Server and Check Browser
```bash
cd /home/dojevou/projects/midi-software-center/app
pnpm dev
```
Open browser to `http://localhost:5173/` and:
1. Press F12 → Network tab
2. Verify `app.css` loads
3. Check if Tailwind utilities are injected
4. Inspect an element with `dark:bg-menu` class
5. See if the dark mode styles are present in computed CSS

---

## 🔧 Phase 2: Fix Tailwind v4 Configuration (15 minutes)

### Option A: Fix Tailwind v4 Setup (Recommended First Try)

#### Step 2A.1: Install Tailwind v4 PostCSS Plugin
Tailwind v4 requires a different PostCSS plugin:

```bash
cd /home/dojevou/projects/midi-software-center/app
pnpm add -D @tailwindcss/postcss
```

#### Step 2A.2: Update PostCSS Configuration
Edit `app/postcss.config.js`:

```javascript
export default {
  plugins: {
    '@tailwindcss/postcss': {},
    autoprefixer: {},
  },
}
```

#### Step 2A.3: Verify CSS Import Syntax
Check `app/src/app.css` has correct v4 syntax (should already be correct):

```css
/* Tailwind CSS v4 imports */
@import "tailwindcss";

/* Tailwind CSS v4 theme configuration */
@theme {
  --color-app-bg: #1e1e1e;
  --color-app-text: #e0e0e0;
  /* ... rest of theme */
}
```

#### Step 2A.4: Test the Fix
```bash
# Stop existing dev server (Ctrl+C)
pnpm dev
```

Open `http://localhost:5173/` and check:
- [ ] Dark background visible (#1e1e1e)
- [ ] Menu bar with proper styling
- [ ] Text is white on dark backgrounds
- [ ] Window borders visible

---

### Option B: Downgrade to Tailwind v3 (Stable Alternative)

If Option A doesn't work, use the stable Tailwind v3:

#### Step 2B.1: Uninstall Tailwind v4
```bash
cd /home/dojevou/projects/midi-software-center/app
pnpm remove tailwindcss
pnpm remove @tailwindcss/postcss  # if installed from Option A
```

#### Step 2B.2: Install Tailwind v3
```bash
pnpm add -D tailwindcss@3.4.17
```

#### Step 2B.3: Create Tailwind v3 Config
```bash
npx tailwindcss init -p
```

This creates `tailwind.config.js`. Edit it:

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx,svelte}",
  ],
  darkMode: 'class', // Enable dark mode with class strategy
  theme: {
    extend: {
      colors: {
        'app-bg': '#1e1e1e',
        'app-text': '#e0e0e0',
        'menu': '#2d2d2d',
        'window': '#252525',
        'window-border': '#3e3e3e',
        'window-subtle': '#1f1f1f',
        'primary': '#3498db',
        'primary-dark': '#2980b9',
        'secondary': '#95a5a6',
        'secondary-dark': '#7f8c8d',
        'success': '#27ae60',
        'error': '#e74c3c',
        'error-dark': '#c0392b',
        'hover': 'rgba(52, 152, 219, 0.1)',
        'input': '#2a2a2a',
      },
    },
  },
  plugins: [],
}
```

#### Step 2B.4: Update app/src/app.css for v3 Syntax
Replace the top of `app/src/app.css`:

**Remove this** (Tailwind v4 syntax):
```css
/* Tailwind CSS v4 imports */
@import "tailwindcss";

/* Tailwind CSS v4 theme configuration */
@theme {
  --color-app-bg: #1e1e1e;
  /* ... */
}
```

**Replace with** (Tailwind v3 syntax):
```css
/* Tailwind CSS v3 directives */
@tailwind base;
@tailwind components;
@tailwind utilities;
```

**Keep all the custom CSS** that comes after (the `:root`, resets, utilities, etc.)

#### Step 2B.5: Update PostCSS Config for v3
Edit `app/postcss.config.js`:

```javascript
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
```

#### Step 2B.6: Test Tailwind v3
```bash
pnpm dev
```

Check browser at `http://localhost:5173/`:
- [ ] Dark background visible
- [ ] All components styled correctly
- [ ] Dark mode classes working

---

## 🧪 Phase 3: Verification (10 minutes)

### Step 3.1: Visual Inspection
Open the app and verify:

1. **MenuBar** (`app/src/lib/components/MenuBar.svelte`)
   - [ ] Dark background (#2d2d2d)
   - [ ] White text visible
   - [ ] Dropdown menus styled correctly

2. **StatusBar** (`app/src/lib/components/StatusBar.svelte`)
   - [ ] Fixed at bottom
   - [ ] Dark background
   - [ ] Green/gray status indicators visible

3. **Windows** (DAW, Mixer, Database, Pipeline)
   - [ ] Window borders visible (#3e3e3e)
   - [ ] Window backgrounds dark (#252525)
   - [ ] Title bars with proper styling
   - [ ] Content areas visible

4. **Interactive Elements**
   - [ ] Buttons show hover states
   - [ ] Inputs have focus styles
   - [ ] Primary buttons are blue (#3498db)
   - [ ] Error buttons are red (#e74c3c)

### Step 3.2: Browser DevTools Check
1. Open F12 → Elements tab
2. Inspect an element with `dark:bg-menu`
3. Check Computed styles:
   - Should show `background-color: rgb(45, 45, 45)` (which is #2d2d2d)
4. Check if Tailwind utilities are in the `<style>` tag or loaded CSS file

### Step 3.3: CSS File Size Check
Tailwind should generate a substantial CSS file:

```bash
# After build
pnpm build
ls -lh dist/assets/*.css
```

**Expected**: CSS file should be 50-200KB (contains all Tailwind utilities)

### Step 3.4: Test All Components
Navigate through the app:
- [ ] Click each menu item (File, Edit, View, Transport, Help)
- [ ] Toggle each window (F1-F4 shortcuts)
- [ ] Check all window controls (minimize, maximize, close)
- [ ] Verify scrollbars have custom styling

---

## 🚨 Phase 4: Troubleshooting (If Issues Persist)

### Issue A: Styles Still Not Applying

**Diagnosis**:
```bash
# Check if CSS is being processed
pnpm dev
# In another terminal:
curl http://localhost:5173/src/app.css | head -50
```

**Solutions**:
1. Clear Vite cache:
   ```bash
   rm -rf node_modules/.vite
   pnpm dev
   ```

2. Hard refresh browser: `Ctrl+Shift+R`

3. Check Vite config includes CSS processing:
   ```bash
   cat vite.config.ts
   ```
   Verify it has `svelte()` plugin configured.

### Issue B: Dark Mode Not Working

**Check HTML element**:
Open browser DevTools → Elements → `<html>` tag should have `class="dark"`

**Fix**: Verify in `app/index.html`:
```html
<html lang="en" class="dark">
```

### Issue C: Some Classes Not Generating

**Diagnosis**: Tailwind might not be scanning all files

**Fix for v3**: Update `tailwind.config.js` content array:
```javascript
content: [
  "./index.html",
  "./src/**/*.{js,ts,jsx,tsx,svelte}",
  "./src/lib/**/*.{svelte,ts}",
  "./src/lib/components/**/*.svelte",
  "./src/lib/windows/**/*.svelte",
],
```

**Fix for v4**: Check `@import "tailwindcss"` is the first line in app.css

### Issue D: CPU-Only System Rendering Issues

If styles work but rendering is still problematic:

**Set environment variables** before running:
```bash
export WEBKIT_DISABLE_COMPOSITING_MODE=1
pnpm dev
```

Or create `app/launch-cpu-only.sh`:
```bash
#!/bin/bash
export WEBKIT_DISABLE_COMPOSITING_MODE=1
export LIBGL_ALWAYS_SOFTWARE=1
pnpm dev
```

Make executable and run:
```bash
chmod +x app/launch-cpu-only.sh
./app/launch-cpu-only.sh
```

---

## 📋 Phase 5: Production Build (5 minutes)

Once dev server works correctly:

### Step 5.1: Build the App
```bash
cd /home/dojevou/projects/midi-software-center/app
pnpm build
```

### Step 5.2: Preview Production Build
```bash
pnpm preview
```

Open `http://localhost:4173/` and verify all styling works.

### Step 5.3: Build Tauri Desktop App
```bash
cd /home/dojevou/projects/midi-software-center/app
pnpm tauri build
```

### Step 5.4: Test Desktop App
The built app will be in:
- Linux: `app/src-tauri/target/release/midi-software-center`
- Or: `app/src-tauri/target/release/bundle/`

Run it and verify all styling is correct in the desktop app.

---

## ✅ Success Criteria Checklist

After following this guide, verify:

### Visual Elements
- [ ] Background color is dark (#1e1e1e)
- [ ] MenuBar visible with dark background (#2d2d2d)
- [ ] StatusBar fixed at bottom with status indicators
- [ ] 4 windows (DAW, Mixer, Database, Pipeline) visible with borders
- [ ] All text is white/light colored on dark backgrounds
- [ ] Buttons have proper colors (blue, red, gray)
- [ ] Hover states work on all interactive elements

### Technical Elements
- [ ] Tailwind CSS is processing correctly (check DevTools)
- [ ] All 77 `dark:` classes are compiled to CSS
- [ ] PostCSS pipeline is working
- [ ] CSS file size is reasonable (50-200KB)
- [ ] No console errors related to CSS

### Browser Compatibility
- [ ] Works in Chrome/Chromium
- [ ] Works in Firefox (if testing)
- [ ] Works in Tauri desktop app

---

## 🔄 Recommended Approach: Which Option to Use?

### Start with Option A (Fix Tailwind v4)
**Try this first if:**
- ✅ You want to use the latest Tailwind features
- ✅ You're comfortable with newer tooling
- ✅ The project is in active development

**Time investment**: 10-15 minutes

### Use Option B (Tailwind v3) if:
- ✅ Option A doesn't work after troubleshooting
- ✅ You need a stable, production-ready solution
- ✅ You want to avoid bleeding-edge issues

**Time investment**: 20-25 minutes (includes conversion)

---

## 📝 Notes

### Current File Structure
```
app/
├── src/
│   ├── main.ts              # Entry point (imports app.css)
│   ├── App.svelte           # Root component (uses dark: classes)
│   ├── app.css              # Main stylesheet (Tailwind v4 syntax currently)
│   └── lib/
│       ├── components/      # 3 components with dark: classes
│       └── windows/         # 4 windows with dark: classes
├── index.html               # HTML entry (has class="dark")
├── package.json             # Dependencies
├── postcss.config.js        # PostCSS config (needs fixing)
├── vite.config.ts           # Vite config (correct)
└── tsconfig.json            # TypeScript config
```

### Tailwind v4 vs v3 Differences

| Feature | Tailwind v4 | Tailwind v3 |
|---------|-------------|-------------|
| Config file | CSS-based `@theme {}` | JavaScript `tailwind.config.js` |
| Import syntax | `@import "tailwindcss"` | `@tailwind base; @tailwind components; @tailwind utilities;` |
| PostCSS plugin | `@tailwindcss/postcss` | `tailwindcss` |
| Stability | Beta (may have issues) | Stable (production-ready) |
| Performance | Faster (advertised) | Fast |

---

## 🆘 Getting Help

If issues persist after following this guide:

1. **Check Vite logs** for CSS processing errors
2. **Check browser console** for any errors
3. **Verify file paths** are correct (no typos)
4. **Check node_modules** - try removing and reinstalling:
   ```bash
   rm -rf node_modules pnpm-lock.yaml
   pnpm install
   ```

---

**Last Updated**: 2025-11-10
**Project**: MIDI Software Center (Tauri + Svelte)
**Issue Reference**: WHITE-SCREEN-FIXED.md, midi_software_center_analysis.md
