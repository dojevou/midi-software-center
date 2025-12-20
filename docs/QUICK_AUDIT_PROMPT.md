# Quick Audit Prompts

Ready-to-use prompts for auditing Tauri + Svelte + Rust integrations. Copy and paste directly into Claude Code.

---

## 1. Full Audit (Detailed - Recommended for Most Projects)

```
Perform a complete integration audit of this Tauri application to verify all GUI controls are properly connected to the backend.

## Phase 1: Discover Components
1. Find all Svelte components in src/lib/components/
2. List each component file found

## Phase 2: Create Audit Table
For each component, create a table with these columns:
| Component | Interactive Element | Handler Function | API Call | Tauri Command | Backend Implementation | Status |

## Phase 3: Trace Each Element
For every button, input, slider, checkbox, or other interactive element:
1. Component Level: Identify the event handler
2. API Layer: Find the API function it calls
3. IPC Call: Verify it uses invoke() to call a Tauri command
4. Backend Command: Find the #[tauri::command] function
5. Registration: Confirm it's registered in .invoke_handler()
6. Implementation: Verify backend implements the feature

## Phase 4: Mark Status
✅ Verified working | ⚠️ Untested | ❌ Broken/missing | 🔍 Needs investigation

## Phase 5: Report
Provide:
- Summary stats (X components, Y elements, status breakdown)
- Complete audit table
- Critical issues (❌) with recommended fixes
- Warnings (⚠️) for untested integrations
- Recommendations for improvements
```

**When to use:** Major releases, onboarding, post-refactor, quarterly reviews

**Time estimate:** 2-4 hours for medium projects

---

## 2. Quick Audit (Minimal)

```
Perform a quick integration audit focusing on recent changes:

1. List recently modified Svelte components (from git diff or status)
2. For each changed component:
   - Find new or modified interactive elements
   - Verify invoke() calls match registered commands
3. Check if new commands are in invoke_handler()
4. Report any broken integrations

Provide:
- List of changed components
- Status: ✅ OK | ❌ Issues found
- Critical issues only (skip comprehensive table)
- Quick fix recommendations
```

**When to use:** Pre-release checks, PR reviews, quick validations

**Time estimate:** 30-60 minutes

---

## 3. User Journey Testing

```
Audit this Tauri application by testing key user journeys:

## Define Journeys
List 3-5 critical user workflows (e.g., "Import files", "Apply filters", "Export data")

## For Each Journey:
1. List all UI interactions in order
2. Trace each interaction through the stack:
   - Component → Handler → API → Command → Backend
3. Test the complete workflow manually
4. Mark status: ✅ Works | ❌ Broken | 🔍 Unclear

## Report:
- Journey completion status
- Broken steps with details
- UX issues encountered
- Recommendations

Focus on end-to-end functionality, not exhaustive element coverage.
```

**When to use:** Feature releases, UX validation, user acceptance testing

**Time estimate:** 1-2 hours per journey

---

## 4. Single Component Deep Dive

```
Perform a detailed audit of a single component:

Component: [COMPONENT_NAME].svelte

## Analysis:
1. List every interactive element in the component
2. For each element, trace the full stack:
   - Event handler → API function → invoke() call → Backend command → Implementation
3. Test each interaction manually
4. Check error states and edge cases
5. Verify loading/disabled states

## Create Table:
| Element | Handler | API | Command | Backend | Status | Notes |

## Report:
- Element count and status breakdown
- Issues found (with severity)
- Missing error handling
- UX concerns
- Recommended fixes
```

**When to use:** Debugging specific features, new component validation, focused fixes

**Time estimate:** 30 minutes per component

**Usage:** Replace `[COMPONENT_NAME]` with actual component name

---

## 5. Test Suite Generation

```
Based on this Tauri application's components, generate integration tests:

1. Scan all Svelte components for interactive elements
2. For each verified integration (UI → Backend):
   - Generate a Vitest frontend test
   - Generate a Rust backend test
   - Suggest E2E test scenarios

## Test Coverage:
- Happy path tests
- Error handling tests
- Edge case tests

## Output:
- Test files ready to create
- Test structure/organization
- Mock data suggestions
- Test priority ranking

Focus on critical user paths and high-risk areas.
```

**When to use:** After audit to improve test coverage, CI/CD setup, quality gates

**Time estimate:** 1-2 hours + implementation time

---

## Customization Tips

### Replace Paths
If your project structure differs:
- Change `src/lib/components/` to your component directory
- Adjust `src/lib/api/` to your API layer location

### Focus Areas
Add focus constraints to any prompt:
- "Focus only on components in src/lib/components/VIP3/"
- "Only audit admin panel components"
- "Prioritize payment-related features"

### Combine Approaches
Mix prompts for better results:
```
First do a Quick Audit (#2), then for any ❌ issues found, do a Single Component Deep Dive (#4) on those components.
```

### Iterate
After getting results:
```
Based on the audit report you just provided, fix the top 3 critical issues. Then re-audit those components to verify the fixes.
```

---

## Example Workflow

**New Project / Major Release:**
1. Start with **Full Audit (#1)**
2. Fix all ❌ critical issues
3. Run **Test Suite Generation (#5)** for coverage
4. Use **User Journey Testing (#3)** for final validation

**Regular Development:**
1. After each feature: **Single Component Deep Dive (#4)**
2. Before each release: **Quick Audit (#2)**
3. Monthly: **User Journey Testing (#3)** on critical flows

**Debugging:**
1. **Quick Audit (#2)** to identify problem area
2. **Single Component Deep Dive (#4)** on suspect component
3. Fix and verify with component-specific re-audit

---

## Status Legend Reference

| Icon | Meaning | Action Required |
|------|---------|-----------------|
| ✅ | Verified working - full chain traced | None - working as expected |
| ⚠️ | Untested - appears complete | Test manually when possible |
| ❌ | Broken/missing - integration gap | Fix immediately - blocks functionality |
| 🔍 | Needs investigation - unclear flow | Review code, clarify logic |

---

## Common Issues Quick Reference

**Frontend:**
- Missing `on:click={handler}` binding
- Handler function undefined
- Wrong parameter passing to API

**API Layer:**
- Command name typo
- Missing try-catch
- Wrong invoke() syntax

**Backend:**
- Command not in `invoke_handler()`
- Missing `#[tauri::command]`
- Type mismatch (String vs i32)
- Not public (`pub fn`)

**Integration:**
- Async/await issues
- State not updating (store)
- Race conditions
- Missing error handling

---

## Pro Tips

1. **Start small:** Test on 2-3 components before full audit
2. **Fix and re-audit:** After fixes, run audit again to verify
3. **Save reports:** Keep audit tables in docs/ for reference
4. **Automate checks:** Use quick audit in CI/CD
5. **Regular schedule:** Monthly full audits prevent drift
6. **Team involvement:** Share audit process with all devs
7. **Prioritize critical paths:** Focus on user-facing features first

---

**See Also:** `TAURI_INTEGRATION_AUDIT_TEMPLATE.md` for comprehensive methodology and customization guide.
