#!/bin/bash
set -euo pipefail

# ── Configuration ──────────────────────────────────────────────────────────
MCP_ENDPOINT="http://localhost:8765/mcp"
WT_NAME="test-wt"
WT_PATH=".worktrees/${WT_NAME}"
FULL_WT_PATH="${PWD}/${WT_PATH}"
PASS=0
FAIL=0

# ── Helpers ────────────────────────────────────────────────────────────────

call_tool() {
    local name="$1" id="$2" json_args="$3"
    local resp
    resp=$(curl -s -X POST "$MCP_ENDPOINT" \
        -H "Content-Type: application/json" \
        -H "Accept: application/json" \
        -H "mcp-session-id: $SESSION_ID" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"$name\",\"arguments\":$json_args},\"id\":$id}")
    echo "$resp"
}

inner() {
    jq -c '(.result.structuredContent.result // .result.content[0].text // empty) | if type == "string" then fromjson else . end'
}

has_error() { jq -e '.error // empty' > /dev/null 2>&1; }

pass()      { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail_test() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

cleanup_wt() {
    if [ -d "$FULL_WT_PATH" ]; then
        echo "  Cleaning up worktree..."
        git worktree remove --force "$FULL_WT_PATH" 2>/dev/null || true
        # Prune any stale worktree metadata
        git worktree prune 2>/dev/null || true
    fi
}

# ── 01. Initialize session ─────────────────────────────────────────────────

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  code-graph-mcp — Worktree Index Test                       ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "━━━ [01] INITIALIZE ━━━"
RESP=$(curl -s -i -X POST "$MCP_ENDPOINT" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json, text/event-stream" \
    -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test-worktree","version":"1.0"}},"id":1}')

SESSION_ID=$(echo "$RESP" | grep -i "mcp-session-id" | awk '{print $2}' | tr -d '\r\n ')
if [ -z "$SESSION_ID" ]; then
    fail_test "Failed to extract session ID"
    exit 1
fi
echo "  Session: $SESSION_ID"
pass "Session initialized"

# ── 02. Initialized notification ───────────────────────────────────────────

echo ""
echo "━━━ [02] INITIALIZED ━━━"
curl -s -X POST "${MCP_ENDPOINT}?mcp-session-id=${SESSION_ID}" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"notifications/initialized"}' > /dev/null
pass "Initialized notification sent"

# ── 03. Cleanup stale worktree from previous run ───────────────────────────

echo ""
echo "━━━ [03] CLEANUP STALE WORKTREE ━━━"
cleanup_wt
pass "Stale worktree cleaned (if any)"

# ── 04. Create worktree ────────────────────────────────────────────────────

echo ""
echo "━━━ [04] CREATE WORKTREE ━━━"
# Create worktree from the current HEAD in the workspace
git worktree add "$FULL_WT_PATH" HEAD 2>&1
if [ -d "$FULL_WT_PATH" ] && [ -f "$FULL_WT_PATH/src/main.rs" ]; then
    pass "Worktree created at $WT_PATH"
else
    fail_test "Worktree creation failed"
    exit 1
fi

# ── 05. Trigger workspace re-index ─────────────────────────────────────────

echo ""
echo "━━━ [05] INDEX WORKSPACE ━━━"
# Index the full workspace — includes worktree files
resp=$(call_tool "index_workspace" 5 '{}')
if echo "$resp" | has_error; then
    fail_test "index_workspace returned error"
else
    echo "  Indexed. Waiting for processing..."
    # Indexing is async — give it time to process
    sleep 5
    pass "Workspace re-index triggered"
fi

# ── 06. Search for symbol in worktree ──────────────────────────────────────

echo ""
echo "━━━ [06] FIND SYMBOL IN WORKTREE ━━━"
# Search for a known symbol from the Rust source. Use Person struct from src/main.rs
resp=$(call_tool "find_symbol" 6 '{"name":"Person"}')
if echo "$resp" | has_error; then
    fail_test "find_symbol returned error"
else
    inner_val=$(echo "$resp" | inner)
    count=$(echo "$inner_val" | jq 'length')
    if [ "$count" -gt 0 ]; then
        # Check if any result's file path includes the worktree directory
        wt_hits=$(echo "$inner_val" | jq -r --arg wt "$WT_PATH" '[.[] | select(.symbol_file | contains($wt))] | length')
        if [ "${wt_hits:-0}" -gt 0 ]; then
            wt_file=$(echo "$inner_val" | jq -r --arg wt "$WT_PATH" '.[] | select(.symbol_file | contains($wt)) | .symbol_file' | head -1)
            pass "Found symbol in worktree: $wt_file"
        else
            fail_test "Symbol 'Person' not found in worktree path ($WT_PATH)"
            echo "    All hits:"
            echo "$inner_val" | jq -r '.[] | "    \(.symbol_file):\(.symbol_line) \(.symbol_name)"'
        fi
    else
        fail_test "find_symbol returned no results for 'Person'"
    fi
fi

# ── 07. Verify worktree file appears in file_symbols ───────────────────────

echo ""
echo "━━━ [07] GET FILE SYMBOLS FROM WORKTREE ━━━"
resp=$(call_tool "get_file_symbols" 7 "{\"file_path\":\"${WT_PATH}/src/main.rs\"}")
if echo "$resp" | has_error; then
    fail_test "get_file_symbols returned error for worktree path"
else
    inner_val=$(echo "$resp" | inner)
    sym_count=$(echo "$inner_val" | jq 'length')
    if [ "${sym_count:-0}" -gt 0 ]; then
        pass "get_file_symbols returned $sym_count symbols for worktree file"
    else
        fail_test "get_file_symbols returned 0 symbols for worktree file"
    fi
fi

# ── 08. Cleanup worktree ───────────────────────────────────────────────────

echo ""
echo "━━━ [08] CLEANUP WORKTREE ━━━"
cleanup_wt
if [ ! -d "$FULL_WT_PATH" ]; then
    pass "Worktree removed"
else
    fail_test "Worktree removal failed"
fi

# ── Summary ────────────────────────────────────────────────────────────────

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Worktree index test complete.                              ║"
printf "║  Passed: %-2d / %-2d                                              ║\n" "$PASS" "$((PASS + FAIL))"
if [ "$FAIL" -gt 0 ]; then
    printf "║  Failed: %-2d                                                  ║\n" "$FAIL"
fi
echo "╚══════════════════════════════════════════════════════════════╝"

exit $FAIL
