# v70.6.0 Plan — `bind` 分割束縛拡張 / Named Destructuring

Date: 2026-08-09
Status: 計画中

---

## 実装ステップ（依存順）

### Step 1: 既存実装を確認

1. `parse_bind_stmt`（parser.rs line 2680）が `parse_pattern()` を呼ぶことを確認済み
   - Record: `Pattern::Record` → `bind {name} <- user` で動作
   - List: `Pattern::List` → `bind [head, ..tail] <- items` で動作（`DotDot` トークン使用）
2. checker.rs が `BindStmt` の `Pattern::Record` / `Pattern::List` を正しく型チェックするか確認
3. compiler.fav line 1494: `TkBind` 後は `TkIdent` のみ処理。`TkLBrace` / `TkLBracket` 未対応

---

### Step 2: compiler.fav の `TkBind` ハンドラに `TkLBrace`（Record）分岐を追加

Record 分割束縛 `bind {field1, field2} <- expr` を以下にデシュガーする:
```
EBind("$_d", rhs_expr,
  EBind("field1", EAccess(EVar("$_d"), "field1"),
    EBind("field2", EAccess(EVar("$_d"), "field2"), cont)))
```

実装方針:
- `TkLBrace` を検出したら `parse_destr_fields(rest1)` ヘルパーで `["field1", "field2"]` のリストを取得
- `parse_destr_fields` は `{` 消費済みの状態で `TkIdent(fname) + TkComma*` を収集し `}` で終了
- `TkBackArrow` を消費して RHS を `parse_expr` でパース
- `"$_d"` という一時変数名でネストした `EBind` チェーンを生成

```favnir
fn parse_destr_fields(toks: List<Token>, acc: List<String>) -> Result<FieldsParse, String> {
    match peek(toks) {
        Some(TkRBrace) => {
            bind rest <- advance(toks);
            Result.ok(FieldsParse { fields: List.reverse(acc)  rest: rest })
        }
        Some(TkIdent(fname)) => {
            bind rest1 <- advance(toks);
            match peek(rest1) {
                Some(TkComma) => {
                    bind rest2 <- advance(rest1);
                    parse_destr_fields(rest2, List.cons(fname, acc))
                }
                _ => parse_destr_fields(rest1, List.cons(fname, acc))
            }
        }
        _ => Result.err("expected field name or } in bind destructure")
    }
}

fn make_destr_binds(tmp: String, fields: List<String>, cont: Expr) -> Expr {
    match fields {
        [] => cont
        [fname, ..rest] =>
            EBind(fname, EAccess(EVar(tmp), fname), make_destr_binds(tmp, rest, cont))
    }
}
```

`TkBind` ハンドラに `TkLBrace` アームを追加:
```favnir
Some(TkLBrace) => {
    match parse_destr_fields(rest1, []) {
        Err(e) => Result.err(e)
        Ok(fields_p) => {
            match expect_tok(fields_p.rest, TkBackArrow) {
                Err(e) => Result.err(e)
                Ok(rest2) => {
                    match parse_expr(rest2) {
                        Err(e) => Result.err(e)
                        Ok(val_p) => {
                            match parse_block_inner(match peek(val_p.rest) {
                                Some(TkSemicolon) => advance(val_p.rest)
                                _ => val_p.rest
                            }) {
                                Err(e) => Result.err(e)
                                Ok(cont_p) => {
                                    bind inner <- Result.ok(make_destr_binds("$_d", fields_p.fields, cont_p.expr));
                                    Result.ok(ExprParse { expr: EBind("$_d", val_p.expr, inner)  rest: cont_p.rest })
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

**List スプレッド（`TkLBracket`）は compiler.fav スコープ外（v70.7.0 以降）:**
`List.nth` / `List.drop` の VM プリミティブ確認が必要なため次版に先送り。
Rust パーサーは `bind [head, ..tail] <- items`（`..` = `TokenKind::DotDot`）を既にサポート済み — E2E テストは Rust パイプラインのみ確認。

**ヘルパー命名（ロードマップとの差異）:**
ロードマップでは `parse_bind_lhs` と記載されているが、実装では `parse_destr_fields`（フィールドリストのみを扱う）+ `make_destr_binds`（EBind チェーン生成）に分割する。

**テスト数:**
現在 3570 tests → v70.6.0 で +2 = **3572 tests**（ロードマップの 3571 は誤記）。

確認: `cargo test` で既存テスト（3570 件）が全 pass することを確認。

---

### Step 3: `v706000_tests` モジュールを driver.rs 末尾に追加

```rust
#[cfg(test)]
mod v706000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    #[test]
    fn bind_destructure_record() {
        // Record 分割束縛（bind {field} <- expr）の全パイプライン確認
        let src = concat!(
            "type User = { name: String score: Int }\n",
            "fn greet(u: User) -> String {\n",
            "    bind {name, score} <- u\n",
            "    name\n",
            "}\n",
            "public fn main() -> Bool { true }\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse should succeed");
        let (errors, _) = Checker::check_program(&prog);
        assert!(
            errors.is_empty(),
            "record bind destructure should type-check; errors: {:?}",
            errors
        );
        let _artifact = super::build_artifact(&prog);
    }

    #[test]
    fn bind_destructure_list_spread() {
        // List スプレッド束縛（bind [head, ..tail] <- items）の全パイプライン確認
        let src = concat!(
            "fn first_item(items: List<Int>) -> Int {\n",
            "    bind [head, ..tail] <- items\n",
            "    head\n",
            "}\n",
            "public fn main() -> Bool { true }\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse should succeed");
        let (errors, _) = Checker::check_program(&prog);
        assert!(
            errors.is_empty(),
            "list spread bind destructure should type-check; errors: {:?}",
            errors
        );
        let _artifact = super::build_artifact(&prog);
    }
}
```

確認: `cargo test v706000` で 2 件 pass することを確認。

---

### Step 4: Cargo.toml バージョン更新

- `fav/Cargo.toml` の `version = "70.5.0"` → `"70.6.0"`
- driver.rs 内の `"70.5.0"` を `replace_all: true` で `"70.6.0"` に一括更新
  - 対象: `cargo_toml_version_is_70_5_0` テスト関数内の `"70.5.0"` 文字列

---

### Step 5: CHANGELOG.md 更新

```markdown
## [v70.6.0] — 2026-08-09 — `bind` 分割束縛拡張

### Added
- `v706000_tests`: 2 件追加（3570 → 3572 tests）
  - `bind_destructure_record` — Record 分割束縛の parse + typecheck + compile
  - `bind_destructure_list_spread` — List スプレッド束縛の parse + typecheck + compile

### Fixed
- `compiler.fav` `TkBind` ハンドラ: `TkLBrace` 対応追加（`bind {field} <- expr` を `EBind` チェーンにデシュガー）
  - `parse_destr_fields` ヘルパー追加
  - `make_destr_binds` ヘルパー追加

### Verified
- Rust パイプライン（parser / compiler.rs / codegen.rs）における Record/List 分割束縛の E2E コンパイル動作を確認
```

---

### Step 6: 最終確認

- `cargo test v706000` で 2 件 pass
- `cargo test` 全体で 3572 tests pass（0 failures）
- `versions/current.md` を v70.6.0 進行中に更新
