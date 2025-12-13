# Akai Force MIDI Integration - Complete Research Guide

**Created:** November 22, 2025
**Research Summary:** Progressions, Arp Patterns, .mid file usage, and commercial packs

---

## 🎹 1. Progressions Integration (Pad Perform Menu)

### ✅ **WORKS PERFECTLY - Simple Installation**

**Location:** Root of any storage device (SD, USB, SATA)
```
/media/dojevou/RYXSTR/Progressions/
```

### How to Access on Force:

1. **Navigate to Pad Perform Mode**
   - Press **Shift** + **Notes** button
   - Or go to Notes mode → Shift + Notes

2. **Configure Note Settings**
   - Press **Shift + Notes** to open Note Config
   - Set **Type** to "Progressions"

3. **Select Your Progression**
   - Under **Progression** field, browse progressions
   - Custom progressions appear under **"Other"** tab
   - Select desired progression

4. **Play**
   - Pads now trigger chord progressions
   - +/- buttons shift octave

### File Format Required:
- ✅ **.progression files** (proprietary format)
- ❌ **.mid files** (must be converted)

### Conversion Tools:
- **[Amit's Progression Builder](https://midi.amitszone.com/FORCE/PBUILDER/)** (FREE web tool)
- **GNMidi Pro** ($49) + custom add-on (€20) for batch conversion

### Current Status:
✅ Folder created: `/media/dojevou/RYXSTR/Progressions/` with 24 key subfolders
⏳ Files need conversion from .mid to .progression format

---

## 🎛️ 2. Arpeggiator Patterns Integration

### ⚠️ **REQUIRES SSH ACCESS - Advanced Only**

**Official Method:** ❌ Not supported in standalone mode
**Workaround:** ✅ SSH access to internal system folder

### Internal Location:
```
/usr/share/Akai/SME0/Arp Patterns/
```

### Installation Methods:

#### Option 1: MockbaMod Firmware (Easiest)
- Custom firmware with SSH enabled
- Includes **CustomArpPatterns AddOn**
- Allows easy addition of MIDI files as arp patterns
- ⚠️ Voids warranty, unofficial modification

#### Option 2: SSH Access (Manual)
**Requirements:**
- SSH-enabled Force firmware
- SCP access to device
- Terminal/command line knowledge

**Steps:**
1. Enable SSH on Force (requires custom firmware)
2. Mount filesystem read/write: `mount -o rw,remount /`
3. Copy .mid files via SCP: `scp pattern.mid force:/usr/share/Akai/SME0/Arp Patterns/`
4. Restart Force

**⚠️ Warnings:**
- Firmware updates will wipe custom patterns
- Many patterns increase boot time (cached to RAM)
- Not accessible via USB in standalone mode

### Access on Force:
- Open Arpeggiator settings
- Custom patterns appear in pattern selection dropdown
- Available when arpeggiator is enabled on track

### Recommendation:
**❌ NOT RECOMMENDED** for your use case:
- Too technical (SSH/filesystem modification)
- Lost on firmware updates
- Better to use patterns as regular MIDI clips instead

---

## 📁 3. Best Way to Use .mid Files on Force

### **The Reality: Force Doesn't Browse .mid Files** 🚨

**What Force CAN browse:**
- ✅ .mpcpattern files (proprietary)
- ✅ .progression files (chord progressions)
- ✅ .xpm files (audio programs/kits)
- ❌ .mid files (not recognized in browser)

### Recommended Workflows:

#### **Option A: Convert to .mpcpattern** ⭐ BEST FOR PATTERNS

**Tools:**
- **[Midian](http://www.fentonia.com/catnip/midianmpc/)** (FREE web tool)
  - One file at a time
  - Client-side processing
  - Instant download

- **MPC Software** (Official, paid)
  - Drag .mid into track
  - File → Export → Pattern
  - Saves as .mpcpattern

**Placement:**
```
/media/dojevou/RYXSTR/MIDI_Patterns/
└── (organized by BPM/genre as we created)
```

**Access:**
- Browser → Tap "bars" icon (pattern filter)
- Load directly to clips
- Works in current project

---

#### **Option B: Convert to .progression** ⭐ BEST FOR CHORDS

**Tools:**
- **[Amit's Progression Builder](https://midi.amitszone.com/FORCE/PBUILDER/)** (FREE)
- **GNMidi Pro** ($49 + €20 add-on) for batch conversion

**Placement:**
```
/media/dojevou/RYXSTR/Progressions/
└── (organized by key as we created)
```

**Access:**
- Pad Perform mode
- Type: Progressions
- Under "Other" tab

---

#### **Option C: Import via MPC Software** (Hybrid Workflow)

**Process:**
1. Keep .mid files on computer/drive
2. When needed, import to MPC Software
3. Export as .mpcpattern or .progression
4. Copy to Force

**Benefits:**
- Full control over conversion
- Can edit before export
- Add MPC-specific features (ratchets, 16 levels)

**Drawbacks:**
- Manual, one-by-one
- Requires MPC Software installed
- Extra step in workflow

---

#### **Option D: Keep as .mid Reference Library** (Our Current Approach)

**What We Built:**
```
/media/dojevou/RYXSTR/
├── Progressions/ (24 key folders)
├── Arp Patterns/ (6 BPM folders)
└── MIDI_Patterns/ (organized patterns)
```

**Workflow:**
1. Browse organized .mid files on drive
2. When you find something useful:
   - Convert with Midian (quick, free)
   - Or import via MPC Software
   - Or load in DAW and re-export
3. Now usable on Force

**Benefits:**
- Organized library of 20,000+ files
- Easy to browse on computer
- Convert on-demand, not all upfront
- Keep original .mid files intact

---

## 🏪 4. Commercial MIDI Expansion Packs

### **YES, They Exist!** ✅

Major providers sell MIDI pattern libraries for Force/MPC:

#### **MPC-Samples.com**
- MIDI patterns in .mpcpattern format
- Drum grooves played by real drummers
- Chord progressions in .progression format
- Genre packs: Funk, Jazz, Rock, Hip-Hop
- Drag & drop installation
- **Free packs available**

Example Products:
- "MPC Drum Patterns Bundle" - thousands of .mpcpattern files
- "Funk MIDI Grooves" - real drummer performances
- "MPC Chord Progressions" - 500+ progressions

#### **Superb-Sound.com**
- Premium MPC Expansions
- Includes MIDI chord progressions
- Optimized for standalone workflow
- Compatible with Force, MPC Live, X, One, Key

#### **TheMPCStore.com**
- Genre-specific MIDI packs
- Sequences + patterns included
- New Jack, Trap, Latin Urban styles

#### **TheCycleKit.com**
- "Masada MIDI Progressions Bundle"
- Almost 400 MIDI progressions
- Soul, Trap, R&B, Dance, Hip-hop

### Standard Pack Contents:
- ✅ .mpcpattern files (drum grooves, patterns)
- ✅ .progression files (chord progressions)
- ✅ .xpm programs (audio kits)
- ✅ Samples (.wav files)
- ✅ Sequences (full arrangements)

### Installation:
**Simple drag & drop to Force's Expansions folder:**
```
/media/dojevou/RYXSTR/Expansions/
└── [Pack Name]/
    ├── Programs/ (.xpm files)
    ├── Samples/ (.wav files)
    ├── MIDI Patterns/ (.mpcpattern files)
    └── Progressions/ (.progression files)
```

---

## 🎯 Recommendations for Your Library

Based on research, here's the optimal strategy:

### Phase 1: Keep Current Structure ✅ (DONE)
```
/media/dojevou/RYXSTR/
├── Progressions/ (organized by key)
├── Arp Patterns/ (organized by BPM)
└── MIDI_Patterns/ (organized by type/BPM)
```

All folders populated with .mid files for reference.

---

### Phase 2: Selective Conversion (On-Demand)

**Start Small:**
1. Export 100 best progressions
2. Convert via [Amit's Builder](https://midi.amitszone.com/FORCE/PBUILDER/)
3. Test on Force
4. Scale up if workflow works

**For Patterns:**
1. Export 500 best drum patterns
2. Convert via [Midian](http://www.fentonia.com/catnip/midianmpc/)
3. Test loading in Force clips
4. Scale up if needed

---

### Phase 3: Hybrid Workflow

**Reference Library (Computer):**
- Keep full 2.8M MIDI database
- Browse/search with database queries
- Organized folders on Force drive

**Working Library (Force):**
- 500-2,000 converted .mpcpattern files
- 200-500 .progression files
- Curated, performance-ready

**Conversion Bridge:**
- Use Midian for quick pattern conversion
- Use Amit's Builder for progressions
- MPC Software for complex edits

---

## 📋 Action Plan Summary

### Immediate (What We Built):
✅ Folder structure created
✅ Export script ready
✅ 20,000+ MIDI files organized on drive

### Next Steps:

**1. Convert Sample Set (Test)**
```bash
# Export 100 best progressions
# Manually convert via Amit's Builder
# Copy to Progressions/ folders
# Test on Force in Pad Perform mode
```

**2. If Successful, Scale Up**
```bash
# Consider GNMidi Pro for batch conversion
# Convert 5,000-10,000 progressions
# Automate with scripts if possible
```

**3. For Patterns**
```bash
# Export 500 drum patterns
# Batch convert with Midian or MPC Software
# Copy to MIDI_Patterns/Drums/
# Test loading in Force clips
```

**4. Skip Arp Patterns**
❌ Too complex (SSH required)
✅ Use as regular MIDI patterns instead

---

## 🔑 Key Takeaways

1. **Progressions:** ✅ Simple, works great, needs .progression format
2. **Arp Patterns:** ❌ Complex, requires SSH, skip for now
3. **.mid Files:** ❌ Not browseable on Force, must convert
4. **Commercial Packs:** ✅ Yes! Many providers sell MIDI expansions
5. **Best Workflow:** Hybrid (reference library + converted working set)

---

## 🛠️ Tools You'll Need

**Free Tools:**
- ✅ [Amit's Progression Builder](https://midi.amitszone.com/FORCE/PBUILDER/) - Web-based
- ✅ [Midian](http://www.fentonia.com/catnip/midianmpc/) - Pattern converter

**Paid Tools (Optional):**
- GNMidi Pro ($49) + custom add-on (€20) - Batch progression conversion
- MPC Software ($) - Official Akai conversion method

**Scripts We Built:**
- ✅ `scripts/export-force-midi.py` - Database export to Force folders

---

## 📂 File Format Cheat Sheet

| Format | Force Browsers? | Use Case | Location | Access Method |
|--------|----------------|----------|----------|---------------|
| .mid | ❌ No | Reference only | Any folder | Computer browsing |
| .mpcpattern | ✅ Yes | Drum/bass patterns | Any folder | Browser → Bars icon |
| .progression | ✅ Yes | Chord progressions | Progressions/ | Pad Perform → Other |
| .xpm | ✅ Yes | Audio programs | Expansions/ | Browser → Programs |
| Arp .mid | ⚠️ SSH only | Arpeggiator | System folder | Arp settings (if SSH) |

---

**Bottom Line:** Your current approach with organized .mid files is PERFECT as a reference library. Convert to Force-native formats on-demand rather than converting everything upfront.
