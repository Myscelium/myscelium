#!/usr/bin/env python3
"""
agent_log2mermaid.py - incremental “agent” that mines host/client logs and
                       builds a Mermaid sequence diagram with help from a
                       local LLM (llama-3-8b, tinyllama, phi …)

requirements:
    pip install ollama python-dateutil tqdm
"""

from __future__ import annotations
from pathlib import Path
import json, pathlib, argparse, textwrap, datetime as dt, itertools, subprocess
from typing import Iterator, List, Dict
from dateutil import parser as dtparse
from tqdm import tqdm
import ollama     # local model – zero cloud tokens
import re
import time
import pytest

THIS_DIR = Path(__file__).parent
TEMP_FOLDER = THIS_DIR / "Temp"
CLIENT_DATA = TEMP_FOLDER / "Client1Data" / "logs.txt"
HOST_DATA = TEMP_FOLDER / "Data"/ "logs.txt"
OLLAMA_MODEL = "llama3.1:8b"
OUT_FILE = THIS_DIR / "mermaid_graph.md"

print("Client log exists:", CLIENT_DATA.exists())
print("Host log exists:", HOST_DATA.exists())

MAX_TOKENS = 2000     # safety margin
LINES_PER_CHUNK = 250 # ≈ tokens/10, tune to your model

# ─── helper regexes ─────────────────────────────────────────────────
WRAPPED_JSON_RX = re.compile(r'\{[^{}]*"command"\s*:\s*\{.*\}\s*[^{}]*\}$')
NOT_SYNC_RX = re.compile(r'Client:\s*"(?P<ck>[^"]+)"\s+not\s+sync', re.I)


def _pick_inner_json(s: str) -> str | None:
    """
    Walk the string `s`, tracking { … } nesting. Whenever a balanced
    block closes (level returns to 0), record that slice. At the end,
    return the *last* such slice, or None if none found.

    Example:
      'foo {"a":1,"command":{…}} bar {"b":2}' → '{"b":2}'
    """
    level = 0
    start = None
    last_candidate = None

    for i, ch in enumerate(s):
        if ch == '{':
            if level == 0:
                start = i
            level += 1

        elif ch == '}' and level > 0:
            level -= 1
            if level == 0 and start is not None:
                last_candidate = s[start : i + 1]

    return last_candidate


# ─── 2. Helper to translate the inner wrapper into a flat event ────────
def translate_wrapper(obj: dict) -> dict | None:
    """
    Convert a dict like:
      {
        "client_key": "...",
        "command": { "actf":"…", "origin":"…", "target":"…", "type":"…", "mode":"…" },
        "parity_id": "...",
        … optionally "ts", "priority", etc.
      }
    into a single flat event dict.

    Returns None if “command” is missing or not a dict.
    """
    cmd = obj.get("command")
    if not isinstance(cmd, dict):
        return None

    return {
        "ts":         obj.get("ts", time.time()),           # use provided ts or now
        "actor":      cmd.get("origin", "Host"),
        "target":     cmd.get("target", "Client"),
        "actf":       cmd.get("actf", ""),
        "parity":     obj.get("parity_id"),
        "cmd_type":   cmd.get("type"),
        "direction":  "-->>" if cmd.get("mode") == "Response" else "->>",
        "stage":      "reply" if cmd.get("mode") == "Response" else "normal",
        "who_intern": "",
    }

def events_from_log_line(line: str) -> list[dict]:
    """
    Recognise these patterns in `line` (a JSON-encoded string) and return
    exactly one [event_dict] or []:

      A) Top-level wrapper:
         { "client_key": "...", "command": { … }, "parity_id": "...", … }

      B) Encoded inside `.msg`:
         { "log_level": "...",
           "msg": "…{\"client_key\":\"...\",\"command\":{…}}…" }

      C) Heartbeat “not sync yet”:
         { "log_level": "...",
           "msg": "Client: \"…\" not sync yet, …" }

      D) “Sending to client” with unescaped JSON:
         { "log_level": "...",
           "msg": "Sending …: {\"client_key\":\"...\",\"command\":{…}}…" }

      E) Any case where braces are unescaped but nested inside `msg`.

    Returns a singleton list if we extract a valid event, otherwise [].
    """
    # 1) Try to parse the outer JSON. If invalid, bail immediately.
    try:
        outer = json.loads(line)
    except json.JSONDecodeError:
        return []

    # ── Case A: Already top‐level “command” wrapper?
    if isinstance(outer, dict) and "command" in outer:
        ev = translate_wrapper(outer)
        return [ev] if ev else []

    # From here on, we require a “msg” field to proceed.
    if not (isinstance(outer, dict) and "msg" in outer):
        return []

    msg = outer["msg"]

    # ── Case C: “not sync yet” heartbeat (looser regex)
    m_sync = NOT_SYNC_RX.search(msg)
    if m_sync:
        client = m_sync.group("ck")
        ping_event = {
            "ts":        outer.get("ts", time.time()),
            "actor":     "Host",
            "target":    "Client",
            "actf":      "C207",
            "parity":    None,
            "cmd_type":  "SpecialFunction",
            "direction": "-->>",
            "stage":     "ping",
            "who_intern": "",
            "msg":       f"{client} not sync yet",
        }
        return [ping_event]

    # ── Case B / D / E: Look for *any* balanced “{…}” block inside msg
    inner_txt = _pick_inner_json(msg)
    if not inner_txt:
        return []

    try:
        inner = json.loads(inner_txt)
    except json.JSONDecodeError:
        return []

    if isinstance(inner, dict) and "command" in inner:
        ev = translate_wrapper(inner)
        return [ev] if ev else []

    return []

# ───────────────────────── chunk utilities ──────────────────────────
def chunk_iterable(it: Iterator[str], n: int) -> Iterator[List[str]]:
    buf: List[str] = []
    for line in it:
        buf.append(line)
        if len(buf) >= n:
            yield buf
            buf = []
    if buf:
        yield buf


def run_llm_chunk(lines: list[str], model: str) -> list[dict]:
    """
    Send *only those lines the local parser couldn’t digest* to the LLM,
    and return a list[event-dict].
    """
    log_block = "\n".join(lines)
    prompt = f"""
        Return ONLY a JSON array of event objects – no wrapper object and no extra keys.
        Schema of each object:

        {{
        "ts":        float,
        "actor":     "Client" | "Host",
        "target":    string,
        "actf":      string,
        "parity":    string | null,
        "cmd_type":  string,
        "direction": "->>" | "-->>",
        "stage":     "handshake" | "command_push" | "reply" | "ping" | "normal",
        "who_intern": "HostScheduler" | "ClientTransposer" | ""
        }}

        If no events exist, reply with an empty array [].

        LOG
        ────────────────────
        {log_block}
        ────────────────────
    """.strip()

    rsp = ollama.generate(
        model=model,
        prompt=prompt,
        format="json",
        options={"temperature": 0, "num_predict": 512},
        stream=False,
    )

    raw = rsp["response"]

    # 1️⃣  already good?
    if isinstance(raw, list):
        return raw
    if isinstance(raw, dict):
        if "events" in raw and isinstance(raw["events"], list):
            return raw["events"]
        if {"ts", "actor"} <= raw.keys():
            return [raw]

    # 2️⃣  string cleanup → json.loads
    if isinstance(raw, str):
        txt = raw.strip()
        if txt.startswith("```"):
            txt = txt.split("```")[1] if "```" in txt else txt
            txt = txt.strip()
        try:
            parsed = json.loads(txt)
        except Exception:
            parsed = None
    else:
        parsed = None

    if parsed is not None:
        if isinstance(parsed, list):
            return parsed
        if isinstance(parsed, dict) and "events" in parsed:
            return parsed["events"]
        if isinstance(parsed, dict) and {"ts", "actor"} <= parsed.keys():
            return [parsed]

    # 3️⃣  give up – persist raw reply for inspection
    bad = THIS_DIR / "last_llm_reply.txt"
    bad.write_text(json.dumps(raw, indent=2) if not isinstance(raw, str) else raw,
                   encoding="utf-8")
    print(f"[warn] unexpected model output – saved to {bad}")
    return []


# ───────────────────────── merging & folding ────────────────────────
def merge_events(all_events: List[Dict]) -> List[Dict]:
    # Defensive check at the very top of merge_events:
    if not all(isinstance(e, dict) for e in all_events):
        raise TypeError(
            "merge_events(): at least one item is not a dict – "
            "run the normaliser or inspect your LLM output."
        )

    # 1) global sort – make sure “ts” is numeric even if the model
    #    emitted it as a quoted string.
    ev = sorted(
        all_events,
        key=lambda e: float(e.get("ts", 0.0)), # robust even if "ts" missing
    )

    # 2) collapse identical immediate repeats
    merged: List[Dict] = []
    for k, group in itertools.groupby(ev, lambda e: (e["actor"], e["actf"], e["cmd_type"])):
        seq = list(group)
        # inside merge_events, replace the surrogate creation block with:
        if len(seq) > 10 and seq[0]["actf"] in {"C206", "C207"}:
            surrogate = {**seq[0], "loop_count": len(seq), "stage": "ping"}
            merged.append(surrogate)
        else:
            merged.extend(seq)
    return merged


# ───────────────────────── Mermaid writer / patcher ─────────────────
def events_to_mermaid(events: List[Dict]) -> str:
    """
    Take the time-sorted & already-merged event list and emit a **complete**
    Markdown snippet that reproduces the desired diagram.

    The output looks like:

        In the current state this is what is happening:

        ```mermaid
        sequenceDiagram
            ...
        ```
    """
    # 0)  book-keeping so we print each banner only once
    printed = {
        "handshake"      : False,
        "host_push"      : False,   # command_push via HostScheduler
        "client_push"    : False,   # command_push via ClientTransposer
        "ping"           : False,
        "normal"         : False,
    }

    diagram: list[str] = [
        "sequenceDiagram",
        "    autonumber",
        "    participant Client",
        "    participant Host",
        "    participant HostScheduler as Host Scheduler",
        "    participant ClientTransposer as Client Transposer",
    ]

    def d_line(s: str, indent: int = 0) -> None:
        diagram.append("    " * indent + s)

    # 1) iterate in chronological order, printing section banners on first hit
    for e in events:
        stage = e["stage"]
        who   = e.get("who_intern", "")

        # ── section banners ───────────────────────────────────────────
        if stage == "handshake" and not printed["handshake"]:
            d_line("%% ─────────── 1. first contact ───────────")
            printed["handshake"] = True

        elif stage == "command_push" and who == "HostScheduler" and not printed["host_push"]:
            d_line("%% ─────────── 2. host pushes its command list ───────────")
            printed["host_push"] = True

        elif stage == "command_push" and who == "ClientTransposer" and not printed["client_push"]:
            d_line("%% ─────────── 3. client processes list and answers back ───────────")
            printed["client_push"] = True

        elif stage == "ping" and not printed["ping"]:
            d_line("%% ─────────── 4. ongoing ping / sync attempts ───────────")
            printed["ping"] = True

        elif stage == "normal" and not printed["normal"]:
            d_line("%% ─────────── 5. client schedules a normal call (shown once) ───────────")
            printed["normal"] = True

        # ── body lines (including optional loop compression) ──────────
        if "loop_count" in e:                      # ← compression already done
            d_line("loop every ≈1 s", 1)
            d_line(
                f"{e['actor']}{e['direction']}{e['target']}: "
                f"**{e['actf']}** ({e['cmd_type']})", 2
            )
            d_line("end", 1)
            continue

        # choose visible actor (internal schedulers become the sender)
        actor  = who or e["actor"]
        target = e["target"]
        label  = f"**{e['actf']}**"
        if e.get("cmd_type"):
            label += f" ({e['cmd_type']})"
        if e.get("parity"):
            label += f" {e['parity'][:6]}"
        if e.get("msg"):                           # optional free-text
            label += f" – “{e['msg']}”"

        d_line(f"{actor}{e['direction']}{target}: {label}", 1)

        # special note: scheduled python_function that is *not* executed yet
        if e["actf"].startswith("python_function"):
            d_line(
                "Note over Host: Not yet executed because<br/>"
                "client is still flagged “not sync”.", 1
            )

    # 2) wrap the diagram into a markdown fence & leading line
    header = ["In the current state this is what is happening:", "", "```mermaid"]
    trailer = ["```", ""]
    return "\n".join(header + diagram + trailer)


# def _translate_command_wrapper(obj: dict) -> dict | None:
#     """
#     Convert the {client_key, command:{...}, parity_id, ...} wrapper
#     into the flat event dict used downstream.  Returns None if required
#     fields are missing.
#     """
#     cmd = obj.get("command", {})
#     if not cmd:
#         return None

#     return {
#         "ts":          obj.get("ts", 0.0),                   # or synthesize
#         "actor":       cmd.get("origin", "Host"),
#         "target":      cmd.get("target", "Client"),
#         "actf":        cmd.get("actf", ""),
#         "parity":      obj.get("parity_id"),
#         "cmd_type":    cmd.get("type"),
#         "direction":   "-->>" if cmd.get("mode") == "Response" else "->>",
#         "stage":       "reply" if cmd.get("mode") == "Response" else "normal",
#         "who_intern":  "",                                   # could infer
#     }


# ───────────────────────── main orchestrator  ───────────────────────
def main() -> None:
    all_events: list[dict] = []

    print("Client log exists:", CLIENT_DATA.exists())
    print("Host   log exists:", HOST_DATA.exists())

    # stream both files in chronological order
    log_lines = itertools.chain.from_iterable(
        p.open(encoding="utf-8") for p in map(Path, [CLIENT_DATA, HOST_DATA])
    )

    for chunk in tqdm(chunk_iterable(log_lines, LINES_PER_CHUNK), desc="chunks"):

        # ❶ fast local parsing
        fast_events: list[dict] = []
        remainder:  list[str]  = []
        for ln in chunk:
            ev = events_from_log_line(ln)
            if ev:
                fast_events.extend(ev)
            else:
                remainder.append(ln)

        # ❷ optional LLM parsing for the remainder
        llm_events = run_llm_chunk(remainder, OLLAMA_MODEL) if remainder else []

        # ❸ aggregate
        all_events.extend(fast_events)
        all_events.extend(llm_events)

        if not all_events:
            continue  # still nothing to write – next chunk

        try:
            merged = merge_events(all_events)  # ← your existing function
        except TypeError as exc:
            print("[fatal] merge_events():", exc)
            return

        OUT_FILE.write_text(events_to_mermaid(merged), encoding="utf-8")

    print("Done. Mermaid diagram →", OUT_FILE)

if __name__ == "__main__":
    main()

### TESTS:

@pytest.mark.parametrize("input_str, expected", [
    # a) Simple JSON blob at end
    ('foo bar {"a":1,"command":{"x":2}}', '{"a":1,"command":{"x":2}}'),
    # b) Nested braces inside
    ('prefix {"foo":{"nested": {"x": 1}},"command":{"c":true}}', 
     '{"foo":{"nested": {"x": 1}},"command":{"c":true}}'),
    # c) Extra braces earlier but take last balanced block
    ('{"ignore":123} some text {"keep":456,"command":{"y":3}} tail', 
     '{"keep":456,"command":{"y":3}}'),
    # d) No valid JSON at end → None
    ('no braces here', None),
    ('incomplete { "foo": 1 ', None),
])
def test_pick_inner_json(input_str, expected):
    result = _pick_inner_json(input_str)
    assert result == expected

def test_translate_wrapper_minimal():
    inner = {
        "ts":  12345.0,
        "command": {
            "actf":   "C123",
            "origin": "Host",
            "target": "Client",
            "type":   "SpecialFunction",
            "mode":   "Response",
        },
        "parity_id": "ABC",
    }

    ev = translate_wrapper(inner)
    assert isinstance(ev, dict)

    # Check that fields got copied / inferred correctly
    assert ev["ts"] == 12345.0
    assert ev["actor"] == "Host"
    assert ev["target"] == "Client"
    assert ev["actf"] == "C123"
    assert ev["parity"] == "ABC"
    assert ev["cmd_type"] == "SpecialFunction"
    assert ev["direction"] == "-->>"       # because mode=="Response"
    assert ev["stage"] == "reply"
    assert ev["who_intern"] == ""

def test_translate_wrapper_missing_command():
    bad = {"foo": "bar", "ts": 999}
    assert translate_wrapper(bad) is None

def test_events_from_top_level_wrapper():
    top_level = {
        "ts":  54321.0,
        "client_key": "client-XYZ",
        "command": {
            "actf":   "C207",
            "origin": "Host",
            "target": "Client",
            "type":   "SpecialFunction",
            "mode":   "Response",
        },
        "parity_id": "FOO123",
        "priority": 11
    }
    line = json.dumps(top_level)

    ev_list = events_from_log_line(line)
    assert isinstance(ev_list, list) and len(ev_list) == 1

    ev = ev_list[0]
    # We expect translate_wrapper(inner) to produce an event with these fields:
    assert ev["ts"] == 54321.0
    assert ev["actor"] == "Host"
    assert ev["target"] == "Client"
    assert ev["actf"] == "C207"
    assert ev["parity"] == "FOO123"
    assert ev["cmd_type"] == "SpecialFunction"
    assert ev["direction"] == "-->>"
    assert ev["stage"] == "reply"

def test_events_from_msg_wrapper():
    inner = {
        "ts":  10101.0,
        "client_key": "client‐ABC",
        "command": {
            "actf":   "C300",
            "origin": "Client",
            "target": "Host",
            "type":   "DirectFunction",
            "mode":   "Request",
        },
        "parity_id": "BAR456",
        "priority": 5
    }
    # embed inner JSON as an escaped string inside msg
    escaped_inner = json.dumps(inner).replace('"', '\\"')
    outer = {
        "log_level": "DEBUG",
        "msg": f"Some prefix text: {escaped_inner}"
    }
    line = json.dumps(outer)

    ev_list = events_from_log_line(line)
    assert isinstance(ev_list, list) and len(ev_list) == 1

    ev = ev_list[0]
    assert ev["ts"] == 10101.0
    assert ev["actor"] == "Client"
    assert ev["target"] == "Host"
    assert ev["actf"] == "C300"
    assert ev["parity"] == "BAR456"
    assert ev["cmd_type"] == "DirectFunction"
    assert ev["direction"] == "->>"
    assert ev["stage"] == "normal"

def test_events_from_not_sync_heartbeat():
    # embed a ts to avoid using time.time()
    outer = {
        "ts":  22222.0,
        "log_level": "INFO",
        "msg": 'Client: "foo‐123" not sync yet, trying again in: 44 seconds!'
    }
    line = json.dumps(outer)

    ev_list = events_from_log_line(line)
    assert isinstance(ev_list, list) and len(ev_list) == 1

    ev = ev_list[0]
    assert ev["ts"] == 22222.0
    assert ev["actor"] == "Host"
    assert ev["target"] == "Client"
    assert ev["actf"] == "C207"
    assert ev["parity"] is None
    assert ev["cmd_type"] == "SpecialFunction"
    assert ev["direction"] == "-->>"
    assert ev["stage"] == "ping"
    assert "not sync yet" in ev["msg"]


def test_events_from_sending_to_client():
    # The “msg” wraps inner JSON without escaping
    inner = {
        "ts":  33333.0,
        "client_key": "foo‐XYZ",
        "command": {
            "actf":   "C500",
            "origin": "Host",
            "target": "Origin",
            "type":   "SpecialFunction",
            "mode":   "Response",
        },
        "parity_id": "QUX789",
        "priority": 8
    }
    # put the inner JSON unescaped inside msg
    outer = {
        "log_level": "DEBUG",
        "msg": f'Sending to client foo: {json.dumps(inner)}'
    }
    line = json.dumps(outer)

    ev_list = events_from_log_line(line)
    assert isinstance(ev_list, list) and len(ev_list) == 1

    ev = ev_list[0]
    assert ev["ts"] == 33333.0
    assert ev["actor"] == "Host"
    assert ev["target"] == "Origin"
    assert ev["actf"] == "C500"
    assert ev["parity"] == "QUX789"
    assert ev["cmd_type"] == "SpecialFunction"
    assert ev["direction"] == "-->>"
    assert ev["stage"] == "reply"

@pytest.mark.parametrize("line", [
    "",                                 # empty
    "just random text",                 # not JSON at all
    '{"foo":123, "bar":"baz"}',         # JSON but neither "command" nor "msg"
    '{"log_level":"INFO","msg":"hello"}'  # JSON with msg but no inner JSON / no “not sync”
])
def test_events_from_no_match(line):
    assert events_from_log_line(line) == []