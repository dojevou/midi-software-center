# Path B: Real-World Deployment Validation Plan

**Date**: November 5, 2025
**Status**: Deployment in progress
**Goal**: Validate core library functionality with real MIDI data

---

## ✅ Pre-Deployment Checklist

- [x] Core library builds successfully (`cargo build -p midi-pipeline` - 0 errors)
- [x] PostgreSQL database running (port 5433)
- [x] Database schema ready (15 tables, 60+ indexes)
- [ ] Frontend dependencies installed (pnpm)
- [ ] Rust builds completed (pipeline + DAW)
- [ ] Database migrations applied
- [ ] Dev servers launching

---

## 🎯 Phase 1: Import Validation

**Objective**: Verify MIDI file import pipeline works end-to-end

### Test Data
- Source: Phase 9 dataset (1,603 real MIDI files)
- Location: `/path/to/midi/collection`
- Categories: Africa.zip, Asia.zip, 1200 Chords.zip

### Test Steps
1. Launch Pipeline UI on `http://localhost:5173`
2. Navigate to Import section
3. Select first batch (50-100 files) from test dataset
4. Click "Import"
5. Monitor progress in real-time

### Success Criteria
- ✅ Files upload without errors
- ✅ Progress indicator shows expected timing
- ✅ Database records created for each file
- ✅ No crashes or UI freezes
- ✅ Import completes in < 2 minutes (3,915 files/sec target)

### Expected Performance
- **Target**: 3,915 files/sec (from Phase 9)
- **Duration**: 50 files ≈ 12ms, 100 files ≈ 25ms

---

## 🎯 Phase 2: Analysis Validation

**Objective**: Verify BPM and key detection accuracy

### Test Data
- Subset: 20-30 files with known BPM/key
- Examples:
  - Chords library (files with consistent key signatures)
  - Beat-heavy tracks (clear BPM)
  - Complex pieces (layered analysis)

### Test Steps
1. From imported files, select 20 for analysis
2. Click "Analyze Selected"
3. Monitor analysis progress
4. Review results in file details

### Success Criteria
- ✅ Analysis completes without errors
- ✅ BPM detection: ±5% accuracy (typical for unknown files)
- ✅ Key detection: 85%+ accuracy for classical/jazz
- ✅ Analysis speed: < 2 min for 20 files
- ✅ Results stored correctly in database

### Expected Performance
- **Target**: 90.5 files/sec (from Phase 9)
- **Duration**: 20 files ≈ 220ms

### Verification Method
```
For known files:
  - Open file details
  - Check BPM field
  - Check key signature field
  - Verify confidence scores
```

---

## 🎯 Phase 3: DAW Integration Validation

**Objective**: Verify DAW can load files and perform playback operations

### Test Data
- Subset: 5-10 imported files
- Include: Different formats (0, 1, 2), different track counts

### Test Steps
1. Launch DAW UI on `http://localhost:5174`
2. Click "Load File"
3. Select from imported files
4. Verify file loads in sequencer
5. Test piano roll rendering
6. Test basic playback (if MIDI hardware available)

### Success Criteria
- ✅ Files load without errors
- ✅ Piano roll displays tracks correctly
- ✅ Track list shows proper channel count
- ✅ No memory leaks or crashes
- ✅ Database queries respond in < 8.2ms (Phase 9 target)

### Expected Performance
- **Target**: 8.2ms query latency
- **Query**: Load file metadata + track info + events

---

## 📊 Real-Time Metrics to Track

### Import Phase
- Files imported per second
- Peak memory usage
- Database insert time
- UI responsiveness

### Analysis Phase
- Files analyzed per second
- CPU utilization
- BPM accuracy (spot checks)
- Key detection accuracy (spot checks)

### DAW Phase
- File load time
- Piano roll render time
- Query response time
- Memory usage with large files (2000+ notes)

---

## 🚨 Potential Issues & Troubleshooting

| Issue | Resolution |
|-------|-----------|
| Import hangs | Check database connectivity, restart container |
| Analysis stuck | Monitor logs, might need to reduce batch size |
| DAW won't load files | Verify database has imported records, check migrations |
| Memory spike | Check for memory leak in MIDI parsing (monitor via Activity Monitor) |
| Slow queries | Run `ANALYZE` on database, check index usage |

---

## 📝 Reporting Template

After each phase, document:

```markdown
## Phase X Results

**Status**: ✅ PASS / ⚠️ PARTIAL / ❌ FAIL

**Metrics**:
- Performance: [actual vs. target]
- Success rate: [X/Y tests passed]
- Errors: [list any issues encountered]

**Details**:
[Include observations, timings, any anomalies]

**Next Steps**:
[What to test next or issues to resolve]
```

---

## ✨ Success Criteria (Overall)

All three phases pass:
- ✅ Import: 100% success, < 2min for 100 files
- ✅ Analysis: 100% completion, > 85% accuracy
- ✅ DAW: All files load, < 8.2ms queries

**If all phases pass**: System is production-ready for:
- Deployment to staging/production
- Real-world user testing
- Integration with larger datasets

---

## 🔄 If Issues Found

1. Document the issue with exact reproduction steps
2. Check logs: `docker-compose logs postgres`
3. Verify database state: `SELECT COUNT(*) FROM files;`
4. Check test output for specific error messages
5. If critical: Rollback, investigate, redeploy

---

**Estimated Total Time**: 30-60 minutes (builds + all 3 phases)

Start time: [TBD when builds complete]
