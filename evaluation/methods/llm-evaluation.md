# Local LLM Evaluation

この文書は、LLMを使ったMarkdown評価をローカル環境だけで実行する手順を定義する。
外部API送信は扱わない。

## Ollama構築手順

Ollamaを使う場合は、公式配布物をインストールしてローカルAPIを起動する。
インストール後、モデルを取得する。

```bash
ollama pull qwen2.5:7b-instruct
ollama serve
```

別ターミナルで疎通を確認する。

```bash
ollama list
```

`atom-llm-eval` は `http://127.0.0.1:11434` または `http://localhost:11434`
にだけ送信する。評価対象ファイルとMarkdownはローカルで読み、プロンプトとして
Ollamaへ渡す。

## 入力

先にLLMなし評価を実行し、`review_candidates` を含むreport JSONを生成する。

```bash
make corpus-eval-full
```

ドライランではOllamaを呼ばず、LLMへ渡す予定のJSONLだけを生成する。

```bash
make llm-eval LLM_EVAL_DRY_RUN=1
```

Ollamaへ実際に送る場合は、モデルを起動した状態で実行する。

```bash
make llm-eval LLM_EVAL_MODEL=qwen2.5:7b-instruct
```

## JSONLスキーマ

出力先は既定で `evaluation/reports/llm-eval.jsonl` とする。
各行は1件のレビュー候補に対応する。

### Dry-run request

```json
{
  "schema_version": "atom.llm-eval.v1",
  "model": "qwen2.5:7b-instruct",
  "input": "input.pdf",
  "priority": "high",
  "reasons": ["atom_failed_but_baseline_succeeded"],
  "prompt_kind": "pair_comparison",
  "prompt": "..."
}
```

### Evaluation result

```json
{
  "schema_version": "atom.llm-eval.v1",
  "model": "qwen2.5:7b-instruct",
  "input": "input.pdf",
  "priority": "high",
  "reasons": ["atom_failed_but_baseline_succeeded"],
  "prompt_kind": "pair_comparison",
  "response": "{\"schema_version\":\"atom.llm-eval.v1\",...}",
  "error": null
}
```

LLMの応答本文には次のJSON形を要求する。

```json
{
  "schema_version": "atom.llm-eval.v1",
  "scores": {
    "title_heading": 0,
    "text_paragraph": 0,
    "list": 0,
    "table": 0,
    "figure_image": 0,
    "caption": 0,
    "footnote": 0,
    "formula_value": 0,
    "reading_order": 0,
    "warnings": 0
  },
  "winner": null,
  "confidence": "low",
  "findings": ["short evidence-based finding"],
  "fixture_candidates": ["minimal reproducible issue"]
}
```

## プロンプト

`single_markdown_score` は、成功出力が1つ以下の候補を単体採点する。
`pair_comparison` は、複数ツールのMarkdown出力を比較する。

どちらも同じルーブリックを使い、0から2点で採点する。
入力ファイルそのものは送らず、LLMなし評価で生成したMarkdown出力を送る。

## 改善候補とfixture化候補

LLM応答の `findings` は実装修正候補として扱う。
`fixture_candidates` は、次の条件を満たす場合だけfixture化へ進める。

- 再配布可能な公開データ、または最小再現ファイルで確認できる。
- 問題が特定形式の一般的な破綻として説明できる。
- expected Markdownだけでなく、warningまたはreportで失敗理由を確認できる。
- `evaluation/public-corpus*.md` に出典、ライセンス、観測結果を記録できる。
