# Golden Review

この文書は、実コーパス比較で `atom` が既存ツールより優れているかを
人手確認するためのgoldenレビュー表を定義する。

## 判定方針

- 既存ツールの出力を正解にしない。
- 入力文書を見て、保持すべき構造を先に固定する。
- 自動スコアは候補順位にだけ使い、優位性の最終判定には使わない。
- `warning` と `report` が失敗理由を説明できる場合は加点対象にする。
- 変換不能、timeout、対象外は成功扱いにしない。

## レビュー項目

| 項目 | 必須条件 | 優位条件 |
| ---- | ---- | ---- |
| 見出し | タイトルと主要節が本文から分離される | 階層が妥当 |
| 本文 | 日本語本文が欠落しない | 読順と段落境界が自然 |
| list | 箇条書き記号や番号が残る | list構造として出る |
| table | 行列が追跡できる | 結合セルや表範囲の理由が出る |
| media | 画像参照が消えない | media idと本文参照が対応する |
| caption | 図表説明が残る | caption候補と距離がreportに出る |
| warning | 失敗が無警告で潰れない | 原因とfallbackが追跡できる |

## Golden Cases

| Case ID | Input | Format | 現状 |
| ---- | ---- | ---- | ---- |
| GOLDEN-001 | `meiji-ppt2020-1-ex1.pptx` | pptx | Pandoc僅差優位 |
| GOLDEN-002 | `meiji-ppt2020-2-ex1.pdf` | pdf | PyMuPDF4LLM優位 |
| GOLDEN-003 | `meiji-ppt2020-3-ex1.pptx` | pptx | Pandoc優位 |
| GOLDEN-004 | `meiji-sample-graph.xlsx` | xlsx | atom自動指標優位 |
| GOLDEN-005 | `mhlw-trial-plan-old.xlsx` | xlsx | atom改善済み |
| GOLDEN-006 | `mhlw-trial-plan-example.pdf` | pdf | PDF改善対象 |
| GOLDEN-007 | `digital-agency-booklet.pdf` | pdf | PDF改善対象 |
| GOLDEN-008 | `meiji-sample-text.docx` | docx | 既存ツール同等 |

## Case Details

### GOLDEN-001

- 期待:
  - `明治大学について` を見出しとして出す。
  - `目次`、`明治大学の理念`、`データでみる明治大学` を節として出す。
  - `建学の精神` のような日本語run分割を単語途中で改行しない。
  - 画像参照を保持する。
- 改善タスク: TASK-056、TASK-059。

### GOLDEN-002

- 期待:
  - PDFから `PDF content requires OCR...` だけを出さない。
  - スライドタイトルを見出しとして出す。
  - 箇条書きをlistとして出す。
  - 抽出backendとfallback理由をreportに出す。
- 改善タスク: TASK-054、TASK-055。

### GOLDEN-003

- 期待:
  - グラフ画像と近傍説明を同じmedia候補として追跡する。
  - 出典URLを本文として保持する。
  - slide上の読順を人が読める順序にする。
- 改善タスク: TASK-056、TASK-059。

### GOLDEN-004

- 期待:
  - sheet単位の見出しを出す。
  - 空行空列に引きずられず、表範囲を分ける。
  - グラフ元データを表として保持する。
- 改善タスク: TASK-057。

### GOLDEN-005

- 期待:
  - `rPh` のふりがなを本文へ混ぜない。
  - 結合セルはHTML tableまたは説明付きfallbackにする。
  - 帳票ラベルと入力欄の位置関係を失わない。
- 改善タスク: TASK-057。

### GOLDEN-006

- 期待:
  - 帳票PDFで主要ラベルを本文として抽出する。
  - 記入例の表構造を一次元テキストへ潰さない。
  - 抽出不能箇所はwarningへ出す。
- 改善タスク: TASK-054、TASK-055。

### GOLDEN-007

- 期待:
  - PDF内部のバイナリ断片を本文として出さない。
  - 冊子見出しと本文を復元する。
  - 画像由来ページではOCR要否をreportに出す。
- 改善タスク: TASK-054、TASK-055。

### GOLDEN-008

- 期待:
  - 日本語本文の段落を保持する。
  - 出典URLを欠落させない。
  - DOCX styleやhyperlinkをreportへ出す。
- 改善タスク: TASK-058。
