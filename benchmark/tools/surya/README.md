# Surya

Surya OCR の導入手順。今回のエンジン比較
（[methods/ocr-engine-evaluation.md](../../methods/ocr-engine-evaluation.md)）では
実測対象外だが、`atom` の subprocess エンジンとして将来使うための手順を残す。

## インストール

```bash
pip install surya-ocr
```

CPU・Apple Silicon では推論バックエンドに `llama.cpp` が必要。

```bash
brew install llama.cpp
```

NVIDIA GPU では Docker と NVIDIA Container Toolkit を入れ、`vllm` バックエンドを
使う。モデルは初回実行時に自動取得され、手動ダウンロードは不要。既存の推論サーバーを
使う場合は `SURYA_INFERENCE_URL=http://host:port/v1` を指定する。

出典は [Surya 公式リポジトリ](https://github.com/VikParuchuri/surya)。

## CLI の挙動

```bash
surya_ocr DATA_PATH --output_dir OUT
```

主なフラグは `--images`（注釈画像の保存）、`--output_dir`（結果ディレクトリ）、
`--page_range`（ページ指定）、`--keep_server`（サーバー常駐）。出力は標準出力ではなく、
結果ディレクトリへ `results.json`（検出ブロック・バウンディングボックス・信頼度・
HTML・読み順）を書く。

## `atom` 連携時の注意（契約の不一致）

`atom` の `src/integrations/ocr.rs::run_subprocess` は
`Command::new("surya_ocr").arg(input)` を実行し、認識テキストを標準出力から読む。
一方 Surya は標準出力にテキストを出さず `results.json` をディレクトリへ書くため、
そのままでは噛み合わない。連携にはラッパーが必要。

ラッパーの方針（未検証）。

- `surya_ocr <input> --output_dir <tmp>` を実行する。
- `<tmp>/results.json` を読み、テキストブロックを読み順に連結して標準出力へ出す。
- このラッパーを `--ocr surya`（コマンド名 `surya_ocr`）の代わりに、または
  `--ocr external:<wrapper>` 経路で呼ぶ。

`results.json` の正確なスキーマと読み順フィールドは、実際に `surya_ocr` を1回
走らせて確認すること（本手順では未実行）。

## ライセンス上の注意

- コードは Apache-2.0。
- モデル重みは「modified AI Pubs Open Rail-M」で、研究・個人利用・小規模スタートアップ
  （資金または売上 500 万ドル未満）は無償。商用は別途ライセンスが必要になり得るため、
  `atom` の配布形態によっては確認が要る。
