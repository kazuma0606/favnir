# TiDB 検証 仕様書

更新日: 2026-06-03

## 概要

TiDB（MySQL 互換 分散 SQL データベース）を Favnir から型安全に扱えることを検証する。
将来的な `tidb` Rune 実装と Zenn 記事（TiDB × Favnir AI データ基盤）の基礎とする。

---

## スコープ

### 含むもの

- Docker による TiDB ローカル起動
- Rust（vm.rs）からの MySQL プロトコル接続（sqlx MySQL feature）
- `TiDb.query_raw` / `TiDb.execute_raw` primitive の追加
- `runes/tidb/` — `connect` / `query<T>` / `execute` の基本 Rune
- Vector Search の検証（`TiDb.vector_search_raw`）
- `!Db` エフェクトでの型チェック通過確認
- 証跡ファイル（`lab/tidb/proof/`）への実行ログ保存

### 含まないもの

- `Effect::TiDb` の新規追加（既存の `!Db` を流用する）
- fav.toml `[tidb]` セクション（今回はスコープ外）
- TiDB Cloud / AWS 上での本番検証（ローカル Docker のみ）
- Zenn 記事の執筆（実装完了後に別タスク）
- バージョン番号のインクリメント（lab 検証のため version 管理外）

---

## 接続設計

### ローカル Docker

```bash
docker run -d --name tidb-lab \
  -p 4000:4000 \
  pingcap/tidb:latest
```

接続 URL: `mysql://root@127.0.0.1:4000/test`

### Rust 側の依存追加

```toml
# fav/Cargo.toml
sqlx = { version = "0.8", features = ["mysql", "runtime-tokio-native-tls"] }
```

### 環境変数

| 変数名 | 説明 | デフォルト |
|---|---|---|
| `TIDB_URL` | 接続 URL | `mysql://root@127.0.0.1:4000/test` |

---

## VM Primitive 設計

### 基本 CRUD

```
TiDb.query_raw(url: String, sql: String) -> Result<String, String>
  — SELECT 結果を JSON 文字列で返す

TiDb.execute_raw(url: String, sql: String) -> Result<Int, String>
  — INSERT/UPDATE/DELETE を実行し、影響行数を返す
```

### Vector Search

```
TiDb.vector_search_raw(url: String, table: String, vec_json: String, top_k: Int)
  -> Result<String, String>
  — ベクトル類似検索。vec_json は "[0.1, 0.2, ...]" 形式
  — 結果は JSON 文字列
```

---

## Rune 設計

```favnir
import rune "tidb"

// 接続
fn connect(url: String) -> TiDbConn !Db

// クエリ
fn query<T>(conn: TiDbConn, sql: String) -> List<T> !Db

// DML
fn execute(conn: TiDbConn, sql: String) -> Int !Db

// Vector Search（AI 統合用）
fn vector_search<T>(conn: TiDbConn, table: String, vec: List<Float>, top_k: Int)
  -> List<T> !Db
```

### 使用例

```favnir
import rune "tidb"
import rune "llm"

type Doc = { id: Int  content: String  embedding: List<Float> }

stage EmbedAndSearch: String -> List<Doc> !Db !Llm = |query| {
  bind conn   = tidb.connect(Env.get("TIDB_URL"))
  bind vec    = llm.embed(query)
  tidb.vector_search<Doc>(conn, "docs", vec, 5)
}

seq RAGSearch = EmbedAndSearch
```

---

## エフェクト方針

TiDB は既存の `!Db` エフェクトを流用する。
新規エフェクト追加（`!TiDb`）は将来の Snowflake 対応（v10.3.0）パターンを確立してから検討。

---

## 証跡

`lab/tidb/proof/` に以下を保存する。

| ファイル | 内容 |
|---|---|
| `01_connect.log` | Docker 起動 + 接続確認 |
| `02_crud.log` | INSERT / SELECT / UPDATE / DELETE |
| `03_vector_search.log` | Vector Search クエリ結果 |
| `04_favnir_pipeline.log` | Favnir から tidb rune を呼んだ実行ログ |
