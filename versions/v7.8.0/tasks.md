# Favnir v7.8.0 Tasks

Date: 2026-05-28
Theme: checker.fav ジェネリクス対応（型変数 / 基本単一化 / parameterized builtin 推論）

---

## Phase A: 型変数と置換環境（checker.fav）

- [x] A-1: `Type` sum type に `TyVar(String)` を追加、`type_to_str` に `TyVar(name) => name` を追加
- [x] A-2: `subst_empty` / `subst_insert` / `subst_lookup` — `KVPair` 流用の置換環境ヘルパー
- [x] A-3: `is_type_var(s: String) -> Bool` — 1 文字大文字 A〜Z かを判定
- [x] A-4: `apply_subst(subst: List<KVPair>, ty: String) -> String` — 型変数を置換
- [x] A-5: `type_str_outer(s: String) -> String` — `"List<Int>"` → `"List"`
- [x] A-6: `type_str_inner(s: String) -> String` — `"List<Int>"` → `"Int"`

---

## Phase B: 基本単一化（checker.fav）

- [x] B-1: `unify(t1: String, t2: String, subst: List<KVPair>) -> Result<List<KVPair>, String>`
  - `t1 == t2` → `Ok(subst)`
  - `t1` or `t2` == `"Unknown"` → `Ok(subst)`（後方互換）
  - `is_type_var(t1)` → subst に `t1 → t2` を挿入（競合チェックあり）
  - `is_type_var(t2)` → subst に `t2 → t1` を挿入（競合チェックあり）
  - それ以外 → `Err(fmt_err("E0005", "cannot unify t1 with t2"))`
- [x] B-2: `fmt_err` テーブルに E0005 を追加（spec/docs のエラーコード表に記載）

---

## Phase C: ジェネリクス対応 builtin 推論（checker.fav）

- [x] C-1: `unwrap_ty(r: Result<String, String>) -> String` — Ok → ty, Err → "Unknown"
- [x] C-2: `infer_arg_tys(args: Expr, env: List<KVPair>) -> List<String>` — EArgList を走査して型リスト収集
- [x] C-3: `wrap_in(outer: String, inner: String) -> String` — inner が空なら bare 型名、そうでなければ `"outer<inner>"`
- [x] C-4: `infer_generic_list(fname: String, arg_tys: List<String>) -> String`
  - first / find → `Option<T>`
  - filter / push / drop / take / take_while / drop_while / concat → `List<T>`（T は arg0 の inner 型）
  - singleton → `List<arg0>`（arg0 が要素型）
  - map → bare `"List"`（HM は v7.9.0）
  - その他 → `list_fn(fname)` にフォールバック
- [x] C-5: `infer_generic_opt(fname: String, arg_tys: List<String>) -> String`
  - and_then → bare `"Option"`
  - その他 → `opt_fn(fname)`
- [x] C-6: `infer_generic_res(fname: String, arg_tys: List<String>) -> String`
  - and_then → bare `"Result"`
  - その他 → `res_fn(fname)`
- [x] C-7: `infer_call` を更新 — List / Option / Result に `infer_generic_*` を使用、それ以外は `builtin_ret_ty`

---

## Phase D: ユーザー定義ジェネリクス型（基本）（checker.fav）

- [x] D-1: `TypeDef` に `type_params: List<String>` フィールドを追加
- [x] D-2: `type_param_env_inner` / `type_param_env` — 型パラメータ名 → 型引数のマッピング構築
- [x] D-3: `check_item` の `IType` ハンドラで型パラメータ情報を保持（arity チェック用）

---

## Phase E: テスト（checker.fav 末尾に 10 件追加）

- [x] E-1: `is_type_var upper` — `is_type_var("A") && is_type_var("Z")`
- [x] E-2: `is_type_var not` — `is_type_var("Int") == false && is_type_var("a") == false`
- [x] E-3: `apply_subst resolves` — `subst_insert([], "A", "Int")` → `apply_subst → "Int"`
- [x] E-4: `apply_subst no match` — `apply_subst([], "String") == "String"`
- [x] E-5: `unify same` — `unify("Int", "Int", []) == Ok([])`
- [x] E-6: `unify var left` — `unify("A", "Bool", [])` → Ok, lookup "A" → "Bool"
- [x] E-7: `unify conflict` — `unify("Int", "String", [])` → Err で E0005 を含む
- [x] E-8: `type_str_inner list` — `type_str_inner("List<Int>") == "Int"`
- [x] E-9: `type_str_inner option` — `type_str_inner("Option<String>") == "String"`
- [x] E-10: `generic list first` — `infer_generic_list("first", ["List<Int>"]) == "Option<Int>"`

---

## Phase F: driver.rs 統合テスト（3 件追加）

- [x] F-1: `checker_fav_generic_list_first` — `infer_generic_list("first", ["List<Int>"])` → `Value::Str("Option<Int>")`
- [x] F-2: `checker_fav_unify_var` — `unify("A", "Bool", subst_empty())` → Ok, lookup "A" → `Value::Str("Bool")`
- [x] F-3: `checker_fav_type_str_inner` — `type_str_inner("Option<String>")` → `Value::Str("String")`

---

## Phase G: 最終確認・ドキュメント

- [x] G-1: `fav check fav/self/checker.fav` — no errors
- [x] G-2: `cargo test` — 1107+ tests passing（+13 新規）
- [x] G-3: `site/content/docs/language/self-host-checker.mdx` に v7.8.0 セクション追記
- [x] G-4: このファイルを完了状態に更新
- [x] G-5: commit

---

## 完了条件

- `fav check fav/self/checker.fav` — no errors
- `infer_generic_list("first", ["List<Int>"])` == `"Option<Int>"`
- `unify("A", "Int", subst_empty())` が `Ok` で `A → Int` を含む
- `type_str_inner("List<Int>")` == `"Int"`
- 統合テスト 13 件追加済み（checker.fav 内 10 + driver.rs 3）
- 既存テスト 1094 件が全件通る（1107+ passing）

---

## 実装ノート（既知の制約）

- `bind inside closure 不可` → `infer_arg_tys` で `unwrap_ty` を外部関数として定義
- `else if` 非対応 → `else { if ... }` + 閉じ括弧数 = N-1（N は if-branch 数）
- `String.split(s, "<")` が `["List", "Int>"]` を返すので、inner 取得後に末尾 `">"` を `String.slice(rest, 0, len - 1)` で除去
- `List.push(list, item)` は先頭に追加（prepend）なので `infer_arg_tys` の引数順が逆になる
  → `infer_generic_*` での `List.first(arg_tys)` が最初の引数（第0引数）を返す
- `TypeDef.type_params` 追加後は既存テスト内の `TypeDef { ... }` リテラルに `type_params: List.empty()` を追加すること
- `unify` の競合チェック: `subst_lookup` で既存バインドを確認し、異なる型が来たら E0005 を返す
- `wrap_in("Option", "")` は `"Option"` を返す（inner 空のときは bare 型名）
- `infer_arg_tys` の返す List は逆順（push が prepend のため）。`List.first` で最初の引数が取れるのは push が先頭追加だから
