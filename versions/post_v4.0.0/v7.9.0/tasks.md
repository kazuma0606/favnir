# Favnir v7.9.0 Tasks

Date: 2026-05-28
Theme: checker.fav HM 型推論（完全版）— `infer_hm` + `unify_deep` + `occurs_in`

---

## Phase A: occurs_in と unify_deep（checker.fav）

- [x] A-1: `occurs_in(var: String, ty: String) -> Bool` — 型変数が型文字列中に出現するか確認
- [x] A-2: `unify_deep(t1: String, t2: String, subst: List<KVPair>) -> Result<List<KVPair>, String>`
  - flat 型変数（A〜Z 1文字）: occurs check（E0006）+ subst 挿入
  - 両方がパラメータ化型: outer 比較 → 不一致 E0005、一致 → inner を再帰的に `unify_deep`
  - それ以外: `Err(E0005: cannot unify t1 with t2)`
- [x] A-3: `fmt_err` テーブルに E0006 を追加（spec/docs のエラーコード表に記載）

---

## Phase B: InfState / InfResult と補助関数（checker.fav）

- [x] B-1: `type InfState = { subst: List<KVPair>, counter: Int }` を型定義セクションに追加
- [x] B-2: `type InfResult = { ty: String, subst: List<KVPair>, counter: Int }` を型定義セクションに追加
- [x] B-3: `fresh_var(counter: Int) -> String` — `"t"` + `Int.to_string(counter)`
- [x] B-4: `inf_state_new(subst: List<KVPair>, counter: Int) -> InfState`
- [x] B-5: `inf_result_of(ty: String, state: InfState) -> InfResult`
- [x] B-6: `inf_state_of(r: InfResult) -> InfState`
- [x] B-7: `type InfLambdaCtx = { env: List<KVPair>, counter: Int }` — lambda param 処理用

---

## Phase C: infer_hm ヘルパー群（checker.fav）

- [x] C-1: `infer_hm_binop(op, l, r, env, state) -> Result<InfResult, String>`
  — `Result.and_then` チェーンで l → r を順推論 → `unify_deep` で型を統合
- [x] C-2: `infer_hm_let(name, val, body, env, state) -> Result<InfResult, String>`
  — val を推論 → env に `(name, ty)` 追加 → body を推論
- [x] C-3: `infer_hm_if(cond, then_, else_, env, state) -> Result<InfResult, String>`
  — cond → then → else を順推論 → then/else 型を `unify_deep`
- [x] C-4: `infer_hm_add_params(params: List<Param>, env, counter) -> InfLambdaCtx`
  — 各 param に `fresh_var` を割り当てて env に追加（再帰）
- [x] C-5: `infer_hm(expr: Expr, env: List<KVPair>, state: InfState) -> Result<InfResult, String>`
  - `EInt/EFloat/EBool/EStr/EUnit` → 対応する型文字列を直接返す
  - `EVar(name)` → env_lookup → Some は型返却、None は fresh_var 生成
  - `EBinop` → `infer_hm_binop`
  - `ELet` → `infer_hm_let`
  - `EIf` → `infer_hm_if`
  - `ELambda` → `infer_hm_add_params` → body 推論 → `"Fn"` 返却
  - `_` → `infer_expr` にフォールバック

---

## Phase D: check_fn_def 更新（checker.fav）

- [x] D-1: `check_fn_def` を `infer_expr` → `infer_hm` に差し替え
  — `inf_state_new(subst_empty(), 0)` で初期状態を生成し `infer_hm(fd.body, env, init_state)` を呼ぶ

---

## Phase E: テスト（checker.fav 末尾に 10 件追加）

- [x] E-1: `occurs_in same` — `occurs_in("A", "A") == true`
- [x] E-2: `occurs_in different` — `occurs_in("A", "Int") == false`
- [x] E-3: `occurs_in nested` — `occurs_in("A", "List<A>") == true`
- [x] E-4: `fresh_var zero` — `fresh_var(0) == "t0"`
- [x] E-5: `fresh_var ten` — `fresh_var(10) == "t10"`
- [x] E-6: `unify_deep same` — `unify_deep("Int", "Int", []) == Ok([])`
- [x] E-7: `unify_deep var nested` — `unify_deep("List<A>", "List<Int>", [])` → Ok, `subst_lookup("A") == "Int"`
- [x] E-8: `unify_deep outer mismatch` — `unify_deep("List<A>", "Option<Int>", [])` → Err E0005
- [x] E-9: `infer_hm int` — `infer_hm(EInt(42), [], state0)` → Ok, `r.ty == "Int"`
- [x] E-10: `infer_hm evar unknown fresh` — `infer_hm(EVar("x"), [], state0)` → Ok, `r.ty == "t0" && r.counter == 1`

---

## Phase F: driver.rs 統合テスト（3 件追加）

- [x] F-1: `checker_fav_occurs_in` — `occurs_in("A", "List<A>")` → `Value::Bool(true)`
- [x] F-2: `checker_fav_unify_deep_nested` — `unify_deep("List<A>", "List<Int>", subst_empty())` → Ok, lookup `"A"` → `Value::Str("Int")`
- [x] F-3: `checker_fav_fresh_var` — `fresh_var(0)` → `Value::Str("t0")`

---

## Phase G: 最終確認・ドキュメント

- [x] G-1: `fav check fav/self/checker.fav` — no errors
- [x] G-2: `cargo test` — 1120+ tests passing（+13 新規）
- [x] G-3: `site/content/docs/language/self-host-checker.mdx` に v7.9.0 セクション追記
- [x] G-4: このファイルを完了状態に更新
- [x] G-5: commit

---

## 完了条件

- `fav check fav/self/checker.fav` — no errors
- `occurs_in("A", "List<A>")` == `true`
- `unify_deep("List<A>", "List<Int>", subst_empty())` が `Ok` で `A → Int` を含む
- `fresh_var(0)` == `"t0"`
- `infer_hm(EInt(42), [], state)` → `InfResult { ty: "Int", ... }`
- 統合テスト 13 件追加済み（checker.fav 内 10 + driver.rs 3）
- 既存テスト 1107 件が全件通る（1120+ passing）

---

## 実装ノート（既知の制約）

- `unify_deep` の閉じ括弧: 6 if-branch + 内部ネスト → `fav check` 後に `expected item, got RBrace` が出たら 1 つ減らす
- `bind inside closure 不可` → `Result.and_then` チェーン内は `r.field` アクセスのみ、複雑なロジックは外部ヘルパーに切り出す
- `InfLambdaCtx` は lambda param の env/counter を同時に返すためのワークアラウンド
- `fresh_var` が生成する `"t0"` 等は `is_type_var` では `false`（1文字大文字でない）→ `subst` ではなく `env` で追跡
- `check_fn_def` の差し替え後、エフェクトチェック（`check_effects_all`）は引き続き `infer_expr_effects` を使用（`infer_hm` はエフェクト追跡しない）
- `EMatch` は `infer_expr` フォールバックで処理（match exhaustiveness チェックは v7.7.0 実装を使用）
