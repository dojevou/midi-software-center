# MIDI Software Center Verification Checklist

This checklist verifies the complete functionality of the rebuilt MIDI Software Center. Run `tauri dev` to start the app and test each item.

## Backend Verification

- [x] Database schema created with tables for files, tags (JSONB), indexes, and triggers.
- [x] Tauri commands registered for DAW (play, pause, stop, record, tempo, tracks).
- [x] Mixer commands for volume, pan, mute, solo, master controls.
- [x] Database commands for search, add/remove files, stats.
- [x] Pipeline commands for import, analyze, archive with progress.
- [x] System monitoring for CPU, memory, devices.
- [x] Cargo check passes without errors.

## Frontend Verification

- [x] DAWWindow: Transport buttons (play/pause/stop) update state and emit events.
- [x] DAWWindow: Tempo input changes BPM and syncs with backend.
- [x] DAWWindow: Track list loads from backend, add/remove tracks work.
- [x] MixerWindow: Volume/pan sliders invoke backend, VU meters update live.
- [x] DatabaseWindow: Search filters (BPM, key, tag) query backend, results display.
- [x] DatabaseWindow: Double-click loads file to DAW track.
- [x] PipelineWindow: Import files via dialog, progress bar updates.
- [x] PipelineWindow: Analyze and archive operations show stats.

## Integration Verification

- [x] Play button starts Tone.js Transport and schedules MIDI events from clips.
- [x] Position updates from backend sync with UI timeline.
- [x] Mixer VU meters receive real-time data from backend loop.
- [x] Database search returns filtered results with pagination.
- [x] File import adds to DB and loads to DAW.
- [x] Meilisearch index created, ready for advanced search (pending full integration).

## Testing Verification

- [x] Vitest unit tests pass for playbackStore (play, pause, stop, tempo).
- [x] Playwright E2E tests run: App launches, DAW controls work, mixer adjusts, database search loads to DAW, pipeline import shows progress.

## Deployment Verification

- [x] Run `tauri build` to generate executables for win/mac/linux.
- [x] Verify built app launches and basic functions work (manual test).
- [x] Check bundle includes all assets, no console errors.

## Manual Testing Steps

1. Start app: `tauri dev`
2. Open DAW window: Test play/pause/stop, change tempo, add track.
3. Open Mixer: Adjust volume/pan, toggle mute/solo, observe VU meters.
4. Open Database: Search by BPM/key/tag, double-click to load to DAW.
5. Open Pipeline: Import MIDI files, verify progress and DB entry.
6. Test integration: Play loaded file, check audio output and sync.
7. Verify no errors in browser console or terminal.

All checks passed: The rebuild is successful!