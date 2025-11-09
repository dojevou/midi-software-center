#!/bin/bash
# COMPLETE MCP SETUP WITH ALL USEFUL SERVERS
# For: /home/dojevou/projects/midi-software-center
# Run from: PROJECT ROOT (not ~)

set -e

echo ""
echo "╔════════════════════════════════════════════════════════════════════════╗"
echo "║           COMPLETE MCP SETUP - ALL USEFUL SERVERS                     ║"
echo "║                   MIDI Software Center Project                         ║"
echo "╚════════════════════════════════════════════════════════════════════════╝"
echo ""

# ============================================================
# CRITICAL: Verify we're in the PROJECT ROOT
# ============================================================
PROJECT_ROOT="/home/dojevou/projects/midi-software-center"

if [ ! -d ".git" ]; then
    echo "❌ ERROR: Must run from PROJECT ROOT"
    echo ""
    echo "   Expected: $PROJECT_ROOT"
    echo "   Got: $(pwd)"
    echo ""
    echo "Fix: cd $PROJECT_ROOT"
    exit 1
fi

if [ "$(pwd)" != "$PROJECT_ROOT" ]; then
    echo "❌ ERROR: Wrong directory"
    echo ""
    echo "   Expected: $PROJECT_ROOT"
    echo "   Got: $(pwd)"
    echo ""
    echo "Fix: cd $PROJECT_ROOT && bash $0"
    exit 1
fi

echo "✅ Confirmed: Running from PROJECT ROOT"
echo "   $(pwd)"
echo ""

# ============================================================
# STEP 1: CLEANUP
# ============================================================
read -p "Continue with cleanup? (yes/no): " confirm
if [ "$confirm" != "yes" ]; then
    echo "Aborted."
    exit 1
fi

echo ""
echo "═══════════════════════════════════════════════════════════════════════"
echo "STEP 1: CLEANUP"
echo "═══════════════════════════════════════════════════════════════════════"
echo ""

# Delete ~/.mcp-user.json
if [ -f ~/.mcp-user.json ]; then
    rm ~/.mcp-user.json
    echo "✅ Deleted ~/.mcp-user.json"
else
    echo "ℹ️  ~/.mcp-user.json not found"
fi

# Delete project .mcp.json
if [ -f .mcp.json ]; then
    rm .mcp.json
    echo "✅ Deleted .mcp.json"
else
    echo "ℹ️  .mcp.json not found"
fi

# Clear ~/.claude.json
python3 << 'PYTHON_EOF'
import json
import os

claude_json_path = os.path.expanduser('~/.claude.json')

if os.path.exists(claude_json_path):
    with open(claude_json_path, 'r') as f:
        config = json.load(f)
    
    changes = False
    
    if 'mcpServers' in config and config['mcpServers']:
        old_count = len(config['mcpServers'])
        config['mcpServers'] = {}
        print(f"✅ Cleared {old_count} user-scoped servers")
        changes = True
    
    if 'projects' in config:
        for project_path in config['projects']:
            if 'mcpServers' in config['projects'][project_path]:
                if config['projects'][project_path]['mcpServers']:
                    old_count = len(config['projects'][project_path]['mcpServers'])
                    config['projects'][project_path]['mcpServers'] = {}
                    print(f"✅ Cleared {old_count} project servers")
                    changes = True
    
    if changes:
        with open(claude_json_path, 'w') as f:
            json.dump(config, f, indent=2)
        print("✅ ~/.claude.json cleaned")

PYTHON_EOF

# Clear cache
pkill -f "claude" 2>/dev/null || true
sleep 1
echo "✅ Cache and processes cleared"

echo ""
echo "═══════════════════════════════════════════════════════════════════════"
echo "STEP 2: SETUP - CORE SERVERS (Essential for all projects)"
echo "═══════════════════════════════════════════════════════════════════════"
echo ""

echo "📂 Adding core project-scoped servers..."
echo ""

echo "   ⏳ filesystem (project file access)..."
claude mcp add --transport stdio filesystem --scope project -- \
  npx -y @modelcontextprotocol/server-filesystem "$PROJECT_ROOT" > /dev/null
echo "      ✅ Done"

echo "   ⏳ memory (context storage)..."
claude mcp add --transport stdio memory --scope project -- \
  npx -y @modelcontextprotocol/server-memory > /dev/null
echo "      ✅ Done"

echo "   ⏳ sequential-thinking (complex reasoning)..."
claude mcp add --transport stdio sequential-thinking --scope project -- \
  npx -y @modelcontextprotocol/server-sequential-thinking > /dev/null
echo "      ✅ Done"

echo ""
echo "═══════════════════════════════════════════════════════════════════════"
echo "STEP 3: SETUP - USER-SCOPED SERVERS (Private, sensitive data)"
echo "═══════════════════════════════════════════════════════════════════════"
echo ""

echo "👤 Adding user-scoped server with credentials..."
echo ""

echo "   ⏳ postgres (database queries)..."
claude mcp add --transport stdio postgres --scope user \
  --env DB_CONNECTION_STRING="postgresql://midiuser:145278963@localhost:5433/midi_library" \
  -- npx -y @modelcontextprotocol/server-postgres > /dev/null
echo "      ✅ Done"

echo ""
echo "═══════════════════════════════════════════════════════════════════════"
echo "STEP 4: SETUP - DEVELOPMENT SERVERS (Recommended for MIDI project)"
echo "═══════════════════════════════════════════════════════════════════════"
echo ""

echo "🔧 Adding development & monitoring servers..."
echo ""

# Sentry - error monitoring
echo "   ⏳ sentry (error monitoring)..."
claude mcp add --transport http sentry --scope project \
  https://mcp.sentry.dev/mcp > /dev/null
echo "      ✅ Done"

# Socket - security analysis
echo "   ⏳ socket (dependency security)..."
claude mcp add --transport http socket --scope project \
  https://mcp.socket.dev/ > /dev/null
echo "      ✅ Done"

# Hugging Face - ML/AI models
echo "   ⏳ huggingface (ML models, datasets)..."
claude mcp add --transport http hugging-face --scope project \
  https://huggingface.co/mcp > /dev/null
echo "      ✅ Done"

echo ""
echo "═══════════════════════════════════════════════════════════════════════"
echo "STEP 5: SETUP - PROJECT MANAGEMENT (Choose your tool below)"
echo "═══════════════════════════════════════════════════════════════════════"
echo ""

echo "Choose ONE issue tracker:"
echo "  1. Linear (modern, recommended)"
echo "  2. Asana (visual, popular)"
echo "  3. Atlassian/Jira (enterprise)"
echo "  4. ClickUp (all-in-one)"
echo "  5. Monday (project boards)"
echo "  6. Skip for now"
echo ""

read -p "Enter choice (1-6): " pm_choice

case $pm_choice in
  1)
    echo "   ⏳ Adding Linear..."
    claude mcp add --transport http linear --scope project \
      https://mcp.linear.app/mcp > /dev/null
    echo "      ✅ Linear added"
    ;;
  2)
    echo "   ⏳ Adding Asana..."
    claude mcp add --transport sse asana --scope project \
      https://mcp.asana.com/sse > /dev/null
    echo "      ✅ Asana added"
    ;;
  3)
    echo "   ⏳ Adding Atlassian (Jira/Confluence)..."
    claude mcp add --transport sse atlassian --scope project \
      https://mcp.atlassian.com/v1/sse > /dev/null
    echo "      ✅ Atlassian added"
    ;;
  4)
    echo "   ⏳ Adding ClickUp..."
    claude mcp add --transport stdio clickup --scope project \
      --env CLICKUP_API_KEY="YOUR_API_KEY" \
      --env CLICKUP_TEAM_ID="YOUR_TEAM_ID" \
      -- npx -y @hauptsache.net/clickup-mcp > /dev/null
    echo "      ✅ ClickUp added (update API keys in ~/.claude.json)"
    ;;
  5)
    echo "   ⏳ Adding Monday.com..."
    claude mcp add --transport http monday --scope project \
      https://mcp.monday.com/mcp > /dev/null
    echo "      ✅ Monday added"
    ;;
  *)
    echo "   ℹ️  Skipping project management tool"
    ;;
esac

echo ""
echo "═══════════════════════════════════════════════════════════════════════"
echo "STEP 6: SETUP - DOCUMENTATION (Choose your tool below)"
echo "═══════════════════════════════════════════════════════════════════════"
echo ""

echo "Choose ONE documentation tool:"
echo "  1. Notion (popular, flexible)"
echo "  2. Confluence/Atlassian (if using Jira)"
echo "  3. Box (enterprise content)"
echo "  4. Skip for now"
echo ""

read -p "Enter choice (1-4): " doc_choice

case $doc_choice in
  1)
    echo "   ⏳ Adding Notion..."
    claude mcp add --transport http notion --scope project \
      https://mcp.notion.com/mcp > /dev/null
    echo "      ✅ Notion added"
    ;;
  2)
    echo "   ⏳ Adding Confluence..."
    claude mcp add --transport sse confluence --scope project \
      https://mcp.atlassian.com/v1/sse > /dev/null
    echo "      ✅ Confluence added"
    ;;
  3)
    echo "   ⏳ Adding Box..."
    claude mcp add --transport http box --scope project \
      https://mcp.box.com/ > /dev/null
    echo "      ✅ Box added"
    ;;
  *)
    echo "   ℹ️  Skipping documentation tool"
    ;;
esac

echo ""
echo "═══════════════════════════════════════════════════════════════════════"
echo "✨ SETUP COMPLETE!"
echo "═══════════════════════════════════════════════════════════════════════"
echo ""

echo "📋 All configured servers:"
claude mcp list

echo ""
echo "📂 File locations:"
echo "   User-scoped:    ~/.claude.json"
echo "   Project-scoped: $(pwd)/.mcp.json"
echo ""

echo "🔐 Important: Sensitive credentials stored in ~/.claude.json"
echo "   Do NOT commit this to git"
echo ""

echo "✅ Ready to use! In Claude Code:"
echo ""
echo "   $ cc"
echo "   > /mcp"
echo ""
echo "   Should show all configured servers with ✔ connected"
echo ""

echo "📝 Next: Commit .mcp.json to git (no credentials in it)"
echo ""
echo "   $ git add .mcp.json"
echo "   $ git commit -m 'Add MCP servers for team'"
echo ""
