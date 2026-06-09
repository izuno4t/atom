#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import subprocess
import sys
import tomllib
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path


STATUS_TABLE = Path("benchmark/reports/directory-evaluation-status.tsv")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config")
    parser.add_argument("--status-table", default=str(STATUS_TABLE))
    parser.add_argument("--directory-id", action="append")
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument("--file-jobs", type=int)
    parser.add_argument("--skip-done", action="store_true")
    parser.add_argument("--rerun-all", action="store_true")
    parser.add_argument("--per-file-timeout-seconds", type=int, default=180)
    parser.add_argument("--source-timeout-seconds", type=int, default=120)
    args = parser.parse_args()

    config = load_config(args.config)
    input_root = Path(required(config, "input_dir"))
    status_table = Path(args.status_table)
    rows = read_status_table(status_table)
    targets = select_targets(rows, args.directory_id, args.skip_done)
    if not targets:
        print("no target directories")
        return 0

    if args.jobs < 1:
        raise ValueError("--jobs must be >= 1")
    file_jobs = args.file_jobs or args.jobs
    if file_jobs < 1:
        raise ValueError("--file-jobs must be >= 1")

    failures = 0
    with ThreadPoolExecutor(max_workers=args.jobs) as executor:
        futures = {
            executor.submit(
                run_directory,
                row,
                input_root,
                args.per_file_timeout_seconds,
                args.source_timeout_seconds,
                file_jobs,
                args.rerun_all,
            ): row
            for row in targets
        }
        for future in as_completed(futures):
            row = futures[future]
            try:
                note = future.result()
                update_row(row, "done", note)
                print(f"{row['directory_id']} done: {note}")
            except Exception as error:
                failures += 1
                update_row(row, "error", f"{type(error).__name__}: {error}")
                print(f"{row['directory_id']} error: {error}", file=sys.stderr)
            write_status_table(status_table, rows)
    return 1 if failures else 0


def load_config(config_arg: str | None) -> dict[str, str]:
    config_path = Path(config_arg) if config_arg else Path("benchmark/benchmark.config.toml")
    with config_path.open("rb") as file:
        data = tomllib.load(file)
    return {str(key): str(value) for key, value in data.items()}


def required(config: dict[str, str], key: str) -> str:
    value = config.get(key)
    if not value:
        raise ValueError(f"{key} is required")
    return value


def read_status_table(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as file:
        return list(csv.DictReader(file, delimiter="\t"))


def write_status_table(path: Path, rows: list[dict[str, str]]) -> None:
    fieldnames = [
        "directory_id",
        "directory_name",
        "file_count",
        "markitdown_status",
        "atom_status",
        "comparison_status",
        "notes",
    ]
    with path.open("w", encoding="utf-8", newline="") as file:
        writer = csv.DictWriter(file, fieldnames=fieldnames, delimiter="\t", lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)


def select_targets(
    rows: list[dict[str, str]], directory_ids: list[str] | None, skip_done: bool
) -> list[dict[str, str]]:
    selected = rows
    if directory_ids:
        wanted = set(directory_ids)
        selected = [row for row in selected if row["directory_id"] in wanted]
    selected = [row for row in selected if int(row["file_count"]) > 0]
    if skip_done:
        selected = [
            row
            for row in selected
            if not (
                row["markitdown_status"] == "done"
                and row["atom_status"] == "done"
                and row["comparison_status"] == "done"
            )
        ]
    return sorted(selected, key=lambda row: (int(row["file_count"]), row["directory_name"]))


def run_directory(
    row: dict[str, str],
    input_root: Path,
    per_file_timeout_seconds: int,
    source_timeout_seconds: int,
    file_jobs: int,
    rerun_all: bool,
) -> str:
    directory_id = row["directory_id"]
    input_dir = input_root / row["directory_name"]
    if not input_dir.is_dir():
        raise NotADirectoryError(input_dir)

    report_dir = Path("benchmark/reports/by-directory") / directory_id
    markitdown_output = Path("benchmark/outputs/by-directory/markitdown") / directory_id
    atom_output = Path("benchmark/outputs/by-directory/atom") / directory_id
    report_dir.mkdir(parents=True, exist_ok=True)

    markitdown_inventory = report_dir / "markitdown-inventory.tsv"
    atom_inventory = report_dir / "atom-inventory.tsv"
    markitdown_inspection = report_dir / "markitdown-inspection.tsv"
    atom_inspection = report_dir / "atom-inspection.tsv"
    comparison_report = report_dir / "comparison-report.md"

    run(
        [
            "benchmark/.venv/bin/python",
            "benchmark/scripts/markitdown_inventory.py",
            "--input-dir",
            str(input_dir),
            "--out",
            str(markitdown_inventory),
            "--output-root",
            str(markitdown_output),
            "--per-file-timeout-seconds",
            str(per_file_timeout_seconds),
            "--jobs",
            str(file_jobs),
        ]
        + retry_arguments(rerun_all)
    )
    run(
        [
            "benchmark/.venv/bin/python",
            "benchmark/scripts/atom_inventory.py",
            "--input-dir",
            str(input_dir),
            "--out",
            str(atom_inventory),
            "--output-root",
            str(atom_output),
            "--atom-command",
            "target/debug/atom",
            "--per-file-timeout-seconds",
            str(per_file_timeout_seconds),
            "--jobs",
            str(file_jobs),
        ]
        + retry_arguments(rerun_all)
    )
    run_inspection(
        markitdown_inventory,
        markitdown_output,
        markitdown_inspection,
        report_dir / "markitdown-report.md",
        source_timeout_seconds,
    )
    run_inspection(
        atom_inventory,
        atom_output,
        atom_inspection,
        report_dir / "atom-report.md",
        source_timeout_seconds,
    )
    run(
        [
            "benchmark/.venv/bin/python",
            "benchmark/scripts/compare_atom_markitdown.py",
            "--markitdown-inventory",
            str(markitdown_inventory),
            "--atom-inventory",
            str(atom_inventory),
            "--markitdown-inspection",
            str(markitdown_inspection),
            "--atom-inspection",
            str(atom_inspection),
            "--report",
            str(comparison_report),
        ]
    )
    return write_analysis(report_dir, row)


def run_inspection(
    inventory: Path,
    markdown_root: Path,
    out: Path,
    report: Path,
    source_timeout_seconds: int,
) -> None:
    run(
        [
            "benchmark/.venv/bin/python",
            "benchmark/scripts/inspect_markitdown_outputs.py",
            "--inventory",
            str(inventory),
            "--markdown-root",
            str(markdown_root),
            "--out",
            str(out),
            "--report",
            str(report),
            "--source-timeout-seconds",
            str(source_timeout_seconds),
        ]
    )


def retry_arguments(rerun_all: bool) -> list[str]:
    if not rerun_all:
        return []
    return ["--retry-statuses", "ok,error,empty,timeout"]


def run(command: list[str]) -> None:
    completed = subprocess.run(command, check=False, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    if completed.returncode != 0:
        raise RuntimeError(f"{' '.join(command)}\n{completed.stdout}")
    if completed.stdout.strip():
        print(completed.stdout, end="" if completed.stdout.endswith("\n") else "\n")


def write_analysis(report_dir: Path, row: dict[str, str]) -> str:
    markitdown_rows = read_tsv(report_dir / "markitdown-inventory.tsv")
    atom_rows = read_tsv(report_dir / "atom-inventory.tsv")
    markitdown_counts = count_statuses(markitdown_rows)
    atom_counts = count_statuses(atom_rows)
    atom_unsupported = sum(
        1
        for item in atom_rows
        if item["status"] == "error" and item["error"].startswith("Unsupported input format:")
    )
    markitdown_unsupported = sum(
        1
        for item in markitdown_rows
        if "UnsupportedFormatException" in item["error"]
    )
    note = (
        f"MarkItDown ok={markitdown_counts.get('ok', 0)} error={markitdown_counts.get('error', 0)} "
        f"empty={markitdown_counts.get('empty', 0)}; "
        f"atom ok={atom_counts.get('ok', 0)} error={atom_counts.get('error', 0)} "
        f"empty={atom_counts.get('empty', 0)}"
    )
    lines = [
        f"# {row['directory_name']} Directory Analysis",
        "",
        "## Scope",
        "",
        f"- Directory ID: `{row['directory_id']}`",
        f"- Directory name: `{row['directory_name']}`",
        f"- Direct file count: {row['file_count']}",
        "",
        "## Result",
        "",
        "| Tool | total | ok | empty | error | timeout |",
        "| ---- | ----- | -- | ----- | ----- | ------- |",
        format_counts("MarkItDown", markitdown_rows, markitdown_counts),
        format_counts("atom", atom_rows, atom_counts),
        "",
        "## Cause Classification",
        "",
        "| Classification | Count | Evidence | Improvement Candidate |",
        "| ---- | ----- | ---- | ---- |",
    ]
    if atom_unsupported:
        lines.append(
            f"| 対象外形式の成功扱い修正後の非成功 | {atom_unsupported} | "
            "`atom_inventory.py` が `Unsupported input format:` をerror分類。 | "
            "残り評価ではこの分類を通常成功から除外する。 |"
        )
    if markitdown_unsupported:
        lines.append(
            f"| 入力形式対象外 | {markitdown_unsupported} | "
            "MarkItDownのUnsupportedFormatException。 | 通常文書評価から分離する。 |"
        )
    if not atom_unsupported and not markitdown_unsupported:
        lines.append(
            "| 原因未分類 | 0 | 棚卸し上の明確な非成功原因なし。 | 検査TSVのwarnを全体集計で確認する。 |"
        )
    lines.extend(["", "## Notes", "", f"- {note}"])
    (report_dir / "analysis.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    return note


def read_tsv(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as file:
        return list(csv.DictReader(file, delimiter="\t"))


def count_statuses(rows: list[dict[str, str]]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for row in rows:
        counts[row["status"]] = counts.get(row["status"], 0) + 1
    return counts


def format_counts(tool: str, rows: list[dict[str, str]], counts: dict[str, int]) -> str:
    return (
        f"| {tool} | {len(rows)} | {counts.get('ok', 0)} | {counts.get('empty', 0)} | "
        f"{counts.get('error', 0)} | {counts.get('timeout', 0)} |"
    )


def update_row(row: dict[str, str], status: str, note: str) -> None:
    row["markitdown_status"] = status
    row["atom_status"] = status
    row["comparison_status"] = status
    row["notes"] = note


if __name__ == "__main__":
    raise SystemExit(main())
