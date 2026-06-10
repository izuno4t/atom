# ディレクトリ別評価実行計画

この文書は、指定された入力ルート配下のディレクトリを単位として、
MarkItDown棚卸し、`atom`棚卸し、比較、原因分析、改善候補抽出を
順に実施するための計画を定義する。

## 目的

多数の実文書を一度に雑に流すのではなく、ディレクトリ単位で評価を完了させる。
各ディレクトリで成功率、処理時間、文字数、Markdownハッシュ、失敗理由を
蓄積し、最終的に十分なサンプル数から `atom` の処理精度と改善優先度を
判断できる状態にする。

## 前提

- 入力ルートの実パスはGit管理対象へ保存しない。
- 実文書、Markdown出力、TSVレポートはGit管理外に置く。
- サブディレクトリは辿らない。
- 各ディレクトリ直下の通常ファイルだけを評価対象にする。
- MarkItDownとの比較は、`atom` が扱える拡張子だけを対象にする。
  対象外拡張子は棚卸し前に除外し、比較成功率には含めない。
- 外部ツール比較は `benchmark/` の責務として扱う。
- `evaluation/` は `atom` 本体のfixture評価と回帰検出に限定する。

## 成果物

| 成果物 | 保存先 | Git管理 |
| ---- | ---- | ---- |
| 実行計画 | `benchmark/methods/directory-evaluation-plan.md` | 管理対象 |
| ディレクトリ別状態表 | `benchmark/reports/directory-evaluation-status.tsv` | 管理外 |
| MarkItDown棚卸し | `benchmark/reports/by-directory/` | 管理外 |
| `atom`棚卸し | `benchmark/reports/by-directory/` | 管理外 |
| Markdown出力 | `benchmark/outputs/by-directory/` | 管理外 |
| 集計サマリ | `benchmark/reports/directory-evaluation-summary.md` | 管理外 |
| 判断サマリ | `docs/` | 管理対象 |

管理外レポートは途中再開と詳細確認に使う。
管理対象の判断サマリには、対象条件、件数、分類結果、改善方針だけを残す。

## 実行順序

### 1. 対象ディレクトリ一覧を作成する

入力ルート直下のディレクトリだけを列挙する。
各ディレクトリについて、直下の通常ファイル数を数える。

状態表には次を記録する。

- `directory_id`
- `directory_name`
- `file_count`
- `markitdown_status`
- `atom_status`
- `comparison_status`
- `notes`

`directory_id` は、実行順が安定するよう `dir-001` から順に振る。
絶対パスはGit管理対象へ書かない。

### 2. 実行対象を選ぶ

ファイル数が0件のディレクトリは評価対象外として記録する。
通常ファイルがあるディレクトリは、ファイル数が少ない順に実行する。
同数の場合はディレクトリ名順にする。

この順序により、早く完了するディレクトリから結果を蓄積し、
長いディレクトリで止まっても途中成果が残る。

### 3. MarkItDown棚卸しを実行する

各ディレクトリに対して `benchmark/scripts/markitdown_inventory.py` を実行する。
この棚卸しは `benchmark/scripts/atom_supported_extensions.py` の定義に従い、
`atom` が扱える拡張子だけを処理する。

記録列は次を使う。

- `path`
- `status`
- `elapsed_ms`
- `chars`
- `sha256`
- `error`

`status` は `ok`、`empty`、`error`、`timeout` を区別する。
既存TSVがある場合は記録済みファイルをスキップし、途中再開できるようにする。

### 4. `atom`棚卸しを実行する

同じディレクトリに対して `benchmark/scripts/atom_inventory.py` を実行する。
この棚卸しもMarkItDown棚卸しと同じsupported拡張子定義を使う。
実行前に `atom --version` を記録し、どのバイナリで評価したかを固定する。

MarkItDownと同じ列を記録し、出力Markdownを
`benchmark/outputs/by-directory/atom/<directory_id>/` に保存する。

### 5. ディレクトリ内比較を作成する

MarkItDownと `atom` のTSVを突き合わせ、次の分類を作る。

- MarkItDown成功、`atom`成功
- MarkItDown成功、`atom`非成功
- MarkItDown非成功、`atom`成功
- MarkItDown非成功、`atom`非成功

成功同士では、処理時間、文字数比、Markdownハッシュ差分を記録する。
非成功を含むものでは、失敗理由を分類する。

### 6. 原因分析する

原因は、ファイル形式と失敗メッセージから最低限次へ分類する。

- PDFのテキスト層なし
- PDFのUnicode map不足
- PDFの権限制限
- PDFの読順またはレイアウト復元不足
- Office package部品解決不足
- Office文書種別固有部品不足
- Writer整形による欠落
- 入力形式対象外
- timeout
- 原因未分類

`原因未分類` は残してよいが、最終判断では改善候補として優先的に見る。

### 7. 改善候補を抽出する

改善候補は原因単位でまとめる。
同じ原因が複数ディレクトリで出た場合は、件数が多いものを優先する。

優先度は次の順に決める。

1. MarkItDown成功、`atom`非成功の件数が多い原因
2. MarkItDown非成功、`atom`も非成功だが理由分類が粗い原因
3. `atom`成功だが文字数比や人手確認で品質不安が大きい原因
4. 処理時間が極端に遅い原因

改善は原因単位で行い、複数原因を1つの変更に混ぜない。

### 8. 再評価する

改善後は、改善前と同じディレクトリ集合とsupported拡張子定義で再評価する。
対象集合またはsupported拡張子定義を変えた場合は、前後比較として扱わず、
別スコープの評価として記録する。

再評価では、次を確認する。

- 成功数が増えたか
- `error`、`empty`、`timeout` が減ったか
- 失敗理由の未分類が減ったか
- 処理時間が悪化していないか
- 代表サンプルのMarkdownが読めるか

## 自動化runner

`TASK-005` から `TASK-010` までの初期評価で、対象外形式を `atom` が
Markdown本文として出力し、棚卸し上は `ok` になる共通原因が見つかった。
このため、`benchmark/scripts/atom_inventory.py` は
`Unsupported input format:` で始まる出力を `error` として記録する。

残りディレクトリは `benchmark/scripts/run_directory_evaluation.py` で再開できる。
このrunnerは状態表を読み、ディレクトリ単位でMarkItDown棚卸し、`atom`棚卸し、
出力検査、比較レポート、分析メモ作成、状態表更新を順に実行する。

```bash
benchmark/.venv/bin/python benchmark/scripts/run_directory_evaluation.py \
  --status-table benchmark/reports/directory-evaluation-status.tsv \
  --skip-done \
  --jobs 2
```

個別ディレクトリだけを再実行する場合は `--directory-id dir-011` のように指定する。
複数指定もできる。

```bash
benchmark/.venv/bin/python benchmark/scripts/run_directory_evaluation.py \
  --directory-id dir-011 \
  --directory-id dir-013 \
  --jobs 2
```

`--jobs` はディレクトリ単位の並列数である。
単一ディレクトリ内のファイル処理は `--file-jobs` で並列化する。
`--file-jobs` を省略した場合は `--jobs` と同じ値を使う。

```bash
benchmark/.venv/bin/python benchmark/scripts/run_directory_evaluation.py \
  --directory-id dir-007 \
  --jobs 1 \
  --file-jobs 4
```

既存の棚卸し結果を作り直す場合は `--rerun-all` を指定する。
このオプションは `ok`、`error`、`empty`、`timeout` を再試行対象にし、
対象ディレクトリのTSVを実質的に再生成する。

```bash
benchmark/.venv/bin/python benchmark/scripts/run_directory_evaluation.py \
  --directory-id dir-007 \
  --jobs 1 \
  --file-jobs 4 \
  --rerun-all
```

Markdown出力ファイル名は、元ファイルパスをASCII安全な短縮名に変換し、
SHA-256の短い接尾辞を付ける。これは長い実ファイル名でOSのファイル名長上限を
超えないようにするためである。

人手確認用のサンプルは `benchmark/scripts/select_review_samples.py` で抽出する。
このスクリプトはディレクトリ別の棚卸しTSVを読み、次の分類から指定件数ずつ
`benchmark/reports/review-samples.md` に出力する。

- MarkItDown成功、`atom`非成功
- 両方成功
- MarkItDown非成功、`atom`成功
- 両方非成功

```bash
benchmark/.venv/bin/python benchmark/scripts/select_review_samples.py \
  --limit-per-bucket 10
```

## 停止条件

次のいずれかに該当した場合、ディレクトリ単位の処理を止めて記録する。

- そのディレクトリの全ファイルが棚卸し済み
- per-file timeoutにより個別ファイルが停止した
- 比較対象ツールの環境が壊れている
- `atom` バイナリが起動できない
- 入力ディレクトリへアクセスできない

個別ファイルのtimeoutはディレクトリ全体の停止理由にしない。
timeout行を記録して、次のファイルへ進む。

## 完了条件

この評価プロセスの完了条件は次の通り。

- 対象ルート直下の全ディレクトリについて状態表が作成されている
- 通常ファイルがある各ディレクトリでMarkItDown棚卸しが完了している
- 同じ各ディレクトリで `atom` 棚卸しが完了している
- ディレクトリごとの比較分類が作成されている
- 全体集計で成功率、処理時間、失敗理由内訳が確認できる
- 改善候補が原因単位で列挙されている
- 最終判断サマリがGit管理対象の `docs/` に保存されている

## 次に実行する作業

この計画を承認済みの実行手順として扱い、次の順序で進める。

1. 入力ルート直下のディレクトリ一覧を作成する。
2. `benchmark/reports/directory-evaluation-status.tsv` を作成する。
3. ファイル数が少ないディレクトリからMarkItDown棚卸しを実行する。
4. 同じディレクトリで `atom` 棚卸しを実行する。
5. ディレクトリごとの比較を作成する。
6. 全ディレクトリ完了後に全体集計と判断サマリを作成する。
