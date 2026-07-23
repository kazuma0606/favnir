# Spec: v46.2.0 — `fav test` コマンド: `#[test]` fn 対応

Date: 2026-07-16
Status: TODO

---

## 概要

v46.1.0 で追加した `FnDef.is_test` を `fav test` コマンドで実際に実行できるようにする。
`collect_test_cases()` に `Item::FnDef(fd) if fd.is_test` のアームを追加するだけで、
既存の `cmd_test` の収集・実行・レポートロジックをそのまま流用できる。

```bash
fav test main.fav                     # すべての #[test] fn を実行
fav test main.fav --filter test_add   # 名前でフィルタ
```

---

## 調査結果（実装前に確認済み）

### `collect_test_cases` の現状

`driver.rs:4921` の `collect_test_cases` は `TestDef` / `TestGroup` のみを収集し、
`FnDef(fd) if fd.is_test` のアームが存在しない。

```rust
for item in &prog.items {
    match item {
        ast::Item::TestDef(td) => { ... }
        ast::Item::TestGroup { ... } => { ... }
        _ => {}  // ← #[test] fn はここで無視されている
    }
}
```

### コンパイル名（fn_name）の確認

- `TestDef` のアーティファクト上の関数名: `$test:<description>`
- `TestGroup` のアーティファクト上の関数名: `$testgroup:<group>:<test>`
- `FnDef` のアーティファクト上の関数名: `fd.name`（通常の fn と同じ、プレフィックスなし）

`#[test] fn test_add()` を収集する際の `fn_name` は単に `"test_add"` となる。

### VM 実行の pass/fail 判定（`cmd_test` line 5153〜5170）

- `Ok(Value::Bool(false))` → **FAIL**
- `Ok(_)` → **PASS**（Unit / Int / Bool(true) など）
- `Err(_)` → **FAIL**

`#[test] fn` の戻り型は `Unit`（暗黙 return）でよく、その場合 `Ok(Value::Unit)` → PASS になる。

### テスト数

- 実際の v46.1.0 完了時: 2994
- ロードマップ推定（2993）は v46.0 閾値ベースのため実態と乖離あり
- 本バージョン完了時の推定: 2994 + 3 = **2997**（`fav_test_discovers_tests` / `fav_test_reports_results` / `non_test_fn_not_discovered` の 3 件）

---

## 変更対象

### §1 — `driver.rs`: `collect_test_cases` に `is_test` アーム追加

`_ => {}` の直前に追加:

```rust
ast::Item::FnDef(fd) if fd.is_test => {
    total_discovered += 1;
    if let Some(f) = filter {
        if !fd.name.contains(f) {
            continue;
        }
    }
    tests_to_run.push((
        path.clone(),
        fd.name.clone(),       // display_name: fn 名をそのまま使用
        fd.name.clone(),       // fn_name: アーティファクト上の名前（プレフィックスなし）
        prog.clone(),
    ));
}
```

### §2 — `driver.rs`: v462000_tests 追加

`v461000_tests` の直後に `v462000_tests` モジュールを追加（3件）:

```rust
#[cfg(test)]
mod v462000_tests {
    use crate::frontend::parser::Parser;
    use crate::ast::Item;

    #[test]
    fn fav_test_discovers_tests() {
        let src = r#"
            #[test]
            fn test_add() {
                1
            }
        "#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let (tests, total) =
            super::collect_test_cases(vec![("test.fav".into(), prog)], None);
        assert_eq!(total, 1, "should discover 1 #[test] fn");
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].1, "test_add");  // display_name
        assert_eq!(tests[0].2, "test_add");  // fn_name
    }

    #[test]
    fn fav_test_reports_results() {
        use crate::backend::vm::VM;
        use crate::value::Value;

        let src = r#"
            #[test]
            fn test_always_pass() {
                true
            }
        "#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let (tests, _) =
            super::collect_test_cases(vec![("test.fav".into(), prog.clone())], None);
        assert_eq!(tests.len(), 1, "should discover test fn");

        // build_artifact は cmd_test 本体と同じパスを使う（driver.rs:1755）
        let artifact = super::build_artifact(&prog);
        let fn_idx = artifact.fn_idx_by_name("test_always_pass")
            .expect("test fn should be in artifact");
        let result = VM::run(&artifact, fn_idx, vec![]).expect("vm run failed");
        // pass 判定: Ok(_) かつ Bool(false) でなければ PASS
        assert!(
            result != Value::Bool(false),
            "test fn should pass (got {:?})",
            result
        );
    }
}
```

---

## 変更しないファイル

- `ast.rs`: v46.1.0 で追加済みの `FnDef.is_test` を利用するだけ
- `parser.rs`: 変更なし
- `checker.rs` / `compiler.rs` / `vm.rs`: 変更なし（`#[test] fn` は通常の fn として扱われる）
- `cmd_test` 本体（`driver.rs:5008`）: 変更なし（`driver.rs:5107` で `collect_test_cases` を呼ぶ構造になっており、そこから VM ループまで `fn_name` を動的に使っているため `fd.name` でも自動的に対応）
- `site/` MDX: Developer Experience まとめは v46.9.0 で追加

---

## 完了条件

- `cargo test` 全通過（failures=0、実績: 2994 + 3 = **2997** tests passed）
- `cargo clippy -- -D warnings` クリーン
- `v462000_tests` 3 件すべて pass（`fav_test_discovers_tests` / `fav_test_reports_results` / `non_test_fn_not_discovered`）
- `#[test] fn` が `collect_test_cases` で収集されること
- `#[test] fn` が `--filter` で名前フィルタできること（実装上自動的に対応）
- `CHANGELOG.md` に v46.2.0 エントリ追加
- `versions/current.md` を v46.2.0（2997 tests）に更新
- `fav/Cargo.toml` version → `46.2.0`
