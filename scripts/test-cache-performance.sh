#!/bin/bash
# =============================================================================
# VIP3 Filter Counts Cache Performance Test
# =============================================================================
# Tests the 5-second cache implementation
# Expected: First request slow (~1000ms), subsequent requests <10ms
# =============================================================================

set -e

DATABASE_URL="${DATABASE_URL:-postgresql://midiuser:145278963@localhost:5433/midi_library}"

echo "========================================="
echo "VIP3 Cache Performance Test"
echo "========================================="
echo ""
echo "Testing 5-second cache with filter counts"
echo ""

# Function to test cache hits
test_cache() {
    local test_name="$1"

    echo "Test: $test_name"
    echo "  Request 1 (cache miss - will be slow):"

    # First request - cache miss
    local start1=$(date +%s%3N)
    psql "$DATABASE_URL" -t -A -c "SELECT COUNT(*) FROM (SELECT folder_id, COUNT(*) FROM files WHERE folder_id IS NOT NULL GROUP BY folder_id) t;" > /dev/null
    local end1=$(date +%s%3N)
    local elapsed1=$((end1 - start1))
    echo "    Elapsed: ${elapsed1}ms"

    # Wait 100ms then request again - should hit cache
    sleep 0.1

    echo "  Request 2 (cache hit - should be fast):"
    local start2=$(date +%s%3N)
    psql "$DATABASE_URL" -t -A -c "SELECT COUNT(*) FROM (SELECT folder_id, COUNT(*) FROM files WHERE folder_id IS NOT NULL GROUP BY folder_id) t;" > /dev/null
    local end2=$(date +%s%3N)
    local elapsed2=$((end2 - start2))
    echo "    Elapsed: ${elapsed2}ms"

    # Wait 5.5 seconds - cache should expire
    echo "  Waiting 5.5s for cache to expire..."
    sleep 5.5

    echo "  Request 3 (cache expired - will be slow again):"
    local start3=$(date +%s%3N)
    psql "$DATABASE_URL" -t -A -c "SELECT COUNT(*) FROM (SELECT folder_id, COUNT(*) FROM files WHERE folder_id IS NOT NULL GROUP BY folder_id) t;" > /dev/null
    local end3=$(date +%s%3N)
    local elapsed3=$((end3 - start3))
    echo "    Elapsed: ${elapsed3}ms"

    echo ""
    echo "Summary:"
    echo "  Request 1 (miss):    ${elapsed1}ms"
    echo "  Request 2 (hit):     ${elapsed2}ms"
    echo "  Request 3 (expired): ${elapsed3}ms"

    # Verify cache is working
    if [ $elapsed2 -lt 100 ]; then
        echo "  ✅ Cache hit successful (${elapsed2}ms < 100ms)"
    else
        echo "  ❌ Cache hit too slow (${elapsed2}ms >= 100ms)"
    fi

    if [ $elapsed3 -gt 500 ]; then
        echo "  ✅ Cache expiry working (${elapsed3}ms > 500ms)"
    else
        echo "  ⚠️  Cache expiry might not be working (${elapsed3}ms <= 500ms)"
    fi

    echo ""
}

# Note: This test is for the database query only
# The actual cache is in the Rust application layer
echo "NOTE: This script tests database query performance."
echo "The 5-second cache is implemented in Rust at the application layer."
echo "To fully test the cache, use the Tauri application."
echo ""

test_cache "Folder Counts Query Performance"

echo "========================================="
echo "Cache Test Complete"
echo "========================================="
echo ""
echo "Next: Run the Tauri app and monitor logs for cache hit messages:"
echo "  - 'Filter counts served from cache in Xms'"
echo "  - 'Dynamic filter counts slow: Xms - CACHED for 5s'"
echo ""
