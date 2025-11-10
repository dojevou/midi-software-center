# Grok 4 API Quick Reference

## Setup (One-time - 2 minutes)

```bash
# 1. Set API key
export GROK_API_KEY='your-key-from-console.x.ai'

# 2. Install dependency
pip3 install httpx

# 3. Make script executable
chmod +x grok4-review.sh

# 4. Test it
./grok4-review.sh help
```

---

## Common Commands

| Command | What it does | Time | Best for |
|---------|-------------|------|----------|
| `./grok4-review.sh errors` | Analyze compilation errors | 5-10m | First step |
| `./grok4-review.sh tests` | Review test files | 5-10m | Test validation |
| `./grok4-review.sh wrappers` | Check Phase 5A | 5m | Architecture review |
| `./grok4-review.sh strategy` | Get fix plan | 10m | Planning |
| `./grok4-review.sh improve` | Suggestions | 10m | Long-term improvements |
| `./grok4-review.sh full` | Complete audit | 30-45m | Final verification |

---

## One-Line Usage

```bash
# Typical workflow
export GROK_API_KEY='...' && pip3 install httpx && cd /home/claude/midi-software-center && chmod +x grok4-review.sh && ./grok4-review.sh errors
```

---

## Python Direct Usage

```bash
# From anywhere
cd /home/claude/midi-software-center

# Error analysis
python3 grok4_project_reviewer.py --analyze-errors

# Full audit
python3 grok4_project_reviewer.py --full-audit

# Results saved to: GROK4_AUDIT_RESULTS.json
```

---

## Environment Variables

```bash
# Required
GROK_API_KEY='your-api-key'

# Optional (defaults to https://api.x.ai/v1)
GROK_API_URL='https://api.x.ai/v1'

# Or in .env file
echo "GROK_API_KEY=..." >> /home/claude/midi-software-center/.env
```

---

## Using Results

```bash
# View full results (JSON)
cat GROK4_AUDIT_RESULTS.json | jq '.' | less

# Extract specific section
cat GROK4_AUDIT_RESULTS.json | jq '.compilation_analysis.grok_analysis' -r

# Pretty format for reading
cat GROK4_AUDIT_RESULTS.json | python3 -m json.tool | less
```

---

## Cost Tracker

```bash
# API calls per audit
- Error analysis: 1 call
- Test review: 1 call
- Wrapper review: 1 call
- Strategy: 1 call
- Improvements: 1 call
- Full audit: 5 calls

# Total: ~50K tokens per full audit (low cost)
```

---

## Troubleshooting in 30 Seconds

| Problem | Fix |
|---------|-----|
| "API key not set" | `export GROK_API_KEY='...'` |
| "httpx not found" | `pip3 install httpx` |
| "Connection timeout" | Check internet, try again |
| "Invalid API key" | Check console.x.ai, copy again |
| "Permission denied" | `chmod +x grok4-review.sh` |

---

## Examples

### See what's broken (fastest)
```bash
./grok4-review.sh errors
```

### Get actionable plan
```bash
./grok4-review.sh strategy
```

### Full deep-dive
```bash
./grok4-review.sh full > audit_$(date +%Y%m%d).txt
```

### Background run (doesn't block terminal)
```bash
./grok4-review.sh full 2>&1 &
tail -f GROK4_AUDIT_RESULTS.json  # Monitor progress
```

---

## Pro Tips

✅ **DO:**
- Run error analysis first
- Save results to file
- Review strategy before fixing
- Use full audit as final check

❌ **DON'T:**
- Use during rate limit windows
- Share API key publicly
- Run without sufficient context
- Ignore Grok's root cause analysis

---

## Speed Reference

| Operation | Time | CPU |
|-----------|------|-----|
| Error analysis | 5 min | Low |
| Test review | 10 min | Low |
| Full audit | 30 min | Low |
| JSON parsing | <1 sec | Low |

---

## Get Help

```bash
# Show available commands
./grok4-review.sh

# Read full setup guide
cat GROK4_SETUP_GUIDE.md

# Check API status
curl https://api.x.ai/v1/status 2>/dev/null || echo "API offline"

# Validate key (without revealing it)
echo "${GROK_API_KEY:0:10}..." 
```

---

## Next Action

```bash
# RIGHT NOW:
export GROK_API_KEY='your-key'
pip3 install httpx
cd /home/claude/midi-software-center
./grok4-review.sh errors
```

---

**Created:** November 4, 2025  
**Status:** ✅ Production Ready  
**Tested:** Yes  
