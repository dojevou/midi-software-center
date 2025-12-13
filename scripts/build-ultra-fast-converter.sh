#!/bin/bash
set -e

# Build ultra-optimized MIDI to .mpcpattern converter
# Estimated build time: 5-10 minutes (one-time)
# Expected performance: 2,000-5,000 files/sec

echo "🚀 Building ULTRA-FAST .mpcpattern Converter"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Optimizations enabled:"
echo "  ✅ Native CPU features (AVX2, SSE4, FMA)"
echo "  ✅ Link-time optimization (LTO)"
echo "  ✅ jemalloc allocator"
echo "  ✅ Rayon parallel processing"
echo "  ✅ Memory-mapped I/O (zero-copy)"
echo "  ✅ Single codegen unit"
echo ""
echo "⏱️  Expected build time: 5-10 minutes"
echo "⚡ Expected performance: 2,000-5,000 files/sec"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cd "$(dirname "$0")/../pipeline/src-tauri"

# Set CPU-specific optimizations
export RUSTFLAGS="-C target-cpu=native -C target-feature=+avx2,+fma"

# Build with release profile (maximum optimizations)
echo "🔨 Building with --release (maximum optimizations)..."
echo ""

cargo build \
    --bin midi_to_mpcpattern_parallel \
    --release \
    --locked

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ BUILD COMPLETE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

BINARY="target/release/midi_to_mpcpattern_parallel"

if [ -f "$BINARY" ]; then
    SIZE=$(du -h "$BINARY" | cut -f1)
    echo "📦 Binary location: $BINARY"
    echo "📏 Binary size: $SIZE"
    echo ""
    echo "Usage:"
    echo "  # Single file"
    echo "  $BINARY input.mid output.mpcpattern"
    echo ""
    echo "  # Batch conversion (all cores)"
    echo "  $BINARY --batch /path/to/midi /path/to/output"
    echo ""
    echo "  # Batch with limit (test 100 files)"
    echo "  $BINARY --batch /path/to/midi /path/to/output 100"
    echo ""
else
    echo "❌ Binary not found at $BINARY"
    echo "Build may have failed. Check output above."
    exit 1
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
