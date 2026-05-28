# Favnir v7.9.0 仕様書

Date: 2026-05-28
Theme: checker.fav HM 型推論（完全版）— `infer_hm` + `unify_deep` + `occurs_in`

---

## 概要

v7.9.0 では `checker.fav` に **Hindley-Milner スタイルの型推論** を実装する。
v7.8.0 で構築した `unify`（flat 型のみ）/ `subst_*` ヘルパーを基盤として、以下を追加する。

1. **`occurs_in`** — 型変数が型文字列中に現れるかを確認（無限型防止、E0006）
2. **`unify_deep`** — `"List<A>"` vs `"List<Int>"` のようなネスト型の単一化
3. **`InfState` / `InfResult`** — 推論状態レコード（substitution + fresh 変数カウンタ）
4. **`fresh_var`** — `"t0"`, `"t1"`, ... の新鮮型変数生成
5. **`infer_hm`** — 状態を引き回す完全な HM 推論パス
6. **`check_fn_def` 更新** — `infer_expr` → `infer_hm` への差し替え

v7.9.0 完了後、`checker.fav` は Rust 製 `checker.rs` の基本型推論機能と同等になる。

---

## 新規型定義

### `InfState`

推論の途中状態。substitution と fresh 変数カウンタを保持する。

```favnir
type InfState = {
    subst:   List<KVPair>
    counter: Int
}
```

### `InfResult`

推論結果。推論した型文字列と更新後の状態を同時に返す。

```favnir
type InfResult = {
    ty:      String
    subst:   List<KVPair>
    counter: Int
}
```

---

## 新規関数

### `occurs_in(var: String, ty: String) -> Bool`

型変数 `var` が `ty` の中に出現するかを確認する。
現バージョンでは flat 型文字列と単一レベルのパラメータ化型を対象とする。

```
occurs_in("A", "A")           → true
occurs_in("A", "Int")         → false
occurs_in("A", "List<A>")     → true
occurs_in("A", "List<Int>")   → false
occurs_in("A", "Option<A>")   → true
```

実装方針:
- `ty == var` → `true`
- `String.contains(ty, "<")` → inner を取り出して `inner == var || String.contains(inner, var)` で確認
- それ以外 → `false`

### `unify_deep(t1: String, t2: String, subst: List<KVPair>) -> Result<List<KVPair>, String>`

v7.8.0 の `unify`（flat 型のみ）を拡張し、パラメータ化型のネストを再帰的に処理する。

```
unify_deep("A",          "Int",       []) → Ok([A→Int])
unify_deep("List<A>",    "List<Int>", []) → Ok([A→Int])
unify_deep("Option<A>",  "Option<B>", []) → Ok([A→B])
unify_deep("List<Int>",  "List<Int>", []) → Ok([])
unify_deep("List<A>",    "Option<A>", []) → Err("E0005: cannot unify List with Option")
```

実装方針:
1. `t1 == t2` → `Ok(subst)`
2. `t1 == "Unknown" || t2 == "Unknown"` → `Ok(subst)`
3. `is_type_var(t1)` → occurs_in チェック後 subst に挿入（conflict/occurs チェック）
4. `is_type_var(t2)` → 同上
5. 両方がパラメータ化型 (`contains "<"`) → outer を比較、不一致なら `Err`、一致なら inner を再帰的に `unify_deep`
6. それ以外 → `Err("E0005: cannot unify t1 with t2")`

occurs check: 型変数 `A` が `ty` に出現している場合は `Err("E0006: occurs check failed: A in ty")` を返す（無限型防止）。

### `fresh_var(counter: Int) -> String`

カウンタから新鮮型変数名を生成する。

```
fresh_var(0) → "t0"
fresh_var(1) → "t1"
fresh_var(9) → "t9"
```

実装: `String.concat("t", Int.to_string(counter))`

### `inf_state_new(subst: List<KVPair>, counter: Int) -> InfState`

`InfState` コンストラクタヘルパー。

### `inf_result_of(ty: String, state: InfState) -> InfResult`

`InfResult` を `(ty, state)` から構築するヘルパー。

```favnir
fn inf_result_of(ty: String, state: InfState) -> InfResult {
    InfResult { ty: ty, subst: state.subst, counter: state.counter }
}
```

### `inf_state_of(r: InfResult) -> InfState`

`InfResult` から `InfState` を取り出す。

```favnir
fn inf_state_of(r: InfResult) -> InfState {
    InfState { subst: r.subst, counter: r.counter }
}
```

### `infer_hm(expr: Expr, env: List<KVPair>, state: InfState) -> Result<InfResult, String>`

HM スタイルの型推論。`infer_expr` の上位互換。状態を引き回しながら型を推論する。

```favnir
fn infer_hm(expr: Expr, env: List<KVPair>, state: InfState) -> Result<InfResult, String>
```

各ノードの処理:

| ノード | 処理 |
|--------|------|
| `EInt(_)` | `"Int"` を返す、state 変更なし |
| `EFloat(_)` | `"Float"` を返す、state 変更なし |
| `EBool(_)` | `"Bool"` を返す、state 変更なし |
| `EStr(_)` | `"String"` を返す、state 変更なし |
| `EUnit` | `"Unit"` を返す、state 変更なし |
| `EVar(name)` | `env_lookup(env, name)` → `Some(ty)` なら `ty`、`None` なら `fresh_var(state.counter)` を生成（counter+1） |
| `EBinop(op, l, r)` | 左右を順番に infer_hm → `unify_deep` で型を統合 |
| `ECall(ns, fname, args)` | `infer_call` をそのまま使用（現行通り）、state はスルー |
| `EIf(cond, then, else_)` | cond を infer_hm → then/else を順に infer_hm → then/else の型を `unify_deep` |
| `ELet(name, val, body)` | val を infer_hm → env に `(name, ty)` を追加 → body を infer_hm |
| `ELambda(params, body)` | 各 param に fresh_var を割り当て → env に追加 → body を infer_hm → `"Fn"` を返す（v7.9.0 スコープ） |
| `EMatch(scrut, arms)` | scrut を infer_hm → arms を順次 infer_hm → 全アーム型を `unify_deep` で統合 |
| それ以外 | `infer_expr` にフォールバック → state 変更なし |

#### 状態スレッディングパターン

Favnir では `bind inside closure` が不可なため、状態の引き回しは専用ヘルパー関数に分割する。

```favnir
// EBinop の例
fn infer_hm_binop(op: String, l: Expr, r: Expr, env: List<KVPair>, state: InfState)
    -> Result<InfResult, String>
{
    Result.and_then(infer_hm(l, env, state), |lr| {
        Result.and_then(infer_hm(r, env, inf_state_of(lr)), |rr| {
            Result.and_then(unify_deep(lr.ty, rr.ty, rr.subst), |s2| {
                Result.ok(inf_result_of(apply_subst(s2, lr.ty), inf_state_new(s2, rr.counter)))
            })
        })
    })
}
```

クロージャ内でフィールドアクセス（`lr.ty`, `lr.subst`）は可能。`bind` 文は不可。

### `check_fn_def` 更新

`infer_expr` の呼び出しを `infer_hm` に差し替え。

```favnir
fn check_fn_def(fd: FnDef, env: List<KVPair>) -> Option<String> {
    bind init_state <- InfState { subst: subst_empty(), counter: 0 }
    Result.and_then(infer_hm(fd.body, env, init_state), |r| {
        // ret 型との照合（未宣言エフェクトチェックは別途）
        ...
    })
}
```

---

## エラーコード

| コード | 意味 |
|--------|------|
| `E0001` | arithmetic 型ミスマッチ |
| `E0002` | logical 演算子に Bool 以外 |
| `E0003` | undeclared effect |
| `E0004` | non-exhaustive match |
| `E0005` | 型単一化の失敗（conflict / cannot unify）|
| `E0006` | occurs check 失敗（無限型）|

---

## 既知の制約（v7.9.0 スコープ外）

- **多相型**: Let ポリモーフィズム（型スキーム generalization）は v8.0.0 へ持ち越し
- **lambda 戻り型**: `"Fn"` のまま（型パラメータ化は v8.0.0）
- **ネスト 2 段以上のパラメータ化型**: `"Map<String, List<Int>>"` の inner 単一化は v8.0.0
- **型変数の多文字対応**: `"T1"`, `"Elem"` 等は v8.0.0（現在は 1 文字大文字のみ）
- **複数パラメータ型の完全単一化**: `"Result<A, B>"` vs `"Result<Int, String>"` — outer 比較のみ（inner 展開は v8.0.0）

---

## テスト計画

### Favnir 内テスト（10 件）

| ID | テスト名 | 検証内容 |
|----|----------|----------|
| E-1 | `occurs_in same` | `occurs_in("A", "A") == true` |
| E-2 | `occurs_in different` | `occurs_in("A", "Int") == false` |
| E-3 | `occurs_in nested` | `occurs_in("A", "List<A>") == true` |
| E-4 | `fresh_var zero` | `fresh_var(0) == "t0"` |
| E-5 | `fresh_var ten` | `fresh_var(10) == "t10"` |
| E-6 | `unify_deep same` | `unify_deep("Int", "Int", []) == Ok([])` |
| E-7 | `unify_deep var nested` | `unify_deep("List<A>", "List<Int>", [])` → Ok, `A → Int` |
| E-8 | `unify_deep outer mismatch` | `unify_deep("List<A>", "Option<Int>", [])` → Err E0005 |
| E-9 | `infer_hm int literal` | EInt → `"Int"` |
| E-10 | `infer_hm let binding` | `let x = 1 in x` → `"Int"` |

### driver.rs 統合テスト（3 件）

| ID | テスト名 | 検証内容 |
|----|----------|----------|
| F-1 | `checker_fav_occurs_in` | `occurs_in("A", "List<A>")` → `true` |
| F-2 | `checker_fav_unify_deep_nested` | `unify_deep("List<A>", "List<Int>", [])` → Ok, A→Int |
| F-3 | `checker_fav_fresh_var` | `fresh_var(0)` → `"t0"` |

---

## 完了条件

- `fav check fav/self/checker.fav` — no errors
- `unify_deep("List<A>", "List<Int>", subst_empty())` が `Ok` で `A → Int` を含む
- `occurs_in("A", "List<A>")` → `true`
- `fresh_var(0)` → `"t0"`
- `infer_hm(EInt(42), [], state)` → `InfResult { ty: "Int", ... }`
- 統合テスト 13 件追加（checker.fav 内 10 + driver.rs 3）
- 既存テスト 1107 件が全件通る（1120+ passing）
