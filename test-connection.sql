-- SQLTools Test Queries for MIDI Library Database
-- After connecting, run these queries to verify everything works

-- 1. Check database version and connection
SELECT
    version() as postgres_version,
    current_database() as database_name,
    current_user as connected_as;

-- 2. Verify all main tables exist and show row counts
SELECT
    schemaname as schema,
    tablename as table,
    n_live_tup as estimated_rows
FROM pg_stat_user_tables
ORDER BY n_live_tup DESC;

-- 3. Check VIP3 category data (should see real data, not mock)
SELECT 'Timbres' as category, COUNT(*) as count FROM timbres
UNION ALL
SELECT 'Styles', COUNT(*) FROM styles
UNION ALL
SELECT 'Articulations', COUNT(*) FROM articulations
UNION ALL
SELECT 'BPM Ranges', COUNT(*) FROM bpm_ranges
UNION ALL
SELECT 'Musical Keys', COUNT(*) FROM musical_keys
UNION ALL
SELECT 'Files', COUNT(*) FROM files
UNION ALL
SELECT 'Tags', COUNT(*) FROM tags
UNION ALL
SELECT 'Instruments (tags)', COUNT(*) FROM tags WHERE category = 'instrument';

-- 4. Sample timbres (verify real data)
SELECT id, name, description, sort_order
FROM timbres
ORDER BY sort_order
LIMIT 10;

-- 5. Sample styles (verify real data)
SELECT id, name, description, sort_order
FROM styles
ORDER BY sort_order
LIMIT 10;

-- 6. Sample articulations (verify real data)
SELECT id, name, description, sort_order
FROM articulations
ORDER BY sort_order
LIMIT 10;

-- 7. Check folders (should show real folder paths, not "Folder 1", "Folder 2")
SELECT DISTINCT parent_folder
FROM files
WHERE parent_folder IS NOT NULL
ORDER BY parent_folder
LIMIT 20;

-- 8. Check instruments from tags (should show real instrument names)
SELECT DISTINCT name
FROM tags
WHERE category = 'instrument' AND is_active = true
ORDER BY name
LIMIT 20;

-- 9. Sample files with all metadata
SELECT
    f.id,
    f.filename,
    f.parent_folder,
    f.manufacturer,
    mm.bpm,
    mm.key_signature
FROM files f
LEFT JOIN musical_metadata mm ON f.id = mm.file_id
LIMIT 10;
