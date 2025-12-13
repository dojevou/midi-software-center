# Claude Code Agent Setup Guide
## MIDI Software Center Project

This guide explains how to use specialized Claude Code agents for your MIDI software project.

## 📋 Agent Overview

### 1. **rust-backend** (Sonnet)
**When to use**: Backend Rust development, Tauri commands, async operations
- Understands Three Archetypes pattern
- Enforces no .unwrap() rule
- Implements entry + implementation pattern
- Knows MIDI processing patterns
- Handles error types correctly

**Best for**:
- Creating Tauri commands
- Writing async services
- Building pure core logic
- MIDI file I/O

### 2. **frontend** (Sonnet)
**When to use**: Svelte/TypeScript UI development, stores, components
- Knows Svelte reactivity patterns
- Implements proper store management
- Handles Tauri IPC correctly
- Manages loading/error states

**Best for**:
- Creating Svelte components
- Building stores with Tauri integration
- Writing pure utility functions
- Type-safe frontend code

### 3. **architecture-reviewer** (Sonnet)
**When to use**: Code review, ensuring architectural compliance
- Reviews Three Archetypes classification
- Checks file placement
- Enforces coding rules
- Validates test coverage

**Best for**:
- Reviewing PRs
- Checking new features
- Ensuring consistency
- Validating architecture

### 4. **database** (Sonnet)
**When to use**: PostgreSQL schema design, migrations, repositories
- Knows SQLx patterns
- Implements repository pattern
- Writes type-safe queries
- Handles transactions

**Best for**:
- Creating migrations
- Building repositories
- Writing database tests
- Query optimization

### 5. **midi-hardware** (Sonnet)
**When to use**: MIDI processing, hardware integration, ALSA
- Understands MIDI protocol
- Knows hardware devices (MPC ONE, UR22)
- Implements pure parsing
- Handles real-time constraints

**Best for**:
- MIDI parsing logic
- Hardware device integration
- BPM/key detection
- Real-time audio code

## 🚀 Installation

### Option 1: Personal Agents (~/.claude/agents/)
```bash
# Copy agents to personal directory
mkdir -p ~/.claude/agents
cp *.toml ~/.claude/agents/

# Restart Claude Code
# Agents will be available in all projects
```

### Option 2: Project Agents (.claude/agents/)
```bash
# Copy agents to project directory
mkdir -p .claude/agents
cp *.toml .claude/agents/

# Commit to git for team sharing
git add .claude/agents/
git commit -m "Add Claude Code agents"
```

## 💡 Usage Patterns

### Pattern 1: New Backend Feature
```bash
# 1. Use rust-backend to implement feature
> /agent rust-backend
"Create a Tauri command to import MIDI files with analysis"

# 2. Review with architecture-reviewer
> /agent architecture-reviewer
"Review the new import command for Three Archetypes compliance"

# 3. Test and iterate
```

### Pattern 2: Database Changes
```bash
# 1. Use database agent for migration
> /agent database
"Create a migration for storing MIDI analysis results"

# 2. Create repository with database agent
> /agent database
"Create a repository for the midi_analysis table"

# 3. Integrate with rust-backend
> /agent rust-backend
"Add Tauri command to save MIDI analysis to database"
```

### Pattern 3: Full-Stack Feature
```bash
# 1. Database schema
> /agent database
"Create tables for MIDI projects and tracks"

# 2. Backend implementation
> /agent rust-backend
"Create Tauri commands for CRUD operations on projects"

# 3. Frontend integration
> /agent frontend
"Create Svelte store and components for project management"

# 4. Architecture review
> /agent architecture-reviewer
"Review the entire project management feature"
```

### Pattern 4: MIDI Processing
```bash
# 1. Pure parsing logic
> /agent midi-hardware
"Create MIDI parser for .mid files in core/"

# 2. Hardware integration
> /agent midi-hardware
"Create MidiDeviceManager for MPC ONE integration"

# 3. Backend commands
> /agent rust-backend
"Add Tauri commands to connect to MIDI devices"

# 4. Frontend UI
> /agent frontend
"Create component to display and connect MIDI devices"
```

## 🎯 Agent Specialization Matrix

| Task | Primary Agent | Secondary Agent | Review Agent |
|------|--------------|-----------------|--------------|
| Tauri commands | rust-backend | - | architecture-reviewer |
| Pure Rust logic | rust-backend | - | architecture-reviewer |
| MIDI parsing | midi-hardware | - | architecture-reviewer |
| Database migration | database | - | architecture-reviewer |
| Repository pattern | database | rust-backend | architecture-reviewer |
| Svelte components | frontend | - | architecture-reviewer |
| Stores + IPC | frontend | rust-backend | architecture-reviewer |
| Hardware I/O | midi-hardware | rust-backend | architecture-reviewer |
| Architecture review | architecture-reviewer | - | - |

## 🔄 Multi-Agent Workflows

### Workflow 1: Complete Feature Development
```
1. architecture-reviewer → Classify feature into archetypes
2. database → Schema if needed
3. rust-backend → Core logic + commands
4. frontend → UI components + stores
5. architecture-reviewer → Final review
```

### Workflow 2: Bug Fix
```
1. architecture-reviewer → Identify which archetype has the bug
2. [specialized-agent] → Fix the bug
3. architecture-reviewer → Verify fix doesn't violate patterns
```

### Workflow 3: Refactoring
```
1. architecture-reviewer → Identify violations
2. [specialized-agent] → Refactor to correct archetype
3. architecture-reviewer → Verify compliance
```

## 🛠️ Best Practices

### 1. Always Start with Classification
```bash
> /agent architecture-reviewer
"I want to add a feature to detect BPM from uploaded MIDI files. 
Which archetypes do I need?"

# Response will guide you to use:
# - Trusty Module for BPM algorithm
# - Grown-up Script for file reading
# - Task-O-Matic for Tauri command
```

### 2. Use the Right Agent for Each Layer
```bash
# ❌ WRONG - Using frontend agent for Rust code
> /agent frontend
"Write Rust code to parse MIDI files"

# ✅ CORRECT - Using midi-hardware for MIDI parsing
> /agent midi-hardware
"Write pure Rust function to parse MIDI files"
```

### 3. Always Review Architecture
```bash
# After implementing a feature
> /agent architecture-reviewer
"Review the BPM detection feature I just implemented"

# Will check:
# - Correct archetype classification
# - Proper file placement
# - No violations of rules
# - Test coverage
```

### 4. Chain Agents for Complex Tasks
```bash
# Task: Full-stack MIDI import feature

# Step 1: Schema
> /agent database
"Create migration for midi_imports table"

# Step 2: Pure logic
> /agent midi-hardware
"Create MIDI parser in core/midi/parser.rs"

# Step 3: Repository
> /agent database
"Create MidiImportRepository"

# Step 4: Service layer
> /agent rust-backend
"Create import service using parser and repository"

# Step 5: Tauri commands
> /agent rust-backend
"Create Tauri commands for import workflow"

# Step 6: Frontend store
> /agent frontend
"Create importStore with Tauri IPC"

# Step 7: UI components
> /agent frontend
"Create ImportDialog.svelte component"

# Step 8: Review
> /agent architecture-reviewer
"Review entire MIDI import feature"
```

## 📝 Agent Communication

### Passing Context Between Agents
When working with multiple agents, you can reference previous work:

```bash
# Agent 1 creates foundation
> /agent rust-backend
"Create MIDI parsing service"

# Agent 2 builds on top
> /agent frontend
"Create UI for the MIDI parsing service created by rust-backend agent.
The service has these commands: parse_midi_file, get_midi_info"

# Agent 3 reviews everything
> /agent architecture-reviewer
"Review both the MIDI parsing service and its frontend integration"
```

## 🔍 Debugging with Agents

### When Things Go Wrong
```bash
# Error: .unwrap() in production code
> /agent rust-backend
"I'm getting a panic from .unwrap(). Fix this function: [paste code]"

# Error: Component too large
> /agent frontend
"This component is 400 lines. Split it following best practices: [paste code]"

# Error: Missing tests
> /agent architecture-reviewer
"Check test coverage for core/midi/parser.rs"

# Error: Wrong archetype
> /agent architecture-reviewer
"I put parsing logic in commands/. Where should it be?"
```

## 🎓 Learning the Architecture

### For New Team Members
```bash
# 1. Understand the pattern
> /agent architecture-reviewer
"Explain the Three Archetypes pattern with examples from our codebase"

# 2. Learn backend patterns
> /agent rust-backend
"Show me an example of the entry + implementation pattern"

# 3. Learn frontend patterns
> /agent frontend
"Show me how to create a store with Tauri IPC"

# 4. Learn database patterns
> /agent database
"Show me the repository pattern for our project"

# 5. Learn MIDI patterns
> /agent midi-hardware
"How should I structure MIDI parsing vs hardware I/O?"
```

## 🚨 Common Mistakes to Avoid

### Mistake 1: Using Wrong Agent
```bash
# ❌ Don't ask frontend agent about Rust
> /agent frontend
"How do I fix this Rust compiler error?"

# ✅ Use rust-backend for Rust questions
> /agent rust-backend
"How do I fix this Rust compiler error?"
```

### Mistake 2: Skipping Architecture Review
```bash
# ❌ Don't skip the review
> /agent rust-backend
[implement feature]
# Ship it! (without review)

# ✅ Always review architecture
> /agent rust-backend
[implement feature]
> /agent architecture-reviewer
"Review this feature"
# Fix any issues, then ship
```

### Mistake 3: Not Following Agent Guidance
```bash
# ❌ Ignoring agent recommendations
> /agent architecture-reviewer
"This code should be in core/, not commands/"
# User: "I'll leave it in commands/" (BAD!)

# ✅ Follow agent recommendations
> /agent architecture-reviewer
"This code should be in core/, not commands/"
# User: "You're right, let me move it" (GOOD!)
```

## 📚 Additional Resources

### Project Documentation
- `docs/architecture/ARCHITECTURE-REFERENCE.md` - High-level architecture
- `docs/architecture/layered/` - Layer-specific patterns
- `docs/architecture/examples/` - Code examples
- `docs/architecture/workflows/` - Development workflows

### Agent Configuration Files
- `rust-backend-agent.toml` - Backend agent config
- `frontend-agent.toml` - Frontend agent config
- `architecture-reviewer-agent.toml` - Reviewer agent config
- `database-agent.toml` - Database agent config
- `midi-hardware-agent.toml` - MIDI agent config

## 🎯 Quick Reference

### Agent Command Syntax
```bash
# Select agent
> /agent [agent-name]

# List available agents
> /agents

# Switch agent
> /agent [different-agent]

# Use general-purpose Claude
> /agent general-purpose
```

### Agent Selection Decision Tree
```
What are you working on?

├─ Backend Rust code?
│  ├─ MIDI processing? → midi-hardware
│  ├─ Database access? → database
│  └─ Other backend? → rust-backend
│
├─ Frontend code?
│  └─ → frontend
│
├─ Code review?
│  └─ → architecture-reviewer
│
└─ Not sure?
   └─ → architecture-reviewer (will guide you)
```

## 🔧 Customization

### Modifying Agents
Agents are defined in `.toml` files. You can customize:
- System prompts
- Tool availability
- Context files
- Model selection (sonnet/opus)

### Creating New Agents
```toml
name = "your-agent"
model = "sonnet"
description = "Your agent description"

[system_prompt]
content = """
Your specialized knowledge here...
"""

[tools]
enabled = ["read", "write", "search", "terminal"]

[context]
include = ["**/*.your-files"]
exclude = ["target/**"]
```

## 📞 Getting Help

### In Claude Code
```bash
# Get help with agents
> /help agents

# Ask architecture-reviewer for guidance
> /agent architecture-reviewer
"I'm new to the project. What should I know?"
```

### Team Communication
- Share agent configurations in `.claude/agents/`
- Document agent usage in team wiki
- Create team-specific agents for common patterns

---

**Remember**: The agents are here to help enforce best practices and maintain consistency. Use them liberally, especially the architecture-reviewer for validation!
