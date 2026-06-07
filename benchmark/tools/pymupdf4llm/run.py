import json
import sys
from pathlib import Path

import pymupdf4llm


def main() -> int:
    if len(sys.argv) != 4:
        print("usage: pymupdf4llm-runner <input> <output-md> <report-json>", file=sys.stderr)
        return 64

    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])
    report_path = Path(sys.argv[3])
    markdown = pymupdf4llm.to_markdown(str(input_path))
    output_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(markdown, encoding="utf-8")
    report = {
        "tool": "pymupdf4llm",
        "output": str(output_path),
        "bytes": len(markdown.encode("utf-8")),
    }
    report_path.write_text(json.dumps(report, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps({"tool": "pymupdf4llm", "status": "ok", "output": str(output_path)}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
