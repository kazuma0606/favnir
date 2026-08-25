# v84.0.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,905 tests pass、0 failures であることを確認する（前提: v83.9.0 完了済み）

## T1: `cargo clean`

- [x] `cd fav && cargo clean` を実施する
- [x] `cargo clean` 実施後、`fav/tmp/hello.fav` が存在することを確認する（存在しない場合は `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }` の内容で復元する）

## T2: `Cargo.toml` バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"83.0.0"` から `"84.0.0"` に変更する

## T3: ドキュメント更新

- [x] `CHANGELOG.md` の先頭に v84.0.0 エントリを追加する（宣言文・達成内容を含む）
- [x] `MILESTONE.md` に v84.0 — Observability 2.0 達成内容を追記する
- [x] `README.md` に `fav observe` の言及を追加する

## T4: `driver.rs` に `v84000_tests` を追加

- [x] `v83900_tests` の直後に `#[cfg(test)] mod v84000_tests` を追加する（`use` 文不要）
  - `cargo_toml_version_is_84_0_0`: `include_str!("../Cargo.toml")` に `"version = \"84.0.0\""` が含まれることを確認
  - `changelog_has_v84_0_0`: `include_str!("../../CHANGELOG.md")` に `"v84.0.0"` が含まれることを確認
  - `milestone_has_observability_2`: `include_str!("../../MILESTONE.md")` に `"Observability 2.0"` が含まれることを確認
  - `readme_mentions_fav_observe`: `include_str!("../../README.md")` に `"fav observe"` が含まれることを確認

## T5: テスト通過確認

- [x] `cargo test` が 3,909 tests pass（+4）、0 failures であることを確認する

## T6: ロードマップ・バージョン管理ファイル更新

- [x] `versions/current.md` の「現行マスターロードマップ」欄が `roadmap-v80.1-v85.0.md` を指していることを確認してから v84.0.0（3909 tests）に更新する
- [x] `versions/roadmap/roadmap-v80.1-v85.0.md` の Sprint 4 テーブル（v83.1〜v84.0 各行）を「完了」に更新し、テスト数を実際値に修正する

## T7: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
