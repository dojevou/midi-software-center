# 🚀 QUICK START: SETUP-PROJECT-STRUCTURE.SH

**Script:** `setup-project-structure.sh`  
**Purpose:** Create complete project folder structure  
**Time:** ~10 seconds  
**Platform:** Linux/macOS (tested on Ubuntu 25.04)

---

## ⚡ QUICKEST START

```bash
# Download the script
cd ~/projects/midi-software-center

# Run it (creates all folders)
bash setup-project-structure.sh

# Done! Your structure is created
```

---

## 📋 WHAT THE SCRIPT DOES

✅ Creates 20+ organized directories  
✅ Creates .gitignore files where needed  
✅ Creates README.md files in each folder  
✅ Validates the structure  
✅ Shows colored progress output  
✅ Takes ~10 seconds  

**Folders Created:**
```
config/              → Centralized configuration
docs/                → Documentation (api, architecture, database, etc.)
database/            → Database setup and migrations
scripts/             → Automation hub (modules, tasks, launch, grown-up, etc.)
shared/              → Shared code (rust, ui, types)
pipeline/            → Batch processor app
daw/                 → Audio workstation app
infrastructure/      → DevOps (docker, kubernetes, github, nginx)
tests/               → Testing (integration, e2e, fixtures)
backups/             → Backup storage
.vscode/             → VS Code configuration
```

---

## 🎯 THREE WAYS TO RUN

### **1. Normal Setup (Recommended)**
```bash
cd ~/projects/midi-software-center
bash setup-project-structure.sh
```

**Output:**
```
=================================================================================
🏗️  MIDI SOFTWARE CENTER - PROJECT STRUCTURE SETUP
=================================================================================

✓ Project root exists: /home/user/projects/midi-software-center
ℹ Working directory: /home/user/projects/midi-software-center

▶ Creating Configuration Directory
✓ Centralized project configuration: config
✓ Placeholder: config/.keep

▶ Creating Documentation Directory
✓ Consolidated documentation: docs
✓ API documentation: docs/api
✓ Architecture documentation: docs/architecture
✓ Database documentation: docs/database
✓ How-to guides: docs/guides
✓ Common workflows: docs/workflows

[... more output ...]

▶ Setup Complete!

✓ Project structure created successfully

ℹ Project Root: /home/user/projects/midi-software-center

ℹ Next Steps:
  1. cd /home/user/projects/midi-software-center
  2. code .                    (Open in VS Code)
  3. make setup                (Install dependencies)
  4. make docker-up            (Start database)
  5. make dev-both             (Start applications)

ℹ Documentation:
  • Read: FILE_PLACEMENT_GUIDE.md
  • Read: RECOMMENDED_PROJECT_STRUCTURE.md
  • Read: QUICK_REFERENCE.md
```

### **2. Dry-Run Mode (Preview)**
```bash
bash setup-project-structure.sh --dry-run
```

**Purpose:** See what WOULD be created without actually creating anything  
**Use When:** You want to preview or test before running  
**Output:** Shows all actions with ✓ but makes no changes  

```bash
# Example with dry-run
▶ Creating Configuration Directory
→ Would create: config (Centralized project configuration)
→ Would create: config/.keep (Placeholder)
...

⚠ Dry run completed - no changes were made

To actually create the structure, run without --dry-run:
  bash setup-project-structure.sh
```

### **3. Verbose Mode (Detailed)**
```bash
bash setup-project-structure.sh --verbose
```

**Purpose:** Show detailed output for every action  
**Use When:** Debugging or you want to see everything  
**Output:** Shows every file and folder being created  

```bash
# Example with verbose
→ Creating directory: config
→ Creating directory: config/.keep
→ Creating directory: docs
→ Creating directory: docs/api
→ Creating directory: docs/architecture
...
```

---

## 🔧 MANUAL INSTALLATION

If you prefer to run it from anywhere or download it differently:

```bash
# 1. Save the script
curl -O https://your-repo/setup-project-structure.sh
chmod +x setup-project-structure.sh

# 2. Run it
./setup-project-structure.sh

# 3. That's it!
```

---

## 📊 BEFORE & AFTER

### **Before Running Script**
```
~/projects/midi-software-center/
├── pipeline/              (existing)
├── daw/                   (existing)
├── database/              (existing)
├── scripts/               (existing, maybe messy)
├── Makefile               (existing)
└── ... (scattered files)
```

### **After Running Script**
```
~/projects/midi-software-center/
├── 📄 Root files (README, Makefile, .env.example)
├── ⚙️  config/
│   └── .keep
├── 📚 docs/
│   ├── api/
│   ├── architecture/
│   ├── database/
│   ├── guides/
│   ├── workflows/
│   └── README.md
├── 🗄️ database/
│   ├── migrations/
│   ├── queries/
│   ├── seeds/
│   ├── scripts/
│   ├── config/
│   └── README.md
├── 🔧 scripts/
│   ├── modules/
│   ├── tasks/
│   │   ├── db/
│   │   ├── build/
│   │   ├── deploy/
│   │   ├── dev/
│   │   └── test/
│   ├── launch/
│   ├── grown-up/
│   ├── maintenance/
│   ├── legacy/
│   └── README.md
├── 🗂️ shared/
│   ├── rust/
│   ├── ui/
│   ├── types/
│   └── README.md
├── 🚀 pipeline/
│   ├── tests/
│   ├── docs/
│   └── (existing code)
├── 🎹 daw/
│   ├── tests/
│   ├── docs/
│   └── (existing code)
├── ⚡ infrastructure/
│   ├── docker/
│   ├── kubernetes/
│   ├── github/
│   │   └── workflows/
│   └── nginx/
├── 📊 tests/
│   ├── integration/
│   ├── e2e/
│   └── fixtures/
│       └── midi-files/
├── 🔒 backups/
│   └── .gitignore
└── .vscode/
    └── (VS Code settings)
```

---

## ✅ VERIFICATION

After running the script, verify it worked:

```bash
# Check main directories exist
ls -la ~/projects/midi-software-center/ | grep "^d"

# Should show:
# drwxr-xr-x config
# drwxr-xr-x docs
# drwxr-xr-x database
# drwxr-xr-x scripts
# drwxr-xr-x shared
# drwxr-xr-x pipeline
# drwxr-xr-x daw
# drwxr-xr-x infrastructure
# drwxr-xr-x tests
# drwxr-xr-x backups
# drwxr-xr-x .vscode

# Count total directories
find ~/projects/midi-software-center -type d | wc -l

# Should be around 50+ directories
```

---

## 🐛 TROUBLESHOOTING

### **Problem: "Permission denied" when running script**

**Solution:**
```bash
# Make script executable
chmod +x setup-project-structure.sh

# Then run
./setup-project-structure.sh
```

### **Problem: "No such file or directory" for project root**

**Solution:**
```bash
# Create project root first
mkdir -p ~/projects/midi-software-center

# Then run script
cd ~/projects/midi-software-center
bash setup-project-structure.sh
```

### **Problem: "bad variable name" or syntax errors**

**Solution:**
```bash
# Run with bash explicitly (not sh)
bash setup-project-structure.sh

# NOT: sh setup-project-structure.sh
```

### **Problem: Script says folders already exist**

**Solution:**
```bash
# This is fine! Script is smart:
# - Won't delete existing folders
# - Will skip if already created
# - Safe to run multiple times
```

---

## 🎯 COMMON QUESTIONS

### **Q: Is it safe to run multiple times?**
**A:** Yes! The script is idempotent - it's safe to run multiple times. It won't delete or overwrite existing files.

### **Q: Can I run it from anywhere?**
**A:** Yes, the script automatically detects the project root. Just make sure it's in your project directory.

### **Q: Will it delete my existing code?**
**A:** No! The script only creates folders and README files. It won't touch your existing code in `pipeline/`, `daw/`, etc.

### **Q: How long does it take?**
**A:** About 5-10 seconds on most systems.

### **Q: Do I need internet?**
**A:** No, the script runs completely offline.

### **Q: What if I mess up?**
**A:** No problem! You can just delete the folders and run it again. Or use `--dry-run` first to preview.

---

## 📝 WHAT'S IN THE .gitignore FILES

### **backups/.gitignore**
```
# Ignore all backup files
*
!.gitignore
!README.md
!README
```
Only keeps documentation, ignores actual backups.

---

## 📚 WHAT'S IN THE README.md FILES

Each folder gets a README.md with:
- 📖 Description of the folder
- 📁 Structure and organization
- 🚀 Getting started instructions
- 📝 Common commands

Example (config/README.md):
```markdown
# 📚 Configuration

Central location for all project configuration files.

## Files
- defaults.conf - Default settings for all environments
- development.conf - Development environment overrides
- production.conf - Production environment overrides
- testing.conf - Testing environment overrides
- load-config.sh - Configuration loader script

## Usage
Source the configuration loader in scripts:
```bash
source config/load-config.sh
```
```

---

## 🚀 NEXT STEPS AFTER SCRIPT COMPLETES

```bash
# 1. Verify structure was created
cd ~/projects/midi-software-center
ls -la

# 2. Open in VS Code
code .

# 3. Read the documentation
cat docs/README.md
cat config/README.md
cat scripts/README.md

# 4. Install dependencies
make setup

# 5. Start development
make docker-up
make dev-both
```

---

## 💡 PRO TIPS

✅ **Run dry-run first** to see what will happen
```bash
bash setup-project-structure.sh --dry-run
```

✅ **Use verbose mode** to debug if something goes wrong
```bash
bash setup-project-structure.sh --verbose
```

✅ **Redirect output** to a file for record-keeping
```bash
bash setup-project-structure.sh > setup.log 2>&1
```

✅ **Run it in VS Code terminal** to stay in one window
```
Ctrl+` (backtick) to open terminal in VS Code
Then: bash setup-project-structure.sh
```

---

## 📞 SUPPORT

If something goes wrong:

1. **Check the error message** - it usually tells you what's wrong
2. **Try verbose mode** - `bash setup-project-structure.sh --verbose`
3. **Try dry-run first** - `bash setup-project-structure.sh --dry-run`
4. **Check permissions** - `chmod +x setup-project-structure.sh`
5. **Use bash not sh** - `bash setup-project-structure.sh` (not `sh`)

---

## 🎉 YOU'RE ALL SET!

Your project structure is now organized and ready for development!

```
✓ Folders created
✓ Documentation ready
✓ .gitignore configured
✓ README.md files added
✓ Structure validated

Ready to start coding! 🚀
```

---

**Script Location:** `/mnt/user-data/outputs/setup-project-structure.sh`  
**Copy To:** `~/projects/midi-software-center/setup-project-structure.sh`  
**Run:** `bash setup-project-structure.sh`  
**Time:** ~10 seconds  
**Difficulty:** ⭐ (Very Easy)

