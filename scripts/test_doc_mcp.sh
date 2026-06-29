#!/bin/bash
set -euo pipefail

# ── Configuration ──────────────────────────────────────────────────────────
MCP_ENDPOINT="http://localhost:8000/mcp"
RESULTS_DIR="scripts/test-results/doc-mcp"
PASS=0
FAIL=0

rm -rf "$RESULTS_DIR"
mkdir -p "$RESULTS_DIR"

# ── Helpers ────────────────────────────────────────────────────────────────

call_tool() {
    local name="$1" id="$2" json_args="$3"
    local out="$RESULTS_DIR/${id}_${name}.json"
    local resp
    resp=$(curl -s -X POST "$MCP_ENDPOINT" \
        -H "Content-Type: application/json" \
        -H "Accept: application/json" \
        -H "mcp-session-id: ${SESSION_ID:-}" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"$name\",\"arguments\":$json_args},\"id\":$id}")
    echo "$resp" | jq . > "$out"
    echo "$resp"
}

# Extract the inner result from a tool call response.
# doc-mcp runs with json_response=True, so structuredContent.result is the
# actual JSON value (not a string-encoded version).
inner() {
    jq -c '.result.structuredContent.result // .result.content[0].text // empty'
}

# Check if the response contains an error
has_error() { jq -e '.error // empty' > /dev/null 2>&1; }

pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail_test() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

# ── 01. Initialize ─────────────────────────────────────────────────────────

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  doc-mcp API Test Suite                                     ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "━━━ [01] INITIALIZE ━━━"
RESP=$(curl -s -i -X POST "$MCP_ENDPOINT" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json, text/event-stream" \
    -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test-suite","version":"1.0"}},"id":1}')

SESSION_ID=$(echo "$RESP" | grep -i "mcp-session-id" | awk '{print $2}' | tr -d '\r\n ' || true)
if [ -z "$SESSION_ID" ]; then
    echo "  NOTE: No session ID received (stateless mode). Continuing without."
    SESSION_ID=""
else
    echo "  Session: $SESSION_ID"
fi
pass "Initialize completed"

# ── 02. Initialized notification ───────────────────────────────────────────

echo ""
echo "━━━ [02] INITIALIZED ━━━"
SESS_PARAM=""
[ -n "$SESSION_ID" ] && SESS_PARAM="?mcp-session-id=$SESSION_ID"
curl -s -X POST "${MCP_ENDPOINT}${SESS_PARAM}" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"notifications/initialized"}' > /dev/null 2>&1 || true
pass "Initialized notification sent"

# ── 03. Index documents ────────────────────────────────────────────────────

echo ""
echo "━━━ [03] INDEX SPEC (payment-gateway) ━━━"
resp=$(call_tool "index_document" 3 '{"path":"scripts/data/spec-payment-gateway.md"}')
if echo "$resp" | has_error; then
    fail_test "index_document payment-gateway returned error"
else
    chunks=$(echo "$resp" | inner | jq -r '.indexed_chunks')
    path=$(echo "$resp"  | inner | jq -r '.path')
    if [ "$chunks" -gt 0 ] && [ "$path" = "scripts/data/spec-payment-gateway.md" ]; then
        pass "Indexed $chunks chunks for $path"
    else
        fail_test "Unexpected response: chunks=$chunks path=$path"
    fi
fi

echo ""
echo "━━━ [04] INDEX SPEC (user-auth) ━━━"
resp=$(call_tool "index_document" 4 '{"path":"scripts/data/spec-user-auth.md"}')
if echo "$resp" | has_error; then
    fail_test "index_document user-auth returned error"
else
    chunks=$(echo "$resp" | inner | jq -r '.indexed_chunks')
    path=$(echo "$resp"  | inner | jq -r '.path')
    if [ "$chunks" -gt 0 ] && [ "$path" = "scripts/data/spec-user-auth.md" ]; then
        pass "Indexed $chunks chunks for $path"
    else
        fail_test "Unexpected response: chunks=$chunks path=$path"
    fi
fi

echo ""
echo "━━━ [05] INDEX ARCHITECTURE (payments) ━━━"
resp=$(call_tool "index_document" 5 '{"path":"scripts/data/architecture-payments.md"}')
if echo "$resp" | has_error; then
    fail_test "index_document architecture-payments returned error"
else
    chunks=$(echo "$resp" | inner | jq -r '.indexed_chunks')
    if [ "$chunks" -gt 0 ]; then
        pass "Indexed $chunks chunks for architecture-payments"
    else
        fail_test "No chunks indexed for architecture-payments"
    fi
fi

echo ""
echo "━━━ [06] INDEX ADR (database-choice) ━━━"
resp=$(call_tool "index_document" 6 '{"path":"scripts/data/adr-database-choice.md"}')
if echo "$resp" | has_error; then
    fail_test "index_document adr-database-choice returned error"
else
    chunks=$(echo "$resp" | inner | jq -r '.indexed_chunks')
    if [ "$chunks" -gt 0 ]; then
        pass "Indexed $chunks chunks for adr-database-choice"
    else
        fail_test "No chunks indexed for adr-database-choice"
    fi
fi

echo ""
echo "━━━ [07] INDEX CHARTER (payments-team) ━━━"
resp=$(call_tool "index_document" 7 '{"path":"scripts/data/charter-payments-team.md"}')
if echo "$resp" | has_error; then
    fail_test "index_document charter-payments-team returned error"
else
    chunks=$(echo "$resp" | inner | jq -r '.indexed_chunks')
    if [ "$chunks" -gt 0 ]; then
        pass "Indexed $chunks chunks for charter-payments-team"
    else
        fail_test "No chunks indexed for charter-payments-team"
    fi
fi

echo ""
echo "━━━ [08] INDEX SUPERSEDED SPEC (deprecated-feature) ━━━"
resp=$(call_tool "index_document" 8 '{"path":"scripts/data/spec-deprecated-feature.md"}')
if echo "$resp" | has_error; then
    fail_test "index_document spec-deprecated-feature returned error"
else
    chunks=$(echo "$resp" | inner | jq -r '.indexed_chunks')
    status=$(echo "$resp" | inner | jq -r '.status')
    if [ "$chunks" -gt 0 ] && [ "$status" = "superseded" ]; then
        pass "Indexed $chunks chunks (status=$status)"
    else
        fail_test "Unexpected: chunks=$chunks status=$status"
    fi
fi

echo ""
echo "━━━ [09] INDEX NONEXISTENT FILE (expect error) ━━━"
resp=$(call_tool "index_document" 9 '{"path":"scripts/data/nonexistent-file.md"}')
if echo "$resp" | has_error; then
    pass "Correctly rejected nonexistent file"
else
    fail_test "Should have rejected nonexistent file but did not"
fi

# ── 04. List active ────────────────────────────────────────────────────────

echo ""
echo "━━━ [10] LIST ACTIVE (no filters) ━━━"
resp=$(call_tool "list_active" 10 '{}')
if echo "$resp" | has_error; then
    fail_test "list_active returned error"
else
    count=$(echo "$resp" | inner | jq 'length')
    # 5 active docs: payment-gateway, user-auth, architecture, adr, charter
    # (deprecated-feature is superseded, so excluded)
    if [ "$count" -ge 4 ]; then
        paths=$(echo "$resp" | inner | jq -r '.[].path')
        pass "list_active returned $count documents"
        echo "$paths" | sed 's/^/    /'
    else
        fail_test "Expected >= 4 active docs, got $count"
    fi
fi

echo ""
echo "━━━ [11] LIST ACTIVE (type=spec) ━━━"
resp=$(call_tool "list_active" 11 '{"type":"spec"}')
count=$(echo "$resp" | inner | jq 'length')
if [ "$count" -ge 2 ]; then
    paths=$(echo "$resp" | inner | jq -r '.[].path')
    pass "type=spec returned $count documents"
    echo "$paths" | sed 's/^/    /'
    # Verify only spec-type docs
    types=$(echo "$resp" | inner | jq -r '.[].type' | sort -u)
    if [ "$types" = "spec" ]; then
        pass "All results have type=spec"
    else
        fail_test "Expected only type=spec, got: $types"
    fi
else
    fail_test "Expected >= 2 spec docs, got $count"
fi

echo ""
echo "━━━ [12] LIST ACTIVE (domain=payments) ━━━"
resp=$(call_tool "list_active" 12 '{"domain":"payments"}')
count=$(echo "$resp" | inner | jq 'length')
if [ "$count" -ge 3 ]; then
    paths=$(echo "$resp" | inner | jq -r '.[].path')
    pass "domain=payments returned $count documents"
    echo "$paths" | sed 's/^/    /'
else
    fail_test "Expected >= 3 payments docs, got $count"
fi

echo ""
echo "━━━ [13] LIST ACTIVE (type=architecture, domain=payments) ━━━"
resp=$(call_tool "list_active" 13 '{"type":"architecture","domain":"payments"}')
count=$(echo "$resp" | inner | jq 'length')
if [ "$count" -ge 1 ]; then
    path=$(echo "$resp" | inner | jq -r '.[0].path')
    expected="scripts/data/architecture-payments.md"
    if [ "$path" = "$expected" ]; then
        pass "Filtered to expected document: $path"
    else
        fail_test "Expected $expected, got $path"
    fi
else
    fail_test "Expected >= 1 architecture+payments doc, got $count"
fi

# ── 05. Search ─────────────────────────────────────────────────────────────

echo ""
echo "━━━ [14] SEARCH (payment gateway integration) ━━━"
resp=$(call_tool "search" 14 '{"query":"payment gateway integration"}')
hits=$(echo "$resp" | inner | jq 'length')
if [ "$hits" -gt 0 ]; then
    top_path=$(echo "$resp" | inner | jq -r '.[0].path')
    top_score=$(echo "$resp" | inner | jq -r '.[0].score')
    pass "Search returned $hits hits (top: $top_path, score=$top_score)"
else
    fail_test "Search returned no hits"
fi

echo ""
echo "━━━ [15] SEARCH (database choice) ━━━"
resp=$(call_tool "search" 15 '{"query":"database choice PostgreSQL"}')
hits=$(echo "$resp" | inner | jq 'length')
if [ "$hits" -gt 0 ]; then
    found_adr=$(echo "$resp" | inner | jq -r '[.[] | select(.path == "scripts/data/adr-database-choice.md")] | length')
    if [ "$found_adr" -gt 0 ]; then
        pass "ADR found in search results (score=$(echo "$resp" | inner | jq -r '.[0].score'))"
    else
        fail_test "ADR not found in search results"
    fi
else
    fail_test "Search returned no hits"
fi

echo ""
echo "━━━ [16] SEARCH WITH TYPE FILTER (type=adr, query=database) ━━━"
resp=$(call_tool "search" 16 '{"query":"database","type":"adr"}')
hits=$(echo "$resp" | inner | jq 'length')
all_adr=$(echo "$resp" | inner | jq -r '[.[].type] | unique | length')
if [ "$all_adr" -le 1 ]; then
    types=$(echo "$resp" | inner | jq -r '[.[].type] | unique | .[]')
    if [ "$types" = "adr" ] || [ "$hits" -eq 0 ]; then
        pass "type=adr filter returned only ADR documents ($hits hits)"
    else
        fail_test "Expected only adr type, got: $types"
    fi
else
    fail_test "Multiple types returned with type=adr filter"
fi

echo ""
echo "━━━ [17] SEARCH WITH DOMAIN FILTER (domain=auth) ━━━"
resp=$(call_tool "search" 17 '{"query":"authentication login","domain":"auth"}')
hits=$(echo "$resp" | inner | jq 'length')
if [ "$hits" -gt 0 ]; then
    found_spec=$(echo "$resp" | inner | jq -r '[.[] | select(.path == "scripts/data/spec-user-auth.md")] | length')
    if [ "$found_spec" -gt 0 ]; then
        pass "domain=auth filter found user-auth spec ($hits hits)"
    else
        fail_test "user-auth spec not found with domain=auth filter"
    fi
else
    fail_test "No hits with domain=auth filter"
fi

echo ""
echo "━━━ [18] SEARCH DEPRECATED (include_deprecated=false) ━━━"
resp=$(call_tool "search" 18 '{"query":"legacy batch processing","include_deprecated":false}')
hits=$(echo "$resp" | inner | jq 'length')
found_deprecated=$(echo "$resp" | inner | jq -r '[.[] | select(.path == "scripts/data/spec-deprecated-feature.md")] | length')
if [ "$found_deprecated" -eq 0 ]; then
    pass "Superseded doc correctly excluded from default search ($hits hits)"
else
    fail_test "Superseded doc should not appear with include_deprecated=false"
fi

echo ""
echo "━━━ [19] SEARCH DEPRECATED (include_deprecated=true) ━━━"
resp=$(call_tool "search" 19 '{"query":"legacy batch processing","include_deprecated":true}')
hits=$(echo "$resp" | inner | jq 'length')
found_deprecated=$(echo "$resp" | inner | jq -r '[.[] | select(.path == "scripts/data/spec-deprecated-feature.md")] | length')
if [ "$found_deprecated" -gt 0 ]; then
    hit_status=$(echo "$resp" | inner | jq -r '.[] | select(.path == "scripts/data/spec-deprecated-feature.md") | .doc_status')
    pass "Superseded doc found with include_deprecated=true (status=$hit_status)"
else
    fail_test "Superseded doc should appear with include_deprecated=true"
fi

# ── 06. Get content ────────────────────────────────────────────────────────

echo ""
echo "━━━ [20] GET CONTENT (spec-payment-gateway.md) ━━━"
resp=$(call_tool "get_content" 20 '{"path":"scripts/data/spec-payment-gateway.md"}')
if echo "$resp" | has_error; then
    fail_test "get_content returned error"
else
    # Verify we get back a string with expected content
    content=$(echo "$resp" | inner | jq -r '.')
    if echo "$content" | grep -q "Payment Gateway Integration"; then
        pass "Retrieved content for payment-gateway spec"
    else
        fail_test "Content does not contain expected text"
    fi
fi

echo ""
echo "━━━ [21] GET CONTENT (nonexistent file) ━━━"
resp=$(call_tool "get_content" 21 '{"path":"scripts/data/nonexistent-file.md"}')
if echo "$resp" | has_error; then
    pass "Correctly rejected request for nonexistent file"
else
    fail_test "Should have rejected nonexistent file but returned content"
fi

# ── 07. Mark deleted ───────────────────────────────────────────────────────

echo ""
echo "━━━ [22] MARK DELETED (spec-deprecated-feature.md) ━━━"
resp=$(call_tool "mark_deleted" 22 '{"path":"scripts/data/spec-deprecated-feature.md"}')
if echo "$resp" | has_error; then
    fail_test "mark_deleted returned error"
else
    chunks_updated=$(echo "$resp" | inner | jq -r '.chunks_updated')
    blob_sha=$(echo "$resp" | inner | jq -r '.blob_sha')
    if [ "$chunks_updated" -gt 0 ]; then
        pass "Tombstoned $chunks_updated chunks (blob_sha=$blob_sha)"
    else
        fail_test "No chunks updated for mark_deleted"
    fi
fi

echo ""
echo "━━━ [23] LIST ACTIVE AFTER DELETION ━━━"
resp=$(call_tool "list_active" 23 '{}')
if echo "$resp" | has_error; then
    fail_test "list_active returned error after deletion"
else
    found_deprecated=$(echo "$resp" | inner | jq -r '[.[] | select(.path == "scripts/data/spec-deprecated-feature.md")] | length')
    if [ "$found_deprecated" -eq 0 ]; then
        pass "Deprecated doc correctly absent from list_active after tombstone"
    else
        fail_test "Deprecated doc still appears in list_active after mark_deleted"
    fi
fi

echo ""
echo "━━━ [24] SEARCH DELETED (include_deprecated=true) ━━━"
resp=$(call_tool "search" 24 '{"query":"legacy batch processing","include_deprecated":true}')
found_deleted=$(echo "$resp" | inner | jq -r '[.[] | select(.path == "scripts/data/spec-deprecated-feature.md")] | length')
if [ "$found_deleted" -gt 0 ]; then
    hit_status=$(echo "$resp" | inner | jq -r '.[] | select(.path == "scripts/data/spec-deprecated-feature.md") | .doc_status')
    if [ "$hit_status" = "deleted" ]; then
        pass "Deleted doc still searchable, status='$hit_status'"
    else
        fail_test "Expected status=deleted, got status=$hit_status"
    fi
else
    fail_test "Deleted doc not found in deprecated search"
fi

echo ""
echo "━━━ [25] MARK DELETED FOR NONEXISTENT ━━━"
resp=$(call_tool "mark_deleted" 25 '{"path":"scripts/data/nonexistent-file.md"}')
if echo "$resp" | has_error; then
    pass "Correctly rejected mark_deleted for nonexistent/unindexed file"
else
    fail_test "Should have rejected mark_deleted for nonexistent file"
fi

# ── 08. Summary ────────────────────────────────────────────────────────────

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Test suite complete.                                       ║"
echo "║  Passed: $PASS / $((PASS + FAIL))                                         ║"
if [ "$FAIL" -gt 0 ]; then
    echo "║  Failed: $FAIL                                                 ║"
fi
echo "║  Results saved to:  $RESULTS_DIR/                            ║"
echo "╚══════════════════════════════════════════════════════════════╝"

exit $FAIL
