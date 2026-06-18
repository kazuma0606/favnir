# Favnir v12.6.0 仕様書

Date: 2026-06-08
Theme: Postgres Rune TLS 対応 + エラー詳細化

---

## 概要

v12.5.0 でデバッグ可視性が整った。
v12.6.0 は RDS PostgreSQL 16（`rds.force_ssl=1` デフォルト）をはじめとする
SSL 必須環境で Postgres Rune が正常動作することを目的とする。

現状の `tokio_postgres::NoTls` のみの実装では、本番 RDS に接続した瞬間に
"db error" で失敗し、しかもそのエラー文字列に原因が含まれない。
本バージョンでこの 2 つの問題を同時に解消する。

---

## 問題の再確認

### 問題 1: TLS 非対応

`fav/src/backend/vm.rs` の `pg_connect` 呼び出し（2 箇所）が `NoTls` のみ:

```rust
tokio_postgres::connect(conn_str, tokio_postgres::NoTls).await
    .map_err(|e| e.to_string())?
```

RDS PostgreSQL 16 はデフォルト `rds.force_ssl = 1`。
この状態では接続確立時点で失敗し、パイプラインが動かない。

### 問題 2: エラー詳細が失われる

`tokio_postgres::Error::to_string()` は "db error" のみを返す。
`DbError::message()` / `code()` / `detail()` の情報が捨てられている。

```
before: "db error"
after:  "db error: SSL connection is required (SQLSTATE 08P01, detail: ...)"
```

AI にとって "db error" は診断不能。詳細情報が必須。

---

## 機能 1: `sslmode` による TLS 切り替え

### TLS ライブラリの選択

`tokio-postgres-rustls` を採用する（理由: OpenSSL 不要、Linux/Windows 共通ビルド）。

必要な追加依存:
```toml
# Cargo.toml
tokio-postgres-rustls = "0.12"      # tokio-postgres 0.7 対応版
rustls = { version = "0.23", features = ["ring"] }
webpki-roots = "0.26"
```

### `sslmode` の読み取り順序

1. `DATABASE_URL` のクエリパラメータ: `?sslmode=require`
2. `fav.toml` の `[postgres] sslmode = "require"`
3. 環境変数 `PGSSLMODE`
4. デフォルト: `"prefer"`（TLS 試行、失敗なら NoTls フォールバック）

### sslmode の動作定義

| sslmode | 動作 |
|---|---|
| `disable` | `NoTls`（既存動作）|
| `prefer` | TLS 試行、失敗（接続拒否）なら `NoTls` で再試行 |
| `require` | TLS 必須、失敗なら即エラー |
| `verify-ca` / `verify-full` | 本バージョンのスコープ外（`require` と同等に扱う）|

### pg_connect の変更

```rust
// 変更前
let (client, connection) =
    tokio_postgres::connect(conn_str, tokio_postgres::NoTls).await
        .map_err(|e| format_pg_error(&e))?;

// 変更後
let tls = make_tls_connector(sslmode);
let (client, connection) =
    tokio_postgres::connect(conn_str, tls).await
        .map_err(|e| format_pg_error(&e))?;
```

### `make_tls_connector` 関数

```rust
enum PgTls {
    Disable(NoTls),
    Rustls(MakeRustlsConnect),
}

impl tokio_postgres::tls::MakeTlsConnect<...> for PgTls { ... }

fn make_tls_connector(sslmode: &str) -> PgTls {
    match sslmode {
        "disable" => PgTls::Disable(NoTls),
        _ => {
            let mut root_store = RootCertStore::empty();
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            let config = Arc::new(
                ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_no_client_auth()
            );
            PgTls::Rustls(MakeRustlsConnect::new(config))
        }
    }
}
```

---

## 機能 2: エラー詳細化

### `format_pg_error` 関数

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

### 出力例

```
// RDS force_ssl=1 の場合
before: "db error"
after:  "db error: SSL connection is required (SQLSTATE 08P01)"

// テーブルが存在しない場合
before: "db error"
after:  "db error: relation \"txn_rows\" does not exist (SQLSTATE 42P01)"

// 型不一致の場合
before: "db error"
after:  "db error: invalid input syntax for type json (SQLSTATE 22P02), detail: ..."
```

---

## 機能 3: `fav.toml [postgres]` セクション拡充

```toml
[postgres]
url     = "postgresql://user:pass@host:5432/db"
sslmode = "require"    # disable / prefer / require
```

`sslmode` キーを `FavToml.postgres: Option<PostgresConfig>` に追加:

```rust
#[derive(Deserialize, Default)]
pub struct PostgresConfig {
    pub url:     Option<String>,
    pub sslmode: Option<String>,
}
```

---

## 機能 4: Terraform テンプレート更新

`fav new --template postgres-etl` で生成される Terraform に
`aws_db_parameter_group` を標準同梱する。

```hcl
resource "aws_db_parameter_group" "pg" {
  family = "postgres16"
  name   = "${var.name}-pg16"

  parameter {
    name  = "rds.force_ssl"
    value = "0"
  }
}

resource "aws_db_instance" "db" {
  # ...
  parameter_group_name = aws_db_parameter_group.pg.name
}
```

開発環境（`sslmode=disable`）でも本番環境（`sslmode=require`）でも
明示的に設定を書かせることで「暗黙の失敗」を防ぐ。

---

## テストケース

| テスト名 | 内容 |
|---|---|
| `postgres_sslmode_from_url` | `?sslmode=require` を DATABASE_URL から読む |
| `postgres_sslmode_from_toml` | `fav.toml [postgres] sslmode` を読む |
| `postgres_sslmode_disable_uses_notls` | `sslmode=disable` → `NoTls` パスを通る |
| `postgres_error_includes_message` | `DbError.message()` がエラー文字列に含まれる |
| `postgres_error_includes_sqlstate` | `SQLSTATE XXXXX` がエラー文字列に含まれる |
| `format_pg_error_non_db_error` | DB 以外のエラー（接続拒否等）も適切に文字列化 |
| `version_is_12_6_0` | `CARGO_PKG_VERSION == "12.6.0"` |

---

## 完了条件

- [ ] `sslmode=require` 指定で TLS ハンドシェイクが通る（ローカル SSL Postgres or モック）
- [ ] `sslmode=disable` 指定で既存の NoTls 接続が通る（リグレッションなし）
- [ ] エラー文字列に `DbError.message()` と SQLSTATE が含まれる
- [ ] `DATABASE_URL?sslmode=require` パースが通る
- [ ] `fav.toml [postgres] sslmode` が読まれる
- [ ] `cargo test` 全通過

---

## 非目標

- クライアント証明書認証（`verify-ca` / `verify-full` の厳密な実装）
- Snowflake Rune の TLS 対応（Snowflake は HTTP ベースのため別問題）
- 接続プーリング（本バージョンのスコープ外）
- `sslmode=prefer` の「TLS 失敗 → NoTls フォールバック」の完全実装（接続確立エラーを区別する必要があり複雑。初版は `require` と同等でよい）
