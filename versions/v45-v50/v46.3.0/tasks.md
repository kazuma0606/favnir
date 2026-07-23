# Tasks: v46.3.0 — assertion 拡充

Status: COMPLETE
Date: 2026-07-17

---

## T0 — 事前確認

- [x] `cargo test` 2997 passed, 0 failed を確認

## T1 — `checker.rs`: `check_test_def` に assert_ok/assert_err 追加

- [x] `check_test_def` の `assert_ne` 定義の直後に `assert_ok`/`assert_err` を追加
- [x] `Type::Unknown` を使用（他の assert_eq/assert_ne と同様）

## T2 — `checker.rs`: `check_fn_def` に is_test ガード追加

- [x] `check_fn_def` の `self.env.push()` 直後に `if fd.is_test { ... }` ブロックを追加
- [x] `assert`/`assert_eq`/`assert_ne`/`assert_ok`/`assert_err` の 5 種を登録

## T3 — `driver.rs`: `v463000_tests` 追加

- [x] `v462000_tests` の直後に `v463000_tests` モジュールを追加
- [x] `assert_ok_passes`: `assert_ok(Result.ok(42))` の `#[test] fn` が PASS すること
- [x] `assert_err_passes`: `assert_err(Result.err("oops"))` の `#[test] fn` が PASS すること

## T4 — テスト＆完了

- [x] `cargo test` 2999 passed, 0 failed（2997 + 2件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] checker 動作確認: `check_fn_def` の `fd.is_test` ガードにより assert_ok/assert_err が env に登録される（T2 の実装で対応）
- [x] `fav/Cargo.toml` version → `46.3.0`
- [x] `CHANGELOG.md` に v46.3.0 エントリ追加
- [x] `versions/current.md` を v46.3.0（2999 tests）に更新
- [x] tasks.md を COMPLETE に更新（T0〜T4 全チェック）

## コードレビュー指摘と対応

| 重大度 | 箇所 | 内容 | 対応 |
|---|---|---|---|
| [MED] | `v463000_tests` assert 条件 | `result != Value::Bool(false)` は弱い検証（何でもパスする） | `assert_eq!(result, Value::Int(42))` / `assert_eq!(result, Value::Str("oops"))` に強化 |
| [LOW] | `check_test_def` / `check_fn_def` | assert 登録が 2 箇所に重複（将来の拡張時に漏れるリスク） | 現時点では唯一の参照箇所につき共通ヘルパー化は過剰設計と判断し見送り |
