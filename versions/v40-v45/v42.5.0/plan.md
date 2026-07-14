# v42.5.0 実装計画 — Back-pressure `#[max_inflight]`

## 実装順序

---

## T0 — 事前確認

1. `cargo test` が 2886 tests / 0 failures であることを確認
2. `fav/Cargo.toml` version が `42.4.0` であることを確認
3. `ast.rs` の `CircuitBreakerAnnotation` 構造体末尾行番号を記録（`MaxInflightAnnotation` 挿入位置）
4. `ast.rs` の `TrfDef.circuit_breaker` フィールド行番号を記録（`max_inflight` フィールド挿入位置）
5. `ast.rs` の `parse_trf_def` 内 `circuit_breaker: None,` 行番号を記録
6. `parser.rs` の `parse_circuit_breaker_annotation` 関数末尾行番号を記録（`parse_max_inflight_annotation` 挿入位置）
7. `parser.rs` の `let circuit_breaker_ann = ...` 行番号を記録（呼び出し挿入位置）
8. `parser.rs` の `td.circuit_breaker = circuit_breaker_ann;` が出現する両行番号を記録（`td.max_inflight = ...` 挿入位置）
9. `driver.rs` の `v42400_tests` 閉じ `}` 行番号を記録（`v42500_tests` 挿入位置）
10. `versions/roadmap/roadmap-v42.1-v43.0.md` に v42.5.0 エントリが存在することを確認

---

## T1 — `ast.rs` — `MaxInflightAnnotation` 構造体追加

`CircuitBreakerAnnotation` の直後（SLA Annotations セクション末尾）に追加:

```rust
/// v42.5.0: `#[max_inflight(n)]` annotation on stage definitions.
/// n は同時処理上限（正の整数 ≥ 1）。runtime 強制は v44.x 以降。
#[derive(Debug, Clone)]
pub struct MaxInflightAnnotation {
    pub n: u64,
    pub span: Span,
}
```

---

## T2 — `ast.rs` — `TrfDef.max_inflight` フィールド追加

`circuit_breaker: Option<CircuitBreakerAnnotation>` の直後に追加:

```rust
/// v42.5.0: `#[max_inflight(n)]` annotation.
pub max_inflight: Option<MaxInflightAnnotation>,
```

---

## T3 — `parser.rs` — `parse_max_inflight_annotation()` 追加

`parse_circuit_breaker_annotation()` 関数の直後に追加:

```rust
/// v42.5.0: parse optional `#[max_inflight(n)]` annotation.
fn parse_max_inflight_annotation(&mut self) -> Result<Option<crate::ast::MaxInflightAnnotation>, ParseError> {
    let is_max_inflight = self.peek() == &TokenKind::Hash
        && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
        && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "max_inflight"));
    if !is_max_inflight {
        return Ok(None);
    }
    let start = self.peek_span().clone();
    self.advance();                              // #
    self.expect(&TokenKind::LBracket)?;          // [
    self.expect_ident_name("max_inflight")?;
    self.expect(&TokenKind::LParen)?;
    let n = match self.peek().clone() {
        TokenKind::Int(raw) => {
            let span = self.peek_span().clone();
            self.advance();
            if raw <= 0 {
                return Err(ParseError::new(
                    format!("max_inflight value {} must be a positive integer (>= 1)", raw),
                    span,
                ));
            }
            raw as u64
        }
        other => return Err(ParseError::new(
            format!("expected positive integer after `max_inflight(`, got {:?}", other),
            self.peek_span().clone(),
        )),
    };
    self.expect(&TokenKind::RParen)?;
    self.expect(&TokenKind::RBracket)?;
    Ok(Some(crate::ast::MaxInflightAnnotation { n, span: self.span_from(&start) }))
}
```

---

## T4 — `parser.rs` — `parse_item` に `parse_max_inflight_annotation` 呼び出し追加

`let circuit_breaker_ann = self.parse_circuit_breaker_annotation()?;` の直後:

```rust
let max_inflight_ann = self.parse_max_inflight_annotation()?; // v42.5.0
```

---

## T5 — `parser.rs` — `TokenKind::Stage` アームと `Async + Stage` アームに `td.max_inflight` 代入追加

`td.circuit_breaker = circuit_breaker_ann;` が出現する **2 か所**（同期 stage と async stage）の直後にそれぞれ:

```rust
td.max_inflight = max_inflight_ann;  // v42.5.0
```

---

## T6 — `parser.rs` — `parse_trf_def` の `TrfDef` 返却に `max_inflight: None` 追加

`circuit_breaker: None,` の直後:

```rust
max_inflight: None,
```

---

## T7 — `driver.rs` — `v42500_tests` モジュール追加

`v42400_tests` の閉じ `}` の直前（降順配置）に挿入。2 テスト:

### `cargo_toml_version_is_42_5_0`
```rust
// NOTE: この assert は次バージョン bump 時にスタブ化すること
#[test]
fn cargo_toml_version_is_42_5_0() {
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains("42.5.0"), "Cargo.toml version must be 42.5.0");
}
```

### `max_inflight_annotation_parses`
```rust
#[test]
fn max_inflight_annotation_parses() {
    let src = r#"
#[max_inflight(100)]
stage SlowSink: List -> List = |ctx| {
    ctx
}
"#;
    let program = crate::frontend::parser::Parser::parse_str(src, "max_inflight.fav")
        .expect("parse ok");
    let item = program.items.first().expect("one item");
    if let crate::ast::Item::TrfDef(td) = item {
        let ann = td.max_inflight.as_ref().expect("max_inflight annotation present");
        assert_eq!(ann.n, 100, "max_inflight n must be 100");
    } else {
        panic!("expected TrfDef, got {:?}", item);
    }
}
```

---

## T8 — Cargo.toml バージョン bump

```
version = "42.4.0"  →  version = "42.5.0"
```

---

## T9 — `CHANGELOG.md` 更新

`[v42.4.0]` エントリの直前に追加:

```markdown
## [v42.5.0] — 2026-07-12

### Added
- `#[max_inflight(n)]` アノテーション — `stage` 定義に back-pressure 宣言を追加
- `MaxInflightAnnotation { n: u64 }` AST 構造体追加（`TrfDef.max_inflight` フィールド）
- `parse_max_inflight_annotation()` パーサー実装（n <= 0 はパース時エラー）
- `v42500_tests`: `max_inflight_annotation_parses`
```

---

## T10 — テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

- failures = 0
- テスト数 = 2888（2886 + 2）
- `v42500_tests` 2 件 pass

---

## T11 — バージョン管理ドキュメント更新

1. `versions/current.md` を v42.5.0（最新安定版）・v42.6.0（次に切る版）に更新
2. `versions/roadmap/roadmap-v42.1-v43.0.md` の v42.5.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）`）
3. `versions/v40-v45/v42.5.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）

---

## 変更ファイルサマリー

| ファイル | 変更 |
|---|---|
| `fav/src/ast.rs` | `MaxInflightAnnotation` 構造体、`TrfDef.max_inflight` フィールド |
| `fav/src/frontend/parser.rs` | `parse_max_inflight_annotation()`、`parse_item` 呼び出し（2 か所）、`parse_trf_def` デフォルト |
| `fav/src/driver.rs` | `v42500_tests` 2 件 |
| `fav/Cargo.toml` | `42.4.0` → `42.5.0` |
| `CHANGELOG.md` | `[v42.5.0]` エントリ |
| `versions/current.md` | 最新安定版・次版更新 |
