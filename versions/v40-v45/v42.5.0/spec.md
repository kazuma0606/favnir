# v42.5.0 仕様書 — Back-pressure `#[max_inflight]`

## 概要

`#[max_inflight(n)]` アノテーションを `stage` 定義に追加する。
パーサーで構文解析し、AST の `TrfDef` に格納する。
実際の runtime back-pressure（上流ステージ一時停止）は v44.x 以降（スケジューラー整備後）に実装予定。

---

## 背景・動機

v42.4.0 で Stream join を追加した。リアルタイムパイプラインでは、低速な sink ステージが高速な source に追い付けない「back-pressure」問題が発生する。
`#[max_inflight(n)]` アノテーションはステージが同時に処理する最大リクエスト数を宣言し、将来の runtime スケジューラーによる流量制御を可能にする。

---

## Favnir 構文（実装する形式）

ロードマップの `@max_inflight(100)` は概念表記。Favnir の既存アノテーション規則（`#[name(...)]` 形式）に合わせて実装する:

```favnir
#[max_inflight(100)]
stage SlowSink: Rows -> Unit = |ctx, rows| {
  bind _ <- Db.batch_insert(ctx, rows)
}
```

引数: `n` — 同時処理上限（正の整数 ≥ 1）。0 以下はパーサーレベルでエラー。

**アノテーション構文の位置引数について**: 既存の `#[timeout(seconds = N)]` / `#[retry(max = N, backoff = "...")]` は名前付き引数形式。`#[max_inflight(n)]` は引数が 1 つだけのため名前を省略した位置引数形式を採用する。将来的に引数が増える場合は名前付き形式に移行する。

---

## 実装スコープ

### 1. `ast.rs` — `MaxInflightAnnotation` 構造体追加

`CircuitBreakerAnnotation` の直後（SLA Annotations セクション末尾）に追加:

```rust
/// v42.5.0: `#[max_inflight(n)]` annotation on stage definitions.
/// n は同時処理上限（正の整数）。runtime 強制は v44.x 以降。
#[derive(Debug, Clone)]
pub struct MaxInflightAnnotation {
    pub n: u64,
    pub span: Span,
}
```

### 2. `ast.rs` — `TrfDef` に `max_inflight` フィールド追加

`circuit_breaker` フィールドの直後に追加:

```rust
/// v42.5.0: `#[max_inflight(n)]` annotation.
pub max_inflight: Option<MaxInflightAnnotation>,
```

### 3. `parser.rs` — `parse_max_inflight_annotation()` 追加

`parse_circuit_breaker_annotation()` の直後に追加:

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

### 4. `parser.rs` — `parse_item` に呼び出し追加

`let circuit_breaker_ann = ...` の直後（line 611 相当）に追加:

```rust
let max_inflight_ann = self.parse_max_inflight_annotation()?; // v42.5.0
```

`TokenKind::Stage` アーム（line 643 相当）と `TokenKind::Async + Stage` アーム（line 670 相当）の両方に追加:

```rust
td.max_inflight = max_inflight_ann; // v42.5.0
```

### 5. `parser.rs` — `parse_trf_def` 戻り値に `max_inflight: None` 追加

`circuit_breaker: None,` の直後に:

```rust
max_inflight: None,
```

### 6. `driver.rs` — `v42500_tests` 追加（2 テスト）

```rust
// -- v42500_tests (v42.5.0) -- Back-pressure #[max_inflight] --
mod v42500_tests {
    fn cargo_toml_version_is_42_5_0()
    fn max_inflight_annotation_parses()  // #[max_inflight(100)] stage が parse 成功、n=100 を確認
}
```

`max_inflight_annotation_parses`:
```rust
let src = r#"
#[max_inflight(100)]
stage SlowSink: List -> List = |ctx| {
    ctx
}
// `List` 単独（型引数なし）はパーサーで受け付けられる（型チェック段階で検証）
"#;
// Parser::parse_str → Item::TrfDef → max_inflight = Some(MaxInflightAnnotation { n: 100 })
```

---

## テスト計画

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_42_5_0` | Cargo.toml に "42.5.0" が含まれる |
| `max_inflight_annotation_parses` | `#[max_inflight(100)]` stage が parse 成功、`max_inflight.n == 100` を確認 |

**推定テスト数**: 2886 + 2 = **2888**

---

## 影響範囲

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/ast.rs` | 変更 | `MaxInflightAnnotation` 構造体追加、`TrfDef.max_inflight` フィールド追加 |
| `fav/src/frontend/parser.rs` | 変更 | `parse_max_inflight_annotation()` + 呼び出し + `TrfDef` デフォルト |
| `fav/src/driver.rs` | 変更 | `v42500_tests` 2 件追加 |
| `fav/Cargo.toml` | 変更 | version `42.4.0` → `42.5.0` |
| `CHANGELOG.md` | 変更 | `[v42.5.0]` エントリ追加 |
| `versions/current.md` | 変更 | 最新安定版 v42.5.0・次版 v42.6.0 に更新 |

---

## 非スコープ

- runtime back-pressure（上流ステージ一時停止）— VM にスケジューラー未実装のため v44.x 以降
- checker.rs でのセマンティクス検証（n > 0 はパーサーレベルで保証済み）
- `@max_inflight(...)` 構文（ロードマップ表記は概念表記；Favnir では `#[max_inflight(...)]`）
- `AbstractTrfDef`（abstract stage）への `max_inflight` 追加（具体 stage のみ対象）
- `fmt.rs` / `emit_python.rs` の `TrfDef` 出力部分への追記（新フィールドは `None` のままフォールスルー）
  - 注意: `fav fmt` を実行すると `#[max_inflight(n)]` アノテーションが出力から消える（ラウンドトリップが破損する）。v44.x の runtime 実装時に fmt.rs も合わせて対応予定。
