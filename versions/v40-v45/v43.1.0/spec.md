# v43.1.0 仕様書 — 戻り値型推論（Return type omission）

**フェーズ**: Language Expressiveness（v43.x スプリント）
**前バージョン**: v43.0.0（Real-Time Power 宣言、2900 tests）
**目標テスト数**: 2903（+3）

> **注**: ロードマップは「推定 2895 tests」と記載しているが、v43.0.0 完了時点の実績が 2900 のため、2900 + 3 = 2903 を実装目標とする。

---

## 概要

`fn f(x: Int) { x * 2 }` のように、`-> RetType` を省略した関数定義を受理できるようにする。
省略時は末尾式の型を戻り値型として確定（Return type omission）。

```favnir
// 従来（必須）
fn double(x: Int) -> Int { x * 2 }

// v43.1.0 以降（省略可）
fn double(x: Int) { x * 2 }   // -> Int をブロック末尾式から推論
```

---

## 現状分析

| 箇所 | 状態 |
|---|---|
| `ast.rs` `FnDef.return_ty` | **`Option<TypeExpr>`** — 既に省略可能 ✅ |
| `parser.rs` `parse_fn_def()` | line 1981-1986: `return_ty.is_none()` + block body の組み合わせを**エラー**にしている |
| `checker.rs` `check_fn_def()` | line 3109-3122: `return_ty: None` 時は body_ty を使用 — **既に対応済み** ✅ |
| `compiler.fav` `parse_fn_def_after_params()` | `TkArrow` を **必須**として `expect_tok` している |
| `checker.fav` `check_body_ty()` | `type_expr_to_str(ret)` を使い宣言型と比較 — `ret == TeSimple("")` 時を**未対応** |

**主な変更点は parser.rs の制限除去のみ。** Rust の checker.rs はすでに動作する。
compiler.fav / checker.fav は self-hosted パスの基礎対応として修正する。

---

## スコープ

### v43.1.0 に含む

1. **`fav/src/frontend/parser.rs`**
   - `parse_fn_def()` line 1981-1986 の制限ブロック（`return_ty.is_none()` 時のエラー）を除去
   - `fn double(x: Int) { x * 2 }` が `return_ty: None` でパースされるようにする

2. **`fav/self/compiler.fav`**
   - `parse_fn_def_after_params()`: `)` 後のトークンが `TkArrow` でなければ `TeSimple("")` で `parse_fn_def_after_ret` を呼ぶ

3. **`fav/self/checker.fav`**
   - `check_body_ty()`: `type_expr_to_str(ret) == ""` の場合は型チェックをスキップし `Result.ok(fname)` を返す

4. **`fav/src/driver.rs`** — テスト 3 件追加

5. **`fav/Cargo.toml`** — version: `43.0.0` → `43.1.0`

6. **`CHANGELOG.md`** — `[v43.1.0]` エントリ追加

### スコープ外

- E0410 / E0411 エラーコード追加 — v43.2.0
- `fav check --show-types` での推論型表示 — v43.2.0
- 再帰関数での戻り値型省略対応（E0274 は維持）
- compiler.fav の `collect_fn_scheme_str` での推論型補完 — v43.2.0 以降
  ※ v43.1.0 では self-hosted パスで `-> Type` を省略した関数のスキーム文字列が `"(args) -> "` となる既知制限あり

---

## 実装詳細

### 1. parser.rs 変更（主体）

**削除する 5 行**（line 1981-1985 の `if` ブロックのみ。前後の `} else {` / `self.parse_block()?` / `};` は変更しない）:
```rust
            if return_ty.is_none() {
                return Err(ParseError::new(
                    "function return type can only be omitted with `= expr` syntax",
                    self.peek_span().clone(),
                ));
            }
```

**変更前**（full context、削除行を `// ← DELETE` でマーク）:
```rust
        } else {
            if return_ty.is_none() {                                                // ← DELETE
                return Err(ParseError::new(                                         // ← DELETE
                    "function return type can only be omitted with `= expr` syntax",// ← DELETE
                    self.peek_span().clone(),                                        // ← DELETE
                ));                                                                  // ← DELETE
            }                                                                        // ← DELETE
            self.parse_block()?
        };
```

**変更後**:
```rust
        } else {
            self.parse_block()?
        };
```

`return_ty.is_none()` の `if` ブロック 6 行を削除するだけ（`} else {` と `self.parse_block()?` と `};` は変更しない）。checker.rs はすでに `return_ty: None` → body_ty 推論に対応している。

### 2. compiler.fav 変更

> **確認済み（[HIGH-2]対応）**: `Some(TkArrow)` のようなネストパターンマッチは compiler.fav で多用されている（line 1247, 1261, 1290-1295 等）。本変更で使用するパターンは有効。

> **記法注意（[MED-3]対応）**: compiler.fav のレコードリテラルはフィールド間をスペースで区切る（カンマ不要）。`TypeExprParse { ty: TeSimple("")  rest: rest4 }` の ` ` はプロジェクト既定記法。

**変更前** (`parse_fn_def_after_params` line 2029-2044):
```favnir
fn parse_fn_def_after_params(is_pub: Bool, fname: String, params_p: ParamsParse) -> Result<FnDefParse, String> {
    match expect_tok(params_p.rest, TkRParen) {
        Err(e) => Result.err(e)
        Ok(rest4) => {
            match expect_tok(rest4, TkArrow) {
                Err(e) => Result.err(e)
                Ok(rest5) => {
                    match parse_type_expr(rest5) {
                        Err(e) => Result.err(e)
                        Ok(ret_p) => parse_fn_def_after_ret(is_pub, fname, params_p, ret_p)
                    }
                }
            }
        }
    }
}
```

**変更後**:
```favnir
fn parse_fn_def_after_params(is_pub: Bool, fname: String, params_p: ParamsParse) -> Result<FnDefParse, String> {
    match expect_tok(params_p.rest, TkRParen) {
        Err(e) => Result.err(e)
        Ok(rest4) => {
            // v43.1.0: `->` optional — if absent, use TeSimple("") as placeholder (return type inferred from body)
            match List.first(rest4) {
                Some(TkArrow) => {
                    match expect_tok(rest4, TkArrow) {
                        Err(e) => Result.err(e)
                        Ok(rest5) => {
                            match parse_type_expr(rest5) {
                                Err(e) => Result.err(e)
                                Ok(ret_p) => parse_fn_def_after_ret(is_pub, fname, params_p, ret_p)
                            }
                        }
                    }
                }
                _ => parse_fn_def_after_ret(is_pub, fname, params_p, TypeExprParse { ty: TeSimple("")  rest: rest4 })
            }
        }
    }
}
```

### 3. checker.fav 変更

> **確認済み（[MED-4]対応）**: checker.fav の `type_expr_to_str` が `""` を返すのは `TeSimple("")` のみ（`TeList(inner)` は `"List<...>"` 、他のバリアントも非空文字列を返す）。`type_expr_to_str(ret) == ""` の判定は安全。

**変更前** (`check_body_ty` line 1925-1931):
```favnir
fn check_body_ty(fname: String, ret: TypeExpr, r: InfResult) -> Result<String, String> {
    if types_compatible(apply_subst(r.subst, r.ty), type_expr_to_str(ret)) {
        Result.ok(fname)
    } else {
        Result.err(fmt_err("E0009", ...))
    }
}
```

**変更後**:
```favnir
fn check_body_ty(fname: String, ret: TypeExpr, r: InfResult) -> Result<String, String> {
    // v43.1.0: ret == TeSimple("") means return type was omitted — infer from body (always OK)
    if type_expr_to_str(ret) == "" {
        Result.ok(fname)
    } else {
        if types_compatible(apply_subst(r.subst, r.ty), type_expr_to_str(ret)) {
            Result.ok(fname)
        } else {
            Result.err(fmt_err("E0009", String.concat(fname, String.concat(": declared return ", String.concat(type_expr_to_str(ret), String.concat(" but body infers ", apply_subst(r.subst, r.ty)))))))
        }
    }
}
```

---

## テスト設計（3 件）

```rust
// -- v43100_tests (v43.1.0) -- 戻り値型推論（Return type omission）--
#[cfg(test)]
mod v43100_tests {
    #[test]
    fn cargo_toml_version_is_43_1_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("43.1.0"), "Cargo.toml must contain version 43.1.0");
    }

    #[test]
    fn return_type_omission_block_parseable() {
        use crate::frontend::parser::Parser;
        let src = "fn double(x: Int) { x * 2 }";
        let result = Parser::parse_str(src, "test.fav");
        assert!(result.is_ok(), "fn without -> RetType should parse: {:?}", result.err());
    }

    #[test]
    fn return_type_omission_return_ty_is_none() {
        use crate::frontend::parser::Parser;
        use crate::ast::Item;
        let src = "fn double(x: Int) { x * 2 }";
        let prog = Parser::parse_str(src, "test.fav").expect("parse ok");
        let Item::FnDef(ref fd) = prog.items[0] else {
            panic!("expected FnDef");
        };
        assert!(fd.return_ty.is_none(), "return_ty should be None when -> is omitted");
    }
}
```

---

## 既存コードへの影響

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/frontend/parser.rs` | 変更 | `parse_fn_def()` line 1981-1986 の制限ブロック（5 行）を削除 |
| `fav/self/compiler.fav` | 変更 | `parse_fn_def_after_params()` を `->` オプション対応に変更 |
| `fav/self/checker.fav` | 変更 | `check_body_ty()` に `ret == ""` 時の早期 OK パスを追加 |
| `fav/src/driver.rs` | 変更 | `v43100_tests` 3 件追加（`v43000_tests` の直前） |
| `fav/Cargo.toml` | 変更 | version: `43.0.0` → `43.1.0` |
| `CHANGELOG.md` | 変更 | `[v43.1.0]` エントリ追加 |
| `versions/current.md` | 変更 | 最新安定版 v43.1.0 に更新 |
| `versions/roadmap/roadmap-v43.1-v44.0.md` | 変更 | v43.1.0 を完了済みにマーク、推定テスト数を 2895 → 実績 2903 に修正 |
| `versions/v40-v45/v43.1.0/tasks.md` | 変更 | COMPLETE ステータスに更新 |

---

## 完了条件

- `cargo test` 全通過（2903 tests passed, 0 failed）
- `v43100_tests::cargo_toml_version_is_43_1_0` pass
- `v43100_tests::return_type_omission_block_parseable` pass
- `v43100_tests::return_type_omission_return_ty_is_none` pass
- `fn double(x: Int) { x * 2 }` が `fav check` でエラーなしに処理される（Rust checker）
- `fn f(x: Int) = x * 2` の `= expr` 構文が引き続き動作する（既存 2900 テストで回帰カバー済み）
- 再帰関数 `fn fact(n: Int) { ... }` で E0274 が正しく発火する（既存動作の維持）

---

## 非スコープ

- `fav check --show-types` での推論型表示 — v43.2.0
- E0410 / E0411 エラーコード — v43.2.0
- self-hosted パスの `collect_fn_scheme_str` での推論型補完 — v43.2.0
