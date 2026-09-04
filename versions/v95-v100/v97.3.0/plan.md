# Plan: v97.3.0 — `!Approval` エフェクトマーカー + `ApprovalClient` interface

## 実装ステップ

1. **`runes/sap-odata/workflow.fav` に `ApprovalClient` interface を追加**
   - `ApprovalClient` interface 型（`request_approval: fn(String, String) -> TaskDecision`）
   - `ApprovalClient.request_approval` スタブ関数（常に `Approve` を返す）
   - スタブである旨のコメントを付ける

2. **`runes/ctx/ctx.fav` に `approval` フィールドを追加**
   - 現在の `ctx.fav` の `use` 宣言と `AppCtx` 定義を確認する
   - `AppCtx` レコード型に `approval: ApprovalClient` フィールドを追加する
   - `ApprovalClient` を参照するための `use` 宣言が必要な場合は追加する

3. **`fav/src/driver.rs` に `mod v97300_tests` を追加**
   - `mod v97200_tests` の直後に追加
   - テスト 1: `workflow_fav_has_approval_client` — `workflow.fav` に `ApprovalClient` が含まれることを確認
   - テスト 2: `ctx_fav_has_approval_field` — `ctx.fav` に `approval` が含まれることを確認
   - ファイルパス: `std::fs::read_to_string("../runes/sap-odata/workflow.fav")` / `std::fs::read_to_string("../runes/ctx/ctx.fav")`
   - `use super::*` は不要

4. **`cargo test` で 4,219 tests, 0 failures を確認**

5. **CI 事前確認**
   - `cargo clippy --locked -- -D warnings` pass
   - `./target/debug/fav fmt --check self/compiler.fav` pass
   - `./target/debug/fav fmt --check self/checker.fav` pass

6. **`CHANGELOG.md` に `[v97.3.0]` エントリを追加**

7. **`versions/current.md` を v97.3.0 に更新**

## 注意事項

- `pub enum Effect {}` は Rust に存在しない。Rust 側の Effect enum 追加・exhaustive match 更新は不要。
- `ns_to_effect` は namespace 直接呼び出し（`IO.println` 等）用。`ctx.approval.*()` はメソッドチェーンのため対象外。`ns_to_effect` の更新は本バージョンのスコープ外。
- `interface ApprovalClient` の構文は既存の interface 定義パターン（`runes/ctx/ctx.fav` 等）に倣う
- driver.rs の `ctx.fav` パス: `std::fs::read_to_string("../runes/ctx/ctx.fav")`
