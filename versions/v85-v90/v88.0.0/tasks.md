# Tasks: v88.0.0 — SAP Sales 1.0 宣言 ★クリーンアップ

Status: COMPLETE

## code-reviewer 指摘と対応

- [BUG] `mod v87000_tests` 内の `cargo_toml_version_is_88_0_0` 関数名が汚染されていた → `cargo_toml_version_is_87_0_0` に修正（`"87.0.0"` → `"88.0.0"` 一括置換の巻き添えで変更されてしまったため）。なお、アサート内容（`"88.0.0"`）は Cargo.toml の実際の内容と一致するため維持。
- [BUG] `changelog_has_v87_0_0` のアサートが `[v88.0.0]` に汚染されていた → `[v87.0.0]` に修正（CHANGELOG.md は v87.0.0 エントリを保持しており、歴史的チェックとして正しい）。
- [STYLE] `versions/current.md` のマイルストーン進捗テーブルに v88.0 行が未追加 → SAP Sales 1.0 エントリを追加。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,993 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v87900_tests` が存在することを確認する（v87.9.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `87.0.0` であることを確認する（これから 88.0.0 に更新する）

## T1: `cargo clean` 実施

- [x] `cargo clean` を実行してビルド成果物を削除する

## T2: `Cargo.toml` バージョン更新

- [x] `fav/Cargo.toml` の `version = "87.0.0"` を `version = "88.0.0"` に変更する

## T3: `driver.rs` 内の既存 `cargo_toml_version_is_` テストを一括更新

- [x] `driver.rs` 内のすべての `"87.0.0"` を `"88.0.0"` に replace_all: true で一括置換する
- [x] `cargo_toml_version_is_87_0_0` 関数名を `cargo_toml_version_is_88_0_0` に更新する（最新テスト名のみ）
- Note: 過去のバージョン関数名（`cargo_toml_version_is_86_0_0` 等）は変更しない。バージョン文字列（`"87.0.0"` → `"88.0.0"`）のみ一括置換する

## T4: `CHANGELOG.md` に v88.0.0 エントリを追加

- [x] `## [v87.0.0]` の直前に v88.0.0 エントリを追加する（日付: 2026-08-23）
- [x] 宣言文・Added・Changed セクションを記載する（テスト数 3,997 (+4) を含む）

## T5: `MILESTONE.md` に SAP Sales 1.0 マイルストーンを追加

- [x] 先頭（`## v87.0.0` の直前）に v88.0.0 SAP Sales 1.0 マイルストーンエントリを追加する
- [x] v87.1〜v87.9 の達成内容（SalesOrder CRUD・ページネーション・売上レポート）を記載する

## T6: `README.md` を更新

- [x] 最新バージョン `v88.0.0`・テスト数 `3,997` を反映する

## T7: `versions/current.md` を更新

- [x] `v87.0.0` → `v88.0.0` に更新する（最新安定版・次に切る版・更新日）

## T8: `driver.rs` に `mod v88000_tests` を追加

- [x] `mod v87900_tests { ... }` の直後に `#[cfg(test)] mod v88000_tests { ... }` を追加する
- [x] `cargo_toml_version_is_88_0_0` テストを実装する（`Cargo.toml` に `version = "88.0.0"` を確認）
- [x] `changelog_has_v88_0_0` テストを実装する（`CHANGELOG.md` に `[v88.0.0]` を確認）
- [x] `milestone_has_sap_sales` テストを実装する（`MILESTONE.md` に `SAP Sales` を確認）
- [x] `sap_odata_rune_has_sales_order_type` テストを実装する（`sap_odata.fav` に `SalesOrder` を確認）

## T9: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,997 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- Note: T9（`cargo test`）完了後に実行すること（`cargo clean` 後に `target/debug/fav` が再生成されている必要がある）
- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
