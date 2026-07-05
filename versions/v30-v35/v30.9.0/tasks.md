# v30.9.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `30.8.0` であること
- [x] `cargo test 2>&1 | grep "test result"` が `2415 passed` を含むこと
- [x] `driver.rs` に `mod v309000_tests` が存在しないこと
- [x] `toml.rs` に `"project"` セクション認識がないこと（`section = "project"` がないこと）
- [x] v30.8.0 が COMPLETE であること

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `30.8.0` → `30.9.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_30_8_0` をスタブ化
- [x] **T3** `fav/src/toml.rs` — `[project]` セクション認識を追加（`"rune" | "project"` アーム）
- [x] **T4** `fav/src/driver.rs` — `load_all_items` 非 rune `ImportDecl` を `root.join(import_name)` で解決（`src_dir.join` から変更）
- [x] **T5** `fav/src/driver.rs` — `fav test` false 返却時メッセージに hint を追加
- [x] **T6** `fav/src/main.rs` — `fav new` 引数なし時エラーに `fav new --list` ヒントを追加
- [x] **T7** `fav/src/driver.rs` — `v309000_tests`（3 件）を追加（`use super::*` なし）
- [x] **T8** `CHANGELOG.md` — `[v30.9.0]` セクションを先頭に追記
- [x] **T9** `benchmarks/v30.9.0.json` — 新規作成（tests_passed: 2418）
- [x] **T10** `versions/current.md` — 「最新安定版」欄を v30.9.0 に更新

---

## テスト確認

- [x] **T11** `cargo test --bin fav v309000 2>&1 | tail -8` — 3/3 PASS
- [x] **T12** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2418 passed、0 failures）

---

## 完了処理

- [x] **T13** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"30.9.0"`
- [x] `toml.rs` が `[project]` セクションを認識し `src` フィールドをパースする
- [x] `load_all_items` の非 rune ImportDecl が `root.join(import_name)` で解決される
- [x] `fav test` false 返却時に `assert_eq!` / `assert!` ヒントが表示される
- [x] `fav new`（引数なし）に `fav new --list` ヒントが表示される
- [x] `cargo test v309000` — 3/3 PASS
- [x] `cargo test` — 全件 PASS（2418 passed）
- [x] `CHANGELOG.md` に `[v30.9.0]` セクション
- [x] `benchmarks/v30.9.0.json` 存在
- [x] `versions/current.md` を v30.9.0 に更新
- [x] tasks.md を COMPLETE に更新

---

## コードレビューチェックリスト

- [x] `[project]` セクション認識が `if trimmed.starts_with('[') { section = ""; }` の直前に追加されていること
- [x] `"rune" | "project"` アームの内容が同一であること（`edition` は `_ => {}` で無視）
- [x] `load_all_items` の rune import（`is_rune: true`）アームが変更されていないこと
- [x] `src_dir` 変数が `program.uses` 解決に引き続き使われていること（削除されていない）
- [x] `v309000_tests` に `use super::*` がないこと（`crate::toml::parse_fav_toml_pub(...)` フルパス参照）
- [x] `project_section_sets_src_dir` が `toml.src == "src"` を検証していること
- [x] `v309000_tests` に `benchmark_v30_9_0_exists` テストがあること
- [x] `fav new` の hint 追加が `--list` ハンドラより後（`args.get(2)` unwrap 内）であること

---

## コードレビュー指摘・対応記録

spec-reviewer 指摘 2 件をすべて spec/plan/tasks に反映:
- [MED] `use super::*` の説明誤り → spec/tasks/plan から削除、フルパス参照（`crate::toml::parse_fav_toml_pub(...)`）を明記。実装も `use super::*` なしで実施
- [LOW] Fix 2 後方互換性説明の不正確さ → spec を「Fix 1 + Fix 2 の組み合わせで `src_dir` 値に依存しない」と正確な記述に修正
