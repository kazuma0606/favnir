# Spec — v55.6.0 — CEP（複合イベント処理）Stream 統合

## 概要

v55.6.0 は Streaming Native 2.0 スプリント（v55.1〜v55.9）の第 6 弾。
v42.1〜v42.3 で実装済みの `CepPatternDef` / `CepExpr::Seq` / `CepExpr::Any` の
**パース・AST 層**に加え、**VM 実行層**の統合を追加する。

具体的には `CEP.sequence` / `CEP.skip_until` を `call_builtin` に追加し、
リストベースのイベント列に対して順序付きパターンマッチング・スキップを実行できるようにする。
v55.5.0 で追加した `State.get_or_default` と組み合わせて CEP 処理結果を
`STATE_VALUE_STORE` に永続化できることもテストで確認する。

具体的には以下を実装する：
1. `compiler.rs` の namespace 登録リストに `"CEP"` を追加
2. `checker.rs` に `("CEP", "sequence")` / `("CEP", "skip_until")` を型登録
3. `vm.rs` の `call_builtin` に `CEP.sequence` / `CEP.skip_until` を追加
4. `driver.rs` に `v55600_tests` を追加（`cep_stream_integration` / `cep_stateful_persistence`）

---

## ロードマップ参照

- `versions/roadmap/roadmap-v55.1-v56.0.md` — v55.6.0 セクション
- `versions/roadmap/roadmap-v55.1-v60.0.md` — v55.6.0 行
- ベーステスト数: **3215**（v55.5.0 完了時点の実績値）
- 目標テスト数: **3217**（+2、削除なし）

> **注記**: ロードマップ上のベース値が 3216（3215 + 1 のずれ）と記載されているため、
> 完了条件が 3218 と記載されているが、v55.5.0 の実績が 3215 のため
> 本バージョンの目標は **3217**（3215 + 2）とする。
> ロードマップの 3218 記載は v55.6.0 実装前に訂正する。

---

## 既存実装との関係

| 要素 | バージョン | 状態 |
|------|-----------|------|
| `CepPatternDef` AST ノード | v42.1.0 | 実装済み（parser / ast.rs） |
| `CepExpr::Seq` / `CepExpr::Any` / `CepExpr::Not` | v42.2.0 | 実装済み（ast.rs） |
| CEP x Refinement type チェック | v44.2.0 | 実装済み（`collect_cep_refinement_event_refs`） |
| `CEP` namespace（`compiler.rs` 登録） | — | **未実装（v55.6.0 で追加）** |
| `CEP.sequence` / `CEP.skip_until` VM primitive | — | **未実装（v55.6.0 で追加）** |
| `("CEP", ...)` 型テーブル（`checker.rs`） | — | **未実装（v55.6.0 で追加）** |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "55.6.0"
```

---

### 2. `fav/src/middle/compiler.rs` — `"CEP"` namespace 登録追加

`"State"` エントリ（v55.5.0 で追加）の直後に追加する。

```rust
// v55.6.0 CEP（複合イベント処理 — CEP.sequence/skip_until namespace として登録）
"CEP",
```

---

### 3. `fav/src/middle/checker.rs` — `CEP.sequence` / `CEP.skip_until` 型登録

`("Stream", "join_left")` エントリ（L6964 付近）と `("Stream", _)` ワイルドカードの直後（次の namespace ブロックの前）に追加する。

```rust
// CEP (v55.6.0): 複合イベント処理 VM primitive
("CEP", "sequence")   => Some(Type::List(Box::new(Type::Unknown))), // v55.6.0
("CEP", "skip_until") => Some(Type::List(Box::new(Type::Unknown))), // v55.6.0
("CEP", _)            => Some(Type::Unknown),
```

> **注記**: `CEP.sequence` の戻り値は `List<List<T>>` だが、内側リストの型変数表現は
> 現チェッカーの能力を超えるため `List(Unknown)` で登録する。

---

### 4. `fav/src/backend/vm.rs` — `CEP.sequence` / `CEP.skip_until` 追加

`call_builtin` 内の `Stream.join_left` アームの直後（`// ── end v26.4.0 / v55.4.0 Stream.* ──` コメントの直前）に挿入する。

`CEP.sequence` / `CEP.skip_until` はクロージャ（述語）を実行するため `&mut self` が必要であり、
`vm_call_builtin`（free function）ではなく `call_builtin`（`&mut self` メソッド）に追加する。

#### `CEP.sequence`

```rust
// ── v55.6.0: CEP 複合イベント処理 ────────────────────────────────────────────
// CEP.sequence(events: List, preds: List<Fn>) -> List<List>
// イベントリストに対して述語列を順番に適用し、すべてマッチした部分列を返す。
// 各開始位置から preds[0](events[i]) が true なら、i より後の位置で
// preds[1](events[j]) を探し、以降同様に連鎖させた部分列を収集する。
"CEP.sequence" => {
    if args.len() != 2 {
        return Err(self.error(artifact, "CEP.sequence requires 2 arguments: (events: List, preds: List<Fn>)"));
    }
    let mut it = args.into_iter();
    let events = match it.next().unwrap() {
        VMValue::List(l) => l.to_vec(),
        other => return Err(self.error(artifact, &format!(
            "CEP.sequence: first argument must be a List, got {}", vmvalue_type_name(&other)
        ))),
    };
    let preds = match it.next().unwrap() {
        VMValue::List(l) => l.to_vec(),
        other => return Err(self.error(artifact, &format!(
            "CEP.sequence: second argument must be a List of predicates, got {}", vmvalue_type_name(&other)
        ))),
    };
    if preds.is_empty() {
        return Ok(VMValue::List(FavList::new(vec![])));
    }
    let mut results = Vec::new();
    for start in 0..events.len() {
        // 先頭述語チェック
        let first_ok = self.call_value(artifact, preds[0].clone(), vec![events[start].clone()])?;
        if !matches!(first_ok, VMValue::Bool(true)) {
            continue;
        }
        // 残りの述語を順番に探索
        let mut current = vec![events[start].clone()];
        let mut pos = start + 1;
        let mut pred_i = 1;
        while pred_i < preds.len() && pos < events.len() {
            let m = self.call_value(artifact, preds[pred_i].clone(), vec![events[pos].clone()])?;
            if matches!(m, VMValue::Bool(true)) {
                current.push(events[pos].clone());
                pred_i += 1;
            }
            pos += 1;
        }
        if pred_i == preds.len() {
            results.push(VMValue::List(FavList::new(current)));
        }
    }
    Ok(VMValue::List(FavList::new(results)))
}
```

#### `CEP.skip_until`

```rust
// CEP.skip_until(events: List, pred: Fn) -> List
// イベントリストを先頭から走査し、pred が最初に true になった要素から末尾まで返す。
// pred が一度も true にならない場合は空リストを返す。
"CEP.skip_until" => {
    if args.len() != 2 {
        return Err(self.error(artifact, "CEP.skip_until requires 2 arguments: (events: List, pred: Fn)"));
    }
    let mut it = args.into_iter();
    let events = match it.next().unwrap() {
        VMValue::List(l) => l.to_vec(),
        other => return Err(self.error(artifact, &format!(
            "CEP.skip_until: first argument must be a List, got {}", vmvalue_type_name(&other)
        ))),
    };
    let pred = it.next().unwrap();
    let mut result = Vec::new();
    let mut found = false;
    for event in events {
        if !found {
            let m = self.call_value(artifact, pred.clone(), vec![event.clone()])?;
            if matches!(m, VMValue::Bool(true)) {
                found = true;
                result.push(event);
            }
        } else {
            result.push(event);
        }
    }
    Ok(VMValue::List(FavList::new(result)))
}
```

---

### 5. `fav/src/driver.rs` — `v55600_tests` 追加

`v55500_tests` の直前に挿入する（逆順挿入の慣行に従う）。

```rust
// -- v55600_tests (v55.6.0) -- CEP（複合イベント処理）Stream 統合 --
#[cfg(test)]
mod v55600_tests {
    use super::{build_artifact, exec_artifact_main};
    use crate::backend::vm::clear_state_value_store;
    use crate::frontend::parser::Parser;

    #[test]
    fn cep_stream_integration() {
        // CEP.sequence: 2 述語に対して順序付きマッチを行い、マッチ件数を検証
        let src = r#"public fn main() -> Int {
            let events = ["login", "purchase", "logout", "login", "purchase"]
            let matches = CEP.sequence(events, [|e| e == "login", |e| e == "purchase"])
            List.length(matches)
        }"#;
        let program = Parser::parse_str(src, "cep_stream_integration.fav").expect("parse ok");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec ok");
        assert_eq!(
            value,
            crate::value::Value::Int(2),
            "CEP.sequence should find 2 (login,purchase) pairs, got {:?}", value
        );
    }

    #[test]
    fn cep_stateful_persistence() {
        // CEP.skip_until + State: スキップ結果の長さを State に保存し取り出す
        clear_state_value_store();
        let src = r#"public fn main() -> Int {
            let events = ["noise", "noise", "start", "a", "b"]
            let filtered = CEP.skip_until(events, |e| e == "start")
            bind _ <- State.set("v55600_cep_len", List.length(filtered))
            bind n <- State.get_or_default("v55600_cep_len", 0)
            n
        }"#;
        let program = Parser::parse_str(src, "cep_stateful_persistence.fav").expect("parse ok");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec ok");
        assert_eq!(
            value,
            crate::value::Value::Int(3),
            "CEP.skip_until should skip 2 noises, return [start,a,b] len=3, got {:?}", value
        );
    }
}
```

---

## テスト仕様

### `cep_stream_integration`

`CEP.sequence(events, [pred1, pred2])` でイベントリストから順序付き部分列を抽出する。

- events = `["login", "purchase", "logout", "login", "purchase"]`
- preds = `[|e| e == "login", |e| e == "purchase"]`
- マッチ（各開始位置の判定）:
  - start=0: `"login"` ✓ pred[0]、次に pos=1 `"purchase"` ✓ pred[1] → `["login","purchase"]`
  - start=1: `"purchase"` ✗ pred[0]（`"purchase" != "login"`）→ スキップ
  - start=2: `"logout"` ✗ pred[0] → スキップ
  - start=3: `"login"` ✓ pred[0]、次に pos=4 `"purchase"` ✓ pred[1] → `["login","purchase"]`
  - start=4: `"purchase"` ✗ pred[0] → スキップ
- `List.length(matches)` = 2
- 期待値: `Value::Int(2)`

### `cep_stateful_persistence`

`CEP.skip_until` でイベントリストを先頭から走査し、条件を満たした位置以降を取得。
取得結果の長さを `State.set` で保存し、`State.get_or_default` で取り出す。

- events = `["noise", "noise", "start", "a", "b"]`
- pred = `|e| e == "start"`
- `CEP.skip_until` → `["start", "a", "b"]`（先頭 2 件スキップ）
- `List.length(filtered)` = 3
- `State.set("v55600_cep_len", 3)` → `Unit`
- `State.get_or_default("v55600_cep_len", 0)` → `3`
- 期待値: `Value::Int(3)`

> **テストキー命名**: `v55600_cep_len` に `v55600_` プレフィックスを付与し thread-local 汚染を防ぐ。

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（3217 tests passed, 0 failed）
- `cargo clippy -- -D warnings` クリーン
- `cep_stream_integration` pass
- `cep_stateful_persistence` pass
- `compiler.rs` に `"CEP"` が namespace 登録されている
- `checker.rs` に `("CEP", "sequence")` / `("CEP", "skip_until")` が登録されている
- `vm.rs` に `CEP.sequence` / `CEP.skip_until` が `call_builtin` に追加されている
- `CHANGELOG.md` に v55.6.0 エントリが追加されている
- `versions/current.md` が v55.6.0 / 3217 tests を反映
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.6.0 実績を COMPLETE に更新
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.6.0 実績欄も COMPLETE に更新

---

## 備考

- `CEP.sequence` / `CEP.skip_until` は述語クロージャを呼び出す必要があるため
  `vm_call_builtin`（free function）ではなく `call_builtin`（`&mut self` メソッド）に実装する。
  `List.map` / `List.filter` と同様のパターン。
- `CEP.sequence` の探索アルゴリズム: 各開始位置からの greedy 前向き探索。
  複数の開始位置からマッチが得られる場合はすべて収集する。
  同一イベントを複数の開始位置で使い回すことを許容する（オーバーラップ許可）。
- `CEP.skip_until` は pred が最初に true になった要素を**含む**サフィックスを返す。
  pred が一度も true にならない場合は空リストを返す。
- `"CEP"` namespace は `is_known_builtin_namespace`（`vm.rs`）への追加が**必須**。
  v55.5.0 で `"State"` は既存登録済みだったが、`"CEP"` は未登録のため両方の追加が必要：
  1. `compiler.rs` 登録リスト（未登録だと `IRExpr::Global(u16::MAX)` → runtime error）
  2. `is_known_builtin_namespace`（未登録だと `LoadGlobal` での名前解決に失敗）
- ロードマップ記載の `CEP.match` / `CEP.then` / `within_sec` 構文は
  本バージョンでは実装しない（より複雑な構文解析が必要）。
  `CEP.sequence(events, [pred1, pred2])` の関数 API で代替する。
- ドキュメント MDX は v55.8 でまとめて追加するため本バージョンでは不要。
