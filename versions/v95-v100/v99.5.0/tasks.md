# Tasks: v99.5.0 — GDPR データマスキング

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/v95-v100/v99.4.0/tasks.md` の Status が `COMPLETE` であることを確認する
- [x] `versions/current.md` の最新安定版が `v99.4.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v99400_tests` が存在することを確認する（v99.4.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,265 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `99.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 99.0.0 のまま）
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: privacy.fav を新規作成

- [x] `runes/sap-odata/privacy.fav` を新規作成する
- [x] ファイル先頭コメントに `-- runes/sap-odata/privacy.fav` が含まれることを確認する
- [x] `Masked<T>` 型（`inner: T` フィールド）が定義されていることを確認する
- [x] `UnmaskClient` interface（`fn unmask<T>(masked: Masked<T>) -> Result<T, String>`）が定義されていることを確認する
- [x] `mask<T>(value: T) -> Masked<T>` 関数が実装されていることを確認する
- [x] `unmask_mock<T>(masked: Masked<T>) -> Result<T, String>` 関数が実装されていることを確認する
- [x] コメントが `--` スタイルであることを確認する（`//` 不可）

## T2: sap_odata.fav に use と re-export を追加

- [x] `runes/sap-odata/sap_odata.fav` の `use` 宣言ブロックに `use sap_odata.privacy` を追加する（`use sap_odata.tenant` の直後）
- [x] `sap_odata.fav` 末尾に `-- GDPR データマスキング型 re-export（v99.5.0〜）` セクションを追加する
- [x] `Masked<T>` / `UnmaskClient` / `mask` / `unmask_mock` の 4 シンボルが re-export されていることを確認する
- [x] re-export 関数の戻り型が re-export 済みエイリアスを使用していることを確認する（`Masked<T>` not `privacy.Masked<T>`）

## T3: ctx.fav に unmask フィールドを追加

- [x] `runes/ctx/ctx.fav` に `use sap_odata.tenant` が存在することを確認する（挿入位置の前提）
- [x] `runes/ctx/ctx.fav` に `use sap_odata.privacy` を追加する（`use sap_odata.tenant` の直後）
- [x] `AppCtx` 型の `audit: AuditClient` 行の直後に `unmask: UnmaskClient` フィールドを追加する
- [x] コメントに `（v99.5.0 追加）` と記述する

## T4: driver.rs に mod v99500_tests を追加

- [x] `mod v99400_tests` の直後に `mod v99500_tests`（2 テスト）を追加する:
  - `privacy_fav_exists`: `runes/sap-odata/privacy.fav` の存在を確認
  - `privacy_fav_has_masked`: `Masked` / `UnmaskClient` / `mask` / `unmask_mock` が含まれることを確認
- [x] `mod v99500_tests` ブロック先頭に `// use super::* は不要（std::fs のみ使用）` という Rust コメントを 1 行追記する

## T5: cargo test で全 pass 確認

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,267 tests, 0 failures であることを確認する

## T6: CHANGELOG.md に v99.5.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v99.5.0]` エントリを追加する

## T7: versions/current.md 更新

- [x] `最終更新:` ヘッダーを `v99.5.0` に更新する
- [x] 最新安定版を `v99.5.0` に更新する（テスト数 4,267）

<!-- MILESTONE.md 更新は宣言版（v100.0.0）で対応予定（patch version は対象外） -->
<!-- site MDX ドキュメントは v99.8.0 で対応予定（本バージョンはスコープ外） -->
<!-- Effect::Unmask の Rust enum 追加と checker.fav exhaustive match 更新は将来バージョンで対応予定 -->

## T-last: CI 事前確認（T5 の `cargo test` 全 pass 確認後・T6/T7 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 指摘対応（実装後）

| 優先度 | 指摘 | 対応 |
|--------|------|------|
| [LOW] | `driver.rs` の `content.contains("mask")` が `unmask_mock` にも部分一致する偽陽性リスク | `content.contains("fn mask<")` に変更 |
| 情報 | `sap_odata.fav` の v98〜v99 既存 re-export 関数に未エイリアス負債あり | 今回の変更範囲外のため対応なし（将来バージョンで対応予定） |
