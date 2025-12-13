#!/bin/bash
# Launch MIDI Software Center on CPU-only systems (no GPU)
# This script disables hardware acceleration for WebKit rendering

cd "$(dirname "$0")"

# Disable WebKit hardware acceleration
export WEBKIT_DISABLE_COMPOSITING_MODE=1
export WEBKIT_DISABLE_DMABUF_RENDERER=1

# Disable GPU acceleration in various subsystems
export LIBGL_ALWAYS_SOFTWARE=1
export GALLIUM_DRIVER=llvmpipe

echo "🚀 Launching MIDI Software Center (CPU rendering mode)"
echo "   Hardware acceleration: DISABLED"
echo "   Renderer: llvmpipe (software)"
echo ""

pnpm tauri dev
