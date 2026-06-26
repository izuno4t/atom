#!/usr/bin/env python3
"""日本語OCRエンジン比較ドライバ (ocr-rs vs PP-OCRv6)。

各エンジンを同一フィクスチャに通し、CER を算出する。CER 指標は atom 本体
`evaluation/src/metrics.rs::evaluate_ocr_cer` を忠実に再現する:

    score = 1 - min(levenshtein(expected, actual) / max(len(expected), 1), 1)

文字単位の Levenshtein。グループ集計は同 `evaluate_ocr_cer_by_group` と同じく
グループ内で expected/actual を連結してから算出する。

正規化は2系統で報告する:
  strip : 空白・改行を全除去 (レイアウト差を無視し、純粋な文字認識を測る)
  nfkc  : strip に加え NFKC 折り畳み (全角/半角・互換文字の差を吸収)

PP-OCRv6 は paddlepaddle 不要の onnxruntime バックエンドで実行する。
ocr-rs は benchmark/tools/ocr-rs-probe(MNN) を subprocess で叩く。
"""

from __future__ import annotations

import argparse
import json
import subprocess
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EVAL_DIR = Path(__file__).resolve().parent
FIX_DIR = EVAL_DIR / "fixtures"
MODELS = EVAL_DIR / "models"
OUT_DIR = EVAL_DIR / "out"

OCR_RS_PROBE = ROOT / "benchmark/tools/ocr-rs-probe/target/release/ocr-rs-probe"
OCR_RS_DET = MODELS / "ocr-rs/PP-OCRv5_mobile_det.mnn"
OCR_RS_REC = MODELS / "ocr-rs/PP-OCRv5_mobile_rec.mnn"
OCR_RS_CHARSET = MODELS / "ocr-rs/ppocr_keys_v5.txt"


# ---- metric (mirror of atom evaluate_ocr_cer) ----------------------------


def levenshtein(a: str, b: str) -> int:
    if a == b:
        return 0
    if not a:
        return len(b)
    if not b:
        return len(a)
    prev = list(range(len(b) + 1))
    for i, ca in enumerate(a, 1):
        cur = [i]
        for j, cb in enumerate(b, 1):
            cost = 0 if ca == cb else 1
            cur.append(min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + cost))
        prev = cur
    return prev[-1]


def cer_score(expected: str, actual: str) -> tuple[float, int, int]:
    distance = levenshtein(expected, actual)
    total = max(len(expected), 1)
    score = 1.0 - min(distance / total, 1.0)
    return score, distance, total


def norm_strip(s: str) -> str:
    return "".join(s.split())


def norm_nfkc(s: str) -> str:
    return norm_strip(unicodedata.normalize("NFKC", s))


# ---- engines -------------------------------------------------------------


def build_paddleocr_v6(tier: str = "medium"):
    from paddleocr import PaddleOCR

    det = str((MODELS / f"PP-OCRv6_{tier}_det").resolve())
    rec = str((MODELS / f"PP-OCRv6_{tier}_rec").resolve())
    ocr = PaddleOCR(
        text_detection_model_name=f"PP-OCRv6_{tier}_det",
        text_detection_model_dir=det,
        text_recognition_model_name=f"PP-OCRv6_{tier}_rec",
        text_recognition_model_dir=rec,
        use_doc_orientation_classify=False,
        use_doc_unwarping=False,
        use_textline_orientation=False,
        engine="onnxruntime",
        device="cpu",
    )

    def run(png: Path) -> str:
        res = ocr.predict(str(png))
        return "\n".join(res[0].get("rec_texts", []))

    return run


def run_ocr_rs(png: Path) -> str:
    # MNN は能力バナーを stdout に出すため、結果はファイルへ分離して受け取る。
    out = OUT_DIR / "ocr-rs-tmp.txt"
    out.parent.mkdir(parents=True, exist_ok=True)
    proc = subprocess.run(
        [
            str(OCR_RS_PROBE),
            str(png),
            str(OCR_RS_DET),
            str(OCR_RS_REC),
            str(OCR_RS_CHARSET),
            str(out),
        ],
        capture_output=True,
        text=True,
        timeout=120,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"ocr-rs-probe failed: {proc.stderr.strip()[:200]}")
    return out.read_text(encoding="utf-8")


# ---- driver --------------------------------------------------------------


def evaluate(engines: list[str], tier: str) -> dict:
    manifest = json.loads((FIX_DIR / "manifest.json").read_text(encoding="utf-8"))
    runners = {}
    if "paddleocr-v6" in engines:
        runners["paddleocr-v6"] = build_paddleocr_v6(tier)
    if "ocr-rs" in engines:
        if not OCR_RS_PROBE.exists():
            raise SystemExit(f"ocr-rs probe not built: {OCR_RS_PROBE}")
        runners["ocr-rs"] = run_ocr_rs

    rows = []
    for item in manifest:
        png = FIX_DIR / item["png"]
        expected = (FIX_DIR / item["gt"]).read_text(encoding="utf-8")
        row = {k: item[k] for k in ("id", "sample", "font", "orientation")}
        row["engines"] = {}
        for name, run in runners.items():
            try:
                actual = run(png)
            except Exception as e:  # noqa: BLE001
                row["engines"][name] = {"error": str(e)[:200]}
                continue
            s_strip, d_strip, _ = cer_score(norm_strip(expected), norm_strip(actual))
            s_nfkc, _, _ = cer_score(norm_nfkc(expected), norm_nfkc(actual))
            row["engines"][name] = {
                "actual": actual,
                "cer_strip": round(s_strip, 4),
                "cer_nfkc": round(s_nfkc, 4),
                "dist_strip": d_strip,
            }
        rows.append(row)

    # group aggregates (concatenate within group, mirror evaluate_ocr_cer_by_group)
    groups: dict = {}

    def add(gkey: str, name: str, exp: str, act: str):
        groups.setdefault(gkey, {}).setdefault(name, ["", ""])
        groups[gkey][name][0] += exp
        groups[gkey][name][1] += act

    for item in manifest:
        expected = (FIX_DIR / item["gt"]).read_text(encoding="utf-8")
        match = next(r for r in rows if r["id"] == item["id"])
        for name, res in match["engines"].items():
            if "actual" not in res:
                continue
            for gkey in ("overall", f"orient:{item['orientation']}", f"font:{item['font']}"):
                add(gkey, name, norm_strip(expected), norm_strip(res["actual"]))

    aggregates = {}
    for gkey, per_engine in groups.items():
        aggregates[gkey] = {}
        for name, (exp, act) in per_engine.items():
            s, d, t = cer_score(exp, act)
            aggregates[gkey][name] = {"cer_strip": round(s, 4), "dist": d, "chars": t}

    return {"rows": rows, "aggregates": aggregates, "tier": tier, "engines": engines}


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--engines", default="ocr-rs,paddleocr-v6")
    ap.add_argument("--tier", default="medium", choices=["tiny", "small", "medium"])
    ap.add_argument("--out", default=str(OUT_DIR / "result.json"))
    args = ap.parse_args()

    engines = [e.strip() for e in args.engines.split(",") if e.strip()]
    result = evaluate(engines, args.tier)
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    Path(args.out).write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")

    # console summary
    print(f"\n=== overall CER (strip) | tier={args.tier} ===")
    for name, m in result["aggregates"].get("overall", {}).items():
        print(f"  {name:14s} {m['cer_strip']:.4f}  (dist={m['dist']} / {m['chars']} chars)")
    print("\n=== by orientation ===")
    for gkey in sorted(k for k in result["aggregates"] if k.startswith("orient:")):
        print(f"  [{gkey}]")
        for name, m in result["aggregates"][gkey].items():
            print(f"    {name:14s} {m['cer_strip']:.4f}")
    print(f"\nwrote {args.out}")


if __name__ == "__main__":
    main()
