# MarkItDown非成功PDF/Officeのrelease確認

実施日: 2026-06-09

## 対象

- `benchmark/reports/markitdown-inventory.tsv` で MarkItDown が `ok` ではない
- 拡張子が `.pdf`、`.docx`、`.pptx`、`.xlsx`
- 上記条件に一致する16件
- 実行コマンド: `target/release/atom`

対象16件はすべてPDFだった。今回のinventory範囲では、MarkItDownが
非成功だったOffice文書は含まれていない。

`benchmark/reports` はGit管理外の実行成果物置き場であるため、本書に
判断サマリを記録する。

## 結果

| 項目 | 結果 |
| ---- | ---- |
| 対象ファイル数 | 16 |
| PDF | 16 |
| Office文書 | 0 |
| 現行atom成功数 | 0 |
| 読めない理由を分類できた件数 | 16 |
| 抽出可能テキスト層なし | 15 |
| Standard security handler | 1 |

## ファイル別分類

理由コード:

- `ocr_required`: ページ画像またはアウトライン化された文字のみで、
  抽出可能なテキスト層がない。OCRが必要。
- `standard_security_handler`: PDFにStandard security handlerがあり、
  権限制限の可能性がある。

- `10_inui.pdf`: error, `ocr_required`
- `2026_Global_AI_Governance_Playbook.pdf`: error, `ocr_required`
- `9_inui.pdf`: error, `ocr_required`
- `Architecture_Before_Generation.pdf`: error, `ocr_required`
- `Claude_Code_Safety_Handbook.pdf`: error, `ocr_required`
- `Claude_Enterprise_IT_Blueprint.pdf`: error, `ocr_required`
- `Context_Engineering_Blueprint.pdf`: error, `ocr_required`
- `Cross-Encoder_RAG_Blueprint.pdf`: error, `ocr_required`
- `Enterprise_RAG_Blueprint.pdf`: error, `ocr_required`
- `Modern_Search_Toolbox.pdf`: error, `ocr_required`
- `The_Context_Engine_Evolution.pdf`: error, `ocr_required`
- `The_Deep_Agent_Blueprint.pdf`: error, `ocr_required`
- `The_Japan_AI_Strategic_Blueprint.pdf`: error, `ocr_required`
- `finance-summit-keynote01.pdf`: error, `ocr_required`
- `understanding-the-spiral-of-technologies-2025-edition.pdf`:
  error, `ocr_required`
- `tic.pdf`: error, `standard_security_handler`

## 判断

この対象集合は、通常のPDFテキスト抽出器だけではMarkdown化できない。
15件はOCR対象であり、現在のローカル環境では `pdfinfo`、`pdftotext`、
`qpdf`、`tesseract` の各コマンドは見つからなかった。atom本体のOCR境界は
既にあるが、OCRモデルまたはOCRバックエンドが明示設定されていないため、
デフォルトでは本文抽出まで進めない。

`tic.pdf` はStandard security handler由来の失敗として扱う。暗号化または
権限制限付きPDFの扱いは、所有者権限、抽出許可、パスワード入力、警告方針を
分けて設計する必要がある。

CLIのバージョン確認として `atom --version` を追加し、releaseビルドで
`atom 0.1.0` を出力することを確認した。
