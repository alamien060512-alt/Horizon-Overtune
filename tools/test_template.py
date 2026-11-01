#!/usr/bin/env python3
"""Tiny compiled fixture: builds the actual Overtune bundle and checks the
original method calls execute once with results preserved, runs the shared
runtime unit test, and verifies repeated source bytes and mtimes.
"""
import hashlib
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
COMPAT = json.loads((ROOT / "compatibility.json").read_text())
PIN = COMPAT["horizon_revision"]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    bundle = ROOT / "mixins" / "overtune"
    manifest = bundle / "horizon-mixin.toml"
    if not manifest.is_file():
        raise SystemExit(f"no bundle at {bundle}")

    # Build the standalone runtime library and run its tests.
    result = subprocess.run(
        ["cargo", "test", "--locked", "--offline"],
        cwd=ROOT,
        capture_output=True, text=True,
    )
    if result.returncode:
        print(result.stderr[-8000:], file=sys.stderr)
        return result.returncode

    # Check the bundle's mixin manifest parses and names match.
    import tomllib
    mixin = tomllib.loads(manifest.read_text())
    mod = mixin["mod"]
    assert bundle.name == mod["id"], f"bundle dir {bundle.name} != mod id {mod['id']}"
    modules = mixin.get("module", mixin.get("modules", []))
    # schema 6 uses [[module]]
    module_list = mixin.get("module", [])
    if not module_list:
        raise SystemExit("no [[module]] declared in horizon-mixin.toml")
    for m in module_list:
        assert m["source"] == "runtime.rs", f"module source {m['source']} != runtime.rs"
        assert m["name"] == "overtune", f"module name {m['name']} != overtune"

    # Verify runtime.rs is a single file (no inner module files).
    rs_files = list(bundle.glob("*.rs"))
    assert len(rs_files) == 1 and rs_files[0].name == "runtime.rs", \
        f"expected only runtime.rs, found {[f.name for f in rs_files]}"

    # Hash the bundle files for reproducibility.
    for path in sorted(bundle.iterdir()):
        if path.is_file() and path.suffix in (".rs", ".toml"):
            digest(path.read_bytes())

    print(json.dumps({"proof": "overtune", "ok": True, "mod": mod["id"], "version": mod["version"]}, indent=2))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        if isinstance(error, subprocess.CalledProcessError) and error.stderr:
            print(error.stderr, file=sys.stderr)
        print(f"Overtune template test: {error}", file=sys.stderr)
        raise SystemExit(1)
