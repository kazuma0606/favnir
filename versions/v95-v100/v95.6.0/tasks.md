# Tasks: v95.6.0 — Function Import / Action Import

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v95.5.0` であることを確認する
- [x] `runes/sap-odata/sales_order.fav` に `create_sales_order_deep` が存在することを確認する（v95.5.0 完了済みの証拠）
- [x] `fav/src/driver.rs` に `mod v95500_tests` が存在することを確認する（v95.5.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,174 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `95.0.0` であることを確認する
  （スプリント内バージョンでは Cargo.toml を更新しない）

## T1: `runes/sap-odata/rpc.fav` 新規作成

- [x] `rpc.fav` を新規作成する
- [x] `FunctionImportParam` 型エイリアス（`(String, String)` タプル）を定義する
- [x] `function_import<T>` スタブ関数を定義する
  （`cfg: SapConfig, function_name: String, params: List<FunctionImportParam>) -> Result<T, String>`、戻り値 `Result.err("not implemented")`）
- [x] `action_import` スタブ関数を定義する
  （`cfg: SapConfig, action_name: String, params: List<FunctionImportParam>) -> Result<Unit, String>`、戻り値 `Result.err("not implemented")`）
- [x] 各定義に `public` を付与する

## T2: `driver.rs` にテストを追加

- [x] `mod v95500_tests` の直後に `#[cfg(test)] mod v95600_tests { ... }` を追加する
- [x] `rpc_fav_exists` テストを追加する（`../runes/sap-odata/rpc.fav` が存在する）
- [x] `rpc_fav_has_function_import` テストを追加する（`rpc.fav` に `function_import` が含まれる）
- [x] `rpc_fav_has_action_import` テストを追加する（`rpc.fav` に `action_import` が含まれる）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,177 tests, 0 failures であることを確認する

## T4: CHANGELOG / current.md 更新

- [x] `CHANGELOG.md` の先頭に `[v95.6.0]` エントリを追加する
- [x] `versions/current.md` の最新安定版を `v95.6.0` に更新する

## T5: tasks.md 更新

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする

## T-last: CI 事前確認（T3 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
