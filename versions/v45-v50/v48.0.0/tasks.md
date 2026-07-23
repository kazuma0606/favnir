# Tasks: v48.0.0 — Standard Library 2.0 宣言 ★クリーンアップ

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3041 passed, 0 failed を確認
- [x] `MILESTONE.md` に `"Standard Library 2.0"` が存在しないことを確認（新規追加対象）
- [x] `README.md` に `"Standard Library 2.0"` が存在しないことを確認（新規追加対象）

## T1 — 宣言ドキュメント更新

- [x] `MILESTONE.md` に v48.0.0 Standard Library 2.0 エントリを先頭に追加
  - [x] 宣言文（`> 「List・String・Float...」`）
  - [x] 達成コンポーネント一覧（v47.1〜v47.9 の 9 件）
- [x] `README.md` に `"Standard Library 2.0"` への言及を追加（v47.0 エントリの直後）

## T2 — `driver.rs`: `v48000_tests` 追加

- [x] `v48000_tests` モジュールを `v479000_tests` の直前に追加（4テスト）
  - [x] `cargo_toml_version_is_48_0_0`: `../Cargo.toml` に `version = "48.0.0"` が含まれる
  - [x] `changelog_has_v48_0_0`: `../../CHANGELOG.md` に `[v48.0.0]` が含まれる
  - [x] `milestone_has_stdlib_v2`: `../../MILESTONE.md` に `"Standard Library 2.0"` が含まれる
  - [x] `readme_mentions_stdlib_v2`: `../../README.md` に `"Standard Library 2.0"` が含まれる

## T3 — バージョン更新・テスト・完了

- [x] `fav/Cargo.toml` version → `"48.0.0"`
- [x] `CHANGELOG.md` に v48.0.0 エントリ追加
- [x] `cargo test` 3045 passed, 0 failed（3041 + 4 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v48.0.0（3045 tests）に更新、進行中バージョンを `v48.1.0` に更新
- [x] `versions/roadmap/roadmap-v47.1-v48.0.md` の v48.0.0 完了条件テスト数（3045）を実績で確認
- [x] `versions/roadmap/roadmap-v45.1-v50.0.md` に v48.0.0 完了を反映（実績 3045 tests）
- [x] tasks.md を COMPLETE に更新（T0〜T4 全 `[x]`）

## T4 — ★クリーンアップ

- [x] `cargo clean` 実施
- [x] `cargo clean` 後 `fav/tmp/hello.fav` の存在を確認（消えていなかった）
- [x] `cargo test` 再実行（クリーン後も 3045 passed, 0 failed を確認）

> **注記**: マスターロードマップ（`roadmap-v45.1-v50.0.md`）への反映は本バージョン（v48.0.0）完了時に実施済み

---

## コードレビュー指摘と対応（spec-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [MED] | plan.md 実装順序にマスターロードマップ更新ステップが欠落 | Step 9 として `roadmap-v45.1-v50.0.md` 更新を追加 |
| [MED] | tasks.md T3 にマスターロードマップ更新チェックボックスが欠落 | T3 に `roadmap-v45.1-v50.0.md` 更新チェックボックスを追加 |
| [LOW] | plan.md 挿入位置の行番号が v479000_tests 追加後の実態とズレ | `（現在 47076 行付近）` の記載を削除 |
