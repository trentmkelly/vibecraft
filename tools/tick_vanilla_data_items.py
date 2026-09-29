#!/usr/bin/env python3
"""Tick CHECKLIST_VANILLA_DATA_RESOURCES.md lines for verified resource paths.

Usage: tick_vanilla_data_items.py VERIFIED_PATHS_FILE [CHECKLIST]

VERIFIED_PATHS_FILE holds one resource path per line (as written inside the
backticks of the checklist item). Only lines whose path matches exactly are
ticked; nothing else is modified.
"""

import sys
from pathlib import Path

PREFIX = "- [ ] Audit vanilla data resource `"


def main() -> int:
    verified = {l.strip() for l in Path(sys.argv[1]).read_text().splitlines() if l.strip()}
    target = Path(sys.argv[2] if len(sys.argv) > 2 else "CHECKLIST_VANILLA_DATA_RESOURCES.md")
    out, ticked = [], 0
    for line in target.read_text().splitlines(keepends=True):
        if line.startswith(PREFIX) and line[len(PREFIX):].split("`")[0] in verified:
            line = "- [x]" + line[5:]
            ticked += 1
        out.append(line)
    target.write_text("".join(out))
    print(f"ticked {ticked} of {len(verified)} verified paths")
    return 0


if __name__ == "__main__":
    sys.exit(main())
