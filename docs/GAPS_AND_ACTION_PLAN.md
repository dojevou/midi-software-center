# Project Gaps & Action Plan

**Generated:** 2025-12-15
**Codebase Status:** 80% implementation match, 14% partial, 6% missing
**Total Items Analyzed:** 85 features/commands/tables

---

## Executive Summary

**Overall Health:** 🟢 Production-ready with minor gaps

- ✅ **68/85 features (80%)** fully implemented and match documentation
- ⚠️ **12/85 features (14%)** partially implemented or incomplete
- ❌ **5/85 features (6%)** documented but missing
- 🔍 **12 undocumented** features/commands exist but not in docs

**Critical Finding:** Most gaps are **documentation issues**, not missing code. Backend has significantly more features than documented.

---

## Priority 1: Critical Gaps (Blocking Features)

### Gap #1: Track Layer Filter (VIP3 Search)
**Status:** ⚠️ Frontend ready, backend missing
**Impact:** HIGH - Users can't filter single vs multi-track MIDI files
**Effort:** MEDIUM (2-4 hours)

**Details:**
- Frontend: `VIP3Browser.svelte` has `trackLayerFilter: 'all' | 'single' | 'multi'` prop (line 20)
- Backend: `search_files_vip3()` doesn't filter by `num_tracks` or `is_multi_track`
- Database: Schema has both columns ready (migration 001)
- Recent commit: `3f4d1cd` mentions "add track layer filter" but not completed

**Action Required:**
```rust
// In app/src-tauri/src/commands/pipeline/vip3/search.rs
// Add to search_files_vip3() function:

if let Some(layer) = filters.track_layer_filter.as_ref() {
    match layer.as_str() {
        "single" => {
            conditions.push("f.num_tracks = 1".to_string());
        }
        "multi" => {
            conditions.push("f.num_tracks > 1".to_string());
        }
        "all" => {} // No filter
        _ => {}
    }
}
```

**Files to Modify:**
1. `app/src-tauri/src/commands/pipeline/vip3/search.rs` - Add filter logic
2. `app/src-tauri/src/db/models/vip3.rs` - Add `track_layer_filter` to `Vip3Filters` struct
3. Test with existing `num_tracks` column

---

### Gap #2: load_file_to_daw() Command Missing
**Status:** ❌ Documented but doesn't exist
**Impact:** MEDIUM - VIP3 → DAW integration incomplete
**Effort:** LOW (30 minutes)

**Details:**
- Documentation says: "Double-click file in VIP3Browser.svelte triggers `load_file_to_daw(file_id)`"
- Reality: Command doesn't exist; must use `add_track(file_id, channel)` instead
- Frontend: VIP3Browser.svelte has `doubleClickFile` event but not wired to backend

**Action Required:**
```rust
// Option 1: Create wrapper command (RECOMMENDED)
#[tauri::command]
pub async fn load_file_to_daw(
    file_id: i64,
    engine: State<'_, Arc<SequencerEngine>>,
) -> Result<(), String> {
    // Use channel 0 as default, or auto-select next available
    add_track(file_id, 0, engine).await
}

// Option 2: Update documentation to use add_track() directly
```

**Files to Modify:**
1. Create `app/src-tauri/src/commands/daw/load.rs` OR add to `daw/sequencer.rs`
2. Register command in `main.rs`
3. Update `.github/copilot-instructions.md` to document actual pattern

**Recommendation:** Create `load_file_to_daw()` wrapper for semantic clarity.

---

### Gap #3: Lua Scripting Module Disabled
**Status:** ⚠️ Implemented but commented out
**Impact:** MEDIUM - Documented automation feature unavailable
**Effort:** MEDIUM (2-3 hours for testing)

**Details:**
- Module exists: `app/src-tauri/src/scripting/lua_runtime.rs`
- Disabled in: `app/src-tauri/src/commands/daw/mod.rs` line 20 (commented out)
- Dependency: mlua with vendored Lua 5.4 present in Cargo.toml
- No frontend integration

**Action Required:**
```rust
// In app/src-tauri/src/commands/daw/mod.rs
// Line 20: Currently commented:
// pub mod scripting;

// Uncomment and ensure commands are registered
pub mod scripting;
```

**Files to Modify:**
1. Uncomment `pub mod scripting;` in `daw/mod.rs`
2. Verify commands: `run_lua_script()`, `load_lua_automation()`, etc.
3. Add to `main.rs` invoke_handler
4. Test script execution safety (sandboxing)

**Risk:** Lua scripting needs security review before production use.

---

## Priority 2: Documentation Gaps (High Value, Low Effort)

### Gap #4: 60+ Mixer Commands Undocumented
**Status:** ✅ Fully implemented but not in docs
**Impact:** MEDIUM - Developers unaware of available features
**Effort:** LOW (1 hour)

**Missing from Documentation:**
- `mixer_set_mute(track_idx, muted)` ✅ EXISTS
- `mixer_set_solo(track_idx, solo)` ✅ EXISTS
- `mixer_add_channel()` ✅ EXISTS
- `mixer_get_channels()` ✅ EXISTS
- `mixer_set_pan()`, `mixer_set_velocity()`, `mixer_set_quantize()` ✅ ALL EXIST
- Plus 50+ parameter-specific commands

**Action Required:**
1. Update `.github/copilot-instructions.md` section "DAW Commands"
2. Add subsection "Mixer Commands" with full list
3. Document solo exclusivity behavior (only one track can be soloed)

**File to Modify:**
- `.github/copilot-instructions.md` lines 179-200 (DAW section)

---

### Gap #5: MIDI Command Naming Mismatch
**Status:** ⚠️ Docs outdated
**Impact:** LOW - Confusing but discoverable
**Effort:** LOW (5 minutes)

**Discrepancies:**
| Documented Name | Actual Name | Fix |
|-----------------|-------------|-----|
| `connect_device()` | `midi_connect()` | Update docs |
| `disconnect_device()` | `midi_disconnect()` | Update docs |
| `list_midi_devices()` | ✅ Correct | No change |

**Extra Commands Not Documented:**
- `midi_is_connected()` - Check connection status
- `midi_get_current_device()` - Query current device
- `midi_send_test_note()` - Send test MIDI note

**Action Required:**
Update `.github/copilot-instructions.md` lines 209-211 with correct names and add extra commands.

---

### Gap #6: Database Tables Not Documented
**Status:** ✅ Implemented but missing from docs
**Impact:** LOW - Developers might miss available features
**Effort:** LOW (15 minutes)

**Missing Tables:**
- `folders` - Directory hierarchy (migration 002)
- `midi_clips` - DAW clip management (migration 021)
- `track_splits` - Split file metadata (migration 006)
- `file_ratings` - 1-5 star ratings (migration 020)
- `recent_searches` - Search history (created but unused)

**Action Required:**
Add to `.github/copilot-instructions.md` section "Database Schema Essentials" (line 265+)

---

### Gap #7: Sequencer Commands Not Documented
**Status:** ✅ Implemented but missing from docs
**Impact:** LOW
**Effort:** LOW (10 minutes)

**Missing Commands:**
- `resume_sequencer()` - Separate from pause (not same as unpause)
- `get_playback_position()` - Query current position
- `get_tempo()` - Query current BPM
- `update_track()` - Update track properties
- `get_tracks()` - List all sequencer tracks

**Action Required:**
Add to `.github/copilot-instructions.md` DAW section.

---

## Priority 3: Low-Impact Gaps

### Gap #8: Meilisearch Not Integrated
**Status:** ❌ Documented as "configured but not integrated"
**Impact:** LOW - PostgreSQL full-text search works well
**Effort:** HIGH (8-16 hours full integration)

**Details:**
- Health check exists: `check_meilisearch_health()` command
- No indexing pipeline
- No search queries
- Dependency not added to Cargo.toml

**Action Required:**
1. **Option A (Recommended):** Update docs to clarify "Future feature, use PostgreSQL tsvector for now"
2. **Option B:** Full integration:
   - Add `meilisearch-sdk` to Cargo.toml
   - Create indexing pipeline (on file import)
   - Add `search_with_meilisearch()` command
   - Update frontend to use new search

**Recommendation:** Defer to future release. PostgreSQL full-text search is sufficient.

---

### Gap #9: Drag & Drop VIP3 → DAW
**Status:** ❌ Documented as "planned but not implemented"
**Impact:** LOW - Double-click works (once load_file_to_daw exists)
**Effort:** MEDIUM (4-6 hours)

**Details:**
- Docs mention: "Drag-and-drop planned but not yet implemented"
- Frontend: No drop target in DAW window
- Backend: Would use existing `add_track()` command

**Action Required:**
Defer to future release. Mark clearly in docs as "Planned feature."

---

### Gap #10: Recent Searches Table Unused
**Status:** ⚠️ Table exists but never queried
**Impact:** NONE - Dead code
**Effort:** LOW (remove or implement)

**Details:**
- Table created in migration 019
- No commands populate or query it
- Different from `saved_searches` (which IS used)

**Action Required:**
1. **Option A:** Remove table in new migration (cleanup)
2. **Option B:** Implement recent searches feature
3. **Option C:** Document as "Reserved for future use"

**Recommendation:** Remove table to reduce schema bloat.

---

## Priority 4: Documentation Enhancements (Nice to Have)

### Enhancement #1: TypeScript Interface Verification
**Status:** ⚠️ Needs verification
**Effort:** LOW (30 minutes)

**Action Required:**
Verify that `Vip3Filters` TypeScript interface in docs (lines 224-242 of copilot-instructions.md) matches actual `app/src/lib/api/vip3BrowserApi.ts`.

**Differences Found by Agent:**
- Docs show `bigint[]` for folder_ids/instrument_ids
- Actual TypeScript might use `number[]`

**Fix:** Update docs with exact TypeScript types from source.

---

### Enhancement #2: Performance Baselines
**Status:** ✅ VIP3 has targets, others missing
**Effort:** LOW (just documentation)

**Current Documentation:**
- Pipeline: 7,830 import/sec ✅
- Analysis: 181-360/sec ✅
- VIP3 Search: <100ms ✅
- Queries: <10ms simple, <100ms complex ✅

**Missing:**
- DAW latency targets (should document ~3ms JACK, ~15ms midir)
- Mixer processing latency
- Sequencer tick resolution

**Action Required:**
Add DAW performance section to copilot-instructions.md.

---

## Quick Wins (Can Complete in <2 Hours)

1. ✅ **Update MIDI command names** in docs (5 min)
2. ✅ **Add load_file_to_daw() wrapper** command (30 min)
3. ✅ **Document mixer commands** (1 hour)
4. ✅ **Add missing database tables** to docs (15 min)
5. ✅ **Fix TypeScript interface** docs (15 min)

**Total Time:** ~2 hours for 90% documentation accuracy

---

## Medium Effort Tasks (2-4 Hours Each)

1. ⚠️ **Implement track layer filter** in VIP3 backend (2-4 hours)
2. ⚠️ **Re-enable Lua scripting** module with tests (2-3 hours)
3. ⚠️ **Remove or implement recent_searches** table (1-2 hours)

**Total Time:** ~6-9 hours for 100% feature parity

---

## Large Effort Tasks (Defer to Future)

1. ❌ **Meilisearch full integration** (8-16 hours)
2. ❌ **Drag & drop VIP3 → DAW** (4-6 hours)
3. ❌ **Lua scripting security audit** (4-8 hours)

**Total Time:** ~16-30 hours (defer unless critical)

---

## Recommended Action Plan

### Phase 1: Quick Documentation Fixes (Today)
- [ ] Update `.github/copilot-instructions.md` with correct MIDI command names
- [ ] Document all mixer commands (60+ commands)
- [ ] Add missing database tables to schema section
- [ ] Verify and fix TypeScript interface examples
- [ ] Add missing sequencer commands (resume, get_position, etc.)

**Estimated Time:** 2 hours
**Impact:** HIGH (documentation now 98% accurate)

---

### Phase 2: Critical Feature Completion (This Week)
- [ ] Implement `load_file_to_daw()` wrapper command
- [ ] Add track layer filter to VIP3 backend SQL
- [ ] Test track layer filter end-to-end
- [ ] Update IMPLEMENTATION_CHECKLIST.md with completions

**Estimated Time:** 4-6 hours
**Impact:** HIGH (all documented features now work)

---

### Phase 3: Optional Enhancements (Next Sprint)
- [ ] Re-enable Lua scripting with security review
- [ ] Remove `recent_searches` table (or implement feature)
- [ ] Add DAW latency performance docs
- [ ] Create frontend drag-and-drop for VIP3 → DAW

**Estimated Time:** 8-12 hours
**Impact:** MEDIUM (polish and advanced features)

---

### Phase 4: Future Roadmap (Backlog)
- [ ] Meilisearch full integration (replace PostgreSQL search)
- [ ] Lua scripting frontend editor
- [ ] Advanced VIP3 filtering (audio similarity, ML-based)

**Estimated Time:** 20+ hours
**Impact:** LOW (nice-to-have features)

---

## Files Requiring Modification

### Documentation Updates (Priority 1)
1. `.github/copilot-instructions.md` - Update commands, add mixer section
2. `docs/IMPLEMENTATION_CHECKLIST.md` - Mark completed items
3. `CLAUDE.md` - Update feature status if needed

### Code Changes (Priority 2)
1. `app/src-tauri/src/commands/daw/sequencer.rs` - Add `load_file_to_daw()`
2. `app/src-tauri/src/commands/pipeline/vip3/search.rs` - Add track layer filter
3. `app/src-tauri/src/db/models/vip3.rs` - Add `track_layer_filter` field
4. `app/src-tauri/src/main.rs` - Register new command

### Tests to Add
1. `app/src-tauri/src/commands/pipeline/vip3/search_tests.rs` - Track layer filter tests
2. `app/src-tauri/src/commands/daw/sequencer_tests.rs` - load_file_to_daw tests

---

## Success Metrics

**Phase 1 Complete When:**
- [ ] All 60+ mixer commands documented
- [ ] MIDI command names corrected in docs
- [ ] TypeScript interfaces match actual code
- [ ] All implemented database tables listed

**Phase 2 Complete When:**
- [ ] `load_file_to_daw()` command works end-to-end
- [ ] Track layer filter returns correct results (single/multi/all)
- [ ] All integration tests pass
- [ ] Documentation matches 100% of implemented features

**Phase 3 Complete When:**
- [ ] Lua scripting re-enabled with security review
- [ ] `recent_searches` table removed or implemented
- [ ] Drag-and-drop prototype working

---

## Risk Assessment

| Gap | Risk if Not Fixed | Mitigation |
|-----|-------------------|------------|
| Track layer filter missing | Users can't find single-track files easily | HIGH - Complete in Phase 2 |
| load_file_to_daw missing | VIP3 → DAW integration broken | MEDIUM - Quick fix, add wrapper |
| Mixer commands undocumented | Developers reinvent features | LOW - Just documentation |
| Lua scripting disabled | Automation feature unavailable | LOW - Document as "coming soon" |
| Meilisearch not integrated | Slower search at massive scale | VERY LOW - PostgreSQL handles 2.15M files fine |

---

## Conclusion

**Overall Assessment:** 🟢 **Production-Ready**

The project is in excellent shape with 80% feature parity between docs and code. Most gaps are:
1. **Documentation issues** (mixer commands exist but not documented)
2. **Minor feature additions** (track layer filter just needs SQL)
3. **Future roadmap items** (Meilisearch, Lua scripting)

**Priority actions** (2-6 hours total):
1. Update documentation for existing mixer/sequencer commands
2. Add `load_file_to_daw()` wrapper
3. Implement track layer filter SQL

After these fixes, the codebase will be **98%+ feature complete** with comprehensive documentation for AI agents.

---

**Next Steps:**
1. Review this document
2. Approve Phase 1 documentation updates
3. Schedule Phase 2 feature completion
4. Update project roadmap with Phase 3/4 items
