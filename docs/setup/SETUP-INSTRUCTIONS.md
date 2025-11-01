# Practical Claude Code Setup Instructions

## 📋 Pre-Setup Checklist

Before creating agents, ensure you have:
- [ ] Claude Code installed and working
- [ ] Project structure in place
- [ ] Architecture documentation accessible
- [ ] Ubuntu Studio 25.04 environment ready

## 🚀 Step-by-Step Agent Creation

### Method 1: Via Claude Code CLI

```bash
# Navigate to your project
cd ~/midi-software-center

# Launch Claude Code
claude-code

# In Claude Code, type:
/agents
```

### Method 2: Manual Creation

Each agent needs:
1. **Name** - Descriptive name
2. **System Prompt** - Agent's role and rules (from the .md files)
3. **Model** - Claude model to use
4. **Context** - Files the agent can access

## 🎯 Creating Each Agent

### 1. Rust Backend Architect

```
In Claude Code:
> /agents
> Create new agent
> Name: rust-backend-architect
> Model: claude-sonnet-4-5-20250929
> Location: Personal (~/.claude/agents/)

System Prompt:
```
Paste the entire content from `rust-backend-architect.md`

**Context Files to Include:**
```
docs/architecture/layered/backend-architecture.md
shared/rust/src/core/**/*.rs
src-tauri/src/core/**/*.rs
.cursor/rules/rust-rules.mdc
```

---

### 2. Tauri Command Specialist

```
In Claude Code:
> /agents  
> Create new agent
> Name: tauri-commands
> Model: claude-sonnet-4-5-20250929
> Location: Personal
```

Paste content from `tauri-command-specialist.md`

**Context Files:**
```
src-tauri/src/commands/**/*.rs
src-tauri/src/services/**/*.rs
src-tauri/src/db/repositories/**/*.rs
docs/architecture/layered/backend-architecture.md
```

---

### 3. Svelte Frontend Specialist

```
In Claude Code:
> /agents
> Create new agent
> Name: svelte-frontend
> Model: claude-sonnet-4-5-20250929
> Location: Personal
```

Paste content from `svelte-frontend-specialist.md`

**Context Files:**
```
src/lib/components/**/*.svelte
src/lib/stores/**/*.ts
src/lib/utils/**/*.ts
docs/architecture/layered/frontend-architecture.md
.cursor/rules/svelte-rules.mdc
```

---

### 4. Database Architect

```
In Claude Code:
> /agents
> Create new agent
> Name: database
> Model: claude-sonnet-4-5-20250929
> Location: Personal
```

Paste content from `database-architect.md`

**Context Files:**
```
database/migrations/**/*.sql
src-tauri/src/db/**/*.rs
src-tauri/src/models/**/*.rs
docs/architecture/layered/database-architecture.md
```

---

### 5. Testing Specialist

```
In Claude Code:
> /agents
> Create new agent
> Name: testing
> Model: claude-sonnet-4-20250514
> Location: Personal
```

Paste content from `testing-specialist.md`

**Context Files:**
```
tests/**/*.rs
src/**/*.test.ts
docs/architecture/workflows/testing-workflow.md
```

---

### 6. Code Reviewer

```
In Claude Code:
> /agents
> Create new agent
> Name: reviewer
> Model: claude-opus-4-1-20250514
> Location: Personal
```

Paste content from `code-reviewer.md`

**Context Files:**
```
docs/ARCHITECTURE-REFERENCE.md
docs/architecture/layered/**/*.md
.cursor/rules/**/*.mdc
```

## 🧪 Testing Your Agents

After creating all agents, test each one:

### Test 1: Rust Backend Architect
```
In Claude Code:
> @rust-backend-architect Create a simple function to validate MIDI note numbers (0-127) with tests

Expected output:
- Pure function in a code block
- No I/O operations
- Comprehensive tests
- 80%+ coverage
```

### Test 2: Tauri Command Specialist
```
> @tauri-commands Create a Tauri command to get all files in a workspace

Expected output:
- #[tauri::command] entry point
- Separate implementation function
- Error handling with String return type
- Testable implementation
```

### Test 3: Svelte Frontend Specialist
```
> @svelte-frontend Create a simple loading spinner component

Expected output:
- .svelte file with TypeScript
- Proper structure (script, template, style)
- Props typed correctly
```

### Test 4: Database Architect
```
> @database Create a migration for a simple users table

Expected output:
- UP and DOWN sections
- UUID primary key
- Timestamps
- Indexes
- Triggers
```

### Test 5: Testing Specialist
```
> @testing Write tests for a function that calculates BPM

Expected output:
- Unit tests in #[cfg(test)]
- Multiple test cases
- Edge cases covered
- Property-based tests if applicable
```

### Test 6: Code Reviewer
```
> @reviewer Review this code: [paste code with .unwrap()]

Expected output:
- Identifies .unwrap() issue
- Suggests proper error handling
- References documentation
- Clear action items
```

## 📁 Project Structure Verification

Ensure these directories exist:

```bash
mkdir -p .claude/agents
mkdir -p docs/architecture/layered
mkdir -p docs/architecture/examples
mkdir -p docs/architecture/workflows
mkdir -p .cursor/rules
```

## 📝 Configuration Files

### Create .claude/config.json

```bash
cat > .claude/config.json << 'EOF'
{
  "agents": {
    "autoSelect": true,
    "confirmDelegation": false,
    "defaultContext": [
      "docs/**/*.md",
      ".cursor/rules/**/*.mdc"
    ]
  },
  "files": {
    "include": [
      "**/*.rs",
      "**/*.svelte", 
      "**/*.ts",
      "**/*.sql",
      "**/*.md"
    ],
    "exclude": [
      "target/**",
      "node_modules/**",
      "dist/**",
      ".git/**",
      "coverage/**"
    ]
  }
}
EOF
```

### Update .vscode/settings.json

Add to your existing settings:

```json
{
  "claude.code.agents.enabled": true,
  "claude.code.agents.autoSelect": true,
  "claude.code.agents.showDelegation": true,
  "claude.code.context.maxTokens": 100000,
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
    ".git/**"
  ]
}
```

## 🎯 Usage Examples

### Example 1: Full Feature Implementation

```
You: I need to add MIDI file analysis with BPM and key detection

Claude Code will:
1. @rust-backend-architect
   └─> Create pure BPM/key detection algorithms in core/
   
2. @database  
   └─> Create midi_analysis table migration
   
3. @tauri-commands
   └─> Create analyze_file command
   
4. @svelte-frontend
   └─> Create analysis display component
   
5. @testing
   └─> Add comprehensive tests
   
6. @reviewer
   └─> Review all changes
```

### Example 2: Bug Fix

```
You: There's a panic in the MIDI parser, need to fix

Claude Code:
1. @reviewer - Identifies the .unwrap() causing panic
2. @rust-backend-architect - Fixes with proper error handling
3. @testing - Adds regression test
4. @reviewer - Verifies fix meets standards
```

### Example 3: Refactoring

```
You: This component is too large, needs splitting

Claude Code:
1. @reviewer - Analyzes current structure
2. @svelte-frontend - Splits into smaller components
3. @testing - Updates tests
4. @reviewer - Validates architecture compliance
```

## 🔧 Troubleshooting

### Agent Not Responding
```bash
# Check agent exists
claude-code agents list

# Verify context files exist
ls -la docs/architecture/

# Restart Claude Code
claude-code restart
```

### Agent Using Wrong Pattern
```
Solution: Update agent's system prompt with more specific examples
Location: ~/.claude/agents/[agent-name]/config.json
```

### Agent Can't Access Files
```
Solution: Check context paths in agent configuration
Ensure files are not in excluded directories
```

## 📊 Verification Checklist

After setup, verify:

- [ ] All 6 agents created
- [ ] Each agent tested with simple task
- [ ] Agents can delegate to each other
- [ ] Code Reviewer catches .unwrap()
- [ ] Testing Specialist checks coverage
- [ ] Project context files accessible
- [ ] Configuration files in place

## 🎓 Next Steps

1. **Read the Documentation**
   ```
   Start with: docs/ARCHITECTURE-REFERENCE.md
   Then: docs/architecture/layered/
   ```

2. **Try a Simple Feature**
   ```
   Example: Add a new MIDI file property
   Practice delegation to appropriate agents
   ```

3. **Review Existing Code**
   ```
   Run @reviewer on current codebase
   Identify areas needing refactoring
   ```

4. **Establish Workflow**
   ```
   Define when to use each agent
   Set up pre-commit hooks
   Configure CI/CD with agents
   ```

## 🚦 Quality Gates

Before any merge:

```bash
# 1. Run tests
cargo test --workspace
npm test

# 2. Check coverage
cargo tarpaulin --out Html

# 3. Review with agent
# In Claude Code:
> @reviewer Review changes in [files]

# 4. Verify no .unwrap()
rg "\.unwrap\(\)" --type rust src-tauri/src

# 5. Check clippy
cargo clippy -- -D warnings
```

## 💡 Pro Tips

1. **Use @agent prefix** for explicit delegation
2. **Let agents work together** - they can delegate to each other
3. **Review agent suggestions** - agents learn from feedback
4. **Keep agents updated** - update system prompts as patterns evolve
5. **Monitor token usage** - Opus for review, Sonnet for most work

## 📞 Getting Help

```
Architecture questions:
> @reviewer Explain the Three Archetypes pattern

Code examples:
> @[specialist-agent] Show me an example of [pattern]

Testing help:
> @testing What should I test for [function]?

General questions:
> Ask general-purpose Claude first, then delegate
```

## ✅ Success Criteria

Your setup is complete when:
- [ ] All agents respond correctly to test queries
- [ ] Agents enforce architecture patterns
- [ ] Code Reviewer catches common mistakes
- [ ] Testing Specialist achieves 80%+ coverage
- [ ] You can complete a full feature using delegation
- [ ] Team understands agent usage patterns

---

## 🎉 You're Ready to Build!

Your Claude Code agents are configured to maintain:
- ✅ Clean architecture (Three Archetypes)
- ✅ High test coverage (80%+ for core)
- ✅ Production-ready code (no .unwrap())
- ✅ Best practices enforcement

**Start building with confidence!**
