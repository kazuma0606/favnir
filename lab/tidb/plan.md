# TiDB 検証 実装計画

更新日: 2026-06-03

## 方針

- 実装は小さく・段階的に進める
- 各フェーズ完了時に `proof/` へ証跡ログを保存する
- エラーが出たら原因を記録してから次フェーズへ進む
- Cargo.toml の変更は最小限（sqlx MySQL feature のみ追加）
- 既存テストを壊さないことを `cargo test` で毎フェーズ確認する

---

## Phase A: Docker 環境構築 + 接続確認

**目標**: TiDB がローカルで起動し、MySQL クライアントから接続できることを確認する。

### 手順

1. Docker で TiDB を起動する
   ```bash
   docker run -d --name tidb-lab \
     -p 4000:4000 \
     pingcap/tidb:latest
   ```

2. 接続確認（mysql クライアント or Rust の sqlx ping）
   ```bash
   mysql -h 127.0.0.1 -P 4000 -u root -e "SELECT version();"
   ```

3. テスト用 DB・テーブルを作成する
   ```sql
   CREATE DATABASE IF NOT EXISTS favnir_lab;
   USE favnir_lab;
   CREATE TABLE orders (
     id    INT PRIMARY KEY AUTO_INCREMENT,
     item  VARCHAR(255),
     amount FLOAT
   );
   ```

### 成果物

- `proof/01_connect.log`（接続確認の出力）

---

## Phase B: Rust VM Primitive 追加（基本 CRUD）

**目標**: `vm.rs` に `TiDb.query_raw` / `TiDb.execute_raw` を追加し、
Rust の統合テストから呼べることを確認する。

### 設計方針

- `sqlx::MySqlPool` を使った非同期接続
- クエリ結果は `serde_json::to_string` で JSON 文字列に変換して返す
- 接続プールは primitive 呼び出し毎に作成（シンプル優先）
- エラー時は `Err(メッセージ)` を返す

### 追加する primitive

```rust
// vm.rs
"TiDb.query_raw"   => tidb_query_raw(url, sql),
"TiDb.execute_raw" => tidb_execute_raw(url, sql),
```

### テスト

```rust
// driver.rs
#[test]
fn tidb_query_raw_returns_json() {
    // Docker が起動していない場合は skip
    // TIDB_URL 環境変数がなければ skip
}
```

### 成果物

- `fav/src/vm.rs` に TiDb primitive 追加
- `proof/02_crud.log`（INSERT/SELECT/DELETE の実行ログ）

---

## Phase C: Rune 実装（runes/tidb/）

**目標**: `import rune "tidb"` で Favnir から TiDB を操作できるようにする。

### ディレクトリ構成

```
runes/tidb/
  rune.toml
  client.fav    — connect(url) -> TiDbConn
  query.fav     — query<T>(conn, sql) -> List<T>
  execute.fav   — execute(conn, sql) -> Int
```

### 設計方針

- `TiDbConn` は接続 URL の文字列ラッパーとして実装（VM は URL を直接受け取る）
- `query<T>` は JSON 文字列を `Schema.adapt_list` でデシリアライズ
- 既存の DuckDB Rune / SQL Rune を参考にする

### テスト

- `cargo test tidb_rune_loads` — rune ロード確認
- `fav run lab/tidb/demo.fav` — 手動 E2E 確認

### 成果物

- `runes/tidb/` ディレクトリ一式
- `lab/tidb/demo.fav`（動作確認用 Favnir スクリプト）
- `proof/04_favnir_pipeline.log`

---

## Phase D: Vector Search 検証

**目標**: TiDB の Vector Search 機能を Favnir から呼べることを確認する。

### 前提

TiDB v8.0+ では `VECTOR` 型と `VEC_COSINE_DISTANCE` 関数が利用可能。

```sql
CREATE TABLE docs (
  id        INT PRIMARY KEY AUTO_INCREMENT,
  content   TEXT,
  embedding VECTOR(1536)
);

SELECT id, content,
       VEC_COSINE_DISTANCE(embedding, '[0.1, 0.2, ...]') AS score
FROM docs
ORDER BY score
LIMIT 5;
```

### VM Primitive

```rust
"TiDb.vector_search_raw" => tidb_vector_search_raw(url, table, vec_json, top_k),
```

### Rune 追加

```
runes/tidb/vector.fav — vector_search<T>(conn, table, vec, top_k) -> List<T>
```

### AI パイプライン検証

```favnir
import rune "tidb"
import rune "llm"

stage SearchDocs: String -> List<Doc> !Db !Llm = |query| {
  bind conn = tidb.connect(Env.get("TIDB_URL"))
  bind vec  = llm.embed(query)
  tidb.vector_search<Doc>(conn, "docs", vec, 5)
}
```

### 成果物

- `vm.rs` に `TiDb.vector_search_raw` 追加
- `runes/tidb/vector.fav`
- `lab/tidb/rag_demo.fav`
- `proof/03_vector_search.log`

---

## Phase E: 証跡整理 + まとめ

**目標**: 証跡を整理し、Zenn 記事・Snowflake ロードマップへのインプットを整理する。

### 作業

- `proof/` 内のログを確認・整理
- `lab/tidb/findings.md` に気づき・制限事項・Snowflake 対比を記録
- Zenn 記事の構成案を `zenn/tidb-ai-pipeline.md` に起こす（骨格のみ）

### 完了条件

- 4 つの証跡ログがすべて `proof/` に揃っている
- `cargo test` が全件通過している
- `findings.md` が存在する

---

## フェーズ依存関係

```
Phase A（Docker 起動）
    │
Phase B（Rust primitive）
    │
Phase C（Rune 実装）
    │
Phase D（Vector Search）
    │
Phase E（証跡整理）
```

---

## リポジトリへの影響

| 変更対象 | 内容 |
|---|---|
| `fav/Cargo.toml` | sqlx MySQL feature 追加 |
| `fav/src/vm.rs` | TiDb.* primitive 追加 |
| `runes/tidb/` | 新規ディレクトリ |
| `lab/tidb/` | 検証スクリプト・証跡（本ファイル） |
