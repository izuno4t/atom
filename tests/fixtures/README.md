# Fixtures

評価データセットは `unit`、`integration`、`regression` の3層で管理する。

- `unit`: 1ファイル1機能を確認する最小入力
- `integration`: 実文書に近い中規模入力
- `regression`: 過去に壊れたケースの最小再現

`*.expected.md` は人間レビュー対象であり、自動更新しない。

## Fixture 方針

unit fixture は最小構文だけでなく、実文書で頻出する「構造が曖昧な」
小さなパターンも含める。
きれいに構造化された入力だけを増やしても、本ツールの問題意識を検証できない。
ただし、公開文書そのものではなく、リポジトリ内で作成した合成fixtureを使う。

代表パターン:

- 見出し、本文、箇条書きが混在するスライド
- 複数のテキストボックスがあるスライド
- 見出し行、空セル、複数データ行を含む表
- shared strings を使う表
- セル内改行や数値セルを含む表
- 見た目上は表だが、内部構造が不完全な文書
- 視覚順とXML順がずれやすいOffice文書
- PDF化で見出し、段落、キャプションの意味が失われた文書

外部公開文書や手元の実文書は、Git管理外の `evaluation/inputs/` に置いて
実コーパス評価で扱う。Git管理する場合は `integration/MANIFEST.md` または
`regression/MANIFEST.md` に出典とライセンスを記録する。

評価やfixture化の前に変換結果だけを確認する手元入力は、Git管理外の
`tests/inputs/` に置く。`tests/fixtures/` へ移すのは、期待Markdownと
レビュー観点を固定できる状態になってからにする。
