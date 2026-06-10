from __future__ import annotations

from pathlib import Path


ATOM_SUPPORTED_EXTENSIONS = {
    ".ai",
    ".csv",
    ".docx",
    ".gdoc",
    ".htm",
    ".html",
    ".md",
    ".pdf",
    ".pptx",
    ".svg",
    ".txt",
    ".xlsm",
    ".xlsx",
    ".xml",
}


def is_atom_supported_path(path: Path) -> bool:
    return path.suffix.lower() in ATOM_SUPPORTED_EXTENSIONS
