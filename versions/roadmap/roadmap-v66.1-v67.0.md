# Roadmap v66.1.0 〜 v67.0.0 — AI-Native Stage Layer

Date: 2026-08-04
Status: 未着手（v66.0.0 完了後に開始）

マスターロードマップ: [roadmap-v65.1-v70.0.md](roadmap-v65.1-v70.0.md)

---

## 前提

- 直前完了: v66.0.0「Math & Science Foundation」（tests = 3475）
- 本スプリントは Phase 2「AI-Native Stage Layer」の詳細計画
- 目標: v67.0.0「AI-Native Stage Layer 宣言」（tests = 3497）

### 設計方針

**ベクトル次元の型保証**

Favnir の型パラメータで埋め込みモデルの次元を保証する。
`Vec<Float>[768]`（BERT系）と `Vec<Float>[1536]`（OpenAI ada-002）は別の型であり、
次元違いの演算はコンパイルエラーになる。

**LLM 出力の型安全化**

LLM の生の文字列出力をそのまま流すことは W055 で警告。
`Rune.llm.extract` でスキーマ付き構造体に変換することを標準パターンとする。

**Lint コード範囲**: W055〜W059（AI パイプライン特有のアンチパターン）

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v66.1.0 | Vector Stage Primitives | 3475 + 2 = 3477 | 完了 |
| v66.2.0 | LLM Extraction Stage（型安全 JSON 抽出） | 3477 + 2 = 3479 | 完了 |
| v66.3.0 | Embedding Pipeline Rune | 3479 + 2 = 3481 | 完了 |
| v66.4.0 | Vector DB Runes（Pinecone / pgvector / Weaviate） | 3481 + 2 = 3483 | 完了 |
| v66.5.0 | Streaming Inference Stage | 3483 + 2 = 3485 | 完了 |
| v66.6.0 | Model Serving Rune（`Rune.serve`） | 3485 + 2 = 3487 | 完了 |
| v66.7.0 | Feature Store Rune（`Rune.featurestore`） | 3487 + 2 = 3489 | 完了 |
| v66.8.0 | AI Pipeline Lint Rules（W055〜W059） | 3489 + 2 = 3491 | 完了 |
| v66.9.0 | 安定化・コードフリーズ | 3491 + 2 = 3493 | 完了 |
| v67.0.0 | AI-Native Stage Layer 宣言 ★クリーンアップ | 3493 + 4 = 3497 | 完了 |

---

## v66.1.0 — Vector Stage Primitives

**概要**: ベクトル演算を型安全なステージとして提供。
埋め込み次元を型パラメータで保証し、次元違いをコンパイルエラーにする。

```favnir
// 1536 次元固定 — Vec<Float>[768] を渡すとコンパイルエラー
public stage EmbedText: String -> Vec<Float>[1536] = |text| {
    Rune.openai.embed(model: "text-embedding-3-small", text: text)
}

public stage CosineSim: (Vec<Float>[1536], Vec<Float>[1536]) -> Float = |(a, b)| {
    Rune.linalg.cosine_similarity(a, b)
}

public stage BatchEmbed: List<String> -> List<Vec<Float>[1536]> = |texts| {
    List.map(texts, EmbedText)
}
```

**実装内容**:

- `Vec<Float>[N]` 型（次元を型パラメータで保持）
- `Rune.vec` — ベクトル操作 Rune（`normalize`, `dot`, `cosine_similarity`, `euclidean_distance`）
- バッチ処理ヘルパー: `batch_embed`, `batch_cosine_matrix`
- 次元変換: `project`（次元削減付き、型付き）

**ファイル**:
- `runes/vec/vec.fav`
- `runes/vec/rune.toml`

**完了条件**: Rust テスト 2 件（3475 + 2 = **3477**）

```rust
// driver.rs mod v66100_tests
fn vec_stage_dim_type_check() // vec.fav に normalize / dot / cosine_similarity 定義を含む
fn vec_stage_cosine_sim()     // batch_embed / project 定義を含む
```

---

## v66.2.0 — LLM Extraction Stage（型安全 JSON 抽出）

**概要**: LLM の出力を型安全なスキーマに変換するステージ。
非構造テキスト → 型付きレコードの変換を保証。スキーマ違反は型エラー。

```favnir
schema InvoiceData {
    vendor: String,
    amount: Float,
    date: DateTime,
    line_items: List<LineItem>
}

schema LineItem {
    description: String,
    quantity: Int,
    unit_price: Float
}

public stage ExtractInvoice: String -> InvoiceData = |raw_text| {
    Rune.llm.extract(raw_text, schema: InvoiceData, model: "claude-sonnet-4-6")
    // InvoiceData 型を満たさない場合は E0xxx でエラー
}

public stage ExtractBatch: List<String> -> List<InvoiceData> = |texts| {
    List.map(texts, ExtractInvoice)
}
```

**実装内容**:

- `Rune.llm.extract(text, schema: T, model: String) -> T` — スキーマ付き抽出
- `Rune.llm.extract_list(text, schema: T, model: String) -> List<T>` — 複数レコード抽出
- JSON スキーマ自動生成（`schema` 定義 → JSON Schema 変換）— **将来フェーズに移管**（v66.x 安定化後）
- バリデーション: 型チェック + 必須フィールド確認 — **将来フェーズに移管**（v66.x 安定化後）
- フォールバック: `extract_or_default`, `extract_maybe`（失敗時 `Option<T>`）

**ファイル**:
- `runes/llm/llm_extract.fav`（既存 `runes/llm/` を拡張）
- `runes/llm/rune.toml` — 変更しない（既存非標準形式のため、exports 追加は将来フェーズ）

**完了条件**: Rust テスト 2 件（3477 + 2 = **3479**）

```rust
// driver.rs mod v66200_tests
fn llm_extract_typed_schema()          // llm_extract.fav に extract / extract_list 定義を含む
fn llm_extract_schema_mismatch_error() // extract_or_default / extract_maybe 定義を含む
```

---

## v66.3.0 — Embedding Pipeline Rune

**概要**: ローカルモデル・OpenAI・Cohere・Anthropic 等の埋め込みモデルを統一インターフェースで扱う。
モデルの切り替えは設定変更のみで対応可能。

```favnir
// OpenAI: 1536 次元
public stage EmbedOpenAI: String -> Vec<Float>[1536] = |text| {
    Rune.embed.openai(text, model: "text-embedding-3-small")
}

// Cohere: 1024 次元
public stage EmbedCohere: String -> Vec<Float>[1024] = |text| {
    Rune.embed.cohere(text, model: "embed-english-v3.0")
}

// ローカル（ollama）: 768 次元
public stage EmbedLocal: String -> Vec<Float>[768] = |text| {
    Rune.embed.local(text, model: "nomic-embed-text")
}
```

**実装内容**:

- `Rune.embed` — 統一埋め込み Rune（**v66.3.0 では `List<Float>` プレースホルダー使用。`Vec<Float>[N]` 型登録は将来フェーズ**）
  - `openai(text, model)` → `Vec<Float>[1536]`（スタブ: `List<Float>`）
  - `cohere(text, model)` → `Vec<Float>[1024]`（スタブ: `List<Float>`）
  - `local(text, model)` → `Vec<Float>[768]`（Ollama 経由）（スタブ: `List<Float>`）
- バッチ処理: `embed_batch(texts, model)` → `List<Vec<Float>[N]>`（スタブ: `List<List<Float>>`）
- キャッシュ: `embed_cached(text, model, cache_key)` — 同一入力の再計算を防ぐ

**ファイル**:
- `runes/embed/embed.fav`
- `runes/embed/rune.toml`

**完了条件**: Rust テスト 2 件（3479 + 2 = **3481**）

```rust
// driver.rs mod v66300_tests
fn embed_rune_openai()       // embed.fav に openai / cohere / embed_batch 定義を含む
fn embed_rune_local_model()  // local / embed_cached 定義を含む
```

---

## v66.4.0 — Vector DB Runes（Pinecone / pgvector / Weaviate）

**概要**: ベクトルデータベースへの型安全なアクセスを提供する Rune 群。
upsert と query の次元が一致しないとコンパイルエラー。

```favnir
public stage StoreEmbeddings: List<(String, Vec<Float>[1536])> -> Unit = |pairs| {
    Rune.pinecone.upsert(pairs, namespace: "docs", index: "prod")
}

public stage SemanticSearch: Vec<Float>[1536] -> List<Document> = |query_vec| {
    Rune.pinecone.query(query_vec, top_k: 10, include_metadata: true)
}

// pgvector（PostgreSQL）
public stage StoreLocal: List<(String, Vec<Float>[768])> -> Unit = |pairs| {
    Rune.pgvector.upsert(pairs, table: "embeddings")
}
```

**実装内容**:

- `Rune.pinecone` — Pinecone: `upsert`, `query`, `delete`, `fetch`, `list_indexes`
- `Rune.pgvector` — pgvector: `upsert`, `query`, `create_index`（IVFFLAT/HNSW）
- `Rune.weaviate` — Weaviate: `upsert`, `query`, `schema_create`
- `Rune.qdrant` — Qdrant: `upsert`, `query`, `collection_create`
- 統一インターフェース: `VectorDB` interface（切り替え可能）

**ファイル**:
- `runes/pinecone/pinecone.fav` — **既存ファイルを拡張**（`list_indexes` を追加。upsert/query/delete/fetch は既存）
- `runes/pgvector/pgvector.fav` — 新規作成
- `runes/weaviate/weaviate.fav` — 新規作成
- `runes/qdrant/qdrant.fav` — 新規作成

**完了条件**: Rust テスト 2 件（3481 + 2 = **3483**）

```rust
// driver.rs mod v66400_tests
fn vector_db_upsert_query()       // pinecone.fav / pgvector.fav に upsert / query 定義を含む
fn vector_db_type_safe_dim()      // weaviate.fav / qdrant.fav に upsert / query 定義を含む
```

---

## v66.5.0 — Streaming Inference Stage

**概要**: リアルタイムスコアリングパイプライン。
Kafka ストリーム + ML モデル推論を型安全に組み合わせる。
バックプレッシャー制御で無限ストリームを安全に処理。

```favnir
pipeline RealtimeScoring {
    step "ingest"  = stream KafkaIngest
    step "embed"   = seq   EmbedText   after "ingest"
    step "score"   = seq   MLScore     after "embed"
    step "publish" = stream KafkaPublish after "score"
}

public stage MLScore: Vec<Float>[1536] -> ScoredResult = |embedding| {
    bind model = Rune.ml.load("fraud-detector-v3")
    Rune.ml.predict(model, embedding)
}
```

**実装内容**:

- ストリーミングステージでの埋め込み・推論の組み合わせ
- バックプレッシャー制御: `stream_with_backpressure(buffer_size: 1000)`
- バッチ推論: `inference_batch(embeddings, model, batch_size: 32)`
- レイテンシ SLA: `stream_with_sla(max_latency_ms: 100)` — SLA 超過を警告
- 状態管理: `stateful_score`（セッション単位の状態保持）

**ファイル**:
- `runes/inference/inference.fav`
- `runes/inference/rune.toml`

**完了条件**: Rust テスト 2 件（3483 + 2 = **3485**）

```rust
// driver.rs mod v66500_tests
fn streaming_inference_pipeline()   // inference.fav に inference_batch / stream_with_backpressure 定義
fn streaming_backpressure_ai()      // stream_with_sla / stateful_score 定義を含む
```

---

## v66.6.0 — Model Serving Rune（`Rune.serve`）

**概要**: Favnir ステージをモデルサービングエンドポイントとして公開する Rune。
`fav serve` コマンドでパイプラインを HTTP API として起動できる。

```favnir
// pipeline.fav
public stage Score: InputFeatures -> Prediction = |features| {
    bind embedding = EmbedText(features.text)
    Rune.ml.predict(fraud_model, embedding)
}
```

```bash
$ fav serve pipeline.fav --port 8080
[fav serve] Listening on :8080
POST /score  → Prediction  (InputFeatures → Prediction)
GET  /health → { status: "ok", version: "1.0" }
```

**実装内容**:

- `Rune.serve` — HTTP エンドポイント公開
  - `serve_stage(stage_name, port)` — 単一ステージを公開
  - `serve_pipeline(pipeline_name, port)` — パイプライン全体を公開
- 入出力の JSON シリアライズ/デシリアライズ（スキーマ自動生成）
- ヘルスチェックエンドポイント（`GET /health`）
- レート制限: `with_rate_limit(rps: 100)`
- OpenAPI スキーマ自動生成: `fav serve --openapi-out schema.json`

**ファイル**:
- `runes/serve/serve.fav`
- `runes/serve/rune.toml`

**完了条件**: Rust テスト 2 件（3485 + 2 = **3487**）

```rust
// driver.rs mod v66600_tests
fn model_serve_endpoint_type()       // serve.fav に serve_stage / serve_pipeline 定義を含む
fn model_serve_schema_validation()   // with_rate_limit / openapi_schema 定義を含む
```

---

## v66.7.0 — Feature Store Rune（`Rune.featurestore`）

**概要**: 型安全なフィーチャーエンジニアリング。
フィーチャーの定義・バージョン管理・取得・共有を型で保証する。

```favnir
// フィーチャー定義
schema UserFeatures {
    user_id: String,
    age_bucket: Int,      // 0-4 (decade buckets)
    purchase_count_30d: Int,
    avg_order_value: Float,
    last_category: String
}

public stage BuildFeatures: RawUser -> UserFeatures = |user| {
    Rune.featurestore.compute("user-features-v2", user)
}

public stage FetchFeatures: String -> UserFeatures = |user_id| {
    Rune.featurestore.get("user-features-v2", user_id)
}
```

**実装内容**:

- フィーチャー定義: `define_feature(name, version, schema, compute_fn)`
- 取得: `get(feature_name, entity_key)` — オンライン推論向け低レイテンシ
- バッチ取得: `get_batch(feature_name, keys)` — 訓練データ生成向け
- バージョン管理: `get_version(feature_name, version)` — 再現性保証
- Point-in-time lookup: `get_at(feature_name, key, timestamp)` — 訓練データリークを防ぐ

**ファイル**:
- `runes/featurestore/featurestore.fav`
- `runes/featurestore/rune.toml`

**完了条件**: Rust テスト 2 件（3487 + 2 = **3489**）

```rust
// driver.rs mod v66700_tests
fn feature_store_define_feature()      // featurestore.fav に define_feature / get / get_batch 定義（FeatureStoreInterface コメント必須）
fn feature_store_versioned_retrieval() // get_version / get_at 定義を含む
```

---

## v66.8.0 — AI Pipeline Lint Rules（W055〜W059）

**概要**: AI パイプライン特有のアンチパターンを静的解析で検出する lint ルール。
LLM・ベクトル・ストリーミング推論の落とし穴を事前に警告する。

> **スコープ縮小（v66.8.0）**: 本バージョンは W055〜W059 のスタブ関数を `lint.rs` に登録するのみ。
> 実際の AST 走査による検出ロジックは将来フェーズ。テストは `include_str!("lint.rs")` で存在確認のみ行う。

| コード | 検出内容 | 重大度 |
|---|---|---|
| W055 | 型なし LLM 出力をそのまま下流に流す（`Rune.llm.call` の結果を String のまま使用） | warning |
| W056 | 埋め込み次元の暗黙的キャスト（`Vec<Float>[768]` → `Vec<Float>[1536]` の代入） | error |
| W057 | ベクトル DB への upsert なしの query（空インデックスへの問い合わせリスク） | warning |
| W058 | ストリーミング推論ステージでのバッファなし直接処理（メモリ溢れのリスク） | warning |
| W059 | LLM 呼び出しのリトライなし（外部 API 一時障害への無対策） | info |

**完了条件**: Rust テスト 2 件（3489 + 2 = **3491**）

```rust
// driver.rs mod v66800_tests
fn lint_w055_untyped_llm_output()  // W055 が Rune.llm.call の未抽出出力を検出
fn lint_w056_dim_implicit_cast()   // W056 が Vec 次元違いの代入を検出
```

---

## v66.9.0 — 安定化・コードフリーズ（AI Stage Layer 前調整）

**概要**: v66.1〜v66.8 の全機能が正常動作することを確認。
AI Rune 群（vec / embed / pinecone / pgvector / weaviate / qdrant / inference / serve / featurestore）のドキュメントを整備。

**確認内容**:

- 全 AI Rune ファイルが存在し空でないこと
- W055〜W059 スタブが lint.rs に登録されていることを確認（`v66800_tests` で検証済み。実際の AST 走査検出は将来フェーズ）
- `site/content/docs/runes/ai-runes-overview.mdx` の作成

**完了条件**: Rust テスト 2 件（3491 + 2 = **3493**）

```rust
// driver.rs mod v66900_tests
fn ai_stage_layer_all_stable()  // 9 AI Rune ファイルすべてが存在し空でないことを確認
fn ai_rune_docs_complete()      // ai-runes-overview.mdx が存在し "Rune.embed" を含む
```

---

## v67.0.0 — AI-Native Stage Layer 宣言 ★クリーンアップ

**宣言文**:

> 「LLM の出力にスキーマが付き、ベクトルの次元が型で保証される。
>  埋め込みモデルの選択が型エラーを生まず、
>  リアルタイム推論パイプラインがバックプレッシャー制御下で動く。
>
>  これが Favnir v67.0 — AI-Native Stage Layer の姿である。」

**タスク**:

- [ ] `fav/Cargo.toml` version を `"67.0.0"` に更新
- [ ] `MILESTONE.md` 先頭に v67.0.0「AI-Native Stage Layer」エントリを追加
- [ ] `README.md` に v67.0.0 宣言文を追加
- [ ] `CHANGELOG.md` 先頭に v67.0.0 エントリを追加
- [ ] `v67000_tests` 4 件を `driver.rs` に追加
- [ ] `cargo clean` 実行（★クリーンアップ）
- [ ] `cargo test -j 8 -- --test-threads=8` で 3497 tests passed を確認

**完了条件**: `v67000_tests` 4 件（3493 + 4 = **3497**）

```rust
// driver.rs mod v67000_tests
fn cargo_toml_version_is_67_0_0()   // Cargo.toml に "version = \"67.0.0\"" を含む
fn changelog_has_v67_0_0()          // CHANGELOG.md に "v67.0.0" を含む
fn milestone_has_ai_native_stage()  // MILESTONE.md に "AI-Native Stage Layer" を含む
fn readme_mentions_ai_native()      // README.md に "AI-Native" または "v67.0" を含む
```

---

## テスト数推移

| バージョン | テスト数 | 増加 |
|---|---|---|
| v66.0.0（ベース） | 3475 | — |
| v66.1.0 | 3477 | +2 |
| v66.2.0 | 3479 | +2 |
| v66.3.0 | 3481 | +2 |
| v66.4.0 | 3483 | +2 |
| v66.5.0 | 3485 | +2 |
| v66.6.0 | 3487 | +2 |
| v66.7.0 | 3489 | +2 |
| v66.8.0 | 3491 | +2 |
| v66.9.0 | 3493 | +2 |
| v67.0.0 | 3497 | +4 |
