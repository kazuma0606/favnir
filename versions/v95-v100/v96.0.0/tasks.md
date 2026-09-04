# Tasks: v96.0.0 — SAP Real-time 1.0 宣言

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v95.9.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v95900_tests` が存在することを確認する（v95.9.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,184 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `95.0.0` であることを確認する（更新前の状態確認）

## T1: `fav/Cargo.toml` のバージョンを更新

- [x] `version = "95.0.0"` → `version = "96.0.0"` に変更する

## T2: `driver.rs` に `mod v96000_tests` を追加

- [x] `mod v95900_tests` の直後に `#[cfg(test)] mod v96000_tests { ... }` を追加する
- [x] `cargo_toml_version_is_96_0_0` テストを追加する（`Cargo.toml` に `version = "96.0.0"` が含まれる）
- [x] `changelog_has_v96_0_0` テストを追加する（`CHANGELOG.md` に `v96.0.0` が含まれる）
- [x] `milestone_has_sap_realtime` テストを追加する（`MILESTONE.md` に `SAP Real-time` が含まれる）
- [x] `readme_mentions_sap_realtime` テストを追加する（`README.md` に `SAP Real-time` が含まれる）
- [x] 既存の `cargo_toml_version_is_X` テスト（42 件）を `"96.0.0"` に一括更新する

## T3: `CHANGELOG.md` に v96.0.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v96.0.0]` 宣言エントリを追加する

## T4: `MILESTONE.md` に v96.0.0 エントリを追加

- [x] `MILESTONE.md` の先頭に v96.0.0 SAP Real-time 1.0 エントリを追加する

## T5: `README.md` に v96.0 セクションを追加

- [x] `## v95.0 — SAP Advanced 1.0` の直前に `## v96.0 — SAP Real-time 1.0` セクションを追加する

## T6: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,188 tests, 0 failures であることを確認する

## T7: `cargo clean` ★クリーンアップ

- [x] `cargo clean` を実行する（11.1 GiB 削除）
- [x] `cargo test 2>&1 | grep "test result"` を再実行し、4,188 tests, 0 failures を再確認する（cargo clean 後）

## T8: `versions/current.md` 更新

- [x] 最新安定版を `v96.0.0` に更新する

## T9: tasks.md 更新

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする

## T-last: CI 事前確認（T6 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
