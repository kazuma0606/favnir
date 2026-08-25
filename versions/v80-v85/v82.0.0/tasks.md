# v82.0.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,861 tests pass、0 failures であることを確認する（前提: v81.9.0 完了済み）

## T1: `cargo clean`

- [x] `cargo clean` を実行してビルドキャッシュを削除する（8.2GiB 削除）

## T2: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"82.0.0"` に更新する

## T3: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v82.0.0 エントリを追加する（宣言文・Sprint Summary・テスト 4 件）

## T4: `v82000_tests` テストモジュール追加

- [x] `fav/src/driver.rs` の末尾に `#[cfg(test)] mod v82000_tests` を追加する
  - `cargo_toml_version_is_82_0_0` ✅
  - `changelog_has_v82_0_0` ✅
  - `milestone_has_data_quality_2` ✅
  - `readme_mentions_quality_gate` ✅

## T5: テスト通過確認（T4 実装後）

- [x] `cargo test --bin fav v82000` が 4 件 pass することを確認する

## T6: MILESTONE.md 更新

- [x] `MILESTONE.md` の先頭に Data Quality 2.0 宣言エントリを追加する（v81.1〜v81.9 達成内容リスト）

## T7: README.md 更新

- [x] `README.md` の先頭バージョンセクションを v82.0 に更新する（`QualityGate` 言及）

## T8: `versions/current.md` 更新

- [x] 最新安定版を `v82.0.0 — Data Quality 2.0 宣言 — 3865 tests（2026-08-20）` に更新する
- [x] 進行中バージョンを `v82.1.0〜v83.0.0（Pipeline Contracts 1.0 スプリント）` に更新する
- [x] マイルストーン進捗テーブルの v82.0 エントリ日付を `2026-08-20` に更新する

## T9: ロードマップ更新

- [x] `versions/current.md` が `roadmap-v80.1-v85.0.md` を指していることを確認する
- [x] `roadmap-v80.1-v85.0.md`: Sprint 2 テーブルの v82.0.0 行はすでに「完了」
- [x] `roadmap-v81.1-v82.0.md`: v82.0.0 行を「完了」に更新する

## T10: 全テスト通過確認

- [x] `cargo test` が 3,865 tests pass（+4）、0 failures であることを確認する

## T11: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

---

## コードレビュー指摘と対応

| 優先度 | 指摘 | 判断・対応 |
|---|---|---|
| [HIGH]（code-reviewer） | 旧バージョン 29 件の置換が「誤り」との指摘 | **却下**。プロジェクト慣例通り（Cargo.toml 版を更新するたびに全置換）。3,865 tests pass で正常動作確認済み |
| [LOW]（独自発見） | v81000_tests のアサーションメッセージが `"should have version 81.0.0"` のまま | メッセージを `"82.0.0"` に修正 ✅ |
