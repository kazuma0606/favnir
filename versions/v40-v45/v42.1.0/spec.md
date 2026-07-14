# v42.1.0 仕様書 — CEP DSL 基盤

**フェーズ**: Real-Time Power（v42.x スプリント）
**前バージョン**: v42.0.0（Type Precision 宣言、2874 tests）
**目標テスト数**: 2877（+3）

> **注**: ロードマップは「推定 2870 tests」と記載しているが、v42.0.0 完了時点の実績が 2874 のため、2874 + 3 = 2877 を実装目標とする。

---

## 概要

Complex Event Processing（CEP）の構文・型・VM サポート基盤を追加する。
`cep pattern` ブロックを parser・AST・checker に追加し、以下の構文を受理できるようにする:

```favnir
cep pattern LoginEvent {
  Login within 60
}
```

**本バージョンのスコープ**: parser + AST のみ。型チェック統合は v42.3.0。

---

## 現状確認

| ファイル | 状態 |
|---|---|
| `fav/src/ast.rs` | `CepPatternDef` 未定義 |
| `fav/src/frontend/parser.rs` | `cep` キーワード未対応 |
| `fav/src/middle/checker.rs` | `CepPatternDef` 未対応 |
| `fav/src/fmt.rs` | `CepPatternDef` 未対応（exhaustive match） |
| `fav/src/driver.rs` | `CepPatternDef` 未対応（exhaustive match） |
| `fav/self/checker.fav` | CEP 設計コメント未追加 |

---

## スコープ

### v42.1.0 に含む

1. **`fav/src/ast.rs`**
   - `CepClause` 構造体追加
   - `CepPatternDef` 構造体追加
   - `Item::CepPatternDef(CepPatternDef)` バリアント追加
   - `Item::span()` の exhaustive match に arm 追加

2. **`fav/src/frontend/parser.rs`**
   - `parse_cep_pattern_def()` 関数追加
   - `parse_item()` に `"cep"` キーワードディスパッチ追加

3. **`fav/src/middle/checker.rs`**
   - 2 箇所のスタブ追加（型チェックは v42.3.0）

4. **`fav/src/fmt.rs`**
   - exhaustive match に `Item::CepPatternDef` スタブ arm 追加

5. **`fav/src/driver.rs`**
   - exhaustive match に `Item::CepPatternDef` スタブ arm 追加（行 13898 付近）

6. **`fav/self/checker.fav`**
   - CEP 型チェック設計コメント追加（v42.3.0 向け）

7. **`fav/src/driver.rs` テスト**（3 件）

### スコープ外

- CEP パターンコンビネータ（`seq`/`any`/`not`）: v42.2.0
- CEP 型チェック統合: v42.3.0
- VM opcode 追加（実行エンジン対応は v42.3.0 以降）
- `exhaustive-match-checker` 対象の他ファイル（`lineage.rs`/`lint.rs`/`emit_python.rs`）は `if let` 使用のため更新不要

---

## AST 設計

```rust
// ── CepPatternDef (v42.1.0) ──────────────────────────────────────────────────

/// 単一イベント節: `Login within 60`
#[derive(Debug, Clone)]
pub struct CepClause {
    pub event: String,            // イベント名 ("Login")
    pub within_secs: Option<i64>, // `within N` 秒 (Some(60) or None)
    pub span: Span,
}

/// `cep pattern Name { clause... }` — CEP パターン宣言 (v42.1.0)
#[derive(Debug, Clone)]
pub struct CepPatternDef {
    pub name: String,
    pub body: Vec<CepClause>,
    pub span: Span,
}
```

`Item` への追加:
```rust
/// `cep pattern Name { ... }` — CEP パターン宣言 (v42.1.0)
CepPatternDef(CepPatternDef),
```

`Item::span()` への追加:
```rust
Item::CepPatternDef(c) => &c.span,
```

---

## パーサー設計

`parse_item()` に `"cep"` キーワードを追加:

```rust
TokenKind::Ident(n) if n == "cep" => {
    Ok(Item::CepPatternDef(self.parse_cep_pattern_def()?))
}
```

`parse_cep_pattern_def()` の手順:
1. `cep` を consume（`advance()`）
2. `expect_ident_name("pattern")` で `pattern` を consume
3. `expect_ident()` で name を parse
4. `expect(&TokenKind::LBrace)` で `{` を consume
5. ループ: `peek() != &TokenKind::RBrace && peek() != &TokenKind::Eof`
   - `expect_ident()` で event name を parse
   - `peek_ident_text("within")` が true なら: `advance()` で `within` を consume → `peek().clone()` で `TokenKind::Int(n)` を確認 → `advance()` で consume → `within_secs: Some(n)`
   - `span_from(&clause_start)` で CepClause の span を生成
6. `expect(&TokenKind::RBrace)` で `}` を consume
7. `span_from(&start)` で CepPatternDef の span を生成

エラーパス: `pattern` キーワードがない / `within` 後に整数がない場合は `ParseError`。

**パーサー API の実態（確認済み）:**
- `peek()` → `&TokenKind`（`Some` ラップなし）
- `advance()` → `&Token`（`Option` ではない）
- `span_from(&start)` で Span 生成（`Span::merge()` は存在しない）

---

## checker.rs スタブ設計

**Pass 1（グローバルシンボル収集）** — line 2368 付近:
```rust
| Item::SchemaDef(..)
| Item::CepPatternDef(..) => {} // v42.1.0: スタブ（型チェックは v42.3.0）
```

**Pass 2（型チェック）** — line 2411 付近:
```rust
Item::SchemaDef(_) => {} // v36.1.0: 型チェックは v36.2 以降
Item::CepPatternDef(_) => {} // v42.1.0: 型チェックは v42.3.0
```

---

## fmt.rs スタブ設計

exhaustive match の末尾（`Item::SchemaDef` の直後）に追加:
```rust
Item::CepPatternDef(cd) => format!("cep pattern {} {{ ... }}", cd.name), // v42.1.0: fmt スタブ
```

---

## driver.rs スタブ設計

line 13898 付近（`Item::SchemaDef(..) => {}` の直後）に追加:
```rust
Item::CepPatternDef(..) => {} // v42.1.0: スタブ
```

---

## checker.fav 設計コメント

`checker.fav` の末尾（または適切な箇所）に追加:
```
// ── CEP パターン型チェック（v42.3.0 以降）─────────────────────────────────────
// v42.1.0 では CepPatternDef は AST ノードとしてパースのみ。
// v42.3.0 で以下を実装予定:
//   - pattern ブロック内のイベント名が型環境に存在するか検証
//   - within_secs が正の整数か検証
//   - E0420: CEP パターンの型不一致エラー
```

---

## テスト設計（3 件）

```rust
// -- v42100_tests (v42.1.0) -- CEP DSL 基盤 --
#[cfg(test)]
mod v42100_tests {
    #[test]
    fn cargo_toml_version_is_42_1_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("42.1.0"), "Cargo.toml must contain version 42.1.0");
    }

    #[test]
    fn cep_pattern_parseable() {
        use crate::frontend::parser::Parser;
        let src = r#"cep pattern LoginEvent { Login within 60 }"#;
        let result = Parser::parse_str(src, "test.fav");
        assert!(result.is_ok(), "cep pattern should parse without error: {:?}", result.err());
    }

    #[test]
    fn cep_pattern_fields_correct() {
        use crate::frontend::parser::Parser;
        use crate::ast::Item;
        let src = r#"cep pattern LoginEvent { Login within 60 }"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse ok");
        let Item::CepPatternDef(ref cd) = prog.items[0] else {
            panic!("expected CepPatternDef");
        };
        assert_eq!(cd.name, "LoginEvent");
        assert_eq!(cd.body.len(), 1);
        assert_eq!(cd.body[0].event, "Login");
        assert_eq!(cd.body[0].within_secs, Some(60));
    }
}
```

---

## 既存コードへの影響

| ファイル | 変更 | 規模 |
|---|---|---|
| `fav/src/ast.rs` | `CepClause` + `CepPatternDef` 構造体追加、`Item::CepPatternDef` バリアント追加、`span()` arm 追加 | 小（約 20 行） |
| `fav/src/frontend/parser.rs` | `parse_cep_pattern_def()` 追加、`parse_item()` dispatch 追加 | 中（約 40 行） |
| `fav/src/middle/checker.rs` | スタブ 2 箇所 | 極小（2 行） |
| `fav/src/fmt.rs` | スタブ arm 1 行 | 極小 |
| `fav/src/driver.rs` | exhaustive match スタブ 1 行 + テスト 3 件追加 | 小 |
| `fav/self/checker.fav` | 設計コメント追加 | 極小 |
| `fav/Cargo.toml` | version: `42.0.0` → `42.1.0` | 1 行 |
| `CHANGELOG.md` | `[v42.1.0]` エントリ追加 | 数行 |

---

## 完了条件

### 自動検証（cargo test）

- `cargo test` 全通過（2877 tests passed, 0 failed）
- `v42100_tests::cargo_toml_version_is_42_1_0` pass
- `v42100_tests::cep_pattern_parseable` pass
- `v42100_tests::cep_pattern_fields_correct` pass

### 実装者による手動確認

- `cep pattern LoginEvent { Login within 60 }` が `fav check` でエラーなしに処理される（型チェックスタブ）
- `fav fmt` が `cep pattern` を含むファイルでクラッシュしない
