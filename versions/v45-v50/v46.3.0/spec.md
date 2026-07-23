# Spec: v46.3.0 — assertion 拡充（`#[test] fn` で assert_ok/assert_err/assert_ne 利用可能化）

Date: 2026-07-17
Status: TODO

---

## 概要

v46.2.0 で `fav test` コマンドが `#[test] fn` を実行できるようになった。
しかし現状、`#[test] fn` の本体で `assert_ok`/`assert_err`/`assert_ne` を呼ぶと
型チェッカーが E0001（未定義変数）を報告する。

`assert_eq`/`assert_ok`/`assert_err`/`assert_ne` はすでに `vm.rs`・`compiler.rs` に実装済み。
不足しているのは **型チェッカー側の登録** のみ。

```favnir
#[test]
fn test_validate() {
  assert_ok(Result.ok(42))
  assert_err(Result.err("oops"))
  assert_ne(1, 2)
}
```

---

## 調査結果（実装前に確認済み）

### 現状の assert primitive 実装状況

| primitive    | vm.rs | compiler.rs PRIMITIVES | checker check_test_def | checker check_fn_def (is_test) |
|---|---|---|---|---|
| assert       | ✅    | ✅                     | ✅                     | ❌ 未登録                      |
| assert_eq    | ✅    | ✅                     | ✅                     | ❌ 未登録                      |
| assert_ne    | ✅    | ✅                     | ✅                     | ❌ 未登録                      |
| assert_ok    | ✅    | ✅                     | ❌ **欠落**            | ❌ 未登録                      |
| assert_err   | ✅    | ✅                     | ❌ **欠落**            | ❌ 未登録                      |

- `check_test_def`（`test "desc" {}` 用）: `assert_ok`/`assert_err` が未定義 → E0001
- `check_fn_def`（`#[test] fn` 用）: `fd.is_test` のとき assert 系が一切未定義 → E0001

### ロードマップとの対応

ロードマップには「`assert_eq` / `assert_ok` / `assert_err` / `assert_ne` を VM primitive として追加。
失敗時の diff メッセージも表示」とあるが、両方ともすでに完了済み:

- VM 実装: `vm.rs` に `assert_ok failed: got err(...)` 等の diff メッセージ付き実装済み（v15.3.0/v16.7.0）
- 失敗時メッセージ例:
  - `assert_eq`: `assert_eq failed:\n  actual:   {}\n  expected: {}`
  - `assert_ok`: `assert_ok failed: got err({msg})`
  - `assert_err`: `assert_err failed: got ok({msg})`
  - `assert_ne`: `assert_ne failed: both equal to {}`

本バージョンでの実作業は **型チェッカーへの登録** と **Rust テスト 2 件追加** のみ。

### テスト数

- v46.2.0 完了時: 2997（ロードマップ推定 2995 より +2 — v462000_tests が 3 件追加されたため）
  - ロードマップ推定 2996（v46.2.0）はさらに v461000_tests の実際の件数（2 件）に基づく推定で、
    実際は `non_test_fn_not_discovered` が追加され 3 件となった
- 本バージョン完了時推定: 2997 + 2 = **2999**

---

## 変更対象

### §1 — `checker.rs`: `check_test_def` に `assert_ok`/`assert_err` 追加

`check_test_def`（`test "desc" {}` 用スコープ）に欠落している 2 件を追加:

```rust
// 追加
self.env.define("assert_ok".to_string(), Type::Unknown);
self.env.define("assert_err".to_string(), Type::Unknown);
```

追加後の `check_test_def` は `assert`/`assert_eq`/`assert_ne`/`assert_ok`/`assert_err` の
5 種すべてを登録する。

### §2 — `checker.rs`: `check_fn_def` に `is_test` ガードで assert 登録

`check_fn_def` の `self.env.push()` 直後（パラメータ bind の前）に追加:

```rust
// v46.3.0: #[test] fn の本体で assert 系 primitive を利用可能にする
if fd.is_test {
    self.env.define("assert".to_string(), Type::Fn(vec![Type::Bool], Box::new(Type::Unit)));
    self.env.define("assert_eq".to_string(), Type::Unknown);
    self.env.define("assert_ne".to_string(), Type::Unknown);
    self.env.define("assert_ok".to_string(), Type::Unknown);
    self.env.define("assert_err".to_string(), Type::Unknown);
}
```

### §3 — `driver.rs`: `v463000_tests` 追加

`v462000_tests` の直後に `v463000_tests` モジュールを追加（2件）:

```rust
#[cfg(test)]
mod v463000_tests {
    use crate::frontend::parser::Parser;
    use crate::backend::vm::VM;
    use crate::value::Value;

    #[test]
    fn assert_ok_passes() {
        let src = r#"
            #[test]
            fn test_ok() {
                assert_ok(Result.ok(42))
            }
        "#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let artifact = super::build_artifact(&prog);
        let fn_idx = artifact.fn_idx_by_name("test_ok").expect("test_ok in artifact");
        let result = VM::run(&artifact, fn_idx, vec![]).expect("assert_ok should pass");
        assert!(result != Value::Bool(false), "assert_ok should pass (got {:?})", result);
    }

    #[test]
    fn assert_err_passes() {
        let src = r#"
            #[test]
            fn test_err() {
                assert_err(Result.err("oops"))
            }
        "#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let artifact = super::build_artifact(&prog);
        let fn_idx = artifact.fn_idx_by_name("test_err").expect("test_err in artifact");
        let result = VM::run(&artifact, fn_idx, vec![]).expect("assert_err should pass");
        assert!(result != Value::Bool(false), "assert_err should pass (got {:?})", result);
    }
}
```

---

## 変更しないファイル

- `ast.rs`: 変更なし（`FnDef.is_test` は v46.1.0 で追加済み）
- `parser.rs`: 変更なし
- `vm.rs`: assert primitive（diff メッセージ含む）はすでに実装済み
- `compiler.rs`: PRIMITIVES リストは変更なし
- `frontend/lexer.rs`: 変更なし
- `site/content/docs/`: assert_ok/assert_err ドキュメントは v46.9.0 で Developer Experience まとめとして追加予定

---

## 完了条件

- `cargo test` 全通過（failures=0、実績: 2997 + 2 = **2999** tests passed）
- `cargo clippy -- -D warnings` クリーン
- `v463000_tests` 2 件すべて pass（`assert_ok_passes` / `assert_err_passes`）
- `#[test] fn` の本体で `assert_ok`/`assert_err`/`assert_ne` を呼んでも E0001 が出ないこと
- `test "desc" {}` の本体でも `assert_ok`/`assert_err` が使えること（check_test_def 修正）
- `CHANGELOG.md` に v46.3.0 エントリ追加
- `versions/current.md` を v46.3.0（2999 tests）に更新
- `fav/Cargo.toml` version → `46.3.0`
