# Benchmark Tools

この文書は、文書形式ごとに `atom` の比較対象とするベンチマークツールを
選定する。

## 選定方針

- Dockerで再現実行できるツールを優先する
- CLIだけでなく、Python/TypeScript/Rustライブラリを薄いラッパーで呼ぶ方式も
  評価対象に含める
- 形式ごとに得意領域が異なるため、単一ツールで全形式を評価しない
- `atom` の要件である人間可読なMarkdown構造を比較できるツールを優先する
- 外部API、GPU、大規模モデルが必要なツールは重いベースラインとして分ける
- PDF入力に非対応など、形式上比較不能なツールは該当形式から除外する

## 形式別の採用ツール

| 形式 | 主要ベースライン | 専門ベースライン | 補助/除外 |
| ---- | ---- | ---- | ---- |
| HTML | Pandoc, Docling, MarkItDown | - | MammothはDOCX専用のため除外 |
| PDF | Docling, PyMuPDF4LLM, Marker | MinerU | PandocはPDF入力非対応のため除外 |
| スキャンPDF | Docling, PyMuPDF4LLM | Marker, MinerU | OCRmyPDF/Tesseractは補助 |
| DOCX | Pandoc, Docling, MarkItDown | Mammoth | Marker/MinerUは重い補助枠 |
| PPTX | Docling, MarkItDown | MinerU | Pandocは採用前に実変換可否を固定する |
| XLSX | Pandoc, Docling, MarkItDown | MinerU | 表構造評価を主指標にする |
| ODT/ODS/ODP | Pandoc, LibreOffice経由 | Docling対応確認後に追加 | 次フェーズ対象 |

## ツール別の扱い

### 実行方式の分類

ベンチマーク対象はCLIツールに限定しない。次の3種類を同列に扱う。

| 種別 | 扱い | 例 |
| ---- | ---- | ---- |
| CLI | Docker内でコマンドを直接実行する | Pandoc, Docling CLI |
| Pythonライブラリ | Docker内のPythonラッパーから呼び出す | PyMuPDF4LLM, Mammoth, MarkItDown |
| TypeScriptライブラリ | Docker内のNode.jsラッパーから呼び出す | Mammoth.js, Turndown |
| Rust/Go等のライブラリ | 小さなCLIラッパーを作って実行する | `lopdf`, `pdf-extract`, Go版MarkItDown |

ライブラリ利用時は、ラッパーの入出力仕様を固定する。

- 入力: `/input/<filename>`
- 出力: 標準出力にMarkdown
- 画像などの副産物: `/output/<tool>/`
- エラー: 標準エラーと終了コード

### Pythonライブラリ候補

| 用途 | ライブラリ | 対象形式 |
| ---- | ---- | ---- |
| PDF軽量抽出 | PyMuPDF4LLM | PDF |
| PDF/Office構造抽出 | Docling | PDF, DOCX, PPTX, XLSX, HTML |
| LLM向け変換 | MarkItDown | PDF, DOCX, PPTX, XLSX, HTML |
| DOCX専門 | Mammoth Python | DOCX |
| OCR前処理 | OCRmyPDF, pytesseract | スキャンPDF、画像 |

### TypeScriptライブラリ候補

TypeScript/Node.js系は、主にOffice文書やHTML変換の比較対象として扱う。
PDF向けの高品質Markdown変換ではなく、HTML中間表現やDOCX専門変換の
ベースラインとして使う。

| 用途 | ライブラリ | 対象形式 |
| ---- | ---- | ---- |
| DOCX専門 | Mammoth.js | DOCX |
| HTMLからMarkdown | Turndown | HTML、Mammoth.jsのHTML出力 |
| Office/汎用変換 | MarkItDown系TypeScriptポート | PDF, DOCX, PPTX, XLSX, HTML |

TypeScriptライブラリ評価では、Node.jsラッパーを用意し、標準出力にMarkdownを
出す。DOCXの場合は `Mammoth.js -> HTML -> Turndown` の経路も明示的に測る。

### Rustライブラリ候補

RustライブラリはMarkdown変換ツールとして完成していないものが多いため、
`atom` の競合ベースラインではなく、処理部品の性能・抽出能力の参考値として扱う。

| 用途 | ライブラリ | 対象形式 |
| ---- | ---- | ---- |
| PDF低レベル解析 | `lopdf` | PDF |
| PDFテキスト抽出 | `pdf-extract` | PDF |
| DOCX/Office ZIP解析 | `zip`, `quick-xml` | DOCX, PPTX, XLSX |

Rustライブラリ評価では、Markdown品質ではなく抽出速度、失敗率、実装複雑度を記録する。

### Pandoc

HTML、DOCX、XLSX、ODT系の汎用ベースラインとして採用する。
PDFについては出力先としては扱えるが、PDF入力からMarkdownへの変換対象ではないため
PDFベンチマークから除外する。

### Docling

PDF、DOCX、PPTX、XLSX、HTMLを横断する主要ベースラインとして採用する。
複数形式をDoclingDocumentに統一してMarkdownへ出力できるため、`atom` の
統一インタフェース要件との比較に向く。

### MarkItDown

LLM/RAG向けMarkdown化の代表として採用する。PDF、Office、HTMLを横断して
比較対象にするが、人間向け高忠実度の基準ツールではなく、軽量ベースラインとして扱う。
Dockerイメージは公式リポジトリのDockerfileからローカルでビルドしたものを使う。

### PyMuPDF4LLM

PDF専用の軽量ベースラインとして採用する。テキストPDF、表、見出し、画像参照、
OCRを含むPDF評価に使う。Office形式には使わない。

### Marker

PDF、画像、Office系に対応する高精度・重めのベースラインとして採用する。
LLMオプションやGPU前提の影響を避けるため、まずはローカル・LLMなし設定で評価する。

### MinerU

PDF、画像、DOCX、PPTX、XLSXを扱える重いベースラインとして採用する。
Docker利用はLinuxまたはWindows WSL2が前提のため、macOSでは通常評価から分ける。

### Mammoth

DOCX専用の専門ベースラインとして採用する。出力はHTML中心のため、
Markdown評価では `mammoth -> HTML -> Markdown` の変換経路を明示する。

### OCRmyPDF/Tesseract

スキャンPDFのOCR下処理ベースラインとして採用する。Markdown変換ツールではないため、
OCR後PDFを別ツールに渡す補助評価として扱う。

## 初期ベンチマークセット

まずは次の組み合わせで固定する。

| 形式 | 採用ツール |
| ---- | ---- |
| HTML | Pandoc, Docling, MarkItDown, Turndown |
| PDF | Docling, PyMuPDF4LLM, Marker |
| スキャンPDF | Docling, PyMuPDF4LLM, Marker, OCRmyPDF/Tesseract |
| DOCX | Pandoc, Docling, MarkItDown, Mammoth, Mammoth.js/Turndown |
| PPTX | Docling, MarkItDown |
| XLSX | Pandoc, Docling, MarkItDown |

## 参照元

- Pandoc User's Guide: <https://pandoc.org/MANUAL.html>
- Docling supported formats: <https://docling-project.github.io/docling/usage/supported_formats/>
- Docling GitHub: <https://github.com/docling-project/docling>
- Microsoft MarkItDown GitHub: <https://github.com/microsoft/markitdown>
- PyMuPDF4LLM GitHub: <https://github.com/pymupdf/pymupdf4llm>
- Marker GitHub: <https://github.com/datalab-to/marker>
- MinerU GitHub: <https://github.com/opendatalab/MinerU>
- Mammoth.js GitHub: <https://github.com/mwilliamson/mammoth.js>
- Turndown GitHub: <https://github.com/mixmark-io/turndown>
