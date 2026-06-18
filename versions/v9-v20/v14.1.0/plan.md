# v14.1.0 Plan — Azure PostgreSQL Rune

Date: 2026-06-11

---

## Phase A — Cargo.toml 依存追加

**ファイル**: `fav/Cargo.toml`

`openssl` と `tokio-postgres-openssl` を追加する:

```toml
[dependencies]
# ... 既存 ...
openssl = { version = "0.10", features = ["v102", "v110"] }
tokio-postgres-openssl = "0.5"
```

> Windows での開発時は `OPENSSL_DIR` 環境変数を設定する（例: vcpkg でインストールした openssl のパス）。

`cargo build` でコンパイルエラーなし確認。

---

## Phase B — VM プリミティブ追加

**ファイル**: `fav/src/vm/builtins.rs`

既存の `"Postgres" =>` ブロックの近くに `"AzurePostgres" =>` ブロックを追加する。

### B-1: `execute_raw`

SQL を実行し、変更行数を `Result<Int, String>` で返す:

```rust
"AzurePostgres" => match method {
    "execute_raw" => {
        // args: [conn_str: String, sql: String, params_json: String]
        let conn_str   = args[0].as_string().map_err(|e| format!("AzurePostgres.execute_raw: {}", e))?;
        let sql        = args[1].as_string().map_err(|e| format!("AzurePostgres.execute_raw: {}", e))?;
        let params_json = args[2].as_string().map_err(|e| format!("AzurePostgres.execute_raw: {}", e))?;

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("AzurePostgres.execute_raw: tokio init failed: {}", e))?;

        rt.block_on(async {
            use openssl::ssl::{SslConnector, SslMethod};
            use tokio_postgres_openssl::MakeTlsConnector;

            let mut builder = SslConnector::builder(SslMethod::tls())
                .map_err(|e| format!("AzurePostgres.execute_raw: TLS builder: {}", e))?;
            builder.set_verify(openssl::ssl::SslVerifyMode::NONE); // 開発用; 本番は CA 検証
            let connector = MakeTlsConnector::new(builder.build());

            let (client, connection) = tokio_postgres::connect(&conn_str, connector)
                .await
                .map_err(|e| format!("AzurePostgres.execute_raw: connect: {}", e))?;

            tokio::spawn(async move { let _ = connection.await; });

            let params = parse_params_json(&params_json)
                .map_err(|e| format!("AzurePostgres.execute_raw: params: {}", e))?;
            let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
                params.iter().map(|p| p as _).collect();

            let n = client
                .execute(&sql, &param_refs)
                .await
                .map_err(|e| format!("AzurePostgres.execute_raw: {}", e))?;

            Ok(Value::Int(n as i64))
        })
        .map_err(|e: String| e)
        .map(|v| v)
    }
    // ...
}
```

### B-2: `query_raw`

SELECT を実行し、行を JSON 配列文字列で返す:

```rust
    "query_raw" => {
        // args: [conn_str: String, sql: String, params_json: String]
        // 戻り値: JSON 配列文字列（各行は { "col": value } オブジェクト）
        // 実装パターンは execute_raw と同様。
        // rows を serde_json::Value の Vec に変換し、JSON 配列文字列として返す。
        // カラム名: row.columns()[i].name()
        // 値: row.get::<_, Option<String>>(i) でまず String として取り出す
    }
```

### B-3: `parse_params_json` ヘルパー

JSON 配列文字列を `Vec<serde_json::Value>` にパースし、各要素を `tokio_postgres::types::ToSql` として返せる形式に変換するヘルパー関数を追加:

```rust
fn parse_params_json(json: &str) -> Result<Vec<serde_json::Value>, String> {
    serde_json::from_str::<serde_json::Value>(json)
        .map_err(|e| format!("invalid params JSON: {}", e))
        .and_then(|v| {
            v.as_array()
                .map(|a| a.to_vec())
                .ok_or_else(|| "params must be a JSON array".to_string())
        })
}
```

> `serde_json::Value` は `tokio_postgres::types::ToSql` を実装しているため直接使用できる。

`cargo test` でコンパイルエラーなし確認。

---

## Phase C — checker.rs: `AzurePostgres` namespace 追加

**ファイル**: `fav/src/middle/checker.rs`

### C-1: `builtin_ret_ty` に追加

既存の `"Postgres" =>` ブロックの近くに追加:

```rust
"AzurePostgres" => match method {
    "execute_raw" => Type::Result(Box::new(Type::Int), Box::new(s())),
    "query_raw"   => Type::Result(Box::new(s()),       Box::new(s())),
    _ => Type::Unknown,
},
```

### C-2: `BUILTIN_EFFECTS` に `"AzureDb"` を追加

既存の `"Postgres"` / `"Snowflake"` の近くに追加:

```rust
"AzureDb" => true,
```

### C-3: `ns_to_effect`（または対応する関数）に追加

`AzurePostgres` namespace が `!AzureDb` エフェクトを要求することをチェッカーに伝える:

```rust
"AzurePostgres" => Some("AzureDb"),
```

`cargo test` でリグレッションなし確認。

---

## Phase D — lineage.rs: `AzureDb` エフェクト追加

**ファイル**: `fav/src/lineage.rs`

### D-1: `EffectKind` enum に追加

既存の `PostgresRead` / `PostgresWrite` の近くに追加:

```rust
AzureDbRead,
AzureDbWrite,
```

### D-2: `collect_call_kinds` のマッピング追加

```rust
("AzurePostgres", "query_raw")   => Some(EffectKind::AzureDbRead),
("AzurePostgres", "execute_raw") => Some(EffectKind::AzureDbWrite),
```

### D-3: `effect_kind_label`（または表示用関数）に追加

```rust
EffectKind::AzureDbRead  => "!AzureDb(read)",
EffectKind::AzureDbWrite => "!AzureDb(write)",
```

`cargo test` でリグレッションなし確認。

---

## Phase E — `runes/azure-postgres/` 新規作成

### E-1: `runes/azure-postgres/client.fav`

```
// runes/azure-postgres/client.fav — Azure DB for PostgreSQL クライアント (v14.1.0)
// AzurePostgres VM primitive への型付きラッパー。
//
// 接続文字列形式:
//   postgresql://user:password@host:5432/db?sslmode=require
//
// 使用例:
//   let ctx = azure_postgres.new_ctx(conn_str)
//   bind rows <- azure_postgres.query<Row>(ctx, "SELECT * FROM t", "[]")

// AzureDbCtx — Azure DB for PostgreSQL 接続コンテキスト（接続文字列ラッパー）
// v14.2.0 で AzureCtx.db フィールドとして組み込む予定
type AzureDbCtx(String)

// new_ctx — 接続文字列から AzureDbCtx を作成
public fn new_ctx(conn_str: String) -> AzureDbCtx {
    AzureDbCtx(conn_str)
}

// execute — DML（INSERT / UPDATE / DELETE / CREATE 等）を実行する
// params: JSON 配列文字列（例: "[\"val1\", 42]"）
// 戻り値: 変更行数（Ok(n)）または Err(message)
public fn execute(ctx: AzureDbCtx, sql: String, params: String) -> Result<Int, String> !AzureDb {
    match ctx {
        AzureDbCtx(conn_str) => AzurePostgres.execute_raw(conn_str, sql, params)
    }
}

// query<T> — SELECT クエリを実行し、行を型 T の List に変換して返す
// params: JSON 配列文字列（例: "[\"val1\"]"）
public fn query<T>(ctx: AzureDbCtx, sql: String, params: String) -> Result<List<T>, String> !AzureDb {
    match ctx {
        AzureDbCtx(conn_str) =>
            match AzurePostgres.query_raw(conn_str, sql, params) {
                Err(e) => Result.err(e)
                Ok(raw) =>
                    match Json.parse_raw(raw) {
                        Err(e) => Result.err(String.concat("azure_postgres.query: ", e))
                        Ok(parsed) =>
                            match Schema.adapt(parsed, type_name_of<T>()) {
                                Err(_) => Result.err("azure_postgres.query: schema error")
                                Ok(rows) => Result.ok(rows)
                            }
                    }
            }
    }
}

// with_transaction は v14.x 以降で追加予定（higher-order fn サポート後）
// fn with_transaction(ctx: AzureDbCtx, f: fn(AzureDbCtx) -> Result<T, String>) -> Result<T, String>
```

### E-2: `runes/azure-postgres/azure_postgres.fav`

```
// runes/azure-postgres/azure_postgres.fav — Azure PostgreSQL Rune public API (v14.1.0)
// Azure DB for PostgreSQL に接続し、SQL を実行・クエリする。
//
// 使用例:
//   import rune "azure-postgres"
//
//   type Row = { id: Int  name: String  amount: Float }
//
//   fn get_rows(conn_str: String) -> Result<List<Row>, String> !AzureDb {
//       let ctx = azure_postgres.new_ctx(conn_str)
//       azure_postgres.query<Row>(ctx, "SELECT * FROM rows", "[]")
//   }
//
// 接続文字列形式:
//   postgresql://user:password@host:5432/db?sslmode=require

use client.{ AzureDbCtx, new_ctx, execute, query }
```

### E-3: 動作確認

```bash
cd fav
cargo build
echo 'import rune "azure-postgres"\nfn main() -> Unit { IO.println("ok") }' > /tmp/test_azure.fav
./target/debug/fav check /tmp/test_azure.fav
```

---

## Phase F — テスト追加

**ファイル**: `fav/src/driver.rs`

`v141000_tests` モジュールを追加:

```rust
#[cfg(test)]
mod v141000_tests {
    use super::*;

    #[test]
    fn version_is_14_1_0() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "14.1.0");
    }

    #[test]
    fn azure_postgres_primitives_registered() {
        // AzurePostgres.execute_raw / query_raw が VM に登録されていることを確認
        // build_artifact でシンプルな fav ファイルをコンパイルし、
        // AzurePostgres.execute_raw 呼び出しが E0007（未定義関数）を出さないことを確認
        let src = r#"
public fn main() -> Unit {
    AzurePostgres.execute_raw("conn", "SELECT 1", "[]")
}
"#;
        let errors = check_source_raw(src);
        let azure_errors: Vec<_> = errors.iter()
            .filter(|e| e.code == "E0007" && e.message.contains("AzurePostgres"))
            .collect();
        assert!(azure_errors.is_empty(),
            "AzurePostgres primitives not recognized: {:?}", azure_errors);
    }

    #[test]
    fn azure_db_effect_in_checker() {
        // !AzureDb を宣言した fn が E0252（未知エフェクト）を出さないことを確認
        let src = r#"
fn fetch_rows(conn_str: String) -> Result<String, String> !AzureDb {
    AzurePostgres.query_raw(conn_str, "SELECT 1", "[]")
}
public fn main() -> Unit { IO.println("ok") }
"#;
        let errors = check_source_raw(src);
        let effect_errors: Vec<_> = errors.iter()
            .filter(|e| e.code == "E0252")
            .collect();
        assert!(effect_errors.is_empty(),
            "!AzureDb caused E0252: {:?}", effect_errors);
    }

    #[test]
    fn azure_db_lineage_tracked() {
        // AzurePostgres.execute_raw が AzureDbWrite として lineage に収集されることを確認
        let src = r#"
fn write_row(conn_str: String) -> Result<Int, String> !AzureDb {
    AzurePostgres.execute_raw(conn_str, "INSERT INTO t VALUES($1)", "[\"x\"]")
}
public fn main() -> Unit { IO.println("ok") }
"#;
        let lineage = collect_lineage_raw(src);
        assert!(
            lineage.iter().any(|e| format!("{:?}", e).contains("AzureDbWrite")),
            "AzureDbWrite not found in lineage: {:?}", lineage
        );
    }
}
```

> `check_source_raw` / `collect_lineage_raw` は既存のテストヘルパー関数を使う。関数名が異なる場合は既存テストを参照して合わせる。

`cargo test v141000` 全件パス（3/3）確認。

---

## Phase G — バージョンバンプ + 全テスト + コミット

1. `fav/Cargo.toml` → `version = "14.1.0"`
2. `cargo test v141000` 全件パス確認（3/3）
3. `cargo test` 全件パス確認（リグレッションなし）
4. `git commit -m "feat: v14.1.0 — Azure PostgreSQL Rune"`

---

## 実装順序

```
A (Cargo.toml) ← 最初に実施（B が依存）
B (builtins.rs) ← A 完了後
C (checker.rs) ← B と並行可
D (lineage.rs) ← C と並行可
E (runes/) ← B/C 完了後（型チェックが通るか確認するため）
F (tests) ← B/C/D 完了後
G (bump+commit) ← 全フェーズ完了後
```

---

## リスク・注意点

1. **`tokio-postgres-openssl` の Windows ビルド**: `OPENSSL_DIR` が未設定だとリンクエラーになる。`OPENSSL_DIR` を vcpkg のパスに設定するか、CI では Linux ランナーを使う。代替として `tokio-postgres-native-tls`（Windows 対応が容易）への切り替えも可能。

2. **`serde_json::Value` の `ToSql` 実装**: `tokio-postgres` の `with-serde_json-1` feature が有効でないと `ToSql` が実装されない。`Cargo.toml` の `tokio-postgres` feature に `"with-serde_json-1"` が含まれていることを確認する。

3. **`SslVerifyMode::NONE` の本番利用**: 開発・デモ用途では証明書検証をスキップしても許容されるが、本番では `set_ca_file` で Azure の CA 証明書を指定するべき。v15.0.0 の E2E デモでは環境変数で切り替え可能にする。

4. **`parse_params_json` の型変換**: `serde_json::Value` の各要素を `$1, $2` プレースホルダーへマッピングする際、Number を `i64` / `f64` に変換し、String はそのまま渡す。Null は `None::<String>` に変換する。

5. **既存 `!Postgres` テストへの影響なし**: `AzurePostgres` は独立した namespace のため、既存の `!Postgres` エフェクト検査・lineage 収集には影響しない。
