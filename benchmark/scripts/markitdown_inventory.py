#!/usr/bin/env python3
import argparse
import hashlib
import time
from pathlib import Path

from markitdown import MarkItDown


SUPPORTED_EXTENSIONS = {
    ".pdf",
    ".html",
    ".htm",
    ".docx",
    ".pptx",
    ".xlsx",
    ".epub",
    ".txt",
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input-dir", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--output-root", required=True)
    args = parser.parse_args()

    input_dir = Path(args.input_dir)
    out = Path(args.out)
    output_root = Path(args.output_root)
    out.parent.mkdir(parents=True, exist_ok=True)
    output_root.mkdir(parents=True, exist_ok=True)

    rows = ["path\tstatus\telapsed_ms\tchars\tsha256\terror"]
    converter = MarkItDown()
    for path in sorted(input_dir.iterdir()):
        if not path.is_file() or path.suffix.lower() not in SUPPORTED_EXTENSIONS:
            continue
        rows.append(inventory_file(converter, path, output_root))

    out.write_text("\n".join(rows) + "\n", encoding="utf-8")
    print(out)
    return 0


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
