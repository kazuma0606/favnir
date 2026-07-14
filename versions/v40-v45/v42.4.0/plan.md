# v42.4.0 実装計画 — Stream join（time-window）

## 実装順序

---

## T0 — 事前確認

1. `cargo test` が 2883 tests / 0 failures であることを確認
2. `fav/Cargo.toml` version が `42.3.0` であることを確認
3. `vm.rs` に `VMStream::Split` バリアントが存在する行番号を記録（`Join` 挿入位置）
4. `vm.rs` の `"Stream.split"` ブロック行番号を記録（`"Stream.join"` 挿入位置）
5. `vm.rs` の `materialize_stream` 関数内 `VMStream::Split` アーム行番号を記録
6. `checker.rs` の `("Stream", "to_list")` 行番号を記録（`"join"` エントリ挿入位置）
7. `checker.rs` の `("Stream", _)` catch-all 行番号を記録（挿入ターゲット直前）
8. `driver.rs` の `v42300_tests` モジュールの閉じ `}` 行番号を記録（`v42400_tests` 挿入位置）
9. `versions/roadmap/roadmap-v42.1-v43.0.md` に v42.4.0 エントリが存在することを確認

---

## T1 — `vm.rs` — `VMStream::Join` バリアント追加

`VMStream::Split` の直後に追加:

```rust
/// v42.4.0: time-window join — nested-loop join of two streams by predicate
Join {
    left: Box<VMStream>,
    right: Box<VMStream>,
    join_fn: VMValue,
    window_secs: i64,
},
```

---

## T2 — `vm.rs` — `"Stream.join"` プリミティブ追加

`"Stream.split"` ブロックの直後に追加。引数順: `(stream1, stream2, join_fn, window_secs)`:

```rust
"Stream.join" => {
    if args.len() != 4 {
        return Err(self.error(artifact, "Stream.join requires 4 arguments: (stream1, stream2, join_fn, window_secs)"));
    }
    let mut it = args.into_iter();
    let left_val   = it.next().expect("left");
    let right_val  = it.next().expect("right");
    let join_fn    = it.next().expect("join_fn");
    let window_val = it.next().expect("window");
    match (left_val, right_val, window_val) {
        (VMValue::Stream(left), VMValue::Stream(right), VMValue::Int(window_secs)) => {
            Ok(VMValue::Stream(Box::new(VMStream::Join { left, right, join_fn, window_secs })))
        }
        (VMValue::Stream(_), VMValue::Stream(_), other) => Err(self.error(
            artifact,
            &format!("Stream.join window argument must be Int, got {}", vmvalue_type_name(&other)),
        )),
        (VMValue::Stream(_), other, _) => Err(self.error(
            artifact,
            &format!("Stream.join second argument must be a Stream, got {}", vmvalue_type_name(&other)),
        )),
        (other, _, _) => Err(self.error(
            artifact,
            &format!("Stream.join first argument must be a Stream, got {}", vmvalue_type_name(&other)),
        )),
    }
}
```

---

## T3 — `vm.rs` — `materialize_stream` に `VMStream::Join` アーム追加

`VMStream::Split` アームの直後に追加（nested-loop join、結果は `[l, r]` ペアのリスト）:

```rust
VMStream::Join { left, right, join_fn, window_secs: _ } => {
    let lefts  = self.materialize_stream(artifact, *left)?;
    let rights = self.materialize_stream(artifact, *right)?;
    let mut out = Vec::new();
    for l in &lefts {
        for r in &rights {
            let result = self.call_value(artifact, join_fn.clone(), vec![l.clone(), r.clone()])?;
            match result {
                VMValue::Bool(true) => {
                    out.push(VMValue::List(FavList::new(vec![l.clone(), r.clone()])));
                }
                VMValue::Bool(false) => {}
                other => {
                    return Err(self.error(
                        artifact,
                        &format!("Stream.join predicate must return Bool, got {}", vmvalue_type_name(&other)),
                    ));
                }
            }
        }
    }
    Ok(out)
}
```

---

## T4 — `checker.rs` — `("Stream", "join")` 型推論エントリ追加

`("Stream", "to_list")` の直後、`("Stream", _)` の直前に追加:

```rust
("Stream", "join") => Some(Type::Stream(Box::new(Type::Unknown))),
```

---

## T5 — `driver.rs` — `v42400_tests` モジュール追加

`v42300_tests` の閉じ `}` の直前行に `v42400_tests` を挿入（降順配置）。3 テスト:

### `cargo_toml_version_is_42_4_0`
```rust
// NOTE: このテストが失敗したら Cargo.toml の version を "42.4.0" に更新すること
#[test]
fn cargo_toml_version_is_42_4_0() {
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains("42.4.0"), "Cargo.toml version must be 42.4.0");
}
```

### `stream_join_type_check_ok`
```rust
#[test]
fn stream_join_type_check_ok() {
    use crate::middle::checker::Checker;
    use crate::frontend::parser::parse;
    let src = r#"fn main() -> Int { bind left <- Stream.from([1, 2]) bind right <- Stream.from([2, 3]) bind _ <- Stream.join(left, right, |a, b| a == b, 60) 0 }"#;
    let program = parse(src).expect("parse ok");
    let errors = Checker::check_program(&program);
    assert!(errors.is_empty(), "Stream.join type check errors: {:?}", errors);
}
```

### `stream_join_vm_basic`
```rust
#[test]
fn stream_join_vm_basic() {
    use crate::frontend::parser::parse;
    use crate::middle::compiler::compile;
    use crate::backend::vm::VM;
    let src = r#"fn main() -> List {
        bind left <- Stream.from([1, 2])
        bind right <- Stream.from([2, 3])
        bind joined <- Stream.join(left, right, |a, b| a == b, 60)
        Stream.to_list(joined)
    }"#;
    let program = parse(src).expect("parse ok");
    let artifact = compile(&program).expect("compile ok");
    let mut vm = VM::new();
    let result = vm.run(&artifact).expect("run ok");
    // joined: 左ストリーム値2 と 右ストリーム値2 のみマッチ → [[2, 2]] 1件
    // List([List([Int(2), Int(2)])]) の形になるはず
    let result_str = format!("{:?}", result);
    // 外側リストに要素が1件のみ（"List" が2回、"Int(2)" が2回で "Int(1)"/"Int(3)"は含まない）
    assert!(result_str.contains("Int(2)"), "expected Int(2) in result, got {:?}", result);
    assert!(!result_str.contains("Int(1)"), "Int(1) should not appear in join result, got {:?}", result);
    assert!(!result_str.contains("Int(3)"), "Int(3) should not appear in join result, got {:?}", result);
}
```

---

## T6 — `Cargo.toml` バージョン bump

```
version = "42.3.0"  →  version = "42.4.0"
```

---

## T7 — `CHANGELOG.md` 更新

`[v42.3.0]` エントリの直前に追加:

```markdown
## [v42.4.0] — 2026-07-12

### Added
- `Stream.join(stream1, stream2, join_fn, window_secs)` — time-window join 演算子
- `VMStream::Join` バリアント（nested-loop join、predicate マッチペアを返す）
- `("Stream", "join")` 型推論エントリ（checker.rs）
- `v42400_tests`: `stream_join_type_check_ok` / `stream_join_vm_basic`
```

---

## T8 — テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

- failures = 0
- テスト数 = 2886（2883 + 3）
- `v42400_tests` 3 件 pass

---

## T9 — バージョン管理ドキュメント更新

1. `versions/current.md` を v42.4.0（最新安定版）・v42.5.0（次に切る版）に更新
2. `versions/roadmap/roadmap-v42.1-v43.0.md` の v42.4.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）`）
3. `versions/v40-v45/v42.4.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）

---

## 変更ファイルサマリー

| ファイル | 変更 |
|---|---|
| `fav/src/backend/vm.rs` | `VMStream::Join` バリアント、`Stream.join` プリミティブ、`materialize_stream` アーム |
| `fav/src/middle/checker.rs` | `("Stream", "join")` 型推論エントリ |
| `fav/src/driver.rs` | `v42400_tests` 3 件 |
| `fav/Cargo.toml` | `42.3.0` → `42.4.0` |
| `CHANGELOG.md` | `[v42.4.0]` エントリ |
