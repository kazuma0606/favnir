# Favnir v8.9.0 実装計画

Date: 2026-05-30

---

## Phase A: `infer_hm` EVar None ケース変更

**変更ファイル**: `fav/self/checker.fav`

`infer_hm` の `EVar` ハンドラで、`env_lookup` が `None` を返したとき
fresh type variable を生成する代わりに E0001 エラーを返す:

```fav
// Before:
EVar(name) => {
    match env_lookup(env, name) {
        Some(ty) => Result.ok(inf_result_of(ty, state))
        None     => {
            bind tv <- fresh_var(state.counter)
            Result.ok(inf_result_of(tv,
                inf_state_new(state.subst, state.counter + 1)))
        }
    }
}

// After:
EVar(name) => {
    match env_lookup(env, name) {
        Some(ty) => Result.ok(inf_result_of(ty, state))
        None     => Result.err(fmt_err("E0001",
                        String.concat("undefined variable: ", name)))
    }
}
```

---

## Phase B: テスト追加

**変更ファイル**: `fav/src/driver.rs`

`checker_v87_tests` または新モジュール `checker_v89_tests` に追加。

### B-1: `undefined_var_e0001`

```rust
#[test]
fn undefined_var_e0001() {
    // 未定義変数の参照 → E0001
    let errors = check_errors(
        "public fn main() -> Int { x }",
    );
    assert!(
        errors.iter().any(|e| e.contains("E0001")),
        "expected E0001 for undefined variable, got: {:?}",
        errors
    );
}
```

### B-2: `fn_param_not_e0001`

```rust
#[test]
fn fn_param_not_e0001() {
    // 関数パラメータは定義済み → エラーなし
    let errors = check_errors(
        "public fn main(n: Int) -> Int { n }",
    );
    assert!(
        errors.is_empty(),
        "fn param should not cause E0001: {:?}",
        errors
    );
}
```

### B-3: `let_bound_not_e0001`

```rust
#[test]
fn let_bound_not_e0001() {
    // bind で束縛した変数は定義済み → エラーなし
    let errors = check_errors(r#"
fn add(a: Int, b: Int) -> Int { a + b }
public fn main() -> Int {
    bind r <- add(1, 2)
    r
}
"#);
    assert!(
        errors.is_empty(),
        "let-bound var should not cause E0001: {:?}",
        errors
    );
}
```

---

## Phase C: テスト実行

```
cargo test checker_v89
cargo test checker_fav         ← 17 + 3 = 20 件通ること
cargo test                     ← 全件通ること（目標 1131 tests）
```

---

## Phase D: 最終確認

- `cargo build` — コンパイルエラーなし
- `checker_fav_wire_self_check` — 64MB スタックで通ること
- tasks.md 完了・commit

---

## 実装ノート

### 変更の単純さ

Phase A は 5 行の削除 + 1 行の追加のみ。他のファイルへの影響なし。
`fresh_var` 関数は削除しない（他所で使われている可能性あり）。

### EIf の condition が E0001 の対象外になる理由

`infer_hm` の EIf ケース:
```fav
EIf(cond, then_e, else_e) =>
    Result.and_then(infer_hm(then_e, env, state), |tr|
    Result.and_then(infer_hm(else_e, env, inf_state_of(tr)), |er|
    ...))
```

`cond` は `infer_hm` の処理対象になっていない（設計上の制限）。
`cond` 内の未定義変数は E0001 にならないが、これは v8.9.0 スコープ外。

### `infer_expr` の EVar は変更しない

`infer_expr` の `EVar None → "Unknown"` はそのまま残す。
これは match アームボディ / lambda 本体 / 関数引数などで使われる
型推論の「寛容モード」として機能しており、E0001 スコープ外のコンテキストを安全に処理する。

### `checker_fav_wire_self_check` の安全性

checker.fav の有効なコードに含まれるすべての EVar は:
- 関数パラメータ（`build_param_env` で登録）
- bind 変数（`infer_hm_let` で登録）
- 関数名（`collect_fn_schemes` で登録）
のいずれかに該当するため、`env_lookup` が None になるケースは存在しない。
match アームや lambda 内の変数は `infer_expr` パスを通るため E0001 の対象外。
