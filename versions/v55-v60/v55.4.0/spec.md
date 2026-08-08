# Spec — v55.4.0 — ストリーム結合（inner join / left outer join）

## 概要

v55.4.0 は Streaming Native 2.0 スプリント（v55.1〜v55.9）の第 4 弾。
`Stream.join_inner` / `Stream.join_left` を VM primitive として追加し、
時間ウィンドウ内（`within_sec` / `window_secs` 引数）でキーマッチングを行う
ストリーム結合を実現する。

既存の `Stream.join`（v42.4.0 実装、inner join、`VMStream::Join` バリアント）を参照実装として活用し、
以下を追加する：
1. `Stream.join_inner` — 明示的 inner join 名称で `VMStream::Join` を生成する新 primitive
2. `Stream.join_left` — left outer join。マッチしない左側要素を `Unit` プレースホルダーとともに保持
3. `VMStream::JoinLeft` — `materialize_stream` で left outer join を実行する新バリアント
4. `v55400_tests` — `stream_join_inner_matches` / `stream_join_left_preserves_unmatched`

---

## ロードマップ参照

- `versions/roadmap/roadmap-v55.1-v56.0.md` — v55.4.0 セクション
- ベーステスト数: **3211**（v55.3.0 完了時点の実績値）
- 目標テスト数: **3213**（+2、削除なし）

> **注記**: ロードマップ上のベース値が 3212（3211 + 1 のずれ）と記載されていたが、
> v55.3.0 の実績が 3211 のため本バージョンの目標は **3213**（3211 + 2）とする。
> ロードマップの 3214 記載は v55.4.0 実装前に訂正する。

---

## 既存実装との関係

| 要素 | バージョン | 状態 |
|------|-----------|------|
| `VMStream::Join` バリアント | v42.4.0 | 実装済み（inner join、nested-loop） |
| `Stream.join` primitive | v42.4.0 | 実装済み（`VMStream::Join` を生成） |
| `Stream.join_inner` primitive | — | **未実装（v55.4.0 で追加）** |
| `Stream.join_left` primitive | — | **未実装（v55.4.0 で追加）** |
| `VMStream::JoinLeft` バリアント | — | **未実装（v55.4.0 で追加）** |

`Stream.join_inner` は `Stream.join` と同一の `VMStream::Join` バリアントを生成する（内部実装を共有）。
`Stream.join_left` は新規の `VMStream::JoinLeft` バリアントを生成する。

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "55.4.0"
```

---

### 2. `fav/src/backend/vm.rs` — `VMStream::JoinLeft` バリアント追加

`VMStream::Join` バリアント（L1615〜L1621）の直後に追加する。

```rust
/// v55.4.0: left outer join — all left items preserved; unmatched right side = Unit
JoinLeft {
    left: Box<VMStream>,
    right: Box<VMStream>,
    join_fn: VMValue,
    window_secs: i64,
},
```

---

### 3. `fav/src/backend/vm.rs` — `Stream.join_inner` / `Stream.join_left` primitive 追加

`Stream.join` アーム（L5205〜L5235 の `// ── end v26.4.0 Stream.* ──` コメントの直前）の
直後（`"Http.serve_raw"` アームの直前）に追加する。

#### `Stream.join_inner`

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
```

#### `Stream.join_left`

```rust
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

### 4. `fav/src/backend/vm.rs` — `VMStream::JoinLeft` materialization 追加

`materialize_stream` の `VMStream::Join` アーム（L6088〜L6110）の直後（`}` 閉じの前）に追加する。

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

### 5. `fav/src/driver.rs` — `v55400_tests` モジュール追加

`v55300_tests` の直前に挿入する（逆順挿入の慣行に従う）。

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
                // left=1: 右側マッチなし → [1, Unit]
                crate::value::Value::List(vec![
                    crate::value::Value::Int(1),
                    crate::value::Value::Unit,
                ]),
                // left=2: right=2 とマッチ → [2, 2]
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

## テスト仕様

### `stream_join_inner_matches`

`Stream.join_inner` が inner join として動作し、両ストリームでマッチした要素ペアのみを返すことを検証する。

- left=`[1, 2]`、right=`[2, 3]`、述語 `|a, b| a == b`
- 期待値: `[[2, 2]]`（1 件、unmatched は除外）

### `stream_join_left_preserves_unmatched`

`Stream.join_left` が left outer join として動作し、左側ストリームの全要素を保持することを検証する。

- left=`[1, 2]`、right=`[2, 3]`、述語 `|a, b| a == b`
- 期待値: `[[1, Unit], [2, 2]]`（2 件、left=1 は右側マッチなしで Unit）

---

## 完了条件

- `cargo build` が `VMStream::JoinLeft` 追加後に non-exhaustive エラーなく通過する
- `cargo test` 全通過（3213 tests passed, 0 failed）
- `cargo clippy -- -D warnings` クリーン
- `stream_join_inner_matches` pass
- `stream_join_left_preserves_unmatched` pass
- `vm.rs` に `VMStream::JoinLeft` バリアントが追加されている
- `vm.rs` に `Stream.join_inner` / `Stream.join_left` primitive が追加されている
- `CHANGELOG.md` に v55.4.0 エントリが追加されている
- `versions/current.md` が v55.4.0 / 3213 tests を反映
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.4.0 実績を COMPLETE に更新

---

## 備考

- `VMStream::JoinLeft` は `materialize_stream` の `match stream { }` に追加するだけでよい。
  `materialize_stream` が `VMStream` の唯一の消費箇所であるため、他ファイルへの変更は不要。
- `Stream.join_inner` は `Stream.join` と同一の `VMStream::Join` を生成する（内部実装を共有）。
  既存の `stream_join_vm_basic` テストが `Stream.join` を検証しているため、
  `Stream.join_inner` は独立したテストで別途検証する。
- left outer join の "no match" 表現として `VMValue::Unit` を使用する。
  `Value::Unit` は `crate::value::Value` に存在するため、テストの `assert_eq!` に使用可能。
  変換は `impl From<VMValue> for Value`（`vm.rs` の `From` 実装）で `VMValue::Unit => Value::Unit`
  として対応済みであり、`exec_artifact_main` が経由する変換パスでカバーされる。
- **ロードマップとの乖離（ハッシュテーブル）**: `roadmap-v55.1-v56.0.md` には「メモリ内ハッシュテーブルで実装」と記載されているが、
  本バージョンでは既存 `VMStream::Join`（v42.4.0）と同一の nested-loop join を採用する。
  理由: 既存実装との一貫性・VM レベルのシンプルさ優先。テスト対象のウィンドウサイズは小さいため性能上の問題はない。
  ハッシュテーブルベースの最適化は将来バージョン（パフォーマンス最適化スプリント）で対応する。
  ロードマップの記述はこの注記をもって「nested-loop で代替実装」と読み替える。
- **ロードマップとの乖離（並列読み込み）**: `roadmap-v55.1-v56.0.md` には「`par [A, B]` 並列基盤を活用して
  ジョインの両ストリームを並列読み込みする」と記載されているが、本バージョンでは実装しない。
  理由: `materialize_stream` はシングルスレッド設計であり、並列化は `VMStream::Join` の改修を要する大規模変更となる。
  v55.4.0 の目的はストリーム結合 API（`join_inner` / `join_left`）の追加であり、
  並列最適化はスコープ外とする。将来の並列化スプリントで対応する。
- `v55300_tests` には `cargo_toml_version_is_55_3_0` テストが存在しないため削除タスクは不要。
- ロードマップ上の「ベース 3212 + 2 = 3214」は v55.3.0 実績（3211）とのずれがあるため、
  本バージョン実装前にロードマップの数値を 3211 + 2 = 3213 に訂正する。
- ドキュメント MDX は v55.8 でまとめて追加するため本バージョンでは不要。
