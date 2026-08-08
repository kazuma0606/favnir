# Spec — v56.3.0 — 行多相レコード活用拡張

## 概要

v33.0 実装済みの `R with { id: Int }` 行多相（型制約経由）に加え、
`{ field: Type | r }` 形式のインライン行変数型を parser で受理する。
`TypeExpr::display()` ヘルパーで `{ name: String | r }` 形式の表示を追加し、
LSP ホバー向けの行変数型表示基盤を整備する。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v56.1-v57.0.md` — v56.3.0 セクション
- `versions/roadmap/roadmap-v55.1-v60.0.md` — v56.3.0 行
- ベーステスト数: **3231**（v56.2.0 完了時点の実績値）
- 目標テスト数: **3233**（+2）
- **注意**: ロードマップには「ベース 3232 + 2 = 3234」と記載されているが、
  v56.2.0 の実績は 3231 であり、本バージョンの目標は 3233 とする（両ロードマップを修正）

---

## 既存実装との関係

| 要素 | バージョン | 状態 |
|------|-----------|------|
| `R with { id: Int }` 行多相（HasField 制約） | v33.0 | 実装済み |
| `TypeExpr::RecordType(Vec<(String, TypeExpr)>, Span)` | v18.2.0 | 実装済み |
| `resolve_field_access` — 末尾フォールバック `_ => Type::Unknown` | v18.2.0 | 実装済み |
| `Type::is_compatible` — `Unknown` は全型と互換（L71-74） | v0.x | 実装済み |
| `unify_deep` / HM 型推論での行変数扱い | 未実装（将来スプリント） | **本バージョン対象外** |
| `{ field: Type | r }` インライン行変数構文 | v56.3.0 | **本バージョンで追加** |

---

## スコープの明確化

### `unify_deep` 拡張を行わない根拠

ロードマップには「HM 型推論の `unify` で行変数を正しく扱う」と記載されているが、
本バージョンでは `unify_deep` / `HM` の変更は行わず、以下の方針で `row_poly_generic_fn` テストを実現する:

- `TypeExpr::RecordType` の resolver が `Type::Unknown` を返す（既存挙動）
- `resolve_field_access` 末尾が `_ => Type::Unknown` を返す（`checker.rs` L5628）
- `Type::Unknown` は `is_compatible` で全型と互換（`checker.rs` L71-74）
- したがって、`record: { name: String | r }` パラメータへの `record.name` アクセスは
  `Type::Unknown` → `String` 互換のため型エラーなし

完全な行型推論（`Type::Row` 追加、`unify_deep` 拡張）は将来のスプリントで対応する。
ロードマップの記述は「将来の拡張を含む全体像」であり、本バージョンは parser + display の基盤整備に集中する。

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "56.3.0"
```

---

### 2. `fav/src/ast.rs` — `TypeExpr::RecordType` に `Option<String>` row_var を追加

```rust
// Before (v18.2.0):
RecordType(Vec<(String, TypeExpr)>, Span),

// After (v56.3.0):
RecordType(Vec<(String, TypeExpr)>, Option<String>, Span),
```

span メソッドの match arm も更新:

```rust
TypeExpr::RecordType(_, _, s) => s,
```

---

### 3. `fav/src/ast.rs` — `TypeExpr::display()` ヘルパー追加

`impl TypeExpr` ブロックに追加（`FlwStep::display_str` とは別）:

```rust
impl TypeExpr {
    /// v56.3.0: human-readable display string (used in LSP hover / tests).
    pub fn display(&self) -> String {
        match self {
            TypeExpr::Named(name, args, _) if args.is_empty() => name.clone(),
            TypeExpr::Named(name, args, _) => {
                let s: Vec<_> = args.iter().map(|a| a.display()).collect();
                format!("{}<{}>", name, s.join(", "))
            }
            TypeExpr::RecordType(fields, row_var, _) => {
                let field_strs: Vec<_> = fields
                    .iter()
                    .map(|(n, t)| format!("{}: {}", n, t.display()))
                    .collect();
                match row_var {
                    Some(r) => format!("{{ {} | {} }}", field_strs.join(", "), r),
                    None => format!("{{ {} }}", field_strs.join(", ")),
                }
            }
            TypeExpr::Arrow(a, b, _) => format!("{} -> {}", a.display(), b.display()),
            _ => "...".to_string(),
        }
    }
}
```

---

### 4. `fav/src/frontend/parser.rs` — `{ field: Type | r }` 解析対応

`parse_base_type` 内の RecordType 解析ループ内で、`}` の前に `| ident` が来た場合に
row_var を取得する:

```rust
if self.peek() == &TokenKind::LBrace {
    self.advance();
    let mut fields = vec![];
    let mut row_var: Option<String> = None;
    while self.peek() != &TokenKind::RBrace && !self.at_end() {
        // `| ident` → row variable (v56.3.0)
        if self.peek() == &TokenKind::Pipe {
            self.advance(); // consume `|`
            let (name, _) = self.expect_ident()?;
            row_var = Some(name);
            break;
        }
        let (field_name, _) = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let field_ty = self.parse_type_expr()?;
        fields.push((field_name, field_ty));
        if self.peek() == &TokenKind::Comma {
            self.advance();
        }
    }
    self.expect(&TokenKind::RBrace)?;
    let span = self.span_from(&start);
    return Ok(TypeExpr::RecordType(fields, row_var, span));
}
```

---

### 5. 全 `RecordType` match arm 更新

`cargo build` のコンパイルエラーを頼りに全箇所を網羅する（推定 24〜25 箇所）。

主要ファイルと変更方針:

| ファイル | 変更方針 |
|---------|---------|
| `ast.rs` span arm | `(_, s)` → `(_, _, s)` |
| `emit_python.rs` | `(_, _)` → `(_, _, _)` |
| `driver.rs`（8 箇所） | `(fields, _)` → `(fields, _, _)` / `(_, _)` → `(_, _, _)` |
| `fmt.rs`（2 箇所） | `(fields, row_var, _)` として row_var も `\| r` 形式で出力 |
| `lint.rs` | `(fields, _)` → `(fields, _, _)` |
| `lsp/references.rs` | `(fields, _)` → `(fields, _, _)` |
| `middle/ast_lower_checker.rs`（2 箇所） | `(_, _)` → `(_, _, _)` |
| `middle/compiler.rs`（3 箇所） | `(_, _)` / `(fields, span)` → row_var を保持して再構築 |
| `middle/checker.rs`（4 箇所） | `(_, _)` / `(fields, _)` → row_var を `_row_var` で束縛 |

#### `middle/compiler.rs` の `substitute_self_in_type_expr` について

`substitute_self_in_type_expr` 関数（L1641）の `RecordType` arm は、
変更後に `row_var` を保持して再構築する必要がある:

```rust
// Before:
TypeExpr::RecordType(fields, span) => TypeExpr::RecordType(
    fields.iter().map(|(n, t)| (n.clone(), substitute_self_in_type_expr(t, type_name))).collect(),
    span.clone(),
),
// After:
TypeExpr::RecordType(fields, row_var, span) => TypeExpr::RecordType(
    fields.iter().map(|(n, t)| (n.clone(), substitute_self_in_type_expr(t, type_name))).collect(),
    row_var.clone(),
    span.clone(),
),
```

---

### 6. `fav/src/driver.rs` — `v56300_tests` モジュール追加 + v56200_tests 更新

#### 6a. `v56200_tests` から `cargo_toml_version_is_56_2_0` を削除

Cargo.toml が 56.3.0 に更新されるため削除。

#### 6b. `v56300_tests` モジュールを `v56200_tests` の直前に挿入

```rust
// -- v56300_tests (v56.3.0) -- 行多相レコード活用拡張 --
#[cfg(test)]
mod v56300_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;
    use crate::ast::TypeExpr;

    fn check_errors(src: &str) -> Vec<String> {
        let program = Parser::parse_str(src, "v56300_test.fav").expect("parse");
        Checker::check_program(&program)
            .0
            .iter()
            .map(|e| e.code.to_string())
            .collect()
    }

    #[test]
    fn cargo_toml_version_is_56_3_0() {
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            cargo_toml.contains("version = \"56.3.0\""),
            "Cargo.toml version should be 56.3.0, got: {}",
            cargo_toml.lines().find(|l| l.contains("version")).unwrap_or("")
        );
    }

    #[test]
    fn row_poly_generic_fn() {
        // { name: String | r } 構文を持つ関数が型チェックでエラーなし
        // 根拠: RecordType → Type::Unknown 、Unknown は is_compatible で全型と互換（L71-74）
        let errors = check_errors(r#"
fn get_name<r>(record: { name: String | r }) -> String {
    record.name
}
fn main() -> String {
    get_name({ name: "Alice" })
}
"#);
        assert!(
            errors.is_empty(),
            "row_poly_generic_fn should not emit errors, got: {:?}",
            errors
        );
    }

    #[test]
    fn row_poly_lsp_hover() {
        // TypeExpr::RecordType with row_var displays as { name: String | r }
        use crate::ast::Span;
        let span = Span { file: "test".to_string(), start: 0, end: 0, line: 1, col: 1 };
        let name_ty = TypeExpr::Named("String".to_string(), vec![], span.clone());
        let te = TypeExpr::RecordType(
            vec![("name".to_string(), name_ty)],
            Some("r".to_string()),
            span,
        );
        let display = te.display();
        assert!(
            display.contains("name: String") && display.contains("| r"),
            "row-poly type should display as '{{ name: String | r }}', got: {:?}",
            display
        );
    }
}
```

---

## テスト仕様

| テスト名 | 検証内容 |
|---------|---------|
| `cargo_toml_version_is_56_3_0` | Cargo.toml が `56.3.0` を反映 |
| `row_poly_generic_fn` | `{ name: String | r }` 構文のパース + 型チェックでエラーなし（Unknown 互換） |
| `row_poly_lsp_hover` | `TypeExpr::RecordType` + row_var → `display()` が `name: String` と `| r` を含む |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3233 tests passed, 0 failed**）
- `cargo clippy -- -D warnings` クリーン
- `cargo_toml_version_is_56_3_0` pass
- `row_poly_generic_fn` pass（`errors.is_empty()` assert）
- `row_poly_lsp_hover` pass（`display()` が `name: String` と `| r` を含む）
- `TypeExpr::RecordType` が `Option<String>` row_var フィールドを持つ
- `{ name: String | r }` がパースエラーなし
- `TypeExpr::display()` が行変数を `| r` で表示する
- `substitute_self_in_type_expr` が row_var を保持して再構築する
- `v56200_tests::cargo_toml_version_is_56_2_0` が削除されている
- `CHANGELOG.md` に v56.3.0 エントリが追加されている（version: `56.2.0 → 56.3.0`）
- `versions/current.md` が v56.3.0 / 3233 tests を反映
- 両ロードマップの v56.3.0 実績を COMPLETE に更新
- 両ロードマップのテスト数（`3232 + 2 = 3234` → `3231 + 2 = 3233`）を修正

---

## 備考

- **型システムへの影響最小化方針**:
  `TypeExpr::RecordType` → `Type::Unknown` の変換は既存のまま維持する。
  `resolve_field_access` 末尾（L5628）が `_ => Type::Unknown` を返し、
  `Type::Unknown` は `is_compatible`（L71-74）で全型と互換のため、
  `record.name` アクセスも `String` 戻り値も型エラーなしで通過する。
  完全な行型推論（`Type::Row` 追加、`unify_deep` 拡張）は将来のスプリントで対応。
- **`row_poly_lsp_hover` の実装形態**:
  フル LSP プロトコルテストではなく、`TypeExpr::display()` の単体テスト。
- **Pipe トークン**: `| r` の `|` は `TokenKind::Pipe`（lexer.rs L462 に定義済み）。
  `|>` は `Pipeline` トークンであり区別される。
- **ロードマップのテスト数誤記**:
  両ロードマップの `3232 + 2 = 3234` を `3231 + 2 = 3233` に修正する（T7/T14）。
