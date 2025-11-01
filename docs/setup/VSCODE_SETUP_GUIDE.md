# 🚀 VS CODE SETUP GUIDE FOR MIDI SOFTWARE CENTER

**Project:** MIDI Software Center  
**Root:** `~/projects/midi-software-center`  
**OS:** Ubuntu Studio 25.04  
**Tools:** VS Code, Claude Code, Kilo Code

---

## ✅ PREREQUISITES (Install First)

### **1. Rust Toolchain**
```bash
# Check if already installed
rustc --version
cargo --version

# If not, install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify
rustc --version  # Should be 1.70+
cargo --version
```

### **2. Node.js & pnpm**
```bash
# Check if already installed
node --version    # Should be 18+
pnpm --version    # Should be 8+

# If not installed
curl -fsSL https://get.pnpm.io/install.sh | sh -

# Verify
node --version
pnpm --version
```

### **3. PostgreSQL & Docker**
```bash
# Check Docker
docker --version
docker-compose --version

# If not installed
sudo apt update
sudo apt install docker.io docker-compose
sudo usermod -aG docker $USER  # Add user to docker group
newgrp docker                  # Apply group without logout

# Verify
docker --version
docker-compose --version
```

### **4. Tauri Dependencies**
```bash
# Linux (Ubuntu) dependencies
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
  build-essential \
  curl \
  wget \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev

# Verify Tauri CLI
cargo install tauri-cli
```

---

## 📂 OPEN PROJECT IN VS CODE

### **Option 1: Command Line (Recommended)**
```bash
cd ~/projects/midi-software-center
code .
```

### **Option 2: VS Code UI**
- File → Open Folder
- Navigate to: `/home/YOUR_USERNAME/projects/midi-software-center`
- Click "Open"

### **Option 3: From File Manager**
- Open file manager
- Navigate to `~/projects/midi-software-center`
- Right-click → "Open with Code"

---

## 🧩 RECOMMENDED VS CODE EXTENSIONS

### **Essential Extensions (Install These First)**

#### **Rust Development**
1. **rust-analyzer** (by rust-lang)
   - ID: `rust-lang.rust-analyzer`
   - Press `Ctrl+P`, paste: `ext install rust-lang.rust-analyzer`
   - Best Rust IDE support

2. **CodeLLDB** (by Vadim Chugunov)
   - ID: `vadimchugunov.vscode-lldb`
   - Rust debugger
   - Press `Ctrl+P`, paste: `ext install vadimchugunov.vscode-lldb`

#### **TypeScript/Svelte Development**
3. **Svelte for VS Code** (by Svelte)
   - ID: `svelte.svelte-vscode`
   - Press `Ctrl+P`, paste: `ext install svelte.svelte-vscode`
   - Svelte syntax highlighting & intellisense

4. **TypeScript Vue Plugin** (by Vue)
   - ID: `Vue.volar`
   - Press `Ctrl+P`, paste: `ext install Vue.volar`
   - TypeScript support

#### **Database & Tools**
5. **SQLTools** (by Matheus Teixeira)
   - ID: `mtxr.sqltools`
   - Query your PostgreSQL database
   - Press `Ctrl+P`, paste: `ext install mtxr.sqltools`

6. **SQLTools PostgreSQL/MySQL Driver** (by Matheus Teixeira)
   - ID: `mtxr.sqltools-driver-pg`
   - PostgreSQL support for SQLTools
   - Press `Ctrl+P`, paste: `ext install mtxr.sqltools-driver-pg`

#### **Quality & Formatting**
7. **Prettier** (by Prettier)
   - ID: `esbenp.prettier-vscode`
   - Code formatter (TypeScript/Svelte)
   - Press `Ctrl+P`, paste: `ext install esbenp.prettier-vscode`

8. **ESLint** (by Microsoft)
   - ID: `dbaeumer.vscode-eslint`
   - Linting for JavaScript/TypeScript
   - Press `Ctrl+P`, paste: `ext install dbaeumer.vscode-eslint`

### **Optional But Useful Extensions**

9. **Thunder Client** (by Rangav)
   - ID: `rangav.vscode-thunder-client`
   - REST API client (like Postman)
   - Test your API endpoints

10. **Docker** (by Microsoft)
    - ID: `ms-azuretools.vscode-docker`
    - Docker container management
    - Press `Ctrl+P`, paste: `ext install ms-azuretools.vscode-docker`

11. **GitHub Copilot** (Optional, requires subscription)
    - ID: `GitHub.copilot`
    - AI code assistant

12. **Tauri** (by Tauri Team)
    - ID: `tauri-apps.tauri-vscode`
    - Tauri development support
    - Press `Ctrl+P`, paste: `ext install tauri-apps.tauri-vscode`

### **Quick Install All Essential Extensions**
```bash
# Copy-paste this into terminal
code --install-extension rust-lang.rust-analyzer
code --install-extension vadimchugunov.vscode-lldb
code --install-extension svelte.svelte-vscode
code --install-extension Vue.volar
code --install-extension mtxr.sqltools
code --install-extension mtxr.sqltools-driver-pg
code --install-extension esbenp.prettier-vscode
code --install-extension dbaeumer.vscode-eslint
```

---

## ⚙️ VS CODE SETTINGS CONFIGURATION

### **Create/Edit `.vscode/settings.json`**

Open VS Code Command Palette: `Ctrl+Shift+P`
Type: "Preferences: Open Workspace Settings (JSON)"

**Paste this configuration:**

```json
{
  // ========== RUST SETTINGS ==========
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.formatOnSave": true
  },
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.inlayHints.enable": true,
  "rust-analyzer.inlayHints.typeHints.enable": true,

  // ========== TYPESCRIPT/SVELTE SETTINGS ==========
  "[typescript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode",
    "editor.formatOnSave": true,
    "editor.codeActionsOnSave": {
      "source.fixAll.eslint": true
    }
  },
  "[svelte]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode",
    "editor.formatOnSave": true
  },
  "[html]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode",
    "editor.formatOnSave": true
  },
  "[css]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode",
    "editor.formatOnSave": true
  },

  // ========== SQL SETTINGS ==========
  "[sql]": {
    "editor.defaultFormatter": "mtxr.sqltools",
    "editor.formatOnSave": true
  },

  // ========== GENERAL EDITOR SETTINGS ==========
  "editor.rulers": [80, 120],
  "editor.wordWrap": "on",
  "editor.minimap.enabled": true,
  "editor.renderWhitespace": "selection",
  "editor.trimAutoWhitespace": true,
  "editor.insertSpaces": true,
  "editor.tabSize": 2,

  // ========== FILE SETTINGS ==========
  "files.exclude": {
    "**/.git": false,
    "**/node_modules": true,
    "**/target": true,
    "**/.svelte-kit": true,
    "**/.DS_Store": true
  },
  "search.exclude": {
    "**/node_modules": true,
    "**/target": true,
    "**/.svelte-kit": true
  },

  // ========== PRETTIER SETTINGS ==========
  "prettier.printWidth": 100,
  "prettier.tabWidth": 2,
  "prettier.useTabs": false,
  "prettier.semi": true,
  "prettier.singleQuote": true,
  "prettier.trailingComma": "es5",

  // ========== ESLINT SETTINGS ==========
  "eslint.enable": true,
  "eslint.lintTask.enable": true,

  // ========== TERMINAL SETTINGS ==========
  "terminal.integrated.defaultProfile.linux": "bash",
  "terminal.integrated.fontSize": 13,

  // ========== GIT SETTINGS ==========
  "git.ignoreLimitWarning": true,

  // ========== FILES TO ASSOCIATE ==========
  "files.associations": {
    "*.task": "bash",
    "*.conf": "ini"
  }
}
```

---

## 🚀 LAUNCH CONFIGURATIONS

### **Create `.vscode/launch.json`**

Open Command Palette: `Ctrl+Shift+P`
Type: "Debug: Add Configuration"

**Paste this:**

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Pipeline (Debug)",
      "type": "lldb",
      "request": "launch",
      "cargo": {
        "args": [
          "build",
          "--manifest-path=pipeline/src-tauri/Cargo.toml",
          "--message-format=json"
        ],
        "filter": {
          "name": "midi-library-pipeline",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}",
      "sourceLanguages": ["rust"]
    },
    {
      "name": "DAW (Debug)",
      "type": "lldb",
      "request": "launch",
      "cargo": {
        "args": [
          "build",
          "--manifest-path=daw/src-tauri/Cargo.toml",
          "--message-format=json"
        ],
        "filter": {
          "name": "midi-library-daw",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}",
      "sourceLanguages": ["rust"]
    }
  ]
}
```

---

## 📋 VS CODE TASKS

### **Create `.vscode/tasks.json`**

Open Command Palette: `Ctrl+Shift+P`
Type: "Tasks: Configure Task"

**Paste this:**

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Start Database",
      "type": "shell",
      "command": "make",
      "args": ["docker-up"],
      "group": {
        "kind": "build",
        "isDefault": false
      },
      "presentation": {
        "echo": true,
        "reveal": "always",
        "panel": "shared"
      }
    },
    {
      "label": "Stop Database",
      "type": "shell",
      "command": "make",
      "args": ["docker-down"],
      "group": {
        "kind": "build",
        "isDefault": false
      }
    },
    {
      "label": "Run Pipeline (Dev)",
      "type": "shell",
      "command": "make",
      "args": ["dev-pipeline"],
      "group": {
        "kind": "build",
        "isDefault": false
      },
      "isBackground": true,
      "problemMatcher": {
        "pattern": {
          "regexp": "^.*$",
          "file": 1,
          "location": 2,
          "message": 3
        },
        "background": {
          "activeOnStart": true,
          "beginsPattern": "^.*$",
          "endsPattern": "^.*ready.*$"
        }
      }
    },
    {
      "label": "Run DAW (Dev)",
      "type": "shell",
      "command": "make",
      "args": ["dev-daw"],
      "group": {
        "kind": "build",
        "isDefault": false
      },
      "isBackground": true
    },
    {
      "label": "Run Both (Dev)",
      "type": "shell",
      "command": "make",
      "args": ["dev-both"],
      "group": {
        "kind": "build",
        "isDefault": true
      },
      "isBackground": true
    },
    {
      "label": "Format Code",
      "type": "shell",
      "command": "make",
      "args": ["format"],
      "group": {
        "kind": "build",
        "isDefault": false
      }
    },
    {
      "label": "Run Tests",
      "type": "shell",
      "command": "make",
      "args": ["test"],
      "group": {
        "kind": "test",
        "isDefault": true
      }
    },
    {
      "label": "Build All",
      "type": "shell",
      "command": "make",
      "args": ["build-all"],
      "group": {
        "kind": "build",
        "isDefault": false
      }
    }
  ]
}
```

---

## 🎯 INITIAL SETUP STEPS

### **Step 1: Install Dependencies (First Time Only)**
```bash
cd ~/projects/midi-software-center

# Install all dependencies
make setup

# This will:
# - Build Rust workspace
# - Install Node dependencies for pipeline
# - Install Node dependencies for DAW
# - Takes 5-10 minutes first time

# Verify installation
rustc --version
node --version
pnpm --version
docker --version
```

### **Step 2: Start Database**
```bash
# Terminal in VS Code: Ctrl+`
# Run: make docker-up

# Or use Task:
# Ctrl+Shift+B → Select "Start Database"

# Verify
docker-compose ps
```

### **Step 3: Open Workspace**
```bash
# File → Open Workspace from File
# Select: midi-library-system.code-workspace

# Or command line:
code ~/projects/midi-software-center/midi-library-system.code-workspace
```

### **Step 4: Run Development Servers**

**Option A: Run Both at Once**
```bash
# Terminal 1
make dev-both

# This starts:
# - Pipeline at http://localhost:5173
# - DAW at http://localhost:5174
```

**Option B: Run Separately**
```bash
# Terminal 1 - Pipeline
make dev-pipeline

# Terminal 2 - DAW
make dev-daw
```

**Option C: Use VS Code Tasks**
```
Ctrl+Shift+B → Select "Run Both (Dev)"
```

---

## 🔄 COMMON VS CODE WORKFLOWS

### **Workflow 1: Development Cycle**

**Start of Day:**
```bash
# Terminal 1: Start database
make docker-up

# Wait for database to be ready
sleep 5

# Terminal 2: Start both apps
make dev-both

# Navigate to:
# http://localhost:5173 (Pipeline)
# http://localhost:5174 (DAW)
```

**During Development:**
- Make code changes (auto-saves)
- Applications hot-reload (Svelte)
- Check console for errors
- Debug as needed

**Format & Lint:**
```bash
# Before committing
make format
make lint
```

**End of Day:**
```bash
# Stop applications (Ctrl+C in terminals)
# Optionally stop database
make docker-down
```

### **Workflow 2: Debugging Rust**

**Set Breakpoint:**
1. Click in left margin of Rust file to add red dot
2. Press F5 or Click "Run" button
3. Select "Pipeline (Debug)" or "DAW (Debug)"
4. Execution will pause at breakpoint

**Debug Controls:**
- F10: Step over
- F11: Step into
- Shift+F11: Step out
- F5: Continue
- Shift+F5: Stop debugging

### **Workflow 3: Testing**

**Run All Tests:**
```bash
make test

# Or in VS Code Task:
Ctrl+Shift+B → Select "Run Tests"
```

**Run Specific Tests:**
```bash
# Rust tests
cd pipeline/src-tauri
cargo test --lib

# Frontend tests
cd pipeline
pnpm test
```

### **Workflow 4: Building for Production**

```bash
# Build both applications
make build-all

# Binaries will be in:
# pipeline/src-tauri/target/release/bundle/
# daw/src-tauri/target/release/bundle/
```

---

## 🐛 TROUBLESHOOTING VS CODE

### **Problem: Rust Analyzer Not Working**

**Solution:**
```bash
# 1. Reload window
Ctrl+Shift+P → "Developer: Reload Window"

# 2. Check rust-analyzer status
Ctrl+Shift+P → "Rust Analyzer: Status"

# 3. Restart extension
Ctrl+Shift+P → "Developer: Restart Extension Host"

# 4. Check Rust installation
rustc --version
cargo --version
```

### **Problem: Port Already in Use (5173/5174)**

**Solution:**
```bash
# Find process using port
lsof -i :5173
lsof -i :5174

# Kill process
kill -9 <PID>

# Or change port in vite.config.ts
```

### **Problem: Database Connection Failed**

**Solution:**
```bash
# Check if Docker is running
docker ps

# Start database
make docker-up

# Check logs
docker-compose logs postgres

# Reset database
make db-reset
```

### **Problem: Module Not Found Errors**

**Solution:**
```bash
# Reinstall dependencies
rm -rf node_modules pnpm-lock.yaml
pnpm install

# Clear Rust cache
cargo clean
cargo build
```

### **Problem: Svelte Components Not Recognized**

**Solution:**
```bash
# Restart VS Code
Ctrl+Shift+P → "Developer: Reload Window"

# Or reinstall Svelte extension
code --install-extension svelte.svelte-vscode --force
```

---

## 🎯 KEYBOARD SHORTCUTS REFERENCE

### **Essential Shortcuts**

```
Ctrl+`           Open/close terminal
Ctrl+Shift+P     Command palette
Ctrl+/           Comment/uncomment line
Ctrl+Shift+F     Format document
Ctrl+Shift+B     Run build task
F5               Start/continue debugging
F10              Step over (debug)
F11              Step into (debug)
Shift+F5         Stop debugging
Ctrl+Shift+D     Debug panel
Ctrl+Shift+U     Output panel
Ctrl+J           Toggle panel
Ctrl+B           Toggle sidebar
Ctrl+L           Select line
Ctrl+H           Find and replace
Ctrl+G           Go to line
Alt+Up/Down      Move line up/down
Shift+Alt+Up/Down Copy line up/down
```

---

## 🔗 USEFUL RESOURCES

### **In-Project Resources**
- [QUICK_REFERENCE.md](../planning/QUICK_REFERENCE.md) - Daily operations
- [README.md](../../README.md) - Project overview
- [SETUP-INSTRUCTIONS.md](./SETUP-INSTRUCTIONS.md) - Initial setup
- [DEVELOPMENT-WORKFLOW.md](../../DEVELOPMENT-WORKFLOW.md) - Dev guide

### **External Resources**
- [VS Code Docs](https://code.visualstudio.com/docs)
- [Rust Analyzer Docs](https://rust-analyzer.github.io/)
- [Svelte Docs](https://svelte.dev/)
- [Tauri Docs](https://tauri.app/)
- [TypeScript Docs](https://www.typescriptlang.org/)

---

## ✅ SETUP VERIFICATION CHECKLIST

After setup, verify everything works:

```bash
# ✅ Rust toolchain
rustc --version    # Should output version

# ✅ Node/pnpm
node --version     # Should be 18+
pnpm --version     # Should be 8+

# ✅ Docker
docker --version
docker-compose ps  # Should show containers

# ✅ VS Code Extensions
# Check sidebar: Extensions (Ctrl+Shift+X)
# Verify installed: rust-analyzer, svelte-vscode, etc.

# ✅ Project dependencies
cd ~/projects/midi-software-center
make setup         # Should complete without errors

# ✅ Database
make docker-up
docker-compose ps  # postgres and meilisearch should be up

# ✅ Applications
make dev-both      # Should start without errors
# Check browser:
# - http://localhost:5173 (Pipeline)
# - http://localhost:5174 (DAW)
```

---

## 🚀 FIRST TIME RUNNING

### **Complete First-Run Sequence**

```bash
# 1. Navigate to project
cd ~/projects/midi-software-center

# 2. Install dependencies (one time)
make setup
# Takes 5-10 minutes...

# 3. Open in VS Code
code .

# 4. In VS Code terminal (Ctrl+`)
# Start database
make docker-up

# 5. Wait 5 seconds for database to be ready
sleep 5

# 6. In another terminal
# Start both applications
make dev-both

# 7. Applications are now running!
# Open browser:
# - Pipeline: http://localhost:5173
# - DAW: http://localhost:5174

# 8. Make some code changes and watch hot-reload!

# 9. When done, stop with Ctrl+C
```

**Expected Output:**
```
✓ Pipeline running on http://localhost:5173
✓ DAW running on http://localhost:5174
✓ Database connected
✓ Ready for development!
```

---

## 💡 PRO TIPS

✅ **Use terminal inside VS Code** (Ctrl+`)
- Keeps everything in one window
- Multiple tabs for different tasks

✅ **Save workspace configuration**
- VS Code remembers your layout
- Settings persist across sessions

✅ **Use tasks for common commands**
- Ctrl+Shift+B to run tasks quickly
- Less typing, faster workflow

✅ **Enable autosave**
- File → Auto Save
- Never lose unsaved work

✅ **Use git integration**
- Left sidebar: Source Control (Ctrl+Shift+G)
- Stage, commit, push without terminal

✅ **Format on save**
- Automatic code formatting
- Consistent code style

✅ **Use debugging instead of console.log**
- Set breakpoints
- Inspect variables
- Much faster debugging

---

**Status:** Ready to develop! 🎉  
**Setup Time:** ~30 minutes (first time)  
**Next Time:** 2-3 minutes to start

Happy coding! 🚀

