#!/usr/bin/env python3
"""Validation: with a Horizon Git checkout containing the pinned commit,
stage the Overtune bundle on the pinned source and check it compiles.
"""
import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
COMPAT = json.loads((ROOT / "compatibility.json").read_text())
PIN = COMPAT["horizon_revision"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--horizon", type=Path, required=True,
                        help="Horizon Git checkout containing the pinned commit")
    args = parser.parse_args()
    horizon = args.horizon.resolve()
    if not (horizon / ".git").exists():
        raise ValueError("--horizon must be a Git checkout")
    subprocess.run(["git", "-C", str(horizon), "cat-file", "-e", PIN + "^{commit}"], check=True, capture_output=True)

    mixins = Path(os.environ.get("HORIZON_MIXINS", ROOT.parent / "horizon-mixins")).resolve()
    revision = subprocess.check_output(["git", "-C", str(mixins), "rev-parse", "HEAD"], text=True).strip()
    if revision != COMPAT["horizon_mixins_revision"]:
        raise ValueError(f"horizon-mixins must be at {COMPAT['horizon_mixins_revision']}, got {revision}")

    # Export pinned sources.
    import tempfile, tarfile, hashlib
    temporary = Path(tempfile.mkdtemp(prefix=".overtune-export-"))
    process = subprocess.Popen(["git", "-C", str(horizon), "archive", "--format=tar", PIN], stdout=subprocess.PIPE)
    try:
        with tarfile.open(fileobj=process.stdout, mode="r|") as archive:
            for member in archive:
                archive.extract(member, temporary, filter="data")
    finally:
        process.stdout.close()
        code = process.wait()
    if code:
        raise ValueError(f"git archive exited {code}")

    def digest(data):
        return hashlib.sha256(data).hexdigest()

    for path, expected in COMPAT["source_sha256"].items():
        if digest((temporary / path).read_bytes()) != expected:
            raise ValueError(f"pinned source hash mismatch: {path}")

    # Build horizon-mixins CLI.
    subprocess.run(["cargo", "build", "--locked", "--offline", "--target-dir", str(mixins / "target"), "-p", "horizon-mixins-cli"], cwd=mixins, check=True, stdout=sys.stderr)
    cli = mixins / "target/debug/horizon-mixins"

    # Plan and apply.
    out = temporary / "modded"
    plan = temporary / "plan.json"
    base = ["--workspace", str(temporary), "--mods", str(ROOT / "mixins")]
    subprocess.run([str(cli), "plan", *base, "--out", str(plan)], check=True, capture_output=True, text=True)
    applied = subprocess.run([str(cli), "apply", *base, "--out", str(out)], check=True, capture_output=True, text=True)
    report = json.loads(applied.stdout)
    print(json.dumps({"stage_changed": report["stage_changed"], "horizon": PIN, "mixins": revision}, indent=2))

    # Verify the stage marker exists.
    marker = out / ".horizon-mod-template.json"
    if not marker.is_file():
        raise SystemExit("stage marker missing after apply")
    print(json.dumps({"proof": "overtune", "ok": True}, indent=2))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        if isinstance(error, subprocess.CalledProcessError) and error.stderr:
            print(error.stderr, file=sys.stderr)
        print(f"Overtune stage test: {error}", file=sys.stderr)
        raise SystemExit(1)
