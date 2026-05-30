# Favnir v7.9.0 実装計画

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|----------|----------|
| `fav/self/checker.fav` | Phase A〜D 全変更（~200 行追加） |
| `fav/src/driver.rs` | `checker_v79_tests` モジュール追加（3 テスト） |
| `site/content/docs/language/self-host-checker.mdx` | v7.9.0 セクション追記 |
| `versions/v7.9.0/tasks.md` | 完了状態に更新 |

---

## Phase A: occurs_in と unify_deep

### A-1: `occurs_in`

```favnir
fn occurs_in(var: String, ty: String) -> Bool {
    if ty == var { true }
    else {
    if String.contains(ty, "<") {
        bind inner <- type_str_inner(ty)
        inner == var || String.contains(inner, var)
    } else { false }
    }
}
```

注意: `else if` 非対応 → `else { if ... } }` + 閉じ括弧は 1 個（2 if-branch → N-1 = 1）。

### A-2: `unify_deep`

v7.8.0 の `unify` を拡張してネスト型に対応。

```favnir
fn unify_deep(t1: String, t2: String, subst: List<KVPair>) -> Result<List<KVPair>, String> {
    if t1 == "Unknown" || t2 == "Unknown" { Result.ok(subst) }
    else {
    if t1 == t2 { Result.ok(subst) }
    else {
    if is_type_var(t1) {
        if occurs_in(t1, t2) {
            Result.err(fmt_err("E0006",
                String.concat("occurs check failed: ", String.concat(t1, String.concat(" in ", t2)))))
        } else {
        match subst_lookup(subst, t1) {
            None => Result.ok(subst_insert(subst, t1, t2))
            Some(existing) =>
                if existing == t2 { Result.ok(subst) }
                else { Result.err(fmt_err("E0005",
                    String.concat("conflict: ", String.concat(t1,
                    String.concat(" was ", String.concat(existing,
                    String.concat(", got ", t2))))))) }
        }
        }
    } else {
    if is_type_var(t2) {
        if occurs_in(t2, t1) {
            Result.err(fmt_err("E0006",
                String.concat("occurs check failed: ", String.concat(t2, String.concat(" in ", t1)))))
        } else {
        match subst_lookup(subst, t2) {
            None => Result.ok(subst_insert(subst, t2, t1))
            Some(existing) =>
                if existing == t1 { Result.ok(subst) }
                else { Result.err(fmt_err("E0005",
                    String.concat("conflict: ", String.concat(t2,
                    String.concat(" was ", String.concat(existing,
                    String.concat(", got ", t1))))))) }
        }
        }
    } else {
    if String.contains(t1, "<") && String.contains(t2, "<") {
        bind o1 <- type_str_outer(t1)
        bind o2 <- type_str_outer(t2)
        if o1 == o2 {
            bind i1 <- type_str_inner(t1)
            bind i2 <- type_str_inner(t2)
            unify_deep(i1, i2, subst)
        } else {
            Result.err(fmt_err("E0005",
                String.concat("cannot unify ", String.concat(o1, String.concat(" with ", o2)))))
        }
    } else {
        Result.err(fmt_err("E0005",
            String.concat("cannot unify ", String.concat(t1, String.concat(" with ", t2)))))
    }}}}}
}
```

閉じ括弧カウント: 6 if-branch（最後のみ `else { expr }`） → 5 個の `}` + ネスト `}` に注意。
正確な末尾: `}}}}}` (6 else-branch のうち最後が `else { Result.err(...) }` なので残りは 5 個の `}`)。

---

## Phase B: InfState / InfResult と補助関数

### B-1: 型定義

checker.fav の型定義セクション（既存の `TypeDef` の後）に追加。

```favnir
type InfState = {
    subst:   List<KVPair>
    counter: Int
}

type InfResult = {
    ty:      String
    subst:   List<KVPair>
    counter: Int
}
```

### B-2: ヘルパー関数

```favnir
fn fresh_var(counter: Int) -> String {
    String.concat("t", Int.to_string(counter))
}

fn inf_state_new(subst: List<KVPair>, counter: Int) -> InfState {
    InfState { subst: subst, counter: counter }
}

fn inf_result_of(ty: String, state: InfState) -> InfResult {
    InfResult { ty: ty, subst: state.subst, counter: state.counter }
}

fn inf_state_of(r: InfResult) -> InfState {
    InfState { subst: r.subst, counter: r.counter }
}
```

---

## Phase C: infer_hm

状態スレッディングが必要なため、複雑なノードは専用ヘルパーに分割する。

### C-1: `infer_hm_binop` ヘルパー

```favnir
fn infer_hm_binop(op: String, l: Expr, r: Expr, env: List<KVPair>, state: InfState)
    -> Result<InfResult, String>
{
    Result.and_then(infer_hm(l, env, state), |lr| {
        Result.and_then(infer_hm(r, env, inf_state_of(lr)), |rr| {
            Result.and_then(unify_deep(lr.ty, rr.ty, rr.subst), |s2| {
                Result.ok(inf_result_of(
                    apply_subst(s2, lr.ty),
                    inf_state_new(s2, rr.counter)))
            })
        })
    })
}
```

### C-2: `infer_hm_let` ヘルパー

```favnir
fn infer_hm_let(name: String, val: Expr, body: Expr, env: List<KVPair>, state: InfState)
    -> Result<InfResult, String>
{
    Result.and_then(infer_hm(val, env, state), |vr| {
        bind env2 <- env_insert(env, name, vr.ty)
        infer_hm(body, env2, inf_state_of(vr))
    })
}
```

### C-3: `infer_hm_if` ヘルパー

```favnir
fn infer_hm_if(cond: Expr, then_: Expr, else_: Expr, env: List<KVPair>, state: InfState)
    -> Result<InfResult, String>
{
    Result.and_then(infer_hm(cond, env, state), |cr| {
        Result.and_then(infer_hm(then_, env, inf_state_of(cr)), |tr| {
            Result.and_then(infer_hm(else_, env, inf_state_of(tr)), |er| {
                Result.and_then(unify_deep(tr.ty, er.ty, er.subst), |s2| {
                    Result.ok(inf_result_of(
                        apply_subst(s2, tr.ty),
                        inf_state_new(s2, er.counter)))
                })
            })
        })
    })
}
```

### C-4: `infer_hm_lambda` ヘルパー

パラメータに fresh_var を割り当て、本体を推論する。戻り型は `"Fn"`（v7.9.0 スコープ）。

```favnir
fn infer_hm_lambda_params(params: List<Param>, env: List<KVPair>, state: InfState)
    -> InfState
{
    match List.first(params) {
        None => state
        Some(p) => {
            bind tv <- fresh_var(state.counter)
            bind env2 <- env_insert(env, p.name, tv)
            infer_hm_lambda_params(
                List.drop(params, 1),
                env2,
                inf_state_new(state.subst, state.counter + 1))
        }
    }
}
```

注意: `infer_hm_lambda_params` は `env` の更新を返せないため、別アプローチが必要。
実際には `env` と `state` をペアで返す `InfLambdaCtx` レコードを使う。

```favnir
type InfLambdaCtx = {
    env:     List<KVPair>
    counter: Int
}

fn infer_hm_add_params(params: List<Param>, env: List<KVPair>, counter: Int) -> InfLambdaCtx {
    match List.first(params) {
        None => InfLambdaCtx { env: env, counter: counter }
        Some(p) => {
            bind tv <- fresh_var(counter)
            infer_hm_add_params(
                List.drop(params, 1),
                env_insert(env, p.name, tv),
                counter + 1)
        }
    }
}
```

### C-5: `infer_hm` 本体

```favnir
fn infer_hm(expr: Expr, env: List<KVPair>, state: InfState) -> Result<InfResult, String> {
    match expr {
        EInt(_)   => Result.ok(inf_result_of("Int",    state))
        EFloat(_) => Result.ok(inf_result_of("Float",  state))
        EBool(_)  => Result.ok(inf_result_of("Bool",   state))
        EStr(_)   => Result.ok(inf_result_of("String", state))
        EUnit     => Result.ok(inf_result_of("Unit",   state))
        EVar(name) => {
            match env_lookup(env, name) {
                Some(ty) => Result.ok(inf_result_of(ty, state))
                None     => {
                    bind tv <- fresh_var(state.counter)
                    Result.ok(inf_result_of(tv, inf_state_new(state.subst, state.counter + 1)))
                }
            }
        }
        EBinop(op, l, r) => infer_hm_binop(op, l, r, env, state)
        ELet(name, val, body) => infer_hm_let(name, val, body, env, state)
        EIf(cond, then_, else_) => infer_hm_if(cond, then_, else_, env, state)
        ELambda(params, body) => {
            bind ctx <- infer_hm_add_params(params, env, state.counter)
            Result.and_then(infer_hm(body, ctx.env, inf_state_new(state.subst, ctx.counter)), |br| {
                Result.ok(inf_result_of("Fn", inf_state_of(br)))
            })
        }
        _ => {
            Result.and_then(infer_expr(expr, env), |ty| {
                Result.ok(inf_result_of(ty, state))
            })
        }
    }
}
```

---

## Phase D: check_fn_def 更新と E0006 追加

### D-1: `fmt_err` テーブルに E0006 を追加

既存の `fmt_err` は汎用フォーマット関数のため変更不要。
ドキュメントとエラーコード一覧に E0006 を追記するのみ。

### D-2: `check_fn_def` の `infer_hm` 統合

```favnir
fn check_fn_def(fd: FnDef, env: List<KVPair>) -> Option<String> {
    bind init_state <- inf_state_new(subst_empty(), 0)
    match infer_hm(fd.body, env, init_state) {
        Err(e) => Some(e)
        Ok(r)  => {
            // エフェクトチェック（v7.7.0 から引き継ぎ）
            bind inferred_effs <- infer_expr_effects(fd.body)
            check_effects_all(fd.effects, inferred_effs)
        }
    }
}
```

---

## Phase E: テスト（checker.fav 末尾に 10 件追加）

```favnir
test "occurs_in same" {
    occurs_in("A", "A")
}

test "occurs_in different" {
    occurs_in("A", "Int") == false
}

test "occurs_in nested" {
    occurs_in("A", "List<A>")
}

test "fresh_var zero" {
    fresh_var(0) == "t0"
}

test "fresh_var ten" {
    fresh_var(10) == "t10"
}

test "unify_deep same" {
    match unify_deep("Int", "Int", subst_empty()) {
        Err(_) => false
        Ok(s)  => List.length(s) == 0
    }
}

test "unify_deep var nested" {
    match unify_deep("List<A>", "List<Int>", subst_empty()) {
        Err(_) => false
        Ok(s)  => match subst_lookup(s, "A") {
            None    => false
            Some(t) => t == "Int"
        }
    }
}

test "unify_deep outer mismatch" {
    match unify_deep("List<A>", "Option<Int>", subst_empty()) {
        Err(e) => String.starts_with(e, "E0005")
        Ok(_)  => false
    }
}

test "infer_hm int" {
    match infer_hm(EInt(42), List.empty(), inf_state_new(subst_empty(), 0)) {
        Err(_) => false
        Ok(r)  => r.ty == "Int"
    }
}

test "infer_hm evar unknown fresh" {
    match infer_hm(EVar("x"), List.empty(), inf_state_new(subst_empty(), 0)) {
        Err(_) => false
        Ok(r)  => r.ty == "t0" && r.counter == 1
    }
}
```

---

## Phase F: driver.rs 統合テスト

```rust
mod checker_v79_tests {
    use super::*;
    use std::path::Path;

    fn checker_src() -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("self").join("checker.fav");
        std::fs::read_to_string(&path).expect("checker.fav")
    }

    fn run_checker_inline(test_main: &str) -> Value {
        let src = format!("{}\n\n{}", checker_src(), test_main);
        let prog = Parser::parse_str(&src, "checker_test.fav").expect("parse");
        let (errors, _) = Checker::check_program(&prog);
        assert!(errors.is_empty(), "type errors: {:?}", errors);
        let ir = compile_program(&prog);
        let artifact = codegen_program(&ir);
        let fn_idx = artifact.fn_idx_by_name("main").expect("main function");
        VM::run(&artifact, fn_idx, vec![]).expect("run")
    }

    #[test]
    fn checker_fav_occurs_in() {
        let result = run_checker_inline(r#"
public fn main() -> Bool {
    occurs_in("A", "List<A>")
}
"#);
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn checker_fav_unify_deep_nested() {
        let result = run_checker_inline(r#"
public fn main() -> String {
    match unify_deep("List<A>", "List<Int>", subst_empty()) {
        Err(_) => "error"
        Ok(s)  => match subst_lookup(s, "A") {
            None    => "none"
            Some(t) => t
        }
    }
}
"#);
        assert_eq!(result, Value::Str("Int".to_string()));
    }

    #[test]
    fn checker_fav_fresh_var() {
        let result = run_checker_inline(r#"
public fn main() -> String {
    fresh_var(0)
}
"#);
        assert_eq!(result, Value::Str("t0".to_string()));
    }
}
```

---

## 実装上の注意点

1. **閉じ括弧カウント（`unify_deep`）**:
   - 6 段の if-else 連鎖 → 末尾に `}` × 5 が必要（最後の `else { expr }` は自己クローズ）
   - さらに `if occurs_in(...)` と `match subst_lookup(...)` の `}` に注意
   - 実装後は `fav check` で必ず確認すること

2. **`bind` の制約**:
   - クロージャ内で `bind x <- ...` は使えない
   - `Result.and_then(..., |r| { ... })` の中では `r.field` アクセスのみ可能
   - 複数ステップが必要な場合はヘルパー関数に切り出す

3. **`infer_hm_lambda_params` 設計**:
   - `env` と `counter` を同時に返す必要があるため `InfLambdaCtx` レコードを使用
   - `env_insert` は `List<KVPair>` を返すので `bind` 可能

4. **`_` パターンの活用**:
   - `infer_hm` の最後の `match` arm に `_ =>` を使い、未実装ノードを `infer_expr` にフォールバック

5. **既存テストへの影響**:
   - `TypeDef` リテラルを含むテストは `type_params: List.empty()` が v7.8.0 で追加済みのため変更不要
   - `check_fn_def` の内部変更はシグネチャが同じなので既存テストは自動的に通る

6. **`unify_deep` と既存 `unify` の共存**:
   - `unify` は v7.8.0 で実装済みのため残す（後方互換）
   - `infer_call` は引き続き `unify_deep` を内部で使わない（引数型の統合は不要）
   - `check_fn_def` と `infer_hm_binop` のみ `unify_deep` を使用

7. **`fresh_var` の型変数 vs 型識別子**:
   - `"t0"`, `"t1"` 等は `is_type_var` では `false`（1文字大文字でない）
   - `apply_subst` が認識するには `subst` に明示的に入れるか、`infer_hm` 内で直接追跡する
   - v7.9.0 では `infer_hm` の fresh_var は `subst` に入れず、`env` に入れる（let/lambda の場合）
