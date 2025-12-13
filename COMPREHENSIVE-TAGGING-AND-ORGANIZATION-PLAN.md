# Comprehensive Plan: Database Tagging, Metadata Organization & Categorization

**Version:** 1.0
**Date:** December 9, 2025
**Scope:** ~/midi FastAPI backend + ~/projects/midi-software-center Rust pipeline
**Target:** 1.72M unique MIDI files

---

## Executive Summary

This plan outlines improvements to the MIDI Software Center's tagging, metadata, and categorization systems based on:
- Analysis of 26 Rust binaries, 24 Python scripts, and 50+ bash scripts
- Review of existing 97 instrument tags and 1,640 curated keywords
- Web research on industry best practices (Cosmos, Sononym, ADSR Sample Manager)
- Integration opportunities between ~/midi API and ~/projects Rust pipeline

**Key Improvements:**
1. Hierarchical taxonomy with 500+ refined tags
2. AI-powered semantic similarity clustering
3. Confidence-weighted tag scoring system
4. Unified metadata schema across both systems
5. Smart renaming with metadata preservation

---

## Part 1: Current State Analysis

### 1.1 Existing Tagging Infrastructure

| System | Location | Tags | Method |
|--------|----------|------|--------|
| Rust Auto-Tagger | `pipeline/src-tauri/src/core/analysis/auto_tagger.rs` | 350+ patterns | Regex + filename |
| Python Fast Tagger | `scripts/fast_multi_level_tagger.py` | 1,640 keywords | 3-level path extraction |
| SQL Instrument Tags | `database/organize_by_instruments.sql` | 97 instruments | Keyword matching |
| FastAPI Backend | `~/midi/api/routes/batch/tagging.py` | Dynamic | 10 microservices |

### 1.2 Current Tag Categories (10)

```
genre, instrument, element, key, tempo, mood, structure, brand, world, technique
```

### 1.3 Gaps Identified

| Gap | Impact | Priority |
|-----|--------|----------|
| No semantic similarity | Can't find "similar sounding" files | HIGH |
| Limited mood detection | Only 15 mood tags vs industry 50+ | MEDIUM |
| No sub-genre hierarchy | "electronic" vs "house/techno/trance" | HIGH |
| Inconsistent confidence scores | Python uses 0-1, Rust uses 0.60-0.95 | MEDIUM |
| No user-defined tags | Community tagging unavailable | LOW |
| No tag relationships | "drums" doesn't link to "percussion" | HIGH |

---

## Part 2: Improved Taxonomy Design

### 2.1 Hierarchical Tag Structure (500+ tags)

```
├── INSTRUMENT (150 tags)
│   ├── drums/
│   │   ├── acoustic/ (kick, snare, hihat, tom, crash, ride, etc.)
│   │   ├── electronic/ (808, 909, tr-707, linn, etc.)
│   │   └── world/ (tabla, djembe, taiko, bongo, conga, etc.)
│   ├── bass/
│   │   ├── electric/ (fingered, slapped, picked, fretless)
│   │   ├── synth/ (sub, reese, acid, wobble)
│   │   └── upright/ (pizzicato, arco, walking)
│   ├── keys/
│   │   ├── piano/ (grand, upright, honky-tonk, prepared)
│   │   ├── organ/ (hammond, church, farfisa, vox)
│   │   ├── synth/ (lead, pad, arp, pluck, brass, string)
│   │   └── electric/ (rhodes, wurlitzer, clavinet, cp80)
│   ├── strings/
│   │   ├── orchestral/ (violin, viola, cello, bass, ensemble)
│   │   └── solo/ (fiddle, erhu, koto, sitar)
│   ├── brass/
│   │   ├── section/ (trumpets, trombones, horns, tubas)
│   │   └── solo/ (trumpet, sax, trombone, flugelhorn)
│   ├── woodwind/
│   │   ├── reed/ (clarinet, oboe, bassoon, harmonica)
│   │   └── flute/ (concert, piccolo, pan, shakuhachi)
│   ├── guitar/
│   │   ├── acoustic/ (steel, nylon, 12-string)
│   │   ├── electric/ (clean, distorted, jazz, country)
│   │   └── world/ (flamenco, oud, bouzouki, banjo)
│   ├── vocals/
│   │   ├── lead/ (male, female, choir)
│   │   └── fx/ (vocoder, talk-box, autotune)
│   └── fx/
│       ├── synth/ (riser, sweep, impact, noise)
│       └── foley/ (ambient, transition, stinger)
│
├── GENRE (100 tags)
│   ├── electronic/
│   │   ├── house/ (deep, tech, progressive, acid, disco)
│   │   ├── techno/ (detroit, berlin, minimal, industrial)
│   │   ├── trance/ (uplifting, psy, progressive, goa)
│   │   ├── dnb/ (liquid, neuro, jump-up, jungle)
│   │   ├── dubstep/ (brostep, melodic, riddim)
│   │   └── ambient/ (dark, space, drone, chillout)
│   ├── hip-hop/
│   │   ├── boom-bap, trap, lo-fi, g-funk, crunk, drill
│   ├── rock/
│   │   ├── classic, alternative, metal, punk, indie, grunge
│   ├── jazz/
│   │   ├── bebop, fusion, smooth, latin, swing, free
│   ├── pop/
│   │   ├── synth-pop, indie-pop, k-pop, disco, funk
│   ├── classical/
│   │   ├── baroque, romantic, contemporary, minimalist
│   └── world/
│       ├── latin, african, asian, middle-eastern, celtic
│
├── MOOD (50 tags)
│   ├── energy/
│   │   ├── high/ (energetic, powerful, aggressive, driving)
│   │   ├── medium/ (groovy, steady, relaxed)
│   │   └── low/ (calm, peaceful, meditative, ambient)
│   ├── emotion/
│   │   ├── positive/ (happy, uplifting, euphoric, hopeful)
│   │   ├── neutral/ (mysterious, tension, suspense)
│   │   └── negative/ (dark, melancholic, sad, angry)
│   └── character/
│       ├── cinematic, epic, intimate, playful, ethereal
│
├── TEMPO (15 tags)
│   ├── very-slow (30-60 BPM)
│   ├── slow (61-90 BPM)
│   ├── mid-tempo (91-120 BPM)
│   ├── upbeat (121-140 BPM)
│   ├── fast (141-180 BPM)
│   └── very-fast (181-300 BPM)
│
├── KEY (24 tags)
│   ├── major/ (C, C#, D, D#, E, F, F#, G, G#, A, A#, B)
│   └── minor/ (Cm, C#m, Dm, D#m, Em, Fm, F#m, Gm, G#m, Am, A#m, Bm)
│
├── STRUCTURE (30 tags)
│   ├── pattern/ (loop, one-shot, phrase, riff)
│   ├── section/ (intro, verse, chorus, bridge, outro, breakdown)
│   ├── technique/ (fill, groove, roll, ghost-notes, double-bass)
│   └── length/ (1-bar, 2-bar, 4-bar, 8-bar, 16-bar)
│
├── ERA (15 tags)
│   ├── decade/ (60s, 70s, 80s, 90s, 2000s, 2010s, 2020s)
│   └── style/ (vintage, retro, modern, futuristic)
│
├── BRAND (50 tags)
│   ├── sample-packs/ (splice, loopmasters, prime-loops, etc.)
│   └── hardware/ (roland, yamaha, korg, native-instruments, etc.)
│
└── QUALITY (20 tags)
    ├── production/ (professional, demo, sketch)
    ├── complexity/ (simple, intermediate, complex)
    └── usability/ (ready-to-use, needs-editing, stems)
```

### 2.2 Tag Relationships (Synonyms & Hierarchies)

```sql
-- New table for tag relationships
CREATE TABLE tag_relationships (
    id SERIAL PRIMARY KEY,
    parent_tag_id INTEGER REFERENCES tags(id),
    child_tag_id INTEGER REFERENCES tags(id),
    relationship_type VARCHAR(20), -- 'parent', 'synonym', 'related'
    strength DECIMAL(3,2) DEFAULT 1.0,
    UNIQUE(parent_tag_id, child_tag_id)
);

-- Example relationships
INSERT INTO tag_relationships (parent_tag_id, child_tag_id, relationship_type) VALUES
-- Parent-child
((SELECT id FROM tags WHERE name='drums'), (SELECT id FROM tags WHERE name='kick'), 'parent'),
((SELECT id FROM tags WHERE name='drums'), (SELECT id FROM tags WHERE name='snare'), 'parent'),
((SELECT id FROM tags WHERE name='electronic'), (SELECT id FROM tags WHERE name='house'), 'parent'),
((SELECT id FROM tags WHERE name='house'), (SELECT id FROM tags WHERE name='deep-house'), 'parent'),

-- Synonyms
((SELECT id FROM tags WHERE name='drums'), (SELECT id FROM tags WHERE name='percussion'), 'synonym'),
((SELECT id FROM tags WHERE name='synth'), (SELECT id FROM tags WHERE name='synthesizer'), 'synonym'),
((SELECT id FROM tags WHERE name='808'), (SELECT id FROM tags WHERE name='tr-808'), 'synonym'),

-- Related
((SELECT id FROM tags WHERE name='house'), (SELECT id FROM tags WHERE name='disco'), 'related'),
((SELECT id FROM tags WHERE name='jazz'), (SELECT id FROM tags WHERE name='funk'), 'related');
```

---

## Part 3: Confidence-Weighted Tagging System

### 3.1 Unified Confidence Scale (0.0 - 1.0)

| Source | Base Confidence | Notes |
|--------|-----------------|-------|
| User-defined | 1.00 | Manual tagging |
| Audio analysis (AI) | 0.90-0.95 | ML model output |
| Pack/collection name | 0.85-0.90 | High reliability |
| Folder structure | 0.75-0.85 | Context-dependent |
| Filename keywords | 0.70-0.80 | Pattern matching |
| MIDI content analysis | 0.65-0.80 | BPM, key, drums |
| Semantic inference | 0.50-0.70 | Related tags |

### 3.2 Confidence Calculation Formula

```rust
// Proposed unified confidence calculation
fn calculate_tag_confidence(
    source: TagSource,
    pattern_specificity: f32,  // 0.0-1.0, how specific the match
    context_support: f32,      // 0.0-1.0, supporting evidence
) -> f32 {
    let base = match source {
        TagSource::UserDefined => 1.0,
        TagSource::AudioAnalysis => 0.92,
        TagSource::PackName => 0.87,
        TagSource::FolderPath => 0.80,
        TagSource::Filename => 0.75,
        TagSource::MidiContent => 0.72,
        TagSource::SemanticInference => 0.60,
    };

    // Adjust for specificity and context
    let adjusted = base * (0.7 + 0.3 * pattern_specificity);
    let final_score = adjusted * (0.8 + 0.2 * context_support);

    final_score.clamp(0.0, 1.0)
}
```

### 3.3 Tag Aggregation Strategy

```sql
-- When a file has multiple sources for the same tag, aggregate:
CREATE OR REPLACE FUNCTION aggregate_tag_confidence(
    file_id INTEGER,
    tag_id INTEGER
) RETURNS DECIMAL(3,2) AS $$
DECLARE
    max_conf DECIMAL(3,2);
    avg_conf DECIMAL(3,2);
    source_count INTEGER;
BEGIN
    SELECT MAX(confidence), AVG(confidence), COUNT(*)
    INTO max_conf, avg_conf, source_count
    FROM file_tags_sources
    WHERE file_id = $1 AND tag_id = $2;

    -- Boost confidence when multiple sources agree
    RETURN LEAST(1.0, max_conf + (source_count - 1) * 0.05);
END;
$$ LANGUAGE plpgsql;
```

---

## Part 4: Metadata Schema Unification

### 4.1 Current Schema Differences

| Field | ~/midi (SQLAlchemy) | ~/projects (PostgreSQL) | Unified |
|-------|---------------------|-------------------------|---------|
| BPM | `midi.bpm` (FLOAT) | `musical_metadata.bpm` (SMALLINT) | `metadata.bpm` (DECIMAL(5,2)) |
| Key | `midi.key` (VARCHAR) | `musical_metadata.key_signature` (VARCHAR) | `metadata.key` (VARCHAR(10)) |
| Duration | `midi.duration` (FLOAT) | `musical_metadata.duration_ms` (INTEGER) | `metadata.duration_ms` (INTEGER) |
| Time Sig | N/A | `musical_metadata.time_signature` | `metadata.time_signature` |
| Tracks | `midi.track_count` | N/A | `metadata.track_count` |

### 4.2 Proposed Unified Metadata Schema

```sql
-- Extended metadata table
CREATE TABLE unified_metadata (
    file_id INTEGER PRIMARY KEY REFERENCES files(id),

    -- Temporal
    bpm DECIMAL(5,2),
    bpm_confidence DECIMAL(3,2),
    duration_ms INTEGER,
    time_signature VARCHAR(10),
    tempo_changes JSONB, -- [{tick: 0, bpm: 120}, {tick: 1920, bpm: 140}]

    -- Harmonic
    key_signature VARCHAR(10),
    key_confidence DECIMAL(3,2),
    mode VARCHAR(10), -- major, minor, dorian, etc.
    chord_progression JSONB, -- ["Cmaj", "Am", "F", "G"]

    -- Structural
    track_count SMALLINT,
    channel_count SMALLINT,
    note_count INTEGER,
    event_count INTEGER,
    bar_count SMALLINT,

    -- Quality metrics
    polyphony_avg DECIMAL(4,2),
    note_density DECIMAL(6,2), -- notes per second
    velocity_range INT4RANGE,
    humanization_score DECIMAL(3,2), -- timing variation

    -- Content flags
    has_drums BOOLEAN DEFAULT FALSE,
    has_melody BOOLEAN DEFAULT FALSE,
    has_chords BOOLEAN DEFAULT FALSE,
    has_bass BOOLEAN DEFAULT FALSE,

    -- Analysis metadata
    analyzed_at TIMESTAMPTZ,
    analyzer_version VARCHAR(20),

    -- Extended analysis (JSON)
    drum_analysis JSONB,
    chord_analysis JSONB,
    structure_analysis JSONB,
    controller_data JSONB,

    CONSTRAINT valid_bpm CHECK (bpm IS NULL OR (bpm >= 20 AND bpm <= 400)),
    CONSTRAINT valid_confidence CHECK (
        (bpm_confidence IS NULL OR bpm_confidence BETWEEN 0 AND 1) AND
        (key_confidence IS NULL OR key_confidence BETWEEN 0 AND 1)
    )
);

-- Indexes for common queries
CREATE INDEX idx_metadata_bpm ON unified_metadata(bpm) WHERE bpm IS NOT NULL;
CREATE INDEX idx_metadata_key ON unified_metadata(key_signature) WHERE key_signature IS NOT NULL;
CREATE INDEX idx_metadata_has_drums ON unified_metadata(has_drums) WHERE has_drums = TRUE;
CREATE INDEX idx_metadata_duration ON unified_metadata(duration_ms);
```

### 4.3 Migration Strategy

```sql
-- Migrate from current musical_metadata to unified_metadata
INSERT INTO unified_metadata (
    file_id, bpm, duration_ms, time_signature, key_signature,
    track_count, note_count, has_drums, has_melody, has_chords, has_bass
)
SELECT
    m.file_id,
    m.bpm,
    m.duration_ms,
    m.time_signature,
    m.key_signature,
    m.track_count,
    m.note_count,
    COALESCE(m.has_drums, FALSE),
    COALESCE(m.has_melody, FALSE),
    COALESCE(m.has_chords, FALSE),
    COALESCE(m.has_bass, FALSE)
FROM musical_metadata m
ON CONFLICT (file_id) DO UPDATE SET
    bpm = EXCLUDED.bpm,
    duration_ms = EXCLUDED.duration_ms,
    time_signature = EXCLUDED.time_signature,
    key_signature = EXCLUDED.key_signature;
```

---

## Part 5: Smart Renaming System

### 5.1 Renaming Philosophy

**Principle:** Filenames should be descriptive, searchable, and DAW-friendly.

**Format:** `[BPM]_[Key]_[Genre]_[Instrument]_[Character]_[Original].mid`

**Examples:**
```
120_Cmaj_house_drums_groove_original-name.mid
85_Amin_hiphop_bass_dark_my-bass-loop.mid
140_Fmin_dnb_synth_aggressive_lead-01.mid
```

### 5.2 Renaming Rules

```rust
// Proposed renaming configuration
struct RenameConfig {
    // Template components (ordered)
    components: Vec<RenameComponent>,

    // Formatting options
    separator: String,           // "_" or "-"
    case: Case,                  // Lower, Upper, Title
    max_length: usize,          // 100 chars default
    preserve_original: bool,     // Append original filename

    // Conflict handling
    duplicate_suffix: DuplicateSuffix, // _01, _02 or (1), (2)
}

enum RenameComponent {
    Bpm { format: BpmFormat },      // "120" or "120bpm"
    Key { include_mode: bool },     // "Cmaj" or "C"
    Genre { max_depth: u8 },        // "house" or "electronic-house"
    Instrument { primary_only: bool },
    Mood { top_n: u8 },
    Structure,                       // "loop", "one-shot", etc.
    Original { truncate_at: usize },
    Custom(String),
}
```

### 5.3 Renaming Safety

```rust
// Always create reversible operations
struct RenameOperation {
    file_id: i64,
    old_path: PathBuf,
    new_path: PathBuf,
    timestamp: DateTime<Utc>,
    reversible: bool,
}

// Store rename history in database
CREATE TABLE rename_history (
    id SERIAL PRIMARY KEY,
    file_id INTEGER REFERENCES files(id),
    old_filename VARCHAR(500),
    new_filename VARCHAR(500),
    old_filepath TEXT,
    new_filepath TEXT,
    renamed_at TIMESTAMPTZ DEFAULT NOW(),
    rename_batch_id UUID,
    reversed_at TIMESTAMPTZ
);

// Create index for fast lookups
CREATE INDEX idx_rename_history_file ON rename_history(file_id);
CREATE INDEX idx_rename_history_batch ON rename_history(rename_batch_id);
```

### 5.4 Existing Renaming Scripts Enhancement

Current scripts to enhance:
- `normalize-midi-filenames.sh` - Add metadata preservation
- `restore-original-filenames.py` - Integrate with rename_history table
- `normalize-files-and-database.py` - Add confidence-based renaming
- `strict-sanitize-filenames.py` - Add smart renaming options

---

## Part 6: AI-Powered Semantic Tagging

### 6.1 Similarity Clustering

Inspired by Cosmos and Sononym:

```python
# Proposed similarity clustering using embeddings
from sklearn.cluster import HDBSCAN
import numpy as np

class MidiEmbedder:
    """Generate embeddings from MIDI features for similarity clustering"""

    def extract_features(self, midi_file) -> np.ndarray:
        """Extract numerical features from MIDI"""
        return np.array([
            midi_file.bpm / 200.0,  # Normalize BPM
            midi_file.key_numeric / 24.0,  # 24 keys
            midi_file.duration_ms / 300000.0,  # Up to 5 minutes
            midi_file.note_density / 50.0,  # Notes per second
            midi_file.polyphony_avg / 10.0,  # Average simultaneous notes
            midi_file.velocity_mean / 127.0,
            midi_file.velocity_std / 40.0,
            int(midi_file.has_drums),
            int(midi_file.has_melody),
            int(midi_file.has_chords),
            midi_file.humanization_score,
            # ... more features
        ])

    def cluster_similar(self, embeddings: np.ndarray, min_cluster_size=50):
        """Cluster similar files using HDBSCAN"""
        clusterer = HDBSCAN(min_cluster_size=min_cluster_size)
        labels = clusterer.fit_predict(embeddings)
        return labels

# Store clusters in database
CREATE TABLE similarity_clusters (
    id SERIAL PRIMARY KEY,
    cluster_id INTEGER,
    file_id INTEGER REFERENCES files(id),
    centroid_distance DECIMAL(6,4),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(cluster_id, file_id)
);

CREATE INDEX idx_clusters_cluster ON similarity_clusters(cluster_id);
CREATE INDEX idx_clusters_file ON similarity_clusters(file_id);
```

### 6.2 Auto-Tag Suggestions

```sql
-- Suggest tags based on similar files
CREATE OR REPLACE FUNCTION suggest_tags_from_similar(
    target_file_id INTEGER,
    similarity_threshold DECIMAL DEFAULT 0.7,
    min_tag_frequency INTEGER DEFAULT 3
) RETURNS TABLE(tag_id INTEGER, tag_name VARCHAR, suggestion_score DECIMAL) AS $$
BEGIN
    RETURN QUERY
    WITH similar_files AS (
        SELECT sc2.file_id, sc1.centroid_distance
        FROM similarity_clusters sc1
        JOIN similarity_clusters sc2 ON sc1.cluster_id = sc2.cluster_id
        WHERE sc1.file_id = target_file_id
          AND sc2.file_id != target_file_id
          AND sc2.centroid_distance < (1 - similarity_threshold)
    ),
    tag_counts AS (
        SELECT ft.tag_id, COUNT(*) as freq
        FROM file_tags ft
        JOIN similar_files sf ON ft.file_id = sf.file_id
        WHERE ft.file_id NOT IN (
            SELECT file_id FROM file_tags WHERE file_id = target_file_id
        )
        GROUP BY ft.tag_id
        HAVING COUNT(*) >= min_tag_frequency
    )
    SELECT tc.tag_id, t.name,
           (tc.freq::DECIMAL / (SELECT COUNT(*) FROM similar_files)) as score
    FROM tag_counts tc
    JOIN tags t ON tc.tag_id = t.id
    ORDER BY score DESC
    LIMIT 20;
END;
$$ LANGUAGE plpgsql;
```

---

## Part 7: Implementation Roadmap

### Phase 1: Foundation (Week 1-2)

**Tasks:**
1. [ ] Create unified_metadata table with migration
2. [ ] Expand tags table with 500+ hierarchical tags
3. [ ] Add tag_relationships table
4. [ ] Update confidence scoring in auto_tagger.rs
5. [ ] Add rename_history table

**Deliverables:**
- SQL migration files (012-015)
- Updated Rust auto_tagger module
- Database backup before migration

### Phase 2: Tagging Enhancement (Week 3-4)

**Tasks:**
1. [ ] Implement hierarchical tag queries
2. [ ] Add synonym/related tag expansion
3. [ ] Enhance fast_multi_level_tagger.py with confidence
4. [ ] Create tag suggestion API endpoint
5. [ ] Build batch re-tagging tool

**Deliverables:**
- Enhanced Python tagger with hierarchy support
- New Tauri commands for tag operations
- Tag management UI components

### Phase 3: Smart Renaming (Week 5-6)

**Tasks:**
1. [ ] Implement RenameConfig system in Rust
2. [ ] Create smart_rename binary
3. [ ] Add rename preview/dry-run mode
4. [ ] Integrate with database tracking
5. [ ] Build rename history viewer

**Deliverables:**
- `smart_rename` binary
- Rename configuration UI
- Rollback capability

### Phase 4: AI Integration (Week 7-8)

**Tasks:**
1. [ ] Implement MidiEmbedder feature extraction
2. [ ] Add similarity clustering pipeline
3. [ ] Create tag suggestion system
4. [ ] Build "find similar" search feature
5. [ ] Integrate with ~/midi microservices

**Deliverables:**
- Python similarity clustering service
- API endpoints for similarity search
- Integration bridge to ~/midi backend

### Phase 5: UI & Polish (Week 9-10)

**Tasks:**
1. [ ] Build tag browser UI with hierarchy
2. [ ] Create bulk tagging interface
3. [ ] Add smart rename wizard
4. [ ] Implement similarity visualization
5. [ ] Performance optimization

**Deliverables:**
- Complete Svelte UI components
- Documentation and user guide
- Performance benchmarks

---

## Part 8: Performance Considerations

### 8.1 Batch Processing Targets

| Operation | Current | Target | Method |
|-----------|---------|--------|--------|
| Tag assignment | 500/sec | 2,000/sec | Batch INSERT with COPY |
| Tag search | 50ms | 10ms | GIN indexes + materialized views |
| Similarity query | N/A | 100ms | Pre-computed clusters |
| Rename operation | 1,000/sec | 5,000/sec | Async + batched DB updates |
| Full re-tag | 60 min | 15 min | Parallel processing |

### 8.2 Index Strategy

```sql
-- New indexes for improved performance
CREATE INDEX CONCURRENTLY idx_tags_hierarchy ON tags USING GIN (
    to_tsvector('english', name || ' ' || COALESCE(category, ''))
);

CREATE INDEX CONCURRENTLY idx_file_tags_covering ON file_tags (
    file_id, tag_id, confidence
) INCLUDE (created_at);

CREATE INDEX CONCURRENTLY idx_metadata_composite ON unified_metadata (
    has_drums, has_melody, has_chords, has_bass
) WHERE bpm IS NOT NULL;

-- Partial indexes for common queries
CREATE INDEX idx_drums_files ON file_tags (file_id)
WHERE tag_id IN (SELECT id FROM tags WHERE name = 'drums');

CREATE INDEX idx_high_confidence ON file_tags (file_id, tag_id)
WHERE confidence >= 0.8;
```

### 8.3 Caching Strategy

```rust
// LRU cache for frequent tag lookups
use lru::LruCache;

struct TagCache {
    tag_by_name: LruCache<String, i32>,
    tag_by_id: LruCache<i32, Tag>,
    file_tags: LruCache<i64, Vec<i32>>,
    hierarchy: LruCache<i32, Vec<i32>>,
}

impl TagCache {
    fn new() -> Self {
        Self {
            tag_by_name: LruCache::new(NonZeroUsize::new(10_000).unwrap()),
            tag_by_id: LruCache::new(NonZeroUsize::new(10_000).unwrap()),
            file_tags: LruCache::new(NonZeroUsize::new(100_000).unwrap()),
            hierarchy: LruCache::new(NonZeroUsize::new(1_000).unwrap()),
        }
    }
}
```

---

## Part 9: Integration Points

### 9.1 ~/midi ↔ ~/projects Bridge

```python
# API bridge for unified access
from fastapi import APIRouter
from typing import List, Optional

router = APIRouter(prefix="/v2/tags")

@router.get("/hierarchy/{tag_name}")
async def get_tag_hierarchy(tag_name: str) -> dict:
    """Get full hierarchy for a tag (parent chain + children)"""
    # Calls Rust pipeline via subprocess or gRPC
    pass

@router.post("/bulk-tag")
async def bulk_tag_files(
    file_ids: List[int],
    tags: List[str],
    confidence: float = 0.85
) -> dict:
    """Bulk tag files using Rust pipeline"""
    pass

@router.get("/similar/{file_id}")
async def find_similar_files(
    file_id: int,
    limit: int = 20,
    threshold: float = 0.7
) -> List[dict]:
    """Find similar files using clustering"""
    pass
```

### 9.2 Tauri Commands

```rust
// New Tauri commands for tagging
#[tauri::command]
async fn tag_files_with_hierarchy(
    file_ids: Vec<i64>,
    tag_name: String,
    include_parent_tags: bool,
    confidence: f32,
    state: State<'_, AppState>,
) -> Result<TagResult, String> {
    // Implementation
}

#[tauri::command]
async fn get_tag_suggestions(
    file_id: i64,
    max_suggestions: usize,
    state: State<'_, AppState>,
) -> Result<Vec<TagSuggestion>, String> {
    // Implementation
}

#[tauri::command]
async fn smart_rename_files(
    file_ids: Vec<i64>,
    config: RenameConfig,
    dry_run: bool,
    state: State<'_, AppState>,
) -> Result<Vec<RenamePreview>, String> {
    // Implementation
}
```

---

## Part 10: Success Metrics

### 10.1 Quantitative Goals

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| Tags per file (avg) | 2.3 | 5.0 | `AVG(tag_count) FROM file_tag_counts` |
| Tag coverage | 37% | 95% | `COUNT(DISTINCT file_id) / total_files` |
| High-confidence tags | 60% | 80% | `WHERE confidence >= 0.8` |
| Search relevance | N/A | 85% | User satisfaction sampling |
| Rename accuracy | N/A | 99% | Manual spot-check |

### 10.2 Qualitative Goals

- [ ] Users can find files by musical characteristics, not just keywords
- [ ] Tag hierarchy makes browsing intuitive
- [ ] Similar file discovery enables creative exploration
- [ ] Renaming preserves discoverability while adding metadata
- [ ] System scales to 10M+ files without degradation

---

## Appendix A: Tag Categories Reference

### A.1 Complete Genre Taxonomy

```
electronic
├── house (deep, tech, progressive, acid, disco, afro, soulful, tribal)
├── techno (detroit, berlin, minimal, industrial, acid, dub)
├── trance (uplifting, progressive, psy, goa, vocal, hard)
├── dnb (liquid, neuro, jump-up, jungle, ragga, halftime)
├── dubstep (brostep, melodic, riddim, tearout, deep)
├── ambient (dark, space, drone, chillout, new-age)
├── edm (big-room, future-house, electro-house, festival)
├── breakbeat (breaks, nu-skool, breakcore, big-beat)
└── experimental (glitch, idm, noise, industrial)

hip-hop
├── boom-bap, trap, lo-fi, g-funk, crunk, drill
├── instrumental, conscious, gangsta, alternative
└── phonk, cloud-rap, memphis

rock
├── classic, alternative, metal, punk, indie, grunge
├── progressive, psychedelic, blues-rock, hard-rock
└── post-rock, shoegaze, emo, hardcore

jazz
├── bebop, fusion, smooth, latin, swing, free
├── modal, cool, hard-bop, post-bop
└── acid-jazz, nu-jazz, jazz-funk

classical
├── baroque, romantic, contemporary, minimalist
├── orchestral, chamber, solo, choral
└── film-score, neo-classical

world
├── latin (salsa, reggaeton, bossa, cumbia, bachata)
├── african (afrobeat, highlife, kwaito)
├── asian (k-pop, j-pop, bollywood, gamelan)
├── middle-eastern (arabic, turkish, persian)
└── celtic, reggae, ska, dub
```

### A.2 Drum Pattern Classifications

```
patterns
├── groove (straight, swing, shuffle, half-time, double-time)
├── fill (short, long, tom, snare, crash)
├── intro (count-in, build-up, fade-in)
├── outro (fade-out, breakdown, ending)
└── breakdown (drop, build, tension, release)

techniques
├── ghost-notes, double-bass, blast-beat
├── polyrhythm, syncopation, off-beat
├── rim-shot, cross-stick, brush
└── electronic (sidechain, glitch, stutter)
```

---

## Appendix B: Related Files

### Scripts (~/projects/midi-software-center)
- `scripts/fast_multi_level_tagger.py`
- `scripts/create-curated-tags.sh`
- `scripts/organize-database.sh`
- `scripts/normalize-midi-filenames.sh`
- `scripts/restore-original-filenames.py`

### Database (~/projects/midi-software-center)
- `database/organize_by_instruments.sql`
- `database/optimizations/add_tagging_indexes.sql`
- `database/migrations/001-011_*.sql`

### Rust Modules (~/projects/midi-software-center)
- `pipeline/src-tauri/src/core/analysis/auto_tagger.rs`
- `pipeline/src-tauri/src/core/analysis/filename_metadata.rs`
- `pipeline/src-tauri/src/core/analysis/drum_analyzer.rs`
- `shared/rust/src/db/models/`

### FastAPI Backend (~/midi)
- `api/routes/batch/tagging.py`
- `api/db_models.py`
- `api/routes/filtering.py`
- `api/routes/recommendations.py`

---

**Document Version:** 1.0
**Last Updated:** December 9, 2025
**Author:** Claude Code
**Status:** READY FOR IMPLEMENTATION
