-- Infer instrument metadata from folder paths for files missing instrument_names_text
-- This script analyzes the filepath/parent_folder to extract instrument information

-- Step 1: Update files in "Drum" folders
UPDATE files
SET instrument_names_text = ARRAY['drums']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ILIKE '%drum%kit%' OR
    filepath ILIKE '%/drum%' OR
    filepath ILIKE '%drums%' OR
    parent_folder ILIKE '%drum%'
  );

-- Step 2: Update files in "Bass" folders
UPDATE files
SET instrument_names_text = ARRAY['bass']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ILIKE '%bass%' OR
    parent_folder ILIKE '%bass%'
  );

-- Step 3: Update files in "Synth" folders
UPDATE files
SET instrument_names_text = ARRAY['synth']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ILIKE '%synth%' OR
    parent_folder ILIKE '%synth%'
  );

-- Step 4: Update files in "Lead" folders
UPDATE files
SET instrument_names_text = ARRAY['lead']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ILIKE '%lead%' OR
    parent_folder ILIKE '%lead%'
  );

-- Step 5: Update files in "Pad" folders
UPDATE files
SET instrument_names_text = ARRAY['pad']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ILIKE '%pad%' OR
    parent_folder ILIKE '%pad%'
  );

-- Step 6: Update files in "Keys/Piano" folders
UPDATE files
SET instrument_names_text = ARRAY['keys']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ILIKE '%keys%' OR
    filepath ILIKE '%piano%' OR
    filepath ILIKE '%keyboard%' OR
    parent_folder ILIKE '%keys%' OR
    parent_folder ILIKE '%piano%'
  );

-- Step 7: Update files in "Guitar" folders
UPDATE files
SET instrument_names_text = ARRAY['guitar']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ILIKE '%guitar%' OR
    parent_folder ILIKE '%guitar%'
  );

-- Step 8: Update files in "Vocal" folders
UPDATE files
SET instrument_names_text = ARRAY['vocal']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ILIKE '%vocal%' OR
    filepath ILIKE '%voice%' OR
    parent_folder ILIKE '%vocal%'
  );

-- Step 9: Update files in "String" folders
UPDATE files
SET instrument_names_text = ARRAY['strings']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ILIKE '%string%' OR
    parent_folder ILIKE '%string%'
  );

-- Step 10: Update files in "Brass" folders
UPDATE files
SET instrument_names_text = ARRAY['brass']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ILIKE '%brass%' OR
    filepath ILIKE '%horn%' OR
    filepath ILIKE '%trumpet%' OR
    parent_folder ILIKE '%brass%'
  );

-- Step 11: Update files in "Percussion" folders
UPDATE files
SET instrument_names_text = ARRAY['percussion']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ILIKE '%perc%' OR
    filepath ILIKE '%percussion%' OR
    parent_folder ILIKE '%perc%'
  );

-- Step 12: Update files in "Arp/Arpeggiated" folders
UPDATE files
SET instrument_names_text = ARRAY['arp']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ILIKE '%arp%' OR
    filepath ILIKE '%arpeggiat%' OR
    parent_folder ILIKE '%arp%'
  );

-- Step 13: Update files in "FX/Effects" folders
UPDATE files
SET instrument_names_text = ARRAY['fx']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ILIKE '%fx%' OR
    filepath ILIKE '%effect%' OR
    parent_folder ILIKE '%fx%'
  );

-- Step 14: Tag groove/fill patterns as drums (common naming convention)
UPDATE files
SET instrument_names_text = ARRAY['drums']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    parent_folder ILIKE '%groove%' OR
    parent_folder ILIKE '%fill%' OR
    parent_folder ILIKE '%beat%' OR
    parent_folder ILIKE '%loop%' OR
    filepath ILIKE '%groove%' OR
    filepath ILIKE '%fill%'
  );

-- Show statistics after inference
SELECT
    'Files with instruments (before)' as metric,
    316595 as count
UNION ALL
SELECT
    'Files with instruments (after)' as metric,
    COUNT(*) as count
FROM files
WHERE array_length(instrument_names_text, 1) > 0
UNION ALL
SELECT
    'Files still missing' as metric,
    COUNT(*) as count
FROM files
WHERE array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0
UNION ALL
SELECT
    'Improvement' as metric,
    COUNT(*) - 316595 as count
FROM files
WHERE array_length(instrument_names_text, 1) > 0;
