#!/usr/bin/env python3
"""Fail if coverage drops below the project's floors.

Usage:
    cargo llvm-cov --workspace --json --summary-only --output-path cov.json
    scripts/check-coverage.py cov.json

Floors apply to total line coverage and to every MCP handler file
(`*/src/server.rs`): handlers are what the audit agent actually executes,
and the self-audit found them at 5-23% coverage while pure helpers were
well covered.
"""

import json
import sys

TOTAL_LINES_MIN = 65.0
SERVER_LINES_MIN = 60.0


def main(path: str) -> int:
    data = json.load(open(path))["data"][0]
    failures = []

    total = data["totals"]["lines"]["percent"]
    print(f"total line coverage: {total:.1f}% (floor {TOTAL_LINES_MIN}%)")
    if total < TOTAL_LINES_MIN:
        failures.append(f"total {total:.1f}% < {TOTAL_LINES_MIN}%")

    for entry in sorted(data["files"], key=lambda f: f["filename"]):
        name = entry["filename"]
        if not name.endswith("/src/server.rs"):
            continue
        short = "/".join(name.split("/")[-3:])
        pct = entry["summary"]["lines"]["percent"]
        print(f"  {short}: {pct:.1f}% (floor {SERVER_LINES_MIN}%)")
        if pct < SERVER_LINES_MIN:
            failures.append(f"{short} {pct:.1f}% < {SERVER_LINES_MIN}%")

    if failures:
        print("coverage below floor:\n  " + "\n  ".join(failures))
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1] if len(sys.argv) > 1 else "cov.json"))
