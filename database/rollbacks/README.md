# Database Rollbacks

This directory contains rollback scripts for database migrations. Each rollback script
reverses the changes made by its corresponding migration.

## Directory Structure

```
rollbacks/
├── README.md                          # This file
├── 012_daw_features_rollback.sql      # Rollback for DAW features migration
└── [future rollbacks...]
```

## Naming Convention

Rollback scripts follow the naming pattern:
```
{migration_number}_{migration_name}_rollback.sql
```

## Usage

### Running a Rollback

**IMPORTANT: Always backup your database before running a rollback!**

```bash
# Create backup first
pg_dump "postgresql://midiuser:145278963@localhost:5433/midi_library" \
  > ~/backups/midi_library_$(date +%Y%m%d_%H%M%S).sql

# Run rollback
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" \
  -f database/rollbacks/012_daw_features_rollback.sql
```

### Verification

Each rollback script includes verification logic that:
1. Checks if the migration was applied
2. Drops all objects in reverse dependency order
3. Verifies all objects were successfully removed
4. Reports success or any remaining objects

### Rolling Back Multiple Migrations

To rollback multiple migrations, run them in **reverse order**:

```bash
# Example: Roll back migrations 012, 011, 010
psql $DB_URL -f database/rollbacks/012_daw_features_rollback.sql
psql $DB_URL -f database/rollbacks/011_enhanced_analysis_rollback.sql  # if exists
psql $DB_URL -f database/rollbacks/010_some_feature_rollback.sql       # if exists
```

## Rollback Script Structure

Each rollback script follows this structure:

1. **Pre-rollback verification** - Check if migration was applied
2. **Drop views** - Views must be dropped before dependent tables
3. **Drop triggers** - Remove update triggers
4. **Drop indexes** - Explicit index removal
5. **Drop tables** - In reverse dependency order (children before parents)
6. **Drop enum types** - After tables that use them
7. **Drop functions** - Only if not used by other objects
8. **Update migration tracking** - Remove from schema_migrations if applicable
9. **Post-rollback verification** - Confirm all objects removed

## Safety Features

- All rollbacks are wrapped in transactions (atomic operations)
- Verification queries check for remaining objects
- CASCADE is used to handle dependencies
- IF EXISTS prevents errors on missing objects
- Detailed NOTICE/WARNING messages for debugging

## Creating New Rollback Scripts

When creating a new migration, always create a corresponding rollback:

1. Copy an existing rollback as a template
2. Update the objects list to match your migration
3. Ensure proper dependency ordering (reverse of migration)
4. Test the rollback on a development database
5. Document any special considerations

## Rollback Availability

| Migration | Rollback | Status |
|-----------|----------|--------|
| 001_initial_schema.sql | N/A | Core schema, no rollback |
| 002-010 | N/A | Legacy migrations |
| 011_enhanced_analysis_json.sql | Pending | Simple ALTER TABLE additions |
| 012_daw_features.sql | ✅ Available | Full rollback script |

## Emergency Recovery

If a rollback fails partway through:

1. Check the error message for which object failed
2. Manually drop any remaining objects
3. Use `\d` in psql to list remaining tables
4. Use `\dT` in psql to list remaining types
5. Run VACUUM ANALYZE after cleanup

```sql
-- Manual cleanup example
DROP TABLE IF EXISTS problematic_table CASCADE;
DROP TYPE IF EXISTS problematic_type CASCADE;
VACUUM ANALYZE;
```

## Contact

For database issues, check the main project documentation at `CLAUDE.md`.
