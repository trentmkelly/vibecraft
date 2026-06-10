#!/usr/bin/env python3
"""Graph VibeCraft checklist progress from git history.

The script treats Markdown task-list rows in CHECKLIST*.md files as the source
of truth:

    - [x] completed item
    - [ ] incomplete item

Every commit that touched a matching checklist file is sampled, then Plotly is
used to write an interactive HTML graph.
"""

from __future__ import annotations

import argparse
import csv
import fnmatch
import json
import re
import subprocess
import sys
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterable


TASK_RE = re.compile(r"^\s*[-*+]\s+\[([^\]])\]\s+")
DEFAULT_PATTERNS = ("CHECKLIST*.md",)
GIT_TASK_RE = r"^[[:space:]]*[-*+][[:space:]]+\[[^]]\][[:space:]]+"
WORKTREE_REV = "__WORKTREE__"


@dataclass(frozen=True)
class ChecklistSnapshot:
    rev: str
    short_rev: str
    timestamp: datetime
    subject: str
    files: int
    completed: int
    incomplete: int
    total: int

    @property
    def percent_complete(self) -> float:
        if self.total == 0:
            return 0.0
        return self.completed / self.total * 100.0


def run_git(repo: Path, args: list[str], *, text: bool = True) -> str | bytes:
    result = subprocess.run(
        ["git", "-C", str(repo), *args],
        check=True,
        capture_output=True,
        text=text,
    )
    return result.stdout


def git_grep_task_lines(repo: Path, rev: str, patterns: Iterable[str]) -> str:
    result = subprocess.run(
        [
            "git",
            "-C",
            str(repo),
            "grep",
            "-h",
            "-E",
            GIT_TASK_RE,
            rev,
            "--",
            *(git_glob(pattern) for pattern in patterns),
        ],
        capture_output=True,
        text=True,
    )
    if result.returncode == 1:
        return ""
    if result.returncode != 0:
        raise subprocess.CalledProcessError(
            result.returncode,
            result.args,
            output=result.stdout,
            stderr=result.stderr,
        )
    return result.stdout


def git_root(repo: Path) -> Path:
    try:
        return Path(str(run_git(repo, ["rev-parse", "--show-toplevel"])).strip())
    except subprocess.CalledProcessError as exc:
        raise SystemExit(f"{repo} is not inside a git repository") from exc


def git_glob(pattern: str) -> str:
    return f":(glob){pattern}"


def matching_files_at_rev(repo: Path, rev: str, patterns: Iterable[str]) -> list[str]:
    output = run_git(repo, ["ls-tree", "-r", "--name-only", rev])
    tree_files = [line for line in str(output).splitlines() if line]
    return sorted(
        path
        for path in tree_files
        if any(fnmatch.fnmatchcase(path, pattern) for pattern in patterns)
    )


def matching_files_in_worktree(repo: Path, patterns: Iterable[str]) -> list[str]:
    files: set[str] = set()
    for pattern in patterns:
        files.update(path.as_posix() for path in repo.glob(pattern) if path.is_file())
    return sorted(files)


def file_text_in_worktree(repo: Path, path: str) -> str:
    return (repo / path).read_text(encoding="utf-8")


def count_tasks(text: str) -> tuple[int, int]:
    completed = 0
    incomplete = 0
    for line in text.splitlines():
        match = TASK_RE.match(line)
        if match is None:
            continue
        if match.group(1).strip().lower() == "x":
            completed += 1
        else:
            incomplete += 1
    return completed, incomplete


def commit_revs(repo: Path, patterns: Iterable[str]) -> list[str]:
    args = [
        "log",
        "--reverse",
        "--format=%H",
        "--",
        *(git_glob(pattern) for pattern in patterns),
    ]
    return [line for line in str(run_git(repo, args)).splitlines() if line]


def commit_metadata(repo: Path, rev: str) -> tuple[datetime, str]:
    output = str(run_git(repo, ["show", "-s", "--format=%cI%x00%s", rev]))
    timestamp_text, subject = output.rstrip("\n").split("\0", 1)
    return datetime.fromisoformat(timestamp_text), subject


def worktree_metadata() -> tuple[datetime, str]:
    return datetime.now(timezone.utc).astimezone(), "working tree"


def snapshot_for_rev(repo: Path, rev: str, patterns: Iterable[str]) -> ChecklistSnapshot:
    if rev == WORKTREE_REV:
        files = matching_files_in_worktree(repo, patterns)
        timestamp, subject = worktree_metadata()
        short_rev = "worktree"
    else:
        files = matching_files_at_rev(repo, rev, patterns)
        timestamp, subject = commit_metadata(repo, rev)
        short_rev = rev[:12]
        completed, incomplete = count_tasks(git_grep_task_lines(repo, rev, patterns))
        return ChecklistSnapshot(
            rev=rev,
            short_rev=short_rev,
            timestamp=timestamp,
            subject=subject,
            files=len(files),
            completed=completed,
            incomplete=incomplete,
            total=completed + incomplete,
        )

    completed = 0
    incomplete = 0
    for path in files:
        file_completed, file_incomplete = count_tasks(file_text_in_worktree(repo, path))
        completed += file_completed
        incomplete += file_incomplete

    return ChecklistSnapshot(
        rev=rev,
        short_rev=short_rev,
        timestamp=timestamp,
        subject=subject,
        files=len(files),
        completed=completed,
        incomplete=incomplete,
        total=completed + incomplete,
    )


def build_snapshots(
    repo: Path,
    patterns: Iterable[str],
    *,
    include_worktree: bool,
) -> list[ChecklistSnapshot]:
    revs = commit_revs(repo, patterns)
    snapshots = [snapshot_for_rev(repo, rev, patterns) for rev in revs]
    if include_worktree:
        snapshots.append(snapshot_for_rev(repo, WORKTREE_REV, patterns))
    return snapshots


def write_csv(path: Path, snapshots: list[ChecklistSnapshot]) -> None:
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=[
                "rev",
                "short_rev",
                "timestamp",
                "subject",
                "files",
                "completed",
                "incomplete",
                "total",
                "percent_complete",
            ],
        )
        writer.writeheader()
        for snapshot in snapshots:
            row = asdict(snapshot)
            row["timestamp"] = snapshot.timestamp.isoformat()
            row["percent_complete"] = f"{snapshot.percent_complete:.4f}"
            writer.writerow(row)


def write_json(path: Path, snapshots: list[ChecklistSnapshot]) -> None:
    rows = []
    for snapshot in snapshots:
        row = asdict(snapshot)
        row["timestamp"] = snapshot.timestamp.isoformat()
        row["percent_complete"] = snapshot.percent_complete
        rows.append(row)
    path.write_text(json.dumps(rows, indent=2), encoding="utf-8")


def write_plot(path: Path, snapshots: list[ChecklistSnapshot], title: str) -> None:
    try:
        import plotly.graph_objects as go
    except ImportError as exc:
        raise SystemExit("Plotly is required. Install it with: python3 -m pip install plotly") from exc

    x = [snapshot.timestamp for snapshot in snapshots]
    hover = [
        (
            f"<b>{snapshot.short_rev}</b><br>"
            f"{snapshot.subject}<br>"
            f"Files: {snapshot.files}<br>"
            f"Completed: {snapshot.completed}<br>"
            f"Incomplete: {snapshot.incomplete}<br>"
            f"Total: {snapshot.total}<br>"
            f"Complete: {snapshot.percent_complete:.2f}%"
        )
        for snapshot in snapshots
    ]

    fig = go.Figure()
    fig.add_trace(
        go.Scatter(
            x=x,
            y=[snapshot.completed for snapshot in snapshots],
            mode="lines+markers",
            name="Completed",
            hovertext=hover,
            hoverinfo="text+x+y",
        )
    )
    fig.add_trace(
        go.Scatter(
            x=x,
            y=[snapshot.incomplete for snapshot in snapshots],
            mode="lines+markers",
            name="Incomplete",
            hovertext=hover,
            hoverinfo="text+x+y",
        )
    )
    fig.add_trace(
        go.Scatter(
            x=x,
            y=[snapshot.total for snapshot in snapshots],
            mode="lines",
            name="Total checklist items",
            hovertext=hover,
            hoverinfo="text+x+y",
            line={"dash": "dot"},
        )
    )
    fig.add_trace(
        go.Scatter(
            x=x,
            y=[snapshot.percent_complete for snapshot in snapshots],
            mode="lines+markers",
            name="Percent complete",
            hovertext=hover,
            hoverinfo="text+x+y",
            yaxis="y2",
        )
    )

    fig.update_layout(
        title=title,
        template="plotly_white",
        xaxis_title="Commit date",
        yaxis={"title": "Checklist items"},
        yaxis2={
            "title": "Percent complete",
            "overlaying": "y",
            "side": "right",
            "range": [0, 100],
            "ticksuffix": "%",
        },
        hovermode="x unified",
        legend={"orientation": "h", "y": -0.2},
        margin={"l": 70, "r": 80, "t": 70, "b": 90},
    )
    fig.write_html(path, include_plotlyjs=True, full_html=True)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Graph CHECKLIST*.md task completion over VibeCraft git history.",
    )
    parser.add_argument(
        "--repo",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="VibeCraft git repository path. Defaults to this script's parent repo.",
    )
    parser.add_argument(
        "--pattern",
        action="append",
        default=None,
        help="Git glob for checklist files, relative to repo root. Repeatable. Default: CHECKLIST*.md",
    )
    parser.add_argument(
        "--include-working-tree",
        action="store_true",
        help="Append the current working tree as the final snapshot.",
    )
    parser.add_argument(
        "--html",
        type=Path,
        default=Path("checklist_progress.html"),
        help="Output Plotly HTML file.",
    )
    parser.add_argument("--csv", type=Path, help="Optional CSV data output.")
    parser.add_argument("--json", type=Path, help="Optional JSON data output.")
    parser.add_argument(
        "--title",
        default="VibeCraft Checklist Progress",
        help="Plot title.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    repo = git_root(args.repo.resolve())
    patterns = tuple(args.pattern or DEFAULT_PATTERNS)
    snapshots = build_snapshots(repo, patterns, include_worktree=args.include_working_tree)
    if not snapshots:
        patterns_text = ", ".join(patterns)
        print(f"No git history found for checklist pattern(s): {patterns_text}", file=sys.stderr)
        return 1

    write_plot(args.html, snapshots, args.title)
    if args.csv:
        write_csv(args.csv, snapshots)
    if args.json:
        write_json(args.json, snapshots)

    latest = snapshots[-1]
    print(
        f"Wrote {args.html} from {len(snapshots)} snapshots. "
        f"Latest: {latest.completed}/{latest.total} complete "
        f"({latest.percent_complete:.2f}%)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
