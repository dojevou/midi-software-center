#!/bin/bash
# MIDI Library System - Desktop Launcher Uninstaller
# Removes .desktop file from all locations

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
INSTALL_DIR="$HOME/.local/share/applications"
DESKTOP_FILE="$INSTALL_DIR/midi-launcher.desktop"
DESKTOP_ICON="$HOME/Desktop/midi-launcher.desktop"

# Print functions
print_status() { echo -e "${BLUE}$1${NC}"; }
print_success() { echo -e "${GREEN}✅ $1${NC}"; }
print_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }

echo ""
echo -e "${BLUE}🗑️  =================================${NC}"
echo -e "${BLUE}🗑️   MIDI Library System Uninstaller${NC}"
echo -e "${BLUE}🗑️  =================================${NC}"
echo ""

# Function to remove file with status
remove_file() {
    local file=$1
    local description=$2

    if [ -f "$file" ]; then
        rm "$file"
        print_success "$description removed"
        return 0
    else
        print_warning "$description not found"
        return 1
    fi
}

# Main uninstallation
main() {
    local removed_any=0

    # Remove from applications directory
    print_status "🗑️  Removing from application menu..."
    if remove_file "$DESKTOP_FILE" "Application menu launcher"; then
        removed_any=1
    fi

    # Remove from Desktop
    print_status "🗑️  Removing from Desktop..."
    if remove_file "$DESKTOP_ICON" "Desktop icon"; then
        removed_any=1
    fi

    # Update desktop database
    print_status "🔄 Updating desktop database..."
    if command -v update-desktop-database &>/dev/null; then
        update-desktop-database "$INSTALL_DIR" 2>/dev/null || true
        print_success "Desktop database updated"
    else
        print_warning "update-desktop-database not found, skipping"
    fi

    echo ""
    echo -e "${BLUE}🗑️  =================================${NC}"
    if [ $removed_any -eq 1 ]; then
        print_success "Uninstallation complete!"
    else
        print_warning "No launcher files were found"
    fi
    echo -e "${BLUE}🗑️  =================================${NC}"
    echo ""

    if [ $removed_any -eq 1 ]; then
        echo "The launcher has been removed from:"
        echo "  - Application menu"
        echo "  - Desktop"
        echo ""
        echo "Note: This does not remove:"
        echo "  - The project files in ~/projects/midi-software-center/"
        echo "  - The launch scripts in scripts/"
        echo "  - Any running services"
        echo ""
        echo "To reinstall the launcher, run:"
        echo "  ~/projects/midi-software-center/scripts/install-launcher.sh"
    fi
    echo ""
}

# Run uninstallation
main
