# Tool Runners

このディレクトリは、比較に利用する外部ツールのDocker実装を管理する。

## 共通契約

各ランナーは次の契約を満たす。

- Dockerコンテナ内で実行する
- 第1引数に入力ファイルパスを受け取る
- 第2引数にMarkdown出力先パスを受け取る
- 第3引数にreport JSON出力先パスを受け取る
- Markdownを指定された出力先へ書く
- 実行サマリを標準出力へ短いJSONとして出す
- エラーは標準エラーと終了コードで返す
- 外部ネットワークなしでも実行できるよう、依存関係はイメージ内に固定する

外部比較用scriptはこの契約を前提に、ツール別Dockerイメージを起動する。

## イメージ名

| ツール | 既定イメージ |
| ---- | ---- |
| Pandoc | `atom-eval-pandoc:latest` |
| MarkItDown | `atom-eval-markitdown:latest` |
| Docling | `atom-eval-docling:latest` |
| PyMuPDF4LLM | `atom-eval-pymupdf4llm:latest` |
| Mammoth.js/Turndown | `atom-eval-mammoth-js:latest` |

## ビルド例

```bash
docker build -t atom-eval-pandoc:latest benchmark/tools/pandoc
docker build -t atom-eval-markitdown:latest benchmark/tools/markitdown
docker build -t atom-eval-docling:latest benchmark/tools/docling
docker build -t atom-eval-pymupdf4llm:latest benchmark/tools/pymupdf4llm
docker build -t atom-eval-mammoth-js:latest benchmark/tools/mammoth-js
```
