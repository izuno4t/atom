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

## 参照した公開資料

TASK-044以降は、要件文の表現をそのまま作業名へ写すのではなく、
公開仕様、公式ツール文書、公開ベンチマークの観点から分解する。

| 種別 | 参照先 | タスク化への使い方 |
| ---- | ---- | ---- |
| 公式仕様 | [ECMA-376 Office Open XML](https://ecma-international.org/publications-and-standards/standards/ecma-376/) | DOCX/PPTX/XLSXをZIP内XML断片ではなく、語彙、部品、関係を持つOOXMLパッケージとして扱う根拠にする。 |
| 公式文書 | [Microsoft Learn: Structure of a PresentationML document](https://learn.microsoft.com/en-us/office/open-xml/presentation/structure-of-a-presentationml-document) | PPTXのslide、shape、picture、table、notes、master/layoutを抽出対象へ含める根拠にする。 |
| 公式文書 | [Microsoft Learn: Shared String Table](https://learn.microsoft.com/en-us/office/open-xml/spreadsheet/working-with-the-shared-string-table) | XLSXの表示テキスト復元でsharedStringsを必須部品として扱う根拠にする。 |
| 公式文書 | [Microsoft Learn: Merge cells](https://learn.microsoft.com/en-us/office/open-xml/spreadsheet/how-to-merge-two-adjacent-cells-in-a-spreadsheet) | XLSXの結合セル、空セル、複数ヘッダーを表構造復元の対象にする根拠にする。 |
| 準公式/仕様解説 | [PDFlib: Tagged PDF Basics](https://www.pdflib.com/pdf-knowledge-base/pdfua/tagged-pdf-basics/) | PDFでは描画順と論理読順が一致しない前提を置き、tag treeと座標レイアウトの両方を評価する根拠にする。 |
| 公式文書 | [Pandoc User's Guide](https://pandoc.org/MANUAL.html) | 既存ツールのAST変換が構造保持を重視しつつ、複雑表などで損失があり得る比較軸にする。 |
| 公式文書 | [Docling Supported formats](https://docling-project.github.io/docling/usage/supported_formats/) | 比較対象がPDF/DOCX/PPTX/XLSXを統一表現へ変換できるため、同じ入力で横比較する根拠にする。 |
| 公式リポジトリ | [Microsoft MarkItDown README](https://github.com/microsoft/markitdown/blob/main/README.md) | Markdown変換ツールとしてPDF/Office系の比較対象に含める根拠にする。 |
| 公式リポジトリ | [PyMuPDF4LLM README](https://github.com/pymupdf/pymupdf4llm) | PDFの画像、ベクター、ページchunk、OCRを比較観点に入れる根拠にする。 |
| 公式リポジトリ | [Mammoth.js README](https://github.com/mwilliamson/mammoth.js/) | DOCXの意味構造とstyle map依存、Markdown出力の制約を比較観点に入れる根拠にする。 |
| 公開ベンチ | [DocLayNet](https://github.com/DS4SD/DocLayNet) | レイアウト分類、人手アノテーション、多様な文書カテゴリを評価ルーブリック設計の根拠にする。 |

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
| TASK-039 | ✅ | 実装する設定ファイルatom.config.toml | TASK-038 |
| TASK-040 | ✅ | 実装するライブラリAPI | TASK-039 |
| TASK-041 | ✅ | 整備する性能ベンチマーク | TASK-040 |
| TASK-042 | ✅ | 整備するクロスプラットフォーム配布 | TASK-041 |
| TASK-043 | ✅ | 分離する評価関連ツールのディレクトリ | TASK-042 |
| TASK-044 | ✅ | 調査する公開仕様とベンチマーク根拠 | TASK-043 |
| TASK-045 | ✅ | 定義する現実文書パターン分類とfixture設計 | TASK-044 |
| TASK-046 | ✅ | 実装するOOXMLパッケージ部品解決 | TASK-045 |
| TASK-047 | ✅ | 実装するPresentationML視覚順序と図形構造復元 | TASK-046 |
| TASK-048 | ✅ | 実装するSpreadsheetML表構造復元 | TASK-046 |
| TASK-049 | ✅ | 実装するPDF論理構造とレイアウト読順復元 | TASK-045 |
| TASK-050 | ✅ | 実装する図表キャプションとメディア対応付け | TASK-047,TASK-049 |
| TASK-051 | ✅ | 整備する公開ベンチ準拠の評価ルーブリック | TASK-045,TASK-050 |
| TASK-052 | ✅ | 比較する実コーパス評価レポート | TASK-051 |
| TASK-053 | ✅ | 作成する優位性判定用goldenレビュー表 | TASK-052 |
| TASK-054 | ✅ | 実装するPDFテキスト抽出バックエンド切替 | TASK-053 |
| TASK-055 | ✅ | 実装するPDF見出しリスト読順復元 | TASK-054 |
| TASK-056 | ✅ | 実装するPPTX本文構造とlist復元 | TASK-053 |
| TASK-057 | ✅ | 実装するXLSX表範囲と複数sheet出力 | TASK-053 |
| TASK-058 | ✅ | 実装するDOCX style mapと注釈復元 | TASK-053 |
| TASK-059 | ✅ | 実装するmedia caption候補report | TASK-056,TASK-058 |
| TASK-060 | ✅ | 実装する評価timeoutと対象外判定 | TASK-052 |
| TASK-061 | ✅ | 判定する公式コーパス優位性レポート | TASK-055,TASK-057,TASK-059,TASK-060 |
| TASK-062 | ✅ | 整備するatomローカルインストールMakeターゲット | TASK-042 |
| TASK-063 | ✅ | 整備するLLMなし大量評価コマンド | TASK-061,TASK-062 |
| TASK-064 | ✅ | 集計する形式別成功率と失敗理由 | TASK-063 |
| TASK-065 | ✅ | 集計するMarkdown構造量スコア | TASK-064 |
| TASK-066 | ⏳ | 抽出する人手レビュー候補 | TASK-065 |
| TASK-067 | ⏳ | 作成するLLMなし評価サマリ | TASK-066 |
| TASK-068 | ⏳ | 整備するOllamaローカルLLM構築手順 | TASK-067 |
| TASK-069 | ⏳ | 定義するローカルLLM評価JSONスキーマ | TASK-068 |
| TASK-070 | ⏳ | 接続するOllamaローカルLLM境界 | TASK-069 |
| TASK-071 | ⏳ | 実装する単体Markdown採点プロンプト | TASK-070 |
| TASK-072 | ⏳ | 実装するMarkdownペア比較プロンプト | TASK-070 |
| TASK-073 | ⏳ | 作成するatom-llm-eval評価バイナリ | TASK-071,TASK-072 |
| TASK-074 | ⏳ | 統合する自動評価からLLM評価への流れ | TASK-073 |
| TASK-075 | ⏳ | 抽出する改善候補とfixture化候補 | TASK-074 |

## タスク詳細（補足が必要な場合のみ）

### 実装判断記録

- 横断責務は `src/lib.rs` に置かず、`core/`、`pipeline/`、`parsers/`、
  `writers/`、`evaluation/`、`integrations/` の機能単位moduleへ分離する。
- `format` という曖昧な横断moduleは作らない。入力形式判定は
  `input_detection`、出力生成は `writers`、形式別特性は各parserへ置く。
- Office系は製品名のbucketではなく、OOXML共通処理を `ooxml` に置き、
  PPTX/XLSXなどの文書種別実装を `ooxml::pptx` / `ooxml::xlsx` に分ける。
- PDFのページ単位OCR fallbackは、Rust内のPDFレンダリングでページ画像を生成し、
  OCR境界へ渡す。`ocr-rs` はモデルファイルを同梱せず、明示的な環境変数で
  detection/recognition/charsetの各モデルを指定する。

### TASK-001

- 補足: `atom input.docx -o out.md` のCLI形だけを先に成立させる。
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
- 追記: 親子関係の崩れを検出するため、フラットな構造ラベル列ではなく
  AST由来の順序付き木編集距離で評価する。

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
- 追記: 評価やfixture化の前段階で手元入力を置くGit管理外ディレクトリとして
  `tests/inputs/` を設ける。ここに置いたファイルはexpectedや評価対象ではなく、
  変換結果の手動確認用とする。

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

### TASK-043

- 補足: 評価、ベンチ、ベースライン比較、実コーパス比較の実行ファイルを専用ディレクトリへ移動する。
- 注意: Cargoのバイナリ名と既存のMakeターゲットは維持する。

### TASK-044

- 補足: OOXML、PDF、既存Markdown変換ツール、公開レイアウトベンチの資料を確認し、評価対象にする文書構造を根拠付きで整理する。
- 対象: ECMA-376、Microsoft Open XML、Tagged PDF、Pandoc、Docling、MarkItDown、PyMuPDF4LLM、Mammoth.js、DocLayNet。
- 成果: `evaluation/methods/` か `docs/tasks.md` に、参照元、採用理由、未採用理由を記録する。
- 注意: 要件文に書かれた問題例をそのままタスク化せず、公開資料から確認できる構造単位へ分解する。

### TASK-045

- 補足: 公開データを変換・評価して失敗パターンを特定し、その問題と同じ文書形式の最小再現fixtureへ落とす。
- 対象: 複数カラム、図表、キャプション、リスト、脚注、表、結合セル、text box、shape、notes、ヘッダー/フッター。
- 成果: `evaluation/methods/fixture-design.md` に公開データ収集、変換、失敗分類、同形式fixture化の手順を記録する。
- 注意: 公開データそのものを安直に同梱せず、出典とライセンスを記録したうえで失敗原因を最小化する。

### TASK-046

- 補足: DOCX / PPTX / XLSXをOOXMLパッケージとして読み、Content Types、relationships、部品パス、media参照を解決する。
- 対象: document、slides、slide layouts、slide masters、worksheets、shared strings、drawings、media、notes。
- 成果: XML断片単体ではなく、部品間関係を保持した入力モデルを追加する。
- 注意: `word/document.xml` や `ppt/slides/slide1.xml` だけを読んで完了扱いにしない。

### TASK-047

- 補足: PresentationMLのshape tree、placeholder、picture、table、group shape、notesをもとに、スライドの読順と構造を復元する。
- 対象: XML順と視覚順の不一致、複数カラム、タイトル/本文/注記/フッター、図形で作られた疑似表。
- 成果: 座標、サイズ、placeholder種別、z-order、グループ構造を使う読順推定とwarning出力を実装する。
- 注意: 確信度が低い疑似表はMarkdown tableにせず、候補と理由をreportへ残して段落へフォールバックする。

### TASK-048

- 補足: SpreadsheetMLのworksheet、sheetData、mergeCells、sharedStrings、inline strings、formula/valueを使い、Markdownで壊れにくい表構造へ復元する。
- 対象: 結合セル、空セル、複数ヘッダー行、表タイトル、注記行、数式結果、表示値、複数シート。
- 成果: Markdown table、HTML table、または警告付きfallbackの選択理由をreportに出す。
- 注意: 数式そのものを出したのか、キャッシュされた表示値を出したのかを必ず記録する。

### TASK-049

- 補足: Tagged PDFの論理構造を優先し、存在しない場合は座標、フォント、行間、列、罫線、画像領域から読順と構造を推定する。
- 対象: 段落、見出し、リスト、脚注、複数カラム、罫線あり/なし表、ヘッダー/フッター、ページ番号。
- 成果: tag tree利用有無、座標推定の信頼度、除外したartifact、fallback理由をreportに出す。
- 注意: 抽出失敗時に無警告のプレーンテキストへ潰さず、失われた構造を明示する。
- 進捗: 明らかなバイナリ断片のスキップ、Tagged PDF検知、fallback理由の
  warning、font size/座標/章番号/list markerによる構造推定、抽出backend名、
  抽出失敗、OCR要否のreport出力を実装した。公式冊子PDFの完全なtag tree
  復元は今後の品質改善対象だが、本タスクの完了範囲は警告付きfallbackと
  構造推定までとする。

### TASK-050

- 補足: Officeのmedia relationships、DrawingML、Word/PDFの近傍テキスト、DocLayNetのFigure/Table/Caption相当分類を使い、図表とキャプションを対応付ける。
- 対象: Officeの図表番号、PDFの近傍キャプション、画像入りセル、スライド上の画像説明、複数候補。
- 成果: Markdown本文、media出力、report JSONで同じmedia idを参照できるようにする。
- 注意: 対応が曖昧な場合は単一決定せず、候補、距離、ページ/スライド、確信度をreportに残す。
- 進捗: DOCX caption付き画像、HTML/PPTX画像参照、画像入りセルをMarkdownと
  reportのmedia/media_candidatesで同じmedia idへ対応付ける。曖昧な候補は
  confidenceを下げ、単一captionとして断定しない。

### TASK-051

- 補足: DocLayNetの人手アノテーションとレイアウト分類を参考に、自動スコアと人手確認を分けた評価ルーブリックを整備する。
- 対象: title、text、list、table、figure、caption、footnote、formula、page header/footer、読順、警告品質。
- 成果: `evaluation/methods/evaluation.md` に採点項目、重大度、合格基準、レビュー手順を記録する。
- 注意: 既存ツールとの差は、自動スコア、目視レビュー、失敗時reportの説明可能性を合わせて判断する。

### TASK-052

- 補足: `evaluation/inputs/` の実コーパスを使い、atomと比較ツールを同一入力、同一出力配置、同一ルーブリックで横並び評価する。
- 対象: Docling、MarkItDown、PyMuPDF4LLM、Mammoth.js、Pandoc。形式ごとに利用可能なPython/Rust/TypeScriptライブラリも比較候補へ含める。
- 成果: `evaluation/outputs/` にMarkdown、tool report、差分、
  目視レビュー用indexを出し、`evaluation/reports/` に集計を残す。
- 注意: `evaluation/inputs`、`evaluation/outputs`、`evaluation/reports` の実データはGit管理外にし、CIでは実コーパス評価を実行しない。
- 進捗: review-index自動生成を追加した。外部Dockerツールを含む
  大学公開文書6件の横並び比較を実行した。公式代表セットの一括比較は
  長時間化したため、timeout制御をTASK-060へ分離する。

### TASK-053

- 補足: 自動スコアだけでは優位性を判定しない。実コーパスごとに、期待する
  見出し、本文、表、画像、caption、warningを人手レビュー表に固定する。
- 対象: `japanese-university-tools` と `japanese-official-expanded` の代表例。
- 成果: `evaluation/methods/golden-review.md` にgoldenレビュー表と判定基準を出す。
- 注意: 既存ツールの出力を正解にせず、入力文書から期待構造を決める。

### TASK-054

- 補足: 現状PDFは簡易parserで、PyMuPDF4LLMに大きく劣る。テキスト抽出を
  外部backendまたは内部抽出層へ切り替え可能にする。
- 対象: 日本語PDF、スライド由来PDF、冊子PDF、画像PDF。
- 成果: backend名、抽出失敗、OCR要否をreportに出す。
- 注意: PDF入力で無意味な短い断片を本文として採用しない。
- 進捗: 内部PDF抽出層にhex string、UTF-16BE、TJ配列の復元テストと
  実装を追加した。PDFテキスト抽出backend境界、基本座標保持、
  backend名、抽出失敗、OCR要否のreport出力を追加した。通常PDFの
  テキスト抽出に組み込みRust backend `pdf-extract` を追加し、
  失敗時のみ内部抽出層へフォールバックする。

### TASK-055

- 補足: PyMuPDF4LLMが保持した見出し/listをatomでも復元する。
- 対象: font size、座標、行間、箇条書き記号、ページ内読順。
- 成果: PDF出力でheading/list/paragraphを区別し、理由をwarningへ残す。
- 注意: スコア稼ぎのために過剰に見出し化しない。
- 進捗: font sizeによる見出し推論に加え、font metadataがないPDF抽出結果
  でも章番号、セクション番号、連続箇条書きをheading/listへ復元する。
  単発の番号付き本文を過剰にlist化しない回帰テストを追加した。

### TASK-056

- 補足: PPTXでPandocが上位になった差分を、本文構造、箇条書き、notes、
  shape groupingの復元として埋める。
- 対象: title/body/footer、list階層、グループ図形、notes slide。
- 成果: PPTX fixtureと実コーパスでPandoc同等以上の構造を出す。
- 注意: 画像だけを増やして本文構造を落とさない。
- 進捗: `a:buChar` / `a:buAutoNum` を持つ本文段落をMarkdown listへ復元する
  回帰テストを追加し、targeted testで通過した。完了判定は `make ci`
  通過後に行う。

### TASK-057

- 補足: XLSXでMarkItDown/Pandocが上位になった差分を、表範囲検出と
  sheet単位出力の改善として埋める。
- 対象: used range、空行空列、複数sheet、chart元データ、表タイトル。
- 成果: sheet見出し、表範囲、表本文をMarkdown tableへ出す。
- 注意: 帳票型の位置情報を単純な一次元テキストへ潰さない。
- 進捗: XLSX sheet parserで空の外周行/列をtrimし、package変換で複数
  sheetに `Sheet N` 見出しを付与する。integration testを追加して通過した。
  完了判定は `make ci` 通過後に行う。

### TASK-058

- 補足: DOCXで既存ツールと同等止まりのため、style map、脚注、URL、
  注釈を構造として復元する。
- 対象: heading style、caption style、footnote、hyperlink、comment。
- 成果: DOCXでMammoth/Pandoc同等以上の意味構造を出す。
- 注意: Word固有style名に過剰依存せず、未対応styleはreportへ出す。
- 進捗: `document.xml` に加えて relationships、footnotes、comments を渡せる
  parser APIを追加し、hyperlink URL、脚注、コメントを構造として復元する。
  integration testを追加して通過した。完了判定は `make ci` 通過後に行う。

### TASK-059

- 補足: 現状は画像参照は出るが、caption候補と距離、ページ/スライド、
  media idの対応をreportで比較できない。
- 対象: PPTX picture、DOCX drawing、PDF figure近傍テキスト。
- 成果: `media_candidates` 相当のreport項目とfixtureを追加する。
- 注意: 曖昧な候補を単一captionとして断定しない。
- 進捗: `ConversionReport` に `media_candidates` を追加し、captionがある
  image/table-cell imageを候補としてJSONへ出す。曖昧な候補はconfidence 0.0
  で断定しない。integration testを追加した。

### TASK-060

- 補足: 公式代表セット比較で外部ツールが長時間戻らなかったため、
  ツール別timeout、形式別対象外、ファイルサイズ制限を評価runnerに入れる。
- 対象: `atom-corpus-eval` のDocker実行境界、report status、error分類。
- 成果: timeout、unsupported、too_largeをJSONとreview-indexへ出す。
- 注意: timeoutを成功扱いにせず、優位性判定から除外する。
- 進捗: `--max-bytes` と `--timeout-ms` を追加し、`too_large` と `timeout`
  statusをJSONへ出す。対象外はwinnerなしの `excluded: too_large` として
  判定から除外する。corpus eval testを追加した。

### TASK-061

- 補足: 改善後に、公式/大学/補助コーパスを同一コマンドで再評価し、
  atomが優れる事例と劣る事例を分けて報告する。
- 対象: PDF、PPTX、XLSX、DOCXの公開コーパス。
- 成果: `evaluation/reports/` に人手レビュー込みの優位性レポートを出す。
- 注意: `superiority_claim` は根拠が揃うまでprovenにしない。
- 進捗: 再配布可能なrepository unit fixturesで `atom-corpus-eval` を実行し、
  `evaluation/reports/official-corpus-superiority.json` とレビュー用Markdownを
  出力した。外部baseline toolと公開コーパスの同時比較は未実行のため、
  `superiority_claim` は `not_proven_without_human_review_or_ground_truth` のまま
  明示した。

### TASK-062

- 補足: release版の`atom`実行ファイルを作成し、`atom`コマンドとして
  ローカルの実行パスへ
  配置できるMakeターゲットを用意する。
- 対象: `Makefile` の開発者向けターゲット。
- 成果: `make install` で `target/release/atom` を作成し、既定では
  `~/bin/atom` へインストールできる。
- 注意: 配置先は `INSTALL_DIR` で上書き可能にし、既存の変換処理には影響
  させない。

### TASK-063

- 補足: LLMを使う前に、既存の `atom-corpus-eval` を大量文書評価の入口として
  使えるように実行条件を固定する。
- 対象: `evaluation/inputs/`、`evaluation/outputs/`、`evaluation/reports/`、
  `--limit`、`--per-ext`、`--ext`、`--tools`、`--timeout-ms`、`--max-bytes`。
- 成果: 形式横断と形式限定の標準コマンドを `evaluation/methods/evaluation.md`
  に記録し、LLMなし評価を再現可能にする。
- 注意: 実コーパスと出力はGit管理外にし、CIでは大量文書評価を実行しない。
- 進捗: 本体設定と分離した `evaluation/atom-evaluation.config.toml` を入力、
  出力、report pathのローカル設定として使い、LLMなし標準条件、HTML重点、
  PDF重点、Office重点の実行コマンドを `evaluation/methods/evaluation.md` に
  記録した。

### TASK-064

- 補足: corpus評価JSONから、形式別の変換成功率と失敗理由を集計する。
- 対象: ok、error、timeout、unsupported、too_large、missing、入力形式、tool名。
- 成果: `evaluation/reports/auto-summary.json` に形式別、tool別、status別の
  集計を出す。
- 注意: timeoutや対象外を成功扱いにせず、優位性判定から除外する。
- 進捗: `atom-corpus-eval` のsummaryへ、形式別、tool別、status別の集計と
  失敗理由の集計を追加した。`make ci` で確認済み。

### TASK-065

- 補足: 出力Markdownをプログラムで解析し、LLMなしで構造量を比較できるように
  する。
- 対象: bytes、heading、paragraph、list item、table、image、code block、
  warning数、report feature。
- 成果: `evaluation/reports/auto-summary.json` に構造量スコアと極端値を出す。
- 注意: 構造量が多いことを品質の証明にせず、レビュー候補抽出の信号として扱う。
- 進捗: `atom-corpus-eval` のsummaryへ、tool別の平均構造量と極端に短い出力を
  集計する。`make ci` で確認済み。

### TASK-066

- 補足: LLMなし評価の結果から、人が見るべき候補を自動抽出する。
- 対象: atomだけ成功、atomだけ失敗、atom出力が極端に短い、warningが多い、
  比較ツールと構造量差が大きい、形式ごとの代表例。
- 成果: `evaluation/reports/auto-review-candidates.md` に候補一覧と理由を出す。
- 注意: 候補抽出は優位性判定ではなく、人手またはLLM評価へ渡す入口にする。

### TASK-067

- 補足: LLMなし評価だけで分かる結果を、人間が読めるsummaryとしてまとめる。
- 対象: 形式別成功率、tool別成功率、失敗理由上位、構造量傾向、レビュー候補。
- 成果: `evaluation/reports/auto-summary.md` を生成する。
- 注意: `superiority_claim` は人手確認またはground truthなしにprovenへ変えない。

### TASK-068

- 補足: LLMなし評価で絞った候補だけを後段評価できるように、Ollamaの導入、
  起動、モデル取得、疎通確認の手順を整備する。
- 対象: macOS、Linux、Dev Container外のローカル実行、モデル容量、
  `ollama serve`、`ollama pull`、`ollama run`、HTTP疎通確認。
- 成果: `evaluation/methods/llm-evaluation.md` にOllama構築手順と
  推奨モデル例を記録する。
- 注意: 具体的なモデル名は例として扱い、環境のメモリやGPU/CPU条件に応じて
  差し替え可能にする。外部APIキーを前提にしない。

### TASK-069

- 補足: ローカルLLM評価の出力JSONを固定し、単体採点とペア比較の両方を
  同じreportで扱えるようにする。
- 対象: input path、candidate tool、overall score、観点別score、
  missing content、major errors、winner、reason、model、prompt version。
- 成果: `evaluation/methods/llm-evaluation.md` にスキーマと採点観点を定義する。
- 注意: LLM判定は最終真実にせず、`superiority_claim` の根拠には
  人手確認またはground truthが必要であることを明記する。

### TASK-070

- 補足: 外部LLMへ送信せず、OllamaのローカルHTTP APIを評価backendとして
  接続する。
- 対象: `--model ollama:<name>`、`--ollama-url`、timeout、retryなしの失敗処理。
- 成果: ローカルLLMへpromptを送り、JSON応答を受け取る境界を実装する。
- 注意: Ollama未起動、モデル未取得、JSON不正応答は評価失敗としてreportへ
  残し、変換成功扱いにしない。

### TASK-071

- 補足: 1つのMarkdown候補を元文書材料に照らして採点するpromptを実装する。
- 対象: reading order、heading structure、paragraph integrity、
  table preservation、image caption traceability、noise、Markdown usability。
- 成果: `mode=score` でcandidateごとの観点別scoreと理由をJSONへ出す。
- 注意: 採点理由は短くし、入力本文の長い引用をreportへ保存しない。

### TASK-072

- 補足: atom出力と比較ツール出力を同じ元文書材料に照らして比較するpromptを
  実装する。
- 対象: atom vs pandoc、atom vs markitdown、atom vs docling、
  atom vs pymupdf4llm、atom vs mammoth-js。
- 成果: `mode=pairwise` でwinner、観点別差分、major errorsをJSONへ出す。
- 注意: 同点または判断不能を許容し、必ず勝者を作らせない。

### TASK-073

- 補足: LLMなし評価で抽出した候補だけを対象に、`atom-corpus-eval` の後段で
  動く `atom-llm-eval` を追加する。
- 対象: Cargo bin、CLI引数、候補一覧読み込み、材料生成、LLM呼び出し、
  report出力。
- 成果: 次の形で実行できる。

  ```bash
  cargo run -p atom-evaluation --bin atom-llm-eval -- \
    --root evaluation/inputs \
    --outputs evaluation/outputs \
    --candidates evaluation/reports/auto-review-candidates.md \
    --out evaluation/reports/llm-review.json \
    --model ollama:qwen2.5vl \
    --mode pairwise
  ```

- 注意: 初期実装はテキスト材料中心とし、PDFページ画像やVLM入力は後続へ分ける。

### TASK-074

- 補足: LLMなし評価の候補抽出から、必要なケースだけローカルLLM評価へ渡す
  2段階フローを整備する。
- 対象: `make corpus-eval`、`make auto-eval-summary`、必要に応じた
  `make llm-eval`、出力先規約、評価手順書。
- 成果: LLMなし評価、候補抽出、ローカルLLM評価、集計の流れを再現可能にする。
- 注意: CIではローカルLLM評価を実行しない。モデル容量と実データは環境依存とする。

### TASK-075

- 補足: LLMなし評価またはLLM評価で低スコア、atom敗北、重大欠落になったケースを、
  実装改善候補とfixture化候補へ分ける。
- 対象: 形式、観点、失敗理由、最小再現可能性、公開データ可否。
- 成果: `evaluation/public-corpus-observations.md` または専用reportに、
  改善候補とfixture化候補を記録する。
- 注意: 自動評価やLLMの指摘だけでexpected Markdownを更新せず、元文書を
  確認してからfixture化する。

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
