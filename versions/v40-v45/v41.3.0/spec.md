# v41.3.0 Spec — タプルパターン match

**バージョン**: v41.3.0
**テーマ**: `match (a, b) { (p1, p2) -> ... }` 構文をパーサーでサポート
**前バージョン**: v41.2.0（Refinement type E0404 系）
**目標テスト数**: 2856（前バージョン 2853 + 3）

---

## 概要

Favnir に `(a, b)` タプル式と `(p1, p2)` タプルパターンを追加する。

**設計方針: パース時デシュガー**
新しい AST ノード（`Expr::Tuple` / `Pattern::Tuple`）を追加せず、
パーサーで既存 AST ノードにデシュガーする。これにより checker.rs・compiler.rs・
fmt.rs・emit_python.rs など全 exhaustive match への影響をゼロにする。

| 構文 | デシュガー後 |
|---|---|
| `(a, b)` 式 | `RecordConstruct("__tuple__", [("_0", a), ("_1", b)])` |
| `(p1, p2)` パターン | `Record([Alias("_0", p1), Alias("_1", p2)])` |
| `(a, b, c)` 式 | `RecordConstruct("__tuple__", [("_0", a), ("_1", b), ("_2", c)])` |

**v41.3.0 スコープ**:
- パーサー対応のみ（`Parser::parse_str` がエラーなく通ること）
- `fav check` での型チェック統合はスコープ外：
  - `RecordConstruct("__tuple__", ...)` は Rust checker.rs で E0102（undefined type `__tuple__`）を出す
  - `Pattern::Record` は `ast_lower_checker.rs` で `PWild`（ワイルドカード）に変換されるため、checker.fav での詳細な型チェックは行われない
  - これらは既知の制限として記録し、v41.4.0 以降で対応

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/frontend/parser.rs` | 式・パターンの `LParen` 分岐にタプルデシュガーを追加 |
| `fav/self/checker.fav` | タプルパターン処理の設計コメント追加（機能変更なし） |
| `fav/src/driver.rs` | `v41300_tests` 追加（3 件）、`v41200_tests::cargo_toml_version_is_41_2_0` スタブ化 |
| `fav/Cargo.toml` | `version = "41.3.0"` に bump |
| `CHANGELOG.md` | `[v41.3.0]` エントリ追加 |

---

## 詳細仕様

### 1. parser.rs — 式側 LParen 分岐

現在の `LParen` 式分岐（`(expr)` はグルーピング、`()` は unit）に追加:

```rust
TokenKind::LParen => {
    self.advance();
    if self.peek() == &TokenKind::RParen {
        self.advance();
        Ok(Expr::Lit(Lit::Unit, self.span_from(&start)))
    } else {
        let first = self.parse_expr()?;
        if self.peek() == &TokenKind::Comma {
            // tuple: (a, b, ...) → RecordConstruct("__tuple__", [("_0", a), ...])
            let mut fields = vec![("_0".to_string(), first)];
            let mut i = 1usize;
            while self.peek() == &TokenKind::Comma {
                self.advance();
                if self.peek() == &TokenKind::RParen { break; } // trailing comma
                fields.push((format!("_{}", i), self.parse_expr()?));
                i += 1;
            }
            self.expect(&TokenKind::RParen)?;
            Ok(Expr::RecordConstruct("__tuple__".to_string(), fields, self.span_from(&start)))
        } else {
            self.expect(&TokenKind::RParen)?;
            Ok(first) // grouping parens: (expr) → expr
        }
    }
}
```

### 2. parser.rs — パターン側 LParen 分岐

現在の `LParen` パターン分岐（`()` unit のみ対応。`(p)` は現状パースエラー）を拡張:

```rust
TokenKind::LParen => {
    self.advance();
    if self.peek() == &TokenKind::RParen {
        self.advance();
        Ok(Pattern::Lit(Lit::Unit, self.span_from(&start)))
    } else {
        let first = self.parse_pattern()?;
        if self.peek() == &TokenKind::Comma {
            // tuple pattern: (p1, p2, ...) → Record([Alias("_0", p1), ...])
            let mut fields = vec![PatternField::Alias(
                "_0".to_string(), Box::new(first), self.span_from(&start),
            )];
            let mut i = 1usize;
            while self.peek() == &TokenKind::Comma {
                self.advance();
                if self.peek() == &TokenKind::RParen { break; }
                fields.push(PatternField::Alias(
                    format!("_{}", i),
                    Box::new(self.parse_pattern()?),
                    self.span_from(&start),
                ));
                i += 1;
            }
            self.expect(&TokenKind::RParen)?;
            Ok(Pattern::Record(fields, self.span_from(&start)))
        } else {
            self.expect(&TokenKind::RParen)?;
            Ok(first) // grouping parens: (pat) → pat
        }
    }
}
```

**注意**: 各フィールドのスパンはタプル全体の開始位置 `start` を指す。これは既存の
`Variant::Tuple` 処理（parser.rs 行 ~2603）と同一の慣行。

### 3. checker.fav — 設計コメント追加

ファイル末尾に追加:

```favnir
// v41.3.0: タプルパターン (p1, p2) の処理設計ノート
// - パーサーで Pattern::Record([Alias("_0", p1), Alias("_1", p2)]) にデシュガーされる
// - ast_lower_checker.rs でこれが PWild に変換されるため、checker.fav での
//   詳細な型チェックは行われない（v41.4.0 以降で PRecord バリアント追加予定）
// - Rust checker.rs は Pattern::Record を正しく処理する
```

---

## テスト設計（v41300_tests）

`use super::*` 不要。

### T1: `cargo_toml_version_is_41_3_0`（NOTE コメント付き）
```rust
#[test]
fn cargo_toml_version_is_41_3_0() {
    // NOTE: この assert は次バージョン bump 時にスタブ化すること
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains("41.3.0"), "Cargo.toml must contain version 41.3.0");
}
```

### T2: `changelog_has_v41_3_0`
```rust
#[test]
fn changelog_has_v41_3_0() {
    let src = include_str!("../../CHANGELOG.md");
    assert!(src.contains("[v41.3.0]"), "CHANGELOG.md must contain [v41.3.0]");
}
```

### T3: `tuple_pattern_match_parseable`
```rust
#[test]
fn tuple_pattern_match_parseable() {
    use crate::frontend::parser::Parser;
    let src = r#"fn f() -> String { match ("ok", 1) { ("ok", 1) => "yes" _ => "no" } }"#;
    let result = Parser::parse_str(src, "test.fav");
    assert!(result.is_ok(), "Tuple pattern match should parse without error: {:?}", result.err());
}
```

---

## 完了条件

- `cargo test` が 2856 tests passed, 0 failed
- `v41300_tests` 3 件すべて pass
- `match ("ok", 1) { ("ok", 1) -> ... _ -> ... }` がパースエラーなし
- `(a, b)` 式が `RecordConstruct("__tuple__", ...)` にデシュガーされる
- `(p1, p2)` パターンが `Pattern::Record` にデシュガーされる

---

## 設計ノート

- `"__tuple__"` 型名は checker.rs で `E0102: undefined type` を出す（既知の制限 → v41.4.0 以降で対応）
- `Pattern::Record` は `ast_lower_checker.rs` で `PWild` に変換（`fav check` での詳細チェックはスコープ外）
- `ast.rs` に `Variant::Tuple(String, Vec<TypeExpr>, Span)` がすでに存在するが、これは型定義のバリアントで `Pattern` とは別の enum。本バージョンでは `Pattern::Tuple` を追加しないため衝突なし
- 単一要素 `(a)` はグルーピング括弧として `a` を返す
- 末尾カンマ `(a, b,)` は許容
- `()` は unit リテラルとして既存動作維持
- ロードマップ推定テスト数 2849 は v41.2.0 前の古い基準値による。実際は 2853 + 3 = 2856 が正しい
