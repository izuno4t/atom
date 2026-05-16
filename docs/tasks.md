# TASKS

マイルストーン: M1-M5
ゴール: docs/requirements.md の主要要件を段階的に満たす実装計画を確定する

## ワークフロールール

- タスク着手時にステータスを 🚧 に更新する
- タスク完了時にステータスを ✅ に更新する
- DependsOn のタスクがすべて ✅ でないタスクには着手しない

## ステータス表記ルール

| Status | 意味 |
| ---- | ----- |
| ⏳ | 未着手、TODO |
| 🚧 | 作業中、IN_PROGRESS |
| 🧪 | 確認待ち、REVIEW |
| ✅ | 完了、DONE |
| 🚫 | 中止、CANCELLED |

## タスク一覧

| ID | Status | Summary | DependsOn |
| ---- | ---- | ---- | ---- |
| TASK-001 | ✅ | 定義するRustワークスペースとCLI骨格 | - |
| TASK-002 | ✅ | 定義する中間ASTの最小モデル | TASK-001 |
| TASK-003 | ✅ | 実装するDOCX入力の基本パーサ | TASK-002 |
| TASK-004 | ✅ | 実装するCommonMarkライター | TASK-002 |
| TASK-005 | ✅ | 接続する変換パイプラインの最小閉ループ | TASK-003,TASK-004 |
| TASK-006 | ✅ | 整備するDOCX用unit fixture | TASK-005 |
| TASK-007 | ✅ | 実装するgolden testとsnapshot比較 | TASK-006 |
| TASK-008 | ✅ | 実装するstructure_fidelity評価 | TASK-007 |
| TASK-009 | ✅ | 実装するlint_score評価 | TASK-007 |
| TASK-010 | ✅ | 出力する評価レポートJSON | TASK-008,TASK-009 |
| TASK-011 | ✅ | 固定するM1の開発コマンド群 | TASK-010 |
| TASK-012 | ✅ | 実装するHTML入力パーサ | TASK-011 |
| TASK-013 | ✅ | 追加するHTML用unit fixture | TASK-012 |
| TASK-014 | ✅ | 実装するMarkdown flavor選択 | TASK-011 |
| TASK-015 | ✅ | 実装するheading_recall評価 | TASK-013,TASK-014 |
| TASK-016 | ✅ | 実装するtable_integrity評価 | TASK-013,TASK-014 |
| TASK-017 | ✅ | 整備するintegration corpora構造 | TASK-016 |
| TASK-018 | ✅ | 構築するCI評価ベースライン | TASK-017 |
| TASK-019 | ✅ | 整備するエージェント開発ガードレール | TASK-018 |
| TASK-020 | ✅ | 実装するテキストPDFパーサ | TASK-019 |
| TASK-021 | ✅ | 実装するPDF見出し推論 | TASK-020 |
| TASK-022 | ✅ | 追加するPDF用unit fixture | TASK-021 |
| TASK-023 | ✅ | 実装するOCRエンジン境界 | TASK-022 |
| TASK-024 | ✅ | 接続するNDLOCR-Lite subprocess | TASK-023 |
| TASK-025 | ✅ | 実装するocr_cer評価 | TASK-024 |
| TASK-026 | ✅ | 実装する画像抽出とmedia出力 | TASK-025 |
| TASK-027 | ✅ | 実装するキャプション保持 | TASK-026 |
| TASK-028 | ✅ | 実装する変換report JSON | TASK-027 |
| TASK-029 | ✅ | 実装するLLMバックエンド抽象化 | TASK-028 |
| TASK-030 | ✅ | 実装するLLM送信確認と同意設定 | TASK-029 |
| TASK-031 | ✅ | 実装するLLM再構造化filter | TASK-030 |
| TASK-032 | ✅ | 実装するLLM翻訳filter | TASK-031 |
| TASK-033 | ✅ | 実装するLLM差分保存 | TASK-032 |
| TASK-034 | ✅ | 実装するtranslation_structure_preserve評価 | TASK-033 |
| TASK-035 | ✅ | 実装するPPTX入力パーサ | TASK-034 |
| TASK-036 | ✅ | 実装するXLSX入力パーサ | TASK-034 |
| TASK-037 | ✅ | 実装するHTMLテーブルフォールバック | TASK-035,TASK-036 |
| TASK-038 | ✅ | 実装するWMF/EMFラスタライズ | TASK-037 |
| TASK-039 | ✅ | 実装する設定ファイルbonjil.toml | TASK-038 |
| TASK-040 | ✅ | 実装するライブラリAPI | TASK-039 |
| TASK-041 | ✅ | 整備する性能ベンチマーク | TASK-040 |
| TASK-042 | ✅ | 整備するクロスプラットフォーム配布 | TASK-041 |

## タスク詳細（補足が必要な場合のみ）

### TASK-001

- 補足: `bonjil input.docx -o out.md` のCLI形だけを先に成立させる。
- 注意: この段階ではDOCX以外を受け付けなくてよい。

### TASK-002

- 補足: Heading / Paragraph / List / Text を最小ASTとする。
- 注意: 後続でTable / Image / Footnoteを追加できるenum設計にする。

### TASK-003

- 補足: DOCXの段落スタイル、段落、リスト、テキスト抽出を対象にする。
- 注意: 画像、脚注、表は後続タスクで扱う。

### TASK-004

- 補足: CommonMarkのみを対象に、見出し、段落、リストを出力する。
- 注意: markdownlint準拠はTASK-014で扱う。

### TASK-005

- 補足: Format Detector、Parser、AST、Writerを直列接続する。
- 注意: 入出力エラーはCLIで人間が読めるメッセージにする。

### TASK-006

- 補足: 見出し、段落、リスト、日本語混在を最小fixtureにする。
- 注意: expected.md は手動レビュー前提で作成する。

### TASK-007

- 補足: `cargo test` で入力からMarkdownまでのsnapshot比較を回す。
- 注意: expected.mdの自動更新を通常フローに入れない。

### TASK-008

- 補足: 期待ASTと出力ASTの構造一致率を0.0から1.0で出す。
- 注意: 文字列一致ではなく構造単位の評価にする。

### TASK-009

- 補足: markdownlint-cli2相当の違反数を評価値として扱う。
- 注意: 外部バイナリ依存が必要な場合はCI導入時に固定する。

### TASK-010

- 補足: fixture別のpass/fail、score、diff_pathをJSONに出す。
- 注意: エージェントが機械的に読める安定スキーマにする。

### TASK-011

- 補足: `just test`、`just eval`、`just review`を定義する。
- 注意: 開発者とエージェントの入口を同じコマンドに寄せる。

### TASK-012

- 補足: HTML5、見出し、段落、リスト、table、pre/codeを扱う。
- 注意: script/styleは出力対象から除外する。

### TASK-013

- 補足: 標準HTML、ブラウザ保存HTML、table、pre/codeを追加する。
- 注意: DOCX fixtureと評価観点が重複しすぎないようにする。

### TASK-014

- 補足: CommonMark、GFM、markdownlint準拠を選択可能にする。
- 注意: flavor差分はWriter設定に閉じ込める。

### TASK-015

- 補足: 入力構造とMarkdown出力から見出し復元率を測る。
- 注意: PDF推論にも使える評価関数として実装する。

### TASK-016

- 補足: rowspan、colspan、セル内改行、画像入りセルを評価対象にする。
- 注意: Markdownで表現困難な場合はフォールバック判定を許容する。

### TASK-017

- 補足: unit、integration、regressionの3層構造を作る。
- 注意: 機密文書やライセンス不明ファイルを含めない。

### TASK-018

- 補足: GitHub Actionsでテスト、評価、ベースライン比較を行う。
- 注意: スコア低下をfailにするしきい値は設定ファイルに分離する。

### TASK-019

- 補足: CLAUDE.md相当の開発ルール、expected更新禁止、評価改竄禁止を明記する。
- 注意: repositoryの既存AGENTS規約と衝突しない内容にする。

### TASK-020

- 補足: pdfium-render等を使い、テキストPDFの文字と座標を抽出する。
- 注意: スキャンPDFはOCR境界ができるまで対象外にする。

### TASK-021

- 補足: フォントサイズ、太字、位置情報からH1-H6を推論する。
- 注意: 不確実な推論は警告としてreportに残す。

### TASK-022

- 補足: 1段組、2段組、見出し付き、目次付きPDFを追加する。
- 注意: スキャンPDF fixtureはTASK-024以降で使う。

### TASK-023

- 補足: OCRエンジンをプロセス境界で差し替えられるAPIにする。
- 注意: 本体にOCR実装を埋め込まない。

### TASK-024

- 補足: NDLOCR-Liteを一級エンジンとして呼び出す。
- 注意: OCR利用の有無とエンジン名をログとreportに残す。

### TASK-025

- 補足: ground truthとOCR結果のCharacter Error Rateを測る。
- 注意: 言語別・縦書き別の集計を可能にする。

### TASK-026

- 補足: `--extract-media` とBase64埋め込みの選択を実装する。
- 注意: デフォルトは外部ファイル抽出にする。

### TASK-027

- 補足: Wordの図表番号やPDFキャプションをMarkdown titleに保持する。
- 注意: キャプション推論に失敗した場合はwarningを出す。

### TASK-028

- 補足: メタデータ、警告、処理時間、利用機能をJSONに出す。
- 注意: 評価JSONとは用途を分け、変換実行の観測性に寄せる。

### TASK-029

- 補足: Anthropic、OpenAI、Ollama、社内OpenAI互換を抽象化する。
- 注意: デフォルトでは外部APIを呼ばない。

### TASK-030

- 補足: 送信先、送信内容、バッチ同意設定を明示する。
- 注意: 確認なしの外部送信を許可しない。

### TASK-031

- 補足: AST filterとして再構造化を適用する。
- 注意: LLM結果をそのまま信用せず、AST validationを通す。

### TASK-032

- 補足: Markdown構造を維持した翻訳をAST filterとして実装する。
- 注意: 原文への対応関係をfront matterに記録する。

### TASK-033

- 補足: LLM適用前後のMarkdownとAST差分を保存する。
- 注意: 機密データをCI artifactに出さない設定を用意する。

### TASK-034

- 補足: 翻訳前後の見出し、リスト、表構造の保持率を測る。
- 注意: 翻訳品質そのものではなく構造保持を評価する。

### TASK-035

- 補足: スライド単位、テキストボックス、画像、表をASTへ変換する。
- 注意: 視覚順序の推定失敗はwarningにする。

### TASK-036

- 補足: シート、セル、結合セル、表範囲をASTへ変換する。
- 注意: 数式と表示値の扱いをreportに明示する。

### TASK-037

- 補足: Markdown表で壊れるケースをHTML tableへ切り替える。
- 注意: flavorごとの許容HTMLを考慮する。

### TASK-038

- 補足: WMF/EMFをLibreOffice非依存でPNG化する。
- 注意: 実装方式のライセンス確認を完了してから着手する。

### TASK-039

- 補足: flavor、OCR、LLM同意、CA証明書、strictを設定可能にする。
- 注意: CLIオプションとの優先順位を明文化する。

### TASK-040

- 補足: CLIと同等の変換をRust APIから呼び出せるようにする。
- 注意: ストリーミングAPIの境界を先に固定する。

### TASK-041

- 補足: 100ページDOCXとテキストPDFの処理時間を継続測定する。
- 注意: 性能劣化はCIで検出できる形にする。

### TASK-042

- 補足: Linux、macOS、Windows向けのビルドと配布を整備する。
- 注意: 単一バイナリ配布と企業プロキシ環境を検証対象にする。

## Backlog一覧

| ID | Status | Summary | DependsOn |
| ---- | ---- | ---- | ---- |
| BACKLOG-001 | ⏳ | 実装するOpenDocument入力パーサ | TASK-042 |
| BACKLOG-002 | ⏳ | 実装するMDXライター | TASK-042 |
| BACKLOG-003 | ⏳ | 実装するHTMLライター | TASK-042 |
| BACKLOG-004 | ⏳ | 実装するHedgeDocスライドライター | TASK-042 |
| BACKLOG-005 | ⏳ | 実装するWASMプラグインSDK | TASK-042 |
| BACKLOG-006 | ⏳ | 接続するNDL古典籍OCR-Lite | TASK-042 |
| BACKLOG-007 | ⏳ | 接続するSurya OCR | TASK-042 |
| BACKLOG-008 | ⏳ | 整備する大規模corpora運用 | TASK-042 |

## Backlog詳細（補足が必要な場合のみ）

### BACKLOG-001

- 補足: ODT / ODS / ODP は要件上次フェーズ扱いのため後回しにする。

### BACKLOG-002

- 補足: MDXは主目的ではないためMarkdown Writer安定後に追加する。

### BACKLOG-003

- 補足: HTML出力はAST共通化の効果を確認してから実装する。

### BACKLOG-004

- 補足: HedgeDoc連携は既存ワークフロー需要が確定してから実装する。

### BACKLOG-005

- 補足: WASMプラグインはコア変換品質が安定してから公開する。

### BACKLOG-006

- 補足: 古典籍OCRは通常OCRの境界と評価が安定してから追加する。

### BACKLOG-007

- 補足: SuryaはライセンスとGPU前提の運用判断が必要。

### BACKLOG-008

- 補足: 大規模corporaはGit管理外の保存先と再現手順を別途決める。
