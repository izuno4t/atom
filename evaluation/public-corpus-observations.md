# Public Corpus Observations

公開データを `evaluation/inputs/` に置き、atomで変換した結果から
fixture化すべき失敗パターンを記録する。

## 2026-05-17 MarkItDown公式テストファイル

入力はMicrosoft MarkItDown公式リポジトリの `packages/markitdown/tests/test_files/`
から取得した。ファイル本体と変換出力はGit管理外に置く。

実行コマンド:

```bash
cargo run --bin atom-corpus-eval -- \
  --root evaluation/inputs/markitdown \
  --out evaluation/reports/markitdown-public.json \
  --output-root evaluation/outputs/markitdown-public \
  --limit 4 \
  --per-ext 4 \
  --tools atom
```

## 観測した失敗パターン

| Corpus ID | 形式 | 観測 |
| ---- | ---- | ---- |
| PCORPUS-001 | PDF | OCR必須のfallback文だけになった。警告品質fixtureが必要。 |
| PCORPUS-002 | PPTX | run分割で `AutoGen:` が見出しと本文に割れた。 |
| PCORPUS-002 | PPTX | 画像は抽出されたが、caption候補とmedia idの対応が不足。 |
| PCORPUS-003 | XLSX | sheet名、表範囲、数式/表示値、結合セルの説明が不足。 |
| PCORPUS-004 | DOCX | tableとimageは出たが、styleなし見出し、caption/media対応、脚注などの確認が必要。 |

## Fixture化方針

- PPTXのrun分割と視覚順序の問題は、同じPPTX形式の最小再現fixtureへ落とす。
- XLSXの表範囲、結合セル、数式表示値の問題は、同じXLSX形式の最小再現fixtureへ落とす。
- PDFのOCR fallbackは、テキスト抽出不可PDFまたは画像PDFの最小再現fixtureへ落とす。
- DOCXのcaption/media対応は、同じDOCX形式の最小再現fixtureへ落とす。

## 2026-05-17 日本語公式公開文書

日本語文書は、国の機関、独立行政法人、大学が公開している文書を主要評価にする。
教材サイトや個人配布に近い文書も補助コーパスとして残し、失敗パターンの幅を広げる。

実行コマンド:

```bash
cargo run --bin atom-corpus-eval -- \
  --root evaluation/inputs/japanese-official \
  --out evaluation/reports/japanese-official.json \
  --output-root evaluation/outputs/japanese-official \
  --limit 10 \
  --per-ext 10 \
  --tools atom
```

| Corpus ID | 形式 | 観測 |
| ---- | ---- | ---- |
| PCORPUS-005 | PDF | 厚労省PDFはバイナリ断片が本文へ混入した。 |
| PCORPUS-006 | PDF | 大阪大学PDFもバイナリ断片と日本語抽出問題がある。 |
| PCORPUS-007 | XLSX | sharedStrings未解決に見える数値が出ている。 |
| PCORPUS-008 | XLSX | 実体がHTMLで、入力妥当性確認の対象にする。 |
| PCORPUS-010 | XLSX | sharedStrings内のふりがなが本文へ混入した。 |
| PCORPUS-012 | PDF | 冊子PDFでバイナリ断片が本文として抽出された。 |
| PCORPUS-013 | PPTX | 日本語文がrun単位で改行され、単語と文が割れた。 |
| PCORPUS-014 | PPTX | 画像と近傍説明の対応はまだreportで追跡できない。 |

## 2026-05-17 追加対応

- 厚労省XLSX由来のふりがな混入を `phonetic-shared-string` fixtureへ落とし、
  sharedStringsの `rPh` を本文から除外した。
- 明治大学PPTX由来の日本語run分割を `split-run-japanese` fixtureへ落とし、
  同一段落内のrunは結合するようにした。
- PDFの明らかなバイナリ断片はスキップするようにしたが、冊子PDFにはまだ
  フォント名や短い断片が残るため、TASK-049は未完了のままにする。

## 2026-05-17 外部ツール比較

実行コマンド:

```bash
cargo run --bin atom-corpus-eval -- \
  --root evaluation/inputs/japanese-university \
  --out evaluation/reports/japanese-university-tools.json \
  --output-root evaluation/outputs/japanese-university-tools \
  --limit 6 \
  --per-ext 6 \
  --tools markitdown,pymupdf4llm,mammoth-js,pandoc
```

観測:

- `meiji-sample-graph.xlsx` は自動指標上atomが最上位だった。
- PPTX、PDF、複数XLSXではPandoc、PyMuPDF4LLM、MarkItDownが上位だった。
- Mammoth.jsはDOCXでは成功し、PPTX/XLSX/PDFでは対象外エラーになった。
- 自動指標だけでは優位性を確定できないため、人手レビューが必要。

## 優位性獲得の問題点

| 問題 | 既存ツールとの差 | 対応タスク |
| ---- | ---- | ---- |
| PDF本文抽出が弱い | PyMuPDF4LLMは日本語PDF本文を多く保持した | TASK-054 |
| PDF構造推定が弱い | PyMuPDF4LLMは見出し/listを多く出した | TASK-055 |
| PPTX本文構造が薄い | PandocはPPTXの構造量がわずかに多い | TASK-056 |
| XLSX表範囲が弱い | MarkItDown/Pandocは表数が多い | TASK-057 |
| DOCX意味構造が同等止まり | Mammoth.jsとの差別化要素が少ない | TASK-058 |
| media対応が説明不足 | 画像とcaption候補をreportで比較できない | TASK-059 |
| 優位性判定が曖昧 | 自動スコアだけでは正しさを判定できない | TASK-053 |

日本語fixtureでは、少なくとも次を含める。

- 省庁または独立行政法人の帳票型XLSXから導いた結合セル/日本語sharedStrings。
- 大学公開PDFから導いたスライド由来PDFの読順と日本語文字抽出。
- 日本語の全角記号、かな、漢字、英数字混在の見出し・表・注記。
