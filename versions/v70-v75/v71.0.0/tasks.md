# v71.0.0 タスクリスト — Language Complete 1.0 宣言

Date: 2026-08-09
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `70.9.0` であることを確認
- [x] `cargo test` が全 pass（3580 tests）であることを確認
- [x] `MILESTONE.md` に "Language Complete" が未記載であることを確認
- [x] `README.md` に "Language Complete" が未記載であることを確認
- [x] driver.rs に `v71000_tests` が未存在であることを確認

---

## T1: cargo clean ★クリーンアップ

- [x] `cd fav && cargo clean` を実行してビルド生成物を削除する

---

## T2: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"70.9.0"` → `"71.0.0"` に変更する
- [x] driver.rs 内の全バージョン文字列（`"70.9.0"`）を `"71.0.0"` に一括更新する（replace_all）
  - 注: `replace_all` の対象は文字列リテラル `"70.9.0"` のみ。関数名・モジュール名（シンボル）は変更しない

---

## T3: MILESTONE.md 更新

- [x] `MILESTONE.md` の先頭（v70.0.0 エントリの直前）に v71.0.0 エントリを追加する
- [x] エントリに宣言文・達成内容（v70.1〜v70.9）を含める
- [x] `MILESTONE.md` に "Language Complete" が含まれることを目視確認する

---

## T4: README.md 更新

- [x] `README.md` の v70.0 セクションの直後に v71.0 セクションを追加する
- [x] "Language Complete" という文字列を含めること
- [x] `README.md` に "Language Complete" が含まれることを目視確認する

---

## T5: driver.rs に `v71000_tests` モジュールを追加

- [x] driver.rs 末尾（`v709000_tests` の直後）に `v71000_tests` モジュールを追加する
- [x] 以下の 4 テストを実装する:
  - [x] `cargo_toml_version_is_71_0_0` — `include_str!("../Cargo.toml")` で `"71.0.0"` を確認
  - [x] `changelog_has_v71_0_0` — `include_str!("../../CHANGELOG.md")` で `[v71.0.0]` を確認
  - [x] `milestone_has_language_complete` — `include_str!("../../MILESTONE.md")` で `Language Complete` を確認
  - [x] `readme_mentions_language_complete` — `include_str!("../../README.md")` で `Language Complete` を確認
- [x] `cargo test v71000` で 4 件 pass することを確認

---

## T6: CHANGELOG.md 更新

- [x] `CHANGELOG.md` の先頭（v70.9.0 エントリの直前）に v71.0.0 エントリを追加する
- [x] エントリに以下を含める:
  - Added: `v71000_tests` 4 件（3580 → 3584 tests）
  - Added: MILESTONE.md v71.0.0 Language Complete 1.0 エントリ追加
  - Added: README.md v71.0 セクション追加

---

## T7: versions/current.md 更新

- [x] 「進行中バージョン」を `v71.0.0`（Language Complete 1.0 宣言）に更新する
- [x] 「次に切る版」を `v71.1.0` に更新する

---

## T8: 最終確認

- [x] `cargo test v71000` で 4 件 pass することを確認
- [x] `cargo test` 全体で 3584 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `71.0.0` であることを確認
- [x] `MILESTONE.md` に "Language Complete" が含まれることを確認
- [x] `README.md` に "Language Complete" が含まれることを確認
- [x] `versions/current.md` が v71.0.0 に更新されていることを確認

---

## コードレビュー指摘対応

（実装後に記録）

---

## 完了チェックリスト

- [x] 全タスク（T0〜T8）が完了している
- [x] `cargo_toml_version_is_71_0_0` が pass
- [x] `changelog_has_v71_0_0` が pass
- [x] `milestone_has_language_complete` が pass
- [x] `readme_mentions_language_complete` が pass
- [x] テスト総数: 3584（+4）
- [x] Language Complete 1.0 宣言が MILESTONE.md・README.md に記録されていることを確認
