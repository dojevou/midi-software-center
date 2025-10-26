# 🎯 SCRIPTS QUICK REFERENCE GUIDE

**Purpose:** Complete guide to all setup and project initialization scripts  
**Status:** Ready to use  
**Platform:** Linux/macOS

---

## 📦 AVAILABLE SCRIPTS

You now have **2 production-ready scripts** to set up your project:

### **Script 1: setup-project-structure.sh** ⭐ MAIN SETUP

**Purpose:** Create complete folder hierarchy  
**Size:** ~12 KB  
**Time:** ~10 seconds  
**Must Run:** YES (Foundation for everything)

```bash
cd ~/projects/midi-software-center
bash setup-project-structure.sh
```

**What it creates:**
- 50+ directories (organized by function)
- .gitignore files (prevents committing backup data)
- README.md files (documentation in each folder)
- Validates the structure

**Options:**
```bash
bash setup-project-structure.sh              # Normal setup
bash setup-project-structure.sh --dry-run    # Preview (no changes)
bash setup-project-structure.sh --verbose    # Detailed output
bash setup-project-structure.sh --help       # Show help
```

**Output:**
```
✓ Project structure created successfully
✓ 50+ directories created
✓ .gitignore files configured
✓ README.md files added
✓ Structure validated
```

---

### **Script 2: create-structure.sh** (Basic Version)

**Purpose:** Simple folder creation (lightweight version)  
**Size:** ~2 KB  
**Time:** ~5 seconds  
**Must Run:** NO (setup-project-structure.sh is better)

```bash
bash create-structure.sh
```

**What it does:**
- Creates main directories only
- No validation
- No README files
- No .gitignore

**When to use:**
- If you want minimal setup
- If you only need basic folder structure
- Quick preview before full setup

---

## 🚀 RECOMMENDED SETUP SEQUENCE

### **STEP 1: Create Project Root Directory**

```bash
# Create the root directory
mkdir -p ~/projects/midi-software-center

# Navigate to it
cd ~/projects/midi-software-center

# Verify
pwd
# Should print: /home/YOUR_USERNAME/projects/midi-software-center
```

### **STEP 2: Copy Setup Scripts**

Download the scripts from `/mnt/user-data/outputs/`:

```bash
# Option A: Download specific script
cp /mnt/user-data/outputs/setup-project-structure.sh ~/projects/midi-software-center/

# Option B: Download both scripts
cp /mnt/user-data/outputs/setup-project-structure.sh ~/projects/midi-software-center/
cp /mnt/user-data/outputs/create-structure.sh ~/projects/midi-software-center/

# Make executable
chmod +x ~/projects/midi-software-center/*.sh
```

### **STEP 3: Run Main Setup Script**

```bash
cd ~/projects/midi-software-center

# Preview first (recommended)
bash setup-project-structure.sh --dry-run

# Then run for real
bash setup-project-structure.sh

# Or verbose output
bash setup-project-structure.sh --verbose
```

**Expected output:**
```
=================================================================================
🏗️  MIDI SOFTWARE CENTER - PROJECT STRUCTURE SETUP
=================================================================================

✓ Project root exists: /home/user/projects/midi-software-center
ℹ Working directory: /home/user/projects/midi-software-center

▶ Creating Configuration Directory
✓ Centralized project configuration: config
...
[50+ more lines]
...

▶ Setup Complete!

✓ Project structure created successfully
```

### **STEP 4: Verify Structure**

```bash
# List main directories
ls -la ~/projects/midi-software-center/ | grep "^d"

# Count total directories
find ~/projects/midi-software-center -type d | wc -l
# Should show: ~50 directories

# Check a specific folder
ls ~/projects/midi-software-center/scripts/
# Should show: modules, tasks, launch, grown-up, maintenance, legacy
```

### **STEP 5: Open in VS Code**

```bash
cd ~/projects/midi-software-center
code .
```

---

## 📋 COMPLETE INITIALIZATION CHECKLIST

**Copy this and check off as you complete:**

```bash
☐ Step 1: Create project root
  mkdir -p ~/projects/midi-software-center
  cd ~/projects/midi-software-center

☐ Step 2: Copy setup scripts
  cp /mnt/user-data/outputs/setup-project-structure.sh .

☐ Step 3: Preview with dry-run
  bash setup-project-structure.sh --dry-run

☐ Step 4: Run setup script
  bash setup-project-structure.sh

☐ Step 5: Verify structure
  ls -la | grep "^d"
  find . -type d | wc -l

☐ Step 6: Open in VS Code
  code .

☐ Step 7: Read documentation
  cat config/README.md
  cat scripts/README.md
  cat docs/README.md

☐ Step 8: Install dependencies
  make setup

☐ Step 9: Start database
  make docker-up

☐ Step 10: Start applications
  make dev-both
```

---

## 🎯 QUICK COMMANDS

### **Most Common**
```bash
# Full setup (everything)
cd ~/projects/midi-software-center
bash setup-project-structure.sh

# Verify it worked
find . -type d | wc -l

# Open in code
code .
```

### **Preview First (Safe)**
```bash
cd ~/projects/midi-software-center
bash setup-project-structure.sh --dry-run
```

### **Detailed Output**
```bash
bash setup-project-structure.sh --verbose
```

### **Save to Log File**
```bash
bash setup-project-structure.sh > setup.log 2>&1
cat setup.log
```

---

## 🗂️ WHAT EACH SCRIPT CREATES

### **setup-project-structure.sh Creates:**

```
✓ Root level
├── Makefile
├── Cargo.toml
├── package.json
└── README.md

✓ config/
├── .keep
└── README.md

✓ docs/
├── api/
├── architecture/
├── database/
├── guides/
├── workflows/
└── README.md

✓ database/
├── migrations/
├── queries/
├── seeds/
├── scripts/
├── config/
└── README.md

✓ scripts/
├── modules/
├── tasks/
│   ├── db/
│   ├── build/
│   ├── deploy/
│   ├── dev/
│   └── test/
├── launch/
├── grown-up/
├── maintenance/
├── legacy/
└── README.md

✓ shared/
├── rust/
├── ui/
├── types/
└── README.md

✓ pipeline/
├── tests/
├── docs/
└── README.md

✓ daw/
├── tests/
├── docs/
└── README.md

✓ infrastructure/
├── docker/
├── kubernetes/
├── github/
│   └── workflows/
├── nginx/
└── README.md

✓ tests/
├── integration/
├── e2e/
├── fixtures/
│   └── midi-files/
└── README.md

✓ backups/
├── .gitignore
└── README.md

✓ .vscode/

TOTAL: 50+ directories + README files + .gitignore files
```

### **create-structure.sh Creates:**

```
✓ Basic folders only (no docs, no validation)
├── config/
├── docs/
├── database/
├── scripts/
├── shared/
├── pipeline/
├── daw/
├── infrastructure/
├── tests/
├── backups/
└── .vscode/

TOTAL: 11 main directories only
```

---

## ⚡ FASTEST PATH TO RUNNING

**Total time: ~5 minutes**

```bash
# 1. Create and navigate (30 seconds)
mkdir -p ~/projects/midi-software-center
cd ~/projects/midi-software-center

# 2. Copy script (10 seconds)
cp /mnt/user-data/outputs/setup-project-structure.sh .

# 3. Run setup (10 seconds)
bash setup-project-structure.sh

# 4. Open VS Code (immediate)
code .

# 5. Done! Structure is ready (5 minutes total)
```

---

## 📊 COMPARISON TABLE

| Feature | setup-project-structure.sh | create-structure.sh |
|---------|---------------------------|-------------------|
| Directories | 50+ with subdirs | 11 main only |
| README files | ✓ Yes | ✗ No |
| .gitignore | ✓ Yes | ✗ No |
| Validation | ✓ Yes | ✗ No |
| Dry-run mode | ✓ Yes | ✗ No |
| Verbose mode | ✓ Yes | ✗ No |
| Error handling | ✓ Robust | ✗ Basic |
| Time | ~10 sec | ~5 sec |
| Size | ~12 KB | ~2 KB |
| **Recommended** | **✓✓✓ YES** | See below |

**Use create-structure.sh ONLY if:**
- You want super minimal setup
- You're on a very old system
- You want to learn by doing it manually
- You're scripting and want lightweight

**Otherwise use setup-project-structure.sh** for professional setup.

---

## 🐛 TROUBLESHOOTING

### **"Permission denied"**
```bash
chmod +x setup-project-structure.sh
bash setup-project-structure.sh
```

### **"No such file or directory"**
```bash
# Make sure you're in the right directory
cd ~/projects/midi-software-center
pwd  # Verify output

# Then run script
bash setup-project-structure.sh
```

### **Script not running?**
```bash
# Use bash explicitly (not sh)
bash setup-project-structure.sh  # ✓ Correct

# NOT this:
sh setup-project-structure.sh    # ✗ Wrong (may fail)
```

### **Permission issues in backups/ folder?**
```bash
# The script creates backups/ with proper permissions
# If you have issues:
mkdir -p backups
chmod 755 backups
echo "*" > backups/.gitignore
```

---

## ✅ SUCCESS CRITERIA

After running setup script, you should have:

```bash
# Check these:
☑ 50+ directories created
☑ config/ exists with README.md
☑ docs/ exists with subdirectories
☑ scripts/ exists with modules/, tasks/, launch/, grown-up/
☑ database/ exists with migrations/, queries/
☑ backups/ has .gitignore file
☑ All main folders have README.md

# Verify with:
find ~/projects/midi-software-center -name "README.md" | wc -l
# Should show: 12+ README files

find ~/projects/midi-software-center -name ".gitignore" | wc -l
# Should show: 2+ .gitignore files
```

---

## 📚 DOCUMENTATION

All scripts come with inline documentation:

```bash
# View script help
bash setup-project-structure.sh --help

# View inline comments
less setup-project-structure.sh

# Check what would run
bash setup-project-structure.sh --dry-run
```

---

## 🎓 LEARNING PATH

1. **Read:** `SETUP_SCRIPT_USAGE.md` (this file's companion)
2. **Preview:** `bash setup-project-structure.sh --dry-run`
3. **Run:** `bash setup-project-structure.sh`
4. **Verify:** `find . -type d | wc -l`
5. **Explore:** `ls -la` to see what was created
6. **Read:** Check README.md files in each folder

---

## 🚀 NEXT AFTER SETUP

```bash
# 1. Files are ready, now install dependencies
cd ~/projects/midi-software-center
make setup

# 2. Start database
make docker-up

# 3. Start development
make dev-both

# 4. Open applications
# http://localhost:5173 (Pipeline)
# http://localhost:5174 (DAW)

# Done! You're ready to code!
```

---

## 📝 SCRIPT LOCATIONS

Both scripts available in `/mnt/user-data/outputs/`:

```
/mnt/user-data/outputs/
├── setup-project-structure.sh     ⭐ Use this one
├── create-structure.sh             (Optional, lightweight)
├── SETUP_SCRIPT_USAGE.md           (Usage guide)
└── VSCODE_SETUP_GUIDE.md          (VS Code setup)
```

**Copy to your project:**
```bash
cp /mnt/user-data/outputs/setup-project-structure.sh ~/projects/midi-software-center/
```

---

## 💡 PRO TIPS

✅ **Always dry-run first**
```bash
bash setup-project-structure.sh --dry-run
```

✅ **Save output for records**
```bash
bash setup-project-structure.sh | tee setup.log
```

✅ **Run in VS Code terminal**
- Press Ctrl+` to open terminal
- Then: `bash setup-project-structure.sh`

✅ **Safe to run multiple times**
- Script is idempotent
- Won't delete existing files
- Safe cleanup if you mess up

✅ **Combine with other setup**
```bash
bash setup-project-structure.sh && \
make setup && \
make docker-up && \
code .
```

---

## 🎉 YOU'RE READY!

**Everything you need is prepared:**
- ✓ Folder structure script (production-ready)
- ✓ Usage guide
- ✓ Troubleshooting tips
- ✓ Next steps

**To get started:**
```bash
cd ~/projects/midi-software-center
bash setup-project-structure.sh
code .
```

**Time to first working development environment: ~5 minutes** ⚡

---

**Status:** Complete and tested  
**Quality:** Production-ready  
**Difficulty:** ⭐ Very Easy  
**Next Step:** Run the script!

