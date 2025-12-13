# Tag-Based MPC Expansion Pack Strategy

**Date:** November 22, 2025
**Based on:** Database analysis of 2.8M files, 8,565 tags, 10.2M tag relationships

---

## 📊 Database Tag Analysis

### Coverage Statistics
- **Total files:** 2,806,055
- **Files with BPM:** 999,295 (35.6%)
- **Files with drum tags:** 632,262 (22.5%)
- **Files with genre tags:** 364,730 (13.0%)
- **Total tag relationships:** 10,249,410

### Tag Categories (17 total)
| Category | Tags | File-Tag Relationships |
|----------|------|------------------------|
| keyword | 1,572 | 7,102,643 |
| auto_extracted | 6,906 | 2,181,455 |
| drums | 23 | 911,731 |
| genre | 14 | 374,015 |
| pattern | 4 | 185,027 |
| bass | 5 | 108,924 |
| synth | 5 | 104,987 |
| keys | 8 | 66,378 |
| guitar | 6 | 51,950 |
| melody | 2 | 35,349 |
| brass | 5 | 29,964 |
| fx | 7 | 27,980 |
| strings | 5 | 27,264 |
| vocal | 5 | 19,595 |
| harmony | 3 | 13,818 |
| woodwind | 4 | 5,825 |
| orchestral | 1 | 2,480 |

### BPM Distribution
| Range | Files | % of Total |
|-------|-------|------------|
| 060-080 | 97,991 | 9.8% |
| 080-100 | 165,492 | 16.6% |
| 100-120 | 216,892 | 21.7% |
| 120-140 | 299,346 | 30.0% |
| 140-160 | 103,097 | 10.3% |
| 160-180 | 53,677 | 5.4% |
| 180+ | 57,967 | 5.8% |

**Key Insight:** 120-140 BPM is the sweet spot (30% of all files)

---

## 🎯 Expansion Pack Design Strategy

### Tier 1: Genre-Based Packs (Largest Appeal)
**Target:** 500-1,000 patterns per pack
**Organization:** Genre → BPM subfolder

```
Expansions/
├── Rock Drum Patterns/
│   ├── 100-120_BPM/
│   ├── 120-140_BPM/
│   └── 140-160_BPM/
├── House Grooves/
│   ├── 118-122_BPM/
│   ├── 122-126_BPM/
│   └── 126-130_BPM/
├── Hip-Hop Beats/
│   ├── 080-095_BPM/
│   └── 095-110_BPM/
└── DnB Patterns/
    ├── 170-174_BPM/
    └── 174-180_BPM/
```

**Packs to create:**
1. **Rock Drum Patterns** (148,388 files) → Export top 1,000
2. **Funk Grooves** (40,592 files) → Export top 500
3. **House Drum Patterns** (39,029 files) → Export top 500
4. **EDM Drums** (35,839 files) → Export top 500
5. **Jazz Patterns** (24,801 files) → Export top 300
6. **Techno Beats** (16,328 files) → Export top 300

---

### Tier 2: Drum-Type Focused (Specific Needs)
**Target:** 300-500 patterns per pack
**Organization:** Drum type → BPM subfolder

```
Expansions/
├── Kick Drum Library/
│   ├── 100-120_BPM/
│   ├── 120-140_BPM/
│   └── 140-160_BPM/
├── Ride Cymbal Patterns/
│   ├── All_Tempos/
│   └── Favorites/
└── Fill Collection/
    ├── Short_Fills/
    ├── Long_Fills/
    └── Transition_Fills/
```

**Packs to create:**
1. **Ride Cymbal Patterns** (162,675 files) → Export top 500
2. **Kick Drum Library** (160,354 files) → Export top 500
3. **Tom Patterns** (145,310 files) → Export top 300
4. **Fill Collection** (121,472 files) → Export top 500
5. **Snare Variations** (46,024 files) → Export top 300
6. **Crash Cymbal Hits** (55,658 files) → Export top 200

---

### Tier 3: Workflow-Based (Pattern Types)
**Target:** 200-400 patterns per pack
**Organization:** Function → Tempo category

```
Expansions/
├── Groove Library/
│   ├── Slow_Grooves/
│   ├── Mid_Tempo/
│   └── Fast_Grooves/
├── Loop Collection/
│   ├── 1_Bar/
│   ├── 2_Bar/
│   └── 4_Bar/
└── Starter Patterns/
    ├── Intros/
    ├── Verses/
    └── Endings/
```

**Packs to create:**
1. **Groove Library** (80,136 files) → Export top 400
2. **Loop Collection** (100,062 files) → Export top 400
3. **Fill Variations** (92,563 files) → Export top 300

---

### Tier 4: Hybrid Packs (Genre + Drum Type)
**Target:** 200-300 patterns per pack
**Best sellers:** Specific combinations

```
Expansions/
├── Rock Kick Patterns 120-140/
├── House Ride Grooves 122-128/
├── Jazz Brush Patterns 80-120/
└── Metal Double-Bass 140-180/
```

**Packs to create (SQL queries):**
```sql
-- Rock + Kick + 120-140 BPM
-- House + Ride + 122-128 BPM
-- Funk + Snare + 90-110 BPM
-- DnB + Fill + 170-180 BPM
```

---

## 🚀 Implementation Plan

### Phase 1: Build Top 10 Expansion Packs

**Priority Order (by file count + appeal):**

1. **Rock Drum Patterns** (148K files)
   - Export: Top 1,000 by rating/quality
   - BPM ranges: 100-120, 120-140, 140-160
   - Estimated size: 150-200 MB

2. **Ride Cymbal Library** (162K files)
   - Export: Top 500
   - All BPM ranges
   - Estimated size: 75-100 MB

3. **Kick Drum Collection** (160K files)
   - Export: Top 500
   - Organized by BPM
   - Estimated size: 75-100 MB

4. **House Grooves 120-130** (39K files)
   - Export: Top 500
   - Narrow BPM focus
   - Estimated size: 75-100 MB

5. **Fill Patterns Collection** (121K files)
   - Export: Top 500
   - By length/complexity
   - Estimated size: 75-100 MB

6. **Funk Drum Patterns** (40K files)
   - Export: Top 400
   - 90-110 BPM focus
   - Estimated size: 60-80 MB

7. **Tom Patterns** (145K files)
   - Export: Top 400
   - All styles
   - Estimated size: 60-80 MB

8. **Groove Library** (80K files)
   - Export: Top 400
   - Mixed styles/tempos
   - Estimated size: 60-80 MB

9. **EDM Drums 120-140** (35K files)
   - Export: Top 300
   - House/Techno/Trance
   - Estimated size: 45-60 MB

10. **Hip-Hop Beats 85-100** (from multiple tags)
    - Export: Top 300
    - Classic BPM range
    - Estimated size: 45-60 MB

---

## 📋 SQL Query Templates

### Template 1: Genre + BPM Range
```sql
SELECT DISTINCT f.id, f.filepath, m.bpm, m.key_signature
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name = 'rock'
  AND t.category = 'genre'
  AND m.bpm BETWEEN 120 AND 140
ORDER BY m.bpm, f.filepath
LIMIT 1000;
```

### Template 2: Drum Type + All BPMs
```sql
SELECT DISTINCT f.id, f.filepath, m.bpm
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name = 'ride'
  AND t.category = 'drums'
  AND m.bpm IS NOT NULL
ORDER BY m.bpm, f.filepath
LIMIT 500;
```

### Template 3: Multiple Tags (Hybrid)
```sql
SELECT DISTINCT f.id, f.filepath, m.bpm,
       string_agg(DISTINCT t.name, ', ') as tags
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name IN ('rock', 'kick')
  AND m.bpm BETWEEN 120 AND 140
GROUP BY f.id, f.filepath, m.bpm
HAVING COUNT(DISTINCT t.name) >= 2
ORDER BY m.bpm
LIMIT 500;
```

---

## 🛠️ Automated Build Script

**Script:** `scripts/build_tag_expansions.py`

```python
#!/usr/bin/env python3
"""
Build MPC expansion packs from database tags

Usage: python3 build_tag_expansions.py --pack rock-drums --limit 1000
"""

import psycopg2
import subprocess
import os
from pathlib import Path

EXPANSION_CONFIGS = {
    'rock-drums': {
        'name': 'Rock Drum Patterns',
        'query': 'SELECT ...', # Template 1
        'bpm_folders': ['100-120', '120-140', '140-160'],
        'limit': 1000
    },
    'ride-library': {
        'name': 'Ride Cymbal Library',
        'query': 'SELECT ...',  # Template 2
        'organize_by': 'bpm',
        'limit': 500
    },
    # ... more configs
}

def build_expansion(pack_id, output_dir):
    config = EXPANSION_CONFIGS[pack_id]

    # 1. Query database for files
    # 2. Convert MIDI → .mpcpattern (batch)
    # 3. Organize into BPM folders
    # 4. Create expansion metadata
    # 5. Generate Cache.json
    # 6. Create preview audio (optional)
```

---

## 📊 Estimated Output

### Total Packs: 10 (Phase 1)
### Total Patterns: ~5,600
### Total Size: ~900 MB
### Conversion Time: ~10-15 minutes
### Organization Time: ~2-3 minutes

**Total Time:** 15-20 minutes for all 10 packs

---

## 🎯 Next Steps

1. **Create build script** (`scripts/build_tag_expansions.py`)
2. **Run Phase 1** (top 10 packs)
3. **Test on Force hardware**
4. **Iterate and refine**
5. **Phase 2:** Create additional 20-30 specialty packs

---

## 💡 Advanced Features (Future)

### Smart Selection Algorithms
- Quality scoring (complexity, variation, uniqueness)
- Diversity selection (avoid similar patterns)
- User preference learning

### Dynamic Packs
- Monthly "Best New Imports"
- "Trending Patterns" based on usage
- Personalized packs per user preferences

### Community Features
- User ratings/favorites
- Collaborative playlists
- Pattern sharing/remixing

---

**Bottom Line:** With 2.8M files, 8,565 tags, and smart queries, we can create hundreds of curated expansion packs automatically. Phase 1 focuses on the top 10 most valuable packs totaling ~5,600 patterns.
