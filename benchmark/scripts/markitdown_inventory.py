#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import tomllib
import time
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config")
    parser.add_argument("--input-dir")
    parser.add_argument("--out")
    parser.add_argument("--output-root")
    args = parser.parse_args()

    config = load_config(args.config)
    input_dir = Path(value_from_args_or_config(args.input_dir, config, "input_dir"))
    out = Path(
        value_from_args_or_config(
            args.out, config, "inventory_out", "benchmark/reports/markitdown-inventory.tsv"
        )
    )
    output_root = Path(
        value_from_args_or_config(
            args.output_root,
            config,
            "markdown_output_root",
            "benchmark/outputs/markitdown",
        )
    )
    if not input_dir.is_dir():
        raise NotADirectoryError(f"input_dir is not a directory: {input_dir}")

    out.parent.mkdir(parents=True, exist_ok=True)
    output_root.mkdir(parents=True, exist_ok=True)

    from markitdown import MarkItDown

    converter = MarkItDown()
    files = sorted(path for path in input_dir.iterdir() if path.is_file())
    with out.open("w", encoding="utf-8") as file:
        file.write("path\tstatus\telapsed_ms\tchars\tsha256\terror\n")
        for index, path in enumerate(files, start=1):
            file.write(inventory_file(converter, path, output_root) + "\n")
            file.flush()
            print(f"{index}/{len(files)} {path}", flush=True)

    print(out)
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


def inventory_file(converter: MarkItDown, path: Path, output_root: Path) -> str:
    started = time.monotonic()
    try:
        result = converter.convert(str(path))
        markdown = result.text_content
    except Exception as error:
        return row(path, "error", started, 0, "", f"{type(error).__name__}: {error}")

    elapsed_ms = int((time.monotonic() - started) * 1000)
    if not markdown.strip():
        return row(path, "empty", started, 0, "", "runner produced empty markdown output")

    output_path = output_root / f"{safe_name(str(path))}.md"
    output_path.write_text(markdown, encoding="utf-8")
    digest = hashlib.sha256(markdown.encode("utf-8")).hexdigest()
    return "\t".join([tsv(path), "ok", str(elapsed_ms), str(len(markdown)), digest, ""])


def row(path: Path, status: str, started: float, chars: int, digest: str, error: str) -> str:
    elapsed_ms = int((time.monotonic() - started) * 1000)
    return "\t".join([tsv(path), status, str(elapsed_ms), str(chars), digest, tsv(error)])


def safe_name(value: str) -> str:
    return "".join(
        character if character.isalnum() or character in "-_" else "_" for character in value
    )


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
