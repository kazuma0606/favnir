# Tasks: v97.3.0 — `!Approval` エフェクトマーカー + `ApprovalClient` interface

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v97.2.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v97200_tests` が存在することを確認する（v97.2.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,217 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `97.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 97.0.0 のまま）

## T1: `runes/sap-odata/workflow.fav` に `ApprovalClient` interface を追加

- [x] `ApprovalClient` interface 型（`request_approval: fn(String, String) -> TaskDecision`）を追加する
- [x] `ApprovalClient.request_approval` スタブ関数を追加する（常に `Approve` を返す、スタブコメント付き）

## T2: `runes/ctx/ctx.fav` に `approval` フィールドを追加

- [x] `ctx.fav` の既存 `use` 宣言と `AppCtx` 定義を確認する
- [x] `AppCtx` レコード型に `approval: ApprovalClient` フィールドを追加する
- [x] `ApprovalClient` を参照するために必要な `use` 宣言が存在するか確認し、必要なら追加する

## T3: `fav/src/driver.rs` に `mod v97300_tests` を追加

- [x] `mod v97200_tests` の直後に `#[cfg(test)] mod v97300_tests { ... }` を追加する
- [x] `workflow_fav_has_approval_client` テストを追加する（`workflow.fav` に `ApprovalClient` が含まれることを確認）
- [x] `ctx_fav_has_approval_field` テストを追加する（`ctx.fav` に `approval` が含まれることを確認）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,219 tests, 0 failures であることを確認する

## T5: `CHANGELOG.md` に v97.3.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v97.3.0]` エントリを追加する

## T6: `versions/current.md` 更新

- [x] 最新安定版を `v97.3.0` に更新する（テスト数 4,219）

## T-last: CI 事前確認（T4 の `cargo test` 全 pass 確認後・T5/T6 完了後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
