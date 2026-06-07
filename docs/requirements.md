# ドキュメント-Markdown変換ツール 要件書

## 概要

各種ドキュメント (HTML / PDF / Office系) を Markdown に変換するツールは
多数存在するが、いずれも次のような問題を抱えている。

- 実行環境がツールごとに異なる (Python / Node.js / Go / Rust / バイナリ)
- フォーマット忠実度に偏りがある (PDFは得意だがDOCXが弱い、など)
- Microsoft 謹製の MarkItDown でも PDF が事実上プレーンテキスト化する
- 複雑なテーブル・画像入りセル・キャプション付き画像で破綻する
- LLMでの後処理を前提としたツールが多く、ローカル完結が難しい

本要件書は、既存ツールの問題点を整理した上で、これらを解決する
新しい変換ツールが満たすべき要件を定義する。

### 課題とモチベーション

変換元の文書が最初から適切に構造化されているなら、本ツールを作る必要は薄い。
その場合は Pandoc、Mammoth、Docling、MarkItDown など既存ツールを使えばよい。

本ツールが解決する対象は、現実によくある次のような文書である。

- 見た目は表だが、内部構造はテキストボックスや絶対配置の集合になっている
- 見出しがスタイルではなく、フォントサイズや太字だけで表現されている
- PDF化によって段落、リスト、表、脚注、キャプションの意味が失われている
- PowerPoint の複数テキストボックスの読み順が視覚順とXML順で一致しない
- Excel の結合セル、空セル、複数ヘッダー行によりMarkdown表が壊れやすい
- 画像入りセル、図表番号、キャプションが既存ツールでは脱落する
- OCRやLLMを使えば読めるが、ローカル完結・再現可能な処理として固定しにくい

したがって、評価対象も「きれいに構造化された文書」だけでは不十分である。
評価fixtureと実コーパスは、構造が曖昧、破損、または視覚表現に依存している
文書パターンを中心に設計する。

### 設計思想

機械可読 (LLM向け) ではなく、**人が読める構造化** を第一目標とする。
MarkItDown 系が割り切った「LLMが食えればいい」路線とは明確に
ポジションを分ける。

LLM はオプションで、再構造化と翻訳の質的向上に使う。デフォルトは
ローカル完結・外部送信ゼロ。

## 既存ツール調査

### 調査対象

| ツール | 実装 | 主用途 | 配布形態 |
| --- | --- | --- | --- |
| Pandoc | Haskell | 汎用ドキュメント変換 | バイナリ |
| MarkItDown | Python | LLM向けMarkdown化 | pip |
| Mammoth | JS / Python | DOCX→HTML/MD | npm / pip |
| Marker | Python (ML) | PDF→Markdown | pip + GPU推奨 |
| Docling | Python (ML) | RAG向け構造保持変換 | pip + ML model |
| MinerU | Python (ML) | PDF高精度抽出 | pip + GPU |
| PyMuPDF4LLM | Python | 軽量PDF抽出 | pip |

### 各ツールの問題点

#### Pandoc

汎用変換のデファクトだが、Markdown出力時に既知の問題が多い。

- DOCX のキャプション付き画像が出力から消える (issue #11412)
- 画像入りテーブルを Markdown に出すとセル幅で文字列が分割され、
  画像パスが折り返されて壊れる (issue #10402, #10315)
- Markdown→DOCX の往復で画像サイズが固定 1.67 inch に丸められる
  (issue #976)
- pipe table を出したいのに simple/multiline/grid table が
  優先されるなど、出力 Markdown 方言の制御が難しい
- Haskell 製でユーザーが拡張を書く場合は Lua filter を要する

#### Microsoft MarkItDown

Microsoft AutoGen チーム製。LLM 向けに割り切った設計。

- 公式が「人間向けの高忠実度変換には最適ではない」と明言
- PDF はプレーンテキスト抽出のみで Markdown 構造化されない
- スキャンPDF (画像PDF) は OCR が走らないため変換できない
  (issue #1373)
- 複雑な PowerPoint レイアウトでは視覚的関係が失われる
- v0.0.1 → v0.1.0 で DocumentConverter のインタフェースが
  ファイルパスからストリームへ破壊的変更
- 画像キャプション生成に外部 LLM (OpenAI など) が必要

#### Mammoth (.js / Python)

DOCX → HTML 専用。Markdown 出力は付帯機能。

- 公式が「Markdown サポートは deprecated」と明記。HTML 経由で
  別ライブラリ (turndown 等) を使うことを推奨
- DOCX と HTML の構造のミスマッチが大きく、複雑な文書では完璧な
  変換は期待できないと公式が認めている
- テーブルの罫線・書式は無視される
- WMF 画像は標準では扱えず、LibreOffice 連携が必要
- テキストボックスは別段落として後ろに付加される (順序が崩れる)
- 入力サニタイズを一切しないので、信頼できない入力には危険

#### Marker / Docling / MinerU (ML系)

PDF を高精度で Markdown 化する最新世代。

- いずれも GPU 推奨。CPU でも動くが現実的な速度ではない
- 数百MB〜数GBのモデルダウンロードを要する
- Marker は商用利用に売上制限あり (年商 5M USD 超は要ライセンス)。
  モデル重みは cc-by-nc-sa-4.0
- LLM オプション (`--use_llm`) を使うと外部API課金が発生
- スキャンPDFのOCRは強いが、Officeネイティブ形式 (DOCX/PPTX) は
  得意ではないか、内部で別経路を通すのでバージョンによって挙動差
- 機密文書を社内で処理したい場合、外部APIに送れない制約と
  ローカル GPU リソースの両立が難しい

### 共通する課題

調査の結果、既存ツール群に共通する問題が見えてくる。

1. **実行環境のばらつき** — Python / Node.js / Haskell バイナリ /
   GPU 必須など、組織で標準化しづらい
2. **フォーマット別の得意不得意** — PDF が強いツールは DOCX が弱く、
   その逆もある。1つのツールで全形式を高品質に処理できない
3. **テーブル・画像の脆弱性** — 画像入りテーブル、キャプション付き
   画像、結合セルでほぼ全ツールが破綻する
4. **出力 Markdown 方言の制御不能** — CommonMark / GFM /
   markdownlint 準拠といった方言を選択・固定する手段が貧弱
5. **外部依存** — LLM API、GPU、巨大モデル、外部バイナリ
   (LibreOffice) などへの依存が前提化している
6. **機密データの扱い** — 外部 LLM への送信を前提とするツールが多く、
   企業内文書をローカル完結で処理する選択肢が少ない
7. **観測性の欠如** — どこが変換に失敗したか、忠実度の低下が
   どこで起きたかをユーザーに伝える仕組みが弱い

## 新ツールの要件

上記課題を踏まえ、新ツールが満たすべき要件を定義する。

### 機能要件

#### F1. 統一インタフェース

入力形式に依存しない単一のコマンド・API で操作できること。

```bash
atom input.docx -o output.md
atom input.pdf  -o output.md
atom input.pptx -o output.md
atom input.html -o output.md
```

#### F2. 対応フォーマット

最低限以下を一級市民として扱う。

- HTML (生HTML、ブラウザ保存形式)
- PDF (テキストPDF / スキャンPDF の両方)
- Microsoft Office (DOCX / XLSX / PPTX)
- OpenDocument (ODT / ODS / ODP) は次フェーズ

#### F3. Markdown 方言の選択

出力する Markdown 方言を明示的に指定できること。

- CommonMark
- GitHub Flavored Markdown (GFM)
- markdownlint-cli2 デフォルトルール準拠
- HedgeDoc / HackMD スライド形式

```bash
atom input.pdf --flavor gfm -o output.md
atom input.docx --flavor markdownlint -o output.md
```

#### F4. テーブル忠実度

以下のケースで破綻しないこと。

- 結合セル (rowspan / colspan)
- セル内の画像
- セル内の改行・複数段落
- ヘッダー行が複数行

セル内画像のように Markdown で表現困難な場合は、HTML テーブル
へのフォールバックを選択可能にする。

#### F5. 画像処理

- 画像はデフォルトで外部ファイルとして抽出 (`--extract-media`)
- インライン Base64 埋め込みも選択可能
- キャプション (Wordの図表番号、PDFの図キャプション) を
  Markdown の `![alt](path "title")` の title 属性に保存
- WMF / EMF はラスタライズしてPNG化 (LibreOffice非依存で実装)

#### F6. OCR 対応

スキャンPDF・画像PDFを検出し、OCR を自動適用できること。

サポート対象 OCR エンジン:

| エンジン | 用途 | ライセンス | GPU |
| --- | --- | --- | --- |
| NDLOCR-Lite | 日本語(近代資料・手書き・英文混在) | CC BY 4.0 | 不要 |
| NDL古典籍OCR-Lite | くずし字・漢籍 | CC BY 4.0 | 不要 |
| Tesseract | 多言語汎用 | Apache 2.0 | 不要 |
| Surya | 多言語(版面解析強い) | GPL/商用要相談 | 推奨 |
| 外部API | クラウドOCR | 各社規約 | 不要 |

国立国会図書館の [NDLOCR-Lite](https://github.com/ndl-lab/ndlocr-lite)
を一級市民として組み込む理由:

- GPU 不要でノートPC・一般的家庭用環境で動作
- Windows 11 / macOS Sequoia / Ubuntu 22.04 で動作確認済み
- 日本語の縦書き・近代資料に対する精度が高い
- 英文・手書き文字も実験的にサポート
- CC BY 4.0 で商用利用可能
- 古典籍向けには NDL古典籍OCR-Lite が別途あり、くずし字対応

エンジン選択ロジック:

- 言語自動判定 (日本語混在も含む) で最適エンジンを選択
- 古典籍 / くずし字検出時は NDL古典籍OCR-Lite へ自動切り替え
- ユーザーによる明示指定も可能 (`--ocr ndlocr-lite`)
- OCR 利用の有無・エンジン名をログに明示

```bash
atom scan.pdf --ocr auto -o out.md        # 自動選択
atom scan.pdf --ocr ndlocr-lite -o out.md # 明示指定
atom kuzushiji.pdf --ocr ndl-koten -o out.md
```

#### F7. 構造化出力 (人間可読性ファースト)

本ツールの中核要件。MarkItDown が割り切った「LLM が読めれば
人間向けには不向き」というポジションを取らない。**出力は常に
人が読んで理解できるよう構造化する** ことを第一目標とする。

具体的に保持・再構築する構造:

- 見出し階層 (H1〜H6)。フォントサイズや太字から推論して再構築
- 段落・改行 (PDF の物理改行と論理改行を区別)
- 箇条書き・番号付きリスト (ネスト含む)
- 図表番号・キャプション
- 脚注・後注・参考文献リンク
- 数式 (TeX形式で出力)
- ソースコード (フォントから判定してコードブロック化)
- 目次 (TOC を検出して相互参照リンクに変換)

構造化の優先順位:

1. ネイティブ形式 (DOCX/PPTX の段落スタイル等) があればそれを使う
2. PDF など視覚情報しかない場合はフォント・レイアウトから推論
3. 推論で不確実な場合は警告を出してフォールバック
4. LLM オプション有効時は LLM による再構造化を適用 (F8参照)

#### F8. LLM による再構造化・翻訳 (オプション)

`--llm` オプションを有効化すると、変換結果に対して LLM による
品質向上を適用できる。デフォルトは無効。

LLM の用途:

- **再構造化**: OCR結果や PDF からの推論結果を LLM が読み直し、
  見出し階層の修正・段落の再整形・誤認識文字の補正を行う
- **翻訳**: 任意言語ペア間の翻訳。原文の Markdown 構造は保持
- **要約・目次生成**: 補助機能として、長文の TOC 自動生成

```bash
# 再構造化のみ
atom input.pdf --llm claude-opus --restructure -o out.md

# 翻訳付き
atom input.pdf --llm claude-opus --translate ja -o out.md
```

サポートする LLM バックエンド:

| バックエンド | 用途 | データ送信先 |
| --- | --- | --- |
| Anthropic Claude API | クラウド最高品質 | Anthropic |
| OpenAI API | クラウド汎用 | OpenAI |
| ローカル LLM (Ollama 等) | オンプレ完結 | ローカルのみ |
| 社内ホスト OpenAI互換 | 企業内 LLM ゲートウェイ | 社内 |

設計上の制約:

- LLM 利用時は **どこにデータを送るかを起動時に明示** し、
  ユーザー確認を取る (機密文書の事故送信防止)
- バッチ処理時は確認をスキップできるが、設定ファイルでの
  事前同意が必要
- LLM 出力は原文と diff を取れる形で保存し、変更箇所を検証可能
- 翻訳結果には原文への対応関係をフロントマターで記録

#### F9. 出力形式の拡張 (優先度低)

主目的は Markdown 出力だが、中間 AST を持つアーキテクチャを
活かして以下も対応可能とする。

- **MDX** — JSX 埋め込み可能な Markdown 拡張。React 系
  ドキュメントサイト (Docusaurus 等) 向け
- **HTML** — スタンドアロン HTML / HTML フラグメント
- **HedgeDoc スライド形式** — 既存ワークフローへの組み込み

```bash
atom input.docx --format mdx  -o out.mdx
atom input.docx --format html -o out.html
```

優先度は低いが、Markdown Writer と並列に MDX Writer / HTML
Writer を実装することで達成する。AST が共通なので追加コストは
小さい。

#### F10. 観測性

変換時に以下を出力する。

- どのページ・どの要素で忠実度が落ちたか (warning)
- 解釈不能な要素の一覧
- 入力ファイルのメタデータ抽出結果
- 変換時の処理時間・使用機能のレポート

```bash
atom input.pdf -o out.md --report report.json
```

### 非機能要件

#### N1. 実行環境

- 単一バイナリで配布できること
- Linux / macOS / Windows をネイティブサポート
- Python / Node.js / Haskell ランタイム不要
- 企業プロキシ環境 (SSL Inspection 有) を前提とした
  CA 証明書設定をサポート

実装言語は N6 (ハーネスエンジニアリング適合性) を満たすものから選定。

#### N2. ローカル完結

- デフォルト構成で外部APIを呼ばないこと
- LLM や OCR の外部利用はオプトインで、明示的フラグが必要
- 機密データのテレメトリ送信を一切行わない

#### N3. パフォーマンス

- 100ページ程度の DOCX を 10秒以内 (テキストPDFは 30秒以内)
- ストリーミング処理対応 (大容量ファイルでメモリ膨張しない)
- バッチ処理時の並列化サポート

#### N4. 拡張性

- フォーマットごとの変換ロジックをプラグインで差し替え可能
- 出力後処理 (filter) を WASM プラグインで書ける
- 設定ファイル (`atom.config.toml` 等) でプロジェクト単位の方言固定

#### N5. 品質保証

- 各フォーマットごとに golden test (入力 → 期待 Markdown) を整備
- markdownlint-cli2 で出力が常に lint passing
- ベンチマークデータセットで Marker / Docling / Pandoc に対する
  忠実度スコアを継続測定

#### N6. ハーネスエンジニアリング適合性

本ツールはルールベース変換と LLM 補正のハイブリッドであり、
品質を一回作って終わりにはできない。**AI エージェントが自律的に
評価関数を回しながら継続改善できる開発ハーネス**を成立させる
ことを非機能要件として明示する。

ハーネスを構成する要素:

- **決定的な評価関数 (eval function)** が言語側で書きやすいこと
- **目的関数 (objective function) の自動最適化**がやりやすいこと
  (パラメータ探索、回帰検出、忠実度スコアの監視)
- **再現性のあるサンドボックス実行**が言語側で保証されること
- **AST レベルの diff/比較**が型システムで安全に書けること
- **エージェント (Claude Code 等) が出力を構造化で受け取れる**
  CLI / API になっていること

具体的に必要な機構:

1. **fixture ベースの golden test** が build-in できる
   (入力ファイル → 期待AST → 期待Markdown のスナップショット)
2. **プロパティベーステスト**が容易に書ける
   (任意の Markdown を生成 → 出力 → 再パース → 元と一致)
3. **メトリクス収集の自動化**
   (CER/WER、構造一致率、忠実度スコアを CI で時系列追跡)
4. **ベンチマークが言語標準で書ける**
   (パフォーマンス回帰を fail にできる)
5. **差分可視化が容易**
   (LLM 再構造化前後、翻訳前後の構造保持率を機械的に検証)
6. **クロスコンパイル容易**
   (CI で全プラットフォーム向けを一発ビルドできる)

これらが「言語+ツールチェーン標準」で揃わない場合、ハーネスを
自前で整備するコストが本体実装と同等になり、AI エージェントが
自律的に改善ループを回すことができない。

### 実装言語の選定

N1〜N6 の要件を踏まえ、候補言語をハーネスエンジニアリング適合性
の観点で評価する。

#### 評価軸

| 軸 | 内容 |
| --- | --- |
| 単一バイナリ | ランタイム不要で配布できるか |
| 型システム | AST 操作・diff・比較が型で守られるか |
| テスト標準 | golden / property / bench が標準ツールにあるか |
| サンドボックス | プロセス分離・WASM 実行環境の標準化度合い |
| 構造化出力 | エージェントに JSON 等で結果を渡しやすいか |
| 並列性 | 大量文書のバッチ評価が容易か |
| エコシステム | PDF/Office パース・OCR バインディングの存在 |

#### 候補比較

| 言語 | 単一バイナリ | 型 | テスト標準 | サンドボックス | エコシステム |
| --- | --- | --- | --- | --- | --- |
| Rust | ◎ | ◎ | ◎ | ◎ (WASM) | ○ |
| Go | ◎ | △ | ○ | △ | ○ |
| Zig | ◎ | ○ | ○ | △ | △ |
| Haskell | ○ | ◎ | ○ | △ | ○ (Pandoc) |
| Python | × | △ | ○ | △ | ◎ |
| TypeScript | △ (Deno/Bun) | ○ | ○ | ○ | ○ |

#### 評価詳細

##### Rust

- `cargo test` / `cargo bench` / `criterion` でテスト・ベンチが標準
- `insta` クレートによるスナップショットテストが golden test に最適
- `proptest` / `quickcheck` でプロパティベーステストが容易
- 型システムが強く、AST の sum type が網羅性検査される
  (LLM が誤った AST 変換を生成しても compile error で弾ける)
- WASM ターゲットが一級市民。プラグインを WASM で書く要件 (N4)
  と完全に整合
- 単一バイナリ・クロスコンパイルが `cargo build --target` で完結
- 結果を `serde` 経由で JSON 化、エージェントへの構造化出力が
  ボイラープレート最小
- 弱点: PDF / Office パースのエコシステムは Python に劣る。
  `pdfium-render`, `lopdf`, `docx-rs` 等で組むか、C 系ライブラリ
  との FFI が必要

##### Go

- 単一バイナリ・クロスコンパイルは強い
- 標準 `testing` パッケージで golden test 可能だが、Rust の
  `insta` ほど洗練されていない
- 型システムが Rust ほど強くない (sum type 不在)。AST 変換の
  網羅性が compile time で保証されず、エージェントによる
  自動改善時のガードレールが弱い
- ジェネリクスが後付けで、AST 操作の表現力に制約
- WASM サポートは Rust より弱い

##### Zig

- 個人プロジェクトとして触っているので候補ではあるが、エコシステム
  が未成熟。PDF/Office パースのライブラリがほぼ存在しない
- comptime による評価関数記述には可能性あり
- 現時点では production 用途より実験用途

##### Haskell

- Pandoc の実装言語であり、AST 操作・型システムは最強クラス
- QuickCheck によるプロパティベーステストが言語起源で強い
- 弱点: 単一バイナリ配布のしやすさ、依存解決の難しさ、ビルド時間
- AI エージェントが触るには学習コスト・型エラーの難解さが障壁

##### Python

- エコシステムが圧倒的 (markitdown / docling / marker / mammoth
  すべて Python)
- 既存実装を流用できる
- 弱点: 単一バイナリ配布が困難 (PyInstaller / Nuitka はあるが
  実用性に難)。Pyright/mypy でも Rust 並の型安全は得られない。
  AST の網羅性検査が弱い

##### TypeScript (Deno / Bun)

- Deno は権限分離 (sandbox) が標準で組み込まれている。
  これは ハーネス要件 (サンドボックス) と相性が良い
- 単一バイナリ配布は Deno/Bun の compile 機能で可能だが Rust より
  バイナリサイズ大
- 型システムは構造的部分型で柔軟だが、Rust のような網羅性検査の
  厳格さはない

#### 結論: Rust を主言語とする

ハーネスエンジニアリング適合性で Rust が最も優位。

選定理由:

1. **評価関数を型で守れる** — AST を `enum` (sum type) で定義
   すれば、AI エージェントが新しい変換ロジックを書いたとき、
   全ケースを網羅していなければ compile error になる。これは
   ハーネスのガードレールとして最強
2. **golden test が `insta` で 1行** — スナップショットの追加・
   更新・diff 表示がワンコマンド。回帰検出が自動化しやすい
3. **`criterion` でベンチが回帰検出付き** — 性能劣化を CI で
   検出できる
4. **WASM プラグインが一級市民** — N4 (拡張性) と整合
5. **`serde` で構造化出力が自動** — エージェントが結果を読み取って
   次のアクションを決める loop が組みやすい
6. **クロスコンパイルが `cargo build --target` で完結** — CI で
   全プラットフォーム向けを並列ビルド

#### 補完戦略

Rust だけでは PDF / Office パースのエコシステムが弱い。これは
以下で補う:

- **NDLOCR-Lite は Python 製** → 別プロセスで呼び出し、JSON で
  通信する境界とする。Rust 本体は OCR エンジンの実装には踏み込まない
- **PDF パースは pdfium (Chrome の PDF エンジン) を `pdfium-render`
  経由で利用** — 実績ある C++ エンジンに型安全な Rust ラッパー
- **OOXML (DOCX/PPTX/XLSX) パースは `docx-rs` / `office-rs`
  系のクレートを使い、不足分は自前実装**
- **LLM クライアントは `anthropic-rs` / `async-openai` を利用**

#### エージェント駆動開発ハーネスの構成

実装フェーズで AI エージェント (Claude Code 等) が自律改善ループを
回すための具体的なハーネス構成。

```text
[テストデータセット] --> [変換実行] --> [評価関数群]
                                            |
                                            v
                                      [メトリクスJSON]
                                            |
                                            v
                              [エージェントが diff を読んで改善]
                                            |
                                            v
                                      [PR / patch 生成]
                                            |
                                            v
                                      [CI で回帰検証]
```

具体的な評価関数 (Rust で実装):

| 評価関数 | 入力 | 出力 |
| --- | --- | --- |
| `structure_fidelity()` | 期待AST, 出力AST | 構造一致率 0.0〜1.0 |
| `heading_recall()` | 原文, Markdown | 見出し復元率 |
| `table_integrity()` | 原文, Markdown | テーブル破損率 |
| `ocr_cer()` | Ground truth, OCR出力 | Character Error Rate |
| `translation_structure_preserve()` | 翻訳前MD, 翻訳後MD | 構造保持率 |
| `lint_score()` | Markdown | markdownlint エラー数 |

これらを CI で時系列追跡し、エージェントは `cargo test` の出力と
メトリクスJSON を読んで次の改善箇所を特定する。

### アーキテクチャ要件

#### A1. パイプライン構造

すべての変換を統一されたパイプラインに乗せる。

```text
Input File
  -> Format Detector
  -> Parser (format-specific)
  -> Intermediate AST (Pandoc AST 互換を検討)
  -> Markdown Writer (flavor-specific)
  -> Linter (post-process)
  -> Output
```

#### A2. 中間表現

Pandoc AST に類似した中間表現を持ち、入力パーサーと出力ライタを
完全に分離する。これにより以下が可能になる。

- 新フォーマット追加はパーサー実装のみで済む
- 出力方言追加はライタ実装のみで済む
- パイプライン途中で AST レベルの変換 filter を挟める
- LLM 再構造化・翻訳は AST に対する filter として実装する

出力 Writer 群 (優先度順):

| Writer | 優先度 | 備考 |
| --- | --- | --- |
| Markdown (CommonMark/GFM/lint準拠) | 高 | 主目的 |
| HedgeDoc スライド | 中 | 既存ワークフロー連携 |
| MDX | 低 | Docusaurus等のドキュメントサイト向け |
| HTML | 低 | スタンドアロン / fragment |

#### A3. プラグイン境界

| 境界 | 形式 | 例 |
| --- | --- | --- |
| Input Parser | dynamic library / WASM | カスタムフォーマット |
| AST Filter | WASM | 見出しレベル正規化 |
| Output Writer | dynamic library | 独自方言出力 |
| OCR Engine | プロセス間通信 | Tesseract差し替え |

### CLI / API 要件

#### C1. CLI

```bash
atom [INPUT] [OPTIONS]

OPTIONS:
  -o, --output <PATH>         出力先 (省略時は stdout)
  -f, --format <FMT>          出力形式 (md/mdx/html, デフォルト md)
  --flavor <FLAVOR>           Markdown方言を指定
  --extract-media <DIR>       画像抽出先ディレクトリ
  --ocr <ENGINE>              OCRエンジン (auto/ndlocr-lite/ndl-koten/
                              tesseract/surya/none)
  --llm <MODEL>               LLMバックエンド (claude-opus/gpt-4/
                              ollama:llama3/none)
  --restructure               LLMで構造を再整形 (要 --llm)
  --translate <LANG>          翻訳先言語 (要 --llm)
  --report <PATH>             変換レポートをJSONで出力
  --strict                    警告をエラーとして扱う
  --config <PATH>             設定ファイルを指定
```

#### C2. ライブラリAPI

CLIと同じことが Rust / Python バインディング経由で可能。
ストリーミング API を提供し、巨大ファイル対応とする。

```rust
let result = anything_to_markdown::Converter::new()
    .flavor(Flavor::Gfm)
    .extract_media("./media")
    .convert_file("input.docx")?;
```

### スコープ外

明示的に対象外とするもの。

- WYSIWYG な双方向編集 (Markdown ↔ DOCX のラウンドトリップ)
- Markdown → Office 形式変換 (Pandoc の領域)
- リアルタイムコラボレーション機能
- クラウドサービス提供 (CLI / ライブラリのみ)
- 独自 OCR モデルの開発 (既存エンジンを統合する側に徹する)
- LLM モデルのファインチューニング

## 評価基準

新ツールが従来ツールを置き換える価値があるかを判定するための指標。
整った構造を持つ入力だけで高得点を取っても、本ツールの価値は証明できない。
評価は、構造が失われた文書、見た目依存の文書、既存ツールが破綻しやすい文書を
中心に行う。

| 指標 | 目標値 |
| --- | --- |
| 対応フォーマット数 | 7形式以上 (HTML/PDF/DOCX/XLSX/PPTX/ODT等) |
| テーブル忠実度スコア | Pandoc比 +20%以上 |
| 構造化忠実度 (人手評価) | MarkItDown 比 +30%以上 |
| 見出し階層復元率 (PDF) | 90%以上 |
| 日本語OCR精度 (NDLOCR-Lite) | 既存NDLOCRと同等以上 |
| 起動時間 | 100ms以下 (Pandoc同等以下) |
| バイナリサイズ | 50MB以下 (ML依存なし時) |
| 依存ランタイム | 0個 (単一バイナリ) |
| 外部送信 | デフォルト構成で0件 |
| LLM翻訳の構造保持率 | 95%以上 (見出し・リスト・表が維持) |

## 参考資料

- [microsoft/markitdown](https://github.com/microsoft/markitdown)
- [jgm/pandoc](https://github.com/jgm/pandoc)
- [mwilliamson/mammoth.js](https://github.com/mwilliamson/mammoth.js)
- [datalab-to/marker](https://github.com/datalab-to/marker)
- [DS4SD/docling](https://github.com/DS4SD/docling)
- [Pandoc User's Guide](https://pandoc.org/MANUAL.html)
- [NDLOCR-Lite](https://github.com/ndl-lab/ndlocr-lite)
- [NDLOCR-Lite 公開のお知らせ](https://lab.ndl.go.jp/news/2025/2026-02-24/)
- [NDL古典籍OCR-Lite の使い方](https://lab.ndl.go.jp/data_set/lite-usage/)
