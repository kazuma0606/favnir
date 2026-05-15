# Favnir v0.4.0 タスク一覧

更新日: 2026-04-29

タスクが完了したら `[ ]` を `[x]` に変える。

---

## Phase 1: 型変数と単一化（Unification）

### Type::Var と Type::Cap

- [x] 1-1: `checker.rs` の `Type` 列挙体に `Var(String)` を追加する
- [x] 1-2: `checker.rs` の `Type` 列挙体に `Cap(String, Vec<Type>)` を追加する
- [x] 1-3: `Type::display()` に `Var` / `Cap` のアームを追加する
  - `Var("T")` → `"T"`
  - `Cap("Ord", [Int])` → `"Ord<Int>"`
- [x] 1-4: `Type::display()` の既存コードが壊れていないことを確認する（ビルドが通る）

### Subst（代入）

- [x] 1-5: `checker.rs` に `Subst` 構造体を追加する
  - フィールド: `map: HashMap<String, Type>`
- [x] 1-6: `Subst::empty()` を実装する
- [x] 1-7: `Subst::singleton(var, ty)` を実装する
- [x] 1-8: `Subst::apply(&self, ty: &Type) -> Type` を実装する
  - `Var(name)` → `map.get(name)` を再帰的に apply
  - `List(t)` → `List(apply(t))`
  - `Arrow(a, b)` → `Arrow(apply(a), apply(b))`
  - `Named(n, args)` → `Named(n, args.iter().map(apply).collect())`
  - その他は素通し
- [x] 1-9: `Subst::compose(self, other: Subst) -> Subst` を実装する
  - `other.map` の各値に `self.apply` してから `self.map` にマージ
- [x] 1-10: `Subst::extend(&mut self, var, ty)` を実装する

### Occurs Check

- [x] 1-11: `occurs(var: &str, ty: &Type) -> bool` を実装する
  - `Var(name)` → `name == var`
  - `List(t)`, `Arrow(a,b)`, `Named(_, args)` → 再帰的にチェック
  - その他 → `false`

### Unification

- [x] 1-12: `unify(t1: &Type, t2: &Type) -> Result<Subst, String>` を実装する
  - `(Var(a), Var(b))` で `a == b` → `Subst::empty()`
  - `(Var(a), t)` / `(t, Var(a))` → occurs check 後 `Subst::singleton`
  - 同一コンクリート型 → `Subst::empty()`
  - `(Option(a), Option(b))` → `unify(a, b)`
  - `(List(a), List(b))` → `unify(a, b)`
  - `(Map(k1,v1), Map(k2,v2))` → 順に unify して compose
  - `(Arrow(a1,b1), Arrow(a2,b2))` → 順に unify して compose
  - `(Named(n1,as1), Named(n2,as2))` 同名・同引数数 → 各引数を順に unify
  - 不一致 → `Err(...)`
- [x] 1-13: `Checker` に `fresh_counter: usize` フィールドを追加する
- [x] 1-14: `Checker` に `subst: Subst` フィールドを追加する
- [x] 1-15: `fresh_var(&mut self) -> Type` メソッドを実装する (`Type::Var("$N")`)
- [x] 1-16: `instantiate(&mut self, type_params: &[String], ty: &Type) -> Type` を実装する

### 単体テスト

- [x] 1-17: `Subst::apply` の単体テストを書く
  - `{T→Int}.apply(Arrow(Var("T"), Var("T")))` → `Arrow(Int, Int)`
- [x] 1-18: `unify` の単体テストを書く
  - `unify(Int, Int)` → ok
  - `unify(Var("T"), Int)` → `{T→Int}`
  - `unify(List(Var("T")), List(Int))` → `{T→Int}`
  - `unify(Var("T"), List(Var("T")))` → Err (occurs check)
  - `unify(Int, String)` → Err

---

## Phase 2: Lexer / Parser の拡張

### Lexer

- [x] 2-1: `TokenKind::Cap` を追加し、`"cap"` キーワードにマップする
- [x] 2-2: `TokenKind::Impl` を追加し、`"impl"` キーワードにマップする
- [x] 2-3: Lexer の単体テストを更新する (`test_keywords` に cap / impl を追加)

### AST — 型パラメータフィールドの追加

- [x] 2-4: `TypeDef` に `type_params: Vec<String>` フィールドを追加する (デフォルト `vec![]`)
- [x] 2-5: `FnDef` に `type_params: Vec<String>` フィールドを追加する (デフォルト `vec![]`)
- [x] 2-6: `TrfDef` に `type_params: Vec<String>` フィールドを追加する (デフォルト `vec![]`)
- [x] 2-7: `TypeDef` / `FnDef` / `TrfDef` を構築している箇所を全て更新する（`type_params: vec![]` を追加）

### AST — CapDef / ImplDef

- [x] 2-8: `CapField` 構造体を追加する (`name: String`, `ty: TypeExpr`, `span: Span`)
- [x] 2-9: `CapDef` 構造体を追加する (`visibility`, `name`, `type_params`, `fields`, `span`)
- [x] 2-10: `ImplDef` 構造体を追加する (`cap_name`, `type_args: Vec<TypeExpr>`, `methods: Vec<FnDef>`, `span`)
- [x] 2-11: `Item` 列挙体に `CapDef(CapDef)` と `ImplDef(ImplDef)` を追加する
- [x] 2-12: `Item::span()` に `CapDef` / `ImplDef` のアームを追加する
- [x] 2-13: `Item::CapDef` / `Item::ImplDef` を参照している match が未網羅でないことを確認する（checker / eval / main に `_ => {}` アームを追加）

### Parser

- [x] 2-14: `parse_type_params(&mut self) -> Result<Vec<String>, ParseError>` を実装する
  - `<` がなければ `Ok(vec![])` を返す
  - `<IDENT ("," IDENT)* >` をパースして `Vec<String>` を返す
- [x] 2-15: `parse_type_def(vis)` で `parse_type_params()` を呼ぶよう更新する
- [x] 2-16: `parse_fn_def(vis)` で関数名の後に `parse_type_params()` を呼ぶよう更新する
- [x] 2-17: `parse_trf_def(vis)` で変換名の後に `parse_type_params()` を呼ぶよう更新する
- [x] 2-18: `parse_cap_def(vis)` を新規実装する
  - `cap IDENT type_params = { (IDENT ":" type_expr)+ }`
- [x] 2-19: `parse_impl_def()` を新規実装する
  - `impl IDENT "<" type_expr ("," type_expr)* ">" "{" fn_def+ "}"`
- [x] 2-20: `parse_item()` に `TokenKind::Cap` → `parse_cap_def`、`TokenKind::Impl` → `parse_impl_def` を追加する
- [x] 2-21: Parser の単体テストを追加する
  - `test_parse_generic_fn` — `fn identity<T>(v: T) -> T { v }` のパース
  - `test_parse_generic_type` — `type Pair<T, U> = { first: T  second: U }` のパース
  - `test_parse_generic_trf` — `trf MapOption<T, U>: Option<T> -> Option<U> = ...` のパース
  - `test_parse_cap_def` — `cap Eq<T> = { equals: T -> T -> Bool }` のパース
  - `test_parse_impl_def` — `impl Eq<Int> { fn equals(...) }` のパース

---

## Phase 3: Checker への統合

### Checker フィールドの追加

- [x] 3-1: `Checker` に `type_params: HashSet<String>` フィールドを追加する
- [x] 3-2: `Checker` に `caps: HashMap<String, CapScope>` フィールドを追加する
- [x] 3-3: `Checker` に `impls: HashMap<(String, String), ImplScope>` フィールドを追加する
- [x] 3-4: `CapScope` 構造体を追加する (`type_params: Vec<String>`, `fields: HashMap<String, TypeExpr>`)
- [x] 3-5: `ImplScope` 構造体を追加する (`methods: HashMap<String, Type>`)
- [x] 3-6: `Checker::new()` に新フィールドの初期値を追加する

### resolve_type_expr の更新

- [x] 3-7: `resolve_type_expr` で `Named(name, [], _)` かつ `type_params.contains(name)` のとき `Type::Var(name)` を返すよう変更する
- [x] 3-8: `resolve_type_expr` で `Named(name, args, _)` の引数が型変数を含む場合も正しく解決されることを確認する

### register_item_signatures の更新

- [x] 3-9: `Item::CapDef(cd)` のアームを追加して `caps` に登録する
- [x] 3-10: `Item::ImplDef(id)` のアームを追加して型を解決し `impls` に登録する
- [x] 3-11: `Item::FnDef(fd)` のシグネチャ登録で `type_params` を考慮した `Type::Fn` を登録する（型変数は `Type::Var` で保持）

### check_fn_def / check_trf_def の更新

- [x] 3-12: `check_fn_def` の先頭で `type_params` スコープをセットし、終了後に元に戻す
- [x] 3-13: `check_trf_def` にも同様の型パラメータスコープ処理を追加する

### check_flw_def の更新（generic trf 対応）

- [ ] 3-14-flw: `register_item_signatures` / `check_flw_def` で、各 step の型を `unify` しながら合成する処理が `Type::Var` を含む場合も正しく動作することを確認する
  - 入力: step trf の型が `Trf(Var("T"), Var("U"), ...)` を含む場合
  - 出力: `flw` の型 = `Arrow(Var("$0"), Var("$1"))` を env に登録する（使用時に単一化）
- [ ] 3-15-flw: `test_flw_generic_compose` — generic trf 2 つを合成した flw の型チェックテスト

### check_apply の更新（型変数の単一化）

- [x] 3-16: `check_apply` で関数型に `Type::Var` が含まれる場合に `instantiate` して unify する処理を追加する
- [x] 3-17: unify に失敗した場合に E018 を報告する
- [x] 3-18: occurs check が失敗した場合に E019 を報告する（現在は E018 としてまとめて報告）

### CapDef / ImplDef のチェック

- [x] 3-19: `check_item` に `Item::CapDef(cd)` のアームを追加する（フィールドの型式を検証）
- [x] 3-20: `check_item` に `Item::ImplDef(id)` のアームを追加する
  - 対応する `CapDef` が存在することを確認 (E020)
  - 各メソッドの型が cap フィールドの型と一致することを確認 (E022)
- [x] 3-21: `FieldAccess` で `cap` インスタンスアクセス (`Int.ord` など) を解決する
  - 左辺が型名で、フィールド名が cap 名の場合 → `Type::Cap(...)` を返す

### 組み込み cap の登録

- [x] 3-22: `register_builtins` で `Eq<Int>`, `Ord<Int>`, `Show<Int>` を `impls` に登録する
- [x] 3-23: `register_builtins` で `Eq<Float>`, `Ord<Float>`, `Show<Float>` を登録する
- [x] 3-24: `register_builtins` で `Eq<String>`, `Ord<String>`, `Show<String>` を登録する
- [x] 3-25: `register_builtins` で `Eq<Bool>`, `Show<Bool>` を登録する

### エラーコードの追加

- [ ] 3-26: E017 — 未解決の型変数（fresh var が代入なしに残った場合）
- [x] 3-27: E018 — 型単一化の失敗
- [x] 3-28: E019 — occurs check の失敗（無限型、現在は E018 で報告）
- [x] 3-29: E020 — 未定義の cap
- [x] 3-30: E021 — cap の実装 (impl) が存在しない型へのアクセス
- [x] 3-31: E022 — impl のメソッドが cap 定義と合わない
- [x] 3-32: E023 — 型パラメータの個数が合わない

### Checker の単体テスト

- [x] 3-33: `test_generic_identity` — `fn identity<T>(v: T) -> T` が型チェックを通る
- [x] 3-34: `test_generic_pair` — `type Pair<T, U>` を構築して型チェックが通る
- [x] 3-35: `test_cap_def` — `cap Eq<T>` の定義が通る
- [x] 3-36: `test_impl_def` — `impl Eq<Int>` が型チェックを通る
- [x] 3-37: `test_impl_method_mismatch` — impl のメソッド名が cap にない → E022
- [x] 3-38: `test_unify_fail` — 型不一致の関数呼び出しで E018 が出る
- [x] 3-39: `test_occurs_check` — 無限型で E019 が出る（現在は E018 として報告）

---

## Phase 4: 評価器の変更

- [x] 4-1: `IMPL_REGISTRY` スレッドローカルを追加する（`HashMap<(String, String), Value>`）
  - キー: `("eq", "Int")` など（cap名は小文字で統一）
  - 値: `Value::Record` (メソッド名 → Builtin or Closure)
- [x] 4-2: `register_items` に `Item::CapDef` / `Item::ImplDef` のアームを追加する
  - `CapDef` → 何もしない（型定義のみ）
  - `ImplDef` → メソッドを評価して `Value::Record` を作り `impl_registry` に登録する
- [x] 4-3: `FieldAccess` の評価で `impl_registry` を参照する処理を追加する
  - `Value::Namespace("type:X")` のフィールドアクセスで cap インスタンスを返す
- [x] 4-4: 組み込み cap インスタンスを `impl_registry` に登録する（Interpreter 初期化時）
  - `("eq", "Int")` / `("ord", "Int")` / `("show", "Int")` など
- [x] 4-5: cap メソッド呼び出し (`cap_val.compare(a, b)`) が既存の `FieldAccess` + `Apply` で動くことを確認する
- [x] 4-6: 評価器のテストを追加する
  - `test_eval_identity_generic` — `identity(42)` が `42` を返す
  - `test_eval_cap_eq_int` — `Int.eq.equals(1, 1)` が `true` を返す
  - `test_eval_cap_ord_int_compare` — `Int.ord.compare(1, 2)` が負の数を返す
  - `test_eval_cap_show_int` — `Int.show.show(42)` が `"42"` を返す
  - `test_eval_user_impl` — ユーザー定義 impl が動く

---

## Phase 5: Option / Result の段階的統一

- [x] 5-1: `unify` 関数で `Type::Option(t)` と `Type::Named("Option", [t])` を同値扱いにする
- [x] 5-2: `Type::Named("Option", [t]).display()` が `"{}?"` 形式で表示されるよう `display` を更新する
- [x] 5-3: `Type::Named("Result", [t, e]).display()` が `"Result<{}, {}>"` 形式で表示されることを確認する
- [x] 5-4: 既存テストが壊れていないことを確認する（`T?` の表示が変わらない）

---

## Phase 6: サンプルと動作確認

- [x] 6-1: `examples/generics.fav` を作成する
  - `identity<T>`, `Pair<T, U>`, `make_pair<A, B>` を含む
  - `fav run examples/generics.fav` が正しく動く
- [x] 6-2: `examples/cap_sort.fav` を作成する
  - `min_by`, `max_by` (Int.ord 使用), `int_eq` (Int.eq 使用) を定義
  - `fav run examples/cap_sort.fav` が正しく動く
- [x] 6-3: `examples/cap_user.fav` を作成する
  - `type User` を定義
  - `impl Eq<User>` を書く
  - `User.eq.equals` を使う
- [x] 6-4: `fav check examples/generics.fav` が型エラーなく通ることを確認する
- [x] 6-5: `fav check examples/cap_sort.fav` が型エラーなく通ることを確認する
- [x] 6-6: `fav check examples/cap_user.fav` が型エラーなく通ることを確認する
- [x] 6-7: `fav explain examples/generics.fav` が型パラメータ付きシグネチャを正しく表示することを確認する

---

## ドキュメント

- [x] 7-1: `README.md` に v0.4.0 の使い方 (generic fn/type, cap, impl) を追記する
- [x] 7-2: `versions/roadmap.md` の v0.4.0 完了日を記録する
- [x] 7-3: `examples/generics.fav` と `examples/cap_sort.fav` にコメントを追加する
