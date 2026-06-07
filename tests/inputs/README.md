# Test Inputs

このディレクトリは、評価やfixture化の前に手元文書を置いて変換出力を確認する
ための作業場所である。

## 用途

- `atom` / `atom` の変換結果を手元で確認する
- fixture化する前のPDF、DOCX、PPTX、XLSX、HTMLを一時的に置く
- 期待Markdownや評価レポートを作る前の入力を整理する

## 管理ルール

- 実ファイルはGit管理しない。
- 機密文書、認証情報、社外秘資料をコミットしない。
- 再現テストとして残す場合は、合成fixtureへ加工して
  `tests/fixtures/` へ移す。
- 実コーパス評価に使う場合は `evaluation/inputs/` へ置く。

## 例

```sh
atom tests/inputs/sample.pdf -o target/manual/sample.md
markdownlint-cli2 target/manual/sample.md
```
