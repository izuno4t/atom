# bonjil Local Agent Rules

このファイルは、このリポジトリだけに適用する補助ルールを定義する。
上位のAGENTS規約と衝突する場合は、上位規約を優先する。

## 作業の進め方

- `docs/requirements.md` を根拠に実装する。
- `docs/tasks.md` のステータスを、着手時は 🚧、完了時は ✅ に更新する。
- DependsOn が未完了のタスクには着手しない。
- 完了扱いにする前に `make ci` を実行し、結果を確認する。

## Fixture と expected

- `tests/fixtures/**/*.expected.md` は人間レビュー対象として扱う。
- 失敗を消す目的で `expected.md` を自動更新しない。
- expected を変更する場合は、入力仕様またはWriter仕様の変更理由を残す。

## 評価の保護

- 評価関数の失敗を隠すために実装を弱めない。
- `tests/thresholds.toml` のしきい値を下げて失敗を隠さない。
- 評価JSON、diff、warningを確認して原因を切り分けてから修正する。

## 外部送信

- デフォルトでは外部LLMや外部OCRサービスへ送信しない。
- 外部送信を追加する場合は、送信先、送信内容、同意設定を明示する。
