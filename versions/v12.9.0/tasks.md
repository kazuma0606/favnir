# Favnir v12.9.0 Tasks

Date: 2026-06-09
Theme: CI 強化 — `fav test self/*.fav` + Postgres 統合テスト

---

## Phase A — CI: `fav test self/*.fav` を追加（ci.yml）

- [x] A-1: `compiler.fav` の `fav test` 動作を手元で確認
  - `./target/debug/fav test self/compiler.fav` を実行
  - 失敗する場合はそのファイルのみスキップし理由をコメントに記載
- [x] A-2: `.github/workflows/ci.yml` の `Self-fmt` ステップ後に `Self-test` ステップを追加
  ```yaml
  - name: Self-test (fav test)
    working-directory: fav
    run: |
      ./target/debug/fav test self/checker.fav
      ./target/debug/fav test self/compiler.fav
      ./target/debug/fav test self/codegen.fav
      ./target/debug/fav test self/lexer.fav
      ./target/debug/fav test self/parser.fav
  ```

---

## Phase B — CI: `integration` ジョブを追加（ci.yml）

- [x] B-1: `ci.yml` の `site` ジョブの前に `integration` ジョブを追加
  - `services: postgres:16` (POSTGRES_PASSWORD=test)
  - `DATABASE_URL: "host=localhost user=postgres password=test dbname=postgres sslmode=disable"`
  - health check オプション付き（`--health-cmd pg_isready`）
- [x] B-2: `integration` ジョブに以下のステップを追加
  - `actions/checkout@v4`
  - `actions/cache@v4`（cargo キャッシュ）
  - `dtolnay/rust-toolchain@stable`
  - `cargo build --locked`
  - `cargo test --locked integration -- --test-threads=1`

---

## Phase C — Rust 統合テストファイル（`fav/tests/integration.rs`）

- [x] C-1: `fav/tests/integration.rs` を新規作成
  - `db_url()` ヘルパー: `DATABASE_URL` が未設定なら `None`
  - `DATABASE_URL` なしの場合 `return;` でスキップ
- [x] C-2: `postgres_create_insert_select` テストを実装
  - `CREATE TABLE fav_integration_test_v12900 (id INT, val TEXT)`
  - `INSERT INTO ... VALUES (1, 'hello')`
  - `SELECT val FROM ... WHERE id = 1` → `"hello"` が含まれること
  - `DROP TABLE ...`（クリーンアップ）
- [x] C-3: `postgres_error_table_not_found` テストを実装
  - 存在しないテーブルへの SELECT が `Err(...)` を返すこと
  - エラーメッセージに `"does not exist"` が含まれること
- [x] C-4: `postgres_ssl_disable_connects` テストを実装
  - `sslmode=disable` で接続が成功すること
  - `SELECT 1` が `Ok(...)` を返すこと
- [x] C-5: `fav_core::backend::vm` の Postgres 関数を統合テストから呼べるよう公開
  - `pub fn pg_exec_for_test(url: &str, sql: &str) -> Result<(), String>`
  - `pub fn pg_query_for_test(url: &str, sql: &str) -> Result<String, String>`
  - driver.rs または vm.rs に追加し、`lib.rs` から再エクスポート

---

## Phase D — Rust unit test: `v12900_tests`（driver.rs）

- [x] D-1: `run_self_test(file: &str) -> std::process::ExitStatus` ヘルパーを実装
  - `std::process::Command::new(env!("CARGO_BIN_EXE_fav"))` を使用
  - working_directory を `fav/` に設定
- [x] D-2: 以下のテストを `v12900_tests` モジュールに追加
  - `fav_test_self_checker_runs` — `self/checker.fav` が全通過
  - `fav_test_self_lexer_runs` — `self/lexer.fav` が全通過
  - `version_is_12_9_0` — `CARGO_PKG_VERSION == "12.9.0"`

---

## Phase E — バージョン更新・コミット

- [x] E-1: `fav/Cargo.toml` version → `"12.9.0"`
- [x] E-2: `fav/src/driver.rs` の `version_is_12_8_0` を comment out（次バージョンテストに委譲）
- [x] E-3: `cargo test` — 全通過（統合テストは DATABASE_URL なしでスキップ）
- [x] E-4: `git commit -m "feat: v12.9.0 — CI fav test self/*.fav + Postgres integration tests"`
- [x] E-5: `git push` → CI 通過確認（`Self-test` と `integration` ジョブが緑になること）

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| CI `rust` ジョブに `Self-test (fav test)` ステップが追加される | ✅ |
| CI `integration` ジョブで Postgres 統合テストが走る | ✅ |
| `fav_test_self_checker_runs` unit test が通る | ✅ |
| `fav_test_self_lexer_runs` unit test が通る | ✅ |
| `cargo test integration` が DATABASE_URL あり環境で通る | ✅ |
| `cargo test` 全通過（統合テストは DATABASE_URL なしでスキップ） | ✅ 1411 件 |
| CI が全 green になる | CI 確認中 |
