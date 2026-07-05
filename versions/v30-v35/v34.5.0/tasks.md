# v34.5.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `34.4.0` であること
- [x] `benchmarks/v34.4.0.json` の `tests_passed` が 2556 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2556 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v345000_tests` が存在しないこと
- [x] v34.4.0 が COMPLETE であること
- [x] `cargo_toml_version_is_34_4_0` が v344000_tests 内に存在すること（スタブ化対象）
  ```bash
  grep -A3 "cargo_toml_version_is_34_4_0" fav/src/driver.rs | head -5
  # assert! が残っていること（スタブ化前）を確認
  ```
- [x] `cargo test --bin fav v344000` が 5/5 PASS であること
- [x] `lint.rs` に W022 が存在しないこと（新規追加対象）
  ```bash
  grep "W022" fav/src/lint.rs | wc -l
  # 0 であることを確認
  ```
- [x] `runes/ctx/io.fav` が存在しないこと（新規作成対象）
  ```bash
  ls runes/ctx/io.fav 2>/dev/null || echo "does not exist"
  # does not exist であることを確認
  ```
- [x] `site/content/docs/tools/migration-effects.mdx` が存在しないこと（新規作成対象）
  ```bash
  ls site/content/docs/tools/migration-effects.mdx 2>/dev/null || echo "does not exist"
  # does not exist であることを確認
  ```
- [x] 既存 lint テストへの W022 影響を調査すること（plan.md Step 2 参照）
  ```bash
  grep -n "fav_lint\|lint_program" fav/src/driver.rs | grep -v "//" | head -40
  # !Effect 付き fn を含むテストソースで warnings.len() を count しているものを確認
  ```

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `34.4.0` → `34.5.0` に更新
- [x] **T2** `fav/src/lint.rs` — `check_w022_deprecated_effect_annotation` を追加し `lint_program()` に組み込む
- [x] **T3** 既存 lint テストへの影響修正（`lint_clean_file_no_errors` の `!Io` fn を純粋関数に変更）
- [x] **T4** `runes/ctx/io.fav` — `IoCtx` interface を新規作成
- [x] **T5** `site/content/docs/tools/migration-effects.mdx` — 移行ガイドを新規作成（W022 / `fav migrate --from-effects` / Before/After 例 / 対応表 / AppCtx 例）
- [x] **T6** `fav/src/driver.rs` — `cargo_toml_version_is_34_4_0` をスタブ化
- [x] **T7** `fav/src/driver.rs` — `v345000_tests`（5 件）を追加
        挿入位置: `v344000_tests` 直後・`// ── v31.7.0 tests` の前
        `use super::*` なし（絶対 `crate::` パスを使用。W021 テストパターンと同一）
- [x] **T8** `CHANGELOG.md` — `[v34.5.0]` セクションを先頭に追記
- [x] **T9** `benchmarks/v34.5.0.json` — 新規作成（暫定 `tests_passed`: 2561）
- [x] **T10** `versions/current.md` — 「最新安定版」欄を v34.5.0 に更新

---

## テスト確認

- [x] **T11** `cargo test --bin fav v345000 2>&1 | tail -8` — 5/5 PASS
- [x] **T12** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2561 passed、0 failures）

---

## 完了処理

- [x] **T13** `benchmarks/v34.5.0.json` の `tests_passed` を実測値で更新（実測値 2561 = 想定値と一致）
- [x] **T14** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `cargo clean` 不要（x.5.0 のため実施しない）
- [x] `Cargo.toml` version = `"34.5.0"`
- [x] `cargo_toml_version_is_34_4_0` が空スタブになっていること
- [x] `cargo test --bin fav v345000` — 5/5 PASS
- [x] `cargo test` — 全件 PASS（2561 件想定 = 2556 + 5、0 failures）
- [x] `lint.rs` に `check_w022_deprecated_effect_annotation` があり `lint_program()` に組み込まれていること
- [x] `runes/ctx/io.fav` が存在し `"IoCtx"` を含むこと
- [x] `site/content/docs/tools/migration-effects.mdx` が存在し `"W022"` を含むこと
- [x] `site/content/docs/tools/migration-effects.mdx` が `"AppCtx"` または `"ctx"` を含むこと
- [x] `CHANGELOG.md` に `[v34.5.0]` セクション
- [x] `benchmarks/v34.5.0.json` 存在かつ `tests_passed` が実測値（2561）
- [x] `versions/current.md` が v34.5.0 に更新されていること
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v345000_tests` に `use super::*` が**ない**こと（絶対 `crate::` パスを使用）
- [x] `w022_deprecated_effect_annotation_fires` が Lexer + Parser + `crate::lint::check_w022_deprecated_effect_annotation` パターンを使用していること（W021 テストと同一パターン）
- [x] `w022_deprecated_effect_annotation_fires` のアサーションが `w.code == "W022"` を使用していること
- [x] `io_ctx_rune_exists` / `migration_guide_page_exists` / `migration_guide_covers_ctx_syntax` は `include_str!` のみ使用
- [x] `cargo_toml_version_is_34_4_0` が空スタブになっていること（コメント付き）
- [x] W022 が `lint_program()` の W021 呼び出しの直後に追加されていること
- [x] `check_w022_deprecated_effect_annotation` の LintError コードが `"W022"` であること
- [x] 挿入位置が `v344000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v34.5.0.json` の `milestone` が `"Production Ready"` であること
- [x] `versions/current.md` が v34.5.0 に更新されていること
- [x] `runes/ctx/io.fav` が `IoCtx` interface の 5 メソッド（println / read_line / read_file / write_file / env）を含むこと
- [x] migration-effects.mdx に `fav migrate --from-effects` の使い方が記載されていること
- [x] migration-effects.mdx に Before/After コード例があること
