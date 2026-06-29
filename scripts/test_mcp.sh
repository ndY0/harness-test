#!/bin/bash
set -euo pipefail

MCP_ENDPOINT="http://localhost:8765/mcp"
RESULTS_DIR="test-results"
rm -rf "$RESULTS_DIR"
mkdir -p "$RESULTS_DIR"

NEXT_ID=3

# ── helpers ──────────────────────────────────────────────────────────────

# Call an MCP tool.  Saves pretty-printed response to $RESULTS_DIR/<name>.json
# and writes the RAW (compact) JSON to stdout for piping into jq.
call_tool() {
    local name="$1" id="$2" json_args="$3"
    local out="$RESULTS_DIR/${name}.json"
    local resp
    resp=$(curl -s -X POST "$MCP_ENDPOINT" \
        -H "Content-Type: application/json" \
        -H "Accept: application/json" \
        -H "mcp-session-id: $SESSION_ID" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"$name\",\"arguments\":$json_args},\"id\":$id}")
    echo "$resp" | jq . > "$out"
    echo "$resp"
}

# Pipe raw MCP response in, get the parsed inner JSON (array/object).
inner()  { jq -r '.result.structuredContent.result | fromjson'; }

# Pipe raw MCP response in, get the inner result as a raw string.
inner_raw() { jq -r '.result.structuredContent.result'; }

# Shortcut: call a tool and print just the inner result
# Usage: call_inner <name> <id> <json_args>
call_inner() {
    local resp
    resp=$(call_tool "$1" "$2" "$3")
    echo "$resp" | inner
}

fail() { echo "❌ $*"; }

# ── 1. Initialize ─────────────────────────────────────────────────────────

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  MCP Code Graph — API Test Suite                            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "━━━ [01] INITIALIZE ━━━"
RESP=$(curl -s -i -X POST "$MCP_ENDPOINT" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json, text/event-stream" \
    -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test-suite","version":"1.0"}},"id":1}')

SESSION_ID=$(echo "$RESP" | grep -i "mcp-session-id" | awk '{print $2}' | tr -d '\r\n ')
if [ -z "$SESSION_ID" ]; then
    fail "Failed to extract session ID"
    echo "$RESP"
    exit 1
fi
echo "Session: $SESSION_ID"
echo "OK" | tee "$RESULTS_DIR/01_init.txt"

# ── 2. Initialized notification ──────────────────────────────────────────

echo ""
echo "━━━ [02] INITIALIZED ━━━"
curl -s -X POST "$MCP_ENDPOINT?mcp-session-id=$SESSION_ID" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"notifications/initialized"}' > /dev/null
echo "OK" | tee "$RESULTS_DIR/02_initialized.txt"

# ── 3. Meta ──────────────────────────────────────────────────────────────

echo ""
echo "━━━ [03] LIST LANGUAGES ━━━"
call_inner "list_languages" 3 "{}" | tee "$RESULTS_DIR/03_languages.txt"
echo ""

echo "━━━ [04] INDEX WORKSPACE ━━━"
curl -s --max-time 300 -X POST "$MCP_ENDPOINT" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json" \
    -H "mcp-session-id: $SESSION_ID" \
    -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"index_workspace","arguments":{}},"id":4}' \
    | jq . | tee "$RESULTS_DIR/04_index_workspace.json"
echo ""

echo "━━━ [05] GET STATS ━━━"
call_inner "get_stats" 5 "{}" | tee "$RESULTS_DIR/05_stats.txt"
echo ""

# ── 4. Symbol navigation ─────────────────────────────────────────────────

echo "━━━ [06] GET FILE SYMBOLS (src/main.rs) ━━━"
resp=$(call_tool "get_file_symbols" 6 '{"file_path":"src/main.rs"}')
echo "$resp" | inner | jq '[.[] | {name: .symbol_name, kind: .symbol_kind, line: .symbol_line, visibility: .symbol_visibility}]' | tee "$RESULTS_DIR/06_file_symbols.json"
echo ""

echo "━━━ [07] FIND SYMBOL (Person) ━━━"
resp=$(call_tool "find_symbol" 7 '{"name":"Person"}')
echo "$resp" | inner | jq '.[] | {symbol_id, symbol_name, symbol_kind, symbol_file, symbol_line}' | tee "$RESULTS_DIR/07_find_person.json"
echo ""

echo "━━━ [08] FIND SYMBOL (Greeter) ━━━"
resp=$(call_tool "find_symbol" 8 '{"name":"Greeter"}')
echo "$resp" | inner | jq '.[] | {symbol_id, symbol_name, symbol_kind}' | tee "$RESULTS_DIR/08_find_greeter.json"
echo ""

echo "━━━ [09] FIND SYMBOL (create_default_team) ━━━"
resp=$(call_tool "find_symbol" 9 '{"name":"create_default_team"}')
echo "$resp" | inner | jq '.[] | {symbol_id, symbol_name, symbol_kind}' | tee "$RESULTS_DIR/09_find_create_default_team.json"
echo ""

echo "━━━ [10] FUZZY FIND (team) ━━━"
resp=$(call_tool "fuzzy_find" 10 '{"partial":"team"}')
echo "$resp" | inner | jq '[.[] | {symbol_id, symbol_name, symbol_kind}]' | tee "$RESULTS_DIR/10_fuzzy_team.json"
echo ""

echo "━━━ [11] GET MODULE TREE ━━━"
call_inner "get_module_tree" 11 "{}" | tee "$RESULTS_DIR/11_module_tree.json"
echo ""

# ── 5. Resolve IDs for downstream queries ─────────────────────────────────

echo "━━━ [12] RESOLVE SYMBOL IDs ━━━"

# Helper: find a symbol, extract the first symbol_id
first_id() {
    local name="$1"
    local resp
    resp=$(call_tool "find_symbol" 99 "{\"name\":\"$name\"}")
    echo "$resp" | inner | jq -r '.[0].symbol_id // empty'
}

PERSON_STRUCT_ID=$(first_id "Person")
GREETER_ID=$(first_id "Greeter")
CREATE_TEAM_ID=$(first_id "create_default_team")
GREET_ID=$(first_id "greet")
GET_AGE_ID=$(first_id "get_age")
MAIN_ID=$(first_id "main")
TEST_PC_ID=$(first_id "test_person_creation")
TEAM_ID=$(first_id "Team")
PERSON_NEW_ID=$(first_id "new")

echo "  Person struct : ${PERSON_STRUCT_ID:-NOT FOUND}"
echo "  Greeter trait : ${GREETER_ID:-NOT FOUND}"
echo "  create_default_team : ${CREATE_TEAM_ID:-NOT FOUND}"
echo "  greet method  : ${GREET_ID:-NOT FOUND}"
echo "  get_age method: ${GET_AGE_ID:-NOT FOUND}"
echo "  main function : ${MAIN_ID:-NOT FOUND}"
echo "  test_person_creation : ${TEST_PC_ID:-NOT FOUND}"
echo "  Team struct   : ${TEAM_ID:-NOT FOUND}"
{
    echo "Person struct: $PERSON_STRUCT_ID"
    echo "Greeter trait: $GREETER_ID"
    echo "create_default_team: $CREATE_TEAM_ID"
    echo "greet: $GREET_ID"
    echo "get_age: $GET_AGE_ID"
    echo "main: $MAIN_ID"
    echo "test_person_creation: $TEST_PC_ID"
    echo "Team struct: $TEAM_ID"
    echo "Person::new: $PERSON_NEW_ID"
} > "$RESULTS_DIR/12_resolved_ids.txt"
echo ""

# ── 6. Impact analysis ───────────────────────────────────────────────────

if [ -n "${GET_AGE_ID:-}" ]; then
    echo "━━━ [13] GET CALLERS (get_age) ━━━"
    call_inner "get_callers" 13 "{\"symbol_id\":\"$GET_AGE_ID\"}" | tee "$RESULTS_DIR/13_callers_get_age.json"
    echo ""
else
    echo "━━━ [13] GET CALLERS — SKIPPED ━━━"
fi

if [ -n "${CREATE_TEAM_ID:-}" ]; then
    echo "━━━ [14] GET CALLEES (create_default_team) ━━━"
    resp=$(call_tool "get_callees" 14 "{\"symbol_id\":\"$CREATE_TEAM_ID\"}")
    echo "$resp" | inner | jq '[.[]? | .symbol_name]' | tee "$RESULTS_DIR/14_callees_create_team.json"
    echo ""
else
    echo "━━━ [14] GET CALLEES — SKIPPED ━━━"
fi

if [ -n "${PERSON_STRUCT_ID:-}" ]; then
    echo "━━━ [15] GET TYPE USAGES (Person) ━━━"
    resp=$(call_tool "get_type_usages" 15 "{\"symbol_id\":\"$PERSON_STRUCT_ID\"}")
    echo "$resp" | inner | jq '[.[]? | .symbol_name]' | tee "$RESULTS_DIR/15_type_usages_person.json"
    echo ""
else
    echo "━━━ [15] GET TYPE USAGES — SKIPPED ━━━"
fi

if [ -n "${GREETER_ID:-}" ]; then
    echo "━━━ [16] GET IMPLEMENTORS (Greeter) ━━━"
    resp=$(call_tool "get_implementors" 16 "{\"trait_symbol_id\":\"$GREETER_ID\"}")
    echo "$resp" | inner | jq '[.[]? | .symbol_name]' | tee "$RESULTS_DIR/16_implementors_greeter.json"
    echo ""
else
    echo "━━━ [16] GET IMPLEMENTORS — SKIPPED ━━━"
fi

if [ -n "${GREETER_ID:-}" ]; then
    echo "━━━ [17] GET TRAIT DEPENDENTS (Greeter) ━━━"
    resp=$(call_tool "get_trait_dependents" 17 "{\"trait_symbol_id\":\"$GREETER_ID\"}")
    echo "$resp" | inner | jq '[.[]? | .symbol_name]' | tee "$RESULTS_DIR/17_trait_dependents_greeter.json"
    echo ""
else
    echo "━━━ [17] GET TRAIT DEPENDENTS — SKIPPED ━━━"
fi

if [ -n "${PERSON_NEW_ID:-}" ]; then
    echo "━━━ [18] GET TESTS FOR (new) ━━━"
    resp=$(call_tool "get_tests_for" 18 "{\"symbol_id\":\"$PERSON_NEW_ID\"}")
    echo "$resp" | inner | jq '[.[]? | .symbol_name]' | tee "$RESULTS_DIR/18_tests_for_new.json"
    echo ""
else
    echo "━━━ [18] GET TESTS FOR — SKIPPED ━━━"
fi

# ── 7. Context assembly ──────────────────────────────────────────────────

if [ -n "${CREATE_TEAM_ID:-}" ]; then
    echo "━━━ [19] GET EDIT SURFACE (create_default_team) ━━━"
    resp=$(call_tool "get_edit_surface" 19 "{\"symbol_id\":\"$CREATE_TEAM_ID\"}")
    echo "$resp" | inner | jq '{symbol_name, symbol_signature, symbol_module, symbol_visibility}' | tee "$RESULTS_DIR/19_edit_surface_create_team.json"
    echo ""
else
    echo "━━━ [19] GET EDIT SURFACE — SKIPPED ━━━"
fi

if [ -n "${GREET_ID:-}" ]; then
    echo "━━━ [20] GET SIGNATURE (greet) ━━━"
    call_inner "get_signature" 20 "{\"symbol_id\":\"$GREET_ID\"}" | tee "$RESULTS_DIR/20_signature_greet.json"
    echo ""
else
    echo "━━━ [20] GET SIGNATURE — SKIPPED ━━━"
fi

# ── 8. Structural tools ───────────────────────────────────────────────────

echo "━━━ [21] GET COUPLING HOTSPOTS ━━━"
resp=$(call_tool "get_coupling_hotspots" 21 "{}")
echo "$resp" | inner | jq '[.[] | {symbol_name, symbol_kind, _total_in_degree}]' | tee "$RESULTS_DIR/21_coupling_hotspots.json"
echo ""

echo "━━━ [22] GET MODULE API (root) ━━━"
resp=$(call_tool "get_module_api" 22 '{"module_path":""}')
echo "$resp" | inner | jq '[.[] | {symbol_name, symbol_kind, symbol_signature}]' | tee "$RESULTS_DIR/22_module_api.json"
echo ""

# ── 9. Incremental re-index ──────────────────────────────────────────────

echo "━━━ [23] INDEX FILE (src/main.rs) ━━━"
call_inner "index_file" 23 '{"file_path":"src/main.rs"}' | tee "$RESULTS_DIR/23_index_file.txt"
echo ""

# ── 10. Summary ──────────────────────────────────────────────────────────

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Test suite complete.                                       ║"
echo "║  Results saved to:  $RESULTS_DIR/                            ║"
echo "╚══════════════════════════════════════════════════════════════╝"

echo ""
echo "Result file summary:"
for f in "$RESULTS_DIR"/*.json "$RESULTS_DIR"/*.txt; do
    [ -f "$f" ] || continue
    name=$(basename "$f")
    if [ -s "$f" ]; then
        lines=$(wc -l < "$f")
        echo "  ✅ $name  ($lines lines)"
    else
        echo "  ⚠️  $name  (empty)"
    fi
done
