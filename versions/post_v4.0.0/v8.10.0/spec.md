# Favnir v8.10.0 Spec

Date: 2026-05-30
Theme: checker.fav — 関数戻り型チェック（E0009）

---

## 背景

v8.9.0 まで、`check_fn_def` は `infer_hm(fd.body, ...)` で本体の型を推論するが、
その結果を関数宣言の戻り型（`fd.ret`）と照合していなかった。
つまり `fn bad() -> Int { "hello" }` のような明らかな型ミスマッチも検出されなかった。

v8.10.0 では推論型と宣言型を比較し、不一致なら **E0009** を返す。

---

## 設計

### 比較の方針

完全な型一致チェックには Unification が必要だが、現状の推論精度を考えると
「寛容な互換チェック（lenient compatibility check）」が適切:

| ケース | 判定 | 理由 |
|---|---|---|
| `inferred == declared` | 互換 | 完全一致 |
| `inferred == "Unknown"` | 互換 | 推論不可 — False Positive 回避 |
| `is_type_var_extended(inferred)` | 互換 | 型変数 / fresh var — 確定不可 |
| `outer_type(inferred) == outer_type(declared)` | 互換 | ベア型 vs パラメータ化型 |
| それ以外 | 非互換 → E0009 | 異なる base type |

`outer_type` は `"<"` で分割した先頭部分:
- `outer_type("List<KVPair>") = "List"`
- `outer_type("List") = "List"` → 「List」vs「List<KVPair>」は互換 ✓
- `outer_type("Int") = "Int"`, `outer_type("String") = "String"` → 不一致 → E0009 ✓

---

### 変更 1: `outer_type` ヘルパー追加

```fav
fn outer_type(s: String) -> String {
    match List.first(String.split(s, "<")) {
        None       => s
        Some(outer) => outer
    }
}
```

---

### 変更 2: `types_compatible` ヘルパー追加

```fav
fn types_compatible(inferred: String, declared: String) -> Bool {
    if inferred == declared { true }
    else if inferred == "Unknown" { true }
    else if is_type_var_extended(inferred) { true }
    else { outer_type(inferred) == outer_type(declared) }
}
```

`is_type_var_extended` は既存関数（"A"〜"Z" の単文字 + "A0"〜"Z9" 形式を認識）。

---

### 変更 3: `check_fn_def` に戻り型照合を追加

```fav
// Before:
None => {
    bind init_state <- inf_state_new(subst_empty(), 0)
    Result.and_then(infer_hm(fd.body, param_env, init_state), |r|
    Result.ok(fd.name))
}

// After:
None => {
    bind init_state <- inf_state_new(subst_empty(), 0)
    Result.and_then(infer_hm(fd.body, param_env, init_state), |r|
        bind inferred <- apply_subst(r.subst, r.ty)
        bind declared <- type_expr_to_str(fd.ret)
        if types_compatible(inferred, declared) {
            Result.ok(fd.name)
        } else {
            Result.err(fmt_err("E0009",
                String.concat(fd.name,
                String.concat(": declared return ",
                String.concat(declared,
                String.concat(" but body infers ", inferred))))))
        })
}
```

---

## `type_expr_to_str` の挙動（重要）

checker.fav 内での `type_expr_to_str` はベア型を返すケースがある:

| TypeExpr | `type_expr_to_str` 結果 |
|---|---|
| `TeSimple("Int")` | `"Int"` |
| `TeList(inner)` | `"List<inner>"` |
| `TeOption(inner)` | `"Option<inner>"` |
| `TeResult(a, b)` | `"Result"` （ベア） |
| `TeMap(k, v)` | `"Map"` （ベア） |
| `TeFn(a, b)` | `"Fn"` （ベア） |

`fn_to_scheme_str` がスキームを構築する際にも `type_expr_to_str` を使うため、
`fn_scheme_ret` が返す型文字列も同じ形式になる。

**例**: `fn check_fn_def(fd: FnDef, env: List<KVPair>) -> Result<String, String>` のスキームは
`"forall||FnDef;List<KVPair>|Result"` → `fn_scheme_ret` = `"Result"`。

よって `Result<...>` を返す関数の戻り型チェック:
- 宣言: `type_expr_to_str(TeResult(...)) = "Result"`
- 推論: `fn_scheme_ret(scheme) = "Result"`（最終呼び出しが Result 型の場合）
- `"Result" == "Result"` → OK ✓

---

## self-check 安全性

checker.fav の関数で潜在的な問題ケースとその対処:

### `List<KVPair>` を返す関数

| 関数 | body 末尾の inferred 型 | 宣言型 | 判定 |
|---|---|---|---|
| `env_empty` | `List.take_while(...)` → "List<KVPair>" | "List<KVPair>" | exact ✓ |
| `env_insert` | `List.push(env, ...)` → "List<KVPair>" | "List<KVPair>" | exact ✓ |
| `subst_empty` | `env_empty()` → "List<KVPair>" | "List<KVPair>" | exact ✓ |
| `collect_fn_schemes` | recursive call → "List<KVPair>" | "List<KVPair>" | exact ✓ |

### `Result<...>` を返す関数

全て `type_expr_to_str(TeResult(...)) = "Result"` で統一されるため、
inferred も `fn_scheme_ret` 経由で "Result" → `"Result" == "Result"` → OK ✓

### `List.empty()` を末尾に持つ関数

`list_fn("empty") = "List"` → ベア "List" が inferred になるケース。
ただし checker.fav 内でこのパターンは末尾式には使われておらず、
`env_empty` は `List.take_while(...)` で "List<KVPair>" を返す。
万一 "List" が inferred になっても `outer_type("List") == outer_type("List<KVPair>")` で互換判定 ✓

### `"Unknown"` になる関数

EMatch → `infer_expr` → `infer_arms` → 型推論不可能なケースは "Unknown" を返す。
`types_compatible("Unknown", declared) = true` → スキップ ✓

---

## 動作確認

```
// E0009 を出すケース
fn bad() -> Int { "hello" }   → E0009: bad: declared return Int but body infers String
fn bad() -> String { 42 }     → E0009: bad: declared return String but body infers Int

// E0009 を出さないケース
fn get() -> Int { 42 }                          → OK
fn double(x: Int) -> Int { x + x }             → OK (推論 "Int" == 宣言 "Int")
fn wrap(x: A) -> A { x }                        → OK (推論 "A" == 宣言 "A")
fn ids(xs: List<A>) -> List<A> { xs }          → OK
public fn main() -> Result<Int, String> { ... } → OK ("Result" == "Result")
```

---

## エラーコード

- **E0009**: 宣言戻り型と推論型の不一致（`check_fn_def` パス）
- 既存: E0001（未定義変数）/ E0007（未定義関数）/ E0008（引数数不一致）は独立して機能
