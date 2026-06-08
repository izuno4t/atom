#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import datetime as dt
import tomllib
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config")
    parser.add_argument("--markitdown-inventory")
    parser.add_argument("--atom-inventory")
    parser.add_argument("--markitdown-inspection")
    parser.add_argument("--atom-inspection")
    parser.add_argument("--report")
    args = parser.parse_args()

    config = load_config(args.config)
    markitdown_inventory = Path(
        value_from_args_or_config(
            args.markitdown_inventory,
            config,
            "inventory_out",
            "benchmark/reports/markitdown-inventory.tsv",
        )
    )
    atom_inventory = Path(
        value_from_args_or_config(
            args.atom_inventory,
            config,
            "atom_inventory_out",
            "benchmark/reports/atom-inventory.tsv",
        )
    )
    markitdown_inspection = Path(
        value_from_args_or_config(
            args.markitdown_inspection,
            config,
            "inspection_out",
            "benchmark/reports/markitdown-inspection.tsv",
        )
    )
    atom_inspection = Path(
        value_from_args_or_config(
            args.atom_inspection,
            config,
            "atom_inspection_out",
            "benchmark/reports/atom-inspection.tsv",
        )
    )
    report = Path(
        value_from_args_or_config(
            args.report,
            config,
            "comparison_report",
            "benchmark/reports/atom-vs-markitdown-report.md",
        )
    )

    markitdown_inventory_rows = read_tsv(markitdown_inventory)
    atom_inventory_rows = read_tsv(atom_inventory)
    markitdown_inspection_rows = read_tsv(markitdown_inspection)
    atom_inspection_rows = read_tsv(atom_inspection)

    report.parent.mkdir(parents=True, exist_ok=True)
    write_report(
        report,
        markitdown_inventory,
        atom_inventory,
        markitdown_inspection,
        atom_inspection,
        markitdown_inventory_rows,
        atom_inventory_rows,
        markitdown_inspection_rows,
        atom_inspection_rows,
    )
    print(report)
    return 0


def load_config(config_arg: str | None) -> dict[str, str]:
    config_path = Path(config_arg) if config_arg else Path("benchmark/benchmark.config.toml")
    if not config_path.exists():
        if config_arg:
            raise FileNotFoundError(f"config file not found: {config_path}")
        return {}
    with config_path.open("rb") as file:
        data = tomllib.load(file)
    return {str(key): str(value) for key, value in data.items()}


def value_from_args_or_config(
    arg_value: str | None, config: dict[str, str], key: str, default: str | None = None
) -> str:
    value = arg_value or config.get(key) or default
    if value is None:
        raise ValueError(f"{key} is required")
    return value


def read_tsv(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as file:
        return list(csv.DictReader(file, delimiter="\t"))


def write_report(
    report: Path,
    markitdown_inventory_path: Path,
    atom_inventory_path: Path,
    markitdown_inspection_path: Path,
    atom_inspection_path: Path,
    markitdown_inventory: list[dict[str, str]],
    atom_inventory: list[dict[str, str]],
    markitdown_inspection: list[dict[str, str]],
    atom_inspection: list[dict[str, str]],
) -> None:
    now = dt.datetime.now(dt.UTC).astimezone().isoformat(timespec="seconds")
    markitdown_by_path = {row["path"]: row for row in markitdown_inventory}
    atom_by_path = {row["path"]: row for row in atom_inventory}
    all_paths = sorted(set(markitdown_by_path) | set(atom_by_path))
    atom_only_ok = [
        path
        for path in all_paths
        if status(atom_by_path.get(path)) == "ok" and status(markitdown_by_path.get(path)) != "ok"
    ]
    markitdown_only_ok = [
        path
        for path in all_paths
        if status(markitdown_by_path.get(path)) == "ok" and status(atom_by_path.get(path)) != "ok"
    ]
    both_ok = [
        path
        for path in all_paths
        if status(markitdown_by_path.get(path)) == "ok" and status(atom_by_path.get(path)) == "ok"
    ]
    neither_ok = [
        path
        for path in all_paths
        if status(markitdown_by_path.get(path)) != "ok" and status(atom_by_path.get(path)) != "ok"
    ]

    atom_inspection_by_path = {row["path"]: row for row in atom_inspection}
    markitdown_inspection_by_path = {row["path"]: row for row in markitdown_inspection}

    lines = [
        f"# atom vs MarkItDown Report - {now}",
        "",
        "## Scope",
        "",
        f"- MarkItDown inventory: `{markitdown_inventory_path}`",
        f"- atom inventory: `{atom_inventory_path}`",
        f"- MarkItDown inspection: `{markitdown_inspection_path}`",
        f"- atom inspection: `{atom_inspection_path}`",
        f"- Compared paths: {len(all_paths)}",
        "",
        "## Conversion Summary",
        "",
        "| Tool | total | ok | empty | error | timeout |",
        "| ---- | ----- | -- | ----- | ----- | ------- |",
        format_inventory_counts("MarkItDown", markitdown_inventory),
        format_inventory_counts("atom", atom_inventory),
        "",
        "## Output Inspection Summary",
        "",
        "| Tool | inspected | language pass | language warn | language unknown | content pass | content warn | content unknown |",
        "| ---- | --------- | ------------- | ------------- | ---------------- | ------------ | ------------ | --------------- |",
        format_inspection_counts("MarkItDown", markitdown_inspection),
        format_inspection_counts("atom", atom_inspection),
        "",
        "## Pairing Summary",
        "",
        f"- Both converted: {len(both_ok)}",
        f"- atom only converted: {len(atom_only_ok)}",
        f"- MarkItDown only converted: {len(markitdown_only_ok)}",
        f"- Neither converted: {len(neither_ok)}",
        "",
        "## atom Warnings",
        "",
        "| Source | Language | Content | Ratio | Overlap | Notes |",
        "| ------ | -------- | ------- | ----- | ------- | ----- |",
    ]
    atom_warn_rows = [
        row
        for row in atom_inspection
        if row["language_status"] == "warn" or row["content_status"] == "warn"
    ]
    lines.extend(format_inspection_issue(row) for row in atom_warn_rows[:50])
    if len(atom_warn_rows) > 50:
        lines.append(f"| truncated | | | | | {len(atom_warn_rows) - 50} more rows in TSV |")

    lines.extend(
        [
            "",
            "## atom Only Converted",
            "",
            "| Source | atom status | MarkItDown status | atom content |",
            "| ------ | ----------- | ----------------- | ------------ |",
        ]
    )
    lines.extend(
        format_pair_row(path, atom_by_path, markitdown_by_path, atom_inspection_by_path)
        for path in atom_only_ok[:50]
    )
    if len(atom_only_ok) > 50:
        lines.append(f"| truncated | | | {len(atom_only_ok) - 50} more rows |")

    lines.extend(
        [
            "",
            "## MarkItDown Only Converted",
            "",
            "| Source | atom status | MarkItDown status | MarkItDown content |",
            "| ------ | ----------- | ----------------- | ------------------ |",
        ]
    )
    lines.extend(
        format_pair_row(path, atom_by_path, markitdown_by_path, markitdown_inspection_by_path)
        for path in markitdown_only_ok[:50]
    )
    if len(markitdown_only_ok) > 50:
        lines.append(f"| truncated | | | {len(markitdown_only_ok) - 50} more rows |")

    report.write_text("\n".join(lines) + "\n", encoding="utf-8")


def status(row: dict[str, str] | None) -> str:
    if row is None:
        return "missing"
    return row["status"]


def format_inventory_counts(tool: str, rows: list[dict[str, str]]) -> str:
    counts = count_values(rows, "status")
    return (
        f"| {tool} | {len(rows)} | {counts.get('ok', 0)} | {counts.get('empty', 0)} | "
        f"{counts.get('error', 0)} | {counts.get('timeout', 0)} |"
    )


def format_inspection_counts(tool: str, rows: list[dict[str, str]]) -> str:
    language = count_values(rows, "language_status")
    content = count_values(rows, "content_status")
    return (
        f"| {tool} | {len(rows)} | {language.get('pass', 0)} | {language.get('warn', 0)} | "
        f"{language.get('unknown', 0)} | {content.get('pass', 0)} | "
        f"{content.get('warn', 0)} | {content.get('unknown', 0)} |"
    )


def count_values(rows: list[dict[str, str]], key: str) -> dict[str, int]:
    counts: dict[str, int] = {}
    for row in rows:
        counts[row[key]] = counts.get(row[key], 0) + 1
    return counts


def format_inspection_issue(row: dict[str, str]) -> str:
    return (
        f"| {escape_cell(Path(row['path']).name)} | {row['language_status']} | "
        f"{row['content_status']} | {row['char_ratio']} | {row['overlap']} | "
        f"{escape_cell(row['notes'])} |"
    )


def format_pair_row(
    path: str,
    atom_by_path: dict[str, dict[str, str]],
    markitdown_by_path: dict[str, dict[str, str]],
    inspection_by_path: dict[str, dict[str, str]],
) -> str:
    inspection = inspection_by_path.get(path)
    content = ""
    if inspection is not None:
        content = f"{inspection['content_status']} ratio={inspection['char_ratio']} overlap={inspection['overlap']}"
    return (
        f"| {escape_cell(Path(path).name)} | {status(atom_by_path.get(path))} | "
        f"{status(markitdown_by_path.get(path))} | {escape_cell(content)} |"
    )


def escape_cell(value: str) -> str:
    return value.replace("|", "\\|").replace("\n", " ")


if __name__ == "__main__":
    raise SystemExit(main())
