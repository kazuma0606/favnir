# Favnir v13.2.0 Tasks

Date: 2026-06-09
Theme: DbRead / DbWrite / StorageRead / StorageWrite capability interface 実装

---

## Phase A — checker.rs: 組み込み capability interface 事前登録

- [x] A-1: `fav/src/middle/checker.rs` — `builtin_capability_interfaces()` 関数を追加
  - `DbRead`: `query`, `query1` → `(String, List<String>) -> Result<String, String>`
  - `DbWrite`: `execute` → `(String, List<String>) -> Result<Int, String>`
  - `StorageRead`: `get` → `(String, String) -> Result<String, String>`, `list` → `(String, String) -> Result<List<String>, String>`
  - `StorageWrite`: `put` → `(String, String, String) -> Result<Unit, String>`, `delete` → `(String, String) -> Result<Unit, String>`
- [x] A-2: `InterfaceRegistry::new()` 末尾で `builtin_capability_interfaces()` を呼び出して登録
- [x] A-3: `fav/src/driver.rs` — `get_help_text` に `"E0020"` エントリを追加
  ```rust
  "E0020" => &[
      "pass a value that implements the required capability interface",
      "available implementations: PostgresDb, SnowflakeDb, S3Storage, MockDb",
  ],
  ```
- [x] A-4: `fav/self/checker.fav` — `capability_interface_methods` 関数を追加
  - `DbRead` / `DbWrite` / `StorageRead` / `StorageWrite` のメソッド戻り値型を文字列で返す
- [x] A-5: `fav/self/compiler.fav` — capability interface 型名を型パース対象に追加

---

## Phase B — checker.rs: `ctx.db.query(...)` 型チェック + E0020

- [x] B-1: `fav/src/middle/checker.rs` — `infer_expr` の `Expr::Apply` に
  `Apply(FieldAccess(FieldAccess(base, field), method), args)` パターンを追加
  - `base` の型 → interface 型 → `field` の型 → interface 型 → `method` の型を lookup
  - 引数型チェック → 戻り値型を返す
- [x] B-2: `fav/src/middle/checker.rs` — E0020 エラーを emit する箇所を追加
  - capability interface フィールドに存在しないメソッドを呼んだ場合
  - 期待する interface を実装していない型を渡した場合
- [x] B-3: `fav check` で `ctx.db.query(sql, params)` が型エラーなしで通ることを手動確認
  ```
  interface WithDb { db: DbRead }
  public fn load(ctx: WithDb) -> Result<String, String> {
    bind rows <- ctx.db.query("SELECT 1", List.empty())
    Result.ok(rows)
  }
  ```

---

## Phase C — Rune 実装ファイルの追加

- [x] C-1: `fav/runes/postgres/postgres_db.fav` を作成
  - `type PostgresDb(String)`
  - `impl DbRead for PostgresDb`: `query` / `query1` → `Postgres.query_raw`
  - `impl DbWrite for PostgresDb`: `execute` → `Postgres.execute_raw`
  - `!Postgres` エフェクト宣言の要否を確認
- [x] C-2: `fav/runes/aws/s3_storage.fav` を作成
  - `type S3Storage(String)`
  - `impl StorageRead for S3Storage`: `get` → `AWS.s3_get_object_raw`, `list` → `AWS.s3_list_objects_raw`（存在確認必要）
  - `impl StorageWrite for S3Storage`: `put` → `AWS.s3_put_object_raw`, `delete` → `AWS.s3_delete_object_raw`（存在確認必要）
- [x] C-3: `fav/runes/aws/dynamo_db.fav` を作成（stub）
  - `type DynamoDb(String)`
  - `impl DbRead for DynamoDb`: `Result.err("not implemented")`
  - `impl DbWrite for DynamoDb`: `Result.err("not implemented")`
- [x] C-4: `fav/runes/snowflake/snowflake_db.fav` を作成
  - `type SnowflakeDb(String)`
  - `impl DbRead for SnowflakeDb`: `query` / `query1` → `Snowflake.query_raw`
  - `impl DbWrite for SnowflakeDb`: `execute` → `Snowflake.execute_raw`
- [x] C-5: `fav/runes/ctx/` ディレクトリを作成し `mock_db.fav` を追加
  - `type MockDb(List<String>)`
  - `fn MockDb.empty()` / `fn MockDb.seed(rows: List<String>)`
  - `impl DbRead for MockDb`: シードデータを JSON で返す
  - `impl DbWrite for MockDb`: `Result.ok(0)`
- [x] C-6: 全 Rune ファイルの `fav check` 検証
  ```bash
  ./target/debug/fav check runes/postgres/postgres_db.fav
  ./target/debug/fav check runes/aws/s3_storage.fav
  ./target/debug/fav check runes/snowflake/snowflake_db.fav
  ./target/debug/fav check runes/ctx/mock_db.fav
  ```

---

## Phase D — lint.rs: W009 deprecated 警告

- [x] D-1: `fav/src/lint.rs` — `DEPRECATED_RUNE_CALLS` 定数を定義
  - `("Postgres", "query_raw", "ctx.db.query(...)")`
  - `("Postgres", "execute_raw", "ctx.db.execute(...)")`
  - `("AWS", "s3_get_object_raw", "ctx.storage.get(...)")`
  - `("AWS", "s3_put_object_raw", "ctx.storage.put(...)")`
  - `("AWS", "s3_list_objects_raw", "ctx.storage.list(...)")`
  - `("Snowflake", "query_raw", "ctx.db.query(...)")`
  - `("Snowflake", "execute_raw", "ctx.db.execute(...)")`
- [x] D-2: `fav/src/lint.rs` — `check_deprecated_rune_calls(program: &Program) -> Vec<LintWarning>` を実装
  - `check_ambient_effects` と同様の AST walk
  - W009 の `LintWarning` を返す
- [x] D-3: `fav/src/driver.rs` — `cmd_check` の `ambient` ブロックで W009 も出力
  ```rust
  if ambient {
      let w008 = check_ambient_effects(&program);
      let w009 = check_deprecated_rune_calls(&program);
      // w008 + w009 を combined して出力
  }
  ```
- [x] D-4: `fav/src/driver.rs` — `get_help_text` に `"W009"` エントリを追加
  ```rust
  "W009" => &[
      "migrate to capability interface: `chain rows <- ctx.db.query(...)`",
      "direct Rune calls will be an error in v14.0",
  ],
  ```
- [x] D-5: `write_ambient_report` を W009 にも対応させる（W008 + W009 を Markdown に出力）

---

## Phase E — テスト追加

- [x] E-1: `fav/src/driver.rs` 末尾に `v132000_tests` モジュールを追加
  - `version_is_13_2_0` — `CARGO_PKG_VERSION == "13.2.0"`
  - `db_read_interface_registered` — `InterfaceRegistry` に `"DbRead"` が存在
  - `db_read_interface_type_check` — `ctx.db.query(...)` が型チェックを通る
  - `db_write_rejects_wrong_ctx` — `DbRead` に `execute` がない → E0020
  - `storage_write_put_type_check` — `ctx.store.put(bucket, key, body)` が通る
  - `w009_postgres_direct_deprecated` — `Postgres.query_raw(...)` + `--ambient` → W009
  - `w009_no_flag_no_warning` — `--ambient` なしでは W009 なし
- [x] E-2: `fav/Cargo.toml` — `version = "13.2.0"` に更新
- [x] E-3: 既存の `version_is_13_1_0` テストを comment out

---

## Phase F — ビルド・テスト・コミット

- [x] F-1: ビルド確認
  ```bash
  cd fav && cargo build
  ```
- [x] F-2: `cargo test` 全通過確認
- [x] F-3: self-check
  ```bash
  ./target/debug/fav check self/compiler.fav
  ./target/debug/fav check self/checker.fav
  ./target/debug/fav lint --deny-warnings self/compiler.fav
  ./target/debug/fav lint --deny-warnings self/checker.fav
  ./target/debug/fav fmt --check self/compiler.fav
  ./target/debug/fav fmt --check self/checker.fav
  ```
- [x] F-4: W009 件数確認
  ```bash
  ./target/debug/fav check --ambient self/compiler.fav
  ./target/debug/fav check --ambient self/checker.fav
  ```
- [x] F-5: `git add -p` で変更確認
- [x] F-6: `git commit -m "feat: v13.2.0 — DbRead/DbWrite/StorageRead/StorageWrite capability interface"`
- [x] F-7: `git push`
- [x] F-8: `gh run watch` で CI 全 green を確認

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `DbRead` / `DbWrite` / `StorageRead` / `StorageWrite` が `checker.rs` に事前登録される | ✅ |
| `ctx.db.query(sql, params)` が型チェックを通る | ✅ |
| E0020 が capability interface 型不一致で検出される | ✅ |
| `runes/postgres/postgres_db.fav` が `fav check` でエラーなし | ✅ |
| `runes/aws/s3_storage.fav` が `fav check` でエラーなし | ✅ |
| `runes/snowflake/snowflake_db.fav` が `fav check` でエラーなし | ✅ |
| `runes/ctx/mock_db.fav` が `fav check` でエラーなし | ✅ |
| `fav check --ambient` で W009 が出力される | ✅ |
| `fav lint --deny-warnings self/*.fav` → exit 0 | ✅ |
| `self/compiler.fav` / `self/checker.fav` が `fav check` でエラーなし | ✅（v13.3.0 で達成） |
| `CARGO_PKG_VERSION == "13.2.0"` | ✅ |
| `cargo test` 全通過 | ✅ |
| CI 全 green | ✅ |

---

## W009 件数（実装後に記録）

| ファイル | W009 件数 |
|---|---|
| self/compiler.fav | Postgres/Snowflake 直接呼び出しなし（IO は v13.3.0 で W009 対象追加） |
| self/checker.fav | 同上 |
| infra/e2e-demo/fav2py/src/pipeline.fav | Postgres 直接呼び出しあり |
| infra/e2e-demo/airgap/src/analyze.fav | AWS 直接呼び出しあり |
| **合計** | v13.3.0 で IO/Http も W009 対象に追加済み |
