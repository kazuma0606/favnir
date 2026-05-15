# Favnir v2.9.0 実装計画

作成日: 2026-05-13

---

## Phase 0 — バージョン更新

`Cargo.toml` を `version = "2.9.0"` に変更。
`src/main.rs` の HELP テキストと `print_welcome()` を `v2.9.0` に更新。

---

## Phase 1 — E067 解消 (`collect` 内 `for`)

### 1-1. `src/middle/checker.rs`

#### E067 ガードを削除

`check_stmt` の `Stmt::ForIn` ハンドラから E067 ブロックを削除する。

**変更前**:
```rust
Stmt::ForIn(f) => {
    // E067: for inside collect is not supported in v1.9.0
    if self.in_collect {
        self.type_error(
            "E067",
            "`for` inside `collect` block is not supported in v1.9.0",
            &f.span,
        );
        return;
    }
    // ... (通常チェック)
}
```

**変更後**:
```rust
Stmt::ForIn(f) => {
    // E067 guard removed in v2.9.0: for inside collect is now supported
    let iter_ty = self.check_expr(&f.iter);
    // ... (通常チェック、変更なし)
}
```

#### `collect_yield_types` ヘルパーを追加

```rust
fn collect_yield_types(&mut self, stmts: &[Stmt]) -> Vec<Type> {
    let mut tys = Vec::new();
    for stmt in stmts {
        match stmt {
            Stmt::Yield(y) => tys.push(self.check_expr(&y.expr)),
            Stmt::ForIn(f) => {
                let iter_ty = self.check_expr(&f.iter);
                let elem_ty = match iter_ty {
                    Type::List(inner) => *inner,
                    Type::Unknown | Type::Error => Type::Unknown,
                    _ => Type::Unknown,
                };
                self.env.push();
                self.env.define(f.var.clone(), elem_ty);
                tys.extend(self.collect_yield_types(&f.body.stmts));
                self.env.pop();
            }
            _ => self.check_stmt(stmt),
        }
    }
    tys
}
```

#### `Expr::Collect` ハンドラを更新

`collect_yield_types` を使って `for` ボディ内の `yield` も型推論に含める:

**変更前**:
```rust
Expr::Collect(block, _span) => {
    let old_in_collect = self.in_collect;
    self.in_collect = true;
    let mut yield_tys: Vec<Type> = Vec::new();
    for stmt in &block.stmts {
        if let Stmt::Yield(y) = stmt {
            yield_tys.push(self.check_expr(&y.expr));
        } else {
            self.check_stmt(stmt);
        }
    }
    self.check_expr(&block.expr);
    self.in_collect = old_in_collect;
    // ...
```

**変更後**:
```rust
Expr::Collect(block, _span) => {
    let old_in_collect = self.in_collect;
    self.in_collect = true;
    let yield_tys = self.collect_yield_types(&block.stmts);
    self.check_expr(&block.expr);
    self.in_collect = old_in_collect;
    // ...
```

#### E067 テストを更新

既存テスト `for_in_in_collect_e067` を削除し、代わりに「E067 が出ない」成功テストを追加する。

### 1-2. VM・コンパイラ変更なし

`for` は `List.fold` に脱糖され、`yield` は `YieldValue` opcode になる。
`YieldValue` opcode は VM-global の `collect_frames` に積まれるため、
collect ブロック内で for ループの closure が実行されても正しく動作する。

---

## Phase 2 — `Type::Stream(Box<Type>)` 追加

### `src/middle/checker.rs`

#### `Type` enum に `Stream` を追加

```rust
// `Stream<T>` lazy sequence (v2.9.0)
Stream(Box<Type>),
```

`Type` を参照している exhaustive match を全て更新:

- `is_compatible`: `(Type::Stream(a), Type::Stream(b)) => a.is_compatible(b)`
- `display`: `Type::Stream(t) => format!("Stream<{}>", t.display())`
- `apply`: `Type::Stream(t) => Type::Stream(Box::new(self.apply(t)))`
- `occurs`: `Type::Stream(t) => occurs(var, t)`
- `unify`: `(Type::Stream(a), Type::Stream(b)) => unify(a, b)`
- `substitute_self_in_type`: `Type::Stream(t) => Type::Stream(Box::new(self.substitute_self_in_type(t, self_ty)))`
- その他 exhaustive match があれば全て `Type::Stream` を追加

#### `parse_type_expr` に `"Stream"` を追加

`Task` の直後に追加:

```rust
"Task" => Type::Task(Box::new(
    resolved_args.into_iter().next().unwrap_or(Type::Unknown),
)),
"Stream" => Type::Stream(Box::new(
    resolved_args.into_iter().next().unwrap_or(Type::Unknown),
)),
```

#### グローバル名前空間リストに `"Stream"` を追加

```rust
for ns in &[
    "Math", "List", "String", "Option", "Result", "Db", "Http", "Map", "Debug", "Emit",
    "Util", "Trace", "File", "Json", "Csv", "Task", "Random", "Stream",  // 追加
] {
```

#### `resolve_field_access_type` に `"Stream"` ケースを追加

既存の `("Task", ...)` ケースの近くに追加:

```rust
("Stream", "from") => Some(Type::Unknown),   // seed T, next T->T → Stream<T> (簡易)
("Stream", "of") => Some(Type::Stream(Box::new(Type::Unknown))),
("Stream", "map") => Some(Type::Stream(Box::new(Type::Unknown))),
("Stream", "filter") => Some(Type::Stream(Box::new(Type::Unknown))),
("Stream", "take") => Some(Type::Stream(Box::new(Type::Unknown))),
("Stream", "collect") => Some(Type::List(Box::new(Type::Unknown))),
```

---

## Phase 3 — `VMValue::Stream` と VM ハンドラ追加

### `src/backend/vm.rs`

#### `VMStream` enum を追加

`VMValue` enum の前に `VMStream` を定義:

```rust
/// Lazy stream representation for Stream<T> (v2.9.0)
#[derive(Debug, Clone)]
pub enum VMStream {
    /// Infinite: generates next value from current seed
    Gen { seed: VMValue, next_fn: VMValue },
    /// Finite: converted from a list
    Of(Vec<VMValue>),
    /// Lazy map: apply map_fn to each element on collect
    Map { inner: Box<VMStream>, map_fn: VMValue },
    /// Lazy filter: apply pred_fn to each element on collect
    Filter { inner: Box<VMStream>, pred_fn: VMValue },
    /// Finite prefix of an inner stream
    Take { inner: Box<VMStream>, n: i64 },
}
```

#### `VMValue::Stream` を追加

```rust
/// `Stream<T>` lazy sequence (v2.9.0)
Stream(Box<VMStream>),
```

`VMValue` を参照している exhaustive match を全て更新（`vmvalue_type_name`/`vmvalue_repr` 等）:

```rust
VMValue::Stream(_) => "Stream",
VMValue::Stream(_) => "<stream>",
```

#### `VM::call_builtin` に `Stream.*` ハンドラを追加

`"List.map"` ハンドラの後に追加:

```rust
"Stream.from" => {
    let mut it = args.into_iter();
    let seed = it.next().ok_or_else(|| self.error(artifact, "Stream.from requires 2 arguments"))?;
    let next_fn = it.next().ok_or_else(|| self.error(artifact, "Stream.from requires 2 arguments"))?;
    Ok(VMValue::Stream(Box::new(VMStream::Gen { seed, next_fn })))
}
"Stream.of" => {
    let list_val = args.into_iter().next()
        .ok_or_else(|| self.error(artifact, "Stream.of requires 1 argument"))?;
    match list_val {
        VMValue::List(items) => Ok(VMValue::Stream(Box::new(VMStream::Of(items)))),
        _ => Err(self.error(artifact, "Stream.of requires a List")),
    }
}
"Stream.map" => {
    let mut it = args.into_iter();
    let stream_val = it.next().ok_or_else(|| self.error(artifact, "Stream.map requires 2 arguments"))?;
    let map_fn = it.next().ok_or_else(|| self.error(artifact, "Stream.map requires 2 arguments"))?;
    match stream_val {
        VMValue::Stream(inner) => Ok(VMValue::Stream(Box::new(VMStream::Map { inner: *inner, map_fn }))),
        _ => Err(self.error(artifact, "Stream.map requires a Stream as first argument")),
    }
}
"Stream.filter" => {
    let mut it = args.into_iter();
    let stream_val = it.next().ok_or_else(|| self.error(artifact, "Stream.filter requires 2 arguments"))?;
    let pred_fn = it.next().ok_or_else(|| self.error(artifact, "Stream.filter requires 2 arguments"))?;
    match stream_val {
        VMValue::Stream(inner) => Ok(VMValue::Stream(Box::new(VMStream::Filter { inner: *inner, pred_fn }))),
        _ => Err(self.error(artifact, "Stream.filter requires a Stream as first argument")),
    }
}
"Stream.take" => {
    let mut it = args.into_iter();
    let stream_val = it.next().ok_or_else(|| self.error(artifact, "Stream.take requires 2 arguments"))?;
    let n_val = it.next().ok_or_else(|| self.error(artifact, "Stream.take requires 2 arguments"))?;
    match (stream_val, n_val) {
        (VMValue::Stream(inner), VMValue::Int(n)) => {
            Ok(VMValue::Stream(Box::new(VMStream::Take { inner: *inner, n })))
        }
        _ => Err(self.error(artifact, "Stream.take requires (Stream, Int)")),
    }
}
"Stream.collect" => {
    let stream_val = args.into_iter().next()
        .ok_or_else(|| self.error(artifact, "Stream.collect requires 1 argument"))?;
    match stream_val {
        VMValue::Stream(inner) => {
            let items = self.materialize_stream(artifact, *inner)?;
            Ok(VMValue::List(items))
        }
        _ => Err(self.error(artifact, "Stream.collect requires a Stream")),
    }
}
```

#### `materialize_stream` ヘルパーを追加

`VM` の impl ブロック内に追加（`call_builtin` と同じ impl ブロック）:

```rust
fn materialize_stream(
    &mut self,
    artifact: &FvcArtifact,
    stream: VMStream,
) -> Result<Vec<VMValue>, VMError> {
    match stream {
        VMStream::Gen { .. } => Err(self.error(
            artifact,
            "Stream.collect: infinite stream cannot be collected; use Stream.take first",
        )),
        VMStream::Of(items) => Ok(items),
        VMStream::Map { inner, map_fn } => {
            let items = self.materialize_stream(artifact, *inner)?;
            items
                .into_iter()
                .map(|x| self.call_value(artifact, map_fn.clone(), vec![x]))
                .collect()
        }
        VMStream::Filter { inner, pred_fn } => {
            let items = self.materialize_stream(artifact, *inner)?;
            let mut out = Vec::new();
            for x in items {
                match self.call_value(artifact, pred_fn.clone(), vec![x.clone()])? {
                    VMValue::Bool(true) => out.push(x),
                    VMValue::Bool(false) => {}
                    other => {
                        return Err(self.error(
                            artifact,
                            &format!(
                                "Stream.filter predicate must return Bool, got {}",
                                vmvalue_type_name(&other)
                            ),
                        ));
                    }
                }
            }
            Ok(out)
        }
        VMStream::Take { inner, n } => match *inner {
            VMStream::Gen { mut seed, next_fn } => {
                let mut out = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    out.push(seed.clone());
                    seed = self.call_value(artifact, next_fn.clone(), vec![seed])?;
                }
                Ok(out)
            }
            other => {
                let items = self.materialize_stream(artifact, other)?;
                Ok(items.into_iter().take(n as usize).collect())
            }
        },
    }
}
```

---

## Phase 4 — コンパイラ グローバル登録

### `src/middle/compiler.rs`

2 つのグローバル登録ループに `"Stream"` を追加（第 1 ループと第 2 ループ）。

---

## Phase 5 — テスト追加

### `src/driver.rs`

#### `collect { for ... { yield ... } }` テスト (3件)

```rust
// collect { for x in list { yield x; } } — 全要素を yield
fn collect_for_in_yield_all() { ... }  // [0,1,2,3,4]

// collect { for x in list { if cond { yield x; } } } — フィルタパターン
fn collect_for_in_yield_filtered() { ... }  // [0,2,4]

// collect { for x in list { yield x * 2; } } — 変換パターン
fn collect_for_in_yield_transformed() { ... }  // [0,2,4,6,8]
```

#### `Stream.*` テスト (7件)

```rust
fn stream_from_take_collect() { ... }       // [0,1,2,3,4]
fn stream_of_collect() { ... }              // リスト変換
fn stream_map_collect() { ... }             // 各要素 *2
fn stream_filter_collect() { ... }          // 偶数のみ
fn stream_take_limits_length() { ... }      // take(3) → 3件
fn stream_of_map_filter_pipeline() { ... }  // of + map + filter 組み合わせ
fn stream_collect_infinite_errors() { ... } // take なし → ランタイムエラー
```

### `src/middle/checker.rs`

#### `Stream<T>` 型チェックテスト (2件)

```rust
fn stream_type_parses_correctly() { ... }   // Stream<Int> が型チェックを通る
fn stream_collect_returns_list() { ... }    // Stream.collect の型が List<T>
```

---

## Phase 6 — examples/stream_demo 作成

### `fav/examples/stream_demo/fav.toml`

```toml
[rune]
name    = "stream_demo"
version = "0.1.0"
src     = "src"
```

### `fav/examples/stream_demo/src/main.fav`

`Stream.from`, `Stream.take`, `Stream.collect`, `Stream.filter`, `Stream.map` と
`collect { for ... { yield ...; } }` の動作例を示すデモ。

---

## Phase 7 — ドキュメント・最終確認

### 最終テスト確認

- `cargo build` で警告ゼロを確認
- `cargo test` で全テスト通過を確認（v2.8.0: 625 → 目標 637）

### ドキュメント作成

- `versions/v2.9.0/langspec.md` を作成
- `versions/v2.9.0/progress.md` を全 [x] に更新

---

## 注意点

### `for` 内 `yield` の型推論

`collect_yield_types` ヘルパーは `for` ボディ内の `yield` 型を返すが、
`check_stmt` も同時に呼ぶことで全ての型チェックが通る設計。
重複チェックにならないよう、`Yield` ステートメントは `collect_yield_types` のみで処理し、
`check_stmt` のルートでは `Stmt::Yield` を `Expr::Collect` 内からは呼ばない。

実際には `Expr::Collect` ハンドラを `collect_yield_types` に一本化することで
`check_stmt` の `yield` チェックとの二重処理を避ける。

### `VMStream` の `Clone` トレイト

`VMStream` は `VMValue` を含むため、`VMValue` が `Clone` を実装していれば `VMStream` も `Clone` 可。
`Call_value` が所有権を取るため、`clone()` が必要な場面が多い。

### 無限ストリームのエラー処理

`Stream.collect` を `Stream.take` なしで呼ぶと実行時エラー。
ユーザーへのメッセージ: `"Stream.collect: infinite stream cannot be collected; use Stream.take first"`

### `Stream.from` の型チェック

`Stream.from(seed: T, f: T -> T) -> Stream<T>` の型を完全に推論するには
`T` の型変数が必要。v2.9.0 では簡易実装として `Stream.from` の戻り型を `Type::Unknown` とし、
`bind s <- Stream.from(0, |n| n + 1)` のような使い方で型エラーを出さない。

### テスト数の見込み

| カテゴリ | 追加 |
|---------|------|
| checker.rs Stream 型テスト | +2 |
| driver.rs collect+for テスト | +3 |
| driver.rs Stream テスト | +7 |
| 既存 E067 テスト削除/更新 | -1〜0 |
| **目標** | **+11〜12 → 637** |
