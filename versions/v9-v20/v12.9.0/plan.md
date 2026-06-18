# Favnir v12.9.0 実装計画

Date: 2026-06-09

---

## Phase A — CI: `fav test self/*.fav` を追加（ci.yml）

### A-1: 対象ファイルと実行コマンドの確認

`self/` 下でテストブロックが存在するファイル:
- `checker.fav` (47 tests)
- `compiler.fav` (12 tests)
- `codegen.fav` (8 tests)
- `lexer.fav` (8 tests)
- `parser.fav` (6 tests)

`cli.fav` はテストブロックなし → スキップ。

`fav test <file>` は既実装（v9.10.0）。引数なしだと `fav.toml` を探すので、
CI では明示的にファイルパスを渡す。

### A-2: `Self-fmt` ステップの後に `Self-test` ステップを追加

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

**注意**: `compiler.fav` が `--legacy-check` なしで `fav test` できるかを事前確認。
`fav test` は `fav check` を内部で呼ぶ場合がある。もし `compiler.fav` のテストが
collect/yield コーナーケースで失敗する場合は、そのファイルのみスキップし
コメントに理由を記載する。

---

## Phase B — CI: `integration` ジョブ追加（ci.yml）

### B-1: `integration` ジョブの構造

`ci.yml` の `rust` ジョブの後に追加:

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
```

### B-2: `integration` ジョブのステップ

```yaml
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

---

## Phase C — Rust 統合テストファイル作成（`fav/tests/integration.rs`）

### C-1: テストファイルの作成

`fav/tests/integration.rs` を新規作成（Rust integration test として自動検出される）:

```rust
//! Postgres integration tests — only run when DATABASE_URL is set.
//! Run with: cargo test integration -- --test-threads=1

fn db_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

#[test]
fn postgres_create_insert_select() {
    let url = match db_url() {
        Some(u) => u,
        None => return,  // skip when no DATABASE_URL
    };
    // vm.rs の pg_execute / pg_query を直接呼び出す
    // ...
}

#[test]
fn postgres_error_table_not_found() {
    let url = match db_url() { Some(u) => u, None => return };
    // ...
}

#[test]
fn postgres_ssl_disable_connects() {
    let url = match db_url() { Some(u) => u, None => return };
    // ...
}
```

### C-2: `pg_execute_raw_pub` / `pg_query_raw_pub` を公開

統合テストから vm.rs の Postgres 関数を呼ぶには、`pub(crate)` または `pub` の関数が必要。
v12.6.0 で追加した `format_pg_error_pub` のパターンを踏襲して、
テスト用の thin wrapper を `driver.rs` または `vm.rs` に追加:

```rust
// driver.rs または vm.rs に追加
pub fn pg_exec_for_test(url: &str, sql: &str) -> Result<(), String> { ... }
pub fn pg_query_for_test(url: &str, sql: &str) -> Result<String, String> { ... }
```

これらは `#[cfg(test)]` または通常の `pub` として公開。

---

## Phase D — Rust unit test: `v12900_tests`（driver.rs）

### D-1: `fav_test_self_checker_runs` / `fav_test_self_lexer_runs`

`cmd_test` を直接呼ぶ代わりに、`self/checker.fav` / `self/lexer.fav` を
プロセスとして fork するか、または `cmd_test` の戻り値を確認する関数を使う。

より簡単な実装:

```rust
fn run_fav_test_file(path: &str) -> bool {
    // cmd_test 相当を呼び出し、全テスト pass かどうかを返す
    // process::exit を呼ばずに bool で返す variant が必要
    // → cmd_test の内部ロジックを抽出するか、std::process::Command で ./target/debug/fav を使う
}
```

`std::process::Command` を使う場合（CI でも動く）:

```rust
fn run_self_test(file: &str) -> std::process::ExitStatus {
    std::process::Command::new(env!("CARGO_BIN_EXE_fav"))
        .args(["test", file])
        .status()
        .expect("failed to run fav test")
}
```

`CARGO_BIN_EXE_fav` は `cargo test` 実行時に自動的に設定されるマクロ変数。

### D-2: テスト定義

```rust
#[test]
fn fav_test_self_checker_runs() {
    let status = run_self_test("self/checker.fav");
    assert!(status.success(), "fav test self/checker.fav failed");
}

#[test]
fn fav_test_self_lexer_runs() {
    let status = run_self_test("self/lexer.fav");
    assert!(status.success(), "fav test self/lexer.fav failed");
}

#[test]
fn version_is_12_9_0() {
    assert_eq!(env!("CARGO_PKG_VERSION"), "12.9.0");
}
```

---

## Phase E — バージョン更新・コミット

- `fav/Cargo.toml` version → `"12.9.0"`
- `cargo test` 全通過確認
- `git commit -m "feat: v12.9.0 — CI fav test self/*.fav + Postgres integration tests"`
- `git push` → CI 通過確認

---

## 実装上の注意

### 1. `compiler.fav` の `fav test` での動作

`compiler.fav` は `collect/yield` 構文を使っている。
`fav test` の内部で `fav check` を走らせる場合、`--legacy-check` が必要になる可能性がある。
実際の動作を確認してから CI に追加する。もし問題があれば compiler.fav のみ除外し、
他の 4 ファイルを実行する。

### 2. Postgres 統合テストのスキップ設計

`DATABASE_URL` が未設定の場合は `return;` でスキップ。
これにより `cargo test` は通常環境（Windows ローカル）でも問題なく動く。

### 3. `tests/integration.rs` のファイル配置

`fav/tests/` ディレクトリは Rust の integration test の慣習的な場所。
`Cargo.toml` の変更は不要（自動検出される）。
ただしこのファイルは `fav_core` ライブラリを対象にするため、
`use fav_core::...` のインポートが必要。
vm.rs の関数を公開する際は `fav_core` の `lib.rs` から再エクスポートする。

### 4. `cargo test integration` のフィルタ

`cargo test integration -- --test-threads=1` で統合テストのみを実行できる。
`--test-threads=1` は Postgres への並行接続を避けるため必要。
CI の `integration` ジョブでのみ実行し、通常の `cargo test` では
`DATABASE_URL` がないためスキップされる。
