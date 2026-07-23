# Tasks: v46.0.0 — Language Refinement 宣言

Status: COMPLETE
Date: 2026-07-16

---

## T0 — 事前確認

- [x] `cargo test` 2988 passed, 0 failed を確認

## T1 — `MILESTONE.md` 更新

- [x] v46.0.0「Language Refinement」エントリを先頭に追記
- [x] `"Language Refinement"` という文字列が含まれていることを確認
- [x] 達成コンポーネント（v45.1〜v45.9）テーブルを記載

## T2 — `README.md` 更新

- [x] `"Language Refinement"` を含む v46.0 達成の記述を追記

## T3 — `driver.rs`: v46000_tests 追加

- [x] `v46000_tests` モジュール追加（`v459000_tests` の直後）
- [x] `cargo_toml_version_is_46_0_0` テスト実装
- [x] `changelog_has_v46_0_0` テスト実装
- [x] `milestone_has_language_refinement` テスト実装
- [x] `readme_mentions_language_refinement` テスト実装

## T4 — バージョン更新・テスト

- [x] `fav/Cargo.toml` version → `46.0.0`
- [x] `cargo test` 2992 passed, 0 failed（2988 + 4件）
- [x] `cargo clippy -- -D warnings` クリーン

## T5 — `cargo clean` ★クリーンアップ

- [x] `cargo clean` 実行（32.5 GiB 削除）
- [x] `fav/tmp/hello.fav` 復元
- [x] `cargo test` 再実行（再ビルド後 2992 passed, 0 failed を確認）

## T6 — ドキュメント完了

- [x] `CHANGELOG.md` に v46.0.0 エントリ追加
- [x] `versions/current.md` を v46.0.0（2992 tests）に更新
- [x] tasks.md を COMPLETE に更新（T0〜T6 全チェック）

## コードレビュー指摘と対応

| 重大度 | 箇所 | 内容 | 対応 |
|---|---|---|---|
| [MED] | `versions/current.md` L21 | `cargo install` バージョンが `45.9.0` のまま（v46.0.0 宣言と不一致） | `46.0.0` に修正 |
