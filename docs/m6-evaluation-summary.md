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
| MarkItDown | 248 | 6 | 0 |
| atom | 248 | 0 | 6 |

比較対象は254件である。対象外の拡張子は評価スクリプトで skipped にした。

同一254件の突き合わせ結果は次の通り。

| MarkItDown | atom | 件数 |
| ---------- | ---- | ---: |
| ok | ok | 248 |
| empty | error | 6 |
| ok | error | 0 |

MarkItDownが `ok` で `atom` が `error` の負けケースは0件になった。
残る6件はMarkItDownも空出力であり、`atom` はPDFまたはPDF互換AIの
テキスト抽出不能として `error` に分類している。

### 速度比較

`ok` / `ok` の248件で処理時間を比較した結果は次の通り。

| Tool | total ms | mean ms | median ms | p95 ms |
| ---- | -------: | ------: | --------: | -----: |
| MarkItDown | 681,780 | 2,749.11 | 735.5 | 13,301 |
| atom | 403,416 | 1,626.68 | 684.5 | 6,436 |

勝敗件数は `atom` 勝ち153件、同値2件、MarkItDown勝ち93件である。
総時間では `atom` が278,364ms速い。

MarkItDown勝ち93件の遅延合計は35,513msであり、拡張子別ではPDFが71件、
34,421msを占める。これは遅延合計の96.9%であり、小さいファイルが主因ではない。
`31_67.pdf` はMarkItDown 1,766msに対し `atom` 2,968msで、差分は1,202msまで
縮小した。

## 拡張子別結果

| 拡張子 | 件数 | MarkItDown | atom |
| ------ | ---: | ---------- | ---- |
| `.pdf` | 193 | ok 189 / empty 4 | ok 189 / error 4 |
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
| PDF処理 | 2MiB以下のPDFでは `raw-content` を `pdf-rs` より先に試し、PDF速度外れ値を抑える。 |

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
残る6件は、MarkItDownも空出力のPDFまたはPDF互換AIである。

速度負け93件は小さいファイルが主因ではなく、PDF抽出経路が主因である。
M6では総時間でMarkItDownを上回り、`31_67.pdf` の外れ値も31,968msから
2,968msへ縮小した。`raw-content` を常に先行させる案は `RM-Final.pdf` を
悪化させるため採用せず、supported PDF集合の候補順序測定で成功数とtimeout数が
改善した2MiB以下PDFに限定して採用した。

## 次の改善候補

- OCRなしでは抽出不能なPDFを、エラー分類とユーザー向け説明の観点で整理する。
- `.xls` と `.xlt` の旧Excel形式を、追加パーサ導入または対象外維持のどちらにするか
  判断する。
- supported拡張子の定義を、新しい入力形式追加時に必ず更新する運用にする。
