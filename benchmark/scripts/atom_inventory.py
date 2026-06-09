#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import subprocess
import tomllib
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config")
    parser.add_argument("--input-dir")
    parser.add_argument("--out")
    parser.add_argument("--output-root")
    parser.add_argument("--atom-command")
    parser.add_argument("--per-file-timeout-seconds", type=int)
    parser.add_argument("--retry-statuses")
    parser.add_argument("--jobs", type=int, default=1)
    args = parser.parse_args()

    config = load_config(args.config)
    input_dir = Path(value_from_args_or_config(args.input_dir, config, "input_dir"))
    out = Path(
        value_from_args_or_config(
            args.out, config, "atom_inventory_out", "benchmark/reports/atom-inventory.tsv"
        )
    )
    output_root = Path(
        value_from_args_or_config(
            args.output_root,
            config,
            "atom_markdown_output_root",
            "benchmark/outputs/atom",
        )
    )
    atom_command = value_from_args_or_config(
        args.atom_command, config, "atom_command", "target/debug/atom"
    )
    timeout_seconds = args.per_file_timeout_seconds
    if timeout_seconds is None:
        timeout_seconds = int(config.get("atom_per_file_timeout_seconds", "180"))
    retry_statuses = parse_statuses(args.retry_statuses or config.get("atom_retry_statuses", ""))

    if not input_dir.is_dir():
        raise NotADirectoryError(f"input_dir is not a directory: {input_dir}")
    if args.jobs < 1:
        raise ValueError("--jobs must be >= 1")

    out.parent.mkdir(parents=True, exist_ok=True)
    output_root.mkdir(parents=True, exist_ok=True)

    files = sorted(path for path in input_dir.iterdir() if path.is_file())
    completed_paths = prepare_existing_inventory(out, retry_statuses)
    pending_files = [
        (index, path)
        for index, path in enumerate(files, start=1)
        if tsv(path) not in completed_paths
    ]
    print(
        f"resume: {len(completed_paths)} recorded, {len(pending_files)} pending",
        flush=True,
    )

    write_header = not out.exists()
    with out.open("a", encoding="utf-8") as file:
        if write_header:
            file.write("path\tstatus\telapsed_ms\tchars\tsha256\terror\n")
        for _index, result in run_inventory_jobs(
            pending_files, atom_command, output_root, timeout_seconds, args.jobs, len(files)
        ):
            file.write(result + "\n")
            file.flush()

    print(out)
    return 0


def run_inventory_jobs(
    pending_files: list[tuple[int, Path]],
    atom_command: str,
    output_root: Path,
    timeout_seconds: int,
    jobs: int,
    total_files: int,
) -> list[tuple[int, str]]:
    if jobs == 1:
        results = []
        for index, path in pending_files:
            result = inventory_file(atom_command, path, output_root, timeout_seconds)
            results.append((index, result))
            print(f"{index}/{total_files} {path}", flush=True)
        return results

    results: dict[int, str] = {}
    with ThreadPoolExecutor(max_workers=jobs) as executor:
        futures = {
            executor.submit(inventory_file, atom_command, path, output_root, timeout_seconds): (
                index,
                path,
            )
            for index, path in pending_files
        }
        for future in as_completed(futures):
            index, path = futures[future]
            results[index] = future.result()
            print(f"{index}/{total_files} {path}", flush=True)
    return sorted(results.items())


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


def parse_statuses(value: str) -> set[str]:
    return {status.strip() for status in value.split(",") if status.strip()}


def prepare_existing_inventory(out: Path, retry_statuses: set[str]) -> set[str]:
    if not out.exists():
        return set()
    lines = out.read_text(encoding="utf-8").splitlines()
    if not retry_statuses:
        return completed_paths(lines)

    retained = [lines[0]] if lines else ["path\tstatus\telapsed_ms\tchars\tsha256\terror"]
    for line in lines[1:]:
        columns = line.split("\t")
        if len(columns) < 2 or columns[1] in retry_statuses:
            continue
        retained.append(line)
    out.write_text("\n".join(retained) + "\n", encoding="utf-8")
    return completed_paths(retained)


def completed_paths(lines: list[str]) -> set[str]:
    return {line.split("\t", 1)[0] for line in lines[1:] if line.strip() and "\t" in line}


def inventory_file(atom_command: str, path: Path, output_root: Path, timeout_seconds: int) -> str:
    started = time.monotonic()
    output_path = output_root / f"{safe_name(str(path))}.md"
    output_path.unlink(missing_ok=True)
    try:
        completed = subprocess.run(
            [atom_command, str(path), "--output", str(output_path)],
            check=False,
            capture_output=True,
            text=True,
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired:
        return row(
            path,
            "timeout",
            started,
            0,
            "",
            f"atom conversion exceeded {timeout_seconds} seconds",
        )

    stderr = completed.stderr.strip()
    stdout = completed.stdout.strip()
    if completed.returncode != 0:
        output_path.unlink(missing_ok=True)
        message = stderr or stdout or f"atom exited with status {completed.returncode}"
        return row(path, "error", started, 0, "", message)

    if not output_path.exists():
        message = stderr or stdout or "atom did not create output file"
        return row(path, "error", started, 0, "", message)

    markdown = output_path.read_text(encoding="utf-8", errors="replace")
    if not markdown.strip():
        return row(path, "empty", started, 0, "", "atom produced empty markdown output")
    unsupported_message = unsupported_input_message(markdown)
    if unsupported_message:
        output_path.unlink(missing_ok=True)
        return row(path, "error", started, 0, "", unsupported_message)

    digest = hashlib.sha256(markdown.encode("utf-8")).hexdigest()
    elapsed_ms = int((time.monotonic() - started) * 1000)
    return "\t".join([tsv(path), "ok", str(elapsed_ms), str(len(markdown)), digest, ""])


def unsupported_input_message(markdown: str) -> str:
    first_line = markdown.splitlines()[0].strip() if markdown.splitlines() else ""
    if first_line.startswith("Unsupported input format:"):
        return first_line
    return ""


def row(path: Path, status: str, started: float, chars: int, digest: str, error: str) -> str:
    elapsed_ms = int((time.monotonic() - started) * 1000)
    return "\t".join([tsv(path), status, str(elapsed_ms), str(chars), digest, tsv(error)])


def safe_name(value: str) -> str:
    stem = "".join(
        character if character.isascii() and (character.isalnum() or character in "-_") else "_"
        for character in value
    ).strip("_")
    digest = hashlib.sha256(value.encode("utf-8")).hexdigest()[:16]
    if not stem:
        stem = "file"
    return f"{stem[:120]}_{digest}"


def tsv(value: object) -> str:
    return (
        str(value)
        .replace("\\", "\\\\")
        .replace("\t", "\\t")
        .replace("\r", "\\r")
        .replace("\n", "\\n")
    )


if __name__ == "__main__":
    raise SystemExit(main())
