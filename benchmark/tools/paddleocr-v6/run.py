#!/usr/bin/env python3
"""PP-OCRv6 を onnxruntime バックエンド (paddlepaddle 不要) で実行する単機能ランナー。

PaddleOCR 3.7 系の共通引数 `engine="onnxruntime"` で ONNX を使う。det/rec の
ONNX モデルディレクトリは Hugging Face の PaddlePaddle/PP-OCRv6_<tier>_*_onnx を
snapshot_download したものを指す (inference.onnx + inference.yml + inference.json)。

使い方:
  python run.py --image foo.png [--tier medium] [--models-dir DIR]
出力:
  認識テキストを行ごとに標準出力へ。
"""

from __future__ import annotations

import argparse
from pathlib import Path

DEFAULT_MODELS = Path(__file__).resolve().parents[2] / "ocr-eval/models"
HF_REPOS = {
    "tier": ["tiny", "small", "medium"],
}


def ensure_models(models_dir: Path, tier: str) -> tuple[str, str]:
    from huggingface_hub import snapshot_download

    det_dir = models_dir / f"PP-OCRv6_{tier}_det"
    rec_dir = models_dir / f"PP-OCRv6_{tier}_rec"
    if not (det_dir / "inference.onnx").exists():
        snapshot_download(f"PaddlePaddle/PP-OCRv6_{tier}_det_onnx", local_dir=det_dir)
    if not (rec_dir / "inference.onnx").exists():
        snapshot_download(f"PaddlePaddle/PP-OCRv6_{tier}_rec_onnx", local_dir=rec_dir)
    return str(det_dir.resolve()), str(rec_dir.resolve())


def build(models_dir: Path, tier: str):
    from paddleocr import PaddleOCR

    det, rec = ensure_models(models_dir, tier)
    return PaddleOCR(
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


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--image", required=True)
    ap.add_argument("--tier", default="medium", choices=["tiny", "small", "medium"])
    ap.add_argument("--models-dir", default=str(DEFAULT_MODELS))
    args = ap.parse_args()

    ocr = build(Path(args.models_dir), args.tier)
    res = ocr.predict(args.image)
    for line in res[0].get("rec_texts", []):
        print(line)


if __name__ == "__main__":
    main()
