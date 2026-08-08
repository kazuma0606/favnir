# v66.9.0 Spec — 安定化・コードフリーズ（AI Stage Layer 前調整）

Version: 66.9.0
Status: 未着手
Base tests: 3491
Target tests: 3493

---

## 概要

v66.1〜v66.8 の全機能が正常動作することを確認し、AI-Native Stage Layer 宣言（v67.0.0）に向けて
コードをフリーズする。9 つの AI Rune 群のドキュメントを整備し、安定性を確認する。

ロードマップ `roadmap-v66.1-v67.0.md` の v66.9.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3491 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（v66.0.0 宣言時に設定済み。v66.x sub-version では更新しない。v67.0.0 宣言時に `"67.0.0"` に更新する）
- `cargo test --bin fav v66800_tests` で 2 件 PASS することを確認（W055〜W059 スタブが lint.rs に登録されていることの確認）
  - これがロードマップ「W055〜W059 が正常に lint 検出できること」の確認に相当する
  - 実際の AST 走査による検出ロジックは将来フェーズ（v66.8.0 ロードマップ注記より）
- 以下の 9 AI Rune ファイルがすべて存在することを確認:
  - `runes/vec/vec.fav`（v66.1.0）
  - `runes/embed/embed.fav`（v66.3.0）
  - `runes/pinecone/pinecone.fav`（既存 + v66.4.0 で拡張）
  - `runes/pgvector/pgvector.fav`（v66.4.0）
  - `runes/weaviate/weaviate.fav`（v66.4.0）
  - `runes/qdrant/qdrant.fav`（v66.4.0）
  - `runes/inference/inference.fav`（v66.5.0）
  - `runes/serve/serve.fav`（v66.6.0）
  - `runes/featurestore/featurestore.fav`（v66.7.0）
- `site/content/docs/runes/ai-runes-overview.mdx` が存在しないことを確認（新規作成対象）
- `driver.rs` に `v66800_tests` が存在することを確認（`v66900_tests` の挿入位置）
- `driver.rs` に `v66900_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v66800_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `lint_w055_untyped_llm_output`, `lint_w056_dim_implicit_cast`
- `versions/current.md` の「進行中バージョン」が `v66.8.0` であることを確認（確認失敗時は前バージョンの tasks.md T4 が完了していることを確認してから current.md を手動修正すること）

---

## 実装スコープ

### 1. `site/content/docs/runes/ai-runes-overview.mdx` — 新規作成

> **注意**: 本バージョンは MDX 作成の例外。通常 v66.x では MDX を省略するが、
> v66.9.0 安定化フェーズにおいて AI Rune 群の概要ドキュメントを一括作成する（ロードマップ準拠）。

最低限含むべき内容:
- `Rune.embed`（テスト `ai_rune_docs_complete` が `overview.contains("Rune.embed")` をアサート）
- 9 AI Rune 群（vec / embed / pinecone / pgvector / weaviate / qdrant / inference / serve / featurestore）の概要

```mdx
# AI Runes Overview

Favnir の AI-Native Stage Layer を構成する 9 つの Rune 群の概要。

## Embedding

- **`Rune.embed`** — 統一埋め込みインターフェース（OpenAI / Cohere / ローカルモデル）
- **`Rune.vec`** — ベクトル演算（normalize / dot / cosine_similarity / euclidean_distance）

## Vector Databases

- **`Rune.pinecone`** — Pinecone ベクトルDB（upsert / query / delete / fetch / list_indexes）
- **`Rune.pgvector`** — PostgreSQL ベクトル拡張（upsert / query / create_index）
- **`Rune.weaviate`** — Weaviate ベクトルDB（upsert / query / schema_create）
- **`Rune.qdrant`** — Qdrant ベクトルDB（upsert / query / collection_create）

## Inference & Serving

- **`Rune.inference`** — ストリーミング ML 推論（inference_batch / stream_with_backpressure / stream_with_sla / stateful_score）
- **`Rune.serve`** — モデルサービングエンドポイント（serve_stage / serve_pipeline / with_rate_limit / openapi_schema）

## Feature Engineering

- **`Rune.featurestore`** — 型安全フィーチャーストア（define_feature / get / get_batch / get_version / get_at）

## Lint Rules（W055〜W059）

AI パイプライン特有のアンチパターンを静的解析で検出する:

| コード | 検出内容 |
|---|---|
| W055 | 型なし LLM 出力をそのまま下流に流す |
| W056 | 埋め込み次元の暗黙的キャスト |
| W057 | ベクトル DB への upsert なしの query |
| W058 | ストリーミング推論ステージでのバッファなし直接処理 |
| W059 | LLM 呼び出しのリトライなし |
```

### 2. `driver.rs` — `v66900_tests` 追加

挿入位置: `// -- v66800_tests (v66.8.0)` コメントの直前

```rust
// -- v66900_tests (v66.9.0) -- AI Stage Layer Stabilization --
#[cfg(test)]
mod v66900_tests {
    #[test]
    fn ai_stage_layer_all_stable() {
        let vec_fav     = include_str!("../../runes/vec/vec.fav");
        let embed       = include_str!("../../runes/embed/embed.fav");
        let pinecone    = include_str!("../../runes/pinecone/pinecone.fav");
        let pgvector    = include_str!("../../runes/pgvector/pgvector.fav");
        let weaviate    = include_str!("../../runes/weaviate/weaviate.fav");
        let qdrant      = include_str!("../../runes/qdrant/qdrant.fav");
        let inference   = include_str!("../../runes/inference/inference.fav");
        let serve       = include_str!("../../runes/serve/serve.fav");
        let featurestore = include_str!("../../runes/featurestore/featurestore.fav");
        assert!(!vec_fav.is_empty(),      "vec.fav should not be empty");
        assert!(!embed.is_empty(),        "embed.fav should not be empty");
        assert!(!pinecone.is_empty(),     "pinecone.fav should not be empty");
        assert!(!pgvector.is_empty(),     "pgvector.fav should not be empty");
        assert!(!weaviate.is_empty(),     "weaviate.fav should not be empty");
        assert!(!qdrant.is_empty(),       "qdrant.fav should not be empty");
        assert!(!inference.is_empty(),    "inference.fav should not be empty");
        assert!(!serve.is_empty(),        "serve.fav should not be empty");
        assert!(!featurestore.is_empty(), "featurestore.fav should not be empty");
    }

    #[test]
    fn ai_rune_docs_complete() {
        let overview = include_str!("../../site/content/docs/runes/ai-runes-overview.mdx");
        assert!(
            overview.contains("Rune.embed"),
            "ai-runes-overview.mdx should reference Rune.embed"
        );
    }
}
```

挿入後、`cargo build` でエラーなしを確認。

---

## 完了条件

- `site/content/docs/runes/ai-runes-overview.mdx` が存在し `"Rune.embed"` を含む
- 9 AI Rune ファイルすべてが存在し空でない
- `cargo build` でエラーなし
- `cargo test --bin fav v66900_tests` で 2 件 PASS
  - `ai_stage_layer_all_stable` PASS
  - `ai_rune_docs_complete` PASS
- `cargo test -j 8 -- --test-threads=8` で 3493 tests passed, 0 failed
- CHANGELOG.md 更新は v67.0.0 宣言時に一括追記（非スコープセクション参照）

---

## 非スコープ

- W055〜W059 の実際の検出ロジック実装 — 将来フェーズ（スタブのまま）
- `Cargo.toml` version 更新 — v67.0.0 宣言時に実施
- CHANGELOG.md 更新 — v67.0.0 宣言時に一括追記
- その他 site/ MDX ドキュメント — v66.9.0 では `ai-runes-overview.mdx` のみ作成

---

## 技術ノート

### テスト数増加の根拠

`v66900_tests` モジュール内の `#[test]` fn 2 件（`ai_stage_layer_all_stable` / `ai_rune_docs_complete`）で +2。

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../../runes/vec/vec.fav"` 〜 `"../../runes/featurestore/featurestore.fav"` — 各 Rune ファイル
- `"../../site/content/docs/runes/ai-runes-overview.mdx"` — MDX ドキュメント（`site/` は `fav/` と同じ親ディレクトリ下）

### 9 AI Rune ファイルの一覧

| Rune | ファイル | 追加バージョン |
|---|---|---|
| vec | `runes/vec/vec.fav` | v66.1.0 |
| embed | `runes/embed/embed.fav` | v66.3.0 |
| pinecone | `runes/pinecone/pinecone.fav` | 既存（v66.4.0 で list_indexes 追加） |
| pgvector | `runes/pgvector/pgvector.fav` | v66.4.0 |
| weaviate | `runes/weaviate/weaviate.fav` | v66.4.0 |
| qdrant | `runes/qdrant/qdrant.fav` | v66.4.0 |
| inference | `runes/inference/inference.fav` | v66.5.0 |
| serve | `runes/serve/serve.fav` | v66.6.0 |
| featurestore | `runes/featurestore/featurestore.fav` | v66.7.0 |
