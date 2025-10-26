#!/bin/bash
# MIDI Library System - Desktop Launcher Installer
# Installs .desktop file with proper terminal emulator detection

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
PROJECT_ROOT="$HOME/projects/midi-library-system"
DESKTOP_FILE="$PROJECT_ROOT/midi-launcher.desktop"
INSTALL_DIR="$HOME/.local/share/applications"
INSTALLED_FILE="$INSTALL_DIR/midi-launcher.desktop"
ICON_PATH="$PROJECT_ROOT/assets/midi-launcher.png"

# Print functions
print_status() { echo -e "${BLUE}$1${NC}"; }
print_success() { echo -e "${GREEN}✅ $1${NC}"; }
print_error() { echo -e "${RED}❌ $1${NC}"; }
print_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }

echo ""
echo -e "${BLUE}📦 =================================${NC}"
echo -e "${BLUE}📦  MIDI Library System Installer${NC}"
echo -e "${BLUE}📦 =================================${NC}"
echo ""

# Function to detect terminal emulator
detect_terminal() {
    print_status "🔍 Detecting terminal emulator..." >&2

    local terminals=(
        "gnome-terminal"
        "konsole"
        "xfce4-terminal"
        "mate-terminal"
        "xterm"
        "kitty"
        "alacritty"
        "terminator"
        "tilix"
        "lxterminal"
        "qterminal"
        "terminology"
    )

    for term in "${terminals[@]}"; do
        if command -v "$term" &>/dev/null; then
            echo "$term"
            return 0
        fi
    done

    print_error "No supported terminal emulator found" >&2
    return 1
}

# Function to get terminal command syntax
get_terminal_command() {
    local terminal=$1
    local script_path=$2
    local keep_open=${3:-true}

    case "$terminal" in
        gnome-terminal|mate-terminal)
            if [ "$keep_open" = true ]; then
                echo "$terminal -- bash -c \"$script_path; exec bash\""
            else
                echo "$terminal -- bash -c \"$script_path; read -p 'Press Enter to close...'\""
            fi
            ;;
        konsole)
            if [ "$keep_open" = true ]; then
                echo "$terminal --hold -e bash -c \"$script_path\""
            else
                echo "$terminal -e bash -c \"$script_path; read -p 'Press Enter to close...'\""
            fi
            ;;
        xfce4-terminal)
            if [ "$keep_open" = true ]; then
                echo "$terminal --hold -e bash -c \"$script_path\""
            else
                echo "$terminal -e bash -c \"$script_path; read -p 'Press Enter to close...'\""
            fi
            ;;
        kitty)
            if [ "$keep_open" = true ]; then
                echo "$terminal --hold bash -c \"$script_path\""
            else
                echo "$terminal bash -c \"$script_path; read -p 'Press Enter to close...'\""
            fi
            ;;
        alacritty)
            if [ "$keep_open" = true ]; then
                echo "$terminal -e bash -c \"$script_path; exec bash\""
            else
                echo "$terminal -e bash -c \"$script_path; read -p 'Press Enter to close...'\""
            fi
            ;;
        xterm|terminator|tilix|lxterminal|qterminal|terminology)
            if [ "$keep_open" = true ]; then
                echo "$terminal -e bash -c \"$script_path; exec bash\""
            else
                echo "$terminal -e bash -c \"$script_path; read -p 'Press Enter to close...'\""
            fi
            ;;
        *)
            # Fallback to generic syntax
            echo "$terminal -e bash -c \"$script_path; exec bash\""
            ;;
    esac
}

# Function to detect or select icon
setup_icon() {
    print_status "🎨 Setting up icon..." >&2

    # Check if custom icon exists
    if [ -f "$ICON_PATH" ]; then
        print_success "Using custom icon: $ICON_PATH" >&2
        echo "$ICON_PATH"
        return 0
    fi

    # Try to find system MIDI icon
    local system_icons=(
        "/usr/share/icons/hicolor/256x256/apps/multimedia-audio-player.png"
        "/usr/share/icons/hicolor/scalable/apps/multimedia-audio-player.svg"
        "/usr/share/pixmaps/multimedia-audio-player.png"
        "audio-midi"
        "audio-x-generic"
        "multimedia-audio-player"
    )

    for icon in "${system_icons[@]}"; do
        if [ -f "$icon" ]; then
            print_success "Using system icon: $icon" >&2
            echo "$icon"
            return 0
        fi
    done

    # Use icon name (will be resolved by system)
    print_warning "No custom icon found, using system default: audio-midi" >&2
    echo "audio-midi"
}

# Main installation process
main() {
    # Check if desktop file exists
    if [ ! -f "$DESKTOP_FILE" ]; then
        print_error "Desktop file not found: $DESKTOP_FILE"
        exit 1
    fi

    # Make launch scripts executable
    print_status "🔧 Making scripts executable..."
    chmod +x "$PROJECT_ROOT/scripts/launch-all.sh" 2>/dev/null || true
    chmod +x "$PROJECT_ROOT/scripts/stop-all.sh" 2>/dev/null || true
    chmod +x "$PROJECT_ROOT/scripts/status.sh" 2>/dev/null || true
    print_success "Scripts are executable"

    # Detect terminal emulator
    TERMINAL=$(detect_terminal)
    if [ $? -ne 0 ]; then
        print_error "Cannot install without a terminal emulator"
        exit 1
    fi
    print_success "Detected terminal: $TERMINAL"

    # Setup icon
    ICON=$(setup_icon)

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Create temporary desktop file with updated paths
    print_status "📝 Creating desktop file with absolute paths..."

    TEMP_DESKTOP=$(mktemp)
    cp "$DESKTOP_FILE" "$TEMP_DESKTOP"

    # Get terminal commands for each script
    LAUNCH_CMD=$(get_terminal_command "$TERMINAL" "$PROJECT_ROOT/scripts/launch-all.sh" true)
    STATUS_CMD=$(get_terminal_command "$TERMINAL" "$PROJECT_ROOT/scripts/status.sh" false)
    STOP_CMD=$(get_terminal_command "$TERMINAL" "$PROJECT_ROOT/scripts/stop-all.sh" false)
    LOGS_CMD="$TERMINAL -e bash -c \"tail -f $PROJECT_ROOT/logs/backend.log\""

    # Update the desktop file with absolute paths and correct terminal
    # Use awk instead of sed for better handling of special characters
    awk -v icon="$ICON" -v launch="$LAUNCH_CMD" -v status="$STATUS_CMD" -v stop="$STOP_CMD" -v logs="$LOGS_CMD" '
        BEGIN { section="main" }
        /^\[Desktop Entry\]/ { section="main" }
        /^\[Desktop Action Status\]/ { section="status"; print; next }
        /^\[Desktop Action Stop\]/ { section="stop"; print; next }
        /^\[Desktop Action Logs\]/ { section="logs"; print; next }
        /^Icon=/ && section=="main" { print "Icon=" icon; next }
        /^Exec=/ && section=="main" { print "Exec=" launch; next }
        /^Exec=/ && section=="status" { print "Exec=" status; next }
        /^Exec=/ && section=="stop" { print "Exec=" stop; next }
        /^Exec=/ && section=="logs" { print "Exec=" logs; next }
        { print }
    ' "$TEMP_DESKTOP" > "$TEMP_DESKTOP.new"
    mv "$TEMP_DESKTOP.new" "$TEMP_DESKTOP"

    # Replace any remaining ~/ with absolute path
    sed -i "s|~/projects/midi-library-system|$PROJECT_ROOT|g" "$TEMP_DESKTOP"

    # Copy to applications directory
    print_status "📦 Installing launcher..."
    cp "$TEMP_DESKTOP" "$INSTALLED_FILE"
    rm "$TEMP_DESKTOP"

    # Make desktop file executable
    chmod +x "$INSTALLED_FILE"

    # Update desktop database
    print_status "🔄 Updating desktop database..."
    if command -v update-desktop-database &>/dev/null; then
        update-desktop-database "$INSTALL_DIR" 2>/dev/null || true
        print_success "Desktop database updated"
    else
        print_warning "update-desktop-database not found, skipping"
    fi

    # Copy desktop file to Desktop folder if it exists (for Ubuntu/GNOME)
    if [ -d "$HOME/Desktop" ]; then
        print_status "📋 Copying launcher to Desktop..."
        cp "$INSTALLED_FILE" "$HOME/Desktop/midi-launcher.desktop"
        chmod +x "$HOME/Desktop/midi-launcher.desktop"

        # Mark as trusted (GNOME specific)
        if command -v gio &>/dev/null; then
            gio set "$HOME/Desktop/midi-launcher.desktop" metadata::trusted true 2>/dev/null || true
        fi
        print_success "Launcher added to Desktop"
    fi

    echo ""
    echo -e "${BLUE}📦 =================================${NC}"
    print_success "Installation complete!"
    echo -e "${BLUE}📦 =================================${NC}"
    echo ""
    echo "You can now:"
    echo "  1. Find 'MIDI Library System' in your application menu"
    echo "  2. Click the icon on your Desktop (if created)"
    echo "  3. Right-click the launcher for quick actions:"
    echo "     - Check Status"
    echo "     - Stop All Services"
    echo "     - View Logs"
    echo ""
    echo "Custom Icon:"
    echo "  To use a custom icon, place a 256x256 PNG file at:"
    echo "  $ICON_PATH"
    echo "  Then run this installer again."
    echo ""
    print_status "Terminal emulator: $TERMINAL"
    print_status "Installed to: $INSTALLED_FILE"
    echo ""
}

# Run main installation
main
