#!/usr/bin/env python3
"""Tick CHECKLIST_VANILLA_DATA_RESOURCES.md lines for explicitly verified resource paths.

Usage: tick_vanilla_data_items.py VERIFIED_PATHS_FILE [--checklist FILE] [--dry-run]

VERIFIED_PATHS_FILE lists one resource path per line (as written in the checklist, e.g.
`decompiled-server-26.1.2/assets/minecraft/lang/en_us.json`; blank lines and `#` comments are
ignored). Only unchecked lines whose backticked path equals a listed path exactly are ticked.
Listed paths matching no checklist line are reported and cause a non-zero exit.
"""
import argparse
import re
import sys
from pathlib import Path

ITEM_RE = re.compile(r"^- \[ \] (.*?`([^`]+)`.*)$")


def tick(lines, verified):
    """Returns (new_lines, ticked_paths); ticks only exact-path matches."""
    out, ticked = [], []
    for line in lines:
        m = ITEM_RE.match(line)
        if m and m.group(2) in verified:
            line = "- [x] " + m.group(1)
            ticked.append(m.group(2))
        out.append(line)
    return out, ticked


def read_paths(path):
    return {
        l.strip()
        for l in Path(path).read_text(encoding="utf-8").splitlines()
        if l.strip() and not l.startswith("#")
    }


def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("verified")
    ap.add_argument("--checklist", default="CHECKLIST_VANILLA_DATA_RESOURCES.md")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args(argv)
    verified = read_paths(args.verified)
    cl = Path(args.checklist)
    new, ticked = tick(cl.read_text(encoding="utf-8").split("\n"), verified)
    missing = verified - set(ticked)
    if not args.dry_run:
        cl.write_text("\n".join(new), encoding="utf-8")
    print(f"ticked {len(ticked)}")
    for p in sorted(missing):
        print(f"not ticked (no unchecked match): {p}", file=sys.stderr)
    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main())
