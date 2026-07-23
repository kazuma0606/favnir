# Tasks: v49.0.0 — Module & Package 2.0 宣言 ★クリーンアップ

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3065 passed, 0 failed を確認（ベース確認）
- [x] `MILESTONE.md` に `"Module & Package 2.0"` が存在しないことを確認（新規追加対象）
- [x] `README.md` に `"Module & Package 2.0"` が存在しないことを確認（新規追加対象）

## T1 — 宣言ドキュメント更新

- [x] `MILESTONE.md` に v49.0.0 Module & Package 2.0 エントリを先頭に追加
  - [x] 宣言文（`> 「パッケージ import とローカル import が...」`）
  - [x] 達成コンポーネント一覧（v48.1〜v48.9 の 9 件）
- [x] `README.md` に `"Module & Package 2.0"` への言及を追加（v48.0 エントリの直後）

## T2 — `driver.rs`: `v49000_tests` 追加

- [x] `v49000_tests` モジュールを `v489000_tests` の直前に追加（4テスト）
  - [x] `cargo_toml_version_is_49_0_0`: `../Cargo.toml` に `version = "49.0.0"` が含まれる
  - [x] `changelog_has_v49_0_0`: `../../CHANGELOG.md` に `[v49.0.0]` が含まれる
  - [x] `milestone_has_module_package_v2`: `../../MILESTONE.md` に `"Module & Package 2.0"` が含まれる
  - [x] `readme_mentions_module_package_v2`: `../../README.md` に `"Module & Package 2.0"` が含まれる

## T3 — バージョン更新・テスト・完了

- [x] `fav/Cargo.toml` version → `"49.0.0"`
- [x] `CHANGELOG.md` に v49.0.0 エントリ追加
- [x] `cargo test` 3069 passed, 0 failed（3065 + 4 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v49.0.0（3069 tests）に更新、進行中バージョンを `v49.1.0` に更新
- [x] `versions/roadmap/roadmap-v48.1-v49.0.md` の v49.0.0 実績を記入
- [x] `versions/roadmap/roadmap-v45.1-v50.0.md` に v49.0 完了を反映（実績 3069 tests）

## T4 — ★クリーンアップ

- [x] `cargo clean` 実施（31792 files, 31.6 GiB 除去）
- [x] `cargo clean` 後 `fav/tmp/hello.fav` の存在を確認
- [x] `cargo test` 再実行（クリーン後も 3069 passed, 0 failed を確認）
- [x] tasks.md を COMPLETE に更新（T0〜T4 全 `[x]`）

---

> **注記**: `cargo clean` は T4 で実施（T3 の全テスト通過後）
> **注記**: マスターロードマップ（`roadmap-v45.1-v50.0.md`）への反映は本バージョン（v49.0.0）完了時に実施
