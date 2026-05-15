# Favnir v2.9.0 仕様書

作成日: 2026-05-13

---

## テーマ

**`Stream<T>` + `collect` 内 `for`**

遅延シーケンス型 `Stream<T>` を追加し、`collect { for x in list { yield x; } }` パターン（E067）を解消する。

---

## 機能 1: `collect` 内 `for` の許可（E067 解消）

### 現状

v1.9.0 で `for x in list { body }` を追加した際、`collect` ブロック内での `for` は
「未サポート（v1.9.0）」として E067 エラーを発生させていた。

```favnir
// E067: `for` inside `collect` block is not supported in v1.9.0
bind evens <- collect {
    for x in List.range(0, 10) {
        if x % 2 == 0 { yield x; }
    }
}
```

### 設計

- E067 を廃止し、`collect` 内 `for` を正式サポート
- `for` ボディ内の `yield` は外側の `collect` に帰属する（VM は既に対応済み）
- `for` 内の `yield` の型を `collect` の要素型推論に含める

### 実装方針

`checker.rs` の `Stmt::ForIn` ハンドラにある E067 ガードを削除する：

```rust
// 削除する:
// if self.in_collect {
//     self.type_error("E067", "`for` inside `collect` block is not supported in v1.9.0", ...);
//     return;
// }
```

`Expr::Collect` のチェック時に `for` ボディ内の `yield` も型収集する：

```rust
// collect_yield_types ヘルパーを追加し、ForIn ボディを再帰スキャン
fn collect_yield_types(&mut self, stmts: &[Stmt]) -> Vec<Type> {
    let mut tys = Vec::new();
    for stmt in stmts {
        match stmt {
            Stmt::Yield(y) => tys.push(self.check_expr(&y.expr)),
            Stmt::ForIn(f) => {
                let iter_ty = self.check_expr(&f.iter);
                let elem_ty = match iter_ty {
                    Type::List(inner) => *inner,
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

VM・コンパイラへの変更は不要（`YieldValue` opcode は VM-global の `collect_frames` に積まれるため）。

### E067 テスト更新

`checker.rs` の `for_in_in_collect_e067` テストを削除または「E067 が出ない」テストに変更する。
`driver.rs` に `collect_for_in_yield` 統合テストを追加する。

---

## 機能 2: `Stream<T>` 遅延シーケンス

### 設計

`Stream<T>` は遅延評価のシーケンス。`Stream.collect` が呼ばれたときに初めて具体値に展開される。

```favnir
// 無限数列 0, 1, 2, 3, ...
bind nats <- Stream.from(0, |n| n + 1)

// 最初の 5 件
bind first5 <- Stream.collect(Stream.take(nats, 5))
// first5 = [0, 1, 2, 3, 4]

// List から Stream へ変換してフィルタ
bind evens <- Stream.collect(Stream.filter(Stream.of(List.range(0, 10)), |x| x % 2 == 0))
// evens = [0, 2, 4, 6, 8]
```

### 型システム

**`Type::Stream(Box<Type>)`** を `checker.rs` の `Type` enum に追加。

| 構文 | 型 |
|------|-----|
| `Stream.from(seed, f)` | `Stream<T>` |
| `Stream.of(list)` | `Stream<T>` |
| `Stream.map(stream, f)` | `Stream<U>` |
| `Stream.filter(stream, pred)` | `Stream<T>` |
| `Stream.take(stream, n)` | `Stream<T>` |
| `Stream.collect(stream)` | `List<T>` |

### VM 実装

**`VMValue::Stream(Box<VMStream>)`** を `VMValue` enum に追加。

```rust
#[derive(Debug, Clone)]
pub enum VMStream {
    Gen { seed: VMValue, next_fn: VMValue },      // 無限: seed + 次状態生成関数
    Of(Vec<VMValue>),                              // 有限: List から生成
    Map { inner: Box<VMStream>, map_fn: VMValue }, // 変換
    Filter { inner: Box<VMStream>, pred_fn: VMValue }, // フィルタ
    Take { inner: Box<VMStream>, n: i64 },         // 件数制限
}
```

`VM::call_builtin` メソッドに `Stream.*` ハンドラを追加（closure 呼び出しが必要なため `vm_call_builtin` ではなく `VM::call_builtin` に実装）。

**`Stream.from(seed, f)`**:
```rust
Ok(VMValue::Stream(Box::new(VMStream::Gen { seed, next_fn: f })))
```

**`Stream.of(list)`**:
```rust
// list が VMValue::List(items) の場合
Ok(VMValue::Stream(Box::new(VMStream::Of(items))))
```

**`Stream.map(stream, f)`**:
```rust
// stream が VMValue::Stream(inner) の場合
Ok(VMValue::Stream(Box::new(VMStream::Map { inner: *inner, map_fn: f })))
```

**`Stream.filter(stream, pred)`**:
```rust
Ok(VMValue::Stream(Box::new(VMStream::Filter { inner: *inner, pred_fn: pred })))
```

**`Stream.take(stream, n)`**:
```rust
Ok(VMValue::Stream(Box::new(VMStream::Take { inner: *inner, n })))
```

**`Stream.collect(stream)`**:
```rust
// materialize(stream) で Vec<VMValue> を生成して VMValue::List に変換
let items = self.materialize_stream(artifact, *inner)?;
Ok(VMValue::List(items))
```

**`materialize_stream` ヘルパー**:
```rust
fn materialize_stream(&mut self, artifact: &FvcArtifact, stream: VMStream) -> Result<Vec<VMValue>, VMError> {
    match stream {
        VMStream::Gen { .. } => Err(vm.error("Stream.collect: cannot collect infinite stream; use Stream.take first")),
        VMStream::Of(items) => Ok(items),
        VMStream::Map { inner, map_fn } => {
            let items = self.materialize_stream(artifact, *inner)?;
            items.into_iter().map(|x| self.call_value(artifact, map_fn.clone(), vec![x])).collect()
        }
        VMStream::Filter { inner, pred_fn } => {
            let items = self.materialize_stream(artifact, *inner)?;
            let mut out = Vec::new();
            for x in items {
                if let VMValue::Bool(true) = self.call_value(artifact, pred_fn.clone(), vec![x.clone()])? {
                    out.push(x);
                }
            }
            Ok(out)
        }
        VMStream::Take { inner, n } => {
            match *inner {
                VMStream::Gen { mut seed, next_fn } => {
                    // Generate n items from infinite stream
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
            }
        }
    }
}
```

### 型チェック実装

**`checker.rs`** への追加:

1. `Type` enum に `Stream(Box<Type>)` を追加
2. `is_compatible`/`display`/`apply`/`occurs`/`unify` に `Stream` ケースを追加
3. `parse_type_expr` に `"Stream" => Type::Stream(...)` を追加（`Task` と同様）
4. グローバル名前空間リストに `"Stream"` を追加
5. `resolve_field_access_type` に `("Stream", method)` のケースを追加:
   - `("Stream", "from")` → `Type::Unknown`（引数型が複雑なため簡易実装）
   - `("Stream", "of")` → `Type::Stream(Type::Unknown)`
   - `("Stream", "map")` → `Type::Stream(Type::Unknown)`
   - `("Stream", "filter")` → `Type::Stream(Type::Unknown)`
   - `("Stream", "take")` → `Type::Stream(Type::Unknown)`
   - `("Stream", "collect")` → `Type::List(Type::Unknown)`

**`compiler.rs`** への追加:
- 2 つのグローバル登録ループに `"Stream"` を追加

---

## Phase 6 — examples/stream_demo の作成

### `fav/examples/stream_demo/fav.toml`

```toml
[rune]
name    = "stream_demo"
version = "0.1.0"
src     = "src"
```

### `fav/examples/stream_demo/src/main.fav`

```favnir
public fn main() -> Unit !Io = {
    // 無限数列 → take → collect
    bind nats_stream <- Stream.from(0, |n| n + 1)
    bind first5 <- Stream.collect(Stream.take(nats_stream, 5))
    IO.println($"first5 = {Debug.show(first5)}");

    // List からフィルタ
    bind even_stream <- Stream.filter(Stream.of(List.range(0, 10)), |x| x % 2 == 0)
    bind evens <- Stream.collect(even_stream)
    IO.println($"evens = {Debug.show(evens)}");

    // collect { for ... { yield ... } }
    bind mapped <- collect {
        for x in List.range(0, 5) {
            yield x * 2;
        }
    }
    IO.println($"mapped = {Debug.show(mapped)}")
}
```

---

## 完了条件

- `Stream.from(0, |n| n + 1)` → `Stream.take(s, 5)` → `Stream.collect(s)` が `[0,1,2,3,4]` を返す
- `Stream.of(list)` → `Stream.collect` が元の `List` を返す
- `Stream.map(stream, f)` が各要素に f を適用したリストを返す
- `Stream.filter(stream, pred)` が条件を満たす要素のみのリストを返す
- 無限ストリームを `Stream.take` なしで `Stream.collect` するとランタイムエラー
- `collect { for x in list { yield x; } }` が E067 なしで動く
- `collect { for x in list { if cond { yield x; } } }` が動く（フィルタパターン）
- `cargo build` 警告ゼロ
- `cargo test` 全テスト通過（目標: 625 → 638 程度）
- `Cargo.toml` バージョンが `"2.9.0"`
- `versions/v2.9.0/langspec.md` 作成済み

---

## テスト数見込み

v2.8.0 ベースライン: 625

| カテゴリ | 追加件数 |
|---------|--------|
| checker.rs `Stream<T>` 型チェックテスト | +2 |
| checker.rs E067 解消テスト（既存テスト更新） | ±0 |
| driver.rs `collect { for ... { yield ... } }` 統合テスト | +3 |
| driver.rs `Stream.*` 統合テスト | +7 |
| **目標合計** | **637（+12）** |
