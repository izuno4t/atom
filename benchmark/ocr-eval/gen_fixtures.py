#!/usr/bin/env python3
"""日本語OCR比較用の合成フィクスチャ生成器。

既知の日本語テキストを画像へ描画する。描画した元テキストがそのまま正解
(ground truth) になるため、CER を正確かつ再現可能に測れる。横書き/縦書き・
複数フォントの条件を振る。機密文書は一切使わない。

出力:
  fixtures/<id>.png        入力画像
  fixtures/<id>.pdf        画像のみ1ページPDF (atom経路で使う場合用)
  fixtures/<id>.gt.txt     正解テキスト (描画した元テキスト)
  fixtures/manifest.json   各フィクスチャのメタ (font, orientation, language)
"""

from __future__ import annotations

import json
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

FIX_DIR = Path(__file__).parent / "fixtures"

# macOS 同梱の日本語フォント (ゴシック/明朝/丸ゴ)
FONTS = {
    "gothic": "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
    "mincho": "/System/Library/Fonts/ヒラギノ明朝 ProN.ttc",
    "maru": "/System/Library/Fonts/ヒラギノ丸ゴ ProN W4.ttc",
}

# 公開・非機密の一般文。漢字/かな/カナ/数字/記号/英字を網羅する。
SAMPLES = {
    "novel": "吾輩は猫である。名前はまだ無い。\nどこで生れたかとんと見当がつかぬ。",
    "tech": "本システムはPDFや画像を解析し、\n構造化されたMarkdownへ変換する。",
    "number": "売上は前年比123.4%増となった。\n対象は2026年6月26日時点の集計です。",
    "katakana": "OCRエンジンPP-OCRv6はONNX\nRuntimeで動作し、50言語に対応する。",
    "vertical": "縦書きの認識を確認する。\n改行と句読点に注意。",
}

# (sample_id, [fonts], orientation)
PLAN = [
    ("novel", ["gothic", "mincho", "maru"], "horizontal"),
    ("tech", ["gothic", "mincho", "maru"], "horizontal"),
    ("number", ["gothic", "mincho"], "horizontal"),
    ("katakana", ["gothic", "mincho"], "horizontal"),
    ("vertical", ["gothic", "mincho"], "vertical"),
]

FONT_SIZE = 40
PAD = 36
LINE_GAP = 18


def load_font(font_key: str) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(FONTS[font_key], FONT_SIZE, index=0)


def render_horizontal(text: str, font: ImageFont.FreeTypeFont) -> Image.Image:
    lines = text.split("\n")
    dummy = ImageDraw.Draw(Image.new("RGB", (1, 1)))
    widths, heights = [], []
    for line in lines:
        box = dummy.textbbox((0, 0), line, font=font)
        widths.append(box[2] - box[0])
        heights.append(box[3] - box[1])
    line_h = max(heights) if heights else FONT_SIZE
    w = max(widths) + PAD * 2
    h = PAD * 2 + line_h * len(lines) + LINE_GAP * (len(lines) - 1)
    img = Image.new("RGB", (w, h), "white")
    draw = ImageDraw.Draw(img)
    y = PAD
    for line in lines:
        draw.text((PAD, y), line, font=font, fill="black")
        y += line_h + LINE_GAP
    return img


def render_vertical(text: str, font: ImageFont.FreeTypeFont) -> Image.Image:
    # 縦書き: 列を右から左へ、各列は文字を上から下へ積む。
    columns = text.split("\n")
    cell = FONT_SIZE + 8
    col_w = cell
    max_chars = max((len(c) for c in columns), default=1)
    w = PAD * 2 + col_w * len(columns)
    h = PAD * 2 + cell * max_chars
    img = Image.new("RGB", (w, h), "white")
    draw = ImageDraw.Draw(img)
    for ci, col in enumerate(columns):
        x = w - PAD - col_w * (ci + 1)
        y = PAD
        for ch in col:
            draw.text((x, y), ch, font=font, fill="black")
            y += cell
    return img


def main() -> None:
    FIX_DIR.mkdir(parents=True, exist_ok=True)
    manifest = []
    for sample_id, font_keys, orientation in PLAN:
        text = SAMPLES[sample_id]
        for font_key in font_keys:
            fid = f"{sample_id}-{font_key}-{orientation[:1]}"
            font = load_font(font_key)
            if orientation == "vertical":
                img = render_vertical(text, font)
            else:
                img = render_horizontal(text, font)
            png = FIX_DIR / f"{fid}.png"
            pdf = FIX_DIR / f"{fid}.pdf"
            gt = FIX_DIR / f"{fid}.gt.txt"
            img.save(png)
            img.convert("RGB").save(pdf, "PDF", resolution=144.0)
            gt.write_text(text, encoding="utf-8")
            manifest.append(
                {
                    "id": fid,
                    "sample": sample_id,
                    "font": font_key,
                    "orientation": orientation,
                    "language": "ja",
                    "png": png.name,
                    "pdf": pdf.name,
                    "gt": gt.name,
                }
            )
    (FIX_DIR / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    print(f"generated {len(manifest)} fixtures into {FIX_DIR}")


if __name__ == "__main__":
    main()
