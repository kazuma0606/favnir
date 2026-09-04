# Tasks: v100.0.0 — Favnir SAP Platform 1.0 宣言 ★大クリーンアップ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/v95-v100/v100.0.0/` ディレクトリが存在することを確認する（存在しなければ作成する）
- [x] `versions/v95-v100/v99.9.0/tasks.md` の Status が `COMPLETE` であることを確認する
- [x] `versions/current.md` の最新安定版が `v99.9.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v99900_tests` が存在することを確認する（v99.9.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,275 であることを確認する（着手前ベースライン。異なる場合は `versions/v95-v100/v99.9.0/tasks.md` の実績テスト数を確認し、ベースライン値を修正してから着手する）
- [x] `fav/Cargo.toml` の version が `99.0.0` であることを確認する（宣言バージョン更新前の状態）
- [x] `fav/tmp/hello.fav` が存在することを確認する
- [x] `fav/tmp/hello.fav` の内容に `fn add` と `fn main` が含まれることを確認する

## T1: fav/Cargo.toml version を 100.0.0 に更新

- [x] `fav/Cargo.toml` の `version = "99.0.0"` を `version = "100.0.0"` に変更する
- [x] 変更後に `version = "100.0.0"` が含まれることを確認する

## T2: MILESTONE.md に v100.0.0 エントリを追加

- [x] `MILESTONE.md` のマイルストーン一覧に `v100.0 — SAP Platform 1.0` エントリを追加する
- [x] `SAP Platform` キーワードが `MILESTONE.md` に含まれることを確認する

## T3: README.md に SAP Platform 1.0 セクションを追加

- [x] `README.md` に `## v100.0 — Favnir SAP Platform 1.0` セクションを追加する
- [x] `SAP Platform` キーワードが `README.md` に含まれることを確認する

## T4: CHANGELOG.md に [v100.0.0] エントリを追加

**注意: T5（driver.rs テスト追加）より前に必ず完了すること**

- [x] `CHANGELOG.md` の先頭に `[v100.0.0]` エントリを追加する
- [x] `[v100.0.0]` キーワードが `CHANGELOG.md` に含まれることを確認する

## T5: driver.rs に mod v100000_tests を追加

**注意: T1〜T4 完了後に実施すること（各テストの前提条件が揃ってから）**

- [x] `mod v99900_tests` の直後に `mod v100000_tests`（4 テスト）を追加する:
  - `cargo_toml_version_is_100_0_0`: `include_str!("../Cargo.toml")` に `"100.0.0"` が含まれることを確認
  - `changelog_has_v100_0_0`: `../CHANGELOG.md` に `"[v100.0.0]"` が含まれることを確認（`std::fs::read_to_string` 使用。`include_str!` ではない）
  - `milestone_has_sap_platform`: `../MILESTONE.md` に `"SAP Platform"` が含まれることを確認
  - `readme_mentions_sap_platform`: `../README.md` に `"SAP Platform"` が含まれることを確認
- [x] `mod v100000_tests` ブロック先頭に `// use super::* は不要（外部シンボル未使用）` という Rust コメントを 1 行追記する
- [x] `cargo_toml_version_is_100_0_0` が `include_str!("../Cargo.toml")` を使用していることを確認する
- [x] 残り 3 テストが `std::fs::read_to_string` を使用していることを確認する

## T6: cargo test で全 pass 確認（大クリーンアップ前）

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,279 tests, 0 failures であることを確認する

## T7: ★大クリーンアップ（cargo clean → cargo test 再確認）

- [x] `cargo clean` を実行する（target/ ディレクトリを削除）
- [x] `fav/tmp/hello.fav` が削除されていないことを確認する
- [x] （消えていた場合）`fav/tmp/hello.fav` を `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }` の内容で復元する
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、cargo clean 後も 4,279 tests, 0 failures であることを再確認する
- [x] `cargo build` を実行し `./target/debug/fav` を再生成する（T-last の `fav fmt --check` の前提）

## T8: versions/current.md を v100.0.0 に更新

- [x] `最終更新:` ヘッダーを `v100.0.0` に更新する
- [x] 最新安定版を `v100.0.0` に更新する（テスト数 4,279）
- [x] `cargo install fav --version "100.0.0"` に更新する

## T9: SAP ロードマップファイルの Status を「完了」に更新

- [x] `versions/roadmap/roadmap-v99.1-v100.0.md` の v100.0.0 セクションまたはファイル冒頭の Status 行を「完了」に更新する
- [x] `versions/roadmap/roadmap-v95.1-v100.0.md` のファイル冒頭の Status 行を「完了」に更新する（存在する場合）

## T-last: CI 事前確認（T7 の `cargo clean` 後・`cargo build` 済みであること。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
