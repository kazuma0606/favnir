# Plan — v55.4.0 — ストリーム結合（inner join / left outer join）

## ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `55.4.0` に更新。

```toml
[package]
version = "55.4.0"
```

---

### Step 2: `vm.rs` — `VMStream::JoinLeft` バリアント追加

`VMStream` enum の `Join` バリアント（L1615〜L1621 付近）直後に追加する。

**変更箇所**: `fav/src/backend/vm.rs`

**変更前（挿入位置の直前）:**
```rust
    /// v42.4.0: time-window join — nested-loop join of two streams by predicate
    Join {
        left: Box<VMStream>,
        right: Box<VMStream>,
        join_fn: VMValue,
        window_secs: i64,
    },
}
```

**変更後:**
```rust
    /// v42.4.0: time-window join — nested-loop join of two streams by predicate
    Join {
        left: Box<VMStream>,
        right: Box<VMStream>,
        join_fn: VMValue,
        window_secs: i64,
    },
    /// v55.4.0: left outer join — all left items preserved; unmatched right side = Unit
    JoinLeft {
        left: Box<VMStream>,
        right: Box<VMStream>,
        join_fn: VMValue,
        window_secs: i64,
    },
}
```

---

### Step 3: `vm.rs` — `Stream.join_inner` / `Stream.join_left` primitive 追加

`Stream.join` アームの直後（`// ── end v26.4.0 Stream.* ──` コメントの直前）に 2 アームを追加する。

**挿入位置**: `"Http.serve_raw"` アーム（L5236 付近）の直前。

```rust
"Stream.join_inner" => {
    if args.len() != 4 {
        return Err(self.error(artifact, "Stream.join_inner requires 4 arguments: (stream1, stream2, join_fn, window_secs)"));
    }
    let mut it = args.into_iter();
    let left_val   = it.next().expect("left");
    let right_val  = it.next().expect("right");
    let join_fn    = it.next().expect("join_fn");
    let window_val = it.next().expect("window");
    match (left_val, right_val, window_val) {
        (VMValue::Stream(left), VMValue::Stream(right), VMValue::Int(window_secs)) => {
            if window_secs <= 0 {
                return Err(self.error(artifact, "Stream.join_inner window_secs must be positive (>= 1)"));
            }
            Ok(VMValue::Stream(Box::new(VMStream::Join { left, right, join_fn, window_secs })))
        }
        (VMValue::Stream(_), VMValue::Stream(_), other) => Err(self.error(
            artifact,
            &format!("Stream.join_inner window argument must be Int, got {}", vmvalue_type_name(&other)),
        )),
        (VMValue::Stream(_), other, _) => Err(self.error(
            artifact,
            &format!("Stream.join_inner second argument must be a Stream, got {}", vmvalue_type_name(&other)),
        )),
        (other, _, _) => Err(self.error(
            artifact,
            &format!("Stream.join_inner first argument must be a Stream, got {}", vmvalue_type_name(&other)),
        )),
    }
}
"Stream.join_left" => {
    if args.len() != 4 {
        return Err(self.error(artifact, "Stream.join_left requires 4 arguments: (stream1, stream2, join_fn, window_secs)"));
    }
    let mut it = args.into_iter();
    let left_val   = it.next().expect("left");
    let right_val  = it.next().expect("right");
    let join_fn    = it.next().expect("join_fn");
    let window_val = it.next().expect("window");
    match (left_val, right_val, window_val) {
        (VMValue::Stream(left), VMValue::Stream(right), VMValue::Int(window_secs)) => {
            if window_secs <= 0 {
                return Err(self.error(artifact, "Stream.join_left window_secs must be positive (>= 1)"));
            }
            Ok(VMValue::Stream(Box::new(VMStream::JoinLeft { left, right, join_fn, window_secs })))
        }
        (VMValue::Stream(_), VMValue::Stream(_), other) => Err(self.error(
            artifact,
            &format!("Stream.join_left window argument must be Int, got {}", vmvalue_type_name(&other)),
        )),
        (VMValue::Stream(_), other, _) => Err(self.error(
            artifact,
            &format!("Stream.join_left second argument must be a Stream, got {}", vmvalue_type_name(&other)),
        )),
        (other, _, _) => Err(self.error(
            artifact,
            &format!("Stream.join_left first argument must be a Stream, got {}", vmvalue_type_name(&other)),
        )),
    }
}
```

---

### Step 4: `vm.rs` — `VMStream::JoinLeft` materialization 追加

`materialize_stream` の `VMStream::Join` アーム終端（L6109 `Ok(out)` の直後）の後、
`match` の閉じ括弧 `}` の直前に追加する。

```rust
// v55.4.0: left outer join — unmatched left rows emitted as [left, Unit]
VMStream::JoinLeft { left, right, join_fn, window_secs: _ } => {
    let lefts  = self.materialize_stream(artifact, *left)?;
    let rights = self.materialize_stream(artifact, *right)?;
    let mut out = Vec::new();
    for l in &lefts {
        let mut matched = false;
        for r in &rights {
            let result = self.call_value(artifact, join_fn.clone(), vec![l.clone(), r.clone()])?;
            match result {
                VMValue::Bool(true) => {
                    out.push(VMValue::List(FavList::new(vec![l.clone(), r.clone()])));
                    matched = true;
                }
                VMValue::Bool(false) => {}
                other => {
                    return Err(self.error(
                        artifact,
                        &format!("Stream.join_left predicate must return Bool, got {}", vmvalue_type_name(&other)),
                    ));
                }
            }
        }
        if !matched {
            // 右側にマッチなし: Unit プレースホルダーで左側要素を保持
            out.push(VMValue::List(FavList::new(vec![l.clone(), VMValue::Unit])));
        }
    }
    Ok(out)
}
```

---

### Step 5: `driver.rs` — `v55400_tests` モジュール追加

`v55300_tests` モジュールの直前（`// -- v55300_tests` コメント行の前）に挿入する。

```rust
// -- v55400_tests (v55.4.0) -- ストリーム結合（inner join / left outer join）--
#[cfg(test)]
mod v55400_tests {
    use super::{build_artifact, exec_artifact_main};
    use crate::frontend::parser::Parser;

    #[test]
    fn stream_join_inner_matches() {
        // left=[1,2], right=[2,3], |a,b| a==b → (2,2) のみマッチ → [[2,2]] 1件
        let src = r#"public fn main() -> List {
            bind left <- Stream.from(List.range(1, 3))
            bind right <- Stream.from(List.range(2, 4))
            bind joined <- Stream.join_inner(left, right, |a, b| a == b, 60)
            Stream.to_list(joined)
        }"#;
        let program = Parser::parse_str(src, "join_inner_test.fav").expect("parse ok");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec ok");
        assert_eq!(
            value,
            crate::value::Value::List(vec![
                crate::value::Value::List(vec![
                    crate::value::Value::Int(2),
                    crate::value::Value::Int(2),
                ]),
            ]),
            "inner join should return only matched pairs, got {:?}", value
        );
    }

    #[test]
    fn stream_join_left_preserves_unmatched() {
        // left=[1,2], right=[2,3], |a,b| a==b
        // left=1: 右側マッチなし → [1, Unit]
        // left=2: right=2 とマッチ → [2, 2]
        // 結果: 2件（unmatched も保持）
        let src = r#"public fn main() -> List {
            bind left <- Stream.from(List.range(1, 3))
            bind right <- Stream.from(List.range(2, 4))
            bind joined <- Stream.join_left(left, right, |a, b| a == b, 60)
            Stream.to_list(joined)
        }"#;
        let program = Parser::parse_str(src, "join_left_test.fav").expect("parse ok");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec ok");
        assert_eq!(
            value,
            crate::value::Value::List(vec![
                crate::value::Value::List(vec![
                    crate::value::Value::Int(1),
                    crate::value::Value::Unit,
                ]),
                crate::value::Value::List(vec![
                    crate::value::Value::Int(2),
                    crate::value::Value::Int(2),
                ]),
            ]),
            "left join should preserve unmatched left items as [val, Unit], got {:?}", value
        );
    }
}
```

---

### Step 6: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo build 2>&1 | tail -5
```

期待結果: `Finished` — エラーなし

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待結果: `3213 tests passed, 0 failed`

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -10
```

期待結果: クリーン

---

### Step 7: ポスト処理

- `CHANGELOG.md` に v55.4.0 エントリ追加
- `versions/current.md` を v55.4.0 / 3213 tests に更新
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.4.0 実績を COMPLETE に更新（3213 tests 訂正含む）
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.4.0 実績欄も COMPLETE に更新

---

## 注意事項

- `VMStream::JoinLeft` を `VMStream` enum に追加すると、`materialize_stream` の match が
  non-exhaustive になりコンパイルエラーが発生する。Step 4 を **Step 2 と同時に** 実施すること
  （または Step 4 → Step 2 の順でも可。Step 2 のみ先行すると `cargo build` 失敗）。
  実装順は **Step 2 と Step 4 を同一 Edit** で適用し、`cargo build` は Step 3 以降に実行する。
- `Stream.join_inner` は `VMStream::Join`（既存バリアント）を生成するため、
  `materialize_stream` に新しいアームは不要（Step 3 は primitive 登録のみ）。
- `v55300_tests` には `cargo_toml_version_is_55_3_0` が存在しないため削除タスクなし。
- `List.range(1, 3)` は `[1, 2]`（上限排他）、`List.range(2, 4)` は `[2, 3]`。
  テストの期待値はこれを前提とする。
- `VMValue::Unit` から `Value::Unit` への変換は `impl From<VMValue> for Value`（`vm.rs` の `From` 実装）で
  `VMValue::Unit => Value::Unit` として対応済みであるため、追加変更は不要。
  `cargo build` 後のテスト実行で変換が正しく動作することを確認する。
