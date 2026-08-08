# Spec — v55.5.0 — Stateful stage（累積状態）

## 概要

v55.5.0 は Streaming Native 2.0 スプリント（v55.1〜v55.9）の第 5 弾。
`State.get` / `State.set` / `State.get_or_default` を VM primitive として追加し、
ストリーム処理ステージが型付き累積状態を保持できるようにする。

v22.3.0 で追加済みの `STATE_STORE: HashMap<String, String>`（文字列ベースの raw API）に加え、
新しい `STATE_VALUE_STORE: HashMap<String, VMValue>`（型付き VMValue ベース）を追加する。
`!State` エフェクト構文は Effect enum が v35.5.0 で削除済みのため追加しない。
E0421 エラーコード（`!State` エフェクトなし state 操作）を `error_catalog.rs` に documentation stub として追加する。

具体的には以下を実装する：
1. `vm.rs` に `STATE_VALUE_STORE` thread-local（`HashMap<String, VMValue>`）を追加
2. `vm_call_builtin` に `State.get` / `State.set` / `State.get_or_default` primitive を追加
3. `error_catalog.rs` に E0421 stub エントリを追加
4. `checker.rs` に `("State", "get_or_default")` の型登録を追加
5. `driver.rs` に `v55500_tests` を追加（`stateful_stage_accumulates` / `stateful_stage_persists`）

---

## ロードマップ参照

- `versions/roadmap/roadmap-v55.1-v56.0.md` — v55.5.0 セクション
- ベーステスト数: **3213**（v55.4.0 完了時点の実績値）
- 目標テスト数: **3215**（+2、削除なし）

> **注記**: ロードマップ上のベース値が 3214（3213 + 1 のずれ）と記載されていたが、
> v55.4.0 の実績が 3213 のため本バージョンの目標は **3215**（3213 + 2）とする。
> ロードマップの 3216 記載は v55.5.0 実装前に訂正する。

---

## 既存実装との関係

| 要素 | バージョン | 状態 |
|------|-----------|------|
| `STATE_STORE` thread-local（`HashMap<String, String>`） | v22.3.0 | 実装済み |
| `State.get_raw` / `State.set_raw` / `State.has_raw` / `State.delete_raw` | v22.3.0 | 実装済み（`vm_call_builtin` 内） |
| `("State", "get")` / `("State", "set")` 型テーブル | v22.3.0 | `checker.rs` に登録済み（`require_state_effect` は v35.x で no-op 化） |
| `STATE_VALUE_STORE` thread-local（`HashMap<String, VMValue>`） | — | **未実装（v55.5.0 で追加）** |
| `State.get` / `State.set` / `State.get_or_default` primitive | — | **未実装（v55.5.0 で追加）** |
| E0421 エラーコード | — | **未実装（v55.5.0 で stub 追加）** |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "55.5.0"
```

---

### 2. `fav/src/backend/vm.rs` — `STATE_VALUE_STORE` 追加

`STATE_STORE` / `STATE_BACKEND` の thread-local ブロック（L1422〜L1428 付近）の直後に追加する。

```rust
/// v55.5.0: 型付き State ストア（String key → VMValue）
/// State.get / State.set / State.get_or_default で使用する。
/// State.get_raw / State.set_raw は引き続き STATE_STORE（String→String）を使用。
thread_local! {
    static STATE_VALUE_STORE: std::cell::RefCell<std::collections::HashMap<String, VMValue>>
        = std::cell::RefCell::new(std::collections::HashMap::new());
}
```

---

### 3. `fav/src/backend/vm.rs` — `State.get` / `State.set` / `State.get_or_default` 追加

`vm_call_builtin`（L10013 付近）の `State.delete_raw` アームの直後（`"Bytes.from_hex"` アームの直前）に挿入する。

#### `State.get`

```rust
// v55.5.0: 型付き State API
"State.get" => {
    let key = match args.into_iter().next() {
        Some(VMValue::Str(s)) => s,
        _ => return Err("State.get requires a String key".to_string()),
    };
    let val = STATE_VALUE_STORE.with(|c| c.borrow().get(&key).cloned());
    Ok(ok_vm(match val {
        Some(v) => VMValue::Variant("some".to_string(), Some(Box::new(v))),
        None    => VMValue::Variant("none".to_string(), None),
    }))
}
```

#### `State.set`

```rust
"State.set" => {
    let mut it = args.into_iter();
    let key = match it.next() {
        Some(VMValue::Str(s)) => s,
        _ => return Err("State.set: key must be a String".to_string()),
    };
    let value = match it.next() {
        Some(v) => v,
        None => return Err("State.set: missing value argument".to_string()),
    };
    STATE_VALUE_STORE.with(|c| c.borrow_mut().insert(key, value));
    Ok(ok_vm(VMValue::Unit))
}
```

#### `State.get_or_default`

```rust
"State.get_or_default" => {
    let mut it = args.into_iter();
    let key = match it.next() {
        Some(VMValue::Str(s)) => s,
        _ => return Err("State.get_or_default: key must be a String".to_string()),
    };
    let default_val = match it.next() {
        Some(v) => v,
        None => return Err("State.get_or_default: missing default argument".to_string()),
    };
    let val = STATE_VALUE_STORE.with(|c| c.borrow().get(&key).cloned())
        .unwrap_or(default_val);
    Ok(ok_vm(val))
}
```

---

### 4. `fav/src/error_catalog.rs` — E0421 stub 追加

E0420 エントリの直後（`// ── E05xx: モジュール ──` コメントの直前）に追加する。

```rust
// v55.5.0: Stateful stage — !State エフェクト enforcement stub
ErrorEntry {
    code: "E0421",
    title: "State operation without !State effect",
    category: "streaming",
    description: "A `State.get` / `State.set` / `State.get_or_default` call was used in a stage \
                  that does not declare the `!State` effect. Declare `!State` in the stage signature \
                  to enable stateful accumulation.",
    example: "stage Count: Stream<Int> -> Stream<Int> = |s| {\n  bind n <- State.get_or_default(\"c\", 0)\n  Ok(n)  // E0421: missing !State\n}",
    fix: "Add `!State` to the stage effect list: `stage Count: Stream<Int> -> Stream<Int> = |s| !State { ... }`",
    suggestion: Some("Declare `!State` in the stage signature to enable stateful accumulation."),
},
```

---

### 5. `fav/src/middle/checker.rs` — `State.get_or_default` 型登録追加

`("State", "get")` エントリ（L6446 付近）の直後に追加する。

```rust
("State", "get_or_default") => Some(Type::Unknown), // v55.5.0
```

> **注記**: `State.get_or_default` の戻り値型はデフォルト引数の型と同じ `T` になるが、
> 型変数での表現は現チェッカーの能力を超えるため `Type::Unknown` で登録する。

---

### 6. `fav/src/driver.rs` — `v55500_tests` 追加

`v55400_tests` の直前に挿入する（逆順挿入の慣行に従う）。

```rust
// -- v55500_tests (v55.5.0) -- Stateful stage（累積状態）--
#[cfg(test)]
mod v55500_tests {
    use super::{build_artifact, exec_artifact_main};
    use crate::frontend::parser::Parser;

    #[test]
    fn stateful_stage_accumulates() {
        // State.set で Int 値を保存し、State.get_or_default で取得できることを検証
        let src = r#"public fn main() -> Int {
            bind _ <- State.set("v55500_counter", 42)
            bind val <- State.get_or_default("v55500_counter", 0)
            val
        }"#;
        let program = Parser::parse_str(src, "stateful_accumulate.fav").expect("parse ok");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec ok");
        assert_eq!(
            value,
            crate::value::Value::Int(42),
            "State.get_or_default should return stored value 42, got {:?}", value
        );
    }

    #[test]
    fn stateful_stage_persists() {
        // State.set で Bool 値を保存し、State.get_or_default でデフォルト値を上書きすることを検証
        let src = r#"public fn main() -> Bool {
            bind _ <- State.set("v55500_ready", true)
            bind val <- State.get_or_default("v55500_ready", false)
            val
        }"#;
        let program = Parser::parse_str(src, "stateful_persist.fav").expect("parse ok");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec ok");
        assert_eq!(
            value,
            crate::value::Value::Bool(true),
            "State.get_or_default should return stored Bool true, not default false, got {:?}", value
        );
    }
}
```

---

## テスト仕様

### `stateful_stage_accumulates`

`State.set("key", Int)` で整数値を保存し、`State.get_or_default("key", 0)` で取得できることを検証する。

- `State.set("v55500_counter", 42)` → `ok(Unit)`（状態を保存）
- `State.get_or_default("v55500_counter", 0)` → `ok(42)`（デフォルト値 0 ではなく保存値 42 を返す）
- 最終的に `val`（= 42）を返す
- 期待値: `Value::Int(42)`

### `stateful_stage_persists`

`State.set("key", Bool)` で Boolean 値を保存し、`State.get_or_default` でデフォルト値が上書きされることを検証する。

- `State.set("v55500_ready", true)` → `ok(Unit)`
- `State.get_or_default("v55500_ready", false)` → `ok(true)`（デフォルト false ではなく保存値 true）
- 期待値: `Value::Bool(true)`

> **テストキー命名**: 他テストとの thread-local 汚染を防ぐため、キー名に `v55500_` プレフィックスを付与する。

---

## 完了条件

- `cargo build` コンパイルエラーなし（`STATE_VALUE_STORE` 型に `'static` 制約違反がないことを含む）
- `cargo test` 全通過（3215 tests passed, 0 failed）
- `cargo clippy -- -D warnings` クリーン
- `stateful_stage_accumulates` pass
- `stateful_stage_persists` pass
- `vm.rs` に `STATE_VALUE_STORE` が追加されている
- `vm.rs` に `State.get` / `State.set` / `State.get_or_default` primitive が追加されている
- `error_catalog.rs` に E0421 エントリが追加されている
- `checker.rs` に `("State", "get_or_default")` が登録されている
- `CHANGELOG.md` に v55.5.0 エントリが追加されている
- `versions/current.md` が v55.5.0 / 3215 tests を反映
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.5.0 実績を COMPLETE に更新
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.5.0 実績欄も COMPLETE に更新

---

## 備考

- `!State` エフェクト構文: Effect enum は v35.5.0 で削除済みのため `Effect::State` の追加は不可。
  E0421 は documentation stub のみとし、checker.rs での実 enforcement は将来の構文強化スプリントで対応する。
  `require_state_effect` は v35.x で no-op 化済みのため既存の `("State", "get")` / `("State", "set")` への影響なし。
- `STATE_VALUE_STORE` vs `STATE_STORE`: 両者は独立したストア。
  `State.get_raw` 等の raw API は引き続き `STATE_STORE`（String→String）を使用。
  `State.get` / `State.set` / `State.get_or_default` は `STATE_VALUE_STORE`（String→VMValue）を使用。
  意図的な分離設計（raw API との後方互換性を維持するため）。
- `STATE_VALUE_STORE` の `'static` 制約: `VMValue` は全フィールドが所有型（`Arc`, `Box`, `Vec`, `HashMap` 等）
  のため `'static` を満たす。`thread_local!` に問題なく使用可能。
- v55.3.0 チェックポイントストアとの自動永続化は将来（v55.7 Checkpoint/Replay API）で対応予定。
  本バージョンは in-memory ストア（`STATE_VALUE_STORE`）として実装し、
  ロードマップ記載の永続化（チェックポイントファイルへの書き出し等）はスコープ外とする。
- `vm_call_builtin` での `args.into_iter().next()` パターン: 既存 `State.get_raw` と同一パターン。
  `args.len()` チェックを省略しているが、呼び出し元（コンパイラ生成コード）は引数数を保証する。
- `v55400_tests` には `cargo_toml_version_is_55_4_0` テストが存在しないため削除タスクは不要。
- テストのキー名に `v55500_` プレフィックスを使用する理由: thread_local の `STATE_VALUE_STORE` はスレッドを
  またぎテストが実行されても、同一スレッド内で順次実行されるテスト間で状態が共有される可能性がある。
  ユニークなキー名で衝突を回避する。
- ドキュメント MDX は v55.8 でまとめて追加するため本バージョンでは不要。
