# Tasks: v46.2.0 — `fav test` コマンド: `#[test]` fn 対応

Status: COMPLETE
Date: 2026-07-16

---

## T0 — 事前確認

- [x] `cargo test` 2994 passed, 0 failed を確認

## T1 — `driver.rs`: `collect_test_cases` 修正

- [x] `_ => {}` の直前に `ast::Item::FnDef(fd) if fd.is_test => { ... }` アームを追加
- [x] `total_discovered += 1;` を含む
- [x] `filter` による名前フィルタを含む（`fd.name.contains(f)`）
- [x] `tests_to_run.push(path, fd.name, fd.name, prog)` — display_name と fn_name の両方が `fd.name`

## T2 — `driver.rs`: v462000_tests 追加

- [x] `v462000_tests` モジュール追加（`v461000_tests` の直後）
- [x] `fav_test_discovers_tests`: `collect_test_cases` が `#[test] fn` を 1 件発見することを確認
- [x] `fav_test_reports_results`: `super::build_artifact` でコンパイル・VM 実行し `Bool(false)` でないことを確認
- [x] `non_test_fn_not_discovered`: 通常 fn が収集されないことを確認（3件目の追加テスト）

## T3 — テスト＆完了

- [x] `cargo test` 2997 passed, 0 failed（2994 + 3件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `fav/Cargo.toml` version → `46.2.0`
- [x] `CHANGELOG.md` に v46.2.0 エントリ追加
- [x] `versions/current.md` を v46.2.0（2997 tests）に更新
- [x] tasks.md を COMPLETE に更新（T0〜T3 全チェック）

## コードレビュー指摘と対応

| 重大度 | 箇所 | 内容 | 対応 |
|---|---|---|---|
| [MED-1] | spec `fav_test_reports_results` | `compile_program` + `codegen_program` 直接呼び出しは `build_artifact` との等価性が未確認 | `super::build_artifact(&prog)` に統一 |
| [MED-2] | roadmap テスト数 | 2993 と spec の 2996 が不一致 | roadmap を 2996（実態 2997）に更新 |
| [LOW-1] | plan.md 行番号 | `driver.rs:4960` が 1 行ずれ | 行番号参照をなくし grep で確認する旨に修正 |
| [LOW-2] | spec 変更しないファイル | `cmd_test` 変更不要の根拠が未記載 | `driver.rs:5107` の構造を根拠として追記 |
