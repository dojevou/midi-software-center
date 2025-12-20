# Puppeteer Testing Guide - Critical Fixes Verification

This guide explains how to run automated Puppeteer tests to verify the 3 critical Tauri integration fixes.

## Prerequisites

1. **Start the development server:**
   ```bash
   cd /home/dojevou/projects/midi-software-center
   pnpm dev
   ```

   Wait for server to start at `http://localhost:5173`

2. **Ensure database is populated:**
   ```bash
   # Import test MIDI files if needed
   ./scripts/run-pipeline-ultra-fast.sh
   ```

## Test Files

### Playwright Automated Tests

**Location:** `e2e/critical-fixes.puppeteer.spec.ts`

**Runs:** Automated browser tests that verify:
- MIDI I/O commands registration
- Export functionality
- VIP3 to DAW file loading
- Integration workflows
- Visual regression screenshots

**Execute:**
```bash
# Run all critical fix tests
npx playwright test e2e/critical-fixes.puppeteer.spec.ts

# Run specific test suite
npx playwright test e2e/critical-fixes.puppeteer.spec.ts -g "MIDI I/O"
npx playwright test e2e/critical-fixes.puppeteer.spec.ts -g "Export"
npx playwright test e2e/critical-fixes.puppeteer.spec.ts -g "VIP3"

# Run with headed browser (see what's happening)
npx playwright test e2e/critical-fixes.puppeteer.spec.ts --headed

# Debug mode
npx playwright test e2e/critical-fixes.puppeteer.spec.ts --debug
```

### Manual Puppeteer MCP Testing

**Location:** `docs/PUPPETEER_MCP_TEST_SCRIPT.md`

**Purpose:** Step-by-step manual testing using Puppeteer MCP tools available in Claude Code

**When to use:** When you want to interactively test specific UI flows with Claude Code assistance

## Test Coverage

### Issue #1: MIDI I/O Commands (6 tests)
- ✅ midi_io_get_state command registration
- ✅ midi_io_detect_ports command works
- ✅ midi_io_add_port command works
- ✅ midi_io_update_port command works
- ✅ midi_io_remove_port command works
- ✅ midi_io_set_port_connected command works

### Issue #2: Export Functionality (3 tests)
- ✅ Export dialog opens without API errors
- ✅ Export uses correct `projectMidi()` API method
- ✅ Export validates file extensions (.mid/.midi)

### Issue #3: VIP3 to DAW Integration (3 tests)
- ✅ VIP3 browser opens without errors
- ✅ load_file_to_daw command is registered
- ✅ Loaded files appear in DAW track list

### Integration Tests (2 tests)
- ✅ VIP3 Load → Export workflow
- ✅ MIDI I/O configuration → Sequencer playback

### Visual Regression (3 screenshots)
- 📸 MIDI I/O Settings dialog
- 📸 Export dialog
- 📸 VIP3 Browser window

## Test Execution Workflow

### 1. Full Automated Test Run

```bash
# Terminal 1: Start dev server
cd /home/dojevou/projects/midi-software-center
pnpm dev

# Terminal 2: Run tests
cd /home/dojevou/projects/midi-software-center
npx playwright test e2e/critical-fixes.puppeteer.spec.ts --reporter=html

# View report
npx playwright show-report
```

### 2. CI/CD Integration

Add to `.github/workflows/test.yml`:

```yaml
- name: Run Puppeteer tests
  run: |
    pnpm dev &
    sleep 10
    npx playwright test e2e/critical-fixes.puppeteer.spec.ts
  env:
    CI: true
```

### 3. Manual Interactive Testing

See `PUPPETEER_MCP_TEST_SCRIPT.md` for step-by-step manual testing using Claude Code's Puppeteer MCP tools.

## Expected Results

### All Tests Passing

```
  ✓ Issue #1.1: MIDI I/O Get State command exists (2s)
  ✓ Issue #1.2: MIDI I/O Detect Ports command works (3s)
  ✓ Issue #1.3: MIDI I/O Update Port command works (2s)
  ✓ Issue #2.1: Export dialog opens without errors (2s)
  ✓ Issue #2.2: Export uses correct API method (3s)
  ✓ Issue #2.3: Export handles file extension validation (2s)
  ✓ Issue #3.1: VIP3 browser opens without errors (2s)
  ✓ Issue #3.2: load_file_to_daw command exists (3s)
  ✓ Issue #3.3: File loads to DAW track list (3s)
  ✓ Integration: VIP3 Load + Export workflow (5s)
  ✓ Integration: MIDI I/O + Sequencer playback (4s)

  17 passed (35s)
```

### Critical Failures (Should NOT Occur)

These errors indicate the fixes didn't work:

❌ **MIDI I/O Failures:**
```
Error: Command midi_io_get_state not found
Error: Command midi_io_detect_ports not found
```

❌ **Export Failures:**
```
TypeError: api.export.exportProject is not a function
Error: Command export_project_midi not found
```

❌ **VIP3 Integration Failures:**
```
Error: Command load_file_to_daw not found
```

## Debugging Failed Tests

### Check Browser Console

```bash
# Run with headed browser to see console
npx playwright test e2e/critical-fixes.puppeteer.spec.ts --headed
```

Look for:
- Red error messages
- "Command not found" errors
- API method errors

### Check Backend Logs

```bash
# Terminal running pnpm dev shows Tauri backend logs
# Look for command registration errors
```

### Verify Commands Registered

Check `app/src-tauri/src/main.rs` line ~325-345:

```rust
midi_app::commands::daw::midi_io::midi_io_get_state,
midi_app::commands::daw::midi_io::midi_io_detect_ports,
midi_app::commands::daw::midi_io::midi_io_add_port,
midi_app::commands::daw::midi_io::midi_io_update_port,
midi_app::commands::daw::midi_io::midi_io_remove_port,
midi_app::commands::daw::midi_io::midi_io_set_port_connected,
// ...
midi_app::commands::daw::sequencer::load_file_to_daw,
```

### Verify API Methods

Check `app/src/lib/api/commands.ts` line ~775:

```typescript
readonly export = {
  projectMidi: (outputPath: string) => invoke<void>(Commands.EXPORT_PROJECT_MIDI, { outputPath }),
};
```

## Performance Benchmarks

Expected test execution times:

| Test | Expected Duration |
|------|------------------|
| MIDI I/O tests | 1-3s each |
| Export tests | 2-4s each |
| VIP3 integration tests | 2-5s each |
| Integration tests | 4-6s each |
| Full suite | 30-40s |

## Screenshots Location

Visual regression screenshots saved to:
```
tests/screenshots/
├── midi-io-dialog.png
├── export-dialog.png
└── vip3-browser.png
```

Use these to verify UI rendering after changes.

## Continuous Testing

### Watch Mode

```bash
# Re-run tests on file changes
npx playwright test e2e/critical-fixes.puppeteer.spec.ts --watch
```

### Pre-commit Hook

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
echo "Running critical fix tests..."
pnpm dev &
DEV_PID=$!
sleep 10
npx playwright test e2e/critical-fixes.puppeteer.spec.ts --reporter=line
TEST_RESULT=$?
kill $DEV_PID
exit $TEST_RESULT
```

## Related Documentation

- [Manual Testing Checklist](MANUAL_TESTING_CHECKLIST.md) - Human-guided testing procedures
- [Tauri Integration Audit](TAURI_INTEGRATION_AUDIT_REPORT.md) - Original issue identification
- [Puppeteer MCP Test Script](PUPPETEER_MCP_TEST_SCRIPT.md) - Interactive testing guide

---

**Last Updated:** 2025-12-16
**Test Coverage:** 17 tests covering 3 critical fixes + 2 integration scenarios
