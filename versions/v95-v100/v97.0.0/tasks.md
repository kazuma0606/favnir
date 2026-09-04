# Tasks: v97.0.0 — SAP Multi-system 1.0 宣言

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v96.9.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v96900_tests` が存在することを確認する（v96.9.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,209 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `96.0.0` であることを確認する

## T1: `fav/Cargo.toml` バージョン更新

- [x] `version = "96.0.0"` → `version = "97.0.0"` に変更する

## T2: `CHANGELOG.md` に v97.0.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v97.0.0]` エントリを追加する
- [x] SAP Multi-system 1.0 宣言の内容（v96.1〜v96.9 で実装した機能サマリー）を記載する

## T3: `MILESTONE.md` に v97.0.0 エントリを追加

- [x] v97.0 — SAP Multi-system 1.0 エントリを追加する

## T4: `README.md` に v97.0 セクションを追加

- [x] `## v97.0 — SAP Multi-system 1.0` セクションを追加する

## T5: `fav/src/driver.rs` に `mod v97000_tests` を追加

- [x] `mod v96900_tests` の直後に `#[cfg(test)] mod v97000_tests { ... }` を追加する
- [x] `cargo_toml_version_is_97_0_0` テストを追加する
- [x] `changelog_has_v97_0_0` テストを追加する
- [x] `milestone_has_sap_multi_system` テストを追加する
- [x] `readme_mentions_sap_multi_system` テストを追加する

## T6: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,213 tests, 0 failures であることを確認する

## T-last: CI 事前確認（T6 の `cargo test` 全 pass 確認後・T7 の `cargo clean` 前に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T7: `cargo clean` を実施（★クリーンアップ）

- [x] `cd fav && cargo clean` を実行する
- [x] `cargo test 2>&1 | grep "test result"` を再実行し、クリーンビルドでも 4,213 tests, 0 failures であることを確認する

## T8: `versions/current.md` 更新

- [x] 最新安定版を `v97.0.0` に更新する（テスト数 4,213）
- [x] 前バージョン欄に v96.9.0 を記載する
- [x] マイルストーン一覧に `v97.0 — SAP Multi-system 1.0` を追加する
