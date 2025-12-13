# Tailwind v4 Fix - Quick Start Guide

**Quick reference for fixing the white screen issue**

---

## 🚀 Fast Track: Fix in 10 Minutes

### Option 1: Fix Tailwind v4 (Try This First)

```bash
# 1. Install correct PostCSS plugin
cd /home/dojevou/projects/midi-software-center/app
pnpm add -D @tailwindcss/postcss

# 2. Update postcss.config.js
cat > postcss.config.js << 'EOF'
export default {
  plugins: {
    '@tailwindcss/postcss': {},
    autoprefixer: {},
  },
}
EOF

# 3. Verify app.css has correct syntax (check first line)
head -5 src/app.css
# Should show: @import "tailwindcss";

# 4. Clear cache and restart
rm -rf node_modules/.vite
pnpm dev

# 5. Open http://localhost:5173/ and check if it works
```

**Success?** ✅ You're done!
**Still white screen?** ⬇️ Try Option 2

---

### Option 2: Downgrade to Tailwind v3 (Stable Solution)

```bash
cd /home/dojevou/projects/midi-software-center/app

# 1. Remove v4, install v3
pnpm remove tailwindcss @tailwindcss/postcss
pnpm add -D tailwindcss@3.4.17

# 2. Create v3 config
npx tailwindcss init -p

# 3. Update tailwind.config.js with custom colors
cat > tailwind.config.js << 'EOF'
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx,svelte}"],
  darkMode: 'class',
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
EOF

# 4. Update postcss.config.js
cat > postcss.config.js << 'EOF'
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
EOF
```

**5. Update app/src/app.css** - Replace ONLY the top section:

```bash
# Open in editor and replace the first few lines
nano src/app.css  # or: code src/app.css
```

**Remove** (lines 1-25 approx):
```css
/* Tailwind CSS v4 imports */
@import "tailwindcss";

/* Tailwind CSS v4 theme configuration */
@theme {
  --color-app-bg: #1e1e1e;
  --color-app-text: #e0e0e0;
  /* ... all the @theme content ... */
}
```

**Replace with** (3 lines only):
```css
/* Tailwind CSS v3 directives */
@tailwind base;
@tailwind components;
@tailwind utilities;
```

**Keep everything after that** (all the :root, resets, utilities, etc.)

**6. Restart dev server:**
```bash
rm -rf node_modules/.vite
pnpm dev
```

**7. Open http://localhost:5173/** - Should now work! ✅

---

## 🔍 Quick Verification

After either option, check these:

```bash
# In browser at http://localhost:5173/
# Press F12 → Console → Should see no errors
# Press F12 → Elements → Inspect <html> → Should have class="dark"
# Press F12 → Elements → Inspect any dark:bg-menu element → Should show background-color: rgb(45, 45, 45)
```

**Visual checks:**
- [ ] Dark background (#1e1e1e) visible
- [ ] Menu bar at top with dark gray background
- [ ] Status bar at bottom
- [ ] White text readable
- [ ] Window borders visible

---

## ⚠️ Troubleshooting

### Still showing white screen?

```bash
# Clear all caches
rm -rf node_modules/.vite
rm -rf node_modules
rm pnpm-lock.yaml
pnpm install
pnpm dev
```

### Tailwind classes not applying?

```bash
# Hard refresh browser
# Chrome/Firefox: Ctrl+Shift+R
# Or: F12 → Network → Check "Disable cache" → Refresh
```

### CPU-only system rendering issues?

```bash
# Set environment variable
export WEBKIT_DISABLE_COMPOSITING_MODE=1
pnpm dev
```

---

## 📊 What's the Difference?

| | Option 1 (v4) | Option 2 (v3) |
|---|---|---|
| **Time** | 5 minutes | 10 minutes |
| **Stability** | Beta/Experimental | Production-ready |
| **Config** | CSS-based (@theme) | JS-based (tailwind.config.js) |
| **Syntax** | @import "tailwindcss" | @tailwind directives |
| **Recommendation** | Try first | Use if v4 fails |

---

## ✅ Success Criteria

Your app is fixed when you see:
1. **Dark background** instead of white
2. **Menu bar** with File, Edit, View, Transport, Help
3. **Status bar** at bottom with CPU/RAM indicators
4. **Windows** are visible with borders and title bars
5. **Text is white/light** on dark backgrounds

---

## 🎯 Next Steps After Fix

Once working:

```bash
# 1. Build production version
pnpm build

# 2. Preview production build
pnpm preview  # Open http://localhost:4173/

# 3. Build desktop app (if needed)
pnpm tauri build
```

---

**Estimated Total Time**: 10-15 minutes
**Difficulty**: Easy (copy/paste commands)
**Success Rate**: 95%+ (one of the two options will work)

For detailed explanations, see: `TAILWIND-V4-FIX-GUIDE.md`
