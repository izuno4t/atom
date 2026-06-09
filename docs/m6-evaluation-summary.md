# M6評価サマリ

## 対象

入力ルート直下の22ディレクトリを確認し、直下通常ファイルがある20ディレクトリを
評価対象にした。`Obsidian` と `docs` は直下通常ファイル0件のため対象外とした。

評価対象ファイルは合計809件である。サブディレクトリは辿っていない。

## 評価結果

| Tool | ok | empty | error |
| ---- | -- | ----- | ----- |
| MarkItDown | 438 | 299 | 72 |
| atom | 221 | 0 | 588 |

`atom` の `error` 588件のうち582件は `Unsupported input format` である。
評価プロセスでは、`Unsupported input format:` を本文として出力した場合も
成功扱いせず `error` として分類する。

## 主な原因分類

| 優先 | 原因 | 件数 | 判断 |
| ---- | ---- | ---- | ---- |
| 1 | `atom` の入力形式対象外 | 582 | M7で対応形式の追加優先度を決める。M6では成功率から分離して記録する。 |
| 2 | MarkItDownの空出力 | 299 | 画像・メディア系が中心。比較対象の非成功として分離する。 |
| 3 | MarkItDownの入力形式対象外 | 45 | 通常文書評価から分離する。 |
| 4 | MarkItDownの変換例外 | 27 | 個別サンプルで確認し、比較不能ケースとして扱う。 |
| 5 | `atom` の未分類エラーまたはpanic | 6 | M7で最初に再現テスト化する。 |

## M6内で修正した評価プロセス

- `atom_inventory.py` で `Unsupported input format:` 出力を `error` として分類する。
- `markitdown_inventory.py` と `atom_inventory.py` に `--jobs` を追加し、
  ディレクトリ内のファイル処理を並列化する。
- `run_directory_evaluation.py` を追加し、ディレクトリ単位の評価を並列実行する。
- `run_directory_evaluation.py` に `--file-jobs` と `--rerun-all` を追加する。
- Markdown出力ファイル名を短縮名とハッシュにし、長い実ファイル名で失敗しない
  ようにする。

## 判断

M6では、比較評価と本体fixture評価の責務を分ける。外部ツール比較、実コーパスの
棚卸し、ディレクトリ別状態表、生成Markdown、比較レポートは `benchmark/` に置く。
`evaluation/` は `atom` 本体のfixture評価と回帰検出に限定する。

変換本体の公開CLI挙動は変更していない。`atom` の対象外形式をCLIレベルで
非ゼロ終了にするかどうかは外部挙動変更に当たるため、M7で方針を決めてから扱う。

## 次の改善候補

- `atom` の未分類エラー5件とpanic1件を再現テスト化する。
- MarkItDown成功かつ `atom` が `Unsupported input format` になった拡張子を集計し、
  M7の対応形式候補を決める。
- 画像・音声・動画・アーカイブなど、通常文書変換と比較すべきでない形式を
  評価対象から分離するフィルタを追加する。
