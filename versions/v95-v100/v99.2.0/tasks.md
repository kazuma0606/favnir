# Tasks: v99.2.0 — `!Audit` エフェクトマーカー + 監査ログ ctx interface

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/v95-v100/v99.1.0/tasks.md` の Status が `COMPLETE` であることを確認する
- [x] `versions/current.md` の最新安定版が `v99.1.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v99100_tests` が存在することを確認する（v99.1.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,259 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `99.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 99.0.0 のまま）
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: audit.fav を新規作成

- [x] `runes/sap-odata/audit.fav` を新規作成する
- [x] ファイル先頭コメントに `-- runes/sap-odata/audit.fav` が含まれることを確認する
- [x] `AuditEvent` 型（`actor` / `action` / `resource` / `timestamp` / `result`）が定義されていることを確認する
- [x] `AuditTrail` 型（`events` / `pipeline` / `started_at`）が定義されていることを確認する
- [x] `AuditClient` interface（`fn log(event: AuditEvent) -> Result<Unit, String>`）が定義されていることを確認する
- [x] `log_audit_event_mock(event: AuditEvent) -> Result<Unit, String>` 関数が実装されていることを確認する
- [x] コメントが `--` スタイルであることを確認する（`//` 不可）

## T2: sap_odata.fav に use と re-export を追加

- [x] `runes/sap-odata/sap_odata.fav` の `use` 宣言ブロックに `use sap_odata.audit` を追加する
- [x] `sap_odata.fav` 末尾に `-- 監査ログ型 re-export（v99.2.0〜）` セクションを追加する
- [x] `AuditEvent` / `AuditTrail` / `log_audit_event_mock` の 3 シンボルが re-export されていることを確認する

## T3: ctx.fav に audit フィールドを追加

- [x] `runes/ctx/ctx.fav` に `use sap_odata.audit` を追加する
- [x] `AppCtx` 型定義に `audit: AuditClient` フィールドを追加する
- [x] コメントで `（v99.2.0 追加）` と記述する

## T4: driver.rs に mod v99200_tests を追加

- [x] `mod v99100_tests` の直後に `mod v99200_tests`（2 テスト）を追加する:
  - `audit_fav_exists`: `runes/sap-odata/audit.fav` の存在を確認
  - `audit_fav_has_audit_event`: `AuditEvent` / `AuditTrail` / `AuditClient` / `log_audit_event_mock` が含まれることを確認
- [x] `mod v99200_tests` ブロック先頭に `// use super::* は不要（std::fs のみ使用）` という Rust コメントを 1 行追記する

## T5: cargo test で全 pass 確認

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,261 tests, 0 failures であることを確認する

## T6: CHANGELOG.md に v99.2.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v99.2.0]` エントリを追加する

## T7: versions/current.md 更新

- [x] `最終更新:` ヘッダーを `v99.2.0` に更新する
- [x] 最新安定版を `v99.2.0` に更新する（テスト数 4,261）

<!-- MILESTONE.md 更新は宣言版（v100.0.0）で対応予定（patch version は対象外） -->
<!-- site MDX ドキュメントは v99.8.0 で対応予定（本バージョンはスコープ外） -->
<!-- !Audit Rust Effect enum 追加は v99.4.0 以降で対応予定（本バージョンはスコープ外） -->

## T-last: CI 事前確認（T5 の `cargo test` 全 pass 確認後・T6/T7 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 指摘対応（実装後）

| 優先度 | 指摘 | 対応 |
|--------|------|------|
| [HIGH] | `ctx.fav` の既存コメントが `//` スタイル、新規追加行のみ `--` で混在 | `ctx.fav` 全行の `//` を `--` に統一 |
| [MED] | `sap_odata.fav` に `AuditClient` interface の re-export がない | `public type AuditClient = audit.AuditClient` を追加（4 シンボルに） |
| [LOW] | `ctx.fav` の `use sap_odata.audit` が先頭で順序が不自然 | 他の `use` ブロックの末尾（`use sap_odata.workflow` の後）に移動 |
