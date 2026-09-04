# Tasks: v96.1.0 — `SapEnvironment` 型 + `ctx.sap_env()`

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v96.0.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v96000_tests` が存在することを確認する（v96.0.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,188 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `96.0.0` であることを確認する（更新前の状態確認）

## T1: `runes/sap-odata/types.fav` に `SapEnvironment` 型を追加

- [x] `SapClient` interface 定義の直後（ファイル末尾）に `SapEnvironment` 直和型を追加する
  - バリアント: `Prd` / `Qas` / `Dev` / `Custom(String)`
- [x] `SapEnvironment.from_string(name: String) -> SapEnvironment` 関数を追加する
  - `"PRD"` → `Prd`、`"QAS"` → `Qas`、`"DEV"` → `Dev`、それ以外 → `Custom(name)`

## T2: `runes/ctx/ctx.fav` に `Ctx.sap_env()` 関数を追加

- [x] `Ctx.mock` 関数の直後（ファイル末尾）に `Ctx.sap_env(name: String) -> Result<SapClient, String>` スタブ関数を追加する
  - 本体: `Result.err("sap_env not implemented: use Ctx.build() for now")`

## T3: `fav/src/driver.rs` に `mod v96100_tests` を追加

- [x] `mod v96000_tests` の直後に `#[cfg(test)] mod v96100_tests { ... }` を追加する
- [x] `sap_environment_type_defined` テストを追加する（`types.fav` に `SapEnvironment` が含まれる）
- [x] `ctx_sap_env_fn_defined` テストを追加する（`ctx.fav` に `sap_env` が含まれる）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,190 tests, 0 failures であることを確認する

## T5: `CHANGELOG.md` に v96.1.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v96.1.0]` エントリを追加する

## T6: `versions/current.md` 更新

- [x] 最新安定版を `v96.1.0` に更新する（テスト数 4,190）

## T-last: CI 事前確認（T4 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
