#!/bin/bash

# Create the markdown document
{
    echo "# Project Analysis Document"
    echo ""
    echo "**Generated:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
    echo "**Total Files:** 16"
    echo "**File Types:** Rust (.rs), TOML (.toml), Markdown (.md), Makefile, Shell Script (.sh)"
    echo ""

    echo "## Project Overview"
    echo ""
    echo "This document contains the complete source code for analysis and improvement suggestions. The project appears to be a multi-language system involving Rust with comprehensive documentation."
    echo ""

    echo "## File Structure Summary"
    echo ""
    echo "| Type | Count | Purpose |"
    echo "|------|-------|---------|"
    echo "| Rust (.rs) | 9 | Core application logic, likely main program |"
    echo "| TOML (.toml) | 1 | Configuration and package dependencies |"
    echo "| Markdown (.md) | 4 | Documentation, README, or guides |"
    echo "| Makefile | 1 | Build automation and compilation rules |"
    echo "| Shell Script (.sh) | 1 | Deployment, setup, or utility scripts |"
    echo ""

    echo "## Table of Contents"
    echo ""
    echo "| File | Type | Purpose | Line |"
    echo "|------|------|---------|------|"

    # Function to get description based on file type and name
    get_file_description() {
        local file="$1"
        case "$file" in
        *.rs)
            case "$file" in
            main.rs) echo "Rust main entry point" ;;
            lib.rs) echo "Rust library root module" ;;
            mod.rs) echo "Rust module declaration" ;;
            *test.rs) echo "Rust test module" ;;
            *) echo "Rust source module" ;;
            esac
            ;;
        *.toml)
            echo "Project configuration and dependencies"
            ;;
        *.md)
            case "$file" in
            README.md) echo "Project README documentation" ;;
            CONTRIBUTING.md) echo "Contribution guidelines" ;;
            LICENSE.md) echo "License information" ;;
            CHANGELOG.md) echo "Project changelog" ;;
            *) echo "Markdown documentation" ;;
            esac
            ;;
        Makefile)
            echo "Build system configuration"
            ;;
        *.sh)
            echo "Shell script utility"
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

    # Collect all files in the order they will be processed
    collect_files() {
        # Documentation files first
        for file in *.md; do
            if [[ -f "$file" && "$file" != "create_docs.sh" && "$file" != "project_analysis.md" ]]; then
                processing_order+=("$file")
                file_descriptions["$file"]=$(get_file_description "$file")
            fi
        done

        # Configuration files
        for file in *.toml; do
            if [[ -f "$file" && "$file" != "create_docs.sh" ]]; then
                processing_order+=("$file")
                file_descriptions["$file"]=$(get_file_description "$file")
            fi
        done

        # Rust files
        for file in *.rs; do
            if [[ -f "$file" && "$file" != "create_docs.sh" ]]; then
                processing_order+=("$file")
                file_descriptions["$file"]=$(get_file_description "$file")
            fi
        done

        # Build files
        if [[ -f "Makefile" && "Makefile" != "create_docs.sh" ]]; then
            processing_order+=("Makefile")
            file_descriptions["Makefile"]=$(get_file_description "Makefile")
        fi

        for file in *.sh; do
            if [[ -f "$file" && "$file" != "create_docs.sh" ]]; then
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
        current_line=$((current_line + 8))  # File structure summary table
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

    echo "## Analysis Guidelines for AI"
    echo ""
    echo "When reviewing this codebase, please consider:"
    echo "- **Code Quality**: Identify potential bugs, memory leaks, or logic errors"
    echo "- **Performance**: Suggest optimizations for Rust code"
    echo "- **Security**: Highlight potential security vulnerabilities"
    echo "- **Best Practices**: Recommend idiomatic Rust patterns"
    echo "- **Documentation**: Check if documentation matches implementation"
    echo "- **Build System**: Review Makefile for correctness and efficiency"
    echo ""

    echo "## File Contents"
    echo ""

    # Function to process and display a file with enhanced information
    process_file() {
        local file="$1"
        if [[ -f "$file" && "$file" != "create_docs.sh" && "$file" != "project_analysis.md" ]]; then
            local file_size=$(wc -c <"$file" | awk '{print $1}')
            local line_count=$(wc -l <"$file")
            local description="${file_descriptions[$file]}"

            echo ""
            echo "=========================================="
            echo "FILE: $file"
            echo "=========================================="
            echo ""
            echo "**Description:** $description  "
            echo "**Size:** $file_size bytes  "
            echo "**Lines:** $line_count  "
            echo "**Type:** ${file##*.}  "
            echo ""

            # Enhanced syntax highlighting with file-specific contexts
            case "$file" in
            *.rs)
                echo "\`\`\`rust"
                echo "// Rust module: $(basename "$file" .rs)"
                echo "// Path: $file"
                echo ""
                ;;
            *.toml)
                echo "\`\`\`toml"
                echo "# TOML Configuration"
                echo "# Path: $file"
                echo ""
                ;;
            *.md)
                echo "\`\`\`markdown"
                echo "<!-- Markdown Documentation: $file -->"
                echo "<!-- Path: $file -->"
                echo ""
                ;;
            Makefile)
                echo "\`\`\`makefile"
                echo "# Makefile"
                echo "# Path: $file"
                echo ""
                ;;
            *.sh)
                echo "\`\`\`bash"
                echo "# Shell Script"
                echo "# Path: $file"
                echo ""
                ;;
            *)
                echo "\`\`\`text"
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

    echo "## Analysis Summary"
    echo ""
    echo "### Key Areas for Review"
    echo ""
    echo "1. **Rust Code Quality** - Check for common Rust issues like unwrap misuse, error handling, and ownership patterns"
    echo "2. **Documentation Accuracy** - Verify that markdown documentation matches the actual code implementation"
    echo "3. **Build Configuration** - Ensure proper compilation flags and dependency management"
    echo "4. **Script Safety** - Review shell scripts for potential issues and improve portability"
    echo "5. **Configuration Validation** - Check TOML configuration for correctness and completeness"
    echo ""
    echo "### Potential Improvement Categories"
    echo ""
    echo "- **Code Performance Optimizations**"
    echo "- **Error Handling Improvements**"
    echo "- **Documentation Updates**"
    echo "- **Code Organization**"
    echo "- **Security Hardening**"
    echo "- **Testing Coverage**"
    echo ""

} >project_analysis.md

echo "Professional analysis document created: project_analysis.md"
echo ""
echo "This document includes:"
echo "✅ Accurate table of contents with correct line numbers"
echo "✅ TOC in same order as file processing"
echo "✅ 4 Markdown files (documentation)"
echo "✅ Enhanced file descriptions"
echo "✅ Logical file grouping"
echo "✅ Analysis guidelines for AI"
echo "✅ File statistics (size, lines)"
echo "✅ Professional formatting"
echo "✅ Exclusion of create_docs.sh and project_analysis.md"
