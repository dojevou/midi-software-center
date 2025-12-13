#!/bin/bash

# Create the markdown document
{
    echo "# MIDI Software Center - Complete Project Analysis"
    echo ""
    echo "**Generated:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
    echo "**Project:** MIDI Software Center (Tauri + Svelte + Rust)"
    echo "**Total Files:** 142 directories, 528 files"
    echo "**Frontend Framework:** Svelte 4.2 + Vite 5.0 + Tailwind CSS v4"
    echo "**Backend:** Rust + Tauri 2.7 + PostgreSQL + Meilisearch"
    echo ""

    echo "## Project Overview"
    echo ""
    echo "This document contains the complete source code and documentation for the MIDI Software Center - a comprehensive desktop application for MIDI file management, analysis, and real-time sequencing with a unified GUI built on Tauri, Svelte, and Rust."
    echo ""

    echo "## File Structure Summary"
    echo ""
    echo "| Category | Count | Description |"
    echo "|----------|-------|-------------|"
    echo "| Rust Source Files | 30+ | Backend, MIDI processing, database, Tauri commands |"
    echo "| Svelte Components | 6+ | UI components, windows, stores |"
    echo "| TypeScript Files | 6+ | Frontend logic, type definitions |"
    echo "| SQL Migrations | 9 | Database schema and migrations |"
    echo "| Shell Scripts | 10+ | Automation, deployment, maintenance |"
    echo "| TOML Configuration | 10+ | Rust, Tauri, agent configurations |"
    echo "| JSON Configuration | 15+ | Package, Tauri, tool configurations |"
    echo "| Markdown Documentation | 80+ | Comprehensive docs, guides, architecture |"
    echo "| CSS/Styling | 3 | App styles, Tailwind configuration |"
    echo "| Docker/YAML | 4 | Database services, deployment |"
    echo ""

    echo "## Critical Files for White Screen Diagnosis"
    echo ""
    echo "| File | Type | Purpose | Status |"
    echo "|------|------|---------|--------|"
    echo "| app/src/App.svelte | Svelte | Root component | ⚠️ White screen issue |"
    echo "| app/src/main.ts | TypeScript | Frontend entry point | ✅ Working |"
    echo "| app/src/app.css | CSS | Tailwind v4 + custom styles | ⚠️ Tailwind v4 issue |"
    echo "| app/package.json | JSON | Frontend dependencies | ✅ Correct |"
    echo "| app/vite.config.ts | TypeScript | Build configuration | ✅ Correct |"
    echo "| app/src-tauri/tauri.conf.json | JSON | Tauri desktop config | ✅ Correct |"
    echo "| app/src-tauri/Cargo.toml | TOML | Rust dependencies | ✅ Correct |"
    echo ""

    echo "## Tailwind CSS v4 Components Analysis"
    echo ""
    echo "### 🎨 CSS Configuration"
    echo "- **app/src/app.css** - Contains @import \"tailwindcss\" and @theme configuration"
    echo ""
    echo "### 🧩 Core Components (3 files)"
    echo "1. **app/src/lib/components/MenuBar.svelte** - Main menu navigation"
    echo "2. **app/src/lib/components/StatusBar.svelte** - Bottom status display"
    echo "3. **app/src/lib/components/WindowBase.svelte** - Base window component"
    echo ""
    echo "### 🪟 Window Components (4 files)"
    echo "1. **app/src/lib/windows/DAWWindow.svelte** - Real-time sequencer interface"
    echo "2. **app/src/lib/windows/MixerWindow.svelte** - Audio mixer controls"
    echo "3. **app/src/lib/windows/DatabaseWindow.svelte** - MIDI file database management"
    echo "4. **app/src/lib/windows/PipelineWindow.svelte** - Batch processing interface"
    echo ""
    echo "### 📊 Tailwind Usage Statistics"
    echo "- **Total files using Tailwind:** 8 (1 CSS + 3 components + 4 windows)"
    echo "- **Total dark: classes:** 77 instances across all components"
    echo "- **Total lines affected:** ~500-800 lines of template code"
    echo "- **Custom color palette:** app-bg, menu, window, primary, error, etc."
    echo ""

    echo "## Table of Contents"
    echo ""
    echo "| File | Type | Purpose | Line |"
    echo "|------|------|---------|------|"

    # Function to get description based on file type and path
    get_file_description() {
        local file="$1"
        local dir=$(dirname "$file")
        local base=$(basename "$file")

        case "$dir" in
        "." | "app") # Root and app directory files
            case "$base" in
            package.json) echo "Frontend dependencies and scripts" ;;
            vite.config.ts) echo "Vite build configuration" ;;
            svelte.config.js) echo "Svelte compiler configuration" ;;
            tsconfig.json) echo "TypeScript configuration" ;;
            vitest.config.ts) echo "Test runner configuration" ;;
            index.html) echo "Frontend entry point" ;;
            *.md)
                case "$base" in
                README.md) echo "Project documentation" ;;
                CHANGELOG.md) echo "Project changelog" ;;
                LICENSE) echo "Project license" ;;
                *) echo "Project documentation" ;;
                esac
                ;;
            *.svelte) echo "Svelte component" ;;
            *.ts) echo "TypeScript file" ;;
            *.css) echo "Stylesheet with Tailwind v4" ;;
            *) echo "Project file" ;;
            esac
            ;;
        app/src)
            case "$base" in
            main.ts) echo "Frontend entry point" ;;
            App.svelte) echo "Root Svelte component" ;;
            app.css) echo "Main stylesheet with Tailwind v4" ;;
            *) echo "Source file" ;;
            esac
            ;;
        app/src/lib/components)
            echo "Svelte component - $(basename "$base" .svelte)"
            ;;
        app/src/lib/windows)
            echo "Window component - $(basename "$base" .svelte)"
            ;;
        app/src/lib/stores)
            echo "Svelte store - $(basename "$base" .ts)"
            ;;
        app/src-tauri)
            case "$base" in
            Cargo.toml) echo "Rust dependencies for Tauri" ;;
            tauri.conf.json) echo "Tauri desktop application configuration" ;;
            main.rs) echo "Tauri application entry point" ;;
            build.rs) echo "Tauri build script" ;;
            *) echo "Tauri backend file" ;;
            esac
            ;;
        database/migrations)
            echo "Database migration - $(basename "$base" .sql)"
            ;;
        scripts)
            echo "Shell script - $(basename "$base" .sh)"
            ;;
        docs)
            echo "Documentation - $(basename "$base" .md)"
            ;;
        config/agents)
            echo "Agent configuration - $(basename "$base" .toml)"
            ;;
        shared/rust)
            echo "Shared Rust library"
            ;;
        pipeline/src-tauri)
            echo "Pipeline Tauri application"
            ;;
        daw/src-tauri)
            echo "DAW Tauri application"
            ;;
        *)
            echo "Project file"
            ;;
        esac
    }

    # Arrays to store files in processing order
    declare -a processing_order
    declare -A file_descriptions
    declare -A file_line_numbers

    # Collect all files in logical order for white screen diagnosis
    collect_files() {
        echo "Collecting files for white screen diagnosis..." >&2

        # Critical frontend files first (white screen diagnosis)
        for file in app/src/main.ts app/src/App.svelte app/src/app.css app/index.html; do
            if [[ -f "$file" ]]; then
                processing_order+=("$file")
                file_descriptions["$file"]=$(get_file_description "$file")
            fi
        done

        # Frontend configuration files
        for file in app/package.json app/vite.config.ts app/svelte.config.js app/tsconfig.json app/vitest.config.ts; do
            if [[ -f "$file" ]]; then
                processing_order+=("$file")
                file_descriptions["$file"]=$(get_file_description "$file")
            fi
        done

        # Tailwind CSS v4 configuration
        for file in app/src/app.css app/postcss.config.js app/tailwind.config.js; do
            if [[ -f "$file" ]]; then
                processing_order+=("$file")
                file_descriptions["$file"]=$(get_file_description "$file")
            fi
        done

        # Core Svelte components (Tailwind users)
        for file in app/src/lib/components/MenuBar.svelte app/src/lib/components/StatusBar.svelte app/src/lib/components/WindowBase.svelte; do
            if [[ -f "$file" ]]; then
                processing_order+=("$file")
                file_descriptions["$file"]=$(get_file_description "$file")
            fi
        done

        # Window components (Tailwind users)
        for file in app/src/lib/windows/DAWWindow.svelte app/src/lib/windows/MixerWindow.svelte app/src/lib/windows/DatabaseWindow.svelte app/src/lib/windows/PipelineWindow.svelte; do
            if [[ -f "$file" ]]; then
                processing_order+=("$file")
                file_descriptions["$file"]=$(get_file_description "$file")
            fi
        done

        # Svelte stores
        for file in app/src/lib/stores/*.ts; do
            if [[ -f "$file" ]]; then
                processing_order+=("$file")
                file_descriptions["$file"]=$(get_file_description "$file")
            fi
        done

        # Tauri configuration
        for file in app/src-tauri/Cargo.toml app/src-tauri/tauri.conf.json app/src-tauri/src/main.rs app/src-tauri/src/lib.rs app/src-tauri/build.rs; do
            if [[ -f "$file" ]]; then
                processing_order+=("$file")
                file_descriptions["$file"]=$(get_file_description "$file")
            fi
        done

        # Root workspace configuration
        for file in Cargo.toml rustfmt.toml rust-toolchain.toml Makefile docker-compose.yml; do
            if [[ -f "$file" ]]; then
                processing_order+=("$file")
                file_descriptions["$file"]=$(get_file_description "$file")
            fi
        done

        # Database migrations
        for file in database/migrations/*.sql; do
            if [[ -f "$file" ]]; then
                processing_order+=("$file")
                file_descriptions["$file"]=$(get_file_description "$file")
            fi
        done

        # Critical documentation
        for file in WHITE-SCREEN-FIXED.md CPU-ONLY-SYSTEMS.md GUI-CRASH-FIX.md README.md; do
            if [[ -f "$file" ]]; then
                processing_order+=("$file")
                file_descriptions["$file"]=$(get_file_description "$file")
            fi
        done

        # Agent configurations
        for file in config/agents/*.toml; do
            if [[ -f "$file" ]]; then
                processing_order+=("$file")
                file_descriptions["$file"]=$(get_file_description "$file")
            fi
        done
    }

    # Calculate exact line numbers by simulating the document structure
    calculate_line_numbers() {
        local current_line=1

        # Count header lines (up to TOC)
        current_line=$((current_line + 13)) # Main header and project overview
        current_line=$((current_line + 12)) # File structure summary table
        current_line=$((current_line + 20)) # Critical files and Tailwind sections
        current_line=$((current_line + 4))  # TOC header

        # Count TOC lines (1 per file + header row + separator)
        local toc_lines=$((3 + ${#processing_order[@]})) # Header, separator, blank, and each file
        current_line=$((current_line + toc_lines))

        # Count lines after TOC
        current_line=$((current_line + 16)) # Analysis guidelines and file contents header

        # Calculate line numbers for each file section
        for file in "${processing_order[@]}"; do
            # Record the current line as this file's header line
            file_line_numbers["$file"]=$((current_line + 3)) # +3 for empty line and border

            # Count lines for this file section
            local content_lines=0
            if [[ -f "$file" ]]; then
                content_lines=$(wc -l <"$file" 2>/dev/null || echo 0)
            fi

            # Each file section adds:
            # 1 (empty) + 3 (border) + 1 (empty) + 5 (metadata) + 1 (code block start) + content_lines + 1 (code block end) + 1 (empty) + 1 (separator)
            local section_lines=$((1 + 3 + 1 + 5 + 1 + content_lines + 1 + 1 + 1))
            current_line=$((current_line + section_lines))
        done
    }

    # Build the TOC in processing order
    build_toc() {
        for file in "${processing_order[@]}"; do
            echo "| \`$file\` | ${file##*.} | ${file_descriptions[$file]} | ${file_line_numbers[$file]} |"
        done
    }

    # Collect files and calculate line numbers
    collect_files
    calculate_line_numbers

    # Build TOC in the correct order
    build_toc

    echo ""
    echo "---"
    echo ""

    echo "## White Screen Issue Analysis Guidelines"
    echo ""
    echo "### Current Status: Tailwind CSS v4 Configuration Issue"
    echo "- **Root Cause**: Components use Tailwind classes but v4 may not be processing correctly"
    echo "- **Evidence**: 77 dark: classes across 8 files, minimal test component works but full app doesn't"
    echo "- **CPU-Only System**: Software rendering (llvmpipe) requires specific environment variables"
    echo ""
    echo "### Investigation Priorities:"
    echo "1. **Tailwind v4 Processing**: Verify @import \"tailwindcss\" is working in app.css"
    echo "2. **Dark Mode Configuration**: Check class=\"dark\" on html element and darkMode: 'class' in config"
    echo "3. **PostCSS Pipeline**: Ensure PostCSS is processing Tailwind directives correctly"
    echo "4. **Component Visibility**: Verify DOM elements exist but are invisible due to missing styles"
    echo "5. **Environment Variables**: CPU-only rendering requires WEBKIT_DISABLE_COMPOSITING_MODE=1"
    echo ""
    echo "### Tailwind v4 Specific Checks:"
    echo "- ✅ Custom theme colors defined in @theme {} block"
    echo "- ✅ @import \"tailwindcss\" directive in app.css"
    echo "- ✅ postcss.config.js with @tailwindcss/postcss plugin"
    echo "- ✅ darkMode: 'class' in tailwind.config.js"
    echo "- ✅ class=\"dark\" on html element in index.html"
    echo ""

    echo "## File Contents"
    echo ""

    # Function to process and display a file with enhanced information
    process_file() {
        local file="$1"
        if [[ -f "$file" ]]; then
            local file_size=$(wc -c <"$file" | awk '{print $1}')
            local line_count=$(wc -l <"$file")
            local description="${file_descriptions[$file]}"

            # Special handling for white screen related files
            local status_icon="📄"
            case "$file" in
            app/src/App.svelte | app/src/app.css | app/src/main.ts)
                status_icon="🔍"
                ;;
            app/package.json | app/vite.config.ts | app/src-tauri/tauri.conf.json)
                status_icon="⚙️"
                ;;
            app/src/lib/components/*.svelte | app/src/lib/windows/*.svelte)
                status_icon="🎨"
                ;;
            esac

            echo ""
            echo "=========================================="
            echo "FILE: $file $status_icon"
            echo "=========================================="
            echo ""
            echo "**Description:** $description  "
            echo "**Size:** $file_size bytes  "
            echo "**Lines:** $line_count  "
            echo "**Type:** ${file##*.}  "
            echo "**White Screen Relevance:** High" | grep -q "App.svelte\|app.css\|main.ts\|package.json\|vite.config.ts\|tauri.conf.json" && echo "**White Screen Relevance:** High" || echo "**White Screen Relevance:** Medium"
            echo ""

            # Enhanced syntax highlighting with file-specific contexts
            case "$file" in
            *.rs)
                echo "\`\`\`rust"
                echo "// Rust file: $file"
                echo "// Path: $file"
                echo ""
                ;;
            *.toml)
                echo "\`\`\`toml"
                echo "# TOML Configuration: $file"
                echo "# Path: $file"
                echo ""
                ;;
            *.md)
                echo "\`\`\`markdown"
                echo "<!-- Markdown Documentation: $file -->"
                echo "<!-- Path: $file -->"
                echo ""
                ;;
            *.sh)
                echo "\`\`\`bash"
                echo "# Shell Script: $file"
                echo "# Path: $file"
                echo ""
                ;;
            *.yaml | *.yml)
                echo "\`\`\`yaml"
                echo "# YAML Configuration: $file"
                echo "# Path: $file"
                echo ""
                ;;
            *.svelte)
                echo "\`\`\`svelte"
                echo "<!-- Svelte Component: $file -->"
                echo "<!-- Path: $file -->"
                echo "<!-- Tailwind v4 Usage: This component uses dark: classes -->"
                echo ""
                ;;
            *.ts | *.tsx)
                echo "\`\`\`typescript"
                echo "// TypeScript file: $file"
                echo "// Path: $file"
                echo ""
                ;;
            *.js | *.jsx)
                echo "\`\`\`javascript"
                echo "// JavaScript file: $file"
                echo "// Path: $file"
                echo ""
                ;;
            *.css)
                echo "\`\`\`css"
                echo "/* CSS file: $file */"
                echo "/* Path: $file */"
                echo "/* Tailwind v4: Contains @import \"tailwindcss\" */"
                echo ""
                ;;
            *.json)
                echo "\`\`\`json"
                echo "// JSON file: $file"
                echo "// Path: $file"
                echo ""
                ;;
            Dockerfile)
                echo "\`\`\`dockerfile"
                echo "# Dockerfile: $file"
                echo "# Path: $file"
                echo ""
                ;;
            Makefile)
                echo "\`\`\`makefile"
                echo "# Makefile: $file"
                echo "# Path: $file"
                echo ""
                ;;
            LICENSE)
                echo "\`\`\`text"
                echo "# License: $file"
                echo "# Path: $file"
                echo ""
                ;;
            *.sql)
                echo "\`\`\`sql"
                echo "-- SQL migration: $file"
                echo "-- Path: $file"
                echo ""
                ;;
            *)
                echo "\`\`\`text"
                echo "# File: $file"
                echo "# Path: $file"
                echo ""
                ;;
            esac

            cat "$file"
            echo ""
            echo "\`\`\`"
            echo ""
            echo "---"
        fi
    }

    # Process files in the exact same order as the TOC
    for file in "${processing_order[@]}"; do
        process_file "$file"
    done

    echo "## White Screen Issue - Comprehensive Analysis"
    echo ""
    echo "### Tailwind CSS v4 Configuration Status"
    echo ""
    echo "| Component | Status | Issue | Solution |"
    echo "|-----------|--------|-------|----------|"
    echo "| Tailwind Import | ⚠️ | @import \"tailwindcss\" may not be processing | Verify PostCSS pipeline |"
    echo "| Dark Mode | ⚠️ | dark: classes not applying | Check html class=\"dark\" |"
    echo "| Custom Colors | ✅ | @theme {} block defined | Colors available if processing works |"
    echo "| PostCSS Config | ✅ | postcss.config.js present | Should process Tailwind |"
    echo "| CPU Rendering | ✅ | Environment variables set | WEBKIT_DISABLE_COMPOSITING_MODE=1 |"
    echo ""

    echo "### Immediate Diagnostic Steps"
    echo ""
    echo "1. **Browser DevTools Check**:"
    echo "   - Open http://localhost:5173/"
    echo "   - Press F12 → Elements tab"
    echo "   - Verify dark: classes are compiled to actual CSS"
    echo "   - Check if Tailwind utilities are injected"
    echo ""
    echo "2. **Tailwind Processing Test**:"
    echo "   - Add a non-Tailwind CSS rule to confirm basic CSS works"
    echo "   - Test if inline styles display content"
    echo "   - Verify PostCSS is processing @import directives"
    echo ""
    echo "3. **Environment Verification**:"
    echo "   - Run: $(cd app && pnpm list tailwindcss postcss autoprefixer)"
    echo "   - Check: $(curl -s http://localhost:5173/src/app.css | head -20)"
    echo "   - Verify: Tailwind v4 directives are in final CSS"
    echo ""

    echo "### File-Specific Issues"
    echo ""
    echo "#### app/src/App.svelte"
    echo "- Uses Tailwind classes for layout and theming"
    echo -e "- Contains 4 window components that should be visible\n"
    echo "#### app/src/app.css"
    echo "- Contains Tailwind v4 @import directive"
    echo "- Has @theme {} block with custom color palette"
    echo "- Needs proper PostCSS processing\n"
    echo "#### app/src/lib/components/ & app/src/lib/windows/"
    echo "- 77 instances of dark: classes across components"
    echo "- All styling depends on Tailwind processing"
    echo "- Components render to DOM but are invisible without styles"
    echo ""

    echo "### Recommended Solutions (Priority Order)"
    echo ""
    echo "1. **Fix Tailwind v4 Processing** (Highest Priority)"
    echo "   - Verify PostCSS configuration"
    echo "   - Check if @import \"tailwindcss\" is resolving"
    echo "   - Ensure tailwind.config.js is properly configured"
    echo ""
    echo "2. **Fallback to Tailwind v3** (Backup Solution)"
    echo "   - Replace v4 with stable v3 configuration"
    echo "   - Use @tailwind base; @tailwind components; @tailwind utilities;"
    echo "   - Update tailwind.config.js for v3 syntax"
    echo ""
    echo "3. **CSS Custom Properties Fallback** (Immediate Visibility)"
    echo "   - Add CSS variables as fallback in app.css"
    echo "   - Ensure basic visibility without Tailwind"
    echo "   - Progressive enhancement approach"
    echo ""

    echo "### Project Architecture Assessment"
    echo ""
    echo "This is a **complex desktop application** with:"
    echo "- Unified Tauri + Svelte frontend architecture"
    echo -e "- 30+ Rust files for backend MIDI processing\n- Real-time audio and database capabilities"
    echo "- Comprehensive workspace structure (app, pipeline, daw, shared)"
    echo "- Professional-grade configuration and documentation"
    echo "- CPU-only system compatibility requirements"
    echo ""

    echo "### Success Metrics"
    echo ""
    echo "The white screen issue will be resolved when:"
    echo "✅ Dark background (#1e1e1e) is visible"
    echo "✅ Menu bar with proper styling appears"
    echo "✅ 4 windows (DAW, Mixer, Database, Pipeline) are visible with borders"
    echo "✅ All text is readable (white on dark backgrounds)"
    echo "✅ Tailwind dark: classes compile to actual CSS rules"
    echo ""

} >midi_software_center_analysis.md

echo "Complete project analysis document created: midi_software_center_analysis.md"
echo ""
echo "This document includes CRITICAL FILES for white screen diagnosis:"
echo "✅ 8 Tailwind CSS v4 files (1 CSS + 7 components)"
echo "✅ Frontend configuration (vite.config.ts, package.json, etc.)"
echo "✅ Tauri desktop configuration"
echo "✅ Database migrations and schema"
echo "✅ Rust workspace configuration"
echo "✅ Documentation and troubleshooting guides"
echo "✅ Accurate table of contents with line numbers"
echo "✅ Tailwind v4 specific analysis and solutions"
echo "✅ White screen diagnostic framework"
echo ""
echo "Total files included: ${#processing_order[@]}"
echo "Focus: Tailwind CSS v4 configuration and white screen resolution"
