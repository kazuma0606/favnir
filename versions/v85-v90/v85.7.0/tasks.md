# Tasks: v85.7.0 — `fav new` テンプレート + `fav.toml [sap]` セクション追加

Status: COMPLETE

> MILESTONE.md / README.md / `site/content/docs/` の更新は v86.0.0 宣言バージョンで実施する。
> 本バージョンは `default_fav_toml()` の修正と Rust テスト追加のみ。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,943 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v85600_tests` が存在することを確認する（v85.6.0 完了済みの証拠）
- [x] `fav/src/driver.rs` の `default_fav_toml()` に `# [snowflake]` が存在することを確認する

## T1: `default_fav_toml()` に `[sap]` コメントブロックを追加

- [x] `# [snowflake]` ブロック末尾の `\n"` を `\n\n` + `[sap]` ブロック + `\n"` に変更する
  - `# [sap]`
  - `# base_url = "${SAP_BASE_URL}"   # SAP S/4HANA エンドポイント`
  - `# client   = "100"                  # SAP クライアント番号`（スペース数は spec.md / plan.md のコードサンプルを正とする）
  - `# username = "${SAP_USER}"`
  - `# password = "${SAP_PASS}"`
  - `# auth     = "basic"             # "basic" | "oauth2"`
- [x] Rust フォーマット文字列の `{{` / `}}` エスケープが正しいことを確認する

## T2: `mod v85700_tests` を追加

- [x] `mod v85600_tests { ... }` の直後に `#[cfg(test)] mod v85700_tests { ... }` を追加する
- [x] `fav_new_template_contains_sap_comment` テストを実装する
  - `super::default_fav_toml("test")` が `"# [sap]"` を含むことを確認
- [x] `sap_toml_section_parses_correctly` テストを実装する
  - `parse_fav_toml_pub("[sap]\nbase_url = \"https://example.com\"\n")` が `sap.base_url = Some("https://example.com")` を返すことを確認

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,945 tests, 0 failures であることを確認する

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v85.7.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 修正事項（spec-reviewer 指摘対応）

- [MED] tasks.md の `client` 行スペース数を spec.md / plan.md に合わせ、注記を追加
- [LOW] spec.md の `sap_toml_section_parses_correctly` に意図的重複（リグレッションガード）の説明を追記
