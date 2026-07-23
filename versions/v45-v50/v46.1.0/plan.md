# Plan: v46.1.0 — `#[test]` ブロック AST + parser

Date: 2026-07-16
Status: TODO

---

## ステップ

### Step 1 — 事前確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

2992 tests passed, 0 failed を確認。

---

### Step 2 — `ast.rs`: `FnDef` に `is_test: bool` 追加

`FnDef` 構造体の `deprecated: bool,` の直後に追加:

```rust
/// v46.1.0: `#[test]` アノテーション付き関数
pub is_test: bool,
```

---

### Step 3 — `parser.rs`: `parse_test_annotation()` 追加

`parse_deprecated_annotation()` の直後に以下を追加:

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

---

### Step 4 — `parser.rs`: `parse_item()` で `test_ann` を取得・付与

`parse_item()` 内の deprecated 処理（`let deprecated_ann = self.parse_deprecated_annotation()?;`）の
**直後** に `test_ann` 取得を追加:

```rust
let deprecated_ann = self.parse_deprecated_annotation()?;
// v46.1.0: #[test] annotation
let test_ann = self.parse_test_annotation()?;
```

そして `fd.deprecated = deprecated_ann;` の直後（2箇所）に:

```rust
fd.is_test = test_ann;
```

を追加する（`Item::FnDef` として返す 2 箇所、`public fn` と非 public fn の両方）。

---

### Step 5 — `parser.rs`: `FnDef { ... }` 構築に `is_test: false` 追加

`parse_fn_def()` 内の `Ok(FnDef { ... })` 構築（`parser.rs:1995` 付近）に:

```rust
is_test: false,  // v46.1.0: set by parse_item if #[test] precedes the fn
```

を追加する（`deprecated: false` の隣に配置）。

---

### Step 6 — `driver.rs`: v461000_tests 追加

`grep -n "v46000_tests" src/driver.rs` でモジュール終端行を確認し、
直後に `v461000_tests` モジュールを追加（2件）:
- `test_block_parses`
- `test_fn_collected`

---

### Step 7 — ビルド確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo build 2>&1 | tail -5
```

`FnDef` フィールド追加によるコンパイルエラーがないことを確認。
（既存コードは `fd.is_test` に依存していないため影響なし）

---

### Step 8 — テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

2994 passed（2992 + 2件）, 0 failed を確認。

---

### Step 9 — Clippy

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -5
```

---

### Step 10 — バージョン・ドキュメント更新

1. `fav/Cargo.toml`: `version = "46.1.0"`
2. `CHANGELOG.md`: v46.1.0 エントリ追加
3. `versions/current.md`: v46.1.0（2994 tests）に更新
4. `versions/v45-v50/v46.1.0/tasks.md`: COMPLETE に更新

---

## 実装順序まとめ

```
Step 1:  cargo test（事前確認: 2992 tests）
Step 2:  ast.rs — FnDef.is_test: bool 追加
Step 3:  parser.rs — parse_test_annotation() 追加
Step 4:  parser.rs — parse_item() で test_ann 取得・付与
Step 5:  parser.rs — FnDef 構築に is_test: false 追加
Step 6:  driver.rs — v461000_tests 追加（2件）
Step 7:  cargo build（コンパイルエラー確認）
Step 8:  cargo test（2994 pass 確認）
Step 9:  cargo clippy
Step 10: バージョン・ドキュメント更新
```
