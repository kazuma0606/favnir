# TiDB 検証 タスク一覧

更新日: 2026-06-03

タスクが完了したら `[ ]` を `[x]` に変える。

---

## Phase A: Docker 環境構築 + 接続確認

- [ ] A-1: Docker で TiDB を起動する
  ```bash
  docker run -d --name tidb-lab -p 4000:4000 pingcap/tidb:latest
  ```
- [ ] A-2: MySQL クライアントまたは sqlx で接続確認
  ```bash
  mysql -h 127.0.0.1 -P 4000 -u root -e "SELECT version();"
  ```
- [ ] A-3: テスト用 DB・テーブルを作成する
  ```sql
  CREATE DATABASE IF NOT EXISTS favnir_lab;
  USE favnir_lab;
  CREATE TABLE orders (
    id INT PRIMARY KEY AUTO_INCREMENT,
    item VARCHAR(255),
    amount FLOAT
  );
  ```
- [ ] A-4: `proof/01_connect.log` に接続確認ログを保存する

---

## Phase B: Rust VM Primitive 追加（基本 CRUD）

- [ ] B-1: `fav/Cargo.toml` に sqlx MySQL feature を追加する
  ```toml
  sqlx = { version = "0.8", features = ["mysql", "runtime-tokio-native-tls"] }
  ```
- [ ] B-2: `vm.rs` に `tidb_query_raw(url, sql) -> Result<String, String>` を実装する
  - SELECT 結果を JSON 文字列で返す
- [ ] B-3: `vm.rs` に `tidb_execute_raw(url, sql) -> Result<Int, String>` を実装する
  - INSERT/UPDATE/DELETE の影響行数を返す
- [ ] B-4: `vm.rs` の dispatch テーブルに `"TiDb.query_raw"` / `"TiDb.execute_raw"` を登録する
- [ ] B-5: `cargo build` が通ることを確認する
- [ ] B-6: `cargo test` が全件通過することを確認する（既存テスト非破壊確認）
- [ ] B-7: Rust から手動で INSERT → SELECT を実行し動作確認する
- [ ] B-8: `proof/02_crud.log` に CRUD 実行ログを保存する

---

## Phase C: Rune 実装（runes/tidb/）

- [ ] C-1: `runes/tidb/rune.toml` を作成する
- [ ] C-2: `runes/tidb/client.fav` を作成する
  - `fn connect(url: String) -> TiDbConn !Db`
- [ ] C-3: `runes/tidb/query.fav` を作成する
  - `fn query<T>(conn: TiDbConn, sql: String) -> List<T> !Db`
- [ ] C-4: `runes/tidb/execute.fav` を作成する
  - `fn execute(conn: TiDbConn, sql: String) -> Int !Db`
- [ ] C-5: `lab/tidb/demo.fav` を作成する（手動 E2E 用）
  ```favnir
  import rune "tidb"

  stage InsertOrder: String -> Int !Db = |item| {
    bind conn = tidb.connect(Env.get("TIDB_URL"))
    tidb.execute(conn, "INSERT INTO orders (item, amount) VALUES ('" + item + "', 100.0)")
  }

  stage FetchOrders: Unit -> List<Order> !Db = |_| {
    bind conn = tidb.connect(Env.get("TIDB_URL"))
    tidb.query<Order>(conn, "SELECT id, item, amount FROM orders")
  }
  ```
- [ ] C-6: `fav run lab/tidb/demo.fav` が通ることを確認する
- [ ] C-7: `cargo test` が全件通過することを確認する
- [ ] C-8: `proof/04_favnir_pipeline.log` に実行ログを保存する

---

## Phase D: Vector Search 検証

- [ ] D-1: TiDB のバージョンが Vector Search に対応しているか確認する
  ```sql
  SELECT version();
  -- v8.0+ であれば VECTOR 型が使用可能
  ```
- [ ] D-2: Vector Search 用テーブルを作成する
  ```sql
  CREATE TABLE docs (
    id        INT PRIMARY KEY AUTO_INCREMENT,
    content   TEXT,
    embedding VECTOR(4)  -- 検証用に次元数を小さく
  );
  INSERT INTO docs (content, embedding) VALUES
    ('Favnir is a pipeline language', '[0.1, 0.2, 0.3, 0.4]'),
    ('TiDB supports vector search',   '[0.5, 0.6, 0.7, 0.8]'),
    ('Rust is a systems language',    '[0.9, 0.1, 0.2, 0.3]');
  ```
- [ ] D-3: SQL で Vector Search が動くことを確認する
  ```sql
  SELECT id, content,
         VEC_COSINE_DISTANCE(embedding, '[0.1, 0.2, 0.3, 0.4]') AS score
  FROM docs
  ORDER BY score LIMIT 3;
  ```
- [ ] D-4: `vm.rs` に `tidb_vector_search_raw(url, table, vec_json, top_k)` を実装する
- [ ] D-5: `runes/tidb/vector.fav` を作成する
  - `fn vector_search<T>(conn, table, vec, top_k) -> List<T> !Db`
- [ ] D-6: `lab/tidb/rag_demo.fav` を作成する（LLM embed + Vector Search）
- [ ] D-7: `fav run lab/tidb/rag_demo.fav` が通ることを確認する（LLM_PROVIDER 設定要）
- [ ] D-8: `proof/03_vector_search.log` に Vector Search 実行ログを保存する

---

## Phase E: 証跡整理 + まとめ

- [ ] E-1: `proof/` 内の 4 ファイルがすべて揃っていることを確認する
  - `01_connect.log`
  - `02_crud.log`
  - `03_vector_search.log`
  - `04_favnir_pipeline.log`
- [ ] E-2: `cargo test` が全件通過することを最終確認する
- [ ] E-3: `lab/tidb/findings.md` を作成する
  - 動作確認できた機能
  - 制限事項・ハマりポイント
  - Snowflake 対応との比較（MySQL vs REST API）
  - Zenn 記事に使えるポイント
- [ ] E-4: `zenn/tidb-ai-pipeline.md` に記事の骨格を作成する（セクション見出しのみでよい）

---

## 完了条件

| 条件 | 確認 |
|---|---|
| Docker TiDB に Rust から接続できる | |
| `TiDb.query_raw` / `TiDb.execute_raw` が動く | |
| `import rune "tidb"` で Favnir から CRUD できる | |
| Vector Search が SQL で動く | |
| Vector Search を Favnir から呼べる | |
| `cargo test` 全件通過 | |
| `proof/` に 4 つの証跡ログがある | |
| `findings.md` が存在する | |
