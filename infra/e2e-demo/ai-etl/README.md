# AI ETL E2E Demo

Favnir v69 の全機能を使った完全な AI ETL デモパイプライン。

## パイプライン概要

```
data/articles.csv
       |
  LoadArticles (seq)
       |
  EmbedAndSummarize x4 (par) ← --cluster workers.yaml
       |
  StoreToVectorDB (seq)       → Pinecone "demo-index"

  SemanticSearch (standalone)  ← --stage SemanticSearch
```

## セットアップ

### 必須

- `fav` CLI（v69.0.0 以降）
- OpenAI API キー（`OPENAI_API_KEY` 環境変数）
- Anthropic API キー（`ANTHROPIC_API_KEY` 環境変数）
- Pinecone API キー（`PINECONE_API_KEY` 環境変数）

### ローカル開発（dev モード）

`fav.toml` の `[ai.dev]` セクションで ollama-local / qdrant-local を使用:

```bash
fav run src/pipeline.fav --env dev
```

## 実行

```bash
# インデックスパイプライン実行
fav run src/pipeline.fav \
  --cluster workers.yaml \
  --checkpoint ./checkpoints/ \
  --distributed-cache redis://localhost:6379

# セマンティック検索（単体実行）
fav run src/pipeline.fav --stage SemanticSearch --input "machine learning pipelines"

# コスト見積もり
fav cost-estimate src/pipeline.fav --provider aws --scale 1M-rows

# スクリプト経由
bash scripts/run.sh
```

## ファイル構成

```
ai-etl/
├── src/
│   └── pipeline.fav     # メインパイプライン定義
├── data/
│   └── articles.csv     # サンプルデータ（10 記事）
├── workers.yaml         # ローカル 4 ワーカー設定
├── scripts/
│   └── run.sh           # 実行スクリプト
└── README.md
```
