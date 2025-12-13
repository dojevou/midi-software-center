# API Documentation - Quick Start

Welcome! This directory contains complete Tauri command documentation for MIDI Software Center.

## Start Here

1. **New to the API?** → Read [API-INDEX.md](API-INDEX.md)
   - Navigation guide
   - All 24 commands overview
   - 5 common workflows with code

2. **Need a specific command?** → Use [API-QUICK-REFERENCE.md](API-QUICK-REFERENCE.md)
   - Command lookup tables
   - TypeScript snippets
   - Response examples

3. **Want full details?** → See [API-COMMANDS.md](API-COMMANDS.md)
   - Complete signatures
   - Parameter descriptions
   - JSON examples
   - Performance notes

4. **Implementing/debugging?** → Check [API-SOURCE-MAP.md](API-SOURCE-MAP.md)
   - Source file locations
   - Function signatures
   - Database schema
   - Integration points

## Quick Links

| Document | Purpose | Best For |
|----------|---------|----------|
| [API-INDEX.md](API-INDEX.md) | Navigation & workflows | First time users |
| [API-QUICK-REFERENCE.md](API-QUICK-REFERENCE.md) | Fast lookup | Command reference |
| [API-COMMANDS.md](API-COMMANDS.md) | Full documentation | Comprehensive reference |
| [API-SOURCE-MAP.md](API-SOURCE-MAP.md) | Implementation details | Developers |

## Command Categories

### Pipeline (11 commands)
- **Import** (3): Single file, directory, archives
- **Analysis** (1): 10-point MIDI analysis
- **Splitting** (1): Multi-track separation
- **Search** (1): Keyword + filter search
- **Tags** (4): Tag management
- **Statistics** (4): Analytics

### DAW (13 commands)
- **Database** (1): File search
- **Sequencer** (8): Playback control
- **MIDI Device** (6): Hardware integration
- **System** (1): Database initialization

## Performance at a Glance

| Operation | Speed |
|-----------|-------|
| Import | 7,830 files/sec |
| Analysis | 181-360 files/sec |
| Search | <100ms |
| Tags | <50ms |

## Common Workflows

### Import and Analyze
```typescript
const imported = await invoke('import_directory', {
  directoryPath: '/path', recursive: true
});
const analyzed = await invoke('start_analysis');
```

### Search with Filters
```typescript
const results = await invoke('search_files', {
  query: 'kick',
  filters: { minBpm: 120, maxBpm: 130 },
  page: 1, pageSize: 50
});
```

### MIDI Hardware Setup
```typescript
await invoke('midi_connect', { deviceName: 'Akai Force' });
await invoke('start_sequencer');
await invoke('set_tempo', { bpm: 120 });
```

See [API-INDEX.md](API-INDEX.md) for more detailed examples.

## File Organization

```
docs/
├── README-API-DOCS.md ← You are here
├── API-INDEX.md (navigation + workflows)
├── API-QUICK-REFERENCE.md (fast lookup)
├── API-COMMANDS.md (complete reference)
└── API-SOURCE-MAP.md (implementation)
```

## Key Features

✅ **24 Commands Documented** - Pipeline + DAW, fully covered
✅ **TypeScript Examples** - 20+ real-world code snippets
✅ **Performance Metrics** - All operations benchmarked
✅ **Source Mapping** - Every command linked to source code
✅ **Database Integration** - Schema and query patterns
✅ **Error Handling** - Common errors and solutions
✅ **Architecture** - Design patterns explained

## Technology Stack

- **Backend:** Rust + Tauri 2.7
- **Database:** PostgreSQL 16
- **Frontend:** TypeScript + Svelte
- **IPC:** Tauri commands (JSON)

## Support

- Documentation: See the 4 files above
- Architecture: See `/docs/ARCHITECTURE-REFERENCE.md`
- Development: See `/docs/DEVELOPMENT-WORKFLOW.md`
- Project Guidance: See `/CLAUDE.md`

## Statistics

- **Total Documentation:** 1,606 lines
- **Commands:** 24 (11 Pipeline + 13 DAW)
- **Examples:** 20+ TypeScript snippets
- **Code Coverage:** 100% of Tauri commands

---

**Last Updated:** November 29, 2025
**Status:** Production Ready ✅
