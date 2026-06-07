# Evaluation

この文書は、実文書コーパスでatomと既存ツールを比較する手順を定義する。
形式ごとの比較対象ツールは [../tools.md](../tools.md) に定義する。

## 目的

「変換できた」だけでは品質を判断しない。次の観点を分けて記録する。

- 変換成功率
- Markdown構造量
- 見出し、表、画像、コードブロック、リストの保持
- 処理時間
- 失敗理由
- 人間レビューが必要な優位性判定

## 実行方法

```bash
cargo run --bin atom-corpus-eval -- \
  --root evaluation/inputs \
  --out evaluation/reports/report.json \
  --output-root evaluation/outputs \
  --limit 30 \
  --per-ext 5 \
  --tools pandoc,markitdown
```

PDFだけを100件評価する場合は、次のように実行する。

```bash
cargo run --bin atom-corpus-eval -- \
  --root evaluation/inputs \
  --out evaluation/reports/pdf-100-report.json \
  --output-root evaluation/outputs \
  --limit 100 \
  --per-ext 100 \
  --ext pdf \
  --tools docling,pymupdf4llm
```

出力Markdownは `evaluation/outputs/<tool>/` に保存される。

## 比較対象

- `atom`: このリポジトリの変換器
- `pandoc`: Dockerイメージ `atom-eval-pandoc:latest` で実行
- `markitdown`: Dockerイメージ `atom-eval-markitdown:latest` で実行
- `docling`: Dockerイメージ `atom-eval-docling:latest` で実行
- `pymupdf4llm`: Dockerイメージ `atom-eval-pymupdf4llm:latest` で実行
- `mammoth-js`: Dockerイメージ `atom-eval-mammoth-js:latest` で実行

Dockerが未導入、イメージが未作成、または変換に失敗したツールは、report JSONに
`missing` または `error` として記録する。

## 優位性の扱い

自動スコアだけでは「既存ツールより優れている」と断定しない。
report JSONの `superiority_claim` は、人間レビューまたはground truthがない限り
`not_proven_without_human_review_or_ground_truth` とする。

優れていると言えるのは、同じ入力に対して以下を確認した場合に限る。

- 既存ツールより構造保持が高い
- 既存ツールが失敗した入力でatomが有用なMarkdownを出す
- 表、画像、キャプション、コードブロックの破損が少ない
- warning/reportにより失敗原因を追跡できる

## 人手レビュー ルーブリック

DocLayNet型のレイアウト分類を参考に、次の項目を0から2点で採点する。

| 項目 | 0点 | 1点 | 2点 |
| ---- | ---- | ---- | ---- |
| title/heading | 見出しが失われる | 一部だけ復元 | 階層と本文分離が妥当 |
| text/paragraph | 本文が欠落または混線 | 大半は読めるが改行や順序に問題 | 段落単位で読める |
| list | 箇条書きが段落化 | 記号は残るが階層が曖昧 | listとして復元 |
| table | 表が欠落または一次元化 | セルは残るが行列/結合が不完全 | 行列、結合、fallback理由が妥当 |
| figure/image | 画像が欠落 | 画像は出るが位置や説明が曖昧 | media idと本文参照が追跡可能 |
| caption | captionが欠落 | caption候補はあるが対応が曖昧 | 図表とcaptionが対応 |
| footnote | 脚注が本文へ混入 | 脚注らしい文字列だけ残る | footnoteとして分離 |
| formula/value | 数式/表示値が欠落 | どちらかは出るが説明なし | 採用値と根拠がreportに出る |
| reading order | 読順が崩れる | 一部崩れるが推測可能 | 人が読む順序として妥当 |
| warnings | 無警告で壊れる | 警告はあるが原因不明 | 原因、fallback、信頼度が追跡可能 |

## Fixture化判定

実コーパスで0点または1点になった項目は、次の条件を満たす場合にfixture化する。

- 公開データまたは再配布可能データで観測された問題である。
- 問題と同じ文書形式で最小再現できる。
- expected Markdownだけでなく、warningまたはreportで失敗理由を確認できる。
- 元データの出典、ライセンス、観測結果を `evaluation/public-corpus*.md` に記録する。
