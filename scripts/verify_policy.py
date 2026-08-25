#!/usr/bin/env python3
from __future__ import annotations

import json
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
DEPENDENCIES = MANIFEST.get("dependencies", {})

for forbidden in ("syncer", "syncer-rs", "syncer-c"):
    if forbidden in DEPENDENCIES:
        raise SystemExit(
            "opto-sync-lib must receive one host-supplied engine capability; "
            f"found forbidden engine dependency {forbidden}"
        )

interfaces = DEPENDENCIES.get("opto-sync-interfaces", {})
revision = interfaces.get("rev") if isinstance(interfaces, dict) else None
if not isinstance(revision, str) or len(revision) != 40:
    raise SystemExit("opto-sync-interfaces must be pinned to an immutable commit")

table = json.loads(
    (ROOT / "formal/optimism-strategies.json").read_text(encoding="utf-8")
)
traces = json.loads((ROOT / "formal/traces.v1.json").read_text(encoding="utf-8"))
if set(table["strategies"]) != {
    "remote_confirmed",
    "local_acknowledged",
    "background_durable",
}:
    raise SystemExit("the optimism strategy matrix is incomplete")
for trace in traces["traces"]:
    if len(trace["states"]) != len(trace["events"]) + 1:
        raise SystemExit(f"trace {trace['name']} does not have one state per transition")

print(
    f"verified three optimism strategies, {len(traces['traces'])} traces, "
    "an immutable interfaces pin, and no bundled sync engine"
)
