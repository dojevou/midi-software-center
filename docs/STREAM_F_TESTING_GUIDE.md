# Stream F: Drag & Drop Testing Guide

## Overview
This guide covers testing the complete drag & drop functionality from VIP3 Browser and Collections to the DAW Sequencer.

## Prerequisites
- Backend `load_file_to_daw` command implemented and tested ✅
- Database populated with MIDI files
- Application running in development mode

## Test Scenarios

### 1. VIP3 Browser → Sequencer Drag & Drop

#### Test 1.1: Basic Single File Drag
**Steps:**
1. Open VIP3 Browser window (if not already open)
2. Navigate to the file list in VIP3Results
3. Click and hold on a MIDI file
4. Drag the file over to the Sequencer window
5. Drop the file onto an existing track or empty area

**Expected Results:**
- ✓ File shows custom drag ghost (blue badge with filename)
- ✓ Sequencer shows drop zone highlight when hovering
- ✓ Loading overlay appears with spinner
- ✓ Success toast notification: "Successfully loaded [filename]"
- ✓ File appears as clip in the sequencer
- ✓ Clip is positioned at the drop location

**Performance:**
- Total load time < 200ms
- UI remains responsive during load

#### Test 1.2: Drag to Empty Area (New Track)
**Steps:**
1. Drag a file from VIP3 to the empty area below existing tracks
2. Drop the file

**Expected Results:**
- ✓ New track created with file name
- ✓ Clip added to the new track
- ✓ Success notification appears

#### Test 1.3: Drag to Existing Track
**Steps:**
1. Drag a file to an existing track
2. Drop at different horizontal positions

**Expected Results:**
- ✓ Clip added to target track
- ✓ Clip positioned according to drop X coordinate
- ✓ Snap to grid works (if enabled)

#### Test 1.4: Visual Feedback
**Steps:**
1. Start dragging a file
2. Observe visual changes

**Expected Results:**
- ✓ Dragged file item shows reduced opacity (0.4)
- ✓ Cursor changes to 'grabbing'
- ✓ Custom drag image shows file icon and name
- ✓ Sequencer shows drag-over state

### 2. Collections → Sequencer Drag & Drop

#### Test 2.1: Drag from Collection
**Steps:**
1. Open Collections panel
2. Open a collection with files
3. Drag a file from the collection to the Sequencer

**Expected Results:**
- ✓ Same behavior as VIP3 Browser drag
- ✓ File loads successfully
- ✓ Visual feedback identical to VIP3
- ✓ Success notification appears

#### Test 2.2: Collection Visual States
**Steps:**
1. Hover over collection file items
2. Start dragging a file

**Expected Results:**
- ✓ Hover state changes background color
- ✓ Cursor changes to 'grab' on hover
- ✓ Cursor changes to 'grabbing' while dragging
- ✓ Dragged item shows reduced opacity

### 3. Error Handling

#### Test 3.1: Invalid File
**Steps:**
1. Modify database to point to non-existent file
2. Try to drag and drop the file

**Expected Results:**
- ✓ Loading overlay appears
- ✓ Error toast notification: "Failed to load [filename]: [error]"
- ✓ No clip created in sequencer
- ✓ UI remains stable

#### Test 3.2: Database Connection Issue
**Steps:**
1. Stop database temporarily
2. Try to drag and drop a file

**Expected Results:**
- ✓ Error toast with descriptive message
- ✓ Application doesn't crash
- ✓ Can retry after database restored

#### Test 3.3: Corrupted MIDI File
**Steps:**
1. Add a corrupted MIDI file to database
2. Try to drag and drop

**Expected Results:**
- ✓ Error toast with MIDI parsing error
- ✓ Graceful failure

### 4. Performance Tests

#### Test 4.1: Rapid Multiple Drags
**Steps:**
1. Drag 5 files quickly one after another
2. Drop each in sequence

**Expected Results:**
- ✓ Each file loads without blocking UI
- ✓ Loading overlays queue properly
- ✓ Toast notifications don't overlap excessively
- ✓ All files load successfully

#### Test 4.2: Large File
**Steps:**
1. Drag a large MIDI file (>1MB, many events)
2. Drop into sequencer

**Expected Results:**
- ✓ Loading indicator shows progress
- ✓ UI doesn't freeze
- ✓ File loads successfully
- ✓ Performance remains < 200ms total

### 5. Toast Notification System

#### Test 5.1: Success Notifications
**Steps:**
1. Successfully drag and drop 3 files

**Expected Results:**
- ✓ Green toast with ✓ icon appears for each
- ✓ Toasts stack vertically (bottom-right)
- ✓ Each toast auto-dismisses after 3 seconds
- ✓ Can manually dismiss by clicking toast

#### Test 5.2: Error Notifications
**Steps:**
1. Trigger an error (invalid file, etc.)

**Expected Results:**
- ✓ Red toast with ✕ icon appears
- ✓ Stays visible for 5 seconds (longer than success)
- ✓ Can be manually dismissed
- ✓ Error details visible in message

#### Test 5.3: Toast Interactions
**Steps:**
1. Generate multiple toasts
2. Click on individual toasts
3. Hover over toasts

**Expected Results:**
- ✓ Clicking dismisses the specific toast
- ✓ Hovering slightly lifts the toast (transform)
- ✓ Close button (×) is visible and functional
- ✓ Toasts don't interfere with each other

### 6. Integration Tests

#### Test 6.1: End-to-End Workflow
**Steps:**
1. Filter files in VIP3 Browser (by BPM, key, etc.)
2. Drag filtered result to Sequencer
3. Add to collection
4. Drag from collection to Sequencer at different position
5. Play back the sequence

**Expected Results:**
- ✓ All steps complete successfully
- ✓ MIDI data loads correctly
- ✓ Playback works with dropped files
- ✓ No data corruption

#### Test 6.2: Multi-Window Layout
**Steps:**
1. Open VIP3 Browser in one area
2. Open Sequencer in another area
3. Open Collections in a third area
4. Drag files between all windows

**Expected Results:**
- ✓ Drag works across window boundaries
- ✓ Drop zones detect properly
- ✓ No visual glitches

### 7. Accessibility

#### Test 7.1: Keyboard Navigation
**Steps:**
1. Tab through file list
2. Use Enter/Space to "activate" drag
3. Arrow keys to navigate drop position

**Expected Results:**
- ✓ Files are keyboard focusable
- ✓ Visual focus indicator visible
- ✓ Keyboard drag alternative works (if implemented)

#### Test 7.2: Screen Reader
**Steps:**
1. Enable screen reader
2. Navigate to file list
3. Start drag operation

**Expected Results:**
- ✓ File items announced with proper labels
- ✓ Drag state announced
- ✓ Drop success/failure announced

## Manual Testing Checklist

### Pre-Test Setup
- [ ] Database running and populated
- [ ] Application compiled and running
- [ ] VIP3 Browser window open
- [ ] Sequencer window open
- [ ] Collections panel accessible

### VIP3 Browser Drag Tests
- [ ] Single file drag to empty area
- [ ] Single file drag to existing track
- [ ] Visual feedback during drag (ghost, opacity)
- [ ] Loading overlay appears
- [ ] Success toast notification
- [ ] Clip appears in sequencer
- [ ] Multiple files in sequence

### Collections Drag Tests
- [ ] Drag from collection viewer
- [ ] Visual feedback identical to VIP3
- [ ] Success/error handling
- [ ] Collection items remain draggable

### Error Handling Tests
- [ ] Invalid file ID
- [ ] Database connection error
- [ ] Corrupted MIDI file
- [ ] Error toasts display correctly

### Toast System Tests
- [ ] Success toasts (green, ✓)
- [ ] Error toasts (red, ✕)
- [ ] Auto-dismiss timers work
- [ ] Manual dismiss works
- [ ] Toast stacking correct
- [ ] Toast animations smooth

### Performance Tests
- [ ] Single file < 200ms
- [ ] Rapid multiple drags
- [ ] Large file handling
- [ ] UI responsiveness maintained

### Integration Tests
- [ ] Full workflow (filter → drag → play)
- [ ] Multi-window drag operations
- [ ] Data integrity maintained

## Automated Test Commands

```bash
# Run backend integration tests
cd app/src-tauri
cargo test --test test_load_file_to_daw -- --ignored

# Run frontend component tests (if available)
cd ../../
npm run test

# Run E2E tests (if available)
npm run test:e2e
```

## Performance Benchmarks

### Target Metrics
- **Load Time:** < 200ms (total from drop to clip visible)
- **Backend:** ~20-65ms (database + MIDI parsing)
- **Frontend:** < 135ms (IPC + UI update)
- **Toast Display:** < 10ms
- **Drag Response:** < 16ms (60fps)

### Measuring Performance
1. Open browser DevTools
2. Go to Performance tab
3. Start recording
4. Perform drag & drop
5. Stop recording
6. Verify timing marks:
   - `dragstart` → `drop`: User drag duration
   - `drop` → `loadFileToDaw` call: Frontend prep
   - `loadFileToDaw` → response: Backend time
   - Response → clip visible: UI update time

## Troubleshooting

### Issue: Drag doesn't start
- Check draggable="true" attribute
- Verify event handlers bound
- Check browser console for errors

### Issue: Drop doesn't work
- Verify drop zone has proper handlers
- Check dataTransfer format matches
- Ensure preventDefault() called

### Issue: Loading overlay doesn't show
- Check isLoadingFile state updates
- Verify binding in template
- Check z-index conflicts

### Issue: Toast doesn't appear
- Verify Toast component in App.svelte
- Check toastStore import
- Inspect $toastStore reactive value

### Issue: Backend fails silently
- Check browser console
- Check Tauri backend logs: `tauri dev`
- Verify database connection
- Test backend directly with manual test script

## Success Criteria

All of these must pass:
- ✅ VIP3 files draggable with visual feedback
- ✅ Collection files draggable with visual feedback
- ✅ Sequencer accepts drops at correct positions
- ✅ Backend loads MIDI successfully (<200ms)
- ✅ Success toasts appear for successful loads
- ✅ Error toasts appear for failures
- ✅ Loading overlay shows during operation
- ✅ No console errors during normal operation
- ✅ UI remains responsive throughout
- ✅ Multi-file operations work correctly

## Related Documentation

- Backend Implementation: `docs/STREAM_F_BACKEND_COMPLETE.md`
- Quick Reference: `docs/STREAM_F_QUICK_REFERENCE.md`
- Integration Tests: `app/src-tauri/tests/test_load_file_to_daw.rs`
- Manual Test Script: `scripts/test-load-file-to-daw.sh`

## Reporting Issues

When reporting issues, include:
1. Test scenario being performed
2. Expected vs actual behavior
3. Browser console output
4. Tauri backend logs
5. Screenshots/recordings if visual issue
6. Database state (file count, sample IDs)
7. Performance metrics (if performance issue)
