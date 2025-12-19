# Puppeteer MCP Interactive Test Script

This guide shows how to use Claude Code's Puppeteer MCP tools to manually test the 3 critical fixes.

## Prerequisites

1. **Start the dev server in a terminal:**
   ```bash
   cd /home/dojevou/projects/midi-software-center
   pnpm dev
   ```

2. **Wait for server:** Ensure `http://localhost:5173` is accessible

3. **Have Claude Code ready:** You'll run Puppeteer commands through Claude Code's MCP tools

## Available Puppeteer MCP Tools

| Tool | Purpose |
|------|---------|
| `puppeteer_navigate` | Navigate to URL |
| `puppeteer_screenshot` | Take screenshot of page/element |
| `puppeteer_click` | Click element by CSS selector |
| `puppeteer_fill` | Fill input field |
| `puppeteer_select` | Select dropdown option |
| `puppeteer_hover` | Hover over element |
| `puppeteer_evaluate` | Execute JavaScript in browser |

## Test Script: Issue #1 - MIDI I/O Commands

### Step 1: Navigate to App

**Claude Code Command:**
```
Use puppeteer_navigate to go to http://localhost:5173
```

**Expected:** Page loads successfully (no connection refused error)

---

### Step 2: Take Initial Screenshot

**Claude Code Command:**
```
Use puppeteer_screenshot with name "app-initial" to capture the initial state
```

**Expected:** Screenshot saved showing the main app interface

---

### Step 3: Open MIDI I/O Settings

**Claude Code Command:**
```
Use puppeteer_evaluate to find and log all buttons:
document.querySelectorAll('button').forEach((btn, i) => console.log(`${i}: ${btn.textContent}`))
```

**Then:**
```
Use puppeteer_click with selector for MIDI/Settings button
(e.g., 'button:has-text("MIDI")' or specific selector from above)
```

**Expected:** MIDI I/O settings dialog opens

---

### Step 4: Screenshot MIDI Settings

**Claude Code Command:**
```
Use puppeteer_screenshot with name "midi-settings-dialog" to capture the dialog
```

**Expected:** Screenshot shows MIDI port list and controls

---

### Step 5: Verify No Console Errors

**Claude Code Command:**
```
Use puppeteer_evaluate to check console errors:
window.__consoleErrors = window.__consoleErrors || [];
console.log('Console errors:', window.__consoleErrors);
```

**Expected:** No "Command midi_io_get_state not found" errors

---

### Step 6: Test Refresh Ports Button

**Claude Code Command:**
```
Use puppeteer_click on 'button:has-text("Refresh")'
```

**Expected:** Port list refreshes without errors

---

### Step 7: Test Port Configuration Toggle

**Claude Code Command:**
```
Use puppeteer_evaluate to find checkboxes:
document.querySelectorAll('input[type="checkbox"]').forEach((cb, i) => console.log(`${i}: ${cb.id || cb.name}`))
```

**Then:**
```
Use puppeteer_click on first checkbox selector
```

**Expected:** Checkbox toggles, no "midi_io_update_port not found" error

---

## Test Script: Issue #2 - Export Functionality

### Step 1: Navigate to File Menu

**Claude Code Command:**
```
Use puppeteer_navigate to http://localhost:5173 (if needed)
```

---

### Step 2: Find Export Option

**Claude Code Command:**
```
Use puppeteer_evaluate to list menu options:
document.querySelectorAll('button, [role="menuitem"]').forEach((el, i) => {
  if (el.textContent.includes('Export') || el.textContent.includes('File')) {
    console.log(`${i}: ${el.textContent}`);
  }
});
```

---

### Step 3: Open Export Dialog

**Claude Code Command:**
```
Use puppeteer_click on 'button:has-text("Export")'
```

**Expected:** Export dialog opens without errors

---

### Step 4: Screenshot Export Dialog

**Claude Code Command:**
```
Use puppeteer_screenshot with name "export-dialog" encoded as true
```

**Expected:** Screenshot shows export format options and location picker

---

### Step 5: Verify Correct API Method

**Claude Code Command:**
```
Use puppeteer_evaluate to inject API spy:
const originalInvoke = window.__TAURI__?.invoke;
if (originalInvoke) {
  window.__TAURI__.invoke = async (cmd, ...args) => {
    console.log(`[API_CALL] ${cmd}`, args);
    return originalInvoke(cmd, ...args);
  };
}
console.log('API spy installed');
```

---

### Step 6: Fill Export Path

**Claude Code Command:**
```
Use puppeteer_fill on 'input[type="text"]' with value "/tmp/test_export.mid"
```

**Expected:** Path input updates

---

### Step 7: Attempt Export

**Claude Code Command:**
```
Use puppeteer_click on 'button:has-text("Export")', 'button:has-text("Confirm")'
```

**Expected:** Export initiates, console shows `[API_CALL] export_project_midi`

---

### Step 8: Verify No API Errors

**Claude Code Command:**
```
Use puppeteer_evaluate to check for API errors:
console.log('Checking for API errors...');
// Should NOT see "api.export.exportProject is not a function"
```

---

## Test Script: Issue #3 - VIP3 to DAW Loading

### Step 1: Navigate to VIP3 Browser

**Claude Code Command:**
```
Use puppeteer_navigate to http://localhost:5173
```

---

### Step 2: Open VIP3 Browser Window

**Claude Code Command:**
```
Use puppeteer_evaluate to find VIP3 button:
document.querySelectorAll('button').forEach((btn, i) => {
  if (btn.textContent.includes('VIP3') || btn.textContent.includes('Browser')) {
    console.log(`${i}: ${btn.textContent}`);
  }
});
```

**Then:**
```
Use puppeteer_click on VIP3/Browser button selector
```

**Expected:** VIP3 browser window opens

---

### Step 3: Screenshot VIP3 Browser

**Claude Code Command:**
```
Use puppeteer_screenshot with name "vip3-browser" to capture file list
```

**Expected:** Screenshot shows filtered MIDI file list

---

### Step 4: Install Command Spy

**Claude Code Command:**
```
Use puppeteer_evaluate:
window.__tauriCommands = [];
const originalInvoke = window.__TAURI__?.invoke;
if (originalInvoke) {
  window.__TAURI__.invoke = async (cmd, ...args) => {
    window.__tauriCommands.push({cmd, args, timestamp: Date.now()});
    console.log(`[TAURI_CMD] ${cmd}`, args);
    return originalInvoke(cmd, ...args);
  };
}
console.log('Command spy installed');
```

---

### Step 5: Select a File

**Claude Code Command:**
```
Use puppeteer_evaluate to find file items:
document.querySelectorAll('.file-item, [data-testid="file"]').forEach((file, i) => {
  console.log(`${i}: ${file.textContent.substring(0, 50)}`);
});
```

**Then:**
```
Use puppeteer_click on first file selector
```

---

### Step 6: Click "Load to DAW"

**Claude Code Command:**
```
Use puppeteer_click on 'button:has-text("Load")', 'button:has-text("Load to DAW")'
```

**Expected:** File loads without errors

---

### Step 7: Verify Command Invoked

**Claude Code Command:**
```
Use puppeteer_evaluate to check commands:
console.log('Commands invoked:', window.__tauriCommands);
const loadCommands = window.__tauriCommands.filter(c => c.cmd === 'load_file_to_daw');
console.log('load_file_to_daw calls:', loadCommands.length);
// Should be >= 1
```

---

### Step 8: Navigate to DAW View

**Claude Code Command:**
```
Use puppeteer_click on 'button:has-text("DAW")', 'button:has-text("Sequencer")'
```

**Expected:** DAW view shows loaded track

---

### Step 9: Verify Track Loaded

**Claude Code Command:**
```
Use puppeteer_evaluate to count tracks:
const tracks = document.querySelectorAll('.track-item, [data-testid="track"]');
console.log(`Tracks loaded: ${tracks.length}`);
// Should be >= 1
```

---

### Step 10: Screenshot Sequencer

**Claude Code Command:**
```
Use puppeteer_screenshot with name "sequencer-with-track" to capture loaded state
```

**Expected:** Screenshot shows track in sequencer

---

## Integration Test: Full Workflow

### Step 1: Reset to Initial State

**Claude Code Command:**
```
Use puppeteer_navigate to http://localhost:5173
```

---

### Step 2: Configure MIDI I/O

**Claude Code Command:**
```
Use puppeteer_click on MIDI settings button
Use puppeteer_click on "Send Clock" checkbox
Use puppeteer_click on dialog close button
```

---

### Step 3: Load File from VIP3

**Claude Code Command:**
```
Use puppeteer_click on VIP3 browser button
Use puppeteer_click on first file
Use puppeteer_click on "Load to DAW" button
```

---

### Step 4: Export Project

**Claude Code Command:**
```
Use puppeteer_click on Export button
Use puppeteer_fill on path input with "/tmp/integration_test.mid"
Use puppeteer_click on Export/Confirm button
```

---

### Step 5: Verify No Errors

**Claude Code Command:**
```
Use puppeteer_evaluate to check error log:
console.log('Final error check:', window.__consoleErrors || []);
// Should be empty or contain only non-critical errors
```

---

## Success Criteria

### ✅ Pass Indicators

1. **No "Command not found" errors** in console
2. **No "is not a function" TypeScript errors**
3. **All dialogs open successfully**
4. **API calls logged correctly:**
   - `midi_io_get_state`
   - `midi_io_update_port`
   - `export_project_midi`
   - `load_file_to_daw`
5. **Screenshots captured successfully** showing working UI
6. **Tracks appear in sequencer** after VIP3 load

### ❌ Fail Indicators

1. ❌ `Error: Command midi_io_get_state not found`
2. ❌ `Error: Command load_file_to_daw not found`
3. ❌ `TypeError: api.export.exportProject is not a function`
4. ❌ Dialogs crash or fail to open
5. ❌ No API calls logged
6. ❌ Tracks don't appear after loading

## Tips for Using Puppeteer MCP

### Finding Selectors

**Use evaluate to inspect:**
```javascript
// Find all buttons
document.querySelectorAll('button').forEach((btn, i) =>
  console.log(`${i}: ${btn.className} - ${btn.textContent}`)
);

// Find all inputs
document.querySelectorAll('input').forEach((inp, i) =>
  console.log(`${i}: ${inp.type} ${inp.id} ${inp.name}`)
);

// Find by data attribute
document.querySelectorAll('[data-testid]').forEach((el, i) =>
  console.log(`${i}: ${el.dataset.testid}`)
);
```

### Waiting for Elements

**Use evaluate with delays:**
```javascript
await new Promise(resolve => setTimeout(resolve, 2000));
console.log('Waited 2 seconds');
```

### Debugging Console Errors

**Install error listener:**
```javascript
window.__consoleErrors = [];
const originalError = console.error;
console.error = function(...args) {
  window.__consoleErrors.push(args.join(' '));
  originalError.apply(console, args);
};
console.log('Error listener installed');
```

---

**Last Updated:** 2025-12-16
**For:** Interactive manual testing of critical Tauri integration fixes
