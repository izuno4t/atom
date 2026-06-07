# Fixture Design

この文書は、TASK-045の成果物として、現実文書で崩れやすい構造を
unit fixtureと実コーパス評価へ落とす基準を定義する。

## 基本方針

- まず公開データを `evaluation/inputs/` に置き、atomと比較ツールで変換する。
- 変換結果から、見出し、表、読順、図表、caption、media、warningの失敗パターンを
  特定する。
- fixtureは、その失敗パターンと同じ文書形式で作る。
- fixtureは、公開データそのものの複製ではなく、失敗原因を最小化した再現文書にする。
- unit fixtureは小さく、原因を一つずつ切り分けられる形にする。
- 実コーパスは `evaluation/inputs/` に置き、出典、ライセンス、選定理由を
  Git管理されるmanifestへ記録する。
- 期待出力はMarkdownだけでなく、warning、report、media対応も確認対象にする。

## Fixture化フロー

1. 公開データを収集し、出典、ライセンス、形式、選定理由を記録する。
2. `benchmark/` のscriptで比較ツールのMarkdown/reportを出力する。
3. 人が `evaluation/outputs/` を確認し、失敗パターンを分類する。
4. 失敗原因を保ったまま、最小のDOCX/PPTX/XLSX/PDFを作成する。
5. 作成した文書を `tests/fixtures/unit/` または `tests/fixtures/integration/` に置く。
6. expected Markdown、expected warning/report断片、fixtureの由来を記録する。

## Fixture由来の記録

各fixtureには、同じstemの `.meta.json` またはmanifestを置く。

```json
{
  "source_kind": "public-derived",
  "source_url": "https://example.test/source.pdf",
  "source_license": "CC-BY-4.0",
  "source_format": "pdf",
  "fixture_format": "pdf",
  "failure_pattern": "two-column reading order",
  "derived_how": "manual minimization of the observed layout failure"
}
```

## 分類

| 形式 | パターン | unit fixture | 実コーパス評価 |
| ---- | ---- | ---- | ---- |
| DOCX | styleがない見出し、図表番号、画像、脚注、表内画像 | 公開データ由来のDOCX再現 | 必須 |
| PPTX | XML順と視覚順の不一致、text box、shape疑似表、notes | 公開データ由来のPPTX再現 | 必須 |
| XLSX | sharedStrings、inline strings、結合セル、複数ヘッダー、数式表示値 | 公開データ由来のXLSX再現 | 必須 |
| PDF | tag treeあり/なし、複数カラム、header/footer、罫線なし表、脚注 | 公開データ由来のPDF再現 | 必須 |
| HTML | ブラウザ保存HTML、表、pre/code、画像caption | 既存拡張 | 任意 |

## Unit Fixture基準

unit fixtureは、1ファイルにつき主目的を一つに絞る。
ただし、形式は失敗元と同じにする。PPTXで見つけた問題はPPTX fixture、
PDFで見つけた問題はPDF fixtureとして固定する。

| fixture名 | 形式 | 目的 |
| ---- | ---- | ---- |
| `package-with-rels` | OOXML | relationshipsとmedia参照を解決できることを確認する。 |
| `visual-order-shapes` | PPTX | shapeの座標順とplaceholder種別で読順を復元できることを確認する。 |
| `pseudo-table-shapes` | PPTX | 図形配置の疑似表を候補として検出し、低信頼ならwarningに残す。 |
| `merged-header-sheet` | XLSX | mergeCells、sharedStrings、複数ヘッダーを保持する。 |
| `formula-display-sheet` | XLSX | formulaと表示値のどちらを採用したかreportへ出す。 |
| `tagged-reading-order` | PDF | tag tree相当の論理順を優先したことをwarning/reportで確認する。 |
| `layout-reading-order` | PDF | tag treeがないPDFで座標から読順を推定する。 |
| `figure-caption-media` | PDF/OOXML | media id、caption、参照対応を確認する。 |

## 実コーパス基準

実コーパスは、`evaluation/inputs/` のGit管理外ファイルを対象にする。
同じディレクトリに置くmanifestだけをGit管理してよい。

manifestには次を記録する。

- ファイル名または匿名化名
- 形式
- 出典URLまたは内部生成手順
- ライセンスまたは利用許諾
- 含まれる構造パターン
- 比較対象ツール
- 期待する確認観点

## 完了条件

- 各形式に少なくとも一つ、公開データの変換失敗から導いたfixtureを置く。
- 実コーパス評価は100 PDFなどの件数指定に対応しつつ、CIでは実行しない。
- 出力は `evaluation/outputs/` にツール別、入力別に配置し、人がMarkdownとreportを
  並べて確認できるようにする。
