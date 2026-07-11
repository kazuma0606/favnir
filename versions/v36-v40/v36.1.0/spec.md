# v36.1.0 spec — `schema` インライン定義構文

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v36.1.0 |
| テーマ | `schema` リテラル定義構文（Data Quality First スプリント開始） |
| 前提 | v36.0.0 COMPLETE — Deployment Story 宣言済み |
| 完了条件 | `v36100_tests` 全テスト pass・`cargo test` 0 failures・`MILESTONE.md` 更新不要 |

## 背景と目的

v32.4.0 で `schema "postgres:users"` 形式（文字列 URI 参照）を実装した。
本バージョンはインライン構造体形式 `schema Orders { id: Int, amount: Float }` を追加し、
Data Quality First スプリントの基盤を確立する。

## 既存実装の確認

| 実装 | 場所 | 内容 |
|---|---|---|
| `TypeExpr::Schema(String, Span)` | `ast.rs` line 116 | 文字列 URI 参照形式（v18.4/v32.4） |
| `parse_base_type` の `schema "..."` 処理 | `parser.rs` line 1646 | TypeExpr として解析 |

`Item` enum に `SchemaDef` は未存在（今回追加）。

## 実装スコープ

### 1. `fav/src/ast.rs` — `SchemaDef` 構造体と `Item::SchemaDef` 追加

derive 属性は ast.rs の他の構造体（`TypeDef` 等）に合わせること（`Debug, Clone` が最小セット、追加 derive が必要な場合は既存ノードを確認して揃える）。

```rust
/// インライン schema 定義（v36.1.0）
/// `schema Orders { id: Int, amount: Float }` 構文
#[derive(Debug, Clone)]
pub struct SchemaDef {
    pub name: String,
    pub fields: Vec<(String, TypeExpr)>,
    pub span: Span,
}
```

`Item` enum に追加:
```rust
SchemaDef(SchemaDef), // schema Orders { id: Int, ... }  v36.1.0
```

### 2. `fav/src/frontend/parser.rs` — トップレベル `schema Name { ... }` 解析

`parse_item` (or `parse_program` のループ) に分岐を追加:

```rust
// schema Orders { id: Int, amount: Float }  (v36.1.0)
if matches!(self.peek(), TokenKind::Ident(n) if n == "schema") {
    // peek ahead: next token must be an Ident (name), not Str (legacy URI form)
    if matches!(self.peek_nth(1), TokenKind::Ident(_)) {
        return Ok(Some(Item::SchemaDef(self.parse_schema_def()?)));
    }
}
```

`parse_schema_def` 関数を追加:
```rust
fn parse_schema_def(&mut self) -> Result<SchemaDef, ParseError> {
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
        // optional newline separator (no comma required)
        if self.peek() == &TokenKind::Comma { self.advance(); }
    }
    self.expect(&TokenKind::RBrace)?;
    Ok(SchemaDef { name, fields, span: self.span_from(&start) })
}
```

### 3. `Item::SchemaDef` を参照する既存 match 文に no-op アーム追加

`SchemaDef` を `Item` に追加すると、`Item` を網羅的に match している箇所でコンパイルエラーが発生する。
主な対象ファイル:
- `fav/src/checker.rs`（`check_item` 等）
- `fav/src/lint.rs`（`check_item` 等）
- `fav/src/lineage.rs`（`collect_lineage` 等）
- `fav/src/fmt.rs`（`fn item` — exhaustive match、`Item::SchemaDef(_) => String::new()` を追加）
- `fav/src/driver.rs`（`render_proto_schema` 等）

各 match 文に `Item::SchemaDef(_) => {}` (no-op) を追加してコンパイルエラーを解消する。
実際の型チェック・lint ロジックは v36.2〜v36.6 に委ねる。

### 4. `fav/src/driver.rs` — テストモジュール

`v36000_tests::cargo_toml_version_is_36_0_0` をスタブ化し、`v36100_tests` を追加。

## v36100_tests の設計

| テスト名 | 検証内容 |
|---|---|
| `cargo_toml_version_is_36_1_0` | Cargo.toml に `"36.1.0"` が含まれる |
| `changelog_has_v36_1_0` | `CHANGELOG.md` に `[v36.1.0]` が含まれる |
| `schema_def_item_in_ast` | `ast.rs` に `SchemaDef` かつ `Item::SchemaDef` または `SchemaDef(SchemaDef)` が含まれる |

## ロードマップとの整合

`roadmap-v36.1-v37.0.md` の v36.1.0 は:
- `schema Name { ... }` 構文が型チェックを通る
- Rust テスト 3 件

型チェックの「通る」はコンパイルエラーなし（no-op アームで対応）を意味し、
W025 lint や E0380〜E0384 エラーコードは後続スプリントで実装する。

ロードマップの「想定構文」には `where { ... }` 制約節が含まれているが、v36.1.0 では**未対応**とする。
`parse_schema_def` のループ内でフィールド後に `where` トークンが現れた場合は**パースエラー**とする（スキップしない）。
`where` 節のサポートは v36.2.0 の `expect` ブロックと合わせて実装する。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `ast.rs` に `SchemaDef` 構造体と `Item::SchemaDef` が含まれる | `schema_def_item_in_ast` テスト |
| 2 | `CHANGELOG.md` に `[v36.1.0]` が含まれる | `changelog_has_v36_1_0` テスト |
| 3 | `Cargo.toml` バージョンが `36.1.0` | `cargo_toml_version_is_36_1_0` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2659） | `cargo test` 実行結果 |
