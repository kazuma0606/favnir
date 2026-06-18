# Favnir v9.7.5 Implementation Plan

Date: 2026-06-02
Theme: 名目型ラッパー完成 — `where` バリデーター + `with` 解析 + E0013

---

## Phase A: compiler.fav — `TkWith` トークン追加

### A-1: `Token` type に `TkWith` を追加

`compiler.fav` の `type Token` (line ~4):

```favnir
| TkWhere       // 既存
| TkWith        // 追加 ← ここ
```

### A-2: `keyword_token` に `"with"` → `TkWith` を追加

```favnir
fn keyword_token(s: String) -> Option<Token> {
    ...
    if s == "where" { Option.some(TkWhere) }
    else {
    if s == "with" { Option.some(TkWith) }   // 追加
    else { Option.none() }
    ...
}
```

### A-3: `token_eq` / `token_to_string` に `TkWith` を追加

```favnir
// token_eq:
TkWith => { match b { TkWith => true _ => false } }

// token_to_string:
TkWith => "with"
```

---

## Phase B: compiler.fav — `WrapperDef` 拡張 + パーサー更新

### B-1: `WrapperDef` に `where_pred: Expr` フィールドを追加

`has_where: Bool` は削除し、`where_pred: Expr` で置き換える。
`ELit(LUnit)` が「`where` なし」を表す。

```favnir
type WrapperDef = {
    name: String
    inner: String
    where_pred: Expr       // ELit(LUnit) = no where clause
    with_impls: List<String>
}
```

### B-2: `default_wrapper_def` ヘルパーを追加（既存コード簡略化用）

```favnir
fn wrapper_def_no_pred(name: String, inner: String) -> WrapperDef {
    WrapperDef {
        name: name
        inner: inner
        where_pred: ELit(LUnit)
        with_impls: List.drop_while(List.singleton(""), |v| true)
    }
}
```

### B-3: `parse_type_or_wrapper_item` を更新

`type Name(Inner)` のパース後に `with` / `where` 節をパースする。

現在（`rest4` = `)` 消費後）:
```favnir
Ok(rest4) => {
    bind empty_impls <- empty_string_list()
    Result.ok(ItemParse {
        item: IWrapper(WrapperDef {
            name: tname
            inner: type_expr_to_str(inner_p.ty)
            has_where: false
            with_impls: empty_impls
        })
        rest: rest4
    })
}
```

新しい実装（`with` → `where` の順でパース、どちらも任意）:

```favnir
Ok(rest4) => {
    // Optional: with Iface1, Iface2
    match parse_with_clause(rest4) {
        Err(e) => Result.err(e)
        Ok(with_p) => {
            // Optional: where |v| pred
            match parse_where_clause(with_p.rest) {
                Err(e) => Result.err(e)
                Ok(where_p) => {
                    Result.ok(ItemParse {
                        item: IWrapper(WrapperDef {
                            name: tname
                            inner: type_expr_to_str(inner_p.ty)
                            where_pred: where_p.expr
                            with_impls: with_p.impls
                        })
                        rest: where_p.rest
                    })
                }
            }
        }
    }
}
```

### B-4: `parse_with_clause` ヘルパー関数を追加

```favnir
type WithClauseParse = {
    impls: List<String>
    rest: List<Token>
}

fn parse_with_idents(toks: List<Token>, acc: List<String>) -> Result<WithClauseParse, String> {
    match peek(toks) {
        Some(TkIdent(iname)) => {
            bind rest1 <- advance(toks)
            match peek(rest1) {
                Some(TkComma) => parse_with_idents(advance(rest1), List.push(acc, iname))
                _ => Result.ok(WithClauseParse { impls: List.push(acc, iname), rest: rest1 })
            }
        }
        _ => Result.err("expected interface name after 'with'")
    }
}

fn parse_with_clause(toks: List<Token>) -> Result<WithClauseParse, String> {
    match peek(toks) {
        Some(TkWith) => {
            bind rest1 <- advance(toks)
            bind empty <- List.drop_while(List.singleton(""), |v| true)
            parse_with_idents(rest1, empty)
        }
        _ => {
            bind empty <- List.drop_while(List.singleton(""), |v| true)
            Result.ok(WithClauseParse { impls: empty, rest: toks })
        }
    }
}
```

### B-5: `parse_where_clause` ヘルパー関数を追加

```favnir
fn parse_where_clause(toks: List<Token>) -> Result<ExprParse, String> {
    match peek(toks) {
        Some(TkWhere) => {
            bind rest1 <- advance(toks)
            // Expect a lambda: |v| pred_body
            match parse_expr(rest1) {
                Err(e) => Result.err(e)
                Ok(pred_p) => Result.ok(pred_p)
            }
        }
        _ => Result.ok(ExprParse { expr: ELit(LUnit), rest: toks })
    }
}
```

---

## Phase C: compiler.fav — `where` バリデーター コード生成

### C-1: `compile_wrapper_validator` 関数を追加

`where_pred` が `ELambda(param, body)` の場合にバリデーター関数を生成する。

```favnir
fn compile_wrapper_validator(wd: WrapperDef, items: List<Item>, acc: List<FnEntry>) -> Result<List<FnEntry>, String> {
    match wd.where_pred {
        ELambda(parts) => {
            bind param <- parts._0
            bind body  <- parts._1
            bind err_msg <- String.concat(wd.name, ": validation failed")
            bind fn_body <- EIf(
                body,
                ECall("Result", "ok",  EArgList(EVar(param), EArgNil)),
                ECall("Result", "err", EArgList(ELit(LStr(err_msg)), EArgNil))
            )
            bind fn_def <- FnDef {
                is_public: false
                name:   wd.name
                params: List.singleton(Param { name: param  ty: TeSimple(wd.inner) })
                ret:    TeResult(TeSimple(wd.name), TeSimple("String"))
                body:   fn_body
            }
            Result.and_then(compile_fn_def(fn_def), |entries|
            compile_items(List.drop(items, 1), List.concat(acc, entries)))
        }
        _ => Result.err(String.concat("where clause must be a lambda: ", wd.name))
    }
}
```

**ポイント:**
- `param` はラムダのパラメータ名（例: `"v"`）をそのまま関数パラメータとして使用
- `body` はラムダ本体で、パラメータ名 `v` への参照を含む → 関数パラメータとして定義されているため自然に解決される
- 生成される関数名は型名と同じ（例: `Percent`）
- VM の `LoadGlobal` は `fn_idx_by_name` を `looks_like_variant_ctor` より先にチェックするため、ユーザー定義関数が優先される

### C-2: `compile_items` の `IWrapper` ハンドラを更新

```favnir
IWrapper(wd) => {
    match wd.where_pred {
        ELit(lit) => {
            match lit {
                LUnit => compile_items(List.drop(items, 1), acc)  // no where clause
                _     => compile_wrapper_validator(wd, items, acc) // has where clause (non-unit lit, unlikely)
            }
        }
        _ => compile_wrapper_validator(wd, items, acc)  // has where clause (lambda or other expr)
    }
}
```

実際には `ELambda` のみを期待するが、`ELit(LUnit)` で「なし」を表すシンプルな判定にする。

### C-3: `pretty_wrapper_def` を更新

`where_pred` / `with_impls` を出力に反映する。

```favnir
fn pretty_wrapper_def(wd: WrapperDef) -> String {
    bind base <- String.concat("type ", String.concat(wd.name, String.concat("(", String.concat(wd.inner, ")"))))
    bind with_part <- if List.length(wd.with_impls) == 0 { "" }
                      else { String.concat(" with ", String.join(wd.with_impls, ", ")) }
    bind where_part <- match wd.where_pred {
        ELit(LUnit) => ""
        pred => String.concat(" where ", pretty_expr(pred, 0))
    }
    String.concat(base, String.concat(with_part, where_part))
}
```

---

## Phase D: checker.fav — E0013 + `has_where` コンストラクタ型

### D-1: `infer_hm` の `EQuestion` ケースに E0013 チェックを追加

現在:
```favnir
EQuestion(inner) => Result.and_then(infer_expr(inner, env), |ity|
    Result.ok("Unknown"))
```

新しい実装:
```favnir
EQuestion(inner) => Result.and_then(infer_expr(inner, env), |ity|
    if String.starts_with(ity, "Result") { Result.ok("Unknown") }
    else { Result.err(String.concat("E0013: ? requires a Result expression, got ", ity)) })
```

### D-2: `collect_variant_constructors` の `IWrapper` ハンドラを更新

現在:
```favnir
IWrapper(wd) => collect_variant_constructors(List.drop(items, 1),
    env_insert(env, wd.name, make_fn_scheme_str("", wd.inner, wd.name)))
```

`has_where: true` の場合、戻り値型を `Result<Name, String>` にする:

```favnir
IWrapper(wd) => {
    bind ret_ty <- if wd.has_where {
        String.concat("Result<", String.concat(wd.name, ", String>"))
    } else { wd.name }
    collect_variant_constructors(List.drop(items, 1),
        env_insert(env, wd.name, make_fn_scheme_str("", wd.inner, ret_ty)))
}
```

---

## Phase E: 統合テスト（`fav/src/driver.rs`）

`v975_tests` モジュールを追加。

### E-1: `where_validator_ok`
```rust
// type Percent(Float) where |v| v >= 0.0 && v <= 100.0
// Percent(50.0) → Ok(50.0) — validates and returns inner value
```

### E-2: `where_validator_err`
```rust
// Percent(150.0) → Err("Percent: validation failed")
```

### E-3: `where_validator_in_fn`
```rust
// validate + use in pipeline: apply_discount(1000.0, 20.0) → Ok(200.0)
// apply_discount(1000.0, 150.0) → Err("Percent: validation failed")
```

### E-4: `with_clause_parses_ok`
```rust
// type UserId(Int) with Serialize
// constructor still works: UserId(42) → 42 at runtime
```

### E-5: `e0013_expr_question_on_option`
```rust
// fn bad() -> Int { Option.some(42)? }
// → compiler.fav error OR checker.fav error: "E0013: ..."
// Use check_source variant to assert error message
```

### E-6: `checker_has_where_constructor_is_result`
```rust
// type Percent(Float) where |v| v > 0.0
// checker recognizes Percent(x) as returning Result<Percent, String>
// (no E0009 mismatch when function returns Result)
```

---

## Phase F: self-check + Bootstrap 検証

```
cargo test checker_fav_wire_self_check  → pass
cargo test bootstrap                    → 23件 pass
cargo test                              → 全件 pass（目標 1205+）
```

---

## Phase G: バージョン更新

- `fav/Cargo.toml` version → `"9.7.5"`
- `fav/self/cli.fav` バージョン文字列 → `"9.7.5"`
- `versions/v9.7.5/tasks.md` 完了チェック
- `memory/MEMORY.md` v9.7.5 完了を記録
- commit

---

## 実装上の注意点

### WrapperDef の `where_pred: Expr` の初期値
- `ELit(LUnit)` を「`where` なし」の sentinel として使用
- `compile_items` で `ELit(LUnit)` かどうかを判定（`match wd.where_pred { ELit(_) => skip  _ => validate }`）
- ただし `ELit(LBool(true))` 等も `ELit` にマッチするため、`LUnit` 専用チェックが必要なら helper fn を追加

### `ELambda` の tuple アクセス
- `ELambda` は 2-tuple: `parts._0 = param: String`, `parts._1 = body: Expr`
- multi-param lambda は `ELambda(p1, ELambda(p2, body))` ネスト
- `where` 述語は single-param lambda を前提 (`|v| expr`)

### `compile_fn_def` の動作
- `fn_def.name` がそのまま `FnEntry.fname` になる
- 戻り型 `TeResult(TeSimple(wd.name), TeSimple("String"))` → `"Result<Name, String>"` として `ret_ty` に保存
- `arity = 1`（single param）

### `has_where` を checker.fav が受け取る方法
- checker.fav は Rust の `lower_wrapper_def` からデータを受け取る
- `lower_wrapper_def` は `!td.invariants.is_empty()` を `has_where` として設定
- Rust パーサーが `where` 節を `invariants` に保存するため、既に正しく機能している

### `TkWith` の位置
- compiler.fav の `Token` 型定義で `TkWhere` の直後に挿入する
- `token_eq` の match は網羅的チェックがないため、追加後に `cargo test` で動作確認

### `where` と `with` の順序
- Rust パーサーの実装（`src/frontend/parser.rs` line 693-713）を見ると:
  `with` → `where` の順でパースされる
- compiler.fav の実装も同じ順序にする（`parse_type_or_wrapper_item` で `with` を先にパース）

### checker.fav の `make_fn_scheme_str` について
- `make_fn_scheme_str("", wd.inner, ret_ty)` で `Inner -> RetTy` のスキームを生成
- `has_where: true` の場合 `ret_ty = "Result<Name, String>"` とする
- `collect_variant_constructors` の既存の `IWrapper` ハンドラを更新するだけでよい
