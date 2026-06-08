# Favnir v12.6.0 実装計画

Date: 2026-06-08

---

## Phase A — 依存クレート追加

### A-1: `Cargo.toml` に TLS 依存を追加

`fav/Cargo.toml` に以下を追加:

```toml
tokio-postgres-rustls = "0.12"
rustls = { version = "0.23", features = ["ring"] }
webpki-roots = "0.26"
```

`cargo build` が通ることを確認する。

### A-2: `use` import 追加

`fav/src/backend/vm.rs` の先頭付近に追加:

```rust
use tokio_postgres_rustls::MakeRustlsConnect;
use rustls::{ClientConfig, RootCertStore};
```

---

## Phase B — エラー詳細化

### B-1: `format_pg_error` 関数を実装

vm.rs に standalone 関数として追加:

```rust
fn format_pg_error(e: &tokio_postgres::Error) -> String {
    if let Some(db_err) = e.as_db_error() {
        let mut msg = format!("db error: {}", db_err.message());
        if let Some(code) = db_err.code().code() {
            msg.push_str(&format!(" (SQLSTATE {})", code));
        }
        if let Some(detail) = db_err.detail() {
            msg.push_str(&format!(", detail: {}", detail));
        }
        msg
    } else {
        format!("db error: {}", e)
    }
}
```

### B-2: 既存の `.map_err(|e| e.to_string())` を `format_pg_error` に置換

vm.rs の `pg_connect`（2 箇所）の `.map_err(|e| e.to_string())?` を
`.map_err(|e| format_pg_error(&e))?` に変更。

execute_raw と query_raw の両方に適用する。

---

## Phase C — sslmode 実装

### C-1: `PostgresConfig` 構造体に `sslmode` を追加

`fav/src/driver.rs`（または `fav/src/config.rs`）の `PostgresConfig`:

```rust
#[derive(Deserialize, Default)]
pub struct PostgresConfig {
    pub url:     Option<String>,
    pub sslmode: Option<String>,
}
```

既存の構造体が存在しない場合は新規作成し、`FavToml` に追加:

```rust
pub struct FavToml {
    // ... 既存フィールド
    pub postgres: Option<PostgresConfig>,
}
```

### C-2: `resolve_sslmode` 関数を実装

以下の優先順位で `sslmode` を決定する関数:

```rust
fn resolve_sslmode(conn_str: &str, toml: &Option<PostgresConfig>) -> String {
    // 1. DATABASE_URL の ?sslmode= クエリパラメータ
    if let Some(pos) = conn_str.find("?sslmode=") {
        let rest = &conn_str[pos + 9..];
        let end = rest.find('&').unwrap_or(rest.len());
        return rest[..end].to_string();
    }
    // 2. fav.toml [postgres] sslmode
    if let Some(pg) = toml {
        if let Some(ref mode) = pg.sslmode {
            return mode.clone();
        }
    }
    // 3. PGSSLMODE 環境変数
    if let Ok(val) = std::env::var("PGSSLMODE") {
        return val;
    }
    // 4. デフォルト: prefer（TLS 試行）
    "prefer".to_string()
}
```

### C-3: `PgTls` enum と `make_tls_connector` を実装

vm.rs に以下を追加:

```rust
// tokio_postgres の MakeTlsConnect trait を両方の variant で満たすための wrapper
enum PgTls {
    Disable,
    Rustls(MakeRustlsConnect),
}
```

`MakeTlsConnect` trait を `PgTls` に実装するか、
または `connect` 呼び出しを分岐させる（simpler）:

```rust
async fn pg_connect_inner(conn_str: &str, sslmode: &str)
    -> Result<tokio_postgres::Client, String>
{
    match sslmode {
        "disable" => {
            let (client, conn) =
                tokio_postgres::connect(conn_str, tokio_postgres::NoTls)
                    .await.map_err(|e| format_pg_error(&e))?;
            tokio::spawn(async move { let _ = conn.await; });
            Ok(client)
        }
        _ => {
            let mut root_store = RootCertStore::empty();
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            let config = Arc::new(
                ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_no_client_auth(),
            );
            let tls = MakeRustlsConnect::new(config);
            let (client, conn) =
                tokio_postgres::connect(conn_str, tls)
                    .await.map_err(|e| format_pg_error(&e))?;
            tokio::spawn(async move { let _ = conn.await; });
            Ok(client)
        }
    }
}
```

### C-4: 既存の `pg_connect` 2 箇所を `pg_connect_inner` で置換

`pg_execute_raw` / `pg_query_raw` の両ハンドラで:

```rust
// 変更前
let (client, connection) =
    tokio_postgres::connect(conn_str, tokio_postgres::NoTls)
        .await.map_err(|e| e.to_string())?;
tokio::spawn(async move { let _ = connection.await; });

// 変更後
let client = pg_connect_inner(conn_str, &sslmode).await?;
```

`conn_str` と `sslmode` は各ハンドラで取得しておく。

---

## Phase D — fav.toml パース更新

### D-1: `FavToml` に `postgres` フィールドを追加（未存在の場合）

`load_run_config` または `parse_fav_toml` を読んで `postgres` セクションが
まだない場合のみ追加。既にある場合は `sslmode` フィールドのみ追加。

### D-2: `inject_postgres_config` 関数（オプション）

Snowflake と同様に、`fav.toml [postgres]` の値を env var に展開する
`inject_postgres_config` を実装してもよい。ただし `DATABASE_URL` は
ほとんどの場合すでに env var 経由で提供されるため、
本バージョンでは `sslmode` のパースに留める。

---

## Phase E — テスト追加

`fav/src/driver.rs` の `v12600_tests` モジュールに追加:

```rust
#[cfg(test)]
mod v12600_tests {
    fn postgres_sslmode_from_url()         { ... }
    fn postgres_sslmode_from_toml()        { ... }
    fn postgres_sslmode_disable_uses_notls() { ... }
    fn postgres_error_includes_message()   { ... }
    fn postgres_error_includes_sqlstate()  { ... }
    fn format_pg_error_non_db_error()      { ... }
    fn version_is_12_6_0()                 { ... }
}
```

`postgres_sslmode_disable_uses_notls` は実際の DB 接続なしにテスト可能
（ハンドラの分岐ロジックを直接テスト）。

`postgres_error_includes_message` / `postgres_error_includes_sqlstate` は
`tokio_postgres::Error` をモックするか、または
`format_pg_error` の出力文字列を単体テストする形で実装。

---

## Phase F — バージョン更新・コミット

- `fav/Cargo.toml` version → `"12.6.0"`
- `cargo test` 全通過確認
- `git commit -m "feat: v12.6.0 — Postgres TLS (rustls) + error detail"`
- `git push`

---

## 実装上の注意

### 1. `Arc` の import

`pg_connect_inner` で `Arc::new(config)` を使うため:

```rust
use std::sync::Arc;
```

vm.rs にすでに `Arc` が import されているか確認する。

### 2. `prefer` の実装

sslmode=`prefer` は「TLS 試行、失敗なら NoTls フォールバック」が本来の動作だが、
TLS エラーと「接続そのものの失敗」を区別する処理が複雑。
初版では `prefer` = `require`（TLS 必須）として扱い、
spec の非目標に明記する。

### 3. tokio-postgres-rustls のバージョン

`tokio-postgres = "0.7"` に対応するバージョンを確認する。
`tokio-postgres-rustls = "0.12"` が 0.7 系対応の最新版。
`cargo add` 後に `cargo build` でバージョン整合を確認。

### 4. webpki-roots の証明書

`webpki_roots::TLS_SERVER_ROOTS` は Mozilla の CA bundle。
RDS は Amazon Root CA を使用しており、bundle に含まれている。
カスタム CA は本バージョンのスコープ外。

### 5. 既存テストへのリグレッション

`pg_execute_raw` / `pg_query_raw` のハンドラを変更するため、
既存の Postgres 関連テストが通ることを確認する。
sslmode がない場合のデフォルト（`prefer`）で NoTls フォールバックが
動作することが重要。
