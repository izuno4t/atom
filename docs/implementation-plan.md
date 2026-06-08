# 実装計画書: ドキュメント-Markdown変換ツール

要件書 `requirements.md` を実装に落とし込むための
進め方をまとめる。

本書は次の3つで構成する。

1. MVPスコープと進め方 — どこから作るか
2. 評価データセット設計 — 何で品質を測るか
3. エージェント駆動ハーネスの具体設計 — Claude Codeをどう回すか

## 1. MVPスコープと進め方

### 設計原則

要件書を一度に全部実装しようとすると確実に詰む。
**最小の閉ループ**を最初に成立させ、評価関数を増やし、入力
フォーマットを増やし、品質を上げる、の順で広げる。

最小の閉ループ:

```text
入力1形式 -> AST -> Markdown -> 評価関数 -> 数値スコア
```

この閉ループが回り始めれば、エージェントは「スコアを上げる」
という単一の目的関数で自律改善できる。閉ループ未成立の状態で
機能を増やすと、改善の体系化ができない。

### マイルストーン

#### M1: 最小閉ループ

骨格と評価ループの成立。

| 項目 | スコープ |
| --- | --- |
| 入力形式 | DOCX のみ |
| 出力形式 | Markdown (CommonMark) のみ |
| AST | 簡易版 (Heading / Paragraph / List / Text のみ) |
| 評価関数 | `structure_fidelity` と `lint_score` の2つ |
| LLM | 無効 |
| OCR | 無効 |
| 配布 | `cargo install` のみ |

DOCX を最初に選ぶ理由は、OOXMLが構造化フォーマットでAST推論が
不要なため、変換ロジックを純粋に評価できるから。
PDFを最初にすると、レイアウト推論の不確実性が評価関数を汚染する。

完了条件:

- `atom input.docx -o out.md` が動く
- `cargo test` が走り、スナップショット比較が回る
- スコアが JSON で出力される

#### M2: ハーネス成立

エージェント駆動の改善ループが回り始める状態を作る。

| 項目 | スコープ |
| --- | --- |
| 入力形式 | DOCX + HTML |
| 評価関数 | `heading_recall` `table_integrity` 追加 |
| ベンチ | `criterion` でパフォーマンス回帰検出 |
| CI | GitHub Actions で eval スコアを時系列保存 |
| エージェント | Claude Code が改善 PR を出せる状態 |

完了条件:

- Claude Code に「スコアを上げて」と頼むと改善PRが出る
- CI で過去スコアと比較して回帰を fail させる仕組みが動く

#### M3: PDFとOCR

| 項目 | スコープ |
| --- | --- |
| 入力形式 | + テキストPDF |
| 評価関数 | `ocr_cer` 追加 |
| OCR | NDLOCR-Lite (subprocess 経由) |
| 構造推論 | フォントサイズベースの見出し階層推論 |

M2 でハーネスが完成しているので、PDFの構造推論ロジックを
エージェントが繰り返し改善できる。

PDF抽出器は atom の中核機能として `src/parsers/pdf/` に実装する。
既存ツールの Markdown 出力へ委譲すると、段落、表、読順、警告、
レポート項目の改善余地が外部実装に固定されるため、MarkItDown、
pdfminer、pdfplumber などの OSS 実装は参照実装として扱う。
Rust crate は PDF 構造、ストリーム展開、演算子走査などの低レベル
部品として使い、文字復元、座標付き単語化、行・表・読順復元は
atom 側の責務に置く。

2026-06-09の性能検証では、MarkItDownがMarkdown化に成功した
直接入力コーパス368件を対象に、releaseビルドのatomで全件成功し、
全件がMarkItDownより高速だった。結果は
`benchmark/reports/atom-release-markitdown-ok.tsv` に保存し、
最悪比率は atom / MarkItDown = 0.548 だった。PDFバックエンドは
ページ数やファイルサイズで分岐せず、文書構造差を吸収できる固定順の
抽出器列として扱う。

同日の再検証では、MarkItDownが成功し旧atom inventoryでは成功して
いなかった34件を現行releaseビルドで再実行し、34件すべてで成功、
34件すべてでMarkItDownより高速だった。詳細は
`docs/markitdown-old-atom-failed-release-check.md`
に保存した。低文字数比の5件はMarkdownサンプルを確認し、本文は
読める状態だった。`designingfortheordinary-220126124909.pdf` に
残っていた2件のCID placeholderはAdobe-Japan1補完で解消した。

MarkItDownが非成功だったPDF/Office文書16件も同じreleaseビルドで
再確認した。対象16件はすべてPDFで、Office文書は含まれていなかった。
15件はページ画像またはアウトライン化された文字のみで抽出可能な
テキスト層がなく、OCRが必要だった。1件はStandard security handlerに
よる権限制限の可能性があった。詳細は
`docs/markitdown-failed-pdf-office-release-check.md`
に保存した。

#### M4: LLM統合

| 項目 | スコープ |
| --- | --- |
| LLM | Anthropic Claude API (`--llm claude-opus`) |
| 機能 | 再構造化、翻訳 |
| 評価関数 | `translation_structure_preserve` 追加 |
| データ送信確認 | 起動時プロンプト実装 |

#### M5以降 (後回し、MVP外)

- PPTX / XLSX 対応
- MDX / HTML 出力
- HedgeDoc スライド形式
- 単一バイナリ配布最適化
- WASM プラグイン
- 古典籍OCR

### MVPでやらないことの明示

| 項目 | 理由 |
| --- | --- |
| PPTX / XLSX | OOXMLバリエーション増。M5以降 |
| スキャンPDF | OCR精度が評価関数を汚染。M3で別軸 |
| 単一バイナリ最適化 | `cargo install` で開発中は十分 |
| 全プラットフォームCI | M2まではLinuxのみで開発速度優先 |
| プラグイン | コア機能の品質確立を優先 |

## 2. 評価データセット設計

### 設計方針

評価関数の数値は、評価データセットの質に依存する。
**入力ファイルと期待結果のペア**を golden として整備し、
これを Git で管理する。

データセットは3層に分ける。

1. **Unit fixtures** — 1ファイル1機能を検証する小さい入力
2. **Integration corpora** — 実際の文書に近い中規模サンプル
3. **Regression set** — 過去にバグを踏んだ実例

### ディレクトリ構造

```text
tests/
├── fixtures/
│   ├── unit/
│   │   ├── docx/
│   │   │   ├── heading-levels.docx
│   │   │   ├── heading-levels.expected.md
│   │   │   ├── heading-levels.meta.json
│   │   │   ├── nested-list.docx
│   │   │   ├── nested-list.expected.md
│   │   │   ├── table-simple.docx
│   │   │   ├── table-with-image.docx
│   │   │   └── ...
│   │   ├── html/
│   │   └── pdf/
│   ├── integration/
│   │   ├── docx/
│   │   │   ├── technical-spec/
│   │   │   │   ├── input.docx
│   │   │   │   ├── expected.md
│   │   │   │   └── meta.json
│   │   │   └── meeting-minutes/
│   │   └── pdf/
│   └── regression/
│       └── issue-001-image-in-table-cell/
│           ├── input.docx
│           ├── expected.md
│           └── README.md  # バグの経緯
└── corpora/  # 大規模、Gitには入れず外部ストレージ
    └── README.md  # ダウンロード手順
```

### Unit fixtures の作り方

1機能を狭く検証するため、入力を最小化する。

例: `heading-levels.docx` の構成

- 内容: H1 → H2 → H3 → H2 → H3 → H4
- 期待 Markdown:

```markdown
# 見出し1

## 見出し2

### 見出し3

## 見出し2-2

### 見出し3-2

#### 見出し4
```

- meta.json:

```json
{
  "description": "見出しレベルの昇降が正しく変換されること",
  "tests": ["structure_fidelity", "heading_recall", "lint_score"],
  "expected_scores": {
    "structure_fidelity": 1.0,
    "heading_recall": 1.0,
    "lint_score": 0
  }
}
```

### Unit fixtures に最低限揃えるもの

DOCX編 (M1で必要):

- 見出し階層 (H1-H6、飛び級なし)
- 見出し階層 (H1→H3 飛び級あり)
- 段落 (改行・空行)
- 順序なしリスト (フラット、ネスト3段)
- 順序ありリスト
- 強調 (bold / italic / 両方)
- インラインコード
- ハイパーリンク
- シンプルテーブル (3x3)
- 結合セルテーブル
- 画像入りセル
- 脚注
- 図キャプション
- コードブロック (等幅フォント自動検出)
- 日本語混在
- 全角空白の扱い

HTML編 (M2で追加):

- 標準的なHTML5
- インラインスタイル付き
- table要素 (rowspan/colspan)
- pre/code
- ブラウザ保存形式 (`<meta>`タグ汚染)
- script/style要素の除去

PDF編 (M3で追加):

- 1段組テキストPDF
- 2段組テキストPDF
- 見出しがフォントサイズで識別できるPDF
- 見出しがフォント太字で識別できるPDF
- 目次付きPDF
- 図表番号付きPDF
- 縦書き日本語PDF
- スキャンPDF (画像PDF)

### Integration corpora

実文書に近いサイズ。1ファイル10〜50ページ程度。

ライセンス上問題ないものを使う:

- 自分で書いた技術文書、議事録、要件書 (NDA対象でないもの)
- 公開資料 (政府白書、IPA、NIST、Wikipediaダンプ)
- パブリックドメイン文書 (青空文庫等)
- NDLデジタルコレクションの公開資料 (NDLOCR-Lite評価と兼用)

評価軸:

- 全体の構造保持率 (見出し復元、章立て)
- リスト・テーブルの整合性
- 数式・コードブロックの破損
- 文字化け率 (とくに日本語)

### Regression set

実装中にバグを踏んだら、最小再現ファイルを `regression/` に
追加する。これが「同じバグを二度踏まない」ハーネスになる。

各 regression エントリの README には:

- 元のバグ症状
- 期待される正しい挙動
- 修正コミットへのリンク
- 関連する評価関数

を記録する。

### Golden の更新ポリシー

期待結果 (expected.md) は手動で書く。
変換ロジック改善で期待結果が変わる場合は、**人間が diff を確認して
承認**してから golden を更新する。エージェントが勝手に golden を
書き換えてはいけない。

実装は `insta` クレートを使い、`cargo insta review` で承認ワーク
フローを回す。

## 3. エージェント駆動ハーネスの設計

### ハーネスの全体像

```text
[評価データセット]
       |
       v
[atom変換実行] --(失敗)--> [エージェント: 修正PR]
       |                            ^
       v                            |
[評価関数群] --(スコア低下)--------+
       |
       v
[メトリクスJSON]
       |
       v
[CI: 過去ベースラインと比較]
```

エージェントが入る箇所は2つ。

1. **失敗ケースを見て修正PRを出す** — テストfailの原因を分析
2. **スコア低下を見て改善PRを出す** — 数値が下がった原因を分析

### CLAUDE.md の構成

リポジトリルートに置く `CLAUDE.md` の骨格。

```markdown
# atom 開発ハーネス

## このリポジトリは何か

ドキュメント (DOCX/PDF/HTML/...) を Markdown に変換する CLI ツール。
要件書は `docs/requirements.md` を参照。

## 改善ループの回し方

`cargo test` で全テストとスコアが出る。スコアが下がった、または
fail があるとき、以下の手順で改善する。

1. `just eval` を実行してスコアサマリを得る
2. 最も低いスコアの評価関数と fixture を特定
3. 該当 fixture の入力と現在出力の diff を確認
4. 原因をパース層 / AST変換層 / 出力層のどこにあるか切り分け
5. 最小修正パッチを作成
6. `cargo test` で回帰なしを確認
7. `just eval` でスコア改善を確認
8. PR を出す

## やってはいけないこと

- `tests/fixtures/*.expected.md` を勝手に書き換えない
  (期待結果の変更は人間レビュー必須)
- 評価関数のしきい値を下げてテストを通さない
- `cargo insta accept` を無条件に実行しない

## 評価関数の意味

(評価関数ごとの定義と目標値を記載)

## アーキテクチャ

(crate構成、AST定義、変換パイプラインの概要)
```

### just タスクの設計

`justfile` で開発タスクを統一する。

```text
# 全テスト + 評価
default:
    cargo test

# 評価関数だけ実行してスコアサマリ
eval:
    cargo test --test eval -- --nocapture
    cat target/eval-report.json | jq '.summary'

# 全fixtureを変換して差分を見る (golden更新の準備)
review:
    cargo insta test --review

# ベンチマーク
bench:
    cargo bench

# 過去スコアとの比較
compare-baseline:
    cargo run --bin compare-baseline -- target/eval-report.json baselines/main.json

# CIで実行する一式
ci: default eval bench compare-baseline
```

### 評価関数の出力フォーマット

エージェントが読みやすいよう、評価結果は構造化JSON で出す。

```json
{
  "timestamp": "2026-05-15T10:00:00Z",
  "commit": "abc123",
  "summary": {
    "total_fixtures": 42,
    "passed": 38,
    "failed": 4,
    "avg_structure_fidelity": 0.87,
    "avg_heading_recall": 0.92,
    "avg_table_integrity": 0.71,
    "lint_total_errors": 3
  },
  "failures": [
    {
      "fixture": "docx/unit/table-with-image",
      "metric": "table_integrity",
      "score": 0.3,
      "expected": 1.0,
      "diff_path": "target/diffs/table-with-image.diff",
      "hint": "セル内画像のMarkdown表現が壊れている可能性"
    }
  ],
  "regressions": [
    {
      "fixture": "docx/unit/heading-levels",
      "metric": "heading_recall",
      "previous": 1.0,
      "current": 0.83,
      "since_commit": "def456"
    }
  ]
}
```

`hint` フィールドはエージェントへのナビゲーション。完全自動ではなく、
人間がドメイン知識をハーネスに埋め込んでおく。

### CI の構成

GitHub Actions で:

1. `cargo test` — 全テスト
2. `just eval` — 評価関数実行、JSON生成
3. `just compare-baseline` — 過去ベースラインとの比較
4. 回帰検出時は PR を fail させる
5. main マージ時、当該コミットのスコアを `baselines/` に追加

ベースラインは Git にコミットして時系列追跡する。

### エージェント呼び出しの典型プロンプト

#### 失敗修正パターン

```text
@claude-code

`cargo test` が fail している。target/eval-report.json の
`failures` セクションを読んで、最もスコアの低い fixture を
1つ選び、原因を分析して修正パッチを作ってほしい。

制約:
- expected.md の書き換えは禁止
- 評価関数のしきい値変更は禁止
- 既存テストの回帰がないことを cargo test で確認

完了したら、何を変えてなぜスコアが上がるかを PR description に
書いてほしい。
```

#### スコア改善パターン

```text
@claude-code

baselines/main.json と直近の target/eval-report.json を比較し、
最もスコアが伸び悩んでいる評価関数を1つ特定して改善案を出してほしい。

順序:
1. どの評価関数か、現在値、目標値を述べる
2. 該当する fixture を3つピックアップ
3. 共通する失敗パターンを分析
4. 最小修正で改善できる箇所を特定
5. パッチを作成し cargo test と just eval で確認
6. PR を出す
```

#### 評価データセット拡充パターン

```text
@claude-code

tests/fixtures/unit/docx/ に不足している fixture を1つ追加してほしい。

順序:
1. 既存 fixture を列挙
2. 要件書の F4 (テーブル忠実度) で扱うべきケースのうち、
   未カバーのものを1つ選ぶ
3. 最小の docx ファイルを生成するスクリプト (Python + python-docx)
   を tests/fixtures/_generators/ に置く
4. expected.md を人間がレビューする前提で雛形を作る
5. meta.json を作成
6. PR description で人間レビュー必要箇所を明示
```

### ガードレール

エージェントが暴走しないための制約。

| ガード | 方法 |
| --- | --- |
| 期待結果の改竄禁止 | `expected.md` の git diff を CI で検知、ラベル必須 |
| 評価関数の改竄禁止 | `src/eval/` の変更は別レビューフロー |
| しきい値の改竄禁止 | しきい値を `tests/thresholds.toml` に分離 |
| LLM 課金の暴走防止 | `--llm` を使うテストは別ジョブ、予算上限あり |
| 機密データの混入防止 | `tests/fixtures/` に PII チェッカー |

### 改善ループの収束条件

エージェントの自律改善を止める判断基準。

1. 全評価関数で目標値 (要件書「評価基準」セクション) を達成
2. 直近 N 回の改善試行でスコアが改善しない (局所最適に到達)
3. 改善のためにアーキテクチャ変更が必要 (人間の判断要)

3つ目に該当したら、エージェントは「これ以上は人間の設計判断が
必要」とPRに書いて止まる。これがエージェント駆動開発の
卒業ポイント。

## 進め方サマリ

| Week | マイルストーン | 主な作業 |
| --- | --- | --- |
| 1-2 | M1 | Rust骨格、DOCXパーサ、AST、Markdown writer、評価関数2つ |
| 3-4 | M2 | HTML対応、評価関数追加、CI、CLAUDE.md整備、エージェント疎通 |
| 5-8 | M3 | PDF対応、NDLOCR-Lite統合、構造推論 |
| 9-12 | M4 | LLM統合、再構造化、翻訳 |
| 13+ | M5+ | 残りのフォーマット、出力拡張、配布最適化 |

## 参考リンク

- 要件書: `doc-to-markdown-requirements.md`
- Rust テストツール: <https://insta.rs/>
- Rust ベンチ: <https://github.com/bheisler/criterion.rs>
- NDLOCR-Lite: <https://github.com/ndl-lab/ndlocr-lite>
- just: <https://github.com/casey/just>
