# v66.4.0 タスクリスト

Status: COMPLETE
Version: 66.4.0
Base tests: 3481
Target tests: 3483
Actual tests: 3483

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3481 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（sub-version では変更しない）
- [x] `runes/pinecone/pinecone.fav` が存在し `upsert` / `query` / `delete` / `fetch` を含むことを確認（既存）
- [x] `runes/pgvector/` ディレクトリが存在しないことを確認（新規作成対象）
- [x] `runes/weaviate/` ディレクトリが存在しないことを確認（新規作成対象）
- [x] `runes/qdrant/` ディレクトリが存在しないことを確認（新規作成対象）
- [x] `driver.rs` に `v66300_tests` が存在することを確認（`v66400_tests` の挿入位置）
- [x] `driver.rs` に `v66400_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v66300_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `embed_rune_openai`, `embed_rune_local_model`
- [x] `versions/current.md` の「進行中バージョン」が `v66.3.0` であることを確認

---

## T1: Rune ファイル作成・更新

### pinecone.fav — 既存ファイルへの追加

- [x] `runes/pinecone/pinecone.fav` の末尾に `fn Pinecone.list_indexes()` を追加
  - 既存の `fn Pinecone.upsert` / `query` / `delete` / `fetch` 構文と統一（`fn Pinecone.` 形式）
  - 戻り値: `Result<List<String>, String>`
- [x] 追記後、既存の `upsert` / `query` / `delete` / `fetch` が壊れていないことを確認

### pgvector（新規）

- [x] `runes/pgvector/` ディレクトリ作成
- [x] `runes/pgvector/rune.toml` 作成（`entry` / `effects = []` / `[dependencies]` 形式）
- [x] `runes/pgvector/pgvector.fav` 作成（以下の全 3 関数を定義）
  - [x] `upsert(table, id, vector, metadata)` — `""` を返すスタブ
  - [x] `query(table, vector, top_k)` — `[]` を返すスタブ
  - [x] `create_index(table, index_type)` — `""` を返すスタブ（コメントに `IndexTypeVec` を含む）
  - [x] ヘッダーコメントに `VectorDBInterface` を含む (**`pgvector.contains("VectorDBInterface")` テストにマッチ**)

### weaviate（新規）

- [x] `runes/weaviate/` ディレクトリ作成
- [x] `runes/weaviate/rune.toml` 作成（`entry` / `effects = []` / `[dependencies]` 形式）
- [x] `runes/weaviate/weaviate.fav` 作成（以下の全 3 関数を定義）
  - [x] `upsert(class_name, id, vector, properties)` — `""` を返すスタブ
  - [x] `query(class_name, vector, top_k)` — `[]` を返すスタブ
  - [x] `schema_create(class_name, description)` — `""` を返すスタブ

### qdrant（新規）

- [x] `runes/qdrant/` ディレクトリ作成
- [x] `runes/qdrant/rune.toml` 作成（`entry` / `effects = []` / `[dependencies]` 形式）
- [x] `runes/qdrant/qdrant.fav` 作成（以下の全 3 関数を定義）
  - [x] `upsert(collection, id, vector, payload)` — `""` を返すスタブ
  - [x] `query(collection, vector, top_k)` — `[]` を返すスタブ
  - [x] `collection_create(collection, vector_size)` — `""` を返すスタブ

### 共通確認

- [x] 新規 `.fav` ファイル内に `let ` が含まれないことを確認
- [x] 新規 `.fav` ファイル内に `bind.*=`（`<-` でない bind）が含まれないことを確認
- [x] 新規 `.fav` ファイル内に `Float.from_int` / `Float.sqrt` が含まれないことを確認

---

## T2: `driver.rs` — `v66400_tests` 追加

- [x] `// -- v66300_tests (v66.3.0)` コメントの直前に `v66400_tests` を挿入
  - [x] `vector_db_upsert_query`:
    - `pinecone.fav` に `"upsert"` / `"query"` / `"list_indexes"` を含む
    - `pgvector.fav` に `"fn upsert("` / `"fn query("` / `"VectorDBInterface"` を含む
    - **注意**: `pgvector.fav` の `VectorDBInterface` はコメント行に存在。削除時はテストも連動更新
  - [x] `vector_db_type_safe_dim`:
    - `weaviate.fav` に `"fn upsert("` / `"fn query("` / `"fn schema_create("` を含む
    - `qdrant.fav` に `"fn upsert("` / `"fn query("` / `"fn collection_create("` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v66400_tests` で 2 件 PASS
  - [x] `vector_db_upsert_query` PASS
  - [x] `vector_db_type_safe_dim` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3483 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v66.1-v67.0.md` のバージョン一覧表で v66.4.0 の「状態」列を「完了」に変更
- [x] `versions/current.md` の「進行中バージョン」を v66.4.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v66.1〜v66.9 では CHANGELOG.md を更新しない。v67.0.0 宣言時に一括追記する。
> **MDX 方針**: `site/` の MDX ドキュメントは v66.9.0 安定化時に一括作成するため今バージョンは省略。

---

## コードレビュー指摘と対応

<!-- 実装完了後に追記 -->
