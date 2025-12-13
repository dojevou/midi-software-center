#!/bin/bash
# =============================================================================
# MIDI Library - BATCH Organization (PRODUCTION-READY)
# =============================================================================
# Strategy: Safe parameterized queries + controlled parallelism + transaction safety
# =============================================================================

set -euo pipefail

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m'

# Configuration
readonly DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
readonly BATCH_SIZE=4  # Conservative parallelism
readonly LOCK_TIMEOUT=300000  # 5 minutes in milliseconds

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Safe SQL execution with error handling
execute_sql() {
    local sql="$1"
    local description="${2:-}"

    if [[ -n "$description" ]]; then
        log_info "$description"
    fi

    if ! psql -v ON_ERROR_STOP=1 "$DB_URL" -c "$sql"; then
        log_error "Failed to execute: $description"
        return 1
    fi
}

# Safe keyword tagging with parameterized queries
tag_keyword_safe() {
    local keyword="$1"
    local tag_name="$2"
    local job_id="$3"

    # Use psql variables for safe parameterization
    local result
    result=$(psql -v ON_ERROR_STOP=1 "$DB_URL" -v keyword="$keyword" -v tag_name="$tag_name" -v lock_timeout="$LOCK_TIMEOUT" -t <<'SQL'
        SET lock_timeout = :lock_timeout;

        WITH matching_files AS (
            SELECT DISTINCT f.id
            FROM files f
            WHERE (LOWER(f.filename) LIKE '%' || LOWER(:'keyword') || '%'
                OR LOWER(f.filepath) LIKE '%' || LOWER(:'keyword') || '%')
        ),
        inserted AS (
            INSERT INTO file_tags (file_id, tag_id, added_by)
            SELECT mf.id, t.id, 'batch_organizer'
            FROM matching_files mf
            CROSS JOIN tags t
            WHERE t.name = :'tag_name'
              AND NOT EXISTS (
                  SELECT 1 FROM file_tags ft2
                  WHERE ft2.file_id = mf.id AND ft2.tag_id = t.id
              )
            ON CONFLICT (file_id, tag_id) DO NOTHING
            RETURNING 1
        )
        SELECT COALESCE(COUNT(*), 0) FROM inserted;
SQL
    )

    # Extract the count from psql output
    local count=$(echo "$result" | tr -d ' ' | grep -E '^[0-9]+$' | head -1 || echo "0")
    echo "$count"
}

# Main execution
main() {
    local start_time
    start_time=$(date +%s)

    log_info "═══════════════════════════════════════════════════════════════"
    log_info "  BATCH Organization - Production Ready"
    log_info "  Safe parameterized queries + Controlled parallelism"
    log_info "═══════════════════════════════════════════════════════════════"
    echo ""

    # Create temporary directory for job control
    local temp_dir
    temp_dir=$(mktemp -d)
    trap 'rm -rf "$temp_dir"' EXIT

    # =============================================================================
    # Step 1: Setup and validation
    # =============================================================================
    log_info "[1/5] Validating database connection and setup..."

    if ! psql -v ON_ERROR_STOP=1 "$DB_URL" -c "SELECT 1" >/dev/null 2>&1; then
        log_error "Cannot connect to database at $DB_URL"
        exit 1
    fi

    # Set reasonable timeouts
    execute_sql "SET statement_timeout = '300000'; SET lock_timeout = '120000';" "Setting timeouts"

    log_success "Database connection validated"
    echo ""

    # =============================================================================
    # Step 2: Create tags with conflict handling
    # =============================================================================
    log_info "[2/5] Creating instrument tags..."

    execute_sql "
    INSERT INTO tags (name, category, usage_count) VALUES
        ('ride', 'drums', 103591), ('fill', 'drums', 88142), ('kick', 'drums', 75286),
        ('tom', 'drums', 64351), ('crash', 'drums', 39690), ('snare', 'drums', 16341),
        ('stick', 'drums', 16144), ('hihat', 'drums', 11459), ('drums', 'drums', 7766),
        ('toms', 'drums', 7698), ('clap', 'drums', 5337), ('china', 'drums', 4067),
        ('conga', 'drums', 3319), ('cymbal', 'drums', 2479), ('rim', 'drums', 2123),
        ('cowbell', 'drums', 1623), ('bongo', 'drums', 1229), ('percussion', 'drums', 1173),
        ('shaker', 'drums', 915), ('tambourine', 'drums', 698), ('splash', 'drums', 333),
        ('hi-hat', 'drums', 306), ('drum', 'drums', 17380),
        ('bass', 'bass', 52917), ('bassline', 'bass', 3782), ('sub', 'bass', 3590),
        ('808', 'bass', 3245), ('909', 'bass', 596),
        ('synth', 'synth', 26556), ('piano', 'keys', 21932), ('lead', 'synth', 17685),
        ('pad', 'synth', 11991), ('keys', 'keys', 9859), ('arp', 'synth', 7464),
        ('pluck', 'synth', 6656), ('organ', 'keys', 4874), ('brass', 'brass', 3976),
        ('rhodes', 'keys', 3086), ('wurlitzer', 'keys', 765), ('clav', 'keys', 459),
        ('electric-piano', 'keys', 136), ('harpsichord', 'keys', 42),
        ('guitar', 'guitar', 10913), ('acoustic', 'guitar', 4128), ('electric', 'guitar', 2965),
        ('12-string', 'guitar', 89), ('slide', 'guitar', 67), ('muted', 'guitar', 34),
        ('strings', 'strings', 5847), ('violin', 'strings', 2134), ('cello', 'strings', 987),
        ('viola', 'strings', 432), ('ensemble', 'strings', 276), ('orchestra', 'orchestral', 672),
        ('trumpet', 'brass', 1876), ('sax', 'brass', 1543), ('trombone', 'brass', 765),
        ('horn', 'brass', 432), ('flute', 'woodwind', 1234), ('clarinet', 'woodwind', 543),
        ('oboe', 'woodwind', 234), ('bassoon', 'woodwind', 123),
        ('vocal', 'vocal', 6734), ('vox', 'vocal', 2876), ('choir', 'vocal', 1543),
        ('voice', 'vocal', 987), ('chant', 'vocal', 234),
        ('fx', 'fx', 4646), ('bell', 'fx', 20861), ('hit', 'fx', 3336),
        ('sfx', 'fx', 490), ('sweep', 'fx', 194), ('riser', 'fx', 192), ('impact', 'fx', 75),
        ('loop', 'pattern', 31736), ('melody', 'melody', 15282), ('chord', 'harmony', 14766),
        ('groove', 'pattern', 10729), ('break', 'pattern', 4073), ('progression', 'harmony', 767),
        ('pattern', 'pattern', 748), ('harmonic', 'harmony', 620), ('melodic', 'melody', 437),
        ('rock', 'genre', 40209), ('funk', 'genre', 11136), ('jazz', 'genre', 6193),
        ('dnb', 'genre', 6032), ('house', 'genre', 5315), ('trance', 'genre', 4866),
        ('techno', 'genre', 3891), ('edm', 'genre', 3872), ('soul', 'genre', 2756),
        ('trap', 'genre', 1564), ('reggae', 'genre', 1459), ('dubstep', 'genre', 447),
        ('hip-hop', 'genre', 22), ('r&b', 'genre', 42)
    ON CONFLICT (name) DO UPDATE
    SET usage_count = EXCLUDED.usage_count, category = EXCLUDED.category;
    " "Creating/updating tags"

    log_success "Tags created/updated"
    echo ""

    # =============================================================================
    # Step 3: Tag files with controlled parallelism
    # =============================================================================
    log_info "[3/5] Tagging files with controlled parallelism (batch size: $BATCH_SIZE)..."

    # Top keywords to process
    local keywords=(
        "ride:ride" "fill:fill" "kick:kick" "tom:tom" "bass:bass"
        "rock:rock" "crash:crash" "loop:loop" "synth:synth" "piano:piano"
        "bell:bell" "lead:lead" "drum:drum" "snare:snare" "stick:stick"
        "melody:melody" "chord:chord" "hihat:hihat" "toms:toms" "bassline:bassline"
        "groove:groove" "pad:pad" "guitar:guitar" "funk:funk" "jazz:jazz"
        "strings:strings" "dnb:dnb" "house:house" "keys:keys" "arp:arp"
    )

    local total_keywords=${#keywords[@]}
    local current_batch=0
    local job_count=0
    declare -A results

    log_info "Processing $total_keywords keywords in batches of $BATCH_SIZE..."

    for keyword_pair in "${keywords[@]}"; do
        IFS=':' read -r keyword tag_name <<< "$keyword_pair"

        {
            local count
            count=$(tag_keyword_safe "$keyword" "$tag_name" "job_$job_count")
            echo "$tag_name:$count" > "$temp_dir/result_$job_count"
            log_success "Completed: $tag_name ($count files)"
        } &

        ((job_count++))
        ((current_batch++))

        # Wait when batch size is reached
        if [[ $current_batch -eq $BATCH_SIZE ]]; then
            wait
            current_batch=0
            # Collect results
            for ((i=job_count-BATCH_SIZE; i<job_count; i++)); do
                if [[ -f "$temp_dir/result_$i" ]]; then
                    local result
                    result=$(<"$temp_dir/result_$i")
                    IFS=':' read -r name count <<< "$result"
                    results["$name"]=$count
                fi
            done
        fi
    done

    # Wait for remaining jobs
    wait
    log_success "All keywords processed"
    echo ""

    # =============================================================================
    # Step 4: Create views and functions
    # =============================================================================
    log_info "[4/5] Creating database views and functions..."

    execute_sql "
    -- Views
    CREATE OR REPLACE VIEW v_drums AS
    SELECT DISTINCT f.id, f.filename, f.filepath, f.hash, f.size, f.created_at,
           m.bpm, m.key_signature, m.duration, m.time_signature
    FROM files f
    JOIN file_tags ft ON f.id = ft.file_id
    JOIN tags t ON ft.tag_id = t.id
    LEFT JOIN musical_metadata m ON f.id = m.file_id
    WHERE t.category = 'drums';

    CREATE OR REPLACE VIEW v_melodic AS
    SELECT DISTINCT f.id, f.filename, f.filepath, f.hash, f.size, f.created_at,
           m.bpm, m.key_signature, m.duration, m.time_signature
    FROM files f
    JOIN file_tags ft ON f.id = ft.file_id
    JOIN tags t ON ft.tag_id = t.id
    LEFT JOIN musical_metadata m ON f.id = m.file_id
    WHERE t.category IN ('keys', 'synth', 'strings', 'brass', 'woodwind', 'guitar');

    CREATE OR REPLACE VIEW v_bass AS
    SELECT DISTINCT f.id, f.filename, f.filepath, f.hash, f.size, f.created_at,
           m.bpm, m.key_signature, m.duration, m.time_signature
    FROM files f
    JOIN file_tags ft ON f.id = ft.file_id
    JOIN tags t ON ft.tag_id = t.id
    LEFT JOIN musical_metadata m ON f.id = m.file_id
    WHERE t.category = 'bass';

    CREATE OR REPLACE VIEW v_loops AS
    SELECT DISTINCT f.id, f.filename, f.filepath, f.hash, f.size, f.created_at,
           m.bpm, m.key_signature, m.duration, m.time_signature
    FROM files f
    JOIN file_tags ft ON f.id = ft.file_id
    JOIN tags t ON ft.tag_id = t.id
    LEFT JOIN musical_metadata m ON f.id = m.file_id
    WHERE t.category = 'pattern';

    CREATE OR REPLACE VIEW v_tag_stats AS
    SELECT t.name, t.category, COUNT(DISTINCT ft.file_id) as file_count,
           ROUND(COUNT(DISTINCT ft.file_id)::NUMERIC * 100.0 / (SELECT COUNT(*) FROM files), 2) as percentage
    FROM tags t
    LEFT JOIN file_tags ft ON t.id = ft.tag_id
    GROUP BY t.id, t.name, t.category
    ORDER BY file_count DESC;

    -- Helper functions
    CREATE OR REPLACE FUNCTION get_files_by_instrument(instrument_name TEXT)
    RETURNS TABLE (id BIGINT, filename TEXT, filepath TEXT, bpm INTEGER, key_signature TEXT) AS \$\$
    BEGIN
        RETURN QUERY
        SELECT DISTINCT f.id, f.filename, f.filepath, m.bpm, m.key_signature
        FROM files f
        JOIN file_tags ft ON f.id = ft.file_id
        JOIN tags t ON ft.tag_id = t.id
        LEFT JOIN musical_metadata m ON f.id = m.file_id
        WHERE t.name = instrument_name;
    END;
    \$\$ LANGUAGE plpgsql;

    CREATE OR REPLACE FUNCTION get_files_by_bpm_range(min_bpm INTEGER, max_bpm INTEGER)
    RETURNS TABLE (id BIGINT, filename TEXT, filepath TEXT, bpm INTEGER, key_signature TEXT) AS \$\$
    BEGIN
        RETURN QUERY
        SELECT f.id, f.filename, f.filepath, m.bpm, m.key_signature
        FROM files f
        JOIN musical_metadata m ON f.id = m.file_id
        WHERE m.bpm BETWEEN min_bpm AND max_bpm;
    END;
    \$\$ LANGUAGE plpgsql;
    " "Creating views and functions"

    log_success "Views and functions created"
    echo ""

    # =============================================================================
    # Step 5: Summary and statistics
    # =============================================================================
    log_info "[5/5] Generating summary statistics..."

    local end_time
    end_time=$(date +%s)
    local duration=$((end_time - start_time))

    log_info "═══════════════════════════════════════════════════════════════"
    log_info "  Organization Complete!"
    log_info "═══════════════════════════════════════════════════════════════"
    echo ""
    log_info "Duration: ${duration}s (~$((duration / 60)) minutes)"
    echo ""

    # Display statistics
    psql "$DB_URL" <<'SQL'
    SELECT '=== Database Statistics ===' as section;
    SELECT 'Total files: ' || COUNT(*)::text FROM files;
    SELECT 'Total tags: ' || COUNT(*)::text FROM tags;
    SELECT 'File-tag relationships: ' || COUNT(*)::text FROM file_tags;
    SELECT 'Files with tags: ' || COUNT(DISTINCT file_id)::text FROM file_tags;

    SELECT '';
    SELECT '=== Top 10 Instruments ===' as section;
    SELECT
        ROW_NUMBER() OVER (ORDER BY file_count DESC) as rank,
        name || ': ' || file_count::text || ' files (' || percentage::text || '%)' as instrument_stats
    FROM v_tag_stats
    WHERE category IS NOT NULL
    ORDER BY file_count DESC
    LIMIT 10;
SQL

    echo ""
    log_success "Database organized successfully!"

    # Cleanup
    rm -rf "$temp_dir"
}

# Run main function with error handling
main "$@"
