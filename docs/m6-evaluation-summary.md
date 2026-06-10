# M6評価サマリ

## 対象

入力ルート直下の22ディレクトリを確認し、直下通常ファイルがある20ディレクトリを
評価対象にした。`Obsidian` と `docs` は直下通常ファイル0件のため対象外とした。

直下通常ファイルは合計809件である。サブディレクトリは辿っていない。

M6再評価では、`atom` が扱える拡張子だけをMarkItDown比較の対象にした。
対象拡張子は `.ai`、`.csv`、`.docx`、`.gdoc`、`.htm`、`.html`、`.md`、
`.pdf`、`.pptx`、`.svg`、`.txt`、`.xlsm`、`.xlsx`、`.xml` である。

`.zip`、`.key`、`.reg`、画像、音声、動画、フォント、旧Office形式などは
比較対象外にした。`atom` に直接渡した場合は従来どおり unsupported document を
Markdownとして返してよいが、MarkItDownとの成功率比較には含めない。

## 評価結果

### 初回全件評価

| Tool | ok | empty | error |
| ---- | -- | ----- | ----- |
| MarkItDown | 438 | 299 | 72 |
| atom | 221 | 0 | 588 |

`atom` の `error` 588件のうち582件は `Unsupported input format` である。
評価プロセスでは、`Unsupported input format:` を本文として出力した場合も
成功扱いせず `error` として分類する。

この数字は全809件を対象にした初回評価であり、扱えない拡張子を含むため、
MarkItDownとの変換能力比較としては使わない。

### supported拡張子のみの再評価

| Tool | ok | empty | error |
| ---- | -- | ----- | ----- |
| MarkItDown | 263 | 7 | 0 |
| atom | 263 | 0 | 7 |

比較対象は270件である。対象外の539件は評価スクリプトで skipped にした。

同一270件の突き合わせ結果は次の通り。

| MarkItDown | atom | 件数 |
| ---------- | ---- | ---: |
| ok | ok | 263 |
| empty | error | 7 |
| ok | error | 0 |

MarkItDownが `ok` で `atom` が `error` の負けケースは0件になった。
残る7件はMarkItDownも空出力であり、`atom` はPDFまたはPDF互換AIの
テキスト抽出不能として `error` に分類している。

### 速度比較

`ok` / `ok` の263件で処理時間を比較した結果は次の通り。

| Tool | total ms | mean ms | median ms | p95 ms |
| ---- | -------: | ------: | --------: | -----: |
| MarkItDown | 698,797 | 2,657.02 | 699 | 13,197 |
| atom | 425,048 | 1,616.15 | 658 | 5,623 |

勝敗件数は `atom` 勝ち153件、同値3件、MarkItDown勝ち107件である。
総時間では `atom` が273,749ms速い。

MarkItDown勝ち107件の遅延合計は59,142msであり、拡張子別ではPDFが82件、
57,395msを占める。これは遅延合計の97.0%であり、小さいファイルが主因ではない。
最大差分は `31_67.pdf` の30,566msで、MarkItDown 1,402msに対し `atom` 31,968ms
だった。

## 拡張子別結果

| 拡張子 | 件数 | MarkItDown | atom |
| ------ | ---: | ---------- | ---- |
| `.pdf` | 209 | ok 204 / empty 5 | ok 204 / error 5 |
| `.md` | 23 | ok 23 | ok 23 |
| `.pptx` | 11 | ok 11 | ok 11 |
| `.ai` | 5 | ok 3 / empty 2 | ok 3 / error 2 |
| `.svg` | 5 | ok 5 | ok 5 |
| `.txt` | 5 | ok 5 | ok 5 |
| `.xlsx` | 5 | ok 5 | ok 5 |
| `.gdoc` | 2 | ok 2 | ok 2 |
| `.csv` | 1 | ok 1 | ok 1 |
| `.docx` | 1 | ok 1 | ok 1 |
| `.html` | 1 | ok 1 | ok 1 |
| `.xlsm` | 1 | ok 1 | ok 1 |
| `.xml` | 1 | ok 1 | ok 1 |

## M6内で修正した変換本体

| 対象 | 対応 |
| ---- | ---- |
| `.md` | Markdown入力を本文として通す。 |
| `.txt` | テキスト入力を本文として通す。 |
| `.csv` | CSVコードブロックとしてMarkdown化する。 |
| `.xml` | XMLコードブロックとしてMarkdown化する。 |
| `.svg` | XMLコードブロックとしてMarkdown化する。 |
| `.gdoc` | JSONコードブロックとしてMarkdown化する。 |
| `.xlsm` | 既存XLSXワークシート解析経路に接続する。 |
| `.ai` | PDF互換AIをPDF解析経路に接続する。 |
| PDF処理 | 通常PDF経路を `atom-pdf-text` として報告し、品質判定つきのPDF抽出経路へ接続する。 |
| PDF処理 | PDF座標異常によるpanicを捕捉し、後続バックエンドへフォールバックする。 |

`.zip` はアーカイブであり、通常文書変換として扱う必要がないため対応対象から外した。
`.key` もZIPベースのパッケージ一覧化に寄せるだけでは文書変換にならないため対象外にした。
`.reg` も通常文書比較の対象ではないため対象外にした。

## M6内で修正した評価プロセス

- `atom_inventory.py` で `Unsupported input format:` 出力を `error` として分類する。
- `markitdown_inventory.py` と `atom_inventory.py` に `--jobs` を追加し、
  ディレクトリ内のファイル処理を並列化する。
- `run_directory_evaluation.py` を追加し、ディレクトリ単位の評価を並列実行する。
- `run_directory_evaluation.py` に `--file-jobs` と `--rerun-all` を追加する。
- MarkItDownと `atom` の棚卸し対象を、`atom` が扱える拡張子だけに揃える。
- Markdown出力ファイル名を短縮名とハッシュにし、長い実ファイル名で失敗しない
  ようにする。

## 判断

M6では、比較評価と本体fixture評価の責務を分ける。外部ツール比較、実コーパスの
棚卸し、ディレクトリ別状態表、生成Markdown、比較レポートは `benchmark/` に置く。
`evaluation/` は `atom` 本体のfixture評価と回帰検出に限定する。

MarkItDownとの比較は、`atom` が扱える拡張子だけに絞る。扱えない拡張子は
CLIとして unsupported document を返してよいが、成功率比較には入れない。

この条件で再評価した結果、MarkItDownが成功し `atom` が失敗するケースは0件になった。
残る7件は、MarkItDownも空出力のPDFまたはPDF互換AIである。

速度負け107件は小さいファイルが主因ではなく、PDF抽出経路が主因である。
M6では総時間でMarkItDownを上回ったが、`31_67.pdf` などPDF個別の外れ値は
残っている。`pdf_extract` を小型PDFで先行させる案も実測したが、同ファイルで
29.9秒かかりpanic文も出たため採用しない。

## 次の改善候補

- OCRなしでは抽出不能なPDFを、エラー分類とユーザー向け説明の観点で整理する。
- `31_67.pdf` を代表とするPDF速度外れ値について、`pdf-rs` に入る前の自前抽出経路を
  追加または改善する。
- `.xls` と `.xlt` の旧Excel形式を、追加パーサ導入または対象外維持のどちらにするか
  判断する。
- supported拡張子の定義を、新しい入力形式追加時に必ず更新する運用にする。
