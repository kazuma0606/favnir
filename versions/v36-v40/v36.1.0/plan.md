# v36.1.0 実装計画 — `schema` インライン定義構文

## 実装順序

| ステップ | 対象 | 内容 |
|---|---|---|
| S1 | `CHANGELOG.md` | `## [v36.1.0]` エントリを追加（`## [35.3.0]` の直前） |
| S2 | `fav/src/ast.rs` | `SchemaDef` 構造体 + `Item::SchemaDef` 追加 |
| S3 | `fav/src/frontend/parser.rs` | `parse_schema_def` 追加 + `parse_item` に分岐追加 |
| S4 | 各ファイルの match 文 | `Item::SchemaDef(_) => {}` no-op アームを追加してコンパイルエラー解消 |
| S5 | `fav/src/driver.rs` | `v36000_tests::cargo_toml_version_is_36_0_0` をスタブ化 |
| S6 | `fav/src/driver.rs` | `v36100_tests` モジュール（3 件）を追加 |
| S7 | `fav/Cargo.toml` | バージョンを `36.0.0` → `36.1.0` に更新（必ず **S4 かつ S5** 完了後） |
| S8 | `cargo test` | 全通過確認（≥ 2659 件） |

## 各ステップの詳細

### S1: CHANGELOG.md

`## [v36.0.0]` エントリの直後（`## [35.3.0]` の直前）に挿入する。
CHANGELOG.md は Keep a Changelog 形式で最新版が先頭のため、v36.1.0 が二番目のエントリになる。

```markdown
## [v36.1.0] — 2026-07-08

### Added
- `schema Name { field: Type }` インライン schema 定義構文
- `ast::SchemaDef` 構造体・`Item::SchemaDef` variant（v36.1.0）
- `parse_schema_def` — トップレベル schema 宣言パーサー
- `schema` キーワードが Ident 名の後に `{` を続けるとき `Item::SchemaDef` に解析

---
```

### S2: ast.rs — SchemaDef 追加

`Item` enum の末尾付近（`TestGroup` の後など）に追加:

```rust
/// インライン schema 定義（v36.1.0）: `schema Orders { id: Int, amount: Float }`
SchemaDef(SchemaDef),
```

ファイル内の適切な場所（`ImplDef` 定義の後など）に `SchemaDef` 構造体を追加:

```rust
// ── SchemaDef (v36.1.0) ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SchemaDef {
    pub name: String,
    pub fields: Vec<(String, TypeExpr)>,
    pub span: Span,
}
```

### S3: parser.rs — parse_schema_def と parse_item 分岐追加

`parse_item` 関数内で `schema` キーワードのトークンを見つけたとき、
次のトークンが `Ident`（名前）なら `parse_schema_def` にディスパッチする。

> **既存との衝突注意**: `parse_base_type` の schema URI 処理（`schema "uri"`）は
> TypeExpr レベルの処理であり、`parse_item` のトップレベル分岐とは競合しない。
> トークン先読みで `Ident` か `Str` かを判定する。

`parse_schema_def`:
```rust
fn parse_schema_def(&mut self) -> Result<crate::ast::SchemaDef, ParseError> {
    let start = self.peek_span().clone();
    self.advance(); // consume `schema`
    let (name, _) = self.expect_ident()?;
    self.expect(&TokenKind::LBrace)?;
    let mut fields = vec![];
    while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
        let (field_name, _) = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let field_ty = self.parse_type_expr()?;
        fields.push((field_name, field_ty));
        if self.peek() == &TokenKind::Comma { self.advance(); }
    }
    self.expect(&TokenKind::RBrace)?;
    Ok(crate::ast::SchemaDef {
        name,
        fields,
        span: self.span_from(&start),
    })
}
```

### S4: match 文への no-op アーム追加

`cargo build 2>&1 | grep "error\[E"` でコンパイルエラーを確認し、
`Item::SchemaDef` が非網羅的な match のエラー箇所を特定して no-op アームを追加する。

主な対象（exhaustive-match-checker エージェントで特定）:
- `checker.rs` の `check_item`
- `lint.rs` の item チェック関数
- `lineage.rs`
- `fmt.rs` の `fn item`（`Item::SchemaDef(_) => String::new()` を追加）
- `driver.rs` の `render_proto_schema` 等

各 match 文に（`fmt.rs` のみ返り値型に合わせて調整）:
```rust
Item::SchemaDef(_) => {} // v36.1.0 — 型チェックは v36.2 以降
```

### S5: driver.rs — v36000_tests スタブ化

対象:
```rust
fn cargo_toml_version_is_36_0_0() {
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains("36.0.0"), "Cargo.toml must contain version 36.0.0");
}
```

変更後:
```rust
fn cargo_toml_version_is_36_0_0() {
    // stubbed: version bumped to 36.1.0
}
```

### S6: driver.rs — v36100_tests モジュール追加

`v36000_tests` モジュールの閉じ `}` の後（ファイル末尾）に追加:

```rust
// ── v36100_tests (v36.1.0) — schema インライン定義構文 ──────────────────────
#[cfg(test)]
mod v36100_tests {
    #[test]
    fn cargo_toml_version_is_36_1_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("36.1.0"), "Cargo.toml must contain version 36.1.0");
    }
    #[test]
    fn changelog_has_v36_1_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v36.1.0]"), "CHANGELOG.md must contain [v36.1.0]");
    }
    #[test]
    fn schema_def_item_in_ast() {
        let src = include_str!("../ast.rs");
        let src = include_str!("../ast.rs");
        assert!(
            src.contains("SchemaDef") && (src.contains("Item::SchemaDef") || src.contains("SchemaDef(SchemaDef)")),
            "ast.rs must contain SchemaDef struct and Item::SchemaDef variant"
        );
    }
}
```

### S7: Cargo.toml バージョン更新

**必ず S4（コンパイルエラー解消）かつ S5（スタブ化）完了後に実行すること**（S4 未完了だと `cargo test` がビルドエラーで終わる）。

`version = "36.0.0"` → `version = "36.1.0"`

### S8: cargo test

期待値: 2656（現在）+ 3（v36100_tests）= **2659 件** pass、0 failures
