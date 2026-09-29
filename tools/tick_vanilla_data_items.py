#!/usr/bin/env python3
"""Tick CHECKLIST_VANILLA_DATA_RESOURCES.md rows for files a Rust test verified.

The advancement corpus test (`src/advancement_codec_tests.rs`) writes the
checklist paths of every file it loaded, validated and round-tripped to
`vanilla-data/reports/vanilla_data_verified_advancements.txt`.  This script
ticks exactly the unchecked rows whose backticked resource path appears in that
list and whose path lies under one of the requested sections (default:
`advancement/`).  Rows outside those sections are never touched, and a row whose
file is missing from the list stays unticked.

Usage:
    tools/tick_vanilla_data_items.py [--section advancement/] [--dry-run]
                                     [--verified FILE] [--checklist FILE]
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
DEFAULT_VERIFIED = REPO / "vanilla-data/reports/vanilla_data_verified_advancements.txt"
DEFAULT_CHECKLIST = REPO / "CHECKLIST_VANILLA_DATA_RESOURCES.md"
RESOURCE_PREFIX = "decompiled-server-26.1.2/data/minecraft/"
ROW = re.compile(r"^- \[ \] (.*?`(?P<path>[^`]+)`.*)$")


def tick(checklist: Path, verified: set[str], sections: list[str], dry_run: bool) -> int:
    """Rewrites `checklist`, returning how many rows were ticked."""
    prefixes = tuple(RESOURCE_PREFIX + section for section in sections)
    ticked = 0
    lines = checklist.read_text(encoding="utf-8").split("\n")
    for index, line in enumerate(lines):
        match = ROW.match(line)
        if not match:
            continue
        path = match.group("path")
        if path.startswith(prefixes) and path in verified:
            lines[index] = "- [x] " + match.group(1)
            ticked += 1
    if not dry_run:
        checklist.write_text("\n".join(lines), encoding="utf-8")
    return ticked


def main() -> int:
    """CLI entry point."""
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n", maxsplit=1)[0])
    parser.add_argument("--section", action="append", default=None,
                        help="resource path prefix under data/minecraft/ (repeatable)")
    parser.add_argument("--verified", type=Path, default=DEFAULT_VERIFIED)
    parser.add_argument("--checklist", type=Path, default=DEFAULT_CHECKLIST)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    verified = {
        line.strip()
        for line in args.verified.read_text(encoding="utf-8").splitlines()
        if line.strip()
    }
    if not verified:
        print(f"{args.verified} is empty; run the corpus test first", file=sys.stderr)
        return 1
    ticked = tick(args.checklist, verified, args.section or ["advancement/"], args.dry_run)
    print(f"ticked {ticked} row(s)" + (" (dry run)" if args.dry_run else ""))
    return 0


if __name__ == "__main__":
    sys.exit(main())
