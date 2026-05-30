# Favnir v8.7.0 Tasks

Date: 2026-05-30
Theme: checker.fav の user-defined fn 完全チェック（E0007 未定義関数 + E0008 引数数）

---

## Phase A: `collect_variant_constructors` 追加

- [x] A-1: `register_variant(type_name, v, env)` を `collect_fn_schemes` の直後に追加
  — `v.payload = None` → `env_insert(env, v.name, type_name)`（引数なしバリアント）
  — `v.payload = Some(te)` → スキーム文字列を直接構築して登録
    `"forall||<param_str>|<type_name>"` （`make_fn_scheme_str` は vars 空時に ret のみ返すので使用不可）
- [x] A-2: `register_variants_inner(type_name, vs, env)` を追加
  — List<VariantDef> を再帰的に処理して各 variant を登録
- [x] A-3: `collect_variant_constructors(items, env)` を追加
  — IType(td) かつ `!td.is_record` の場合に `register_variants_inner` を呼ぶ
  — それ以外は env をそのまま渡す
- [x] A-4: `check(prog)` を更新
  — `collect_fn_schemes` の後に `collect_variant_constructors` を追加
  — `check_items(prog.items, full_env)` を使う

---

## Phase B: E0007 — 未定義関数検出

- [x] B-1: `infer_call_user` の `None` ケースを変更
  — `None => Result.ok(inf_result_of("Unknown", state))` を削除
  — `None => Result.err(fmt_err("E0007", String.concat("undefined function: ", fname)))` に変更

---

## Phase C: E0008 — 引数数チェック（ジェネリック関数）

- [x] C-1: `count_scheme_params(s: String) -> Int` を `instantiate_fn_scheme` の直前に追加
  — `if String.length(s) == 0 { 0 } else { List.length(String.split(s, ";")) }`
- [x] C-2: `infer_call_user` の `is_fn_scheme_str(ty)` ブランチにアリティチェックを追加
  — `fn_scheme_params_str` でパラメータ文字列を取得
  — `count_scheme_params` と `List.length(arg_tys)` を比較
  — 不一致 → `Result.err(fmt_err("E0008", ...))`
  — 一致 → 従来通り `instantiate_fn_scheme(ty, arg_tys, state)`

---

## Phase D: テスト追加

- [x] D-1: `undefined_fn_e0007` テストを追加
  — `public fn main() -> Int { foo(1) }` で E0007 が出ること
- [x] D-2: `variant_constructor_ok` テストを追加
  — user-defined sum type のコンストラクター呼び出しが E0007 を出さないこと
- [x] D-3: `wrong_arity_e0008` テストを追加
  — ジェネリック関数 `identity<A>(x: A)` を 2 引数で呼ぶと E0008 が出ること
- [x] D-4: `cargo test checker_fav_wire_tests` — 既存 3 件 + 新規 3 件通ること
- [x] D-5: `cargo test` — 1125 tests passing ✓

---

## Phase E: 最終確認・ドキュメント

- [x] E-1: `fav check self/checker.fav --legacy-check` — checker.fav 自身が Rust チェッカーでエラーなし
- [x] E-2: `cargo build` — コンパイルエラーなし
- [x] E-3: このファイルを完了状態に更新
- [x] E-4: commit

---

## 完了条件

- `foo(1)`（`foo` 未定義）→ E0007 ✓
- user-defined variant コンストラクター `Blue(5)` でエラーなし ✓
- ジェネリック関数の引数数不一致 → E0008 ✓
- 非ジェネリック関数の引数数チェックはスコープ外（v8.8.0 以降）✓
- 既存テスト全件通る ✓

---

## 実装ノート

### make_fn_scheme_str の落とし穴
`make_fn_scheme_str("", param_str, type_name)` は vars_csv が空の場合に
`type_name` だけを返す（`"forall|..."` 形式にならない）。
→ A-1 では直接 `"forall||<param>|<type>"` を組み立てること。

### infer_call_user の変更順序
Phase B と C は同じ関数の変更。B を先に実装してビルド確認後、C を追加する方が安全。

### 影響範囲の確認
`collect_variant_constructors` 追加後、checker.fav 内で定義される大量の sum type
（`Expr`、`Type`、`Item`、`Pat` 等）の variant が全て env に登録される。
これにより checker.fav 内の variant コンストラクター使用も型チェック対象になる。
checker.fav に E0007 エラーが大量発生する場合、それは checker.fav 内の正当な
variant 使用が誤検知されている可能性がある → `infer_call_user` ではなく
`infer_call`（非 HM パス）で variant が呼ばれているケースを確認する。

### 非ジェネリック variant コンストラクター (`Some(te)` なし) の型
`register_variant` で `None` ペイロードの場合は `env_insert(env, v.name, type_name)` で
`type_name`（e.g. `"Color"`）を登録。`is_fn_scheme_str("Color") = false` なので
`infer_call_user` の `else` ブランチに入り `inf_result_of("Color", state)` を返す。
引数なしバリアントが `ECall("", "None1", EArgNil)` として呼ばれた場合に正常動作する。
