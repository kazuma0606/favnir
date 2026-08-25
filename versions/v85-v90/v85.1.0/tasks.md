# Tasks: v85.1.0 — `SapTomlConfig` + `inject_sap_config()`

Status: COMPLETE

> MILESTONE.md / README.md / `site/content/docs/` の更新は v86.0.0 宣言バージョンで実施する。
> 本バージョンは Rust 基盤（`fav/src/toml.rs` + `fav/src/driver.rs`）のみを変更する。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,931 tests, 0 failures を確認する
- [x] `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "85.0.0"` であることを確認する
- [x] `fav/src/driver.rs` に `mod v85000_tests` が存在することを確認する（v85.0.0 完了済みの証拠）
- [x] `fav/src/toml.rs` に `SnowflakeTomlConfig` など既存 inject パターンが存在することを確認する

## T1: `fav/src/toml.rs` に `SapTomlConfig` を追加

- [x] `BenchTomlConfig` の前に `SapTomlConfig` 構造体を追加する
  - `#[derive(Debug, Clone)]`（既存パターンに合わせて）
  - フィールド: `base_url`, `client`, `username`, `password`, `auth`（すべて `Option<String>`）
- [x] `FavToml` 構造体に `pub sap: Option<SapTomlConfig>` フィールドを追加する
- [x] `parse_fav_toml` に `[sap]` セクションヘッダー検出 + フィールド解析を追加する
- [x] `parse_fav_toml` の return 構造体に `sap: sap_cfg` を追加する
- [x] `checker.rs` / `resolver.rs` / `driver.rs` のインライン `FavToml` リテラルに `sap: None` を追加する

## T2: `fav/src/driver.rs` に `inject_sap_config()` を追加

- [x] `inject_snowflake_config` の直後に `fn inject_sap_config(cfg: &crate::toml::SapTomlConfig)` を追加する
  - pairs スライスで `SAP_BASE_URL` / `SAP_CLIENT` / `SAP_USER` / `SAP_PASS` / `SAP_AUTH` を設定
  - `expand_env_vars()` を適用し、既存 env var は上書きしない（`std::env::var(key).is_err()` ガード）

## T3: `cmd_run` / `cmd_check` で `inject_sap_config` を呼ぶ

- [x] `cmd_run` の `inject_snowflake_config` 呼び出し箇所の直後に `inject_sap_config` 呼び出しを追加する
- [x] `cmd_check` の `inject_snowflake_config` 呼び出し箇所の直後に `inject_sap_config` 呼び出しを追加する

## T4: `mod v85100_tests` を追加

- [x] `mod v85000_tests { ... }` の直後に `#[cfg(test)] mod v85100_tests { ... }` を追加する
- [x] `sap_toml_config_parses_base_url` テストを実装する
  - `parse_fav_toml_pub` で `[sap]` セクションを解析し `SapTomlConfig.base_url` を確認する
- [x] `inject_sap_config_sets_env_vars` テストを実装する
  - `SapTomlConfig` を構築して `inject_sap_config` を呼び、`SAP_BASE_URL` と `SAP_CLIENT` を確認する
  - テスト後 `std::env::remove_var` でクリーンアップする

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,933 tests, 0 failures を確認した

## T6: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v85.1.0 エントリを追加した

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認した（CI と同じフラグ）
- [ ] `./target/debug/fav fmt --check self/compiler.fav` — cargo clean 後のためバイナリ未存在、v86.0.0 クリーンアップ時に確認
- [ ] `./target/debug/fav fmt --check self/checker.fav` — 同上

## 実装メモ（code-reviewer 向け）

- `#[derive(Debug, Clone)]` を使用（serde 不使用 — toml.rs は手動パーサー）
- `[sap]` セクションヘッダー検出が必要（`parse_fav_toml` 内の if ブロックに追加）
- `checker.rs` / `resolver.rs` / `driver.rs` の `FavToml` リテラルに `sap: None` 追加が必要
