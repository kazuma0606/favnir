# v25.4.0 仕様書 — mysql Rune 実質化

## 概要

| 項目 | 内容 |
|---|---|
| バージョン | v25.4.0 |
| フェーズ | Rune Foundation（v25.1〜v26.0） |
| テーマ | mysql Rune の「動く Rune」5 条件達成 |
| 依存関係 | v25.1.0（interface DbConn 定義・examples/ ディレクトリが v25.1.0 で作成済み） |
| 目標テスト数 | 2000 件（+6 件 ≥ ロードマップ最小 4 件） |

---

## 背景と目的

v25.3.0 で redis Rune を実質化した。次は「企業 DB の定番」MySQL を実質化する。
ロードマップ v25.4 の設計方針は「`interface DbConn` を通じて postgres との API を統一する」であり、
MySQL Rune が Postgres Rune と同一の関数シグネチャ（connect / query / execute / transaction）を持つことで
DB 依存を注入パターンで切り替え可能にする。

既存の `runes/mysql/mysql.fav` は v24.5.0 で追加されたスタブのみ（関数定義なし）。

---

## 「動く Rune」5 条件

| # | 条件 | 対象 |
|---|---|---|
| 1 | connect | `MYSQL_URL` 環境変数（例: `mysql://user:pass@localhost:3306/db`）経由で接続確立 |
| 2 | read | `MySQL.query(conn, sql, params)` — 行を JSON 配列文字列として返す |
| 3 | write | `MySQL.execute(conn, sql, params)` — 影響行数返却 |
| 4 | error | `Result<T, String>` 統一、エラーメッセージに SQL を含む |
| 5 | test | `v254000_tests` 6 件 PASS + `examples/mysql_orders_etl.fav` E2E デモ |

---

## 既存実装の現状

| ファイル | 状態 | 備考 |
|---|---|---|
| `runes/mysql/mysql.fav` | スタブのみ（関数なし） | v24.5.0 で追加 |
| `Effect::MySQL` | **未定義** | v25.4.0 で追加（`ast.rs`） |
| `MySQL.*_raw` primitives | **なし** | v25.4.0 で追加（`vm.rs`） |
| `mysql` crate | **未追加** | v25.4.0 で `Cargo.toml` に追加 |

---

## 機能仕様

### 型定義

```favnir
// 接続 URL ラッパー型（"mysql://user:pass@host:port/db" 形式）
// runes/mysql/mysql.fav に直接定義（単一ファイル Rune）
// 将来 impl DbConn に移行する場合は db_conn.fav に分離予定
type MySqlConn(String)
```

### 追加関数一覧

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `MySQL.connect` | `(url: String) -> Result<MySqlConn, String> !MySQL` | 接続確立（PING 確認） |
| `MySQL.query` | `(conn: MySqlConn, sql: String, params: String) -> Result<String, String> !MySQL` | SELECT クエリ（行を JSON 配列文字列として返す） |
| `MySQL.execute` | `(conn: MySqlConn, sql: String, params: String) -> Result<Int, String> !MySQL` | UPDATE / INSERT / DELETE（影響行数を返す） |
| `MySQL.transaction_begin` | `(conn: MySqlConn) -> Result<Unit, String> !MySQL` | BEGIN（トランザクション開始） |
| `MySQL.transaction_commit` | `(conn: MySqlConn) -> Result<Unit, String> !MySQL` | COMMIT |
| `MySQL.transaction_rollback` | `(conn: MySqlConn) -> Result<Unit, String> !MySQL` | ROLLBACK |

> **params 形式**: JSON 配列文字列（例: `"[\"pending\", 30]"`）— `Postgres.query_raw` と同パターン。
>
> **query の戻り値**: 各行を `{"col": "val", ...}` 形式の JSON オブジェクトとして持つ JSON 配列文字列。
> 例: `"[{\"id\": \"1\", \"name\": \"alice\"}]"`
>
> **query_raw / execute_raw / transaction_*_raw の第 1 引数**: `MySqlConn` ではなく `String`（URL 文字列）を受け取る。
> checker.rs は `connect_raw` の戻り型を `Result<String, String>` として扱い、bind で受け取った `conn` は
> String として処理される（`MySqlConn(String)` 名目型のラッパー解除は不要 — PgConn / RedisConn と同パターン）。
>
> **VM 制約（重要）**: `transaction_begin / commit / rollback` は各呼び出し時に独立した新規接続を確立するため、
> 3 つの primitive は同一接続上のトランザクションとして動作しない（擬似実装）。実際の原子性は保証されない。
> これは v25.4.0 の意図的スコープ外。コネクションプールと合わせて v26.x で解決予定（破壊的変更なし）。

> **ロードマップからの意図的逸脱**: ロードマップ v25.4 の `transaction(conn, fn)` は VM からクロージャを
> 引数として渡せない制約により、v25.4.0 では `transaction_begin / commit / rollback` の 3 分割として実装する。
> 将来バージョンで高レベル `MySQL.transaction(conn, fn)` を追加する予定（Postgres と同一形式に統一）。

---

## エフェクト追加仕様（`!MySQL`）

v25.4.0 で `Effect::MySQL` を新たに追加する。

| ファイル | 変更内容 |
|---|---|
| `fav/src/ast.rs` | `Effect` enum に `MySQL` バリアント追加（`Redis` バリアントの直後、`AzureDb` の前） |
| `fav/src/middle/checker.rs` | `ns_to_inferred_effect` / `require_mysql_effect` / MySQL builtin fns 追加 |
| `fav/src/middle/reachability.rs` | `Effect::*` 網羅的 match に `MySQL` 追加 |
| `fav/src/middle/ast_lower_checker.rs` | `ast::Effect::*` 網羅的 match に `MySQL` 追加 |
| `fav/src/emit_python.rs` | `Effect::MySQL => "MySQL"` アームを追加 |
| `fav/src/lineage.rs` | `Effect::MySQL` のリネージ追跡追加（`"write"` / `"DbWrite"` に分類） |
| `fav/src/lint.rs` | `effect_to_str` 網羅的 match に `Effect::MySQL` 追加 |
| `fav/src/error_catalog.rs` | E0321「undeclared !MySQL effect」追加 |
| `fav/src/fmt.rs` | `Effect::MySQL => Some("!MySQL".to_string())` 追加 |
| `fav/src/frontend/parser.rs` | `"MySQL" => Effect::MySQL` アーム追加（`"Redis"` の後） |
| `fav/src/driver.rs` | `format_effects` / `effect_json_name` に `MySQL` アーム追加 |

> **注意**: `Effect::MySQL` 追加で更新が必要なファイルは合計 11 ファイル。
> 実装時は `cargo build` で exhaustive match エラーを確認しながら進めること。

---

## MySQL クライアント実装方針

- `mysql = { version = "24", default-features = false }` を
  `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` に追加
  （crates.io で最新安定版を確認すること。v24 が存在しない場合は v23 にダウングレード）
- 同期 API（`mysql::Conn::new(url)` → `conn.exec_drop` / `conn.exec`）を使用
- `MYSQL_URL` 環境変数を優先、未設定時は `mysql://root@127.0.0.1:3306/test` をフォールバック
- `connect_raw` は URL を `MySqlConn` にラップ（実際の接続確立は各 raw primitive 内で実施）
  — RedisConn / PgConn パターンと同様の設計（接続プールは v26.x 以降）
- `cfg(not(target_arch = "wasm32"))` ガードを全 MySQL primitive に付与

### VM primitives 一覧（6 件）

| primitive 名 | 引数 | 戻り値 |
|---|---|---|
| `MySQL.connect_raw` | `url: String` | `Result<String, String>`（MySqlConn ラッパー） |
| `MySQL.query_raw` | `url: String, sql: String, params_json: String` | `Result<String, String>`（JSON 配列文字列） |
| `MySQL.execute_raw` | `url: String, sql: String, params_json: String` | `Result<Int, String>` |
| `MySQL.transaction_begin_raw` | `url: String` | `Result<Unit, String>` |
| `MySQL.transaction_commit_raw` | `url: String` | `Result<Unit, String>` |
| `MySQL.transaction_rollback_raw` | `url: String` | `Result<Unit, String>` |

> **connect_raw の戻り型**（checker レベル）: `Result<String, String>`。
> `runes/mysql/mysql.fav` では `Result<MySqlConn, String>` として公開するが、
> `MySqlConn(String)` は名目型ラッパーなので checker は String として扱う
> （PgConn / RedisConn と同じパターン — 意図的な簡略化）。

---

## エラーコード

| コード | 名前 | 説明 |
|---|---|---|
| E0321 | UndeclaredMySQLEffect | `!MySQL` エフェクトなしで MySQL 系 Rune を呼び出した場合 |

---

## `examples/mysql_orders_etl.fav`

```favnir
import rune "mysql"

// ── MySQL を使った注文 ETL デモ (v25.4.0) ─────────────────────────────────────
// 前提: docker run -e MYSQL_ROOT_PASSWORD=root -e MYSQL_DATABASE=shop -p 3306:3306 mysql:8
// 実行: fav run examples/mysql_orders_etl.fav

stage LoadPendingOrders: Unit -> String !MySQL = |_| {
    bind conn <- MySQL.connect("mysql://root:root@localhost:3306/shop")
    MySQL.query(conn, "SELECT id, amount FROM orders WHERE status = ?", "[\"pending\"]")
}

stage MarkProcessed: String -> Int !MySQL = |order_id| {
    bind conn <- MySQL.connect("mysql://root:root@localhost:3306/shop")
    MySQL.execute(conn, "UPDATE orders SET status = ? WHERE id = ?", "[\"processed\", " + order_id + "]")
}

pipeline OrdersETL = LoadPendingOrders |> MarkProcessed
```

---

## interface DbConn 統合方針

ロードマップ「postgres と同一の API シグネチャで統一」に準拠し、
`mysql.fav` の公開 API は `postgres/client.fav` と同一関数名・引数構造とする。

| 関数 | Postgres | MySQL |
|---|---|---|
| connect | `Postgres.connect(url)` | `MySQL.connect(url)` |
| query | `Postgres.query(conn, sql, params)` | `MySQL.query(conn, sql, params)` |
| execute | `Postgres.execute(conn, sql, params)` | `MySQL.execute(conn, sql, params)` |
| transaction | `Postgres.transaction_begin/commit/rollback` | `MySQL.transaction_begin/commit/rollback` |

将来の `impl DbConn for MySqlConn` による完全統一は v26.x 以降で実施予定。
ただし v25.4.0 の `transaction` は低レベル 3 分割（begin/commit/rollback）であり、
DbConn interface の `transaction(conn, fn)` シグネチャとは一致しない。
v26.x での統合時に `MySQL.transaction(conn, fn)` を高レベル形式で追加する移行計画が必要（破壊的変更なし）。

---

## やらないこと（スコープ外）

- コネクションプール（v26.x 以降）
- `query[T]` ジェネリクス型推論（`String` 返却のみ、デシリアライズは呼び出し元）
- SSL/TLS 接続（`mysql+tls://` スキーム）
- ストアドプロシージャ / プリペアドステートメントキャッシュ
- MySQL Cluster / Aurora / MariaDB 固有 API

---

## 完了条件

| # | 条件 |
|---|---|
| 1 | `MySQL.connect` が `runes/mysql/mysql.fav` に実装済み |
| 2 | `MySQL.query` / `MySQL.execute` が `runes/mysql/mysql.fav` に実装済み |
| 3 | `MySQL.transaction_begin` / `commit` / `rollback` が `runes/mysql/mysql.fav` に実装済み |
| 4 | `MySQL.*_raw` VM primitives（6 件）が `fav/src/backend/vm.rs` に存在する |
| 5 | `Effect::MySQL` が `fav/src/ast.rs` に存在し、E0321 が `error_catalog.rs` に存在する（`cargo build` で exhaustive match 確認済み） |
| 6 | `examples/mysql_orders_etl.fav` が存在し `import rune "mysql"` + `query` + `execute` を含む |
| 7 | `CHANGELOG.md` に `[v25.4.0]` エントリが存在する |
| 8 | `site/content/docs/runes/mysql.mdx` に新規 API（connect / query / execute / transaction_begin/commit/rollback）が記載済み |
| 9 | `cargo test v254000` で 6 件すべて PASS |
| 10 | 総テスト数 ≥ 2000 件 |

---

## 検証コマンド

```bash
cd fav && cargo test v254000 -- --test-threads=1
cd fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
```
