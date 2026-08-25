# Tasks: v85.2.0 — `SapConfig` Favnir 型 + `sap_config_from_env()`

Status: COMPLETE

> MILESTONE.md / README.md / `site/content/docs/` の更新は v86.0.0 宣言バージョンで実施する。
> 本バージョンは Favnir ファイル（`runes/sap-odata/types.fav`）の新規作成と Rust テスト追加のみ。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,933 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v85100_tests` が存在することを確認する（v85.1.0 完了済みの証拠）
- [x] `runes/` ディレクトリが存在することを確認する

## T1: `runes/sap-odata/` ディレクトリを作成

- [x] `runes/sap-odata/` ディレクトリを作成する

## T2: `runes/sap-odata/types.fav` を作成

- [x] `SapConfig` レコード型を定義する（`base_url`, `client`, `username`, `password`, `auth: String`）
- [x] `sap_config_from_env() -> Result<SapConfig, String>` を実装する
  - `bind base_url <- Env.require("SAP_BASE_URL")`
  - `bind username <- Env.require("SAP_USER")`
  - `bind password <- Env.require("SAP_PASS")`
  - `client: Env.get_or("SAP_CLIENT", "100")`（bind なし — String を直接返す）
  - `auth:   Env.get_or("SAP_AUTH", "basic")`（bind なし — String を直接返す）

## T3: `mod v85200_tests` を追加

- [x] `mod v85100_tests { ... }` の直後に `#[cfg(test)] mod v85200_tests { ... }` を追加する
- [x] `sap_odata_types_fav_defines_sap_config_from_env` テストを実装する
  - `runes/sap-odata/types.fav` が存在し、`sap_config_from_env` と `SapConfig` を含むことを確認する
  - ファイルパス: `std::fs::read_to_string("../runes/sap-odata/types.fav")`（`../../` ではなく `../`）
- [x] `sap_odata_types_fav_requires_sap_base_url` テストを実装する
  - `types.fav` に `Env.require("SAP_BASE_URL")` が含まれることを確認する

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,935 tests, 0 failures であることを確認する

## T5: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v85.2.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 注記

- ファイルパスは `../runes/sap-odata/types.fav`（`fav/` から 1 段上）。`../../` は誤り（spec.md の記述を修正済み）
- master ロードマップ `roadmap-v85.1-v90.0.md` v85.2.0 セクションに `sap_config_from_env_raw()` VM primitive の誤記があったため削除（spec-reviewer [HIGH] 指摘）
