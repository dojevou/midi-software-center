#!/bin/bash
# STEP 2: Install correct MCP servers
# Run this AFTER 01_COMPLETE_MCP_RESET.sh

set -e

echo "📦 INSTALLING CORRECT MCP SERVERS"
echo "=================================="
echo ""

cd /home/dojevou/projects/midi-software-center

# Verify we're in the right directory
if [ ! -d ".git" ]; then
    echo "❌ Not in a git repository. Expected: /home/dojevou/projects/midi-software-center"
    exit 1
fi

echo "✅ Working in: $(pwd)"
echo ""

# ============================================================
# STEP 1: Add PROJECT-SCOPED servers (shared via .mcp.json)
# ============================================================
echo "📂 Adding PROJECT-SCOPED servers (will create .mcp.json)..."
echo ""

echo "   • filesystem..."
claude mcp add --transport stdio filesystem --scope project -- \
  npx -y @modelcontextprotocol/server-filesystem /home/dojevou/projects/midi-software-center

echo "   • memory..."
claude mcp add --transport stdio memory --scope project -- \
  npx -y @modelcontextprotocol/server-memory

echo "   • sequential-thinking..."
claude mcp add --transport stdio sequential-thinking --scope project -- \
  npx -y @modelcontextprotocol/server-sequential-thinking

echo ""
echo "✅ Project-scoped servers added"

# ============================================================
# STEP 2: Add USER-SCOPED server (private, for sensitive data)
# ============================================================
echo ""
echo "👤 Adding USER-SCOPED server (private to your machine)..."
echo ""

echo "   • postgres (with credentials)..."
claude mcp add --transport stdio postgres --scope user \
  --env DB_CONNECTION_STRING="postgresql://midiuser:145278963@localhost:5433/midi_library" \
  -- npx -y @modelcontextprotocol/server-postgres

echo ""
echo "✅ User-scoped server added"

# ============================================================
# STEP 3: Verify
# ============================================================
echo ""
echo "════════════════════════════════════════════"
echo "✨ SETUP COMPLETE!"
echo "════════════════════════════════════════════"
echo ""

echo "📋 Listing all configured servers:"
claude mcp list

echo ""
echo "✅ Configuration locations:"
echo "   • User-scoped:    ~/.claude.json"
echo "   • Project-scoped: $(pwd)/.mcp.json"
echo ""

echo "📝 .mcp.json is git-tracked. You can commit it:"
echo "   git add .mcp.json"
echo "   git commit -m 'Add MCP server configuration'"
echo ""

echo "🚀 Test in Claude Code:"
echo "   cc"
echo "   /mcp"
echo ""
echo "Should see all 4 servers with ✔ connected"
echo ""
