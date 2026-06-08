#!/usr/bin/env python3
from __future__ import annotations

import csv
import io
import time
from dataclasses import dataclass
from pathlib import Path

import pdfminer.high_level
import pdfplumber
from markitdown.converters._pdf_converter import _extract_form_content_from_words


ATOM_INVENTORY = Path("benchmark/reports/atom-inventory.tsv")
MARKITDOWN_INVENTORY = Path("benchmark/reports/markitdown-inventory.tsv")
OUT = Path("benchmark/reports/pdf-markitdown-impl-probe.tsv")


@dataclass
class ProbeResult:
    path: str
    method: str
    status: str
    elapsed_ms: int
    chars: int
    pages: int
    form_pages: int
    error: str


def main() -> int:
    targets = markitdown_ok_atom_timeout_paths()
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with OUT.open("w", encoding="utf-8", newline="") as file:
        writer = csv.DictWriter(
            file,
            delimiter="\t",
            fieldnames=[
                "path",
                "method",
                "status",
                "elapsed_ms",
                "chars",
                "pages",
                "form_pages",
                "error",
            ],
        )
        writer.writeheader()
        for index, path in enumerate(targets, start=1):
            print(f"{index}/{len(targets)} {path}", flush=True)
            for result in probe_path(path):
                writer.writerow(result.__dict__)
                file.flush()
                print(
                    f"  {result.method}: {result.status} {result.elapsed_ms}ms "
                    f"{result.chars} chars pages={result.pages} forms={result.form_pages}",
                    flush=True,
                )
    print(OUT)
    return 0


def markitdown_ok_atom_timeout_paths() -> list[Path]:
    atom = {row["path"]: row for row in read_tsv(ATOM_INVENTORY)}
    markitdown = {row["path"]: row for row in read_tsv(MARKITDOWN_INVENTORY)}
    paths = []
    for path, atom_row in atom.items():
        markitdown_row = markitdown.get(path)
        if (
            atom_row["status"] == "timeout"
            and markitdown_row is not None
            and markitdown_row["status"] == "ok"
        ):
            paths.append(Path(path))
    return sorted(paths)


def read_tsv(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as file:
        return list(csv.DictReader(file, delimiter="\t"))


def probe_path(path: Path) -> list[ProbeResult]:
    data = path.read_bytes()
    return [
        run_probe(path, "pdfminer-direct", lambda: pdfminer_direct(data)),
        run_probe(path, "pdfplumber-markitdown-shape", lambda: pdfplumber_markitdown_shape(data)),
    ]


def run_probe(path: Path, method: str, callback) -> ProbeResult:
    started = time.monotonic()
    try:
        text, pages, form_pages = callback()
        status = "ok" if text.strip() else "empty"
        error = ""
    except Exception as exc:  # noqa: BLE001 - report probe failures verbatim.
        text = ""
        pages = 0
        form_pages = 0
        status = "error"
        error = f"{type(exc).__name__}: {exc}"
    elapsed_ms = int((time.monotonic() - started) * 1000)
    return ProbeResult(
        path=str(path),
        method=method,
        status=status,
        elapsed_ms=elapsed_ms,
        chars=len(text),
        pages=pages,
        form_pages=form_pages,
        error=error.replace("\t", "\\t").replace("\n", "\\n"),
    )


def pdfminer_direct(data: bytes) -> tuple[str, int, int]:
    text = pdfminer.high_level.extract_text(io.BytesIO(data))
    return text, 0, 0


def pdfplumber_markitdown_shape(data: bytes) -> tuple[str, int, int]:
    pdf_bytes = io.BytesIO(data)
    chunks: list[str] = []
    form_pages = 0
    with pdfplumber.open(pdf_bytes) as pdf:
        pages = len(pdf.pages)
        for page in pdf.pages:
            page_content = _extract_form_content_from_words(page)
            if page_content is not None:
                form_pages += 1
                if page_content.strip():
                    chunks.append(page_content)
            else:
                text = page.extract_text()
                if text and text.strip():
                    chunks.append(text.strip())
            page.close()

    if form_pages == 0:
        pdf_bytes.seek(0)
        return pdfminer.high_level.extract_text(pdf_bytes), pages, form_pages
    return "\n\n".join(chunks).strip(), pages, form_pages


if __name__ == "__main__":
    raise SystemExit(main())
