# Tasks: v85.5.0 — OData v4 HTTP クライアント基盤（`odata_get` / `odata_list`）

Status: COMPLETE

> MILESTONE.md / README.md / `site/content/docs/` の更新は v86.0.0 宣言バージョンで実施する。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,939 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v85400_tests` が存在することを確認する（v85.4.0 完了済みの証拠）
- [x] `runes/sap-odata/sap_odata.fav` と `runes/sap-odata/types.fav` が存在することを確認する

## T1: `runes/sap-odata/types.fav` に `ODataParams` 型を追加

- [x] 既存ファイル末尾に `ODataParams` 型定義を追記する
  - `filter: Option<String>`（`$filter`）
  - `select: Option<String>`（`$select`）
  - `expand: Option<String>`（`$expand`）
  - `top: Option<Int>`（`$top`）
  - `skip: Option<Int>`（`$skip`）
  - `orderby: Option<String>`（`$orderby`）

## T2: `runes/sap-odata/client.fav` を新規作成

- [x] `use sap_odata.types` を記述する
- [x] `public fn odata_get(cfg: SapConfig, entity_set: String, key: String) -> Result<String, String>` を実装する
- [x] `public fn odata_list(cfg: SapConfig, entity_set: String, params: ODataParams) -> Result<String, String>` を実装する
- [x] `fn basic_auth_header(username: String, password: String) -> String` を実装する（`bind` 不使用 — `String.concat` は `String` を直接返す）
- [x] `fn build_query_string(params: ODataParams) -> String` を骨格実装する（空文字列返し、v85.9.0 で本実装予定）

## T3: `runes/sap-odata/sap_odata.fav` を更新

- [x] `use sap_odata.client` を追加する
- [x] `public type ODataParams = types.ODataParams` を追加する
- [x] `public fn odata_get(...)` の re-export を追加する
- [x] `public fn odata_list(...)` の re-export を追加する

## T4: `mod v85500_tests` を追加

- [x] `mod v85400_tests { ... }` の直後に `#[cfg(test)] mod v85500_tests { ... }` を追加する
- [x] `odata_list_function_exists_in_rune` テストを実装する
  - `runes/sap-odata/sap_odata.fav` に `odata_list` が含まれることを確認
  - ファイルパス: `../runes/sap-odata/sap_odata.fav`
- [x] `odata_params_type_exists` テストを実装する
  - `runes/sap-odata/types.fav` に `ODataParams` が含まれることを確認
  - ファイルパス: `../runes/sap-odata/types.fav`

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,941 tests, 0 failures であることを確認する

## T6: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v85.5.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 修正事項（spec-reviewer 指摘対応）

- [HIGH] ロードマップ 2 ファイルの `x-csrf-token` 記述を「v86.x 系に延期」に修正
- [MED] `basic_auth_header` の `bind creds <- Result.ok(String)` を `String.concat` 直接ネストに修正
- [MED] ロードマップの関数シグネチャに `public fn` 明記 + re-export 設計の注記を追加
- [LOW] plan.md に `build_query_string` 骨格の意図コメントを追記
