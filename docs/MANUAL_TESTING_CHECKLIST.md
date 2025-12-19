# Manual Testing Checklist - Critical Fixes Verification

**Date:** 2025-12-16
**Fixes Tested:** MIDI I/O Commands, Export Functionality, VIP3→DAW Integration

---

## Pre-Testing Setup

### Build and Run
```bash
cd /home/dojevou/projects/midi-software-center/app/src-tauri
cargo build
cd ../..
pnpm dev
```

### Required Data
- [ ] Database populated with MIDI files
- [ ] At least 1 MIDI output device available (or virtual MIDI port)
- [ ] Test MIDI files in VIP3 browser

---

## Test Suite 1: MIDI I/O Commands (Issue #1)

**Component:** `MidiSyncControls.svelte`
**Commands Tested:** 6 MIDI I/O commands
**Backend:** `app/src-tauri/src/commands/daw/midi_io.rs`

### Test Case 1.1: Get MIDI I/O State
- [ ] **Action:** Open Settings → MIDI I/O Setup window
- [ ] **Expected:** Window opens without errors
- [ ] **Expected:** MIDI port list loads and displays
- [ ] **Verify:** Browser console shows no "command not found" errors
- [ ] **Backend Call:** `midi_io_get_state` command executes

**Pass Criteria:** ✅ Port list displays without errors

---

### Test Case 1.2: Detect/Refresh MIDI Ports
- [ ] **Action:** Click "Refresh Ports" or "Detect Ports" button
- [ ] **Expected:** Port list updates
- [ ] **Expected:** New ports appear if any connected
- [ ] **Verify:** No console errors
- [ ] **Backend Call:** `midi_io_detect_ports` command executes

**Pass Criteria:** ✅ Ports refresh successfully

---

### Test Case 1.3: Update Port - Send Clock
- [ ] **Action:** Toggle "Send Clock" checkbox for a MIDI port
- [ ] **Expected:** Checkbox state changes immediately
- [ ] **Expected:** Port settings update in backend
- [ ] **Verify:** Toggle back and forth works consistently
- [ ] **Backend Call:** `midi_io_update_port` with `send_clock` parameter

**Pass Criteria:** ✅ Send Clock toggles work without errors

---

### Test Case 1.4: Update Port - Send Transport
- [ ] **Action:** Toggle "Send Transport" checkbox for a MIDI port
- [ ] **Expected:** Checkbox state changes immediately
- [ ] **Expected:** Port settings persist
- [ ] **Verify:** No console errors
- [ ] **Backend Call:** `midi_io_update_port` with `send_transport` parameter

**Pass Criteria:** ✅ Send Transport toggles work without errors

---

### Test Case 1.5: Update Port - Display Name
- [ ] **Action:** Edit port display name or alias
- [ ] **Expected:** Name updates in UI
- [ ] **Expected:** Changes persist after refresh
- [ ] **Backend Call:** `midi_io_update_port` with `display_name` parameter

**Pass Criteria:** ✅ Port names update correctly

---

### Test Case 1.6: Add New Port
- [ ] **Action:** Click "Add Port" or "Create Virtual Port" button
- [ ] **Expected:** New port appears in list
- [ ] **Expected:** Port has default settings
- [ ] **Backend Call:** `midi_io_add_port` command executes

**Pass Criteria:** ✅ New ports can be added

---

### Test Case 1.7: Remove Port
- [ ] **Action:** Click "Remove" or "Delete" for a port
- [ ] **Expected:** Port removed from list
- [ ] **Expected:** Confirmation dialog appears (if applicable)
- [ ] **Backend Call:** `midi_io_remove_port` command executes

**Pass Criteria:** ✅ Ports can be removed

---

### Test Case 1.8: Set Port Connected Status
- [ ] **Action:** Connect/disconnect a MIDI port
- [ ] **Expected:** Connection status updates in UI
- [ ] **Expected:** Icon or indicator shows connected/disconnected state
- [ ] **Backend Call:** `midi_io_set_port_connected` command executes

**Pass Criteria:** ✅ Port connection status updates correctly

---

## Test Suite 2: Export Functionality (Issue #2)

**Component:** `ExportWindow.svelte`
**Command Tested:** `export_project_midi`
**Backend:** `app/src-tauri/src/commands/daw/export.rs`

### Test Case 2.1: Open Export Dialog
- [ ] **Action:** File → Export Project (or Export button)
- [ ] **Expected:** Export window opens
- [ ] **Expected:** Export format dropdown shows "MIDI"
- [ ] **Verify:** No console errors on window open

**Pass Criteria:** ✅ Export dialog opens successfully

---

### Test Case 2.2: Select Export Location
- [ ] **Action:** Click "Browse" or "Select Location" button
- [ ] **Expected:** File save dialog opens
- [ ] **Expected:** Can navigate to desired folder
- [ ] **Action:** Select location and confirm
- [ ] **Expected:** Path displays in export dialog

**Pass Criteria:** ✅ Export location can be selected

---

### Test Case 2.3: Export Project (No Tracks)
- [ ] **Action:** Clear all tracks from project
- [ ] **Action:** Click "Export" button
- [ ] **Expected:** Export dialog shows "Preparing export..."
- [ ] **Expected:** Export completes or shows appropriate message
- [ ] **Expected:** No application crash
- [ ] **Verify:** MIDI file created at selected location (check file exists)

**Pass Criteria:** ✅ Export handles empty project gracefully

---

### Test Case 2.4: Export Project (With Tracks)
- [ ] **Action:** Load 2-3 MIDI files into DAW tracks
- [ ] **Action:** Click "Export" button
- [ ] **Expected:** Export dialog shows progress: "Preparing export..." → "Exporting project to MIDI..."
- [ ] **Expected:** Export completes: "Export completed: [path]"
- [ ] **Expected:** Status changes to "completed" (green indicator)
- [ ] **Verify:** MIDI file created and is valid
- [ ] **Verify:** Open exported file in MIDI player/DAW to confirm

**Pass Criteria:** ✅ Export creates valid MIDI file

---

### Test Case 2.5: Export Progress Indicators
- [ ] **Action:** Export a project
- [ ] **Expected:** Progress bar or spinner displays
- [ ] **Expected:** Status message updates: "Preparing..." → "Exporting..." → "Completed"
- [ ] **Expected:** Progress value reaches 100%
- [ ] **Verify:** UI remains responsive during export

**Pass Criteria:** ✅ Progress feedback is clear and accurate

---

### Test Case 2.6: Export Error Handling
- [ ] **Action:** Select invalid export path (e.g., read-only location)
- [ ] **Action:** Click "Export"
- [ ] **Expected:** Error message displays: "Export failed: [error]"
- [ ] **Expected:** Status shows "error" (red indicator)
- [ ] **Expected:** Error details shown in dialog
- [ ] **Verify:** Application doesn't crash

**Pass Criteria:** ✅ Export errors are handled gracefully

---

### Test Case 2.7: Export File Extension Validation
- [ ] **Action:** Enter filename without .mid or .midi extension
- [ ] **Action:** Click "Export"
- [ ] **Expected:** Error message: "Output file must have .mid or .midi extension"
- [ ] **Expected:** Export does not proceed
- [ ] **Action:** Add .mid extension and retry
- [ ] **Expected:** Export succeeds

**Pass Criteria:** ✅ File extension validation works

---

## Test Suite 3: VIP3 to DAW Integration (Issue #3)

**Component:** `VIP3BrowserWindow.svelte`
**Command Tested:** `load_file_to_daw`
**Backend:** `app/src-tauri/src/commands/daw/sequencer.rs`

### Test Case 3.1: Open VIP3 Browser
- [ ] **Action:** Click "VIP3 Browser" button/menu item
- [ ] **Expected:** VIP3 browser window opens
- [ ] **Expected:** File list loads with filters
- [ ] **Verify:** No console errors

**Pass Criteria:** ✅ VIP3 browser opens and displays files

---

### Test Case 3.2: Browse and Filter Files
- [ ] **Action:** Click different filter categories (Timbre, Style, etc.)
- [ ] **Expected:** File list updates based on filter
- [ ] **Expected:** Filter counts update dynamically
- [ ] **Verify:** Filters work as expected

**Pass Criteria:** ✅ Filtering works correctly

---

### Test Case 3.3: Load Single File to DAW
- [ ] **Action:** Select a MIDI file in VIP3 browser
- [ ] **Action:** Click "Load to DAW" button (or double-click file)
- [ ] **Expected:** File loads without errors
- [ ] **Expected:** Success message or indicator appears
- [ ] **Backend Call:** `load_file_to_daw` command executes
- [ ] **Verify:** No console error: "command not found"

**Pass Criteria:** ✅ File loads without "command not found" error

---

### Test Case 3.4: Verify File Loaded in Sequencer
- [ ] **Action:** After loading file from VIP3
- [ ] **Action:** Switch to DAW/Sequencer view
- [ ] **Expected:** New track appears in track list
- [ ] **Expected:** Track name matches loaded file
- [ ] **Expected:** Track shows event count / note count
- [ ] **Verify:** Track ID is assigned correctly

**Pass Criteria:** ✅ Loaded file appears as new track in sequencer

---

### Test Case 3.5: Load Multiple Files
- [ ] **Action:** Load 3 different files from VIP3 browser
- [ ] **Expected:** Each file loads successfully
- [ ] **Expected:** Each creates a new track (3 tracks total)
- [ ] **Expected:** Tracks have sequential IDs
- [ ] **Verify:** All tracks visible in DAW

**Pass Criteria:** ✅ Multiple files can be loaded sequentially

---

### Test Case 3.6: Playback Loaded Track
- [ ] **Action:** Load a file from VIP3 to DAW
- [ ] **Action:** Switch to DAW view
- [ ] **Action:** Press Play button on transport
- [ ] **Expected:** Loaded track plays back
- [ ] **Expected:** MIDI notes are heard (if MIDI output configured)
- [ ] **Verify:** Track events are scheduled correctly

**Pass Criteria:** ✅ Loaded tracks play back correctly

---

### Test Case 3.7: Load File Properties
- [ ] **Action:** Load a file from VIP3
- [ ] **Action:** Check track properties in DAW
- [ ] **Expected:** Track channel is 0 (default)
- [ ] **Expected:** Track file_id matches VIP3 file ID
- [ ] **Expected:** Track has default volume/pan settings
- [ ] **Verify:** Track events loaded correctly

**Pass Criteria:** ✅ Loaded track has correct default properties

---

### Test Case 3.8: Error Handling - Invalid File
- [ ] **Action:** Attempt to load a corrupted/invalid MIDI file
- [ ] **Expected:** Error message displays
- [ ] **Expected:** No track created in DAW
- [ ] **Expected:** Application doesn't crash
- [ ] **Verify:** Error is descriptive

**Pass Criteria:** ✅ Invalid files are handled gracefully

---

## Integration Tests

### Integration Test 1: MIDI I/O + Sequencer Playback
- [ ] **Setup:** Configure MIDI output port with "Send Clock" enabled
- [ ] **Action:** Load file to DAW from VIP3
- [ ] **Action:** Start playback
- [ ] **Expected:** MIDI clock messages sent to configured port
- [ ] **Expected:** Transport messages sent if "Send Transport" enabled
- [ ] **Verify:** External MIDI device receives sync correctly

**Pass Criteria:** ✅ MIDI sync works with loaded tracks

---

### Integration Test 2: VIP3 Load + Export
- [ ] **Action:** Load 2 files from VIP3 browser to DAW
- [ ] **Action:** Export project to MIDI
- [ ] **Expected:** Export includes both loaded tracks
- [ ] **Expected:** Exported MIDI file is valid
- [ ] **Verify:** Re-import exported file to verify content

**Pass Criteria:** ✅ VIP3-loaded files can be exported

---

### Integration Test 3: Complete Workflow
- [ ] **Step 1:** Open VIP3 browser
- [ ] **Step 2:** Filter by style (e.g., "Jazz")
- [ ] **Step 3:** Load 2 jazz files to DAW
- [ ] **Step 4:** Configure MIDI output port
- [ ] **Step 5:** Enable "Send Clock" on output port
- [ ] **Step 6:** Play project
- [ ] **Step 7:** Export project to MIDI
- [ ] **Expected:** All steps complete without errors
- [ ] **Expected:** Final MIDI export is valid

**Pass Criteria:** ✅ End-to-end workflow completes successfully

---

## Browser Console Checks

### Console Error Verification
For ALL tests above, verify:
- [ ] No "command not found" errors
- [ ] No "undefined is not a function" errors
- [ ] No Tauri IPC errors
- [ ] No red error messages in console

**Critical Errors (Auto-Fail):**
- ❌ `Error: Command midi_io_get_state not found`
- ❌ `Error: Command load_file_to_daw not found`
- ❌ `TypeError: api.export.exportProject is not a function`

---

## Performance Checks

### MIDI I/O Performance
- [ ] Port list loads in < 1 second
- [ ] Port updates apply immediately (< 100ms)
- [ ] Refresh ports completes in < 2 seconds

### Export Performance
- [ ] Small project (1-2 tracks) exports in < 5 seconds
- [ ] Large project (10+ tracks) exports in < 30 seconds
- [ ] UI remains responsive during export

### VIP3 Load Performance
- [ ] Single file loads in < 1 second
- [ ] File appears in track list immediately
- [ ] Multiple files load without lag

---

## Test Summary Template

```
Date: _____________
Tester: _____________

Test Suite 1 (MIDI I/O): ___/8 tests passed
Test Suite 2 (Export):   ___/7 tests passed
Test Suite 3 (VIP3→DAW): ___/8 tests passed
Integration Tests:        ___/3 tests passed

Critical Issues Found: ___
Minor Issues Found: ___

Overall Status: [ ] PASS [ ] FAIL

Notes:
_____________________________________________
_____________________________________________
```

---

## Automated Test Execution

To run automated E2E tests (Playwright):
```bash
cd /home/dojevou/projects/midi-software-center
npx playwright test e2e/app.spec.ts
```

To run component tests (Vitest):
```bash
cd /home/dojevou/projects/midi-software-center/app
pnpm test
```

---

## Debugging Tips

### MIDI I/O Issues
- Check browser console for command errors
- Verify MIDI devices are connected
- Check backend logs: `cargo tauri dev` output

### Export Issues
- Verify output path exists and is writable
- Check file permissions
- Verify sequencer has tracks loaded

### VIP3 Load Issues
- Verify file exists in database
- Check file path is valid
- Verify MIDI file is not corrupted
- Check sequencer engine is initialized

---

**Last Updated:** 2025-12-16
**Related:** TAURI_INTEGRATION_AUDIT_REPORT.md
