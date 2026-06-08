# MarkItDown成功・旧atom非成功ファイルのrelease確認

実施日: 2026-06-09

## 対象

- `benchmark/reports/markitdown-inventory.tsv` で MarkItDown が `ok`
- `benchmark/reports/atom-inventory.tsv` で旧atomが `ok` ではない
- 上記条件に一致する34件
- 実行コマンド: `target/release/atom`

`benchmark/reports` はGit管理外の実行成果物置き場であるため、本書に
判断サマリを記録する。

## 結果

| 項目 | 結果 |
| ---- | ---- |
| 対象ファイル数 | 34 |
| 現行atom成功数 | 34 |
| MarkItDownより高速だった件数 | 34 |
| 最大 atom / MarkItDown 処理時間比 | 0.174 |
| atom処理時間中央値 | 379 ms |
| atom最大処理時間 | 3390 ms |
| atom出力に`(cid:)`が残った件数 | 0 |
| atom出力に置換文字が残った件数 | 0 |
| atom出力に強い文字化け兆候があった件数 | 0 |

## 低文字数比の確認

atom / MarkItDown の文字数比が 0.65 未満だったものは5件あった。
Markdownサンプルを確認した結果、atom側の本文は読める状態だった。
MarkItDown側は、スライドやレイアウト由来の断片的な表セル、または
CID placeholder により文字数が膨らんでいるケースが含まれていた。

- `FGg0JdeFX1697170986.pdf`
  - 文字数比: 0.463
  - 処理時間: atom 199 ms / MarkItDown 7096 ms
  - atom本文は日本語として読める。MarkItDownは断片的な表レイアウトを含む。
- `SpeakerDeck掲載用_アルゴリズム問題_231102_coding_test_koma_V3 (1).pdf`
  - 文字数比: 0.620
  - 処理時間: atom 39 ms / MarkItDown 1856 ms
  - atom本文は日本語スライド本文として読める。
- `SpeakerDeck掲載用_アルゴリズム問題_231102_coding_test_koma_V3.pdf`
  - 文字数比: 0.620
  - 処理時間: atom 39 ms / MarkItDown 1730 ms
  - 上記の重複文書。
- `designingfortheordinary-220126124909.pdf`
  - 文字数比: 0.127
  - 処理時間: atom 38 ms / MarkItDown 1687 ms
  - atomは日本語本文を復元できている。MarkItDownは大半が`(cid:)`
    placeholder だった。
- `internal-quality-issues-caused-by-organizational-design.pdf`
  - 文字数比: 0.581
  - 処理時間: atom 3390 ms / MarkItDown 116208 ms
  - atom本文は日本語として読める。MarkItDownは断片的な表レイアウトを含む。

## 判断

この対象集合については、現行atomがMarkItDownより速く、かつ読める
Markdownを出力できている。今後の改善対象はバックエンド切替ではなく、
PDF表紙・スライドの表化アーティファクトや読順復元の品質改善とする。
