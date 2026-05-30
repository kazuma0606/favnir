# Favnir v7.8.0 仕様書

Date: 2026-05-28
Theme: checker.fav ジェネリクス対応（型変数 / 基本単一化 / parameterized builtin 推論）

---

## 目的

v7.7.0 の checker.fav は builtin 戻り型を `"List"` / `"Option"` のような bare 型名で返す。
本バージョンで `"List<Int>"` / `"Option<String>"` のような **parameterized 型文字列**を扱えるようにし、
型変数の置換・単一化を実装する。

---

## 現状ギャップ

| 機能 | checker.rs | checker.fav（v7.7.0） |
|------|-----------|----------------------|
| 型変数 (`TyVar`) | 対応 | `TyUnknown` で代替 |
| 型置換 (Subst) | `HashMap<String, Type>` | なし |
| 単一化 (unify) | Robinson unification | なし |
| 型パラメータ付き戻り型 | `List<Int>` 等 | `"List"`（bare）|
| ジェネリクス builtin | `List.map: (List<A>, A→B) → List<B>` | `list_fn("map") = "List"` |

---

## Phase A: 型変数と置換環境

### A-1: `TyVar(String)` を `Type` sum type に追加

```favnir
type Type =
  | ...（既存）...
  | TyVar(String)   // 型変数 e.g. TyVar("A")
```

`type_to_str` に `TyVar(name) => name` を追加。

### A-2: Subst 型と操作

`KVPair` を流用して `List<KVPair>` を置換環境として使う。

```favnir
fn subst_empty() -> List<KVPair> { env_empty() }
fn subst_insert(subst: List<KVPair>, name: String, ty: String) -> List<KVPair> {
    env_insert(subst, name, ty)
}
fn subst_lookup(subst: List<KVPair>, name: String) -> Option<String> {
    env_lookup(subst, name)
}
```

### A-3: `is_type_var(s: String) -> Bool`

型変数の判定（1文字の大文字アルファベット A〜Z）：

```favnir
fn is_type_var(s: String) -> Bool {
    if String.length(s) == 1 {
        bind c <- String.slice(s, 0, 1)
        c == "A" || c == "B" || c == "C" || c == "D" || c == "E" ||
        c == "F" || c == "G" || c == "H" || c == "I" || c == "J" ||
        c == "K" || c == "L" || c == "M" || c == "N" || c == "O" ||
        c == "P" || c == "Q" || c == "R" || c == "S" || c == "T" ||
        c == "U" || c == "V" || c == "W" || c == "X" || c == "Y" || c == "Z"
    } else { false }
}
```

### A-4: `apply_subst(subst: List<KVPair>, ty: String) -> String`

型文字列に置換を適用：

```favnir
fn apply_subst(subst: List<KVPair>, ty: String) -> String {
    if is_type_var(ty) {
        match subst_lookup(subst, ty) {
            Some(resolved) => resolved
            None => ty
        }
    } else { ty }
}
```

### A-5: `type_str_outer(s: String) -> String` / `type_str_inner(s: String) -> String`

```
"List<Int>"     → outer: "List",    inner: "Int"
"Option<String>" → outer: "Option",  inner: "String"
"Result"        → outer: "Result",  inner: ""
```

実装方針：
- `type_str_outer`: `String.contains(s, "<")` が true なら `String.slice` で `<` 前を取得
- `type_str_inner`: `<` と `>` の間を `String.slice` で取得

`String.split` が利用可能なので `"<"` で split して先頭を outer、残りを inner に使う。

---

## Phase B: 基本単一化

### B-1: `unify(t1: String, t2: String, subst: List<KVPair>) -> Result<List<KVPair>, String>`

単純なフラット単一化（nested types は v7.9.0 で対応）：

```
unify("A", "Int",    []) = Ok([A → Int])
unify("Int", "B",    []) = Ok([B → Int])
unify("Int", "Int",  []) = Ok([])
unify("Int", "String", []) = Err("E0005: cannot unify Int with String")
unify("A", "Int",    [A → Bool]) = Err("E0005: conflict: A was Bool, got Int")
unify("Unknown", t,  []) = Ok([])   // Unknown は全型に適合（v7.7.0 との後方互換）
unify(t, "Unknown",  []) = Ok([])
```

### B-2: `unify_result_ok(subst) / unify_result_err(msg)` — Result.ok/err のエイリアス

コード可読性のための薄いラッパー（実装は省略可）。

---

## Phase C: ジェネリクス対応 builtin 推論

### C-1: `infer_arg_tys(args: Expr, env: List<KVPair>) -> List<String>`

`EArgList(head, tail)` を走査して各引数の型文字列を収集。
エラーは `"Unknown"` にフォールバック（`infer_expr` の Result を unwrap）。

```favnir
fn infer_arg_tys(args: Expr, env: List<KVPair>) -> List<String>
```

### C-2: `infer_generic_list(fname: String, arg_tys: List<String>) -> String`

引数型情報を使って List.* の戻り型を精密化：

| 関数 | 条件 | 戻り型 |
|------|------|--------|
| `first` | arg0 = `"List<T>"` | `"Option<T>"` |
| `find` | arg0 = `"List<T>"` | `"Option<T>"` |
| `filter` | arg0 = `"List<T>"` | `"List<T>"` |
| `push` | arg0 = `"List<T>"`、arg1 は任意 | `"List<T>"` |
| `drop` / `take` / `take_while` / `drop_while` | arg0 = `"List<T>"` | `"List<T>"` |
| `concat` | arg0 = `"List<T>"` | `"List<T>"` |
| `singleton` | arg0 = T | `"List<T>"` |
| `map` | arg0 = `"List<T>"`、arg1 型が `Unknown` | `"List<Unknown>"` |
| 型情報なし | always | bare `list_fn(fname)` にフォールバック |

### C-3: `infer_generic_opt(fname: String, arg_tys: List<String>) -> String`

| 関数 | 条件 | 戻り型 |
|------|------|--------|
| `and_then` | arg0 = `"Option<T>"` | `"Option<Unknown>"` |
| 型情報なし | bare `opt_fn(fname)` |

### C-4: `infer_generic_res(fname: String, arg_tys: List<String>) -> String`

| 関数 | 条件 | 戻り型 |
|------|------|--------|
| `and_then` | arg0 = `"Result<T,E>"` | `"Result<Unknown,E>"` |
| 型情報なし | bare `res_fn(fname)` |

### C-5: `infer_call` を更新

```favnir
fn infer_call(ns, fname, args, env) -> Result<String, String> {
    if ns == "" {
        // ローカル関数呼び出し（変更なし）
    } else {
        bind arg_tys <- infer_arg_tys(args, env)
        if ns == "List"   { Result.ok(infer_generic_list(fname, arg_tys)) }
        else { if ns == "Option" { Result.ok(infer_generic_opt(fname, arg_tys)) }
        else { if ns == "Result" { Result.ok(infer_generic_res(fname, arg_tys)) }
        else { Result.ok(builtin_ret_ty(ns, fname)) }
        }}
    }
}
```

---

## Phase D: ユーザー定義ジェネリクス型（基本）

### D-1: `TypeDef` に `type_params: List<String>` 追加

```favnir
type TypeDef = {
    name: String
    is_record: Bool
    type_params: List<String>   // ["A", "B"] 等
    variants: List<VariantDef>
    fields: List<Param>
}
```

### D-2: `type_param_env(type_params, type_args) -> List<KVPair>`

型パラメータ名から型引数への置換マップを構築：

```
type_param_env(["A","B"], ["Int","String"]) = [{A→Int}, {B→String}]
```

### D-3: `check_item` の IType ハンドラで型定義を env に登録

型定義名と型パラメータ数を env に記録（`"TypeName:arity"` 形式）：

```favnir
IType(td) => {
    bind td_info <- String.concat(td.name, String.concat(":", Int.to_string(List.length(td.type_params))))
    Result.ok(td.name)
}
```

※ 完全な型定義 lookup は v7.9.0 で実装。v7.8.0 では型パラメータ数の記録まで。

---

## Phase E: テスト（checker.fav 内 10 件）

| テスト | 確認内容 |
|--------|----------|
| `is_type_var A` | `is_type_var("A") == true` |
| `is_type_var Int` | `is_type_var("Int") == false` |
| `apply_subst resolves var` | `apply_subst([{A→Int}], "A") == "Int"` |
| `apply_subst no var` | `apply_subst([{A→Int}], "String") == "String"` |
| `unify same types` | `unify("Int", "Int", []) == Ok([])` |
| `unify var left` | `unify("A", "Int", []) == Ok([{A→Int}])` |
| `unify conflict` | `unify("Int", "String", [])` は Err で E0005 を含む |
| `type_str_inner list` | `type_str_inner("List<Int>") == "Int"` |
| `generic list first` | `infer_generic_list("first", ["List<Int>"]) == "Option<Int>"` |
| `generic list filter` | `infer_generic_list("filter", ["List<String>", "Fn"]) == "List<String>"` |

## Phase F: driver.rs 統合テスト（3 件）

`checker_v78_tests` モジュール：

| テスト | 確認内容 |
|--------|----------|
| `checker_fav_generic_list_first` | `infer_generic_list("first", ["List<Int>"])` → `"Option<Int>"` |
| `checker_fav_unify_var` | `unify("A", "Bool", subst_empty())` → Ok, lookup "A" → "Bool" |
| `checker_fav_type_str_inner` | `type_str_inner("Option<String>")` → `"String"` |

---

## Phase G: ドキュメント

`site/content/docs/language/self-host-checker.mdx` 更新（v7.8.0 セクション追記）：

- `TyVar` / Subst / `apply_subst` / `unify` の API 説明
- ジェネリクス対応 builtin 一覧
- v7.9.0 への展望（HM 型推論での活用）

---

## 完了条件

- `fav check fav/self/checker.fav` — no errors
- `List.first(xs)` で `xs: List<Int>` のとき戻り型が `"Option<Int>"` になる
- `unify("A", "Int", [])` が `Ok([{A→Int}])` を返す
- `is_type_var("A") == true`, `is_type_var("Int") == false`
- 統合テスト 13 件追加（checker.fav 内 10 + driver.rs 3）
- 既存テスト全通過（1107+ passing）

---

## 実装ノート

### `type_str_inner` の実装

```favnir
fn type_str_inner(s: String) -> String {
    if String.contains(s, "<") {
        bind parts <- String.split(s, "<")
        match List.first(List.drop(parts, 1)) {
            None => ""
            Some(rest) => String.slice(rest, 0, String.length(rest) - 1)
        }
    } else { "" }
}
```

`"List<Int>"` → split by `"<"` → `["List", "Int>"]` → drop 1 → `["Int>"]` → first → `"Int>"` → slice -1 → `"Int"`

### `type_str_outer` の実装

```favnir
fn type_str_outer(s: String) -> String {
    if String.contains(s, "<") {
        match List.first(String.split(s, "<")) {
            Some(outer) => outer
            None => s
        }
    } else { s }
}
```

### `unify` の実装注意

- `List<KVPair>` を subst として使うので、`subst_lookup` で既存バインドと競合チェック
- 今バージョンでは nested 型（`"List<List<Int>>"` の単一化）は対応外
- `"Unknown"` は top の型（任意に適合）として扱う

### `infer_arg_tys` の実装

`infer_expr` が `Result<String, String>` を返すため、Result を unwrap して失敗時は `"Unknown"` で代替：

```favnir
fn infer_arg_tys(args: Expr, env: List<KVPair>) -> List<String> {
    match args {
        EArgNil => List.empty()
        EArgList(h, t) => {
            bind hty <- match infer_expr(h, env) {
                Ok(ty) => ty
                Err(_) => "Unknown"
            }
            List.push(infer_arg_tys(t, env), hty)
        }
        _ => List.empty()
    }
}
```

**注意**: Favnir では `bind x <- match expr { ... }` の中で match-arm が同じ型を返せばバインドできる。ただし `bind inside closure 不可` は依然として制限として存在する。

---

## 新規追加関数一覧

| 関数 | 行数見積 |
|------|---------|
| `subst_empty/insert/lookup` | 9 行 |
| `is_type_var` | 8 行 |
| `apply_subst` | 7 行 |
| `type_str_outer / type_str_inner` | 20 行 |
| `unify` | 20 行 |
| `infer_arg_tys` | 16 行 |
| `infer_generic_list` | 25 行 |
| `infer_generic_opt` | 10 行 |
| `infer_generic_res` | 10 行 |
| `type_param_env` | 15 行 |
| TypeDef フィールド追加 + check_item 更新 | 5 行差分 |
| テスト 10 件 | 50 行 |
| **合計追加** | **~195 行** |
