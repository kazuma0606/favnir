# v43.1.0 実装計画 — 戻り値型推論（Return type omission）

## 目標

`fn f(x: Int) { x * 2 }` のように `-> RetType` を省略した関数定義を受理する。
主な変更は parser.rs 5 行の削除。checker.rs は変更不要（既に `None` 対応済み）。
compiler.fav / checker.fav も基礎対応として修正する。

---

## T0 — 事前確認

- [ ] `cargo test` が 2900 tests / 0 failures であることを確認
- [ ] `fav/Cargo.toml` version が `43.0.0` であることを確認
- [ ] `fav/src/driver.rs` の `v43000_tests` 冒頭行番号を記録
- [ ] `versions/roadmap/roadmap-v43.1-v44.0.md` に v43.1.0 エントリが存在することを確認
- [ ] `fn double(x: Int) { x * 2 }` が現状でパースエラーになることを確認（制限の存在確認）
- [ ] compiler.fav で `Some(TkArrow)` ネストパターンが有効であることを確認（line 1247, 1261 等で使用済みを確認）✅ 事前確認済み

---

## T1 — `fav/src/frontend/parser.rs` 変更

`parse_fn_def()` の line 1981-1986 を削除:

**削除対象（5 行）:**
```rust
            if return_ty.is_none() {
                return Err(ParseError::new(
                    "function return type can only be omitted with `= expr` syntax",
                    self.peek_span().clone(),
                ));
            }
```

変更後のブロック:
```rust
        } else {
            self.parse_block()?
        };
```

---

## T2 — `fav/self/compiler.fav` 変更

`parse_fn_def_after_params()` を `->` オプション対応に変更（line 2029-2044):

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

---

## T3 — `fav/self/checker.fav` 変更

`check_body_ty()` に `ret == ""` 時の早期 OK パスを追加（line 1925-1931):

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

## T4 — `fav/src/driver.rs` テスト追加

`v43000_tests` の直前に `v43100_tests` を挿入:

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

## T5 — `fav/Cargo.toml` バージョン bump

`version = "43.0.0"` → `"43.1.0"`

---

## T6 — `CHANGELOG.md` 更新

`[v43.0.0]` の直前に `[v43.1.0]` エントリを追加:

```markdown
## [v43.1.0] — 2026-07-12

### Added
- `fav/src/frontend/parser.rs`: `fn f(params) { body }` での戻り値型省略（`-> RetType` 不要）をサポート
- `fav/self/compiler.fav`: `parse_fn_def_after_params()` — `->` オプション対応（`TeSimple("")` プレースホルダ）
- `fav/self/checker.fav`: `check_body_ty()` — `ret == ""` 時に body 推論で OK パス追加
- `v43100_tests`: `cargo_toml_version_is_43_1_0` / `return_type_omission_block_parseable` / `return_type_omission_return_ty_is_none`

### Notes
- `checker.rs` は変更なし（`return_ty: None` → body_ty 推論は既実装）
- self-hosted パスの `collect_fn_scheme_str` での推論型補完は v43.2.0 以降

---
```

---

## T7 — テスト実行・確認

- [ ] `cargo test` 実行
- [ ] failures = 0 を確認
- [ ] テスト数 = 2903 を確認（2900 + 3 件）
- [ ] `v43100_tests` 3 件 pass を確認
- [ ] 既存テストが壊れていないことを確認

---

## T8 — バージョン管理ドキュメント更新

- [ ] `versions/current.md` を v43.1.0（最新安定版、2903 tests）・v43.2.0（次に切る版）に更新
- [ ] `versions/roadmap/roadmap-v43.1-v44.0.md` の v43.1.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）`）
- [ ] 同ロードマップの v43.1.0 推定テスト数を `2895` → 実績 `2903` に修正
- [ ] `versions/v40-v45/v43.1.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）

---

## ファイル変更サマリー

| ファイル | 変更種別 |
|---|---|
| `fav/src/frontend/parser.rs` | 変更（5 行削除） |
| `fav/self/compiler.fav` | 変更（`parse_fn_def_after_params` 書き換え） |
| `fav/self/checker.fav` | 変更（`check_body_ty` に早期 OK パス追加） |
| `fav/src/driver.rs` | 変更（`v43100_tests` 3 件追加） |
| `fav/Cargo.toml` | 変更（version bump） |
| `CHANGELOG.md` | 変更（エントリ追加） |
| `versions/current.md` | 変更 |
| `versions/roadmap/roadmap-v43.1-v44.0.md` | 変更 |
| `versions/v40-v45/v43.1.0/tasks.md` | 変更 |
