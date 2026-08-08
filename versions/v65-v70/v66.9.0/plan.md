# v66.9.0 実装計画 — 安定化・コードフリーズ

Version: 66.9.0
Status: 未着手
Base tests: 3491
Target tests: 3493

---

## 実装ステップ

> **前提**: spec.md の T0 前提確認を完了してから開始する。

### Step 1: `site/content/docs/runes/ai-runes-overview.mdx` 作成

`site/content/docs/runes/` ディレクトリに `ai-runes-overview.mdx` を新規作成する。
必須要件:
- `"Rune.embed"` を含む（`ai_rune_docs_complete` テストがアサート）
- 9 AI Rune 群（vec / embed / pinecone / pgvector / weaviate / qdrant / inference / serve / featurestore）の概要を記述

### Step 2: `driver.rs` テスト追加

- `// -- v66800_tests (v66.8.0)` コメントの直前に `v66900_tests` を挿入
- 2 テスト関数:
  - `ai_stage_layer_all_stable`（9 AI Rune ファイルすべて存在・空でないことを確認）
  - `ai_rune_docs_complete`（ai-runes-overview.mdx が存在し "Rune.embed" を含む）

### Step 3: ビルド・テスト確認

```bash
# 以下は順番に実行すること（前コマンドが PASS してから次へ進む）
cargo build
cargo test --bin fav v66900_tests
cargo test -j 8 -- --test-threads=8
```

---

## `driver.rs` 挿入コード

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

---

## リスク・注意点

- `site/content/docs/runes/` ディレクトリが存在するか事前に確認すること（存在しない場合は作成が必要）
- `include_str!("../../site/content/docs/runes/ai-runes-overview.mdx")` のパスは `fav/src/driver.rs` 起点。`site/` は `fav/` と同じ親ディレクトリ下にある
- `ai_stage_layer_all_stable` は `!is_empty()` チェックのみ（内容の詳細検証は行わない）
- MDX ファイルの先頭に `import` 文を置くと acorn パースエラーになる場合がある（過去に発生。コードブロックや JSX は使わず通常の MDX 構文で記述する）

## 非スコープ

- W055〜W059 の実際の検出ロジック実装 — 将来フェーズ
- `Cargo.toml` version 更新 — v67.0.0 宣言時
- CHANGELOG.md 更新 — v67.0.0 宣言時に一括追記
- その他 site/ MDX ドキュメント — 本バージョンは `ai-runes-overview.mdx` のみ
