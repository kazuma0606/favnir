# v61.2.0 Plan — as-pattern 拡張（ネストパターン・LSP hover 統合）

Date: 2026-07-31
Status: COMPLETE

---

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/lsp/inlay_hints.rs` | 追加 | `collect_as_pattern_hints` 関数 + `handle_inlay_hints` 呼び出し追加 |
| `fav/src/lint.rs` | 追加 | W039 `check_w039_as_name_shadows_inner` + `check_all` 呼び出し追加 |
| `fav/src/driver.rs` | 追加 | `v61200_tests` モジュール（2 件） |

`fav/Cargo.toml` バージョン変更なし（サブバージョン）。`checker.rs` 変更なし。
新規ファイルなし。

---

## 実装ステップ

### Step 1: `inlay_hints.rs` — `collect_as_pattern_hints` 追加

#### 1-1: `collect_as_pattern_hints` 関数追加

`find_as_prefix` ヘルパーは追加しない（dead code 警告を回避するため、オフセット計算をインライン）。
`collect_stage_hints` の直後に追加。

```rust
/// v61.2.0: as-pattern 束縛変数の型を inlay hint 表示。
/// ` as <name>` 形式をテキストスキャンで検出。
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

#### 1-2: `handle_inlay_hints` から呼び出し

L28〜L30 付近（`collect_effect_hints` の直後）に追加:

```rust
// v61.2.0: as-pattern 束縛変数の型ヒント
hints.extend(collect_as_pattern_hints(&doc.source, &doc.type_at));
```

### Step 2: `lint.rs` — W039 追加

#### 2-1: `collect_pattern_bound_names` ヘルパー追加

W038 実装（L3102 付近）の直後に追加。

```rust
// ── W039: as-name shadows inner binding (v61.2.0) ────────────────────────────

/// as-pattern の内側パターンが束縛する変数名を収集する。
/// Pattern::Record のフィールドは PatternField enum
/// （Pun(String,Span) / Alias(String,Box<Pattern>,Span) / Wildcard(Span)）で表現される。
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

#### 2-2: `check_w039_as_name_shadows_inner` 追加

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
        // 他の Expr バリアントは子 Expr を再帰
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

#### 2-3: `check_all` から呼び出し

W038 呼び出し行の直後に追加（L131 付近）:

```rust
// v61.2.0: W039
check_w039_as_name_shadows_inner(program, &mut errors);
```

### Step 3: `driver.rs` — `v61200_tests` モジュール追加

`v61100_tests` の直前（上側）に挿入。

```rust
// -- v61200_tests (v61.2.0) -- as-pattern 拡張 --
#[cfg(test)]
mod v61200_tests {
    use super::*;

    /// as-pattern が Record パターンとネストできることを確認
    #[test]
    fn pattern_as_nested_record() { ... }

    /// as-pattern 束縛変数に inlay hint が生成されることを確認
    #[test]
    fn pattern_as_lsp_hover_type() { ... }
}
```

---

## 挿入位置サマリ

| 対象 | 挿入位置 |
|---|---|
| `v61200_tests` | `driver.rs` の `v61100_tests` の直前（上側） |
| `collect_as_pattern_hints` | `inlay_hints.rs` の `collect_stage_hints` 直後 |
| `handle_inlay_hints` 呼び出し | `collect_effect_hints` 呼び出しの直後（L29〜30 付近） |
| W039 関連関数群 | `lint.rs` の W038 ブロック直後 |
| `check_w039_as_name_shadows_inner` 呼び出し | `lint.rs` L131 の W038 呼び出し直後 |

---

## 注意点

- W038 は v56.7.0 で wildcard import collision として実装済み。**as-name 衝突は W039 を使用**。
  ロードマップ v61.7.0 の `type_hole_inferred` lint コードは W039 → W040 に変更する（T5 で更新）。
- `find_as_prefix` ヘルパーは追加しない。`collect_as_pattern_hints` にインライン実装する（dead code 防止）。
- `collect_as_pattern_hints` はテキストスキャン方式。`use X as Y` の `as` にも誤検出するが、
  `use` 行に `type_at` エントリが存在しないため実害なし（`find_type_at` が None を返す）。
- `collect_pattern_bound_names` の `Pattern::Record` アームは `PatternField` enum を match する。
  `PatternField::Pun(name, _)` と `PatternField::Alias(name, _, _)` から名前を取得し、
  `PatternField::Wildcard(_)` は None として filter_map する。
- `check_w039_in_expr` は Match のみ再帰（意図的なスコープ制限）。FnDef 本体の直接 match 式のみ対象。
  ネストされた match（if/ブロック/クロージャ内）は対象外（他の W03x lint と同一方針）。
- `Cargo.toml` は変更しない（v62.0.0 で "62.0.0" に更新予定）。
