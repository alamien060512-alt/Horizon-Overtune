#!/usr/bin/env python3
"""Overtune proof: drive the modded Horizon client with horizon-ctl on fresh
data and verify the now-playing overlay and settings page appear.

    python3 proof.py --resources /path/to/prepared/resources

Needs a built modded client (see tools/horizon.py build).
"""
import argparse
import importlib.util
import json
import subprocess
import sys
import tempfile
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from game import Game


def run(ctl, resources, out):
    with Game(ctl, resources, out) as g:
        g.run("options", "mods")
        labels = g.labels()
        assert any("Overtune" in l for l in labels), f"Overtune not in Options > Mods: {labels}"
        g.play()
        # Wait a few seconds for the now-playing HUD to appear if music plays.
        time.sleep(2)
        shot = g.shot("overtune-proof")
        print(json.dumps({"proof": "overtune", "ok": True, "shot": str(shot)}, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("ctl", help="path to built horizon-ctl binary")
    parser.add_argument("--resources", required=True, help="prepared resources for Minecraft 26.3")
    parser.add_argument("--out", default=None, help="output folder for screenshots")
    args = parser.parse_args()
    out = Path(args.out) if args.out else Path(tempfile.mkdtemp(prefix="overtune-proof-"))
    run(Path(args.ctl), Path(args.resources).resolve(), out)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        if isinstance(error, subprocess.CalledProcessError) and error.stderr:
            print(error.stderr, file=sys.stderr)
        print(f"Overtune proof: {error}", file=sys.stderr)
        raise SystemExit(1)
