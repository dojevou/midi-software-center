# Project File Format Specification

**Version:** 1.0
**Extension:** `.mscproj` (MIDI Software Center Project)
**Format:** JSON
**Created:** 2025-12-17

## Overview

The MIDI Software Center project file format uses JSON for human-readable, text-based project serialization. Projects can be saved to:
- **Files**: `.mscproj` files for portability and version control
- **Database**: JSONB column in PostgreSQL for fast querying and indexing

## File Structure

### Top-Level Schema

```json
{
  "version": 1,
  "metadata": { /* ProjectMetadata */ },
  "settings": { /* MusicalSettings */ },
  "tracks": [ /* Array of Track */ ],
  "mixer": { /* MixerState (optional) */ },
  "automation": { /* AutomationLanes (optional) */ },
  "routing": { /* RoutingConfig (optional) */ },
  "transport": { /* TransportState */ },
  "markers": [ /* Array of Marker */ ],
  "tags": [ /* Array of String */ ],
  "user_data": { /* Custom key-value pairs */ }
}
```

## Schema Version

**Current Version:** `1`

The `version` field enables forward and backward compatibility:
- **Loading:** If `version > supported_version`, show a warning or error
- **Saving:** Always use the current version number
- **Migration:** Future versions may include migration tools

### Version History

| Version | Date | Changes |
|---------|------|---------|
| 1 | 2025-12-17 | Initial format specification |

## Detailed Schema

### ProjectMetadata

Basic project information and timestamps.

```json
{
  "name": "My Project",
  "description": "Optional description",
  "created_at": "2025-12-17T10:00:00Z",
  "updated_at": "2025-12-17T12:30:00Z",
  "author": "Artist Name",
  "app_version": "0.1.0"
}
```

**Fields:**
- `name` (string, required): Project name
- `description` (string, optional): Project description
- `created_at` (ISO 8601, required): Creation timestamp
- `updated_at` (ISO 8601, required): Last modification timestamp
- `author` (string, optional): Creator name
- `app_version` (string, required): Application version that created the file

### MusicalSettings

Global musical and audio settings.

```json
{
  "bpm": 120.0,
  "time_signature_numerator": 4,
  "time_signature_denominator": 4,
  "key_signature": "C",
  "ppqn": 480,
  "sample_rate": 44100,
  "bit_depth": 16
}
```

**Fields:**
- `bpm` (float, required): Tempo in beats per minute (20.0-400.0)
- `time_signature_numerator` (int, required): Time signature top number (1-32)
- `time_signature_denominator` (int, required): Time signature bottom number (1, 2, 4, 8, 16, 32)
- `key_signature` (string, optional): Key signature (e.g., "C", "G", "Am", "F#m")
- `ppqn` (int, default 480): Pulses per quarter note (24-960)
- `sample_rate` (int, default 44100): Sample rate in Hz
- `bit_depth` (int, default 16): Bit depth for audio

### Track

Individual track with MIDI data.

```json
{
  "id": 1,
  "name": "Piano",
  "track_number": 1,
  "channel": 0,
  "program_number": 0,
  "bank_msb": 0,
  "bank_lsb": 0,
  "muted": false,
  "solo": false,
  "armed": false,
  "volume": 1.0,
  "pan": 0.0,
  "color": "#3B82F6",
  "height": 100,
  "collapsed": false,
  "source_file_id": 12345,
  "clips": [ /* Array of Clip */ ],
  "notes": [ /* Array of Note */ ]
}
```

**Fields:**
- `id` (int, required): Unique track ID within project
- `name` (string, required): Track name
- `track_number` (int, required): Track position/order
- `channel` (int, required): MIDI channel (0-15)
- `program_number` (int, required): MIDI program/instrument (0-127)
- `bank_msb` (int, default 0): Bank select MSB (0-127)
- `bank_lsb` (int, default 0): Bank select LSB (0-127)
- `muted` (bool, default false): Track mute state
- `solo` (bool, default false): Track solo state
- `armed` (bool, default false): Record arm state
- `volume` (float, default 1.0): Volume (0.0-2.0, 1.0 = unity)
- `pan` (float, default 0.0): Pan (-1.0 = left, 0.0 = center, 1.0 = right)
- `color` (string, default "#3B82F6"): Track color (hex RGB)
- `height` (int, default 100): Track height in pixels
- `collapsed` (bool, default false): UI collapse state
- `source_file_id` (int, optional): Reference to file in database
- `clips` (array, default []): MIDI clips/regions
- `notes` (array, default []): Direct MIDI notes (if not using clips)

### Clip

MIDI clip/region within a track.

```json
{
  "id": 1,
  "name": "Intro",
  "color": "#3B82F6",
  "start_tick": 0,
  "duration_ticks": 1920,
  "source_file_id": 12345,
  "muted": false,
  "gain_db": 0.0,
  "source_start_tick": 0,
  "source_end_tick": 1920,
  "notes": [ /* Array of Note */ ]
}
```

**Fields:**
- `id` (int, required): Unique clip ID within project
- `name` (string, required): Clip name
- `color` (string, default "#3B82F6"): Clip color
- `start_tick` (int, required): Start position in ticks
- `duration_ticks` (int, required): Duration in ticks
- `source_file_id` (int, optional): Reference to source file
- `muted` (bool, default false): Clip mute state
- `gain_db` (float, default 0.0): Clip gain in dB
- `source_start_tick` (int, default 0): Offset in source file
- `source_end_tick` (int, optional): End offset in source file
- `notes` (array, default []): MIDI notes in clip

### Note

Individual MIDI note.

```json
{
  "pitch": 60,
  "velocity": 100,
  "start_tick": 0,
  "duration_ticks": 480,
  "channel": 0
}
```

**Fields:**
- `pitch` (int, required): MIDI pitch (0-127)
- `velocity` (int, required): Note velocity (1-127)
- `start_tick` (int, required): Start position in ticks
- `duration_ticks` (int, required): Duration in ticks
- `channel` (int, optional): MIDI channel override (0-15)

### MixerState (Optional)

Mixer configuration (placeholder for Stream B integration).

```json
{
  "master_gain_db": 0.0,
  "master_pan": 0.0,
  "tracks": {
    "1": {
      "gain_db": 0.0,
      "pan": 0.0,
      "effects": [
        {
          "id": 1,
          "effect_type": "reverb",
          "enabled": true,
          "params": {
            "room_size": 0.5,
            "damping": 0.3
          }
        }
      ]
    }
  },
  "buses": [],
  "sends": []
}
```

### AutomationLanes (Optional)

Automation data (placeholder for Stream D integration).

```json
{
  "1": [
    {
      "parameter": "volume",
      "mode": "read",
      "points": [
        {
          "time": 0.0,
          "value": 0.8,
          "curve": "linear"
        },
        {
          "time": 4.0,
          "value": 1.0,
          "curve": "linear"
        }
      ]
    }
  ]
}
```

### RoutingConfig (Optional)

Signal routing configuration.

```json
{
  "track_routing": {
    "1": 0,
    "2": 0
  },
  "sidechains": []
}
```

### TransportState

Playback transport settings.

```json
{
  "position": 0,
  "loop_enabled": false,
  "loop_start": 0,
  "loop_end": 1920,
  "playing": false,
  "recording": false
}
```

**Fields:**
- `position` (int, default 0): Current playhead position in ticks
- `loop_enabled` (bool, default false): Loop on/off
- `loop_start` (int, default 0): Loop start position in ticks
- `loop_end` (int, default 1920): Loop end position in ticks (4 bars at 480 ppqn)
- `playing` (bool, default false): Playback active (usually false when saved)
- `recording` (bool, default false): Recording active (usually false when saved)

### Marker

Timeline marker or region.

```json
{
  "name": "Verse 1",
  "position": 1920,
  "color": "#EAB308",
  "is_region": true,
  "duration": 3840
}
```

**Fields:**
- `name` (string, required): Marker name
- `position` (int, required): Position in ticks
- `color` (string, default "#EAB308"): Marker color
- `is_region` (bool, default false): Whether this is a region with duration
- `duration` (int, optional): Region duration in ticks (if is_region = true)

### Tags

User-defined tags for organization.

```json
["electronic", "draft", "unfinished"]
```

### UserData

Custom key-value pairs for extensibility.

```json
{
  "custom_field": "custom_value",
  "plugin_data": {
    "plugin_name": "settings"
  }
}
```

## Example Complete Project

```json
{
  "version": 1,
  "metadata": {
    "name": "My Song",
    "description": "Electronic track with drums and bass",
    "created_at": "2025-12-17T10:00:00Z",
    "updated_at": "2025-12-17T12:30:00Z",
    "author": "Producer Name",
    "app_version": "0.1.0"
  },
  "settings": {
    "bpm": 128.0,
    "time_signature_numerator": 4,
    "time_signature_denominator": 4,
    "key_signature": "Am",
    "ppqn": 480,
    "sample_rate": 44100,
    "bit_depth": 16
  },
  "tracks": [
    {
      "id": 1,
      "name": "Drums",
      "track_number": 1,
      "channel": 9,
      "program_number": 0,
      "bank_msb": 0,
      "bank_lsb": 0,
      "muted": false,
      "solo": false,
      "armed": false,
      "volume": 1.0,
      "pan": 0.0,
      "color": "#EF4444",
      "height": 100,
      "collapsed": false,
      "source_file_id": 12345,
      "clips": [
        {
          "id": 1,
          "name": "Drum Loop",
          "color": "#EF4444",
          "start_tick": 0,
          "duration_ticks": 7680,
          "source_file_id": 12345,
          "muted": false,
          "gain_db": 0.0,
          "source_start_tick": 0,
          "source_end_tick": null,
          "notes": []
        }
      ],
      "notes": []
    },
    {
      "id": 2,
      "name": "Bass",
      "track_number": 2,
      "channel": 0,
      "program_number": 33,
      "bank_msb": 0,
      "bank_lsb": 0,
      "muted": false,
      "solo": false,
      "armed": false,
      "volume": 0.85,
      "pan": 0.0,
      "color": "#3B82F6",
      "height": 100,
      "collapsed": false,
      "source_file_id": null,
      "clips": [],
      "notes": [
        {
          "pitch": 40,
          "velocity": 100,
          "start_tick": 0,
          "duration_ticks": 480,
          "channel": null
        },
        {
          "pitch": 43,
          "velocity": 95,
          "start_tick": 480,
          "duration_ticks": 480,
          "channel": null
        }
      ]
    }
  ],
  "transport": {
    "position": 0,
    "loop_enabled": false,
    "loop_start": 0,
    "loop_end": 7680,
    "playing": false,
    "recording": false
  },
  "markers": [
    {
      "name": "Intro",
      "position": 0,
      "color": "#EAB308",
      "is_region": true,
      "duration": 3840
    },
    {
      "name": "Drop",
      "position": 3840,
      "color": "#EF4444",
      "is_region": false,
      "duration": null
    }
  ],
  "tags": ["electronic", "wip"],
  "user_data": {}
}
```

## File Storage

### File Extension

**Standard Extension:** `.mscproj`

Alternative extensions for specific purposes:
- `.msctemplate` - Project templates
- `.mscproj.bak` - Backup files

### File Location

**Default Project Directory:** `~/Documents/MIDI Software Center/Projects/`

**Auto-save Location:** `~/Documents/MIDI Software Center/Projects/.autosave/`

### File Naming Conventions

- Use descriptive names: `my-song-v1.mscproj`
- Avoid special characters: `/`, `\`, `<`, `>`, `:`, `"`, `|`, `?`, `*`
- Use hyphens or underscores for spaces: `my-project.mscproj` or `my_project.mscproj`

## Database Storage

### Projects Table

Projects are stored in the `projects` table with JSONB column:

```sql
CREATE TABLE projects (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    bpm REAL NOT NULL DEFAULT 120.0,
    time_signature_numerator INTEGER NOT NULL DEFAULT 4,
    time_signature_denominator INTEGER NOT NULL DEFAULT 4,
    project_data JSONB NOT NULL,  -- Full project JSON stored here
    schema_version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
```

### Database vs. File Storage

| Feature | Database | File |
|---------|----------|------|
| **Indexing** | Fast queries by name, BPM, etc. | Full-text search required |
| **Portability** | Requires database export | Portable `.mscproj` files |
| **Version Control** | Requires custom versioning | Git-friendly JSON format |
| **Backup** | Database backup required | Simple file copy |
| **Sharing** | Export to file required | Direct file sharing |

**Recommendation:** Use both!
- Store projects in database for fast access and querying
- Export to `.mscproj` files for sharing, version control, and backup

## Versioning Strategy

### Schema Evolution

When adding new features:
1. **Backward Compatible:** Add optional fields with defaults
2. **Breaking Changes:** Increment version number

### Migration Tools

Future versions will include:
- `migrate_v1_to_v2()` - Upgrade old projects
- `downgrade_v2_to_v1()` - Export to older format (if possible)

### Validation

The `Project::validate()` method checks:
- Version compatibility
- BPM range (20-400)
- Time signature validity
- Unique track IDs
- Valid MIDI values (pitch, velocity, channel)

## Best Practices

### For Users

1. **Regular Saves:** Save frequently to avoid data loss
2. **Backups:** Keep backups of important projects
3. **Version Control:** Use Git for text-based version control
4. **Naming:** Use descriptive project names

### For Developers

1. **Optional Fields:** Use `#[serde(skip_serializing_if = "Option::is_none")]` for optional fields
2. **Default Values:** Provide sensible defaults for backward compatibility
3. **Validation:** Always validate on load with `Project::validate()`
4. **Pretty Print:** Use pretty JSON for human readability
5. **Testing:** Test serialization/deserialization with unit tests

## Compatibility

### Minimum Requirements

- **Rust:** serde, serde_json, chrono
- **PostgreSQL:** 12+ (for JSONB)
- **JSON Parser:** Any JSON 1.0 compatible parser

### Third-Party Tools

The JSON format can be read by:
- Any text editor
- JSON viewers (online or desktop)
- Python: `json.load()`
- JavaScript: `JSON.parse()`
- jq: Command-line JSON processor

## Future Enhancements

### Version 2 (Planned)

Potential additions:
- Audio clips/samples
- VST plugin state
- Video sync markers
- Tempo map (tempo changes)
- Time signature changes
- Lyrics/annotations
- Collaboration metadata

### Extension Points

The `user_data` field allows custom extensions without breaking compatibility.

## References

- [JSON Specification](https://www.json.org/)
- [ISO 8601 Date/Time Format](https://en.wikipedia.org/wiki/ISO_8601)
- [MIDI Specification](https://www.midi.org/specifications)
- [PostgreSQL JSONB](https://www.postgresql.org/docs/current/datatype-json.html)

## License

This specification is part of the MIDI Software Center project.

---

**Last Updated:** 2025-12-17
**Status:** Active (Version 1.0)
