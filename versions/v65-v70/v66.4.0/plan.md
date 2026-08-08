# v66.4.0 実装計画 — Vector DB Runes（Pinecone / pgvector / Weaviate / Qdrant）

Version: 66.4.0
Status: 未着手
Base tests: 3481
Target tests: 3483

---

## 実装ステップ

### Step 1: 既存 pinecone.fav に `list_indexes` を追加

`runes/pinecone/pinecone.fav` の末尾に `fn Pinecone.list_indexes()` を追加。
既存の `fn Pinecone.upsert` / `query` / `delete` / `fetch` 構文と統一（`public fn` は使わない）。

### Step 2: 新規 Rune ファイル作成

1. `runes/pgvector/rune.toml` + `runes/pgvector/pgvector.fav`（upsert / query / create_index、VectorDBInterface コメント付き）
2. `runes/weaviate/rune.toml` + `runes/weaviate/weaviate.fav`（upsert / query / schema_create）
3. `runes/qdrant/rune.toml` + `runes/qdrant/qdrant.fav`（upsert / query / collection_create）

### Step 3: `driver.rs` テスト追加

- `// -- v66300_tests (v66.3.0)` コメントの直前に `v66400_tests` を挿入
- 2 テスト関数:
  - `vector_db_upsert_query`（pinecone.fav + pgvector.fav の検証）
  - `vector_db_type_safe_dim`（weaviate.fav + qdrant.fav の検証）

### Step 4: ビルド・テスト確認

```bash
# 以下は順番に実行すること（前コマンドが PASS してから次へ進む）
cargo build
cargo test --bin fav v66400_tests
cargo test -j 8 -- --test-threads=8
```

---

## 関数一覧

| Rune | 関数 | 戻り値 | 備考 |
|---|---|---|---|
| pinecone | `list_indexes` | `Result<List<String>, String>` | 既存ファイルへの追記 |
| pgvector | `upsert(table, id, vector, metadata)` | `""` | 新規スタブ |
| pgvector | `query(table, vector, top_k)` | `[]` | 新規スタブ |
| pgvector | `create_index(table, index_type)` | `""` | 新規スタブ |
| weaviate | `upsert(class_name, id, vector, properties)` | `""` | 新規スタブ |
| weaviate | `query(class_name, vector, top_k)` | `[]` | 新規スタブ |
| weaviate | `schema_create(class_name, description)` | `""` | 新規スタブ |
| qdrant | `upsert(collection, id, vector, payload)` | `""` | 新規スタブ |
| qdrant | `query(collection, vector, top_k)` | `[]` | 新規スタブ |
| qdrant | `collection_create(collection, vector_size)` | `""` | 新規スタブ |

---

## `driver.rs` 挿入コード

```rust
// -- v66400_tests (v66.4.0) -- Vector DB Runes --
#[cfg(test)]
mod v66400_tests {
    #[test]
    fn vector_db_upsert_query() {
        let pinecone = include_str!("../../runes/pinecone/pinecone.fav");
        let pgvector = include_str!("../../runes/pgvector/pgvector.fav");
        assert!(pinecone.contains("upsert"), "pinecone.fav should define upsert");
        assert!(pinecone.contains("query"), "pinecone.fav should define query");
        assert!(pinecone.contains("list_indexes"), "pinecone.fav should define list_indexes");
        assert!(pgvector.contains("fn upsert("), "pgvector.fav should define upsert");
        assert!(pgvector.contains("fn query("), "pgvector.fav should define query");
        assert!(
            pgvector.contains("VectorDBInterface"),
            "pgvector.fav should reference VectorDBInterface"
        );
    }

    #[test]
    fn vector_db_type_safe_dim() {
        let weaviate = include_str!("../../runes/weaviate/weaviate.fav");
        let qdrant = include_str!("../../runes/qdrant/qdrant.fav");
        assert!(weaviate.contains("fn upsert("), "weaviate.fav should define upsert");
        assert!(weaviate.contains("fn query("), "weaviate.fav should define query");
        assert!(weaviate.contains("fn schema_create("), "weaviate.fav should define schema_create");
        assert!(qdrant.contains("fn upsert("), "qdrant.fav should define upsert");
        assert!(qdrant.contains("fn query("), "qdrant.fav should define query");
        assert!(
            qdrant.contains("fn collection_create("),
            "qdrant.fav should define collection_create"
        );
    }
}
```

---

## リスク・注意点

- `pinecone.fav` は `fn Pinecone.upsert(...)` 形式（`public fn` なし）。追加する `list_indexes` も同形式で統一
- `pinecone.contains("upsert")` は `fn Pinecone.upsert(` にマッチ（既存コードを変更せず利用）
- `pgvector.contains("VectorDBInterface")` はコメント行にのみ存在するため、コメントを変更した場合はテストも連動更新が必要
- 各新規 Rune は `public fn` 形式でスタブを統一（既存 pinecone と差別化）
