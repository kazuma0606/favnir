# v42.2.0 仕様書 — CEP パターン: `seq` / `any` / `not`

## 概要

v42.1.0 で導入した `cep pattern` 構文にパターンコンビネータ（`seq` / `any` / `not`）を追加する。
CEP 節のボディを単純なイベント名文字列から `CepExpr` 型に昇格させ、複合パターンを表現できるようにする。

---

## 背景・動機

v42.1.0 の `CepClause` は `event: String` フィールドを持ち、単一イベント名のみ記述できた。
実世界の CEP ユースケース（「Login の後 300 秒以内に Purchase」「DiskFull または OOM または NetworkDown」）
を表現するには複合パターンが必要。

---

## 実装スコープ

### 1. `ast.rs` — `CepExpr` 追加・`CepClause` 変更

```rust
pub enum CepExpr {
    Event(String),          // 単純イベント名（従来の event: String の後継）
    Seq(Vec<CepExpr>),      // seq(E1, E2, ...)
    Any(Vec<CepExpr>),      // any(E1, E2, ...)
    Not(Box<CepExpr>),      // not(E)
}

pub struct CepClause {
    pub expr: CepExpr,          // event: String → expr: CepExpr に変更
    pub within_secs: Option<i64>,
    pub span: Span,
}
```

`CepExpr` には `#[derive(Debug, Clone)]` を付与する。

### 2. `parser.rs` — `parse_cep_expr()` 追加・`parse_cep_pattern_def()` 修正

**`parse_cep_expr()`**（`parse_cep_pattern_def()` の直前に追加）:
- `seq` ident → `(` → カンマ区切りで再帰 `parse_cep_expr()` → `)` → `CepExpr::Seq`
- `any` ident → `(` → カンマ区切りで再帰 `parse_cep_expr()` → `)` → `CepExpr::Any`
- `not` ident → `(` → 単一 `parse_cep_expr()` → `)` → `CepExpr::Not`
- それ以外 → `expect_ident()` → `CepExpr::Event(name)`

**`parse_cep_pattern_def()` 修正**:
- 節ループ内の `let (event, _) = self.expect_ident()?;` を `let expr = self.parse_cep_expr()?;` に置き換え
- `CepClause { event, within_secs, span }` → `CepClause { expr, within_secs, span }`

正しいパーサー API（確認済み）:
- `self.peek_ident_text("seq")` → `bool`
- `self.advance()` → `&Token`（戻り値は破棄）
- `self.expect(&TokenKind::LParen)?` / `self.expect(&TokenKind::RParen)?`
- `self.peek() == &TokenKind::Comma` で確認後 `self.advance()`
- `self.peek() == &TokenKind::RParen` または `== &TokenKind::Eof` でループ終了

### 3. `driver.rs` — 既存テスト更新 + `v42200_tests` 追加

**`v42100_tests::cep_pattern_fields_correct` 更新**（v42.2.0 で AST 変更のため必須）:
```rust
// 変更前: assert_eq!(cd.body[0].event, "Login");
// 変更後:
let crate::ast::CepExpr::Event(ref ev) = cd.body[0].expr else {
    panic!("expected CepExpr::Event");
};
assert_eq!(ev, "Login");
```

**`v42100_tests::cargo_toml_version_is_42_1_0` スタブ化**。

**`v42200_tests`**（3 テスト、`v42100_tests` の直前に挿入）:
- `cargo_toml_version_is_42_2_0`（NOTE コメント付き）
- `cep_seq_parseable` — `cep pattern LoginThenPurchase { seq(Login, Purchase) within 300 }` がパースでき、`body[0].expr` が `CepExpr::Seq` であることを確認
- `cep_any_parseable` — `cep pattern AnyAlert { any(DiskFull, OOM, NetworkDown) }` がパースでき、`body[0].expr` が `CepExpr::Any(len=3)` であることを確認

### 4. `fmt.rs` — 変更なし

現在のスタブ `format!("cep pattern {} {{ ... }}", cd.name)` は `CepClause.event` を参照していないため変更不要。

### 5. `checker.rs` / `checker.fav` / `lint.rs` — 変更なし

CepPatternDef スタブはボディをイテレートしないため変更不要。
v42.3.0 で型チェック本実装時に対応。

---

## テスト計画

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_42_2_0` | Cargo.toml に "42.2.0" が含まれる |
| `cep_seq_parseable` | `seq(Login, Purchase) within 300` パース成功 + `CepExpr::Seq` 確認 |
| `cep_any_parseable` | `any(DiskFull, OOM, NetworkDown)` パース成功 + `CepExpr::Any` 長さ 3 確認 |
| `not` パターン | 構文定義（`CepExpr::Not`）は追加するがテストは v42.3.0 でカバー。本バージョンは非スコープ |

**推定テスト数**: 2877 + 3 = **2880**（既存 v42100 テストは修正のみで数は変わらない）

---

## 影響範囲

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/ast.rs` | 変更 | `CepExpr` enum 追加、`CepClause.event` → `.expr` |
| `fav/src/frontend/parser.rs` | 変更 | `parse_cep_expr()` 追加、`parse_cep_pattern_def()` 修正 |
| `fav/src/driver.rs` | 変更 | `cep_pattern_fields_correct` 更新、`v42200_tests` 追加 |
| `fav/Cargo.toml` | 変更 | version `42.1.0` → `42.2.0` |
| `CHANGELOG.md` | 変更 | `[v42.2.0]` エントリ追加 |
| `fav/src/fmt.rs` / `checker.rs` / `lint.rs` | 変更なし | `CepClause.event` フィールドを参照するコードが存在しないことを T0 で確認済み |

---

## 非スコープ

- `not` パターンのパーステスト（v42.2.0 では `seq` / `any` のみテスト。`not` の構文定義は追加するがテストなし）
- CEP 型チェック（v42.3.0）
- VM 実行サポート（v44.x 以降）
