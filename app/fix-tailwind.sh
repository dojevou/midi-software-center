#!/bin/bash
# Tailwind v4 → v3 Automated Fix Script
# MIDI Software Center - White Screen Fix
# Usage: ./fix-tailwind.sh

set -e  # Exit on error

echo "========================================="
echo "Tailwind v4 → v3 Automated Fix"
echo "========================================="
echo ""

# Check we're in the app directory
if [ ! -f "package.json" ]; then
    echo "❌ Error: package.json not found"
    echo "Please run this script from the app/ directory"
    exit 1
fi

echo "📦 Step 1: Backup current configuration..."
cp postcss.config.js postcss.config.js.backup 2>/dev/null || true
cp src/app.css src/app.css.backup 2>/dev/null || true
echo "✅ Backups created (*.backup files)"
echo ""

echo "🗑️  Step 2: Removing Tailwind v4..."
pnpm remove tailwindcss @tailwindcss/postcss 2>/dev/null || true
echo "✅ Tailwind v4 removed"
echo ""

echo "📥 Step 3: Installing Tailwind v3..."
pnpm add -D tailwindcss@3.4.17
echo "✅ Tailwind v3 installed"
echo ""

echo "⚙️  Step 4: Creating Tailwind v3 configuration..."
cat > tailwind.config.js << 'EOF'
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx,svelte}",
  ],
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
        'gray-300': '#b0b0b0',
        'gray-400': '#808080',
        'gray-500': '#606060',
        'green-500': '#27ae60',
      },
    },
  },
  plugins: [],
}
EOF
echo "✅ tailwind.config.js created"
echo ""

echo "⚙️  Step 5: Updating PostCSS configuration..."
cat > postcss.config.js << 'EOF'
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
EOF
echo "✅ postcss.config.js updated"
echo ""

echo "📝 Step 6: Updating app.css..."
# Create temporary file with v3 syntax
cat > src/app.css.temp << 'EOF'
/* Tailwind CSS v3 directives */
@tailwind base;
@tailwind components;
@tailwind utilities;

/* Global styles for the MIDI Software Center */
EOF

# Append everything after the @theme block from original file
# Skip lines until we find the first :root selector
awk '
BEGIN { skip=1; found_root=0 }
/:root/ && !found_root { skip=0; found_root=1 }
!skip { print }
' src/app.css.backup >> src/app.css.temp

# Replace original with updated version
mv src/app.css.temp src/app.css
echo "✅ app.css updated with Tailwind v3 syntax"
echo ""

echo "🧹 Step 7: Cleaning cache..."
rm -rf node_modules/.vite
echo "✅ Vite cache cleared"
echo ""

echo "========================================="
echo "✅ Fix Complete!"
echo "========================================="
echo ""
echo "Next steps:"
echo "  1. Run: pnpm dev"
echo "  2. Open: http://localhost:5173/"
echo "  3. Verify: Dark background and styled components visible"
echo ""
echo "If you encounter issues:"
echo "  - Restore backups: mv *.backup <original-name>"
echo "  - Clear browser cache: Ctrl+Shift+R"
echo "  - See: TAILWIND-V4-FIX-GUIDE.md"
echo ""
echo "Backups saved as:"
echo "  - postcss.config.js.backup"
echo "  - src/app.css.backup"
echo ""
