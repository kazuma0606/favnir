# Tasks: v96.2.0 — `fav.toml [sap.environments]` マルチ環境設定

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v96.1.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v96100_tests` が存在することを確認する（v96.1.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,190 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `96.0.0` であることを確認する（v96.1.0 はパッチ版のため Cargo.toml version は宣言版 96.0.0 のまま）

## T1: `fav/src/toml.rs` に `SapEnvEntry` 構造体と `SapEnvironmentsConfig` 型を追加

- [x] 既存 `SapTomlConfig` 定義の直前に `SapEnvEntry` 構造体を追加する
  - フィールド: `base_url / client / username / password`（すべて `Option<String>`）
- [x] `SapEnvironmentsConfig` 型エイリアス（`HashMap<String, SapEnvEntry>`）を追加する

## T2: `fav/src/toml.rs` — `SapTomlConfig` に `environments` フィールドを追加

- [x] `SapTomlConfig` に `pub environments: SapEnvironmentsConfig` フィールドを追加する
- [x] `driver.rs` 内の `SapTomlConfig { ... }` コンストラクタに `environments: std::collections::HashMap::new()` を追加する（コンパイルエラー修正）

## T3: `fav/src/toml.rs` — `[sap.environments.<NAME>]` セクションのパース処理を追加

- [x] パーサーループ内に `current_sap_env: String` ローカル変数を追加する
- [x] `[sap.environments.<NAME>]` セクションヘッダーの検出処理を追加する
  - `trimmed.starts_with("[sap.environments.")` で判定し `section = "sap_env"` に遷移する
- [x] `"sap_env"` セクション内 KV パース処理を追加する（`base_url / client / username / password`）

## T4: `fav/src/driver.rs` に `mod v96200_tests` を追加

- [x] `mod v96100_tests` の直後に `#[cfg(test)] mod v96200_tests { ... }` を追加する
- [x] `sap_env_entry_struct_defined` テストを追加する（`toml.rs` に `SapEnvEntry` が含まれる）
- [x] `sap_toml_config_has_environments_field` テストを追加する（`toml.rs` に `SapEnvironmentsConfig` が含まれる）

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,192 tests, 0 failures であることを確認する

## T6: `CHANGELOG.md` に v96.2.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v96.2.0]` エントリを追加する

## T7: `versions/current.md` 更新

- [x] 最新安定版を `v96.2.0` に更新する（テスト数 4,192）

## T-last: CI 事前確認（T5 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
