# Tasks: v52.0.0 — Performance & Scale 宣言

Status: COMPLETE
Date: 2026-07-20

---

## T0 — 事前確認

- [x] `cargo test` 3133 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `MILESTONE.md` に `"Performance & Scale"` が**存在しない**ことを確認（新規追加対象）
- [x] `README.md` に `"Performance & Scale"` が**存在しない**ことを確認（新規追加対象）
- [x] `v51900_tests` に `cargo_toml_version_is_51_9_0` が存在することを確認（削除対象）
- [x] `include_str!` パス確認:
  - [x] `../Cargo.toml` → `fav/Cargo.toml` ✓
  - [x] `../../CHANGELOG.md` → `favnir/CHANGELOG.md` ✓
  - [x] `../../MILESTONE.md` → `favnir/MILESTONE.md` ✓
  - [x] `../../README.md` → `favnir/README.md` ✓

## T1 — `MILESTONE.md` 更新

- [x] `MILESTONE.md` 先頭（v51.0.0 エントリの前）に v52.0.0 エントリを追加:
  - [x] `## v52.0.0（2026-07-20）— Performance & Scale` の見出しを含む
  - [x] 宣言文（「並列パイプラインはコアを使い切り...」）を含む
  - [x] `"Performance & Scale"` キーワードを含む（`milestone_has_performance_scale` テスト要件）

## T2 — `README.md` 更新

- [x] `README.md` に `"Performance & Scale"` への言及を追加:
  - [x] v51.0 の直前に v52.0 エントリを追加
  - [x] `"Performance & Scale"` キーワードを含む（`readme_mentions_performance_scale` テスト要件）

## T3 — `CHANGELOG.md` 更新

- [x] `CHANGELOG.md` 先頭に v52.0.0 エントリを追加:
  - [x] `## [v52.0.0]` の見出しを含む（`changelog_has_v52_0_0` テスト要件: `"v52.0.0"` が含まれる）
  - [x] 宣言文・追加機能一覧を記述

## T4 — `v52000_tests` 追加 + バージョン更新

- [x] `driver.rs` の `v51900_tests` 直前に `v52000_tests` モジュールを追加（4 件）:
  - [x] `cargo_toml_version_is_52_0_0`
  - [x] `changelog_has_v52_0_0`
  - [x] `milestone_has_performance_scale`
  - [x] `readme_mentions_performance_scale`
- [x] `v51900_tests` から `cargo_toml_version_is_51_9_0` を削除
- [x] `fav/Cargo.toml` version → `"52.0.0"`

## T5 — ★クリーンアップ（`cargo clean`）

- [x] `cd fav && cargo clean` を実行（32066 files, 31.1GiB 削除）
- [x] `cargo test` 実行 → `v51000_tests::code_freeze_v51_0_0` が FAILED（v51 番兵テストが 52.x で失敗）
  - [x] `code_freeze_v51_0_0` を `v51000_tests` から削除（v51 スプリント終了のため）
- [x] `cargo test` 3135 passed, 0 failed を確認（テスト数 ≥ 3135 の要件を満たす）
  - 計算: 3133 - 1（cargo_toml_version_is_51_9_0）+ 4（v52000_tests）- 1（code_freeze_v51_0_0）= 3135
- [x] `cargo clippy -- -D warnings` クリーンを確認

## T6 — 後処理

- [x] `versions/current.md` を v52.0.0（3135 tests）に更新
- [x] `roadmap-v51.1-v52.0.md` の v52.0.0 実績欄を更新
- [x] tasks.md を COMPLETE に更新（T0〜T6 全 `[x]`）
