# v42.4.0 仕様書 — Stream join（time-window）

## 概要

2 つのストリームを time-window で結合する `Stream.join` 演算子を追加する。
VM プリミティブ・`VMStream::Join` バリアント・checker.rs 型推論エントリの 3 点を実装する。

---

## 背景・動機

v42.1〜v42.3 で CEP 基盤を整備した。
CEP の実世界ユースケースでは「注文ストリームと支払いストリームを 60 秒窓で突き合わせる」のような
2 ストリーム結合が必要。v42.4.0 でその基礎実装を行う。

---

## Favnir 構文（実装する呼び出し形式）

ロードマップの例（`on:` / `window:` 名前付き引数）はまだパーサーがサポートしない構文。
本バージョンでは既存の Stream API と一貫した**位置引数形式**で実装する:

```favnir
bind joined <- Stream.join(orders, payments, |o, p| o.id == p.order_id, 60)
```

引数の順序: `Stream.join(stream1, stream2, join_fn, window_secs)`

- `stream1`: 左ストリーム (`Stream<A>`)
- `stream2`: 右ストリーム (`Stream<B>`)
- `join_fn`: 結合条件クロージャ `|a, b| Bool`
- `window_secs`: 時間窓（秒）を表す `Int`。VM は現状時刻を持たないため、nested-loop join の上限として使用（accept all within window）

---

## 実装スコープ

### 1. `vm.rs` — `VMStream::Join` バリアント追加

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

### 2. `vm.rs` — `Stream.join` プリミティブ追加

`"Stream.split"` ブロックの直後に追加。4 引数（stream1, stream2, join_fn, window_secs）:

```rust
"Stream.join" => {
    if args.len() != 4 {
        return Err(self.error(artifact, "Stream.join requires 4 arguments: (stream1, stream2, join_fn, window_secs)"));
    }
    let mut it = args.into_iter();
    let left_val  = it.next().expect("left");
    let right_val = it.next().expect("right");
    let join_fn   = it.next().expect("join_fn");
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

### 3. `vm.rs` — `materialize_stream` に `VMStream::Join` アーム追加

`VMStream::Split` アームの直後に追加（nested-loop join、結果は `[left_item, right_item]` のリスト）。
`window_secs` は §2 のプリミティブで `Int` 型として受け取り・格納するが、materialize では無視し全ペアを比較する（§非スコープ参照）:

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

### 4. `checker.rs` — `Stream.join` 型推論エントリ追加

`("Stream", "to_list")` の直後、`("Stream", _)` の直前に追加:

```rust
("Stream", "join") => Some(Type::Stream(Box::new(Type::Unknown))),
```

`Stream.join(...)` が `Stream<Unknown>` として型推論され、`Type::Unknown` ではなく Stream 型を持つことが保証される。
結合結果の要素型は実際には `[A, B]` のペアだが、現バージョンでは `Stream<Unknown>` で近似する。
正確なペア型推論（`Stream<Pair<A,B>>` 等）は v43.x 以降に延期。`Stream.to_list(joined)` の結果型も同様に `Unknown` となり後続の型チェックは緩い。

### 5. `driver.rs` — `v42400_tests` 追加（3 テスト）

```rust
// -- v42400_tests (v42.4.0) -- Stream join time-window --
mod v42400_tests {
    fn cargo_toml_version_is_42_4_0()
    fn stream_join_type_check_ok()   // parse + checker で no errors
    fn stream_join_vm_basic()        // VM で join 結果の pairs を確認
}
```

`stream_join_type_check_ok`:
```rust
let src = r#"fn main() -> Int { bind left <- Stream.from([1, 2]) bind right <- Stream.from([2, 3]) bind _ <- Stream.join(left, right, |a, b| a == b, 60) 0 }"#;
```
→ `Checker::check_program` でエラーなし。

`stream_join_vm_basic`:
- `Stream.from([1, 2])` + `Stream.from([2, 3])` を join（`|a, b| a == b`、window=60）
- `Stream.to_list(joined)` → `[[2, 2]]`（左ストリーム値 2 と右ストリーム値 2 がマッチ、1 件のみ）
- アサーションは `[[2, 2]]` の 1 件のみ存在することを検証（マッチ件数 = 1 を確認）

---

## テスト計画

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_42_4_0` | Cargo.toml に "42.4.0" が含まれる |
| `stream_join_type_check_ok` | `Stream.join(...)` を含むプログラムが型チェックを通過 |
| `stream_join_vm_basic` | VM で `Stream.join` の結合結果を確認（pairs のリスト） |

**推定テスト数**: 2883 + 3 = **2886**

---

## 影響範囲

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/backend/vm.rs` | 変更 | `VMStream::Join` バリアント、`Stream.join` プリミティブ、`materialize_stream` アーム |
| `fav/src/middle/checker.rs` | 変更 | `("Stream", "join")` 型推論エントリ追加 |
| `fav/src/driver.rs` | 変更 | `v42400_tests` 3 件追加 |
| `fav/Cargo.toml` | 変更 | version `42.3.0` → `42.4.0` |
| `CHANGELOG.md` | 変更 | `[v42.4.0]` エントリ追加 |
| `versions/current.md` | 変更 | 最新安定版 v42.4.0・次版 v42.5.0 に更新 |

---

## 非スコープ

- 名前付き引数 `on: ..., window: ...` 構文（パーサー未対応）
- 実際の時刻ベース窓（VM はシミュレーション実行のみ）
- `Stream.join` の `window_secs` による上限絞り込み（VM では無視、全ペアを比較）
- checker.fav（Favnir 自己ホスト側）への移植（v43.x 以降）
- join キー型安全チェック（ロードマップ記載の「join キーの型安全チェックを checker.fav に追加」はこれに相当）— 現在 checker.rs エントリは `Stream<Unknown>` 返却のみで型安全性は最小限。v43.x 以降で正確なペア型推論とともに実装予定
- 将来 `on:`/`window:` 名前付き引数構文への移行時、位置引数形式との後方互換性を破壊する可能性がある（破壊的変更の予告）
