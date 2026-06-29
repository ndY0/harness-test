"""
Direct LSP smoke test — spawns rust-analyzer, sends JSON-RPC manually.
Run inside the container or anywhere rust-analyzer is on PATH.
Usage:  docker compose exec code-graph-mcp python /workspace/scripts/test_lsp_direct.py
"""
import asyncio
import json
import sys
from pathlib import Path

WORKSPACE = "/workspace"
FILE = "src/main.rs"

HEADER_FMT = "Content-Length: {}\r\n\r\n"


async def read_message(reader) -> dict | None:
    headers = {}
    while True:
        line = await reader.readline()
        if not line:
            return None
        line = line.decode("ascii", errors="replace").strip()
        if not line:
            break
        if ":" in line:
            k, _, v = line.partition(":")
            headers[k.strip().lower()] = v.strip()
    length = int(headers.get("content-length", 0))
    if not length:
        return None
    body = await reader.readexactly(length)
    return json.loads(body.decode("utf-8"))


async def send_message(stdin, msg: dict):
    body = json.dumps(msg).encode("utf-8")
    header = HEADER_FMT.format(len(body)).encode("ascii")
    stdin.write(header + body)
    await stdin.drain()


async def request(reader, stdin, method: str, params, msg_id: int) -> dict:
    msg = {"jsonrpc": "2.0", "id": msg_id, "method": method, "params": params}
    await send_message(stdin, msg)
    while True:
        resp = await read_message(reader)
        if resp is None:
            raise ConnectionError("LSP closed")
        if resp.get("id") == msg_id:
            if "error" in resp:
                err = resp["error"]
                raise RuntimeError(f"LSP error {err['code']}: {err['message']}")
            return resp.get("result", {})


async def notify(stdin, method: str, params):
    await send_message(stdin, {"jsonrpc": "2.0", "method": method, "params": params})


async def main():
    file_path = Path(WORKSPACE) / FILE
    file_uri = file_path.as_uri()
    text = file_path.read_text()

    print(f"Workspace: {WORKSPACE}")
    print(f"File: {file_uri}")
    print(f"File size: {len(text)} bytes")
    print()

    # Spawn rust-analyzer
    proc = await asyncio.create_subprocess_exec(
        "rust-analyzer",
        stdin=asyncio.subprocess.PIPE,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
        cwd=WORKSPACE,
    )
    assert proc.stdin and proc.stdout, "no pipes"

    reader = proc.stdout
    stdin = proc.stdin
    msg_id = 1

    # 1. Initialize
    print("--- [1] initialize ---")
    result = await request(reader, stdin, "initialize", {
        "processId": None,
        "rootUri": Path(WORKSPACE).as_uri(),
        "capabilities": {
            "textDocument": {
                "documentSymbol": {"hierarchicalDocumentSymbolSupport": True},
                "callHierarchy": {"dynamicRegistration": False},
            },
            "workspace": {"symbol": {"dynamicRegistration": False}},
        },
        "initializationOptions": {
            "callHierarchy": {"enabled": True},
            "cargo": {"buildScripts": {"enable": True}, "features": "all", "targetDir": "/tmp/cargo-target"},
            "check": {"command": "check"},
            "checkOnSave": {"enable": False},
            "diagnostics": {"enable": False},
        },
    }, msg_id)
    msg_id += 1

    caps = result.get("capabilities", {})
    print(f"  callHierarchyProvider: {bool(caps.get('callHierarchyProvider'))}")
    print(f"  typeHierarchyProvider: {bool(caps.get('typeHierarchyProvider'))}")
    print(f"  server: {result.get('serverInfo', {}).get('name', 'unknown')}")

    # 2. Initialized
    print("\n--- [2] initialized ---")
    await notify(stdin, "initialized", {})
    await asyncio.sleep(1)  # let it settle

    # 3. Run cargo check
    print("\n--- [3] cargo check ---")
    cargo_proc = await asyncio.create_subprocess_exec(
        "cargo", "check", "--target-dir", "/tmp/cargo-target",
        cwd=WORKSPACE,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )
    cargo_stdout, cargo_stderr = await asyncio.wait_for(cargo_proc.communicate(), timeout=120)
    cargo_ok = cargo_proc.returncode == 0
    print(f"  exit code: {cargo_proc.returncode}")
    if not cargo_ok:
        err = (cargo_stderr or cargo_stdout or b"").decode(errors="replace")[:500]
        print(f"  stderr: {err}")
    else:
        print("  SUCCESS")

    # 4. Open the file
    print(f"\n--- [4] didOpen: {FILE} ---")
    await notify(stdin, "textDocument/didOpen", {
        "textDocument": {"uri": file_uri, "languageId": "rust", "version": 1, "text": text},
    })
    await asyncio.sleep(0.5)

    # 5. Document symbols
    print("\n--- [5] documentSymbol ---")
    symbols = await request(reader, stdin, "textDocument/documentSymbol", {
        "textDocument": {"uri": file_uri},
    }, msg_id)
    msg_id += 1

    def flatten(syms, depth=0):
        res = []
        for s in (syms or []):
            name = s.get("name", "")
            kind = s.get("kind", 0)
            sel = s.get("selectionRange", {}).get("start", {})
            res.append({"name": name, "kind": kind, "line": sel.get("line", 0), "char": sel.get("character", 0)})
            children = s.get("children", [])
            if children:
                res.extend(flatten(children, depth + 1))
        return res

    flat = flatten(symbols)
    KINDS = {
        5: "Class/Struct", 6: "Method", 9: "Constructor",
        11: "Interface", 12: "Function", 23: "Struct",
    }
    print(f"  Total symbols: {len(flat)}")
    for s in flat:
        kind_str = KINDS.get(s["kind"], f"kind_{s['kind']}")
        print(f"    [{kind_str}] {s['name']} @ L{s['line']}:C{s['char']}")

    if not flat:
        print("  NO SYMBOLS FOUND — aborting")
        proc.terminate()
        return 1

    # 6. Call hierarchy for each callable — focus on fns with bodies
    callable_kinds = {5, 6, 9, 12, 23}
    callables = [s for s in flat if s["kind"] in callable_kinds]

    # Prioritize symbols with bodies over trait declarations
    def _has_body(s):
        # Trait method declarations have kind=6 (Method), name same as impl methods
        # We prefer impl methods which occur later (higher line numbers)
        return s["line"]

    # Filter: skip trait-only declarations where an impl version exists later
    names_seen = set()
    prioritized = []
    for s in sorted(callables, key=lambda x: -x["line"]):  # higher line first
        if s["name"] not in names_seen:
            names_seen.add(s["name"])
            prioritized.append(s)
    prioritized.sort(key=lambda x: x["line"])

    print(f"\n--- [6] callHierarchy ({len(prioritized)} unique callable symbols) ---")

    call_hierarchy_failures = 0
    for sym in prioritized:
        line, col = sym["line"], sym["char"]
        KINDS = {5: "Struct", 6: "Method", 9: "Constructor", 12: "Function", 23: "Struct"}
        kind_str = KINDS.get(sym["kind"], f"kind_{sym['kind']}")
        print(f"\n  [{kind_str}] {sym['name']} @ ({line},{col})")

        # Attempt to recover from content-modified by re-sending file content
        for retry in range(2):
            if retry > 0:
                print("    retrying after didChange...")
                text = file_path.read_text()
                await notify(stdin, "textDocument/didChange", {
                    "textDocument": {"uri": file_uri, "version": 2},
                    "contentChanges": [{"text": text}],
                })
                await asyncio.sleep(1)

            items = None
            for delay in (1, 2, 4, 8, 16, 32):
                await asyncio.sleep(delay)
                try:
                    items = await request(reader, stdin, "textDocument/prepareCallHierarchy", {
                        "textDocument": {"uri": file_uri},
                        "position": {"line": line, "character": col},
                    }, msg_id)
                    msg_id += 1
                except RuntimeError as e:
                    err_msg = str(e)
                    print(f"    -> prepare ERROR: {err_msg[:120]}")
                    if "content modified" in err_msg.lower():
                        break  # will retry
                    items = None
                    break
                if items:
                    break
                print(f"    -> prepare null (waited {delay}s)")

            if items is not None:
                break

        if not items:
            print(f"    -> NO CALL HIERARCHY ITEM after retries")
            call_hierarchy_failures += 1
            continue

        item = items[0]
        print(f"    -> item: name={item.get('name')}, kind={item.get('kind')}")

        async def _call_hierarchy_query(req_name, res_key):
            nonlocal msg_id
            for retry2 in range(3):
                try:
                    result = await request(reader, stdin, req_name, {"item": item}, msg_id)
                    msg_id += 1
                    return result or []
                except RuntimeError as e:
                    err_msg = str(e)
                    if "content modified" in err_msg.lower() and retry2 < 2:
                        print(f"       {res_key} content-modified, retrying...")
                        text = file_path.read_text()
                        await notify(stdin, "textDocument/didChange", {
                            "textDocument": {"uri": file_uri, "version": 3},
                            "contentChanges": [{"text": text}],
                        })
                        await asyncio.sleep(1)
                        continue
                    print(f"       {res_key} ERROR: {err_msg[:120]}")
                    return []
            return []

        outgoing = await _call_hierarchy_query("callHierarchy/outgoingCalls", "outgoing")
        print(f"       outgoing_calls: {len(outgoing)}")
        for oc in (outgoing or []):
            to = oc.get("to", {})
            ranges = oc.get("fromRanges", [])
            print(f"         -> {to.get('name')} ({len(ranges)} refs)")

        incoming = await _call_hierarchy_query("callHierarchy/incomingCalls", "incoming")
        print(f"       incoming_calls: {len(incoming)}")
        for ic in (incoming or []):
            fr = ic.get("from", {})
            ranges = ic.get("fromRanges", [])
            print(f"         <- {fr.get('name')} ({len(ranges)} refs)")

        if not outgoing and not incoming:
            print(f"       (no edges — likely analysis mismatch)")

    # 7. References for the first symbol
    if flat:
        print(f"\n--- [7] references ---")
        first = flat[0]
        refs = await request(reader, stdin, "textDocument/references", {
            "textDocument": {"uri": file_uri},
            "position": {"line": first["line"], "character": first["char"]},
            "context": {"includeDeclaration": False},
        }, msg_id)
        msg_id += 1
        print(f"  [{first['name']}] references: {len(refs or [])}")

    # Cleanup
    await notify(stdin, "textDocument/didClose", {"textDocument": {"uri": file_uri}})
    await asyncio.sleep(0.5)
    await request(reader, stdin, "shutdown", None, msg_id)
    await notify(stdin, "exit", None)
    proc.terminate()

    print(f"\n{'='*50}")
    print(f"Result: {len(flat)} symbols, {len(callables)} callable, {call_hierarchy_failures} prepare failures")
    if call_hierarchy_failures == 0 and len(flat) > 0:
        print("PASS")
        return 0
    else:
        print("FAIL")
        return 1


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))
