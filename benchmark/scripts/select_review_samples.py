#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
from pathlib import Path


DEFAULT_REPORT_ROOT = Path("benchmark/reports/by-directory")
DEFAULT_OUT = Path("benchmark/reports/review-samples.md")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--report-root", default=str(DEFAULT_REPORT_ROOT))
    parser.add_argument("--out", default=str(DEFAULT_OUT))
    parser.add_argument("--limit-per-bucket", type=int, default=10)
    args = parser.parse_args()

    if args.limit_per_bucket < 1:
        raise ValueError("--limit-per-bucket must be >= 1")

    report_root = Path(args.report_root)
    out = Path(args.out)
    buckets = collect_buckets(report_root, args.limit_per_bucket)
    write_report(out, buckets)
    print(out)
    return 0


def collect_buckets(report_root: Path, limit: int) -> dict[str, list[dict[str, str]]]:
    buckets: dict[str, list[dict[str, str]]] = {
        "markitdown-ok-atom-error": [],
        "both-ok": [],
        "markitdown-error-atom-ok": [],
        "both-error": [],
    }
    for directory in sorted(report_root.glob("dir-*")):
        markitdown_rows = rows_by_path(directory / "markitdown-inventory.tsv")
        atom_rows = rows_by_path(directory / "atom-inventory.tsv")
        for path in sorted(set(markitdown_rows) | set(atom_rows)):
            markitdown = markitdown_rows.get(path, {})
            atom = atom_rows.get(path, {})
            key = bucket_key(markitdown.get("status", ""), atom.get("status", ""))
            if key not in buckets or len(buckets[key]) >= limit:
                continue
            buckets[key].append(
                {
                    "directory": directory.name,
                    "path": path,
                    "markitdown_status": markitdown.get("status", ""),
                    "atom_status": atom.get("status", ""),
                    "markitdown_error": markitdown.get("error", ""),
                    "atom_error": atom.get("error", ""),
                }
            )
    return buckets


def rows_by_path(path: Path) -> dict[str, dict[str, str]]:
    if not path.exists():
        return {}
    with path.open(encoding="utf-8", newline="") as file:
        return {row["path"]: row for row in csv.DictReader(file, delimiter="\t")}


def bucket_key(markitdown_status: str, atom_status: str) -> str:
    markitdown_ok = markitdown_status == "ok"
    atom_ok = atom_status == "ok"
    if markitdown_ok and atom_ok:
        return "both-ok"
    if markitdown_ok and not atom_ok:
        return "markitdown-ok-atom-error"
    if not markitdown_ok and atom_ok:
        return "markitdown-error-atom-ok"
    return "both-error"


def write_report(out: Path, buckets: dict[str, list[dict[str, str]]]) -> None:
    out.parent.mkdir(parents=True, exist_ok=True)
    lines = ["# Review Samples", ""]
    for bucket, rows in buckets.items():
        lines.extend(
            [
                f"## {bucket}",
                "",
                "| Directory | MarkItDown | atom | Path | Notes |",
                "| ---- | ---- | ---- | ---- | ---- |",
            ]
        )
        if rows:
            for row in rows:
                notes = row["atom_error"] or row["markitdown_error"]
                lines.append(
                    f"| {cell(row['directory'])} | {cell(row['markitdown_status'])} | "
                    f"{cell(row['atom_status'])} | `{row['path']}` | {cell(notes)} |"
                )
        else:
            lines.append("| - | - | - | - | - |")
        lines.append("")
    out.write_text("\n".join(lines), encoding="utf-8")


def cell(value: str) -> str:
    return value.replace("|", "\\|").replace("\n", " ")


if __name__ == "__main__":
    raise SystemExit(main())
