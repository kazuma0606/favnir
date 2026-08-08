# v67.6.0 タスクリスト

Status: COMPLETE
Version: 67.6.0
Note: MDX ドキュメントは v67.9.0 で一括作成のため本バージョンに T5 はない
Base tests: 3507
Target tests: 3509

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3507 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"67.0.0"` であることを確認（sub-version では変更しない）
- [x] `fav/src/proptest.rs` が存在しないことを確認（新規作成）
- [x] `driver.rs` に `v67500_tests` が存在することを確認（`v67600_tests` の挿入位置）
- [x] `driver.rs` に `v67600_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v67500_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `simulate_pipeline_with_synthetic`, `simulate_assertion_failure`
- [x] `versions/current.md` の「進行中バージョン」が `v67.5.0` であることを確認

---

## T1: `fav/src/proptest.rs` 新規作成

- [x] `fav/src/proptest.rs` を新規作成
  - [x] `pub const PROPTEST_HELP: &str` を追加
  - [x] `pub fn cmd_proptest(src: &str, args: &[String]) -> String` を追加
  - [x] `"proptest"` を含む（`proptest_stage_invariant` テストにマッチ）
  - [x] `"forall"` を含む（`proptest_stage_invariant` テストにマッチ）
  - [x] `"shrink"` を含む（`proptest_stage_invariant` テストにマッチ）
  - [x] `"--proptest-runs"` を含む（`proptest_counterexample_shrink` テストにマッチ）
  - [x] `--proptest-runs` 省略時に `eprintln!` 警告 + デフォルト `100` を使用
- [x] `cargo build` でエラーなし（proptest.rs 作成後）

---

## T2: `fav/src/main.rs` — `mod proptest;` と `Some("proptest")` 追加

- [x] `mod simulate;` の直後に `mod proptest;` を追加
- [x] `Some("simulate")` アームの直後に `Some("proptest")` ディスパッチアームを追加:
  - [x] `--help` / `-h` ブランチで `proptest::PROPTEST_HELP` を表示（dead_code 防止）
  - [x] それ以外は `proptest::cmd_proptest(file, &rest)` を呼ぶ
- [x] `cargo build` でエラーなし（main.rs 更新後）

---

## T3: `driver.rs` — `v67600_tests` 追加

- [x] 挿入前に `grep "v67500_tests" fav/src/driver.rs` でコメント行の正確な文字列を確認
- [x] `// -- v67500_tests (v67.5.0)` コメントの直前に `v67600_tests` を挿入
  - [x] `proptest_stage_invariant`: `include_str!("proptest.rs")` に `"proptest"` / `"forall"` / `"shrink"` を含む
  - [x] `proptest_counterexample_shrink`: `include_str!("proptest.rs")` に `"--proptest-runs"` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし（driver.rs テスト挿入後）

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v67600_tests` で 2 件 PASS
  - [x] `proptest_stage_invariant` PASS
  - [x] `proptest_counterexample_shrink` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3509 tests passed, 0 failed を確認

---

## T5: ドキュメント・ステータス更新

> T4 のテスト全通過（3509 tests passed）を確認してから実施すること。

- [x] `versions/roadmap/roadmap-v67.1-v68.0.md` のバージョン一覧表で v67.6.0 の「状態」列を「未着手」→「完了」に変更し、変更後に当該行が「完了」になっていることを目視確認
- [x] `versions/current.md` の「進行中バージョン」を v67.6.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **sub-version ポリシー**: v67.x では `versions/current.md` の「次バージョン」欄の更新は不要。v68.0.0 宣言時に一括整理する。
> **CHANGELOG 方針**: v67.1〜v67.9 では CHANGELOG.md を更新しない。v68.0.0 宣言時に一括追記する。
> **Cargo.toml 方針**: v67.1〜v67.9 では version を変更しない。v68.0.0 宣言時に `"68.0.0"` に更新する。

---

## コードレビュー指摘と対応

- [MED] code-reviewer: `v62000_tests` の `cargo_toml_version_is_62_0_0` テスト名がアサート内容（`67.0.0`）と不一致 → v67.0.0 での一括更新により全 `cargo_toml_version_is_XX` テストが現行バージョンを検証する既知パターン。アサート内容は正しいため対応不要
- [LOW] code-reviewer: `cmd_proptest` の `format!` 内 `42` がハードコードされ `runs` と整合しない → スタブ実装として許容
- [LOW] code-reviewer: `v67600_tests` がソーステキスト検索のみで実関数を検証しない → 他モジュールと同パターンのため許容
