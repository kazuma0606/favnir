# Favnir v12.9.0 仕様書

Date: 2026-06-09
Theme: CI 強化 — `fav test self/*.fav` + Postgres 統合テスト

---

## 概要

v12.8.0 で AI フレンドリー強化フェーズ（scaffold）が完了した。
v12.9.0 は「CI の構造的ギャップを埋める」フェーズ。

### 問題

1. **Fav レベルのテストが CI で実行されていない**
   `fav test` コマンドは v9.10.0 から実装済み。
   `self/` 下に 81 件のテストブロックが存在する（checker.fav: 47、compiler.fav: 12、
   codegen.fav: 8、lexer.fav: 8、parser.fav: 6）が、
   CI の self-check ステップでは `fav check / fav lint / fav fmt --check` のみで
   **`fav test self/*.fav` が CI で実行されていない**。

2. **Postgres Rune の統合テストがない**
   v12.6.0 で TLS 対応・エラー詳細化を実装したが、
   実際の Postgres に接続する統合テストは Rust unit test レベルでは走っていない。
   CI で docker-compose（または GitHub Actions の `services:` postgres）を使った
   統合テストを追加することで、Rune レベルの動作を保証する。

---

## 機能 1: CI に `fav test self/*.fav` を追加

### 対象ファイル

`.github/workflows/ci.yml` の `Self-check` / `Self-lint` ステップの後に追加:

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

### 備考

- `compiler.fav` のテストは `--legacy-check` が必要かどうか確認する
  （`fav test` は `fav check` より緩い: 実行のみでよい）
- テスト実行が失敗（exit 1）した場合は CI を止める
- `self/cli.fav` はテストブロックがないためスキップ

---

## 機能 2: Postgres 統合テスト（CI）

### GitHub Actions `services:` で Postgres を起動

`.github/workflows/ci.yml` に `integration` ジョブを追加:

```yaml
integration:
  name: Integration — Postgres
  needs: changes
  if: needs.changes.outputs.rust == 'true'
  runs-on: ubuntu-latest
  env:
    CXXFLAGS: ""
    DATABASE_URL: "host=localhost user=postgres password=test dbname=postgres sslmode=disable"
  services:
    postgres:
      image: postgres:16
      env:
        POSTGRES_PASSWORD: test
      ports:
        - 5432:5432
      options: >-
        --health-cmd pg_isready
        --health-interval 10s
        --health-timeout 5s
        --health-retries 5
  steps:
    - uses: actions/checkout@v4
    - name: Cache cargo registry
      uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          fav/target
        key: ${{ runner.os }}-cargo-${{ hashFiles('fav/Cargo.lock') }}
        restore-keys: ${{ runner.os }}-cargo-
    - name: Install Rust stable
      uses: dtolnay/rust-toolchain@stable
    - name: Build
      working-directory: fav
      run: cargo build --locked
    - name: Integration tests
      working-directory: fav
      run: cargo test --locked integration -- --test-threads=1
```

### 統合テストの実装

`fav/tests/integration.rs`（Rust integration test ファイル）:

```rust
// 実行条件: DATABASE_URL 環境変数が設定されている場合のみ
// cargo test integration -- --test-threads=1

#[test]
fn postgres_create_insert_select() {
    // CREATE TABLE / INSERT / SELECT / DROP TABLE が通ること
}

#[test]
fn postgres_error_table_not_found() {
    // 存在しないテーブルへの SELECT が Err("...") を返すこと
}

#[test]
fn postgres_ssl_disable_connects() {
    // sslmode=disable で接続できること
}
```

`DATABASE_URL` が設定されていない場合はテストをスキップ（`return;` で早期終了）。

---

## 機能 3: Rust unit test — `fav_test_self_*`

CI ではなく `cargo test` レベルで self-test の動作を保証するテストを追加。

`driver.rs` の `v12900_tests` モジュール:

```rust
fn fav_test_self_checker_runs()  { ... }  // cmd_test("self/checker.fav") が exit 0
fn fav_test_self_lexer_runs()    { ... }  // cmd_test("self/lexer.fav") が exit 0
fn version_is_12_9_0()           { ... }
```

---

## テストケース

| テスト名 | 内容 |
|---|---|
| `fav_test_self_checker_runs` | `fav test self/checker.fav` が全テスト通過 |
| `fav_test_self_lexer_runs` | `fav test self/lexer.fav` が全テスト通過 |
| `postgres_create_insert_select` | Postgres に CREATE/INSERT/SELECT/DROP が通ること（統合テスト） |
| `postgres_error_table_not_found` | 存在しないテーブルへの SELECT が Err を返すこと（統合テスト） |
| `postgres_ssl_disable_connects` | sslmode=disable で接続できること（統合テスト） |
| `version_is_12_9_0` | `CARGO_PKG_VERSION == "12.9.0"` |

---

## 完了条件

- [ ] CI の `rust` ジョブに `Self-test (fav test)` ステップが追加される
- [ ] CI の `integration` ジョブで Postgres 統合テストが走る
- [ ] `cargo test` の unit test に `fav_test_self_checker_runs` 等が追加される
- [ ] `cargo test integration` で Postgres 統合テストが通る（環境変数あり時）
- [ ] 全 `cargo test` 通過
- [ ] `cargo test` 全通過

---

## 非目標

- Snowflake 統合テスト（本番 Snowflake アカウントが必要）
- fav test コマンド自体の変更（v9.10.0 で実装済み）
- docker-compose.yml の追加（GitHub Actions の `services:` で十分）
- Windows ローカル環境での Postgres テスト自動化
