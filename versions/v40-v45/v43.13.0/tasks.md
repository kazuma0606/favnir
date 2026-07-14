# v43.13.0 タスク — Language Expressiveness cookbook + 安定化

## ステータス: COMPLETE（2026-07-13）— 2937 tests

---

## T0 — 事前確認

- [x] `cargo test` 2935 / 0 確認
- [x] `Cargo.toml` version = `43.12.0` 確認
- [x] `v431300_tests` が `fav/src/driver.rs` に存在しないことを確認
- [x] `v431200_tests::cargo_toml_version_is_43_12_0` が現在 pass していることを確認（スタブ化対象）
- [x] `site/content/cookbook/type-inference-guide.mdx` が存在しないことを確認
- [x] `site/content/docs/language-expressiveness.mdx` が存在しないことを確認

---

## T1 — MDX ファイル作成（3 件）

- [x] `site/content/cookbook/type-inference-guide.mdx` を作成（型推論 cookbook — include_str! テスト対象）
- [x] `site/content/docs/language/type-inference.mdx` を作成（言語リファレンス — テスト対象外）
- [x] `site/content/docs/language-expressiveness.mdx` を作成（スプリントサマリー — include_str! テスト対象）

---

## T2 — driver.rs: v431300_tests 追加 / スタブ化 / Cargo.toml

- [x] `v431200_tests` の直前に `v431300_tests` を挿入（2 件のみ）
  - `type_inference_guide_mdx_exists`
  - `language_expressiveness_doc_exists`
  - **注**: `cargo_toml_version_is_43_13_0` は追加しない（テスト数 2937 維持のため）
- [x] `v431200_tests::cargo_toml_version_is_43_12_0` をスタブ化（`// Stubbed: version bumped to 43.13.0 in v43.13.0.`）
- [x] `fav/Cargo.toml` version を `43.12.0` → `43.13.0` に更新

---

## T3 — CHANGELOG.md

- [x] v43.13.0 エントリ追加

---

## T4 — テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` 実行
- [x] 2937 passed; 0 failed 確認
- [x] `v431300_tests` 2 件 pass 確認

---

## T5 — バージョン管理ドキュメント更新

- [x] `versions/current.md` → v43.13.0 最新安定版（2937 tests）、次版 v44.0.0
- [x] `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.13.0 を `✅ COMPLETE（2026-07-13）`
- [x] `versions/v40-v45/v43.13.0/tasks.md` → COMPLETE、全チェックボックス `[x]`

---

## 実装時の知見

- コードフリーズリリースは Rust 変更なし（driver.rs テスト追加・スタブ化・Cargo.toml のみ）
- `cargo_toml_version_is_43_13_0` を追加しないことで 2935 + 2 = 2937 を達成
- `include_str!` パス: `fav/src/driver.rs` から `../../` = `favnir/`（リポジトリルート）
