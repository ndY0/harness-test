#!/bin/bash
set -euo pipefail

# ── Configuration ──────────────────────────────────────────────────────────
MCP_ENDPOINT="http://localhost:8001/mcp"
RESULTS_DIR="scripts/test-results/jira-mcp"
TEST_TICKET="KAN-1"
FEATURE_PATH="scripts/test-results/jira-mcp/test-feature"
FEATURE_DIR="${PWD}/${FEATURE_PATH}"
PASS=0
FAIL=0
ORIGINAL_STATUS="unknown"
SKIP_TRANSITIONS=0

# Clean root-owned files from previous run before removing results dir
docker exec jira-mcp rm -rf "/repo/$FEATURE_PATH" 2>/dev/null || true
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
        -H "mcp-session-id: $SESSION_ID" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"$name\",\"arguments\":$json_args},\"id\":$id}")
    echo "$resp" | jq . > "$out"
    echo "$resp"
}

# Tool results are JSON-encoded strings (json_response=True).
# Extract and parse.
inner() {
    jq -c '(.result.structuredContent.result // .result.content[0].text // empty) | if type == "string" then fromjson else . end'
}

has_error()  { jq -e '.error // empty' > /dev/null 2>&1; }
has_result() { jq -e '.result // empty' > /dev/null 2>&1; }

pass()      { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail_test() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

# ── 01. Initialize ─────────────────────────────────────────────────────────

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  jira-mcp MCP API Test Suite                                ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "━━━ [01] INITIALIZE ━━━"
RESP=$(curl -s -i -X POST "$MCP_ENDPOINT" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json, text/event-stream" \
    -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test-suite","version":"1.0"}},"id":1}')

SESSION_ID=$(echo "$RESP" | grep -i "mcp-session-id" | awk '{print $2}' | tr -d '\r\n ')
if [ -z "$SESSION_ID" ]; then
    fail_test "Failed to extract session ID"
    echo "$RESP"
    exit 1
fi
echo "  Session: $SESSION_ID"
pass "Initialize completed"

# ── 02. Initialized notification ───────────────────────────────────────────

echo ""
echo "━━━ [02] INITIALIZED ━━━"
curl -s -X POST "${MCP_ENDPOINT}?mcp-session-id=${SESSION_ID}" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"notifications/initialized"}' > /dev/null
pass "Initialized notification sent"

# ── 03. get_ticket (KAN-1) ─────────────────────────────────────────────────

echo ""
echo "━━━ [03] GET TICKET (KAN-1) ━━━"
resp=$(call_tool "get_ticket" 2 "{\"ticket_key\":\"$TEST_TICKET\"}")
if echo "$resp" | has_error; then
    fail_test "get_ticket returned error"
else
    key=$(echo "$resp" | inner | jq -r '.key')
    summary=$(echo "$resp" | inner | jq -r '.summary')
    status=$(echo "$resp" | inner | jq -r '.status')
    issue_type=$(echo "$resp" | inner | jq -r '.issue_type')

    if [ "$key" = "$TEST_TICKET" ]; then
        pass "key=$key, type=$issue_type, status=$status"
        echo "    summary: $summary"
    else
        fail_test "Expected key=$TEST_TICKET, got $key"
    fi

    # Save fields for later tests
    ORIGINAL_STATUS="$status"
    echo "  Saved original status: $ORIGINAL_STATUS"
fi

# ── 03. get_ticket (nonexistent) ───────────────────────────────────────────

echo ""
echo "━━━ [04] GET TICKET (nonexistent) ━━━"
resp=$(call_tool "get_ticket" 3 '{"ticket_key":"NONEXIST-999"}')
if echo "$resp" | has_error; then
    fail_test "get_ticket returned transport error"
else
    inner_val=$(echo "$resp" | inner)
    err_type=$(echo "$inner_val" | jq -r '.error // empty')
    if [ -n "$err_type" ]; then
        pass "Correctly returned error: $err_type"
    else
        fail_test "Expected error response, got success"
    fi
fi

# ── 04. search_tickets (project = KAN) ─────────────────────────────────────

echo ""
echo "━━━ [05] SEARCH TICKETS (project = KAN) ━━━"
resp=$(call_tool "search_tickets" 4 '{"jql":"project = KAN"}')
if echo "$resp" | has_error; then
    fail_test "search_tickets returned error"
else
    total=$(echo "$resp" | inner | jq -r '.total')
    found_kan1=$(echo "$resp" | inner | jq -r '[.issues[] | select(.key == "KAN-1")] | length')
    if [ "$total" -ge 1 ] && [ "$found_kan1" -ge 1 ]; then
        pass "total=$total, KAN-1 found in results"
    else
        fail_test "total=$total, KAN-1 found=$found_kan1"
    fi
fi

# ── 05. search_tickets (with max_results=1) ────────────────────────────────

echo ""
echo "━━━ [06] SEARCH TICKETS (max_results=1) ━━━"
resp=$(call_tool "search_tickets" 5 '{"jql":"project = KAN","max_results":1}')
if echo "$resp" | has_error; then
    fail_test "search_tickets returned error"
else
    total=$(echo "$resp" | inner | jq -r '.total')
    issues_count=$(echo "$resp" | inner | jq -r '.issues | length')
    if [ "$issues_count" -le 1 ] && [ "$total" -ge 1 ]; then
        pass "max_results respected: total=$total, returned=$issues_count"
    else
        fail_test "total=$total, returned=$issues_count (expected <= 1)"
    fi
fi

# ── 06. get_ticket_comments ────────────────────────────────────────────────

echo ""
echo "━━━ [07] GET TICKET COMMENTS (KAN-1) ━━━"
resp=$(call_tool "get_ticket_comments" 6 "{\"ticket_key\":\"$TEST_TICKET\"}")
if echo "$resp" | has_error; then
    fail_test "get_ticket_comments returned error"
else
    count=$(echo "$resp" | inner | jq 'length')
    pass "Returned $count comments"
fi

# ── 07. add_comment ────────────────────────────────────────────────────────

echo ""
echo "━━━ [08] ADD COMMENT ━━━"
COMMENT_BODY="Test comment from MCP test suite — $(date -u +%Y-%m-%dT%H:%M:%SZ)"
resp=$(call_tool "add_comment" 7 "{\"ticket_key\":\"$TEST_TICKET\",\"body\":\"$COMMENT_BODY\"}")
if echo "$resp" | has_error; then
    fail_test "add_comment returned error"
else
    comment_id=$(echo "$resp" | inner | jq -r '.comment_id')
    resp_key=$(echo "$resp" | inner | jq -r '.ticket_key')
    if [ -n "$comment_id" ] && [ "$resp_key" = "$TEST_TICKET" ]; then
        pass "Comment posted (id=$comment_id)"
    else
        fail_test "Unexpected response: comment_id=$comment_id key=$resp_key"
    fi
fi

# ── 08. Verify comment appears ─────────────────────────────────────────────

echo ""
echo "━━━ [09] VERIFY COMMENT ━━━"
resp=$(call_tool "get_ticket_comments" 8 "{\"ticket_key\":\"$TEST_TICKET\"}")
found_comment=$(echo "$resp" | inner | jq -r --arg body "$COMMENT_BODY" '[.[] | select(.body == $body)] | length')
if [ "$found_comment" -ge 1 ]; then
    pass "Test comment found in comment thread"
else
    fail_test "Test comment not found in thread"
fi

# ── 09. set_field (labels) ─────────────────────────────────────────────────

echo ""
echo "━━━ [10] SET FIELD (labels) ━━━"
resp=$(call_tool "set_field" 9 "{\"ticket_key\":\"$TEST_TICKET\",\"field\":\"labels\",\"value\":[\"test-mcp\",\"automated\"]}")
if echo "$resp" | has_error; then
    fail_test "set_field returned error"
else
    field=$(echo "$resp" | inner | jq -r '.field')
    if [ "$field" = "labels" ]; then
        pass "set_field labels succeeded"
    else
        fail_test "Unexpected field in response: $field"
    fi
fi

# ── 10. Verify labels were set ─────────────────────────────────────────────

echo ""
echo "━━━ [11] VERIFY LABELS SET ━━━"
resp=$(call_tool "get_ticket" 10 "{\"ticket_key\":\"$TEST_TICKET\"}")
labels=$(echo "$resp" | inner | jq -r '.labels')
if echo "$labels" | jq -e 'index("test-mcp") and index("automated")' > /dev/null 2>&1; then
    pass "Labels verified: $labels"
else
    fail_test "Labels not set correctly: $labels"
fi

# ── 11. set_field (clear labels) — rollback ─────────────────────────────────

echo ""
echo "━━━ [12] SET FIELD (clear labels) ━━━"
resp=$(call_tool "set_field" 11 "{\"ticket_key\":\"$TEST_TICKET\",\"field\":\"labels\",\"value\":[]}")
if echo "$resp" | has_error; then
    fail_test "set_field clear labels returned error"
else
    pass "Labels cleared"
fi

# ── 12. Verify labels cleared ──────────────────────────────────────────────

echo ""
echo "━━━ [13] VERIFY LABELS CLEARED ━━━"
resp=$(call_tool "get_ticket" 12 "{\"ticket_key\":\"$TEST_TICKET\"}")
labels=$(echo "$resp" | inner | jq -r '.labels')
if [ "$labels" = "[]" ] || [ "$labels" = "null" ] || [ -z "$(echo "$labels" | jq -r '.[]' 2>/dev/null)" ]; then
    pass "Labels correctly cleared: $labels"
else
    fail_test "Labels not cleared: $labels"
fi

# ── 13. Discover available transitions ─────────────────────────────────────

echo ""
echo "━━━ [14] DISCOVER TRANSITIONS ━━━"
# Use a deliberately bad transition name to get the available list
resp=$(call_tool "transition_ticket" 13 "{\"ticket_key\":\"$TEST_TICKET\",\"transition_name\":\"zzz_discover_available\"}")
if echo "$resp" | has_error; then
    fail_test "transition_ticket returned transport error"
    SKIP_TRANSITIONS=1
else
    inner_val=$(echo "$resp" | inner)
    err_type=$(echo "$inner_val" | jq -r '.error // empty')
    available=$(echo "$inner_val" | jq -r '.available // []')
    available_count=$(echo "$available" | jq 'length')

    if [ "$err_type" = "transition_not_found" ] && [ "$available_count" -gt 0 ]; then
        pass "Discovered $available_count available transitions"
        echo "    Available: $(echo "$available" | jq -r '.[]')"
    elif [ "$err_type" = "transition_not_found" ]; then
        pass "No transitions available from current state ($ORIGINAL_STATUS)"
        SKIP_TRANSITIONS=1
    else
        fail_test "Unexpected response: error=$err_type"
        SKIP_TRANSITIONS=1
    fi
fi

# ── 14. Transition round-trip ──────────────────────────────────────────────

if [ "$SKIP_TRANSITIONS" -eq 1 ]; then
    echo ""
    echo "━━━ [15] TRANSITION ROUND-TRIP — SKIPPED ━━━"
    pass "Skipped (no transitions available from $ORIGINAL_STATUS)"
else
    echo ""
    echo "━━━ [15] TRANSITION ROUND-TRIP ━━━"

    # Try each available transition until one actually changes status
    FORWARD_TRANSITION=""
    while IFS= read -r candidate; do
        [ -z "$candidate" ] && continue
        resp=$(call_tool "transition_ticket" 140 "{\"ticket_key\":\"$TEST_TICKET\",\"transition_name\":\"$candidate\"}")
        if echo "$resp" | has_error; then continue; fi
        transitioned=$(echo "$resp" | inner | jq -r '.transitioned_to // empty')
        [ -z "$transitioned" ] && continue
        if [ "$transitioned" != "$ORIGINAL_STATUS" ]; then
            FORWARD_TRANSITION="$candidate"
            break
        fi
    done <<< "$(echo "$available" | jq -r '.[]')"

    if [ -z "$FORWARD_TRANSITION" ]; then
        pass "No forward transition available from $ORIGINAL_STATUS (all return to same status)"
    else
        echo "    Forward: $ORIGINAL_STATUS → $FORWARD_TRANSITION"

        # Verify via get_ticket
        resp=$(call_tool "get_ticket" 114 "{\"ticket_key\":\"$TEST_TICKET\"}")
        current_status=$(echo "$resp" | inner | jq -r '.status')
        if [ "$current_status" != "$ORIGINAL_STATUS" ]; then
            pass "Status changed: $ORIGINAL_STATUS → $current_status"

                # Now discover transitions from the new state
                resp=$(call_tool "transition_ticket" 214 "{\"ticket_key\":\"$TEST_TICKET\",\"transition_name\":\"zzz_discover_back\"}")
                back_available=$(echo "$resp" | inner | jq -r '.available // []')

                # Try each transition from new state; find one back to original
                BACK_TRANSITION=""
                while IFS= read -r candidate; do
                    [ -z "$candidate" ] && continue
                    resp=$(call_tool "transition_ticket" 215 "{\"ticket_key\":\"$TEST_TICKET\",\"transition_name\":\"$candidate\"}")
                    if echo "$resp" | has_error; then continue; fi
                    back_to=$(echo "$resp" | inner | jq -r '.transitioned_to // empty')
                    if [ "$back_to" = "$ORIGINAL_STATUS" ]; then
                        BACK_TRANSITION="$candidate"
                        break
                    fi
                done <<< "$(echo "$back_available" | jq -r '.[]')"

                if [ -n "$BACK_TRANSITION" ]; then
                    pass "Round-trip complete: $ORIGINAL_STATUS → $current_status → $back_to"
                else
                    fail_test "No transition back to $ORIGINAL_STATUS found"
                    echo "    Available from $current_status: $(echo "$back_available" | jq -r '.[]')"
                fi
            else
                fail_test "Status did not change after transition (still $current_status)"
            fi
    fi
fi

# ── 15. link_local_feature_tool ────────────────────────────────────────────

echo ""
echo "━━━ [16] LINK LOCAL FEATURE ━━━"
resp=$(call_tool "link_local_feature_tool" 15 "{\"ticket_key\":\"$TEST_TICKET\",\"feature_path\":\"$FEATURE_PATH\"}")
if echo "$resp" | has_error; then
    fail_test "link_local_feature_tool returned error"
else
    ticket_key=$(echo "$resp" | inner | jq -r '.ticket_key')
    status_at_sync=$(echo "$resp" | inner | jq -r '.status_at_sync')
    if [ "$ticket_key" = "$TEST_TICKET" ] && [ -n "$status_at_sync" ]; then
        pass "Linked $ticket_key → $FEATURE_PATH (status_at_sync=$status_at_sync)"
    else
        fail_test "Unexpected response: key=$ticket_key status=$status_at_sync"
    fi

    # Verify sidecar file was created on disk
    if [ -f "$FEATURE_DIR/jira_ref.json" ]; then
        pass "jira_ref.json written to disk"
    else
        fail_test "jira_ref.json not found at $FEATURE_DIR"
    fi
fi

# ── 16. get_sync_status (linked feature) ───────────────────────────────────

echo ""
echo "━━━ [17] GET SYNC STATUS (linked) ━━━"
resp=$(call_tool "get_sync_status" 16 "{\"feature_path\":\"$FEATURE_PATH\"}")
if echo "$resp" | has_error; then
    fail_test "get_sync_status returned error"
else
    in_sync=$(echo "$resp" | inner | jq -r '.in_sync')
    desc_changed=$(echo "$resp" | inner | jq -r '.description_changed')
    status_changed=$(echo "$resp" | inner | jq -r '.status_changed')
    pending=$(echo "$resp" | inner | jq -r '.has_pending_clarifications')

    if [ "$in_sync" = "true" ]; then
        pass "in_sync=true (desc_changed=$desc_changed, status_changed=$status_changed, pending=$pending)"
    elif [ "$desc_changed" = "true" ] || [ "$status_changed" = "true" ]; then
        pass "Sync diff detected changes (desc=$desc_changed, status=$status_changed)"
    else
        fail_test "Unexpected sync state: in_sync=$in_sync"
    fi
fi

# ── 17. get_sync_status (unlinked path) ────────────────────────────────────

echo ""
echo "━━━ [18] GET SYNC STATUS (unlinked) ━━━"
UNLINKED_PATH="scripts/test-results/jira-mcp/no-feature"
resp=$(call_tool "get_sync_status" 17 "{\"feature_path\":\"$UNLINKED_PATH\"}")
if echo "$resp" | has_error; then
    fail_test "get_sync_status returned error"
else
    in_sync=$(echo "$resp" | inner | jq -r '.in_sync')
    ticket_key=$(echo "$resp" | inner | jq -r '.ticket_key')
    if [ "$in_sync" = "false" ] && [ "$ticket_key" = "" ]; then
        pass "Correctly reports in_sync=false for unlinked path"
    else
        fail_test "Expected in_sync=false, empty key — got in_sync=$in_sync, key=$ticket_key"
    fi
fi

# ── 18. list_synced_features ───────────────────────────────────────────────

echo ""
echo "━━━ [19] LIST SYNCED FEATURES ━━━"
resp=$(call_tool "list_synced_features" 18 '{}')
if echo "$resp" | has_error; then
    fail_test "list_synced_features returned error"
else
    found_kan1=$(echo "$resp" | inner | jq -r "[.[] | select(.ticket_key == \"$TEST_TICKET\")] | length")
    if [ "$found_kan1" -ge 1 ]; then
        pass "KAN-1 found in synced features list"
    else
        fail_test "KAN-1 not found in synced features"
    fi
fi

# ── 19. request_clarification ──────────────────────────────────────────────

echo ""
echo "━━━ [20] REQUEST CLARIFICATION ━━━"
CLARIFY_Q1="What is the target SLA for this feature?"
CLARIFY_Q2="Which team owns the implementation?"
resp=$(call_tool "request_clarification" 19 "{\"ticket_key\":\"$TEST_TICKET\",\"feature_path\":\"$FEATURE_PATH\",\"questions\":[\"$CLARIFY_Q1\",\"$CLARIFY_Q2\"]}")
if echo "$resp" | has_error; then
    fail_test "request_clarification returned error"
else
    parked=$(echo "$resp" | inner | jq -r '.parked')
    comment_id=$(echo "$resp" | inner | jq -r '.comment_id')
    transition_applied=$(echo "$resp" | inner | jq -r '.transition_applied')
    q_count=$(echo "$resp" | inner | jq -r '.questions | length')

    if [ "$parked" = "true" ] && [ -n "$comment_id" ] && [ "$q_count" -ge 1 ]; then
        pass "Clarification posted: parked=$parked, questions=$q_count, transition=$transition_applied"
    else
        fail_test "Unexpected: parked=$parked, comment=$comment_id, questions=$q_count"
    fi

    # Save clarification status for later rollback
    CLARIFY_STATUS=$(echo "$resp" | inner | jq -r '.transition_applied // "none"')
fi

# ── 20. Verify clarification comment appears ───────────────────────────────

echo ""
echo "━━━ [21] VERIFY CLARIFICATION COMMENT ━━━"
resp=$(call_tool "get_ticket_comments" 20 "{\"ticket_key\":\"$TEST_TICKET\"}")
found_clarify=$(echo "$resp" | inner | jq -r '[.[] | select(.body | contains("Clarification requested"))] | length')
if [ "$found_clarify" -ge 1 ]; then
    pass "Clarification comment found in thread"
else
    fail_test "Clarification comment not found"
fi

# ── 21. Verify pending_clarifications in sync status ───────────────────────

echo ""
echo "━━━ [22] VERIFY PENDING CLARIFICATIONS ━━━"
resp=$(call_tool "get_sync_status" 21 "{\"feature_path\":\"$FEATURE_PATH\"}")
has_pending=$(echo "$resp" | inner | jq -r '.has_pending_clarifications')
if [ "$has_pending" = "true" ]; then
    pass "has_pending_clarifications=true"
else
    fail_test "Expected has_pending_clarifications=true, got $has_pending"
fi

# ── 22. Rollback: return ticket to original status ─────────────────────────

echo ""
echo "━━━ [23] ROLLBACK TO ORIGINAL STATUS ━━━"
# First check current status
resp=$(call_tool "get_ticket" 22 "{\"ticket_key\":\"$TEST_TICKET\"}")
current_status=$(echo "$resp" | inner | jq -r '.status')

if [ "$current_status" = "$ORIGINAL_STATUS" ]; then
    pass "Ticket already at original status ($ORIGINAL_STATUS)"
else
    echo "    Current: $current_status, target: $ORIGINAL_STATUS"
    # Discover available transitions from current state
    resp=$(call_tool "transition_ticket" 122 "{\"ticket_key\":\"$TEST_TICKET\",\"transition_name\":\"zzz_rollback_discover\"}")
    rollback_available=$(echo "$resp" | inner | jq -r '.available // []')
    echo "    Available from $current_status: $(echo "$rollback_available" | jq -r '.[]')"

    # Try each transition to find one back to original status
    ROLLBACK_TRANSITION=""
    while IFS= read -r candidate; do
        [ -z "$candidate" ] && continue
        resp=$(call_tool "transition_ticket" 122 "{\"ticket_key\":\"$TEST_TICKET\",\"transition_name\":\"$candidate\"}")
        if echo "$resp" | has_error; then continue; fi
        rolled_to=$(echo "$resp" | inner | jq -r '.transitioned_to // empty')
        if [ "$rolled_to" = "$ORIGINAL_STATUS" ]; then
            ROLLBACK_TRANSITION="$candidate"
            pass "Rolled back: $current_status → $rolled_to"
            break
        fi
    done <<< "$(echo "$rollback_available" | jq -r '.[]')"

    if [ -z "$ROLLBACK_TRANSITION" ]; then
        fail_test "No transition back to $ORIGINAL_STATUS found from $current_status"
    fi
fi

# ── 23. Cleanup ────────────────────────────────────────────────────────────

echo ""
echo "━━━ [24] CLEANUP ━━━"
# jira_ref.json is created inside the Docker container as root — clean up via docker exec
docker exec jira-mcp rm -f "/repo/$FEATURE_PATH/jira_ref.json" 2>/dev/null || true
rmdir "$FEATURE_DIR" 2>/dev/null || true
if [ ! -f "$FEATURE_DIR/jira_ref.json" ]; then
    pass "Cleaned up jira_ref.json and test feature directory"
else
    fail_test "Failed to clean up jira_ref.json"
fi

# ── Summary ────────────────────────────────────────────────────────────────

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Test suite complete.                                       ║"
printf "║  Passed: %-2d / %-2d                                              ║\n" "$PASS" "$((PASS + FAIL))"
if [ "$FAIL" -gt 0 ]; then
    printf "║  Failed: %-2d                                                  ║\n" "$FAIL"
fi
echo "║  Results saved to:  $RESULTS_DIR/                            ║"
echo "╚══════════════════════════════════════════════════════════════╝"

exit $FAIL
