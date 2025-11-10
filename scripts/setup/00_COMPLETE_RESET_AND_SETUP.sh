#!/bin/bash
# ALL-IN-ONE: Complete MCP Reset + Setup
# Does everything in one command

set -e

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║        COMPLETE MCP RESET & CORRECT SETUP                 ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "This will:"
echo "  ✓ Delete old incorrect configs (~/.mcp-user.json, old .mcp.json)"
echo "  ✓ Clear ~/.claude.json MCP settings"
echo "  ✓ Add 4 correct MCP servers (filesystem, memory, thinking, postgres)"
echo ""

read -p "Continue? (type 'yes'): " confirm
if [ "$confirm" != "yes" ]; then
    echo "Aborted."
    exit 1
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "STEP 1: CLEANUP"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Delete ~/.mcp-user.json
if [ -f ~/.mcp-user.json ]; then
    rm ~/.mcp-user.json
    echo "✅ Deleted ~/.mcp-user.json"
else
    echo "ℹ️  ~/.mcp-user.json not found (ok)"
fi

# Delete project .mcp.json
if [ -f /home/dojevou/projects/midi-software-center/.mcp.json ]; then
    rm /home/dojevou/projects/midi-software-center/.mcp.json
    echo "✅ Deleted /home/dojevou/projects/midi-software-center/.mcp.json"
else
    echo "ℹ️  Project .mcp.json not found (ok)"
fi

# Clear ~/.claude.json mcpServers
echo "Clearing ~/.claude.json..."

python3 << 'PYTHON_EOF'
import json
import os

claude_json_path = os.path.expanduser('~/.claude.json')

if os.path.exists(claude_json_path):
    with open(claude_json_path, 'r') as f:
        config = json.load(f)
    
    changes = False
    
    # Clear root-level mcpServers
    if 'mcpServers' in config and config['mcpServers']:
        old_count = len(config['mcpServers'])
        config['mcpServers'] = {}
        print(f"✅ Cleared {old_count} user-scoped servers")
        changes = True
    
    # Clear project-level mcpServers
    if 'projects' in config:
        for project_path in config['projects']:
            if 'mcpServers' in config['projects'][project_path]:
                if config['projects'][project_path]['mcpServers']:
                    old_count = len(config['projects'][project_path]['mcpServers'])
                    config['projects'][project_path]['mcpServers'] = {}
                    print(f"✅ Cleared {old_count} project-scoped servers in {project_path}")
                    changes = True
    
    if changes:
        with open(claude_json_path, 'w') as f:
            json.dump(config, f, indent=2)
        print("✅ ~/.claude.json cleaned")
    else:
        print("ℹ️  No MCP servers to clear")
else:
    print("❌ ~/.claude.json not found!")
    exit(1)

PYTHON_EOF

# Clear cache
echo "Clearing cache..."
rm -rf ~/.cache/claude-code 2>/dev/null || true
rm -rf ~/.config/claude-code 2>/dev/null || true
echo "✅ Cache cleared"

# Kill processes
pkill -f "claude" 2>/dev/null || true
sleep 1
echo "✅ Claude processes terminated"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "STEP 2: SETUP CORRECT SERVERS"
echo "═══════════════════════════════════════════════════════════════"
echo ""

cd /home/dojevou/projects/midi-software-center

if [ ! -d ".git" ]; then
    echo "❌ Not in git repo. Expected: /home/dojevou/projects/midi-software-center"
    exit 1
fi

echo "Working directory: $(pwd)"
echo ""

# Project-scoped servers
echo "📂 Adding PROJECT-SCOPED servers (will create .mcp.json)..."
echo ""

echo "   ⏳ filesystem..."
claude mcp add --transport stdio filesystem --scope project -- \
  npx -y @modelcontextprotocol/server-filesystem /home/dojevou/projects/midi-software-center
echo "      ✅ Done"

echo "   ⏳ memory..."
claude mcp add --transport stdio memory --scope project -- \
  npx -y @modelcontextprotocol/server-memory
echo "      ✅ Done"

echo "   ⏳ sequential-thinking..."
claude mcp add --transport stdio sequential-thinking --scope project -- \
  npx -y @modelcontextprotocol/server-sequential-thinking
echo "      ✅ Done"

# User-scoped server
echo ""
echo "👤 Adding USER-SCOPED server (private to your machine)..."
echo ""

echo "   ⏳ postgres (with DB credentials)..."
claude mcp add --transport stdio postgres --scope user \
  --env DB_CONNECTION_STRING="postgresql://midiuser:145278963@localhost:5433/midi_library" \
  -- npx -y @modelcontextprotocol/server-postgres
echo "      ✅ Done"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "✨ COMPLETE! All servers configured correctly"
echo "═══════════════════════════════════════════════════════════════"
echo ""

echo "📋 Verification:"
echo ""
claude mcp list

echo ""
echo "📂 File locations:"
echo "   User-scoped:    ~/.claude.json"
echo "   Project-scoped: $(pwd)/.mcp.json"
echo ""

echo "✅ Next steps:"
echo ""
echo "   1. Commit .mcp.json to git:"
echo "      $ git add .mcp.json"
echo "      $ git commit -m 'Add MCP server configuration'"
echo ""
echo "   2. Test in Claude Code:"
echo "      $ cc"
echo "      > /mcp"
echo ""
echo "   3. You should see all 4 servers with ✔ connected"
echo ""
