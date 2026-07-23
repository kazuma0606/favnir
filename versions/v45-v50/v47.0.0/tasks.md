# Tasks: v47.0.0 — Developer Experience 宣言 ★クリーンアップ

Status: COMPLETE
Date: 2026-07-17

---

## T0 — 事前確認

- [x] `cargo test` 3012 passed, 0 failed を確認

## T1 — 宣言ドキュメント更新

- [x] `MILESTONE.md` に v47.0.0 Developer Experience エントリを先頭に追加
  - [x] 宣言文（`> 「インラインテスト...」`）
  - [x] 達成コンポーネント一覧（v46.1〜v46.9 の 9 件）
- [x] `README.md` に `"Developer Experience"` への言及を追加

## T2 — `driver.rs`: `v47000_tests` 追加

- [x] `v47000_tests` モジュールを `v469000_tests` の直後に追加（4テスト）
  - [x] `cargo_toml_version_is_47_0_0`: `../Cargo.toml` に `version = "47.0.0"` が含まれる
  - [x] `changelog_has_v47_0_0`: `../../CHANGELOG.md` に `[v47.0.0]` が含まれる
  - [x] `milestone_has_developer_experience`: `../../MILESTONE.md` に `"Developer Experience"` が含まれる
  - [x] `readme_mentions_developer_experience`: `../../README.md` に `"Developer Experience"` が含まれる

## T3 — バージョン更新・テスト・完了

- [x] `fav/Cargo.toml` version → `"47.0.0"`
- [x] `CHANGELOG.md` に v47.0.0 エントリ追加
- [x] `cargo test` 3016 passed, 0 failed（3012 + 4 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v47.0.0（3016 tests）に更新
- [x] tasks.md を COMPLETE に更新（T0〜T4 全チェック）

## T4 — ★クリーンアップ

- [x] `cargo clean` 実施（30.8 GiB、31475 ファイル削除）
- [x] `cargo clean` 後 `fav/tmp/hello.fav` の存在を確認（内容正常、消えていなかった）
- [x] `cargo test` 再実行（クリーン後も 3016 passed, 0 failed を確認）

---

## コードレビュー指摘と対応（spec-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [HIGH] | v47000_tests の挿入位置が「v46000_tests の直前」と誤記 | plan.md を「v469000_tests の直後」に修正 |
| [MED] | 注意事項の `include_str!` パス説明が冗長 | パスまとめ表形式に整理 |
| [MED] | v46000_tests の空実装との差異が未記載 | plan.md 注意事項に「空にしないこと」を追記 |
| [LOW] | tasks.md T4 に `hello.fav` 復元手順が未記載 | T4 にサブチェックボックスで復元手順を追加 |
