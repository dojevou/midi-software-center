# Claude Code Agent Setup for MIDI Software Center

## 🎯 Overview

This document provides a complete setup guide for Claude Code agents optimized for your MIDI software project using the Three Archetypes pattern.

## 📋 Agent Team Structure

### Core Development Team (6 Agents)

1. **Rust Backend Architect** - Pure logic and algorithms (Trusty Modules)
2. **Tauri Command Specialist** - I/O, commands, integration (Grown-up Scripts)
3. **Svelte Frontend Specialist** - UI components and stores
4. **Database Architect** - SQL, migrations, repositories
5. **Testing Specialist** - All test creation and coverage
6. **Code Reviewer** - Architecture enforcement and quality

## 🚀 Quick Start

### Step 1: Choose Agent Location

When Claude Code prompts you:
```
Choose location
1. Project (.claude/agents/)     ← Use this for project-specific
2. Personal (~/.claude/agents/)  ← Use this to share across projects
```

**Recommendation:** Use **Personal** location since these agents can be reused across all your projects.

### Step 2: Create Agents

For each agent, copy the corresponding `.md` file content into the agent configuration.

```bash
# Agent files created for you:
├── rust-backend-architect.md
├── tauri-command-specialist.md
├── svelte-frontend-specialist.md
├── database-architect.md
├── testing-specialist.md
└── code-reviewer.md
```

## 📁 Agent Configuration Details

### 1. Rust Backend Architect

**When to delegate:**
- Creating pure functions in `core/`
- Implementing MIDI parsing algorithms
- Writing BPM/key detection logic
- Creating data structure validation
- Writing 80%+ test coverage

**Model:** Sonnet 4.5 (for complex logic)

**Prompt prefix:**
```
You are the Rust Backend Architect. Focus on pure logic with no I/O.
All code you write must be in core/ directories and be 100% testable.
```

---

### 2. Tauri Command Specialist

**When to delegate:**
- Creating Tauri commands
- Writing database repositories
- Implementing MIDI hardware I/O
- Async operations with error handling
- Entry + Implementation patterns

**Model:** Sonnet 4.5

**Prompt prefix:**
```
You are the Tauri Command Specialist. Focus on I/O, async, and integration.
Always use Entry + Implementation pattern for Tauri commands.
```

---

### 3. Svelte Frontend Specialist

**When to delegate:**
- Building UI components
- Creating Svelte stores
- Implementing Tauri IPC calls
- Frontend state management
- Reactive UI patterns

**Model:** Sonnet 4.5

**Prompt prefix:**
```
You are the Svelte Frontend Specialist. Build reactive UIs with TypeScript.
Keep components under 300 lines and use stores for state management.
```

---

### 4. Database Architect

**When to delegate:**
- Writing SQL migrations
- Creating repository methods
- Designing database schemas
- Query optimization
- Database testing

**Model:** Sonnet 4.5

**Prompt prefix:**
```
You are the Database Architect. Design schemas and write type-safe queries.
All migrations must be reversible with UP and DOWN.
```

---

### 5. Testing Specialist

**When to delegate:**
- Writing unit tests
- Creating integration tests
- Setting up test fixtures
- Checking coverage
- Property-based testing

**Model:** Sonnet 4 (faster for test generation)

**Prompt prefix:**
```
You are the Testing Specialist. Ensure 80%+ coverage for Trusty Modules.
Write comprehensive tests for all new code.
```

---

### 6. Code Reviewer

**When to delegate:**
- Reviewing pull requests
- Checking archetype compliance
- Verifying error handling
- Enforcing test coverage
- Architecture validation

**Model:** Opus 4.1 (for thorough review)

**Prompt prefix:**
```
You are the Code Reviewer. Enforce the Three Archetypes pattern strictly.
Reject any .unwrap() in production code and ensure 80%+ coverage for core/.
```

## 🎨 Usage Patterns

### Pattern 1: New Feature Development

```
You: "Create a BPM detection algorithm"
Claude: Delegating to Rust Backend Architect...

Rust Backend Architect creates:
├── src-tauri/src/core/analysis/bpm_detector.rs (pure logic)
├── Tests with 80%+ coverage
└── Documentation

You: "Connect this to a Tauri command"
Claude: Delegating to Tauri Command Specialist...

Tauri Command Specialist creates:
├── src-tauri/src/commands/analysis.rs (entry point)
└── Integration with pure function
```

### Pattern 2: Database Feature

```
You: "Add MIDI analysis table"
Claude: Delegating to Database Architect...

Database Architect creates:
├── migrations/003_add_midi_analysis.sql
├── Repository in src/db/repositories/
└── Model in src/models/

You: "Add tests for the repository"
Claude: Delegating to Testing Specialist...

Testing Specialist creates:
└── Tests using sqlx::test
```

### Pattern 3: Frontend Feature

```
You: "Create a file browser component"
Claude: Delegating to Svelte Frontend Specialist...

Svelte Frontend Specialist creates:
├── src/lib/components/files/FileBrowser.svelte
├── src/lib/stores/fileStore.ts
└── src/lib/utils/fileHelpers.ts
```

### Pattern 4: Code Review

```
You: "Review this pull request"
Claude: Delegating to Code Reviewer...

Code Reviewer checks:
├── Archetype classification
├── Error handling patterns
├── Test coverage
├── Documentation
└── Provides detailed feedback
```

## 🔧 Project-Specific Configuration

### .claude/config.json (Project Root)

```json
{
  "agents": {
    "rust-backend": {
      "name": "Rust Backend Architect",
      "model": "claude-sonnet-4-5-20250929",
      "context": [
        "docs/architecture/layered/backend-architecture.md",
        "shared/rust/src/core/**/*.rs"
      ]
    },
    "tauri-commands": {
      "name": "Tauri Command Specialist",
      "model": "claude-sonnet-4-5-20250929",
      "context": [
        "src-tauri/src/commands/**/*.rs",
        "src-tauri/src/db/**/*.rs"
      ]
    },
    "svelte-frontend": {
      "name": "Svelte Frontend Specialist",
      "model": "claude-sonnet-4-5-20250929",
      "context": [
        "src/lib/**/*.svelte",
        "src/lib/stores/**/*.ts"
      ]
    },
    "database": {
      "name": "Database Architect",
      "model": "claude-sonnet-4-5-20250929",
      "context": [
        "database/migrations/**/*.sql",
        "src-tauri/src/db/**/*.rs"
      ]
    },
    "testing": {
      "name": "Testing Specialist",
      "model": "claude-sonnet-4-20250514",
      "context": [
        "tests/**/*.rs",
        "src/**/*.test.ts"
      ]
    },
    "reviewer": {
      "name": "Code Reviewer",
      "model": "claude-opus-4-1-20250514",
      "context": [
        "docs/architecture/**/*.md",
        ".cursor/rules/**/*.mdc"
      ]
    }
  }
}
```

### .vscode/settings.json

```json
{
  "claude.code.anthropic.include": [
    "**/*.rs",
    "**/*.svelte",
    "**/*.ts",
    "**/*.sql",
    "**/*.md"
  ],
  "claude.code.anthropic.exclude": [
    "target/**",
    "node_modules/**",
    "dist/**",
    ".git/**",
    "coverage/**"
  ],
  "claude.code.anthropic.agents.autoSelect": true,
  "claude.code.anthropic.agents.confirmDelegation": false
}
```

## 🎯 Decision Tree for Agent Selection

```
Need to write code?
├─ Pure logic, no I/O?
│  └─ Rust Backend Architect
├─ Database operation?
│  └─ Database Architect
├─ Tauri command or async I/O?
│  └─ Tauri Command Specialist
├─ Frontend UI or state?
│  └─ Svelte Frontend Specialist
├─ Tests needed?
│  └─ Testing Specialist
└─ Review existing code?
   └─ Code Reviewer
```

## 📊 Agent Responsibilities Matrix

| Task | Agent | Priority |
|------|-------|----------|
| MIDI parsing logic | Rust Backend | High |
| BPM detection algorithm | Rust Backend | High |
| Tauri commands | Tauri Command | High |
| Database queries | Database | High |
| SQL migrations | Database | High |
| UI components | Svelte Frontend | Medium |
| State management | Svelte Frontend | Medium |
| Unit tests | Testing | High |
| Integration tests | Testing | Medium |
| Code review | Code Reviewer | High |
| Architecture validation | Code Reviewer | High |

## 🚦 Quality Gates

Before merging any code, Code Reviewer checks:

1. ✅ **Archetype Classification** - Code in correct directory
2. ✅ **Error Handling** - No .unwrap()/.expect() in production
3. ✅ **Test Coverage** - 80%+ for Trusty Modules
4. ✅ **Pattern Compliance** - Entry + Implementation for commands
5. ✅ **Documentation** - Public items documented

## 🔄 Workflow Example

### Complete Feature: "Add MIDI file import with analysis"

```
Step 1: Design (You + Claude)
├─ Identify components needed
├─ Determine archetype for each
└─ Plan integration points

Step 2: Pure Logic (Rust Backend Architect)
├─ Create MIDI parser in core/midi/
├─ Write BPM detector in core/analysis/
├─ Write 80%+ test coverage
└─ Document algorithms

Step 3: Database (Database Architect)
├─ Create migration for midi_files table
├─ Create migration for midi_analysis table
├─ Write repository methods
└─ Add database tests

Step 4: Integration (Tauri Command Specialist)
├─ Create import_file command
├─ Create analyze_file command
├─ Connect to pure functions
└─ Handle errors properly

Step 5: Frontend (Svelte Frontend Specialist)
├─ Create ImportDialog component
├─ Create fileStore with actions
├─ Connect to Tauri commands
└─ Show progress/errors

Step 6: Testing (Testing Specialist)
├─ Add integration tests
├─ Test full import workflow
└─ Verify coverage meets 80%

Step 7: Review (Code Reviewer)
├─ Check archetype compliance
├─ Verify error handling
├─ Confirm test coverage
└─ Approve or request changes
```

## 💡 Best Practices

### 1. Always Start with Archetype Classification

Before writing any code, determine:
- Is this Task-O-Matic, Grown-up Script, or Trusty Module?
- Which agent should handle this?
- Where should the file be located?

### 2. Delegate Appropriately

Don't ask general-purpose Claude to write specialized code. Always delegate to the appropriate agent:

❌ Bad: "Claude, write me a MIDI parser"
✅ Good: "Delegate to Rust Backend Architect: Create MIDI parser"

### 3. Use Agent Context

Each agent has access to relevant documentation:
- Architecture patterns
- Code examples
- Best practices

### 4. Review Before Merge

Always run Code Reviewer on changes:
```
> delegate to Code Reviewer: Review this PR
```

### 5. Test Coverage is Mandatory

Testing Specialist enforces:
- 80%+ for Trusty Modules (core/)
- Tests written while coding, not after
- All error paths covered

## 🛠️ Tools Integration

### MCP Servers

```bash
# Install recommended MCP servers
npm install -g @modelcontextprotocol/server-rust
npm install -g @modelcontextprotocol/server-typescript
npm install -g @modelcontextprotocol/server-postgres
```

### VS Code Extensions

```json
{
  "recommendations": [
    "tauri-apps.tauri-vscode",
    "rust-lang.rust-analyzer",
    "svelte.svelte-vscode",
    "bradlc.vscode-tailwindcss"
  ]
}
```

## 📚 Additional Resources

- Architecture Reference: `docs/ARCHITECTURE-REFERENCE.md`
- Backend Patterns: `docs/architecture/layered/backend-architecture.md`
- Frontend Patterns: `docs/architecture/layered/frontend-architecture.md`
- Database Patterns: `docs/architecture/layered/database-architecture.md`

## 🎓 Training the Agents

When you first set up agents, give them context:

```
For each agent, run:
> Read docs/architecture/layered/[relevant-file].md
> Understand the Three Archetypes pattern
> Review existing code examples
```

This ensures agents understand your project structure before they start coding.

## ✅ Setup Checklist

- [ ] Created all 6 agents in Claude Code
- [ ] Configured .claude/config.json
- [ ] Updated .vscode/settings.json
- [ ] Reviewed agent documentation
- [ ] Tested delegation with simple task
- [ ] Ran Code Reviewer on existing code
- [ ] Verified Test Specialist can run coverage
- [ ] Confirmed agents can access project docs

## 🚀 You're Ready!

Your Claude Code agents are now configured to:
- Enforce the Three Archetypes pattern
- Maintain 80%+ test coverage
- Write production-ready code
- Follow project best practices
- Ensure architecture compliance

Start by delegating your next feature to the appropriate agent!
