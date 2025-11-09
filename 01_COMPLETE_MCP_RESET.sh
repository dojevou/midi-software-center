#!/bin/bash
# COMPLETE MCP RESET & CLEANUP
# Removes ALL old configs and resets to clean state

set -e

echo "🧹 COMPLETE MCP CLEANUP"
echo "======================="
echo ""
echo "⚠️  This will:"
echo "   • Delete ~/.mcp-user.json"
echo "   • Clear mcpServers from ~/.claude.json"
echo "   • Delete .mcp.json from project"
echo "   • Reset Claude Code MCP configuration"
echo ""
read -p "Continue? (type 'yes' to confirm): " confirm
if [ "$confirm" != "yes" ]; then
    echo "Aborted."
    exit 1
fi

echo ""
echo "🗑️  Deleting old configuration files..."

# 1. Delete ~/.mcp-user.json (non-standard, not recognized)
if [ -f ~/.mcp-user.json ]; then
    rm ~/.mcp-user.json
    echo "   ✅ Deleted ~/.mcp-user.json"
else
    echo "   ℹ️  ~/.mcp-user.json doesn't exist (ok)"
fi

# 2. Delete project .mcp.json
if [ -f /home/dojevou/projects/midi-software-center/.mcp.json ]; then
    rm /home/dojevou/projects/midi-software-center/.mcp.json
    echo "   ✅ Deleted project .mcp.json"
else
    echo "   ℹ️  Project .mcp.json doesn't exist (ok)"
fi

# 3. Clear mcpServers from ~/.claude.json
echo ""
echo "🔄 Clearing mcpServers from ~/.claude.json..."

python3 << 'PYTHON_EOF'
import json
import os

claude_json_path = os.path.expanduser('~/.claude.json')

if os.path.exists(claude_json_path):
    with open(claude_json_path, 'r') as f:
        config = json.load(f)
    
    # Remove root-level mcpServers (user-scoped)
    if 'mcpServers' in config:
        old_count = len(config['mcpServers'])
        config['mcpServers'] = {}
        print(f"   ✅ Cleared {old_count} user-scoped servers from root level")
    
    # Remove project-level mcpServers (shouldn't be here, but just in case)
    if 'projects' in config:
        for project_path in config['projects']:
            if 'mcpServers' in config['projects'][project_path]:
                old_count = len(config['projects'][project_path]['mcpServers'])
                if old_count > 0:
                    config['projects'][project_path]['mcpServers'] = {}
                    print(f"   ✅ Cleared {old_count} servers from project config: {project_path}")
    
    with open(claude_json_path, 'w') as f:
        json.dump(config, f, indent=2)
    
    print("   ✅ ~/.claude.json cleaned")
else:
    print("   ⚠️  ~/.claude.json not found - this is unusual!")

PYTHON_EOF

# 4. Remove cached MCP configurations
echo ""
echo "🗑️  Cleaning cache and temp files..."

if [ -d ~/.cache/claude-code ]; then
    rm -rf ~/.cache/claude-code 2>/dev/null || true
    echo "   ✅ Cleared ~/.cache/claude-code"
fi

if [ -d ~/.config/claude-code ]; then
    rm -rf ~/.config/claude-code 2>/dev/null || true
    echo "   ✅ Cleared ~/.config/claude-code"
fi

# 5. Remove Claude Code process to force reload
echo ""
echo "🔄 Resetting Claude Code..."
pkill -f "claude mcp serve" 2>/dev/null || true
pkill -f "claude code" 2>/dev/null || true
echo "   ✅ Terminated Claude Code processes"

echo ""
echo "════════════════════════════════════════════"
echo "✨ RESET COMPLETE!"
echo "════════════════════════════════════════════"
echo ""
echo "📋 Current state:"
echo "   • ~/.mcp-user.json: DELETED ✓"
echo "   • project .mcp.json: DELETED ✓"
echo "   • ~/. claude.json mcpServers: CLEARED ✓"
echo ""
echo "🚀 Next steps:"
echo "   1. Run: 02_SETUP_MCP_CORRECT.sh"
echo "   OR use: claude mcp add --transport stdio ..."
echo ""
