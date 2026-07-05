# v34.5A — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `35.0.0` であること
- [x] `benchmarks/v35.0.0.json` の `tests_passed` が 2586 であることを確認
- [x] `driver.rs` に `mod v35100_tests` が存在しないこと
- [x] v35.0.0 が COMPLETE であること
- [x] `cargo_toml_version_is_35_0_0` が v350000_tests 内に存在すること（スタブ化対象）
- [x] `ast.rs` に `is_deprecated` が存在しないこと（追加対象）
- [x] `checker.rs` に `is_deprecated` 呼び出しが存在しないこと（追加対象）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `35.0.0` → `35.1.0` に更新
- [x] **T2** `fav/src/ast.rs` — `Effect::is_deprecated()` メソッドを追加
- [x] **T3** `fav/src/middle/checker.rs` — FnDef チェック時に `!Effect` deprecation 警告を発行
       `check_fn_def` 先頭の `check_effects_declared` 直後に `self.type_warning("W022", ...)` を追加
- [x] **T4** `fav/src/driver.rs` — `cargo_toml_version_is_35_0_0` をスタブ化
- [x] **T5** `fav/src/driver.rs` — `v35100_tests`（5 件）を追加
       挿入位置: `v350000_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし、絶対 `crate::` パス使用
       パス修正: `include_str!("ast.rs")` / `include_str!("middle/checker.rs")`（driver.rs 相対）
- [x] **T6** `CHANGELOG.md` — `[v35.1.0]` セクションを先頭に追記
- [x] **T7** `benchmarks/v35.1.0.json` — 新規作成（`tests_passed`: 2591）
- [x] **T8** `versions/current.md` — 最新安定版を v35.1.0 に更新

---

## テスト確認

- [x] **T9** `cargo test --bin fav v35100 2>&1 | tail -8` — 5/5 PASS
- [x] **T10** `cargo test 2>&1 | grep "test result"` — 2591 passed、0 failures
- [x] **T11** `cargo clippy --locked -- -D warnings` — warnings なし（Finished のみ）

---

## 完了処理

- [x] **T12** `benchmarks/v35.1.0.json` の `tests_passed` を実測値（2591）で確定
- [x] **T13** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"35.1.0"`
- [x] `cargo_toml_version_is_35_0_0` が空スタブになっていること
- [x] `ast.rs` に `Effect::is_deprecated()` が定義されていること
- [x] `checker.rs` に `is_deprecated()` 呼び出しが存在すること
- [x] `cargo test --bin fav v35100` — 5/5 PASS
- [x] `cargo test` — 全件 PASS（0 failures）
- [x] `cargo clippy --locked -- -D warnings` — PASS
- [x] `CHANGELOG.md` に `[v35.1.0]` セクション
- [x] `benchmarks/v35.1.0.json` 存在かつ `tests_passed` が実測値（2591）
- [x] `benchmarks/v35.1.0.json` の `tests_failed` が `0`
- [x] `versions/current.md` が v35.1.0 に更新されていること
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v35100_tests` に `use super::*` が**ない**こと
- [x] `v35100_tests` に不要な import 文が**ない**こと
- [x] `Effect::is_deprecated()` が `Pure` に対して `false` を返すこと
- [x] `Effect::Http` など非 Pure に対して `true` を返すこと
- [x] checker.rs の変更が `self.type_warning("W022", msg, &span)` を使用していること（`CheckWarning` 構造体を使っていないこと）
- [x] checker.rs の変更がエラーではなく**警告**として発行していること
- [x] 既存の `!Effect` チェックロジックを**削除していない**こと（移行期間）
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v35.1.0.json` の `tests_failed` が `0` であること
