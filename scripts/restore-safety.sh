#!/bin/bash
# Restore normal PostgreSQL settings after LUDICROUS SPEED import

set -e

echo "🔄 Restoring Normal PostgreSQL Settings"
echo "═══════════════════════════════════════════════════════════════"
echo ""

docker exec midi-library-postgres psql -U midiuser -d midi_library <<'SQL'
-- Restore safe settings
ALTER SYSTEM SET synchronous_commit = 'on';
ALTER SYSTEM SET fsync = 'on';
ALTER SYSTEM SET full_page_writes = 'on';
ALTER SYSTEM SET wal_level = 'replica';
ALTER SYSTEM SET autovacuum = 'on';
ALTER SYSTEM SET track_counts = 'on';

-- Convert tables back to LOGGED (crash-safe)
ALTER TABLE files SET LOGGED;
ALTER TABLE musical_metadata SET LOGGED;
ALTER TABLE file_instruments SET LOGGED;
ALTER TABLE file_tags SET LOGGED;

SELECT pg_reload_conf();

SELECT 'Normal settings restored' as status;
SQL

# Re-enable swap
sudo swapon -a 2>/dev/null || true

echo "✓ PostgreSQL back to normal safe mode"
echo "✓ Tables converted back to LOGGED"
echo "✓ All safety features restored"
echo ""
echo "Next: Run ./scripts/rebuild-indexes.sh to restore indexes"
