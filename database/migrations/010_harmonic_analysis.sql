-- Migration 010: Add Harmonic Analysis (Phase 1 Enhancement)
-- Date: 2025-11-11
-- Purpose: Add chord progression and harmonic complexity analysis

-- Add harmonic analysis columns to musical_metadata table
ALTER TABLE musical_metadata ADD COLUMN IF NOT EXISTS chord_progression JSONB;
ALTER TABLE musical_metadata ADD COLUMN IF NOT EXISTS chord_types TEXT[];
ALTER TABLE musical_metadata ADD COLUMN IF NOT EXISTS has_seventh_chords BOOLEAN DEFAULT FALSE;
ALTER TABLE musical_metadata ADD COLUMN IF NOT EXISTS has_extended_chords BOOLEAN DEFAULT FALSE;
ALTER TABLE musical_metadata ADD COLUMN IF NOT EXISTS chord_change_rate NUMERIC(5,2);
ALTER TABLE musical_metadata ADD COLUMN IF NOT EXISTS chord_complexity_score NUMERIC(4,3)
    CHECK (chord_complexity_score IS NULL OR (chord_complexity_score BETWEEN 0 AND 1));

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_has_seventh_chords
    ON musical_metadata(has_seventh_chords)
    WHERE has_seventh_chords = true;

CREATE INDEX IF NOT EXISTS idx_has_extended_chords
    ON musical_metadata(has_extended_chords)
    WHERE has_extended_chords = true;

CREATE INDEX IF NOT EXISTS idx_chord_progression
    ON musical_metadata USING GIN(chord_progression);

CREATE INDEX IF NOT EXISTS idx_chord_types
    ON musical_metadata USING GIN(chord_types);

CREATE INDEX IF NOT EXISTS idx_chord_complexity
    ON musical_metadata(chord_complexity_score)
    WHERE chord_complexity_score IS NOT NULL;

-- Comments for documentation
COMMENT ON COLUMN musical_metadata.chord_progression IS
    'JSON array of detected chord names in order, e.g., ["Cm", "Fm", "Gm", "Cm"]';

COMMENT ON COLUMN musical_metadata.chord_types IS
    'Array of chord qualities: major, minor, diminished, augmented, seventh, etc.';

COMMENT ON COLUMN musical_metadata.has_seventh_chords IS
    'True if file contains any 7th chords (dominant, major7, minor7, etc.)';

COMMENT ON COLUMN musical_metadata.has_extended_chords IS
    'True if file contains extended chords (9ths, 11ths, 13ths, etc.)';

COMMENT ON COLUMN musical_metadata.chord_change_rate IS
    'Average number of chord changes per measure (NULL if no time signature)';

COMMENT ON COLUMN musical_metadata.chord_complexity_score IS
    'Normalized score (0-1) based on chord types used: triads=0.3, sevenths=0.6, extended=1.0';
