#!/usr/bin/env python3
"""PP-OCRv6 rec の前処理 + CTC を自前実装し、PaddleOCR の認識結果と一致するか
検証する golden。Rust(ort) 移植の実行可能な仕様を兼ねる。

rec 仕様（inference.yml より）:
  - 入力 x: [N,3,48,W]、CHW、img_mode=BGR
  - RecResizeImg image_shape=[3,48,320]: 高さ48・縦横比維持・最大幅320・0パディング
  - 正規化: (px/255 - 0.5) / 0.5
  - 出力: [N,T,18710]、CTCLabelDecode
  - CTC ラベル: index0=blank, 1..=18708=辞書, 18709=space
"""

from __future__ import annotations

import math
from pathlib import Path

import numpy as np
import onnxruntime as ort
from PIL import Image, ImageDraw, ImageFont

EVAL = Path(__file__).resolve().parent
MODELS = EVAL / "models"
REC_ONNX = MODELS / "PP-OCRv6_medium_rec/inference.onnx"
CHARSET = MODELS / "ppocr_keys_v6.txt"
LINE_DIR = EVAL / "fixtures-line"

FONT = "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc"
LINES = {
    "line-tech": "本システムは画像を解析する",
    "line-num": "売上は前年比123％増",
    "line-ocr": "OCRエンジンPP-OCRv6",
}


def gen_lines() -> None:
    LINE_DIR.mkdir(parents=True, exist_ok=True)
    font = ImageFont.truetype(FONT, 40, index=0)
    for fid, text in LINES.items():
        dummy = ImageDraw.Draw(Image.new("RGB", (1, 1)))
        box = dummy.textbbox((0, 0), text, font=font)
        img = Image.new("RGB", (box[2] - box[0] + 24, box[3] - box[1] + 24), "white")
        ImageDraw.Draw(img).text((12, 6), text, font=font, fill="black")
        img.save(LINE_DIR / f"{fid}.png")
        (LINE_DIR / f"{fid}.gt.txt").write_text(text, encoding="utf-8")


def load_charset() -> list[str]:
    # 1行1文字。改行のみの行(空)は実際の文字ではないので、行末改行のみ除去する。
    raw = CHARSET.read_text(encoding="utf-8")
    chars = raw.split("\n")
    if chars and chars[-1] == "":
        chars.pop()
    return chars


def preprocess(png: Path, imgW: int = 320, imgH: int = 48) -> np.ndarray:
    img = Image.open(png).convert("RGB")
    w, h = img.size
    ratio = w / float(h)
    resized_w = min(imgW, int(math.ceil(imgH * ratio)))
    resized = img.resize((resized_w, imgH), Image.BILINEAR)
    arr = np.asarray(resized).astype(np.float32)  # HWC, RGB
    arr = arr[:, :, ::-1]  # RGB -> BGR
    arr = arr.transpose(2, 0, 1) / 255.0  # CHW
    arr = (arr - 0.5) / 0.5
    padded = np.zeros((3, imgH, imgW), dtype=np.float32)
    padded[:, :, :resized_w] = arr
    return padded[None, ...]  # NCHW


def ctc_decode(logits: np.ndarray, charset: list[str]) -> str:
    # logits: [T, 18710]
    idxs = logits.argmax(axis=1)
    out = []
    prev = -1
    for i in idxs:
        if i != prev and i != 0:  # collapse repeats, drop blank(0)
            if i <= len(charset):
                out.append(charset[i - 1])
            else:
                out.append(" ")  # last index = space
        prev = int(i)
    return "".join(out)


def main() -> None:
    gen_lines()
    charset = load_charset()
    sess = ort.InferenceSession(str(REC_ONNX), providers=["CPUExecutionProvider"])
    in_name = sess.get_inputs()[0].name
    out_name = sess.get_outputs()[0].name

    # reference: PaddleOCR full pipeline
    from paddleocr import PaddleOCR

    ocr = PaddleOCR(
        text_detection_model_name="PP-OCRv6_medium_det",
        text_detection_model_dir=str((MODELS / "PP-OCRv6_medium_det").resolve()),
        text_recognition_model_name="PP-OCRv6_medium_rec",
        text_recognition_model_dir=str((MODELS / "PP-OCRv6_medium_rec").resolve()),
        use_doc_orientation_classify=False,
        use_doc_unwarping=False,
        use_textline_orientation=False,
        engine="onnxruntime",
        device="cpu",
    )

    print(f"{'fixture':12s} {'match':5s}  mine | paddleocr")
    for fid, text in LINES.items():
        png = LINE_DIR / f"{fid}.png"
        x = preprocess(png)
        logits = sess.run([out_name], {in_name: x})[0][0]  # [T,18710]
        mine = ctc_decode(logits, charset)
        ref = "".join(ocr.predict(str(png))[0].get("rec_texts", []))
        ok = "OK" if mine == ref else "DIFF"
        print(f"{fid:12s} {ok:5s}  {mine!r} | {ref!r}")


if __name__ == "__main__":
    main()
