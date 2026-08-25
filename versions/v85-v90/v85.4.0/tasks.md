# Tasks: v85.4.0 — `runes/sap-odata/` 骨格 + `rune.toml`

Status: COMPLETE

> MILESTONE.md / README.md / `site/content/docs/` の更新は v86.0.0 宣言バージョンで実施する。
> 本バージョンは `rune.toml` / `sap_odata.fav` / `sap_odata.test.fav` の新規作成と Rust テスト追加のみ。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,937 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v85300_tests` が存在することを確認する（v85.3.0 完了済みの証拠）
- [x] `runes/sap-odata/types.fav` が存在することを確認する（v85.2.0 作成済み）

## T1: `runes/sap-odata/rune.toml` を作成

- [x] `[rune]` セクションを含む `rune.toml` を作成する
  - `name = "sap-odata"`
  - `version = "85.4.0"`
  - `entry = "sap_odata.fav"`
  - `description = "SAP S/4HANA OData v4 クライアント — ctx パターンで型安全な SAP データアクセスを提供"`

## T2: `runes/sap-odata/sap_odata.fav` を作成

- [x] エントリポイントファイルを作成する（`types.fav` を use して re-export）
  - `use sap_odata.types`
  - `public type SapConfig = types.SapConfig`
  - `public fn sap_config_from_env() -> Result<SapConfig, String>`

## T3: `runes/sap-odata/sap_odata.test.fav` を作成

- [x] テストファイル骨格を作成する（`test_sap_config_fields_exist` 関数）

## T4: `mod v85400_tests` を追加

- [x] `mod v85300_tests { ... }` の直後に `#[cfg(test)] mod v85400_tests { ... }` を追加する
- [x] `sap_odata_rune_toml_exists` テストを実装する
  - `std::path::Path::new("../runes/sap-odata/rune.toml").exists()` を確認
- [x] `sap_odata_rune_entry_exists` テストを実装する
  - `std::path::Path::new("../runes/sap-odata/sap_odata.fav").exists()` を確認

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,939 tests, 0 failures であることを確認する

## T6: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v85.4.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 注記

- ロードマップ両ファイル（roadmap-v85.1-v86.0.md / roadmap-v85.1-v90.0.md）の `types.fav` 箇条書きを「v85.2.0 作成済み・変更なし」に修正（spec-reviewer [HIGH] 指摘）

## code-reviewer 指摘（後続バージョンで対応）

- [LOW] `sap_odata.fav` の re-export 形式が snowflake/postgres と異なる（ラッパー関数 vs 名前付きインポート）— `client.fav` 追加時に形式統一を検討
- [LOW] `sap_odata.test.fav` が `fn` 形式（`test "..."` ブロックではない）— E2E テスト追加バージョンで `test "..."` ブロックに切り替える
