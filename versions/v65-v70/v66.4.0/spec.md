# v66.4.0 Spec — Vector DB Runes（Pinecone / pgvector / Weaviate / Qdrant）

Version: 66.4.0
Status: 未着手
Base tests: 3481
Target tests: 3483

---

## 概要

ベクトルデータベースへの型安全なアクセスを提供する Rune 群を実装する。
Pinecone（既存拡張）・pgvector・Weaviate・Qdrant の 4 種類をサポートする。
`VectorDB` 統一インターフェースにより切り替え可能な設計とする。

ロードマップ `roadmap-v66.1-v67.0.md` の v66.4.0 セクションに準拠。

> **スコープ縮小の明示**: ロードマップの利用例では `Vec<Float>[1536]` 等の次元型パラメータを使用しているが、
> 型システムへの登録は将来フェーズ。本バージョンでは `List<Float>` をプレースホルダーとして
> 関数シグネチャを確立することに専念する。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3481 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（sub-version では Cargo.toml は更新しない）
- `runes/pinecone/pinecone.fav` が存在し `upsert` / `query` / `delete` / `fetch` を含むことを確認（既存）
- `runes/pgvector/` ディレクトリが存在しないことを確認（新規作成対象）
- `runes/weaviate/` ディレクトリが存在しないことを確認（新規作成対象）
- `runes/qdrant/` ディレクトリが存在しないことを確認（新規作成対象）
- `driver.rs` に `v66300_tests` が存在することを確認（`v66400_tests` の挿入位置）
- `driver.rs` に `v66400_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v66300_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `embed_rune_openai`, `embed_rune_local_model`
- `versions/current.md` の「進行中バージョン」が `v66.3.0` であることを確認

---

## 実装スコープ

### 1. `runes/pinecone/pinecone.fav` — 既存ファイルに `list_indexes` を追加

既存の `pinecone.fav` に以下の関数を末尾追加する:

```favnir
// インデックス一覧を取得する（list_indexes）
// NOTE: Pinecone /indexes は GET だが Http.get_with_body 実装まで post_json で代替
fn Pinecone.list_indexes() -> Result<List<String>, String> =
  Http.post_json(
    Env.get_or("PINECONE_BASE_URL", "https://api.pinecone.io") ++ "/indexes",
    {}
  )
```

### 2. `runes/pgvector/rune.toml` — 新規作成

```toml
[rune]
name        = "pgvector"
version     = "0.1.0"
description = "pgvector Rune for Favnir — PostgreSQL vector extension: upsert, query, create_index (IVFFLAT/HNSW)"
entry       = "pgvector.fav"
effects     = []

[dependencies]
```

### 3. `runes/pgvector/pgvector.fav` — 新規作成スタブ

```favnir
// pgvector Rune — PostgreSQL ベクトル拡張
// upsert, similarity query, index management
//
// NOTE: List<Float> は Vec<Float>[N] の将来フェーズ登録までのプレースホルダー。
//       VectorDBInterface — VectorDB 統一インターフェース（将来フェーズで切り替え可能）
//       include_str! テストのみ（型チェックエラーは無視する）。

// ベクトルエントリを追加・更新する
public fn upsert(table: String, id: String, vector: List<Float>, metadata: String) -> String {
    ""
}

// 近傍ベクトルを検索する
public fn query(table: String, vector: List<Float>, top_k: Int) -> List<String> {
    []
}

// ベクトルインデックスを作成する（IVFFLAT / HNSW）
// IndexTypeVec — IVFFLAT または HNSW インデックス種別
public fn create_index(table: String, index_type: String) -> String {
    ""
}
```

### 4. `runes/weaviate/rune.toml` — 新規作成

```toml
[rune]
name        = "weaviate"
version     = "0.1.0"
description = "Weaviate Rune for Favnir — vector database: upsert, query, schema management"
entry       = "weaviate.fav"
effects     = []

[dependencies]
```

### 5. `runes/weaviate/weaviate.fav` — 新規作成スタブ

```favnir
// Weaviate Rune — ベクトルデータベース
// upsert, query, schema management
//
// NOTE: List<Float> は Vec<Float>[N] の将来フェーズ登録までのプレースホルダー。
//       VectorDBInterface — VectorDB 統一インターフェース（将来フェーズで pgvector と統合予定）
//       include_str! テストのみ（型チェックエラーは無視する）。

// ベクトルオブジェクトを追加・更新する
public fn upsert(class_name: String, id: String, vector: List<Float>, properties: String) -> String {
    ""
}

// 近傍ベクトルを検索する
public fn query(class_name: String, vector: List<Float>, top_k: Int) -> List<String> {
    []
}

// スキーマ（クラス定義）を作成する
public fn schema_create(class_name: String, description: String) -> String {
    ""
}
```

### 6. `runes/qdrant/rune.toml` — 新規作成

```toml
[rune]
name        = "qdrant"
version     = "0.1.0"
description = "Qdrant Rune for Favnir — high-performance vector database: upsert, query, collection management"
entry       = "qdrant.fav"
effects     = []

[dependencies]
```

### 7. `runes/qdrant/qdrant.fav` — 新規作成スタブ

```favnir
// Qdrant Rune — 高性能ベクトルデータベース
// upsert, query, collection management
//
// NOTE: List<Float> は Vec<Float>[N] の将来フェーズ登録までのプレースホルダー。
//       VectorDBInterface — VectorDB 統一インターフェース（将来フェーズで pgvector と統合予定）
//       include_str! テストのみ（型チェックエラーは無視する）。

// ベクトルポイントを追加・更新する
public fn upsert(collection: String, id: String, vector: List<Float>, payload: String) -> String {
    ""
}

// 近傍ベクトルを検索する
public fn query(collection: String, vector: List<Float>, top_k: Int) -> List<String> {
    []
}

// コレクションを作成する
public fn collection_create(collection: String, vector_size: Int) -> String {
    ""
}
```

### 8. `driver.rs` — `v66400_tests` 追加

挿入位置: `// -- v66300_tests (v66.3.0)` コメントの直前

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

挿入後、`cargo build` でエラーなしを確認。

---

## 完了条件

- `runes/pinecone/pinecone.fav` に `list_indexes` が追加されている
- `runes/pgvector/pgvector.fav` が存在し `upsert`/`query`/`create_index` を含む
- `runes/weaviate/weaviate.fav` が存在し `upsert`/`query`/`schema_create` を含む
- `runes/qdrant/qdrant.fav` が存在し `upsert`/`query`/`collection_create` を含む
- 各 `rune.toml`（pgvector / weaviate / qdrant）が存在する
- `cargo test --bin fav v66400_tests` で 2 件 PASS
  - `vector_db_upsert_query` PASS
  - `vector_db_type_safe_dim` PASS
- `cargo test -j 8 -- --test-threads=8` で 3483 tests passed, 0 failed

---

## 非スコープ

- `Vec<Float>[N]` 次元型パラメータの型システム登録 — 将来フェーズ
- `VectorDB` 統一インターフェース（Favnir `interface` キーワード実装） — 将来フェーズ
- 実際の DB API 呼び出し実装 — 将来フェーズ（スタブのみ）
- `rune.toml` の `effects` 更新 — 本番 API 呼び出し実装時に `!Http` 等を追加（将来フェーズ）
- `fav check` での型チェック通過 — 今バージョンは `include_str!` テストのみ
- CHANGELOG.md 更新 — v67.0.0 宣言時に一括追記
- site/ MDX ドキュメント作成 — v66.9.0 安定化時に一括作成するため今バージョンは省略

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../../runes/pinecone/pinecone.fav"` → 既存ファイル
- `"../../runes/pgvector/pgvector.fav"` → 新規ファイル
- `"../../runes/weaviate/weaviate.fav"` → 新規ファイル
- `"../../runes/qdrant/qdrant.fav"` → 新規ファイル

### `contains` 判定の設計方針

- `pinecone.contains("upsert")` — 既存の `fn Pinecone.upsert(` にマッチ（`public fn` 不要）
- `pinecone.contains("query")` — 既存の `fn Pinecone.query(` にマッチ
- `pgvector.contains("fn upsert(")` — `public fn upsert(` にマッチ
- `pgvector.contains("fn query(")` — `public fn query(` にマッチ
- `pgvector.contains("VectorDBInterface")` — コメント `// VectorDBInterface — VectorDB 統一インターフェース` でマッチ。**注意**: コメントを変更・削除した場合は当該テストアサーションも連動して更新すること
- `weaviate.contains("fn schema_create(")` — `public fn schema_create(` にマッチ
- `qdrant.contains("fn collection_create(")` — `public fn collection_create(` にマッチ

### 既存 `pinecone.fav` の構文について

既存 `pinecone.fav` は `fn Pinecone.upsert(...)` 形式（`public fn` 修飾子なし）を使用している。
追加する `list_indexes` も同じ形式（`fn Pinecone.list_indexes(...)`）で統一すること。

### Favnir 構文ルール（v66.x 共通）

- `bind x <- expr` は Result/Option を返す式にのみ使用する（スタブでは不要）
- `let` は使わない
- `Math.sqrt` を使う（`Float.sqrt` は VM に存在しない）
- `Float.from_int` は VM に存在しない

### 新規 Rune の rune.toml フォーマット

- `entry = "ファイル名.fav"`（`main` ではなく `entry`）
- `effects = []` を明示
- `[dependencies]` セクションを含める
