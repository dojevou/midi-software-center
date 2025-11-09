#!/bin/bash
# Build and Install Script for Quantum Analyzer

set -e

echo "🦀 QUANTUM ANALYZER - BUILD SCRIPT"
echo "=================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check Rust installation
echo "📦 Checking Rust installation..."
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust is not installed${NC}"
    echo "Install from: https://rustup.rs/"
    exit 1
fi

RUST_VERSION=$(rustc --version)
echo -e "${GREEN}✅ Found: $RUST_VERSION${NC}"
echo ""

# Check Cargo version
CARGO_VERSION=$(cargo --version)
echo -e "${GREEN}✅ Found: $CARGO_VERSION${NC}"
echo ""

# Build release binary
echo "🔨 Building release binary..."
echo "This may take a few minutes on first build..."
cargo build --release

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Build successful!${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

echo ""

# Check binary
BINARY_PATH="target/release/quantum-analyzer"
if [ -f "$BINARY_PATH" ]; then
    BINARY_SIZE=$(ls -lh "$BINARY_PATH" | awk '{print $5}')
    echo -e "${GREEN}✅ Binary created: $BINARY_PATH ($BINARY_SIZE)${NC}"
else
    echo -e "${RED}❌ Binary not found${NC}"
    exit 1
fi

echo ""

# Optional: Install as cargo plugin
echo "📦 Installation Options:"
echo ""
echo "Option 1: Use directly"
echo "  ./target/release/quantum-analyzer"
echo ""
echo "Option 2: Install as cargo plugin"
echo "  cargo install --path ."
echo "  Then use: cargo quantum"
echo ""
echo "Option 3: Add to PATH"
echo "  sudo cp target/release/quantum-analyzer /usr/local/bin/"
echo "  Then use: quantum-analyzer"
echo ""

read -p "Install as cargo plugin? (y/N): " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🔧 Installing as cargo plugin..."
    cargo install --path .
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Installed successfully!${NC}"
        echo "Use with: cargo quantum"
    else
        echo -e "${RED}❌ Installation failed${NC}"
        exit 1
    fi
fi

echo ""
echo "════════════════════════════════════"
echo -e "${GREEN}🎉 Setup Complete!${NC}"
echo "════════════════════════════════════"
echo ""
echo "Quick Start:"
echo "  ./target/release/quantum-analyzer"
echo ""
echo "With auto-fix:"
echo "  ./target/release/quantum-analyzer --autofix"
echo ""
echo "Generate Claude Code tasks:"
echo "  ./target/release/quantum-analyzer --claude-code"
echo ""
echo "Full help:"
echo "  ./target/release/quantum-analyzer --help"
echo ""
