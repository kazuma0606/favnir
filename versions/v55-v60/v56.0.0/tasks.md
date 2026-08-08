# Tasks — v56.0.0 — Streaming Native 2.0 宣言 ★クリーンアップ

## ステータス: COMPLETE（2026-07-25）

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v55.1-v56.0.md` の v56.0.0 セクションを確認
- [x] ベーステスト数 3224（v55.9.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が `55.9.0` であることを確認（更新前）
- [x] `site/content/docs/streaming-native2-overview.mdx` が存在することを確認（v55.9.0 で追加済み）
- [x] `driver.rs` の `v55900_tests` に `cargo_toml_version_is_55_9_0` が含まれることを確認（v56.0 で FAIL するため削除対象）
- [x] `include_str!` パス確認:
  - `../Cargo.toml` → `fav/Cargo.toml`
  - `../../CHANGELOG.md` → `favnir/CHANGELOG.md`
  - `../../MILESTONE.md` → `favnir/MILESTONE.md`
  - `../../README.md` → `favnir/README.md`
- [x] CI self-lint 対象（`self/compiler.fav` / `self/checker.fav`）に影響しないことを確認（driver.rs テスト追加・Cargo.toml バージョン更新・MD ファイル更新のみ）
- [x] `fav/tmp/hello.fav` は `target/` ではなく `tmp/` にあるため `cargo clean` 後も存在することを確認

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `56.0.0` に更新
- [x] T2: `MILESTONE.md` に v56.0.0 — Streaming Native 2.0 宣言文エントリを追加
  - [x] `"Streaming Native 2.0"` キーワードを含む
  - [x] 宣言文（引用ブロック）を含む（ロードマップ記載の宣言文をそのまま使用）
- [x] T3: `README.md` に v56.0 Streaming Native 2.0 マイルストーンを追記
  - [x] `"Streaming Native 2.0"` キーワードを含む
- [x] T4: `CHANGELOG.md` に v56.0.0 エントリを追加
  - [x] `"v56.0.0"` を含む
- [x] T5: `fav/src/driver.rs` に `v56000_tests` モジュールを追加（`v55900_tests` の直前）
  - [x] `cargo_toml_version_is_56_0_0`（Cargo.toml バージョン検証）
  - [x] `changelog_has_v56_0_0`（CHANGELOG.md エントリ検証）
  - [x] `milestone_has_streaming_native2`（MILESTONE.md キーワード検証）
  - [x] `readme_mentions_streaming_native2`（README.md キーワード検証）
- [x] T6: `v55900_tests` から `cargo_toml_version_is_55_9_0` を削除（v56.0 で FAIL するため）

---

## テスト・検証

- [x] T7: `cargo build` でコンパイルエラーがないことを確認（`Finished` を確認）
- [x] T8: `cargo test` 全通過（**3227 tests passed, 0 failed**）
  - `v56000_tests::cargo_toml_version_is_56_0_0` ok
  - `v56000_tests::changelog_has_v56_0_0` ok
  - `v56000_tests::milestone_has_streaming_native2` ok
  - `v56000_tests::readme_mentions_streaming_native2` ok
- [x] T9: `cargo clippy -- -D warnings` クリーン

---

## クリーンアップ（★）

- [x] T10: `cargo clean` 実施（ビルドキャッシュ完全削除 — 33.3GB 削除）
- [x] T11: `fav/tmp/hello.fav` 存在確認（`cargo clean` 後も `tmp/` に残存）

---

## ポスト処理

- [x] T12: `versions/current.md` を v56.0.0 / 3227 tests に更新
- [x] T13: `versions/roadmap/roadmap-v55.1-v56.0.md` の v56.0.0 実績を COMPLETE に更新（目標テスト数も 3227 に修正）
- [x] T14: `versions/roadmap/roadmap-v55.1-v60.0.md` の v56.0.0 実績欄も COMPLETE に更新（目標テスト数も 3227 に修正）

---

## コードレビュー

- [x] コードレビュー実施（`/review code`）
- [x] 指摘事項対応
  - 指摘なし（レビュー完了 — 問題なし）

---

## 完了確認

- [x] `cargo_toml_version_is_56_0_0` pass
- [x] `changelog_has_v56_0_0` pass
- [x] `milestone_has_streaming_native2` pass
- [x] `readme_mentions_streaming_native2` pass
- [x] **3227 tests passed, 0 failed**
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `MILESTONE.md` に v56.0.0 — Streaming Native 2.0 宣言文エントリが含まれる
- [x] `CHANGELOG.md` に v56.0.0 エントリが追加されている
- [x] `versions/current.md` が v56.0.0 / 3227 tests を反映
- [x] T13 / T14 のロードマップ更新が完了している（目標テスト数 3227 に修正済み）
- [x] `cargo clean` 完了（33.3GB 削除）
- [x] `fav/tmp/hello.fav` 存在確認済み
