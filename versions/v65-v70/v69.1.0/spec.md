# v69.1.0 仕様書 — E2E デモ（CSV → Embed → VectorDB → Semantic Search）

Status: DRAFT
Version: 69.1.0
Date: 2026-08-07

---

## 概要

v65〜v69 の全機能を使った完全な AI ETL デモパイプライン。
「実際に動く AI パイプライン」として公開し、採用検討者が即試せる形にする。

パイプライン: 記事 CSV を読み込み → 埋め込みベクトル生成 → Pinecone に保存 → セマンティック検索

---

## スコープ

### IN（本バージョンで実施）

- `infra/e2e-demo/ai-etl/` ディレクトリ作成（以下ファイル）:
  - `src/pipeline.fav` — 4 ステージのデモパイプライン（LoadArticles / EmbedAndSummarize / StoreToVectorDB / SemanticSearch + pipeline `IndexPipeline`）
  - `data/articles.csv` — サンプルデータ（100 記事）
  - `workers.yaml` — ローカル 4 ワーカー設定
  - `README.md` — セットアップ手順
  - `scripts/run.sh` — 実行スクリプト
- `driver.rs` に `v69100_tests` 2 件を追加（`v69000_tests` の直前）

### OUT（本バージョンでは実施しない）

- 実際の API 呼び出し（Pinecone / OpenAI / Claude）: スタブのまま
- `fav.toml` の `[ai]` セクション設定（プロバイダー設定）: pipeline.fav 内のコメントとして言及するが、`fav.toml` ファイルの作成は v69.x 後期または v70.0.0 で実施
- CI 自動実行: 将来フェーズ
- WASM Playground 更新: v69.2.0
- ドキュメントサイト更新: v69.3.0

---

## 成果物仕様

### `infra/e2e-demo/ai-etl/src/pipeline.fav`

以下のキーワードを含むこと（テスト要件）:
- `"IndexPipeline"` — パイプライン名
- `"LoadArticles"` — Stage 1
- `"EmbedAndSummarize"` — Stage 2
- `"StoreToVectorDB"` — Stage 3
- `"SemanticSearch"` — Stage 4

ロードマップのサンプルコードに準拠:
- `schema Article` / `schema IndexedArticle` を定義
- `pipeline IndexPipeline` で 4 ステージを定義
- `par` キーワードで EmbedAndSummarize を並列実行
- `SemanticSearch` は `pipeline IndexPipeline` の step には含まれず、スタンドアロンステージとして定義する（独立クエリ用途。`fav run --stage SemanticSearch` で単体実行可能なデモ構成）

### `infra/e2e-demo/ai-etl/data/articles.csv`

ヘッダ: `id,title,body,tags` の CSV（10 行以上）

### `infra/e2e-demo/ai-etl/workers.yaml`

```yaml
workers:
  - host: localhost
    port: 9001
    cores: 2
  - host: localhost
    port: 9002
    cores: 2
  - host: localhost
    port: 9003
    cores: 2
  - host: localhost
    port: 9004
    cores: 2
```

### `infra/e2e-demo/ai-etl/README.md`

セットアップ手順を記載。

### `infra/e2e-demo/ai-etl/scripts/run.sh`

実行スクリプト（`fav run` コマンドを含む）。

---

## テスト仕様

### `v69100_tests`（2 件、3541 + 2 = **3543**）

```rust
fn ai_etl_e2e_demo_structure()
// include_str!("../../infra/e2e-demo/ai-etl/src/pipeline.fav") で読み込み
// "IndexPipeline" を assert!

fn ai_etl_demo_has_all_stages()
// include_str!("../../infra/e2e-demo/ai-etl/src/pipeline.fav") で読み込み
// "LoadArticles" / "EmbedAndSummarize" / "StoreToVectorDB" / "SemanticSearch" を個別 assert!
```

---

## 完了条件

- `cargo test --bin fav v69100_tests` で 2 件 PASS
- `cargo test -j 8 -- --test-threads=8` で **3543 tests passed, 0 failed**
- `versions/roadmap/roadmap-v69.1-v70.0.md` の v69.1.0 行の状態が「完了」になっていること
- `versions/current.md` の「進行中バージョン」が `v69.1.0` に更新されていること
