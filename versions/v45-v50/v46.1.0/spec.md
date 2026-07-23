# Spec: v46.1.0 — `#[test]` ブロック AST + parser

Date: 2026-07-16
Status: TODO

---

## 概要

`#[test]` アトリビュートを AST に追加し、`parser.rs` で `#[test] fn ...` を解析して
`FnDef.is_test = true` として収集する。
既存の `test "description" { ... }` / `test_group` 構文とは共存する（非破壊的追加）。

---

## 調査結果（実装前に確認済み）

### 現状

- `FnDef` 構造体（`ast.rs:631`）には既に `deprecated: bool` / `api_annotation` などのアトリビュートフィールドがある
- `parse_deprecated_annotation()` がパターンとして存在（`parser.rs:282`）— `#[test]` も同様のルックアヘッドで実装する
- `FnDef { ... }` の構築は `parser.rs:1995` の 1 箇所のみ → `is_test: false` を追加すれば足りる
- `TestDef` / `TestGroup` は既存の `test "desc" { ... }` 構文用で別物 — 今回は触れない
- 自動アノテーション検出（checker/compiler の Item::FnDef マッチ）は bool フィールド追加のみなので既存パターンを壊さない

### `#[deprecated]` 解析パターン（参考）

実際の `peek()` は `&TokenKind`（Option なし）を返す（`parser.rs:284`）:

```rust
fn parse_deprecated_annotation(&mut self) -> Result<bool, ParseError> {
    if self.peek() == &TokenKind::Hash
        && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
        && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "deprecated"))
        && matches!(self.tokens.get(self.pos + 3), Some(t) if t.kind == TokenKind::RBracket)
    {
        self.advance(); // #
        self.advance(); // [
        self.advance(); // deprecated
        self.advance(); // ]
        Ok(true)
    } else {
        Ok(false)
    }
}
```

`#[test]` も同様のルックアヘッドで `parse_test_annotation()` を実装する。

### ロードマップとの設計上の差異

ロードマップ（roadmap-v46.1-v47.0.md line 37）は「`TestBlock` ノードとして収集」と記述しているが、
コードベース調査の結果、`FnDef.is_test: bool` フィールド追加で同等の機能を実現できることが分かった。
理由:
- `#[test]` fn は `fn` 構文の範囲であり、独立した AST ノードを追加するより `FnDef` を再利用する方が実装コストが低い
- `TestDef` / `TestGroup` との名前衝突リスクを避けられる
- v46.2.0 の `fav test` コマンドで `is_test == true` の `FnDef` を収集するだけで足りる

ロードマップの `TestBlock` 記述は本バージョン実装後に `FnDef.is_test` に更新する。

### 同時アノテーションの制限

`#[test]` と `#[deprecated]` の同時付与（例: `#[test] #[deprecated] fn f()`）は v46.1.0 スコープ外。
単独の `#[test]` のみをサポートする。

---

## 変更対象

### §1 — `ast.rs`: `FnDef` に `is_test: bool` 追加

`deprecated: bool` の直後に追加:

```rust
pub struct FnDef {
    pub visibility: Option<Visibility>,
    pub is_async: bool,
    pub name: String,
    pub type_params: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_ty: Option<TypeExpr>,
    pub body: Block,
    pub span: Span,
    pub api_annotation: Option<ApiAnnotation>,
    /// v24.4.0: `#[deprecated]` アノテーション付き関数
    pub deprecated: bool,
    /// v46.1.0: `#[test]` アノテーション付き関数
    pub is_test: bool,
}
```

### §2 — `parser.rs`: `parse_test_annotation()` 追加 + 適用

`parse_deprecated_annotation()` の直後（`parser.rs` 内）に追加:

```rust
/// v46.1.0: `#[test]` アトリビュートを認識して bool を返す。
fn parse_test_annotation(&mut self) -> Result<bool, ParseError> {
    if self.peek() == &TokenKind::Hash
        && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
        && matches!(self.tokens.get(self.pos + 2), Some(t) if t.kind == TokenKind::Test)
        && matches!(self.tokens.get(self.pos + 3), Some(t) if t.kind == TokenKind::RBracket)
    {
        self.advance(); // #
        self.advance(); // [
        self.advance(); // test
        self.advance(); // ]
        Ok(true)
    } else {
        Ok(false)
    }
}
```

`parse_item()` の `fn` 解析箇所（`deprecated_ann` 取得のすぐ後）で `test_ann` を取得し
`fd.is_test = test_ann;` を付与する。

### §3 — `parser.rs`: `FnDef { ... }` 構築に `is_test: false` 追加

`parser.rs:1995` の `FnDef { ... }` 構築に `is_test: false` を追加
（`deprecated: false` の隣に配置）。

### §4 — `driver.rs`: v461000_tests 追加

`v46000_tests` の直後に `v461000_tests` モジュールを追加（2件）:

```rust
#[cfg(test)]
mod v461000_tests {
    use crate::frontend::parser::Parser;
    use crate::ast::Item;

    #[test]
    fn test_block_parses() {
        let src = r#"
            #[test]
            fn test_add() {
                assert_eq(1, 1)
            }
        "#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        assert!(!prog.items.is_empty(), "should parse #[test] fn");
    }

    #[test]
    fn test_fn_collected() {
        let src = r#"
            #[test]
            fn test_add() {
                assert_eq(1, 1)
            }
        "#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let test_fn = prog.items.iter().find_map(|item| {
            if let Item::FnDef(fd) = item {
                if fd.is_test { Some(fd) } else { None }
            } else {
                None
            }
        });
        assert!(test_fn.is_some(), "should collect fn with is_test=true");
        assert_eq!(test_fn.unwrap().name, "test_add");
    }
}
```

---

## 変更しないファイル

- `checker.rs`: `is_test` bool フィールドは型チェックに影響しない
- `compiler.rs` / `codegen.rs` / `vm.rs`: 実行は v46.2.0 スコープ
- `error_catalog.rs`
- `examples/` 以下の .fav ファイル
- `site/` MDX: Developer Experience まとめドキュメントは v46.9.0 で追加

---

## 完了条件

> テスト数注記: ロードマップは v46.0 の閾値（≥ 2989）ベースで「2991」と推定しているが、
> 実際の v46.0.0 完了時点のテスト数は 2992。よって v46.1.0 完了時の推定は 2994。

- `cargo test` 全通過（failures=0、推定: 2992 + 2 = **2994** tests passed）
- `cargo clippy -- -D warnings` クリーン
- `v461000_tests` 2 件すべて pass（`test_block_parses` / `test_fn_collected`）
- `FnDef.is_test` が `#[test]` アノテーション付き fn で `true` になること
- `CHANGELOG.md` に v46.1.0 エントリ追加
- `versions/current.md` を v46.1.0（2994 tests）に更新
- `fav/Cargo.toml` version → `46.1.0`
