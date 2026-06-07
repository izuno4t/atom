# Source Research

この文書は、TASK-044以降の実装と評価で採用する公開資料ベースの判断を記録する。
目的は、`docs/requirements.md` の問題意識を要件文の列挙ではなく、公開仕様、
既存ツール文書、公開ベンチマークから確認できる作業単位へ分解することである。

## 調査方針

- 公式仕様と公式ドキュメントを優先する。
- 公式仕様が実装観点へ直接落としにくい場合は、公式SDK文書や準公式の仕様解説を
  補助資料として扱う。
- 比較対象ツールは、公式READMEまたは公式ドキュメントで確認できる機能だけを
  評価項目にする。
- 公開ベンチマークは、点数そのものよりも分類ラベル、レビュー観点、文書カテゴリを
  ルーブリックへ反映する。

## 採用資料

| 資料 | 信頼度 | 採用理由 |
| ---- | ---- | ---- |
| [ECMA-376 Office Open XML](https://ecma-international.org/publications-and-standards/standards/ecma-376/) | 公式仕様 | DOCX/PPTX/XLSXを部品と関係を持つOOXMLパッケージとして扱う根拠にする。 |
| [Microsoft Learn: Structure of a PresentationML document](https://learn.microsoft.com/en-us/office/open-xml/presentation/structure-of-a-presentationml-document) | 公式文書 | slide、shape、picture、table、notes、master/layoutをPPTX抽出対象に含める根拠にする。 |
| [Microsoft Learn: Shared String Table](https://learn.microsoft.com/en-us/office/open-xml/spreadsheet/working-with-the-shared-string-table) | 公式文書 | XLSXの文字列復元でsharedStringsを解決する根拠にする。 |
| [Microsoft Learn: Merge cells](https://learn.microsoft.com/en-us/office/open-xml/spreadsheet/how-to-merge-two-adjacent-cells-in-a-spreadsheet) | 公式文書 | XLSXの結合セルを表構造復元の対象にする根拠にする。 |
| [PDFlib: Tagged PDF Basics](https://www.pdflib.com/pdf-knowledge-base/pdfua/tagged-pdf-basics/) | 準公式/仕様解説 | PDFでは描画順と論理読順がずれるため、tag treeと座標推定を分けて扱う根拠にする。 |
| [Pandoc User's Guide](https://pandoc.org/MANUAL.html) | 公式文書 | 既存ツールがAST変換で構造を扱う一方、複雑表などで損失があり得る比較軸にする。 |
| [Docling Supported formats](https://docling-project.github.io/docling/usage/supported_formats/) | 公式文書 | PDF/DOCX/PPTX/XLSXを統一表現に変換できる比較対象として扱う根拠にする。 |
| [Microsoft MarkItDown README](https://github.com/microsoft/markitdown/blob/main/README.md) | 公式リポジトリ | Markdown変換の比較対象として扱い、標準出力ではなく出力ファイルとreportで比較する根拠にする。 |
| [PyMuPDF4LLM README](https://github.com/pymupdf/pymupdf4llm) | 公式リポジトリ | PDFの画像、ベクター、ページchunk、OCRを比較観点に含める根拠にする。 |
| [Mammoth.js README](https://github.com/mwilliamson/mammoth.js/) | 公式リポジトリ | DOCXのstyle map依存とMarkdown出力の制約を比較観点にする。 |
| [DocLayNet](https://github.com/DS4SD/DocLayNet) | 公開ベンチ | title、text、table、figure、captionなどの分類と人手レビュー観点をルーブリック化する。 |

## 採用した分解

| 分解軸 | 実装タスク | 評価タスク |
| ---- | ---- | ---- |
| OOXMLパッケージ構造 | TASK-046 | 部品解決、relationships、media参照をreportで確認する。 |
| PresentationMLの視覚構造 | TASK-047 | XML順、座標順、placeholder、shape/tableを確認する。 |
| SpreadsheetMLの表構造 | TASK-048 | sharedStrings、mergeCells、formula/valueを確認する。 |
| PDFの論理構造と読順 | TASK-049 | tag tree、座標推定、header/footer、警告品質を確認する。 |
| 図表キャプション | TASK-050 | media id、caption候補、距離、確信度を追跡する。 |
| 公開ベンチ型の分類 | TASK-051 | title、text、list、table、figure等を採点する。 |
| 実コーパス横比較 | TASK-052 | atomと比較ツールを同じ入力、出力配置、ルーブリックで比較する。 |

## 未採用または後回し

| 候補 | 判断 |
| ---- | ---- |
| 公開PDFそのものをGit管理する | ライセンスとサイズの問題があるため採用しない。 |
| 公開ベンチの学習データを直接同梱する | 変換品質評価には過剰であり、まず分類と採点観点だけを採用する。 |
| 既存ツールの標準出力だけを比較する | Markdown以外のreport、警告、media対応を人が確認できないため採用しない。 |
| CIで実コーパス評価を実行する | 入力データがGit管理外で環境差も大きいため、CIには入れない。 |
