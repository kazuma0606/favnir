# v14.1.0 Spec — Azure PostgreSQL Rune

Date: 2026-06-11

---

## 概要

Azure DB for PostgreSQL への接続・クエリを Favnir から行うための VM プリミティブ・エフェクト宣言・型付き Rune を追加する。

これは CrossCloud E2E Demo（v15.0.0）の基盤となる最初のステップ。既存の `runes/postgres/` が `!Postgres` エフェクトで AWS RDS に接続するのと対称的に、`runes/azure-postgres/` は `!AzureDb` エフェクトで Azure DB for PostgreSQL に接続する。

---

## 1. VM プリミティブ設計

### 1-1. 関数シグネチャ

| プリミティブ | 引数 | 戻り値 | 備考 |
|---|---|---|---|
| `AzurePostgres.execute_raw` | `conn_str, sql, params` | `Result<Int, String>` | 変更行数を返す |
| `AzurePostgres.query_raw` | `conn_str, sql, params` | `Result<String, String>` | JSON 配列文字列を返す |

- `conn_str`: 接続文字列（例: `postgresql://user:pass@host:5432/db?sslmode=require`）
- `sql`: SQL 文字列
- `params`: JSON 配列文字列（例: `["val1", 42]`）

### 1-2. 既存 Postgres との違い

| 項目 | `Postgres.*_raw` | `AzurePostgres.*_raw` |
|---|---|---|
| 接続先 | 環境変数（`DATABASE_URL` 等）から読む | 第1引数 `conn_str` で明示 |
| TLS | 任意（`sslmode=disable` 可） | 必須（Azure 側で強制） |
| 実装 | `tokio-postgres` 直結 | `tokio-postgres` + `openssl` TLS |

接続文字列を引数で渡す設計にする理由:
- CrossCloud シナリオでは複数の Azure PG インスタンスへ接続する可能性がある
- `AzureCtx` に接続文字列を格納し `ctx.db` 経由で渡す想定（v14.2.0）
- 環境変数を使う場合は呼び出し側で `IO.getenv_raw` または `fav.toml` 展開で取得する

### 1-3. TLS 設定

Azure DB for PostgreSQL は SSL を強制する。接続文字列に `sslmode=require` を含める。

Cargo 依存:
```toml
openssl = { version = "0.10", features = ["v102", "v110"] }
tokio-postgres-openssl = "0.5"
```

> **Windows 開発時の注意**: `OPENSSL_DIR` 環境変数が必要になる場合がある（vcpkg または chocolatey で openssl をインストール）。

### 1-4. 接続方式

各プリミティブ呼び出しで新規 TLS 接続を確立し、SQL 実行後に切断する（connection pool は v14.x 以降の検討事項）。接続失敗・SQL エラーはすべて `Err(String)` として返す。

---

## 2. エフェクトシステム

### `!AzureDb` エフェクト追加

| ファイル | 変更内容 |
|---|---|
| `fav/src/middle/checker.rs` | `BUILTIN_EFFECTS` に `"AzureDb"` を追加 |
| `fav/src/middle/checker.rs` | `builtin_ret_ty` に `"AzurePostgres"` namespace を追加 |
| `fav/src/middle/checker.rs` | `ns_to_effect` に `"AzurePostgres" => "AzureDb"` を追加 |

### checker.rs 追加内容

`builtin_ret_ty`:
```rust
"AzurePostgres" => match method {
    "execute_raw" => Type::Result(Box::new(Type::Int), Box::new(s())),
    "query_raw"   => Type::Result(Box::new(s()),       Box::new(s())),
    _ => Type::Unknown,
},
```

`BUILTIN_EFFECTS` / `ns_to_effect`:
```rust
"AzureDb" => true,   // BUILTIN_EFFECTS
// ns_to_effect:
"AzurePostgres" => "AzureDb",
```

E0252（未知エフェクト）が `!AzureDb` で発生しないようにする。

---

## 3. リネージ

### `!AzureDb(read/write)` 区別

`fav/src/lineage.rs` に以下を追加:

```rust
// EffectKind enum
AzureDbRead,
AzureDbWrite,
```

`collect_call_kinds` で `AzurePostgres.execute_raw` → `AzureDbWrite`、`AzurePostgres.query_raw` → `AzureDbRead` にマッピングする。

`fav explain --lineage` 出力例:
```
!AzureDb(write)  AzurePostgres.execute_raw  → Azure DB for PostgreSQL
!AzureDb(read)   AzurePostgres.query_raw    ← Azure DB for PostgreSQL
```

（CrossCloud lineage の完全フォーマットは v14.3.0 で拡充）

---

## 4. `AzureDbCtx` 型 と Rune 設計

### 4-1. ファイル構成

```
runes/azure-postgres/
  azure_postgres.fav   — public API（client.fav を re-export）
  client.fav           — AzureDbCtx 型定義 + execute / query ラッパー
```

### 4-2. `AzureDbCtx` 型

接続文字列を包む名目型（nominal type）:

```
type AzureDbCtx(String)
```

- `AzureDbCtx(conn_str)` で生成
- `match ctx { AzureDbCtx(conn_str) => ... }` で unwrap
- v14.2.0 で `AzureCtx.db` フィールドとして組み込む予定

### 4-3. Rune 公開 API

```
// 生成
fn new_ctx(conn_str: String) -> AzureDbCtx

// DML 実行（INSERT / UPDATE / DELETE）— 変更行数を返す
fn execute(ctx: AzureDbCtx, sql: String, params: String) -> Result<Int, String> !AzureDb

// SELECT クエリ — 行を型 T の List に変換して返す
fn query<T>(ctx: AzureDbCtx, sql: String, params: String) -> Result<List<T>, String> !AzureDb
```

`params` は JSON 配列文字列（例: `"[\"val1\", 42]"`）。空の場合は `"[]"` を渡す。

### 4-4. `AzureDbCtx` は checker.rs への追加不要

`type AzureDbCtx(String)` は Favnir ソースで定義する名目型であり、Rust checker 側に追加する必要はない。checker.rs が必要とするのは `AzurePostgres` namespace（builtin_ret_ty）と `!AzureDb`（BUILTIN_EFFECTS）のみ。

---

## 5. 影響ファイル

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/Cargo.toml` | 依存追加 | `openssl`, `tokio-postgres-openssl` |
| `fav/src/vm/builtins.rs` | 追加 | `AzurePostgres.execute_raw` / `query_raw` |
| `fav/src/middle/checker.rs` | 追加 | `AzurePostgres` namespace + `!AzureDb` エフェクト |
| `fav/src/lineage.rs` | 追加 | `AzureDbRead` / `AzureDbWrite` EffectKind |
| `runes/azure-postgres/client.fav` | 新規 | `AzureDbCtx` 型 + `execute` / `query` |
| `runes/azure-postgres/azure_postgres.fav` | 新規 | public API re-export |

---

## 6. 完了条件

| 確認項目 | 状態 |
|---|---|
| `AzurePostgres.execute_raw` / `query_raw` が VM に登録されている | |
| `fav check` で `!AzureDb` を宣言した fn が E0252 を出さない | |
| `fav explain --lineage` で AzureDb エフェクトが表示される | |
| `runes/azure-postgres/client.fav` が `fav check` でエラーなし | |
| `cargo test v141000` 全件パス（3/3） | |
| `cargo test` 全件パス（リグレッションなし） | |
| `CARGO_PKG_VERSION == "14.1.0"` | |

---

## 7. 設計上の注意点

- **`params` 引数の形式**: 既存 `Postgres.execute_raw(sql, params)` と引数順が異なる（`AzurePostgres.execute_raw(conn_str, sql, params)`）。既存コードとの互換性は不要（新しい NS のため）。
- **接続の1リクエスト1接続方式**: パフォーマンスより実装のシンプルさを優先。v14.x 移行後に connection pool を検討。
- **params の JSON 形式**: `tokio-postgres` の parameterized query では `$1, $2` プレースホルダーを使う。params の JSON 配列を Rust 側でパースして `tokio_postgres::types::ToSql` の Vec に変換する。
- **`with_transaction`**: 高度な closure 型（`fn(T) -> Result<U, String>` の Higher-order function）が必要。v14.x では scope 外とし、`client.fav` にコメントとして記載するのみ。
