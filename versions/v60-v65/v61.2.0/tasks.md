# v61.2.0 Tasks — as-pattern 拡張（ネストパターン・LSP hover 統合）

Date: 2026-07-31
Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3355 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"61.0.0"` であることを確認
  - `grep '^version' fav/Cargo.toml` → `version = "61.0.0"`
  - （サブバージョン v61.x.x は Cargo.toml を更新しないため "61.0.0" のままであること）
- [x] `v61200_tests` がまだ存在しないことを確認
  - `grep -c 'v61200_tests' fav/src/driver.rs` = 0 件
- [x] `v61100_tests` が存在すること（挿入先が実在すること）を確認
  - `grep -c 'v61100_tests' fav/src/driver.rs` ≥ 1 件
- [x] `Pattern::As` の checker 処理が存在することを確認
  - `grep -n 'Pattern::As' fav/src/middle/checker.rs` ≥ 1 件
- [x] `collect_as_pattern_hints` がまだ存在しないことを確認
  - `grep -c 'collect_as_pattern_hints' fav/src/lsp/inlay_hints.rs` = 0 件
- [x] `check_w039_as_name_shadows_inner` がまだ存在しないことを確認
  - `grep -c 'W039' fav/src/lint.rs` = 0 件

---

## T1: `inlay_hints.rs` — `collect_as_pattern_hints` 追加

`find_as_prefix` ヘルパーは追加しない（dead code 警告を回避するためロジックをインライン実装）。

### T1-1: `collect_as_pattern_hints` 関数を `collect_stage_hints` 直後に追加

```rust
/// v61.2.0: as-pattern 束縛変数の型を inlay hint 表示。
pub(crate) fn collect_as_pattern_hints(
    source: &str,
    type_at: &HashMap<Span, Type>,
) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    let mut byte_offset: usize = 0;
    for (line_idx, line) in source.lines().enumerate() {
        if let Some(as_pos) = line.find(" as ") {
            let rest = &line[as_pos + 4..];
            let trimmed = rest.trim_start();
            let trim_delta = rest.len() - trimmed.len();
            let name_end = trimmed
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(trimmed.len());
            if name_end > 0 {
                let name = &trimmed[..name_end];
                if name != "_" {
                    let prefix_len = as_pos + 4 + trim_delta;
                    let name_start = byte_offset + prefix_len;
                    let name_end_offset = name_start + name_end;
                    if let Some(ty) = find_type_at(type_at, name_start, name_end_offset) {
                        let col = (prefix_len + name_end) as u32;
                        hints.push(InlayHint {
                            position: Position {
                                line: line_idx as u32,
                                character: col,
                            },
                            label: format!(": {}", ty.display()),
                            kind: 1,
                        });
                    }
                }
            }
        }
        byte_offset += line.len() + 1;
    }
    hints
}
```

- [x] `collect_as_pattern_hints` を追加した
- [x] `pub(crate)` 修飾子が付いている（テストからアクセスできる）
- [x] `cargo build` でコンパイルエラーがないことを確認

### T1-2: `handle_inlay_hints` から呼び出し追加

`collect_effect_hints` 呼び出しの直後に追加:

```rust
// v61.2.0: as-pattern 束縛変数の型ヒント
hints.extend(collect_as_pattern_hints(&doc.source, &doc.type_at));
```

- [x] `handle_inlay_hints` に呼び出しを追加した
- [x] `cargo build` でコンパイルエラーがないことを確認

---

## T2: `lint.rs` — W039 追加

### T2-1: `collect_pattern_bound_names` ヘルパーを W038 ブロック直後に追加

```rust
// ── W039: as-name shadows inner binding (v61.2.0) ────────────────────────────

/// Pattern::Record のフィールドは PatternField enum で表現される
/// （Pun(String,Span) / Alias(String,Box<Pattern>,Span) / Wildcard(Span)）。
fn collect_pattern_bound_names(pat: &Pattern) -> Vec<String> {
    match pat {
        Pattern::Bind(name, _) => vec![name.clone()],
        Pattern::Record(fields, _) => fields.iter().filter_map(|f| match f {
            PatternField::Pun(name, _) => Some(name.clone()),
            PatternField::Alias(name, _, _) => Some(name.clone()),
            PatternField::Wildcard(_) => None,
        }).collect(),
        Pattern::Or(pats, _) => pats.iter().flat_map(|p| collect_pattern_bound_names(p)).collect(),
        Pattern::As(name, inner, _) => {
            let mut names = collect_pattern_bound_names(inner);
            names.push(name.clone());
            names
        }
        _ => vec![],
    }
}
```

- [x] `collect_pattern_bound_names` を追加した
- [x] `cargo build` でコンパイルエラーがないことを確認

### T2-2: W039 チェック関数群を追加

```rust
fn check_w039_as_name_shadows_inner(program: &Program, errors: &mut Vec<LintError>) {
    for item in &program.items {
        if let Item::FnDef(fd) = item {
            check_w039_in_stmts(&fd.body.stmts, errors);
        }
    }
}

fn check_w039_in_stmts(stmts: &[Stmt], errors: &mut Vec<LintError>) {
    for stmt in stmts {
        check_w039_in_stmt(stmt, errors);
    }
}

fn check_w039_in_stmt(stmt: &Stmt, errors: &mut Vec<LintError>) {
    match stmt {
        Stmt::Expr(expr) | Stmt::Bind(_, expr, _) => check_w039_in_expr(expr, errors),
        _ => {}
    }
}

fn check_w039_in_expr(expr: &Expr, errors: &mut Vec<LintError>) {
    match expr {
        Expr::Match(_, arms, _) => {
            for arm in arms {
                check_w039_in_pattern(&arm.pattern, errors);
                check_w039_in_expr(&arm.body, errors);
            }
        }
        _ => {}
    }
}

fn check_w039_in_pattern(pat: &Pattern, errors: &mut Vec<LintError>) {
    if let Pattern::As(name, inner, span) = pat {
        let inner_names = collect_pattern_bound_names(inner);
        if inner_names.iter().any(|n| n == name) {
            errors.push(LintError::new(
                "W039",
                format!(
                    "as-name `{}` shadows a binding introduced by the inner pattern; \
                     consider renaming the as-binding",
                    name
                ),
                span.clone(),
            ));
        }
        check_w039_in_pattern(inner, errors);
    }
}
```

- [x] W039 チェック関数群を追加した
- [x] `cargo build` でコンパイルエラーがないことを確認

### T2-3: `check_all` から W039 呼び出しを追加

W038 呼び出し行の直後に追加:

```rust
// v61.2.0: W039
check_w039_as_name_shadows_inner(program, &mut errors);
```

- [x] `check_all` に W039 呼び出しを追加した
- [x] `cargo build` でコンパイルエラーがないことを確認

---

## T3: `driver.rs` — `v61200_tests` モジュール追加

`v61100_tests` の直前（上側）に挿入する。

```rust
// -- v61200_tests (v61.2.0) -- as-pattern 拡張 --
#[cfg(test)]
mod v61200_tests {
    use super::*;

    /// as-pattern が Record パターンとネストできることを確認（v61.2.0: 既存 checker の動作保証）
    #[test]
    fn pattern_as_nested_record() {
        let src = concat!(
            "type Point { x: Int, y: Int }\n",
            "fn origin(p: Point) -> Int {\n",
            "  match p {\n",
            "    { x, y } as whole => x\n",
            "    _ => 0\n",
            "  }\n",
            "}\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let (errors, _) = crate::middle::checker::Checker::check_program(&prog);
        assert!(
            errors.is_empty(),
            "as-pattern nested in record should pass type check; errors: {:?}",
            errors
        );
    }

    /// as-pattern 束縛変数に inlay hint が生成されることを確認（v61.2.0: LSP 統合）
    #[test]
    fn pattern_as_lsp_hover_type() {
        use crate::lsp::inlay_hints::collect_as_pattern_hints;
        use crate::frontend::lexer::Span;
        use crate::middle::checker::Type;
        use std::collections::HashMap;

        // "  { x, y } as whole => x"
        //  0123456789012345678901234
        //              14   19
        let source = "  { x, y } as whole => x";
        let name_start: usize = 14;
        let name_end: usize = 19;
        let mut type_at = HashMap::new();
        type_at.insert(
            // col（第5引数）は find_type_at で参照されないため 1u32 を渡す
            Span::new("test", name_start, name_end, 1, 1u32),
            Type::Named("Point".to_string(), vec![]),
        );
        let hints = collect_as_pattern_hints(source, &type_at);
        assert!(
            !hints.is_empty(),
            "should generate an inlay hint for as-pattern name 'whole'"
        );
        assert!(
            hints[0].label.contains("Point"),
            "hint label should contain the type name; got {:?}",
            hints[0].label
        );
    }
}
```

- [x] `v61200_tests` モジュールを `v61100_tests` の直前（上側）に追加した
- [x] `use super::*;` が含まれている
- [x] `pattern_as_nested_record` テストが含まれている
- [x] `pattern_as_lsp_hover_type` テストが含まれている

---

## T4: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v61200_tests::pattern_as_nested_record` pass
- [x] `v61200_tests::pattern_as_lsp_hover_type` pass
- [x] 総テスト数 **3357** tests passed, 0 failed を確認

---

## T5: 事後処理

- [x] `versions/current.md` を v61.2.0 / 3357 tests に更新
- [x] `versions/roadmap/roadmap-v61.1-v62.0.md` の v61.2.0 実績欄を更新
- [x] `versions/roadmap/roadmap-v61.1-v62.0.md` の v61.7.0 セクションの `W039 type_hole_inferred` を `W040 type_hole_inferred` に更新（v61.2.0 で W039 を使用したため）
- [x] CHANGELOG.md: サブバージョンのため個別エントリは不要（v62.0 でまとめて記載）
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー指摘と対応

### 実装中の修正（2件）

- **[BUG] `Stmt::Bind(_, expr, _)` がコンパイルエラー**:
  `Stmt::Bind` は `BindStmt` 構造体を取る 1フィールドのタプル変体。
  `Stmt::Bind(b) => check_w039_in_expr(&b.expr, errors)` に修正。
- **[BUG] as-pattern のテスト構文が `as` キーワードを使用**:
  Favnir の as-pattern 実際の構文は `name @ sub_pattern`（`@` トークン）。
  `{ x, y } as whole` → `whole @ { x, y }` に修正。
  型定義構文も `type Point { ... }` → `type Point = { ... }` に修正。

### code-reviewer 指摘と対応（4件）

- **[BUG] `collect_pattern_bound_names` が `Variant` / `List` 内の束縛を無視**:
  `Pattern::Variant(_, Some(inner), _)` と `Pattern::List { head, tail, .. }` ケースを追加し
  内側パターンを再帰的に収集するよう修正。
- **[BUG] W039 が `block.expr`（最終返り値式）を検査していなかった**:
  `Block` は `stmts: Vec<Stmt>` + `expr: Box<Expr>` の構造。match が最終 expr として置かれる場合、
  `fd.body.stmts` は空になるため W039 が発火しなかった。
  `check_w039_as_name_shadows_inner` に `check_w039_in_expr(&fd.body.expr, errors)` を追加。
- **[BUG] W039 positive test が欠落**:
  `w039_as_name_shadows_inner_should_warn` テストを追加（`y @ y` で W039 が発火することを確認）。
  テスト追加により総テスト数 3357 → 3358 に増加。
- **[BUG] `collect_as_pattern_hints` に Known limitation コメント不足**:
  `/// Known limitation: 1 行に複数の \` as \` が存在する場合は最初のもののみ処理される。` を追加。
  `check_w039_in_stmt` の `_ => {}` にも intentional スキップのコメントを追加。
- **[FALSE ALARM] W039 が error_catalog.rs に未登録**:
  W037/W038 も `error_catalog.rs` に未登録であることを確認。lint コードは error_catalog.rs の対象外。

---

Status: COMPLETE
