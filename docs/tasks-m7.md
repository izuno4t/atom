# TASKS

マイルストーン: M7
ゴール: M6の実コーパス評価で抽出された改善候補を、変換本体の機能改善として実装する

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

## 移動元

- 旧タスク表: `docs/archive/tasks-m1-m5.md`
- 移動対象: 旧 `TASK-076` 以降の未完了タスクとBacklog
- 移動理由: 現行 `docs/tasks.md` をM6評価プロセス専用にするため

## タスク一覧

| ID | Status | Summary | DependsOn |
| ---- | ---- | ---- | ---- |
| TASK-001 | ⏳ | 接続する変換本体のLLMプロバイダ呼び出し | - |
| TASK-002 | ⏳ | 実装するLLM再構造化プロンプト | TASK-001 |
| TASK-003 | ⏳ | 実装するLLM整形結果の検証と差分保存 | TASK-002 |
| TASK-004 | ⏳ | 追記する変換本体のLLM実行手順 | TASK-003 |
| TASK-005 | ⏳ | 検証する画像資料のLLM補助変換方針 | TASK-004 |
| TASK-006 | ⏳ | 実装するatomネイティブPDF抽出器とレイアウト復元 | - |

## タスク詳細（補足が必要な場合のみ）

### TASK-001

- 旧ID: `TASK-076`
- 補足: 変換本体からOllamaまたは外部LLM backendへ実際に送れるようにする。
- 対象: `--llm ollama:<model>`、OpenAI互換endpoint、timeout、
  失敗時warning、`--allow-external-send` による外部送信制御。
- 注意: デフォルトは外部送信ゼロを維持する。

### TASK-002

- 旧ID: `TASK-077`
- 補足: OCR結果やPDF推論結果から、人間が読めるMarkdownへ整えるための
  変換本体用promptを固定する。
- 注意: LLMに原文にない内容を補完させない。

### TASK-003

- 旧ID: `TASK-078`
- 補足: LLM整形結果をASTとして再解析し、構造破壊や大量欠落を検出してから
  反映する。
- 注意: 検証に失敗したLLM応答は破棄し、元の変換結果を出力する。

### TASK-004

- 旧ID: `TASK-079`
- 補足: Ollama導入手順を重複させず、変換本体で `--llm --restructure` を
  使う実行例だけを追記する。
- 注意: 外部APIキーや機密文書送信を前提にしない。

### TASK-005

- 旧ID: `TASK-080`
- 補足: 画像、図、スキャン資料に対し、LLMまたはVLM補助を使う範囲を
  検証する。
- 注意: 画像そのものを外部へ送る処理は、送信先、送信内容、同意条件を
  実装前に明示する。

### TASK-006

- 旧ID: `TASK-081`
- 補足: PDF抽出器と後段整形は `atom` の中核機能として実装する。
- 注意: M6評価で得た原因分類と代表サンプルを入力にして、原因単位で
  実装範囲を切る。

## Backlog一覧

| ID | Status | Summary | DependsOn |
| ---- | ---- | ---- | ---- |
| BACKLOG-001 | ⏳ | 実装するOpenDocument入力パーサ | - |
| BACKLOG-002 | ⏳ | 実装するMDXライター | - |
| BACKLOG-003 | ⏳ | 実装するHTMLライター | - |
| BACKLOG-004 | ⏳ | 実装するHedgeDocスライドライター | - |
| BACKLOG-005 | ⏳ | 実装するWASMプラグインSDK | - |
| BACKLOG-006 | ⏳ | 接続するNDL古典籍OCR-Lite | - |
| BACKLOG-007 | ⏳ | 接続するSurya OCR | - |
| BACKLOG-008 | ⏳ | 整備する大規模corpora運用 | - |

## Backlog詳細（補足が必要な場合のみ）

### BACKLOG-001

- 旧ID: `BACKLOG-001`
- 補足: ODT / ODS / ODP は要件上次フェーズ扱いのため後回しにする。

### BACKLOG-002

- 旧ID: `BACKLOG-002`
- 補足: MDXは主目的ではないためMarkdown Writer安定後に追加する。

### BACKLOG-003

- 旧ID: `BACKLOG-003`
- 補足: HTML出力はAST共通化の効果を確認してから実装する。

### BACKLOG-004

- 旧ID: `BACKLOG-004`
- 補足: HedgeDoc連携は既存ワークフロー需要が確定してから実装する。

### BACKLOG-005

- 旧ID: `BACKLOG-005`
- 補足: WASMプラグインはコア変換品質が安定してから公開する。

### BACKLOG-006

- 旧ID: `BACKLOG-006`
- 補足: 古典籍OCRは通常OCRの境界と評価が安定してから追加する。

### BACKLOG-007

- 旧ID: `BACKLOG-007`
- 補足: SuryaはライセンスとGPU前提の運用判断が必要。

### BACKLOG-008

- 旧ID: `BACKLOG-008`
- 補足: 大規模corporaはGit管理外の保存先と再現手順を別途決める。
