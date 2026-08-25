# Tasks: v87.0.0 — SAP Master Data 1.0 宣言 ★クリーンアップ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,971 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v86900_tests` が存在することを確認する（v86.9.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `86.0.0` であることを確認する

## T1: `CHANGELOG.md` に v87.0.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に v87.0.0 宣言エントリを追加する（テストモジュール追加より先に実施）

## T2: `fav/Cargo.toml` のバージョンを更新

- [x] `version = "86.0.0"` → `version = "87.0.0"` に変更する

## T3: `driver.rs` の `cargo_toml_version` テスト群を一括更新（計 33 件）

- [x] `cargo_toml_version_is_86_0_0` テストを `87.0.0` に更新する（`replace_all` で `"86.0.0"` → `"87.0.0"` 一括置換）
- [x] 更新後、計 38 件が `87.0.0` を参照していることを確認した（ロードマップ推定 33 件より多いが関数名・コメント等を含む全件を正しく更新）

## T4: `mod v87000_tests` を追加

- [x] `mod v86900_tests { ... }` の直後に `#[cfg(test)] mod v87000_tests { ... }` を追加する
- [x] `cargo_toml_version_is_87_0_0` テストを実装する
- [x] `changelog_has_v87_0_0` テストを実装する
- [x] `milestone_has_sap_master_data` テストを実装する
- [x] `sap_odata_rune_toml_has_name_sap_odata` テストを実装する

## T5: `MILESTONE.md` / `README.md` / `versions/current.md` 更新

- [x] `MILESTONE.md` に SAP Master Data 1.0（v87.0.0）エントリを追加する
- [x] `README.md` の最新バージョン記述を v87.0.0 に更新する
- [x] `versions/current.md` を v87.0.0 に更新する（最終更新・最新安定版・マイルストーン表）

## T6: `cargo test` で全 pass 確認（clean 前）

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,975 tests, 0 failures であることを確認する

## T7: `cargo clean` 実施（★クリーンアップ）

- [x] `cargo clean` を実行してビルドキャッシュをクリアする（9.8 GiB 削除）

## T8: `cargo test` で全 pass 再確認（clean 後）

- [x] clean 後に `cargo test 2>&1 | grep "test result"` を実行し、3,975 tests, 0 failures であることを確認する
- [x] `fav/tmp/hello.fav` が消えていたため復元した（`fn add` + `fn main` の内容）

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 修正事項（code-reviewer 指摘対応）

- [LOW] `v86000_tests::changelog_has_v86_0_0` のアサーションが `"v87.0.0"` になっていた（バルク置換の副作用）→ `"[v86.0.0]"` に修正
- [LOW] `v87000_tests::changelog_has_v87_0_0` を `fs::read_to_string` → `include_str!("../../CHANGELOG.md")` に変更（コードベース慣例への統一）
- [LOW] `v87000_tests::milestone_has_sap_master_data` を `fs::read_to_string` → `include_str!("../../MILESTONE.md")` に変更（同上）
