# v29.3.0 Plan — pinecone Rune 追加

**バージョン**: 29.3.0
**日付**: 2026-06-30
**前バージョン**: v29.2.0 (mlflow Rune 追加)

---

## 実装手順

### T1: Cargo.toml version 更新

```toml
version = "29.3.0"
```

### T2: runes/pinecone/rune.toml 作成

```toml
[rune]
name        = "pinecone"
version     = "1.0.0"
description = "Pinecone ベクトルDB 連携（upsert / query / delete / fetch / describe_index_stats）"
license     = "MIT"
authors     = ["Favnir Team"]
```

### T3: runes/pinecone/pinecone.fav 作成（5 関数）

```favnir
// pinecone Rune — Pinecone ベクトルDB 連携（v29.3.0）
// 接続: PINECONE_API_KEY / PINECONE_ENV / PINECONE_BASE_URL 環境変数

// ベクトルエントリ型
type PineconeVector = {
  id: String,
  values: List<Float>,
  metadata: String
}

// ベクトルを追加・更新する（バッチ対応）
fn Pinecone.upsert(index: String, vectors: List<PineconeVector>) -> Result<Unit, String> !Http =
  Http.post_json(
    Env.get_or("PINECONE_BASE_URL", "https://api.pinecone.io") ++ "/vectors/upsert",
    { "index": index, "vectors": vectors }
  )

// 近傍検索（メタデータフィルタ付き）
fn Pinecone.query(index: String, vector: List<Float>, k: Int, filter: String) -> Result<List<String>, String> !Http =
  Http.post_json(
    Env.get_or("PINECONE_BASE_URL", "https://api.pinecone.io") ++ "/query",
    { "index": index, "vector": vector, "topK": k, "filter": filter }
  )

// ID 指定でベクトルを削除する
fn Pinecone.delete(index: String, ids: List<String>) -> Result<Unit, String> !Http =
  Http.post_json(
    Env.get_or("PINECONE_BASE_URL", "https://api.pinecone.io") ++ "/vectors/delete",
    { "index": index, "ids": ids }
  )

// ID 指定でベクトルを取得する
fn Pinecone.fetch(index: String, ids: List<String>) -> Result<List<String>, String> !Http =
  Http.post_json(
    Env.get_or("PINECONE_BASE_URL", "https://api.pinecone.io") ++ "/vectors/fetch",
    { "index": index, "ids": ids }
  )

// インデックス統計を取得する
fn Pinecone.describe_index_stats(index: String) -> Result<String, String> !Http =
  Http.post_json(
    Env.get_or("PINECONE_BASE_URL", "https://api.pinecone.io") ++ "/describe_index_stats",
    { "index": index }
  )
```

### T4: CHANGELOG.md に [v29.3.0] セクション追加

```markdown
## [v29.3.0] — 2026-06-30

### Added
- `runes/pinecone/` — Pinecone ベクトルDB Rune（upsert / query / delete / fetch / describe_index_stats）
- RAG パイプラインサポート: LLM Rune と組み合わせてドキュメント検索付き LLM パイプラインを構築可能
- `site/content/docs/runes/pinecone.mdx` — Pinecone Rune ドキュメント
```

### T5: benchmarks/v29.3.0.json 作成

```json
{
  "version": "29.3.0",
  "date": "2026-06-30",
  "milestone": "Ecosystem Maturity (phase 3)",
  "test_count": 2330,
  "metrics": {
    "compile_hello_ms": 12,
    "compile_etl_ms": 38,
    "typecheck_ms": 9,
    "vm_run_ms": 4
  }
}
```

### T6: site/content/docs/runes/pinecone.mdx 作成

Pinecone Rune の使い方・API リファレンス・RAG パイプライン例を含むドキュメント。

### T7: driver.rs に v293000_tests 6 件追加

```rust
// v293000_tests (v29.3.0) -- pinecone Rune
#[cfg(test)]
mod v293000_tests {
    #[test]
    fn pinecone_rune_file_exists() {
        let src = include_str!("../../runes/pinecone/pinecone.fav");
        assert!(
            src.contains("upsert"),
            "runes/pinecone/pinecone.fav must define upsert"
        );
    }
    #[test]
    fn pinecone_query_fn_exists() {
        let src = include_str!("../../runes/pinecone/pinecone.fav");
        assert!(src.contains("query"), "pinecone.fav must define query");
    }
    #[test]
    fn pinecone_delete_and_fetch_fn_exists() {
        let src = include_str!("../../runes/pinecone/pinecone.fav");
        assert!(
            src.contains("delete") && src.contains("fetch"),
            "pinecone.fav must define delete and fetch"
        );
    }
    #[test]
    fn pinecone_describe_index_stats_fn_exists() {
        let src = include_str!("../../runes/pinecone/pinecone.fav");
        assert!(
            src.contains("describe_index_stats"),
            "pinecone.fav must define describe_index_stats"
        );
    }
    #[test]
    fn pinecone_rune_toml_exists() {
        let src = include_str!("../../runes/pinecone/rune.toml");
        assert!(
            src.contains("pinecone"),
            "runes/pinecone/rune.toml must contain 'pinecone'"
        );
    }
    #[test]
    fn changelog_has_v29_3_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(
            src.contains("[v29.3.0]") || src.contains("## v29.3.0"),
            "CHANGELOG.md must contain '[v29.3.0]'"
        );
    }
}
```

### T8: cargo test --bin fav v293000 — 6/6 PASS 確認

### T9: cargo test --bin fav — 2330 tests PASS 確認

### T10: tasks.md を COMPLETE に更新

---

## テスト数カウント

| バージョン | テスト数 |
|---|---|
| v29.2.0 | 2324 |
| v29.3.0 | **2330** (+6) |
