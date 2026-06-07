# atom 開発ハーネス

## このリポジトリは何か

Anything to Markdown。ドキュメントを人間が読める構造化 Markdown に
変換する Rust 製 CLI / API。
要件は `docs/requirements.md`、実行計画は `docs/tasks.md` を参照する。

## 改善ループの回し方

1. `just test` でテストを確認する。
2. `just eval` で変換レポート JSON を確認する。
3. 失敗 fixture または warning の原因を、入力パーサ、AST、Writer、評価関数に切り分ける。
4. 最小修正を入れる。
5. `just ci` で回帰がないことを確認する。

## やってはいけないこと

- `tests/fixtures/**/*.expected.md` を根拠なく書き換えない。
- 評価関数のしきい値を下げて失敗を隠さない。
- 外部 LLM 送信をデフォルトで有効化しない。
- 機密文書を fixture に追加しない。

## 現在の実装境界

- DOCX は `unzip` で `word/document.xml` を読む最小実装。
- PDF はヒューリスティックなテキスト抽出のみ。
- PPTX / XLSX は入口と warning を実装済みで、構造変換は今後拡張する。
- OCR / LLM は境界を実装済みで、外部呼び出しは明示設定時のみ扱う。
