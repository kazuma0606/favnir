# Favnir v8.0.0 Tasks

Date: 2026-05-28
Theme: checker.fav Let 多相（型スキーム generalization / instantiation）

---

## Phase A: 型変数拡張とリストユーティリティ（checker.fav）

- [x] A-1: `is_type_var_extended(s: String) -> Bool`
  — 1 文字大文字 A〜Z OR 2 文字で 1 文字目が A〜Z かつ 2 文字目が 0〜9
- [x] A-2: `list_dedup_inner` / `list_dedup` — 重複除去（初出順を維持）
- [x] A-3: `collect_type_vars_from_te(te: TypeExpr) -> List<String>`
  — TeSimple で `is_type_var_extended` ならリストに追加、複合型は再帰
- [x] A-4: `collect_type_vars_from_params_inner` / `collect_type_vars_from_params`
  — `List<Param>` の各 `p.ty` から型変数を収集

---

## Phase B: スキーム文字列ヘルパー（checker.fav）

- [x] B-1: `is_fn_scheme_str(s: String) -> Bool` — `String.starts_with(s, "forall|")`
- [x] B-2: `fn_scheme_section(s: String, idx: Int) -> String`
  — `String.split(s, "|")` を `List.drop(list, idx)` + `List.first` で取得
- [x] B-3: `fn_scheme_vars_str` / `fn_scheme_params_str` / `fn_scheme_ret`
  — `fn_scheme_section` の idx=1/2/3 ラッパー
- [x] B-4: `join_strings_inner(lst, sep, acc, first) -> String`
  — `;` や `,` でリストを結合する汎用関数
- [x] B-5: `join_strings(lst: List<String>, sep: String) -> String`
- [x] B-6: `make_fn_scheme_str(vars_csv: String, params_semi: String, ret: String) -> String`
  — vars_csv が空なら ret のみ返す（monotype）

---

## Phase C: スキーム構築と pre-pass（checker.fav）

- [x] C-1: `params_to_type_str_list_inner` / `params_to_type_str_list`
  — `List<Param>` を `List<String>`（型文字列リスト）に変換
- [x] C-2: `fn_to_scheme_str(fd: FnDef) -> String`
  — 型変数を収集 → 空なら monotype、非空なら `make_fn_scheme_str` でスキーム文字列化
- [x] C-3: `collect_fn_schemes(items: List<Item>, env: List<KVPair>) -> List<KVPair>`
  — `IFn` アイテムをスキャンして `fn_to_scheme_str` の結果を env に追加
- [x] C-4: `check` 関数を更新 — `collect_fn_schemes` の pre-pass を追加

---

## Phase D: スキームの具体化と infer_call_hm（checker.fav）

- [x] D-1: `build_scheme_subst_inner(param_tys, arg_tys, subst) -> Result<List<KVPair>, String>`
  — `unify_deep(pt, at, subst)` をリストで繰り返し適用
- [x] D-2: `apply_scheme_subst(ret_ty: String, vars: List<String>, subst: List<KVPair>) -> String`
  — vars の各型変数を `subst_lookup` → `String.replace` で ret_ty に適用
- [x] D-3: `instantiate_fn_scheme(scheme: String, arg_tys: List<String>, state: InfState) -> Result<InfResult, String>`
  — vars/params/ret を抽出 → `build_scheme_subst_inner` → `apply_scheme_subst` → `inf_result_of`
- [x] D-4: `infer_call_user(fname, args, env, state) -> Result<InfResult, String>`
  — env_lookup → スキームなら `instantiate_fn_scheme`、monotype なら `inf_result_of`
- [x] D-5: `infer_call_hm(ns, fname, args, env, state) -> Result<InfResult, String>`
  — `ns == ""` なら `infer_call_user`、それ以外は `infer_call` にフォールバック
- [x] D-6: `infer_hm` を更新 — `ECall` ケースを `_` フォールバックから昇格、`infer_call_hm` を使用

---

## Phase E: テスト（checker.fav 末尾に 10 件追加）

- [x] E-1: `is_type_var_ext single` — `is_type_var_extended("A") && is_type_var_extended("Z")`
- [x] E-2: `is_type_var_ext two-char` — `is_type_var_extended("T0") && is_type_var_extended("A9")`
- [x] E-3: `is_type_var_ext not` — `is_type_var_extended("Int") == false && is_type_var_extended("AB") == false`
- [x] E-4: `collect_te vars simple` — `List.first(collect_type_vars_from_te(TeSimple("A"))) == Some("A")`
- [x] E-5: `collect_te vars none` — `List.length(collect_type_vars_from_te(TeSimple("Int"))) == 0`
- [x] E-6: `is_fn_scheme_str true` — `is_fn_scheme_str("forall|A|List<A>|Option<A>") == true`
- [x] E-7: `is_fn_scheme_str false` — `is_fn_scheme_str("Option<Int>") == false`
- [x] E-8: `fn_scheme_ret` — `fn_scheme_ret("forall|A|List<A>|Option<A>") == "Option<A>"`
- [x] E-9: `instantiate simple` — `instantiate_fn_scheme("forall|A|A|A", ["Int"], state0)` → `r.ty == "Int"`
- [x] E-10: `instantiate nested` — `instantiate_fn_scheme("forall|A|List<A>|Option<A>", ["List<String>"], state0)` → `r.ty == "Option<String>"`

---

## Phase F: driver.rs 統合テスト（3 件追加）

- [x] F-1: `checker_fav_scheme_str` — `is_fn_scheme_str("forall|A|List<A>|Option<A>")` → `Value::Bool(true)`
- [x] F-2: `checker_fav_instantiate_scheme` — `instantiate_fn_scheme("forall|A|List<A>|Option<A>", ["List<Int>"], s0)` → `r.ty == "Option<Int>"`
- [x] F-3: `checker_fav_infer_hm_generic_call` — `ECall("", "first_elem", ...)` with env `{xs: "List<Int>", first_elem: "forall|A|List<A>|Option<A>"}` → `"Option<Int>"`

---

## Phase G: 最終確認・ドキュメント

- [x] G-1: `fav check fav/self/checker.fav` — no errors
- [x] G-2: `cargo test` — 1113+ tests passing（+13 新規）
- [x] G-3: `site/content/docs/language/self-host-checker.mdx` に v8.0.0 セクション追記
- [x] G-4: このファイルを完了状態に更新
- [x] G-5: commit

---

## 完了条件

- `fav check fav/self/checker.fav` — no errors
- `is_type_var_extended("T0") == true`
- `instantiate_fn_scheme("forall|A|List<A>|Option<A>", ["List<Int>"], state0)` → `"Option<Int>"`
- `infer_hm` が `ECall("", "first_elem", ...)` を `"Option<Int>"` と推論できる
- 統合テスト 13 件追加済み（checker.fav 内 10 + driver.rs 3）
- 既存テスト 1100 件が全件通る（1113+ passing）

---

## 実装ノート（既知の制約）

- `list_dedup_inner` の `List.push` は prepend なので返す順序に注意。
  `join_strings` が正しい順序で `,` / `;` 結合できるか確認すること
- `fn_scheme_section` で `String.split(s, "|")` の返す順序を確認すること
  （Favnir の `String.split` は push=prepend なので逆順の可能性あり）
  → 逆順なら `fn_scheme_section(s, idx)` の idx を調整するか `List.drop` 後に `List.first` でなく末尾取得に変える
- `String.replace(ty, var, val)` は全出現を置換。`"Map<A, A>"` → `"Map<Int, Int>"` は正しい動作
- `infer_hm` に `ECall` ケースを追加する際、既存の `_` フォールバックより前に書くこと
- `check` の pre-pass は `env_empty()` から始める（checker.fav 自体のテストでは
  `test` ブロックが env なしで実行されることに注意）
- Phase C の完了後、`fav check checker.fav` を実行してエラーがないことを確認してから Phase D に進む
