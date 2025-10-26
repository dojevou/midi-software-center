# Claude Code Agents for MIDI Software Center

This package contains 5 specialized agents designed to help you build the MIDI Software Center project following the **Three Archetypes** pattern.

## 🎯 Quick Start

```bash
# 1. Run the setup script
./setup-agents.sh

# 2. Choose installation location
# - Personal (~/.claude/agents/) - Available in all projects
# - Project (.claude/agents/) - Available only in this project, can be shared with team

# 3. Start using agents in Claude Code
> /agents                    # List available agents
> /agent rust-backend        # Select an agent
```

## 📦 Included Agents

| Agent | Model | Purpose |
|-------|-------|---------|
| **rust-backend** | Sonnet | Backend Rust, Tauri commands, async operations |
| **frontend** | Sonnet | Svelte/TypeScript, stores, components |
| **architecture-reviewer** | Sonnet | Code review, Three Archetypes compliance |
| **database** | Sonnet | PostgreSQL, migrations, repositories |
| **midi-hardware** | Sonnet | MIDI parsing, hardware integration, ALSA |

## 🏗️ Architecture Overview

All agents understand and enforce the **Three Archetypes** pattern:

### 1. Task-O-Matic (Entry Points)
**What**: Programs you run, components you render  
**Rust**: `main.rs`, `bin/*.rs`  
**Frontend**: `*.svelte`, `routes/*.svelte`  
**Database**: `migrations/*.sql`

### 2. Grown-up Script (I/O & Side Effects)
**What**: Code that talks to the outside world  
**Rust**: `commands/*.rs`, `services/*.rs`, `db/repositories/*.rs`  
**Frontend**: `stores/*.ts`  
**Database**: Repository implementations

### 3. Trusty Module (Pure Logic)
**What**: Pure functions you can trust  
**Rust**: `core/*.rs`, `shared/rust/src/core/*.rs`  
**Frontend**: `utils/*.ts`, `types/*.ts`  
**Database**: `models/*.rs`

## 💡 Usage Examples

### Example 1: New Backend Feature
```bash
> /agent rust-backend
"Create a Tauri command to import MIDI files with BPM analysis"

> /agent architecture-reviewer
"Review the import command for Three Archetypes compliance"
```

### Example 2: Full-Stack Feature
```bash
# 1. Database schema
> /agent database
"Create tables for MIDI projects and tracks"

# 2. Backend implementation
> /agent rust-backend
"Create Tauri commands for project CRUD operations"

# 3. Frontend integration
> /agent frontend
"Create Svelte store and components for project management"

# 4. Review everything
> /agent architecture-reviewer
"Review the entire project management feature"
```

### Example 3: MIDI Processing
```bash
# 1. Pure parsing logic
> /agent midi-hardware
"Create MIDI file parser in core/midi/parser.rs"

# 2. Hardware integration
> /agent midi-hardware
"Create MidiDeviceManager for MPC ONE"

# 3. Tauri commands
> /agent rust-backend
"Add commands to connect to MIDI devices"

# 4. Frontend UI
> /agent frontend
"Create component to display MIDI devices"
```

## 🎓 Learning Path

### For New Team Members
1. **Understand the pattern**
   ```bash
   > /agent architecture-reviewer
   "Explain the Three Archetypes pattern with examples"
   ```

2. **Learn backend patterns**
   ```bash
   > /agent rust-backend
   "Show me the entry + implementation pattern"
   ```

3. **Learn frontend patterns**
   ```bash
   > /agent frontend
   "How do I create a store with Tauri IPC?"
   ```

4. **Learn database patterns**
   ```bash
   > /agent database
   "Show me the repository pattern"
   ```

## 📋 Agent Decision Matrix

| Task | Primary Agent | Secondary | Review |
|------|--------------|-----------|--------|
| Tauri commands | rust-backend | - | architecture-reviewer |
| MIDI parsing | midi-hardware | - | architecture-reviewer |
| Database migration | database | - | architecture-reviewer |
| Svelte components | frontend | - | architecture-reviewer |
| Stores + IPC | frontend | rust-backend | architecture-reviewer |
| Hardware I/O | midi-hardware | rust-backend | architecture-reviewer |

## 🔄 Typical Workflow

```
1. architecture-reviewer → Classify feature into archetypes
2. database → Schema changes if needed
3. rust-backend/midi-hardware → Core logic + commands
4. frontend → UI components + stores
5. architecture-reviewer → Final review
```

## 📚 Documentation

- **AGENT-SETUP-GUIDE.md** - Comprehensive usage guide
  - Multi-agent workflows
  - Best practices
  - Common mistakes
  - Debugging with agents
  
- **Agent Configuration Files** (*.toml)
  - rust-backend-agent.toml
  - frontend-agent.toml
  - architecture-reviewer-agent.toml
  - database-agent.toml
  - midi-hardware-agent.toml

## 🎯 Critical Rules Enforced

All agents enforce these critical rules:

1. **No .unwrap() in production code**
   - Always use proper error handling

2. **80%+ test coverage for Trusty Modules**
   - Required and enforced

3. **No I/O in core/ directories**
   - Core must contain only pure functions

4. **Entry + implementation pattern**
   - For all Tauri commands

5. **Proper error types**
   - anyhow for apps, thiserror for libraries

## 🛠️ Project Stack

### Backend
- **Rust** + **Tauri** - Native performance with web UI
- **tokio** - Async runtime
- **SQLx** - Type-safe database access
- **midir** - MIDI device integration

### Frontend
- **Svelte** - Reactive UI framework
- **TypeScript** - Type safety
- **Tailwind CSS** - Styling

### Database
- **PostgreSQL** - Structured data storage
- **Meilisearch** - Fast full-text search

### Hardware
- AKAI MPC ONE - MIDI controller
- AKAI FORCE - Music production system
- Steinberg UR22 - Audio interface
- NEUMANN TLM 107 - Studio microphone
- EMU PROTEUS 2000 - Sound module

## 🚨 Common Mistakes

### ❌ Wrong Agent Selection
```bash
# DON'T ask frontend agent about Rust
> /agent frontend
"How do I fix this Rust compiler error?"

# DO use rust-backend for Rust
> /agent rust-backend
"How do I fix this Rust compiler error?"
```

### ❌ Skipping Architecture Review
```bash
# DON'T skip the review
> /agent rust-backend
[implement feature]
# Ship it!

# DO always review
> /agent rust-backend
[implement feature]
> /agent architecture-reviewer
"Review this feature"
```

### ❌ Violating Archetype Rules
```bash
# DON'T put I/O in core/
// core/parser.rs
pub fn parse_file(path: &Path) { /* reads file */ }  // ❌

# DO separate concerns
// core/parser.rs (Trusty Module)
pub fn parse_midi(data: &[u8]) { /* pure logic */ }  // ✅

// services/file_service.rs (Grown-up Script)
pub async fn parse_file(path: &Path) {
    let data = tokio::fs::read(path).await?;
    parse_midi(&data)  // Calls Trusty Module
}
```

## 🔧 Customization

### Modifying Agents
Edit the `.toml` files to customize:
- System prompts
- Tool availability
- Context files
- Model selection

### Creating New Agents
```toml
name = "your-agent"
model = "sonnet"
description = "Agent description"

[system_prompt]
content = """
Your specialized knowledge...
"""

[tools]
enabled = ["read", "write", "search", "terminal"]

[context]
include = ["**/*.your-files"]
```

## 📞 Support

### Getting Help
```bash
# In Claude Code
> /help agents

# Ask architecture-reviewer
> /agent architecture-reviewer
"I'm new to the project. What should I know?"
```

### Team Sharing
- Install agents to `.claude/agents/` (project directory)
- Commit to git for team access
- Share AGENT-SETUP-GUIDE.md with team

## 🎉 What Makes This Special?

These agents are **production-ready** with:

✅ **Full architecture knowledge** - Deep understanding of Three Archetypes  
✅ **Stack-specific patterns** - Rust/Tauri, Svelte, PostgreSQL, MIDI  
✅ **Rule enforcement** - No .unwrap(), proper error handling, test coverage  
✅ **Hardware integration** - Real MIDI devices (MPC ONE, UR22, etc.)  
✅ **Team-optimized** - Consistent code across all developers  
✅ **Context-aware** - Understands project structure and patterns  

## 🚀 Getting Started

```bash
# 1. Install agents
./setup-agents.sh

# 2. Read the comprehensive guide
cat AGENT-SETUP-GUIDE.md

# 3. Start building
# Open Claude Code in your project
# Use > /agent [agent-name] to get started

# 4. Always review your work
> /agent architecture-reviewer
"Review my latest changes"
```

---

**Built for**: MIDI Software Center Project  
**Platform**: Ubuntu Studio 25.04  
**Tools**: Claude Code, VS Code  
**Hardware**: MPC ONE, AKAI FORCE, UR22, TLM 107, Proteus 2000  

**Remember**: The agents are here to help maintain consistency and enforce best practices. Use them liberally!
