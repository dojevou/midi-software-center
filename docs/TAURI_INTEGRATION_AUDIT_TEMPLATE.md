# Tauri Integration Audit Template

## Overview

This template provides a comprehensive methodology for auditing Tauri + Svelte + Rust applications to verify all GUI controls are properly connected to backend implementations.

## Audit Methodology

### Phase 1: Component Discovery

**Objective:** Identify all Svelte components that contain interactive elements.

**Steps:**
1. Scan `src/lib/components/` for all `.svelte` files
2. List components by feature area/module
3. Prioritize components with user interactions

**Output:** Complete component inventory

### Phase 2: Audit Table Creation

**Objective:** Create structured tracking for each integration point.

**Table Format:**

| Component | Interactive Element | Handler Function | API Call | Tauri Command | Backend Implementation | Status |
|-----------|---------------------|------------------|----------|---------------|------------------------|--------|
| Component.svelte | Button/Input/Control | on:click handler | apiFunction() | invoke('command') | #[tauri::command] fn | ✅/⚠️/❌/🔍 |

**Columns Explained:**
- **Component:** Svelte component filename
- **Interactive Element:** Specific UI control (button, input, slider, etc.)
- **Handler Function:** Event handler in component (on:click, on:change, etc.)
- **API Call:** API layer function called by handler
- **Tauri Command:** IPC command name passed to invoke()
- **Backend Implementation:** Rust function with #[tauri::command]
- **Status:** Current state (see legend below)

**Status Legend:**
- ✅ **Verified Working:** Full chain traced and confirmed
- ⚠️ **Untested:** Chain appears complete but not manually verified
- ❌ **Broken/Missing:** Missing link in the chain
- 🔍 **Needs Investigation:** Unclear or complex flow

### Phase 3: Integration Tracing

**Objective:** Trace each interactive element through the entire stack.

**For Each Interactive Element:**

1. **Component Level (Svelte):**
   - Identify event handler (on:click, on:input, etc.)
   - Find handler function name
   - Check for error handling UI

2. **API Layer (TypeScript):**
   - Locate API file (e.g., `src/lib/api/*.ts`)
   - Find function called by component
   - Verify invoke() call with command name
   - Check error handling/try-catch

3. **IPC Call (Tauri):**
   - Confirm `invoke('command_name', params)` syntax
   - Verify parameter passing
   - Check response handling

4. **Backend Command (Rust):**
   - Find `#[tauri::command]` function
   - Verify function signature matches frontend call
   - Check parameter types
   - Confirm error handling (Result<T, E>)

5. **Command Registration:**
   - Verify command in `.invoke_handler()` in main.rs or lib.rs
   - Check command name matches exactly

6. **Implementation:**
   - Verify backend logic exists
   - Check database calls if applicable
   - Confirm business logic completeness

### Phase 4: Status Assessment

**Objective:** Categorize each integration point.

**Assessment Criteria:**

**✅ Verified Working:**
- All 6 levels traced successfully
- Command registered in invoke_handler
- Backend implementation complete
- Ideally: manual testing confirms it works

**⚠️ Untested:**
- Chain appears complete
- Not manually verified
- May have edge case issues

**❌ Broken/Missing:**
- Missing handler function
- invoke() calls non-existent command
- Command not registered
- Backend function missing
- Type mismatches

**🔍 Needs Investigation:**
- Complex conditional logic
- Multiple code paths
- Async/await complexity
- Shared state concerns

### Phase 5: Reporting

**Objective:** Generate actionable audit report.

**Report Structure:**

#### Executive Summary
- Total components audited: X
- Total interactive elements: Y
- Status breakdown:
  - ✅ Verified: X (%)
  - ⚠️ Untested: X (%)
  - ❌ Broken: X (%)
  - 🔍 Needs Investigation: X (%)

#### Complete Audit Table
[Full table from Phase 2]

#### Critical Issues (❌)
List all broken integrations with:
- Component and element
- What's missing/broken
- Impact on user
- Recommended fix

#### Warnings (⚠️)
List untested integrations with:
- Component and element
- Why it's untested
- Suggested test approach

#### Investigations Needed (🔍)
List complex areas with:
- Component and element
- What's unclear
- Questions to resolve

#### Recommendations
1. Priority fixes (critical issues)
2. Testing suggestions
3. Architecture improvements
4. Documentation needs

## Testing Approaches

### Full Audit (Comprehensive)

**When to Use:** Major releases, new team members, post-refactor

**Process:**
1. Audit every component
2. Trace every interactive element
3. Manual testing of critical paths
4. Generate complete report

**Time Estimate:** 2-4 hours for medium projects

### Quick Audit (Minimal)

**When to Use:** Regular check-ins, pre-release verification

**Process:**
1. Focus on components changed recently
2. Spot-check critical user flows
3. Verify new commands registered
4. Quick status assessment

**Time Estimate:** 30-60 minutes

### User Journey Testing

**When to Use:** Feature releases, UX validation

**Process:**
1. Define key user journeys (e.g., "Import MIDI files")
2. List all interactions in journey
3. Trace each interaction end-to-end
4. Test complete workflow

**Time Estimate:** 1-2 hours per journey

### Component Focus Audit

**When to Use:** Debugging specific feature, new component

**Process:**
1. Deep dive single component
2. Test every interactive element manually
3. Verify error states
4. Check edge cases

**Time Estimate:** 30 minutes per component

## Common Issues Checklist

### Frontend Issues
- [ ] Event handler not bound (missing `on:click={handler}`)
- [ ] Handler function not defined
- [ ] API function not imported
- [ ] Incorrect parameter passing to invoke()
- [ ] Missing error handling UI
- [ ] No loading states

### API Layer Issues
- [ ] Command name typo (frontend vs backend mismatch)
- [ ] Incorrect parameter serialization
- [ ] Missing try-catch blocks
- [ ] Wrong return type expectations

### Backend Issues
- [ ] Command not registered in invoke_handler
- [ ] Missing `#[tauri::command]` attribute
- [ ] Parameter type mismatch (e.g., String vs i32)
- [ ] Missing error handling (unwrap instead of Result)
- [ ] Function not public (`pub fn`)
- [ ] Command name doesn't match (snake_case vs camelCase)

### Integration Issues
- [ ] Async/await mismatch
- [ ] State not properly managed (stores not updated)
- [ ] Race conditions in async operations
- [ ] Missing database transactions
- [ ] Improper error propagation

## Customization Guide

### For Different Project Structures

**Multiple Frontend Frameworks:**
- Adjust Component Discovery to scan appropriate directories
- Modify table columns for framework-specific patterns

**Microservices Backend:**
- Add "Service" column to track which backend service
- Trace through API gateway/routing layer

**Multiple Windows/Views:**
- Group components by window/view
- Track window-specific commands separately

**Plugin Architecture:**
- Add "Plugin" column for plugin-based features
- Verify plugin registration

### For Different Team Sizes

**Solo Developer:**
- Focus on Critical Issues and Quick Audits
- Skip manual verification for obvious cases

**Small Team (2-5):**
- Full audit quarterly
- Quick audits before releases
- Peer review new components

**Large Team (5+):**
- Automated testing integration
- Continuous audit as PR requirement
- Dedicated QA for manual verification

## Example Outputs

### Example: Minimal Report

```
## Audit Summary
- Components: 12
- Interactive Elements: 47
- Status: ✅ 38 (81%) | ⚠️ 7 (15%) | ❌ 2 (4%)

## Critical Issues
1. FilterPanel.svelte - "Apply Filters" button calls `applyFilters()` but command not registered
2. ExportDialog.svelte - Export button missing handler function

## Recommendations
1. Register `apply_filters` command in main.rs invoke_handler
2. Add `handleExport()` function in ExportDialog.svelte
```

### Example: Detailed Component Entry

```
| FilterPanel.svelte | Apply Filters Button | handleApplyFilters() | vip3Api.applyFilters() | invoke('apply_vip3_filters') | apply_vip3_filters() | ❌ |

**Issue:** Command not registered in invoke_handler
**Fix:** Add to main.rs:
.invoke_handler(tauri::generate_handler![
    apply_vip3_filters,  // <-- Add this
    // ... other commands
])
```

## Automation Opportunities

### Script-Based Discovery
- Parse .svelte files for `on:` event handlers
- Parse API files for `invoke()` calls
- Parse Rust files for `#[tauri::command]`
- Generate initial audit table automatically

### CI/CD Integration
- Pre-commit hook: verify new commands registered
- PR checks: audit changed components
- Release gate: require full audit report

### Testing Integration
- Generate integration tests from audit table
- E2E tests for each verified interaction
- Mock backend commands for frontend testing

## Tips for Success

1. **Start Small:** Audit 2-3 components first to understand the pattern
2. **Fix As You Go:** Don't just document issues, fix them immediately
3. **Re-audit After Fixes:** Verify your fixes worked
4. **Keep Reports:** Track audit history to measure improvement
5. **Automate When Possible:** Reduce manual effort over time
6. **Make It Regular:** Schedule audits, don't wait for bugs
7. **Involve Team:** Make audit process collaborative
8. **Update Template:** Customize this template for your project specifics

## Related Documentation

- [Tauri Command Documentation](https://tauri.app/v1/guides/features/command)
- [Svelte Event Handling](https://svelte.dev/docs#template-syntax-element-directives-on-eventname)
- [TypeScript Invoke API](https://tauri.app/v1/api/js/tauri#invoke)

## Version History

- v1.0 (2024): Initial template creation
- Customizable for any Tauri + Svelte + Rust project

---

**Note:** This template is designed to be copied to any Tauri project and customized as needed. Remove sections that don't apply, add project-specific checks, and adapt the methodology to your team's workflow.
