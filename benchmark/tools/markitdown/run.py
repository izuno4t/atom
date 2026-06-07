import json
import sys
from pathlib import Path

from markitdown import MarkItDown


def main() -> int:
    if len(sys.argv) != 4:
        print("usage: markitdown-runner <input> <output-md> <report-json>", file=sys.stderr)
        return 64

    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])
    report_path = Path(sys.argv[3])
    output_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    try:
        result = MarkItDown().convert(str(input_path))
    except Exception as error:
        report = {
            "tool": "markitdown",
            "status": "error",
            "error": f"{type(error).__name__}: {error}",
        }
        report_path.write_text(json.dumps(report, ensure_ascii=False) + "\n", encoding="utf-8")
        print(report["error"], file=sys.stderr)
        return 1

    output_path.write_text(result.text_content, encoding="utf-8")
    report = {
        "tool": "markitdown",
        "status": "ok",
        "output": str(output_path),
        "bytes": len(result.text_content.encode("utf-8")),
    }
    report_path.write_text(json.dumps(report, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps({"tool": "markitdown", "status": "ok", "output": str(output_path)}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
