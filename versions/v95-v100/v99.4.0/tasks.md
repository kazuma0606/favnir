# Tasks: v99.4.0 — マルチテナント対応

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/v95-v100/v99.3.0/tasks.md` の Status が `COMPLETE` であることを確認する
- [x] `versions/current.md` の最新安定版が `v99.3.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v99300_tests` が存在することを確認する（v99.3.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,263 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `99.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 99.0.0 のまま）
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: tenant.fav を新規作成

- [x] `runes/sap-odata/tenant.fav` を新規作成する
- [x] ファイル先頭コメントに `-- runes/sap-odata/tenant.fav` が含まれることを確認する
- [x] `TenantId` 型エイリアス（`String`）が定義されていることを確認する
- [x] `TenantContext` 型（`tenant_id` / `sap_env` / `schema`）が定義されていることを確認する
- [x] `tenant_context_mock(tenant_id: TenantId) -> TenantContext` 関数が実装されていることを確認する
- [x] コメントが `--` スタイルであることを確認する（`//` 不可）

## T2: sap_odata.fav に use と re-export を追加

- [x] `runes/sap-odata/sap_odata.fav` の `use` 宣言ブロックに `use sap_odata.tenant` を追加する
- [x] `sap_odata.fav` 末尾に `-- マルチテナント型 re-export（v99.4.0〜）` セクションを追加する
- [x] `TenantId` / `TenantContext` / `tenant_context_mock` の 3 シンボルが re-export されていることを確認する

## T3: ctx.fav に Ctx.for_tenant_mock を追加

- [x] `runes/ctx/ctx.fav` に `use sap_odata.tenant` を追加する
- [x] `Ctx.sap_env` 関数の後に `Ctx.for_tenant_mock(tenant_id: String) -> tenant.TenantContext` を追加する
- [x] コメントに `（v99.4.0）` と記述する

## T4: driver.rs に mod v99400_tests を追加

- [x] `mod v99300_tests` の直後に `mod v99400_tests`（2 テスト）を追加する:
  - `tenant_fav_exists`: `runes/sap-odata/tenant.fav` の存在を確認
  - `tenant_fav_has_tenant_context`: `TenantId` / `TenantContext` / `tenant_context_mock` が含まれることを確認
- [x] `mod v99400_tests` ブロック先頭に `// use super::* は不要（std::fs のみ使用）` という Rust コメントを 1 行追記する

## T5: cargo test で全 pass 確認

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,265 tests, 0 failures であることを確認する

## T6: CHANGELOG.md に v99.4.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v99.4.0]` エントリを追加する

## T7: versions/current.md 更新

- [x] `最終更新:` ヘッダーを `v99.4.0` に更新する
- [x] 最新安定版を `v99.4.0` に更新する（テスト数 4,265）

<!-- MILESTONE.md 更新は宣言版（v100.0.0）で対応予定（patch version は対象外） -->
<!-- site MDX ドキュメントは v99.8.0 で対応予定（本バージョンはスコープ外） -->
<!-- ctx.sap.for_tenant() の SapClient interface 実装は将来バージョンで対応予定 -->

## T-last: CI 事前確認（T5 の `cargo test` 全 pass 確認後・T6/T7 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 指摘対応（実装後）

| 優先度 | 指摘 | 対応 |
|--------|------|------|
| [STYLE] | `sap_odata.fav` の `tenant_context_mock` 引数型が `tenant.TenantId`（完全修飾）で re-export エイリアスと不一致 | `TenantId`（エイリアス）に統一 |
