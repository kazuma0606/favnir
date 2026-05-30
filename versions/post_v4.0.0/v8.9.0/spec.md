# Favnir v8.9.0 Spec

Date: 2026-05-30
Theme: checker.fav — 未定義変数検出（E0001）

---

## 背景

v8.8.0 まで、`infer_hm` の `EVar` が env に存在しない変数名を参照した場合、
**fresh type variable（"t0", "t1", ...）を生成して Ok として返す**挙動になっていた。
これは HM 型推論の標準的な fallback だが、Favnir では変数スコープが静的に確定するため、
env に存在しない変数は常に「未定義変数」エラーとして扱うべきである。

v8.9.0 では `EVar None` ケースを E0001 エラーに変更し、未定義変数を検出する。

---

## スコープ

### E0001 が検出される範囲

`check_fn_def` → `infer_hm` のパスで EVar が評価されるコンテキスト:

- 関数ボディの最外側 bind チェーン
- `EIf` の then/else ブランチ（`infer_hm` で再帰）
- `EBind` の rhs および continuation（`infer_hm_let` → `infer_hm` で再帰）
- `ECall` → `infer_call_hm` の直接ハンドラ

### E0001 が検出されない範囲（現バージョン）

- `EMatch` のアーム本体 → `_ =>` fallback → `infer_expr`（pattern-bound vars 安全のため）
- `ELambda` の本体 → `_ =>` fallback → `infer_expr`
- `ECall` の引数 → `infer_arg_tys` → `infer_expr`
- `EIf` の condition → `infer_hm` で処理されない（現状の制限）

この制限は v8.9.0 のスコープ外。`infer_hm` を EMatch / ELambda まで拡張するのは v9.x 以降。

---

## 設計

### 変更: `infer_hm` EVar None ケース

```fav
// Before (v8.8.0):
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

// After (v8.9.0):
EVar(name) => {
    match env_lookup(env, name) {
        Some(ty) => Result.ok(inf_result_of(ty, state))
        None     => Result.err(fmt_err("E0001",
                        String.concat("undefined variable: ", name)))
    }
}
```

---

## env に存在する変数の一覧（有効コードでは必ず in env）

| 変数の種類 | envへの登録タイミング |
|---|---|
| 関数パラメータ | `check_fn_def` → `build_param_env` |
| bind 変数 | `infer_hm_let` → `env_insert(env, vname, rhs_r.ty)` |
| lambda パラメータ | `infer_expr(ELambda(...))` → `env_insert(env, param, "Unknown")` |
| match pattern 変数 | `infer_arms` → `env_from_pat(pat, env)` |
| user-defined 関数名 | `collect_fn_schemes` / `collect_variant_constructors` |
| variant コンストラクター名 | `collect_variant_constructors` |

---

## self-check 安全性

`checker_fav_wire_self_check` では checker.fav 自身が型チェックされる。
checker.fav 内の全変数は上記いずれかのカテゴリに属するため、E0001 は発生しない。

特に注意すべき点:
- **再帰関数**: `collect_fn_schemes` など自身を呼ぶ関数 → `full_env` に関数名が登録されているため OK
- **match アーム内の変数**: `_ =>` fallback → `infer_expr` → `infer_arms` → `env_from_pat` で追加 → `infer_expr` の EVar ハンドラは `"Unknown"` を返す（エラーなし）
- **lambda パラメータ**: `_ =>` fallback → `infer_expr(ELambda(...))` → `env_insert` → OK

---

## 動作確認

```
// E0001 を出すケース
public fn main() -> Int { x }
→ E0001: undefined variable: x

// E0001 を出さないケース
public fn main(n: Int) -> Int { n }
→ OK（n は param として env に登録）

fn add(a: Int, b: Int) -> Int { a + b }
public fn main() -> Int {
    bind r <- add(1, 2)
    r
}
→ OK（r は bind で env に登録）

type MyOpt = | MNone | MSome(Int)
public fn main() -> Int {
    match MSome(42) {
        MNone => 0
        MSome(v) => v
    }
}
→ OK（v はアームボディが infer_expr パスを通り env_from_pat で登録）
```

---

## 注意事項

### `infer_expr` の EVar ハンドラは変更しない

`infer_expr` の `EVar None` → `Result.ok("Unknown")` は**変更しない**。
`infer_expr` は match アーム、lambda 本体、関数引数など、
型が未確定であることが許容されるコンテキストで呼ばれるため。

### 既存テスト: `checker_v87_tests::undefined_fn_e0007`

`undefined_fn_e0007` テスト (`public fn main() -> Int { foo(1) }`) は E0007 を検出する。
`foo` は ECall として処理され、`infer_call_user` で E0007 が出る（EVar ではない）。
E0001 と E0007 は独立して機能する。

### E0001 エラーコードの選択

- E0001: undefined variable（本バージョン）
- E0002: (未使用)
- E0003: undeclared effect
- E0004: non-exhaustive match
- E0005: cannot unify
- E0006: occurs check
- E0007: undefined function
- E0008: arity mismatch
