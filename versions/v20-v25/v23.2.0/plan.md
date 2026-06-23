# v23.2.0 — ビット演算 実装プラン

## 実装順序

```
T1（lexer.rs）  ← 16 進数リテラル（独立）
T2（vm.rs）     ← T1 完了後（hex リテラルをテストで使うため）
T3（checker.rs）← T2 と並列可
T4（driver.rs） ← T2〜T3 完了後、T5 より前に #[ignore] を実施
T5（docs）      ← T4 完了後（Cargo.toml 更新は #[ignore] 後）
```

---

## T1: `fav/src/frontend/lexer.rs` — 16 進数リテラル

### 事前確認

```bash
grep -n "lex_number\|'0'..='9'\|is_ascii_digit" fav/src/frontend/lexer.rs | head -10
```

### 修正内容

`lex_number()` の先頭に `0x`/`0X` チェックを追加する。

**lexer の呼び出し構造（確認済み）:**
`next_token()` は `match c` で文字を peek するが advance しない。`lex_number()` 入場時、最初の `'0'` はまだ pos にある（consume 済みではない）。よって `self.peek() == Some('0')` でチェックできる。

**修正後（概略）:**

```rust
fn lex_number(&mut self) -> Result<TokenKind, LexError> {
    // 16 進数リテラル（0x / 0X）
    // next_token() は advance しないため、入場時に '0' は pos にある
    if self.peek() == Some('0')
        && matches!(self.peek2(), Some('x') | Some('X'))
    {
        self.advance(); // '0'
        self.advance(); // 'x' or 'X'
        let mut hex = String::new();
        while self
            .peek()
            .map(|c| c.is_ascii_hexdigit())
            .unwrap_or(false)
        {
            hex.push(self.advance());
        }
        if hex.is_empty() {
            return Err(LexError::new(
                "invalid hex literal: expected hex digits after '0x'",
                self.span_here(),
            ));
        }
        return i64::from_str_radix(&hex, 16)
            .map(TokenKind::Int)
            .map_err(|_| {
                LexError::new(
                    format!("hex literal '0x{}' overflows i64", hex),
                    self.span_here(),
                )
            });
    }

    // 以下は既存のコード（10 進 / float）
    let mut s = String::new();
    let mut is_float = false;
    // ...
```

---

## T2: `fav/src/backend/vm.rs` — vm_call_builtin に 6 アーム追加

### 事前確認

```bash
grep -n '"Int\.bnot"\|"Int\.bxor"' fav/src/backend/vm.rs | head -5
```

### 追加コード

`"Int.bnot"` ブロックの直後に下記を追加する。

**重要**: 既存の `Int.band` / `Int.shl` 等は `args.into_iter()` + `.next()` を使う（`pop()` ではない）。同じパターンに従うこと。シフト量の範囲チェック（`0..=63`）を追加してパニックを防ぐ。

```rust
// v23.2.0: public API names for bit operations
"Int.bit_and" => {
    let mut it = args.into_iter();
    let x = it.next().ok_or_else(|| "Int.bit_and requires 2 arguments".to_string())?;
    let y = it.next().ok_or_else(|| "Int.bit_and requires 2 arguments".to_string())?;
    match (x, y) {
        (VMValue::Int(x), VMValue::Int(y)) => Ok(VMValue::Int(x & y)),
        _ => Err("Int.bit_and requires two Int arguments".to_string()),
    }
}
"Int.bit_or" => {
    let mut it = args.into_iter();
    let x = it.next().ok_or_else(|| "Int.bit_or requires 2 arguments".to_string())?;
    let y = it.next().ok_or_else(|| "Int.bit_or requires 2 arguments".to_string())?;
    match (x, y) {
        (VMValue::Int(x), VMValue::Int(y)) => Ok(VMValue::Int(x | y)),
        _ => Err("Int.bit_or requires two Int arguments".to_string()),
    }
}
"Int.bit_xor" => {
    let mut it = args.into_iter();
    let x = it.next().ok_or_else(|| "Int.bit_xor requires 2 arguments".to_string())?;
    let y = it.next().ok_or_else(|| "Int.bit_xor requires 2 arguments".to_string())?;
    match (x, y) {
        (VMValue::Int(x), VMValue::Int(y)) => Ok(VMValue::Int(x ^ y)),
        _ => Err("Int.bit_xor requires two Int arguments".to_string()),
    }
}
"Int.bit_not" => {
    let v = args.into_iter().next()
        .ok_or_else(|| "Int.bit_not requires 1 argument".to_string())?;
    match v {
        VMValue::Int(x) => Ok(VMValue::Int(!x)),
        _ => Err("Int.bit_not requires an Int argument".to_string()),
    }
}
"Int.shift_left" => {
    let mut it = args.into_iter();
    let x = it.next().ok_or_else(|| "Int.shift_left requires 2 arguments".to_string())?;
    let n = it.next().ok_or_else(|| "Int.shift_left requires 2 arguments".to_string())?;
    match (x, n) {
        (VMValue::Int(x), VMValue::Int(n)) => {
            if n < 0 || n >= 64 {
                return Err(format!("Int.shift_left: shift amount {} out of range 0..=63", n));
            }
            Ok(VMValue::Int(x << n))
        }
        _ => Err("Int.shift_left requires two Int arguments".to_string()),
    }
}
"Int.shift_right" => {
    let mut it = args.into_iter();
    let x = it.next().ok_or_else(|| "Int.shift_right requires 2 arguments".to_string())?;
    let n = it.next().ok_or_else(|| "Int.shift_right requires 2 arguments".to_string())?;
    match (x, n) {
        (VMValue::Int(x), VMValue::Int(n)) => {
            if n < 0 || n >= 64 {
                return Err(format!("Int.shift_right: shift amount {} out of range 0..=63", n));
            }
            Ok(VMValue::Int(x >> n))
        }
        _ => Err("Int.shift_right requires two Int arguments".to_string()),
    }
}
```

---

## T3: `fav/src/middle/checker.rs` — builtin_ret_ty 更新

### 事前確認

```bash
grep -n '"bnot"\|"band"\|"shl"' fav/src/middle/checker.rs | head -5
```

### 追加コード

既存の `("Int", "bnot") | ("Int", "to_byte")` アームの直後（`// Math` コメントの前）に追加する。
`("Int", "bit_not")` は可読性のため別アームとする（5 件 + 1 件の分割）。

```rust
// Int bit operations v23.2.0 (public API names)
("Int", "bit_and")
| ("Int", "bit_or")
| ("Int", "bit_xor")
| ("Int", "shift_left")
| ("Int", "shift_right") => Some(Type::Int),
("Int", "bit_not") => Some(Type::Int),
```

> **注意**: `compiler.rs` の変更は不要。`"Int"` namespace は既に登録済み（compiler.rs 168 行目）。
> `LoadGlobal("Int") + GetField("bit_and")` → runtime `"Int.bit_and"` 文字列が生成される。

---

## T4: `fav/src/driver.rs` — #[ignore] + v232000_tests 追加

### T4-1: `#[ignore]` 追加（Cargo.toml 変更前に実施）

```bash
grep -n "fn version_is_23_1_0\|mod v231000_tests" fav/src/driver.rs | head -5
```

`v231000_tests::version_is_23_1_0` に `#[ignore]` を追加。

### T4-2: `v232000_tests` モジュール追加

`v231000_tests` モジュールの直後に追加。
**重要**: テストは `Lexer → Parser → build_artifact → exec_artifact_main(&artifact, None)` の
4 ステップパターンを使う（`exec_artifact_main` は文字列を直接受け付けない）。

```rust
// ── v232000_tests (v23.2.0) — ビット演算 ──────────────────────────────────────
#[cfg(test)]
mod v232000_tests {
    use super::*;

    #[test]
    fn version_is_23_2_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"23.2.0\""), "Cargo.toml should have version 23.2.0");
    }

    #[test]
    fn int_bit_and_with_hex_literals() {
        // Int.bit_and(0xFF, 0x0F) = 255 & 15 = 15
        let src = "public fn main() -> Int { Int.bit_and(0xFF, 0x0F) }";
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize().expect("lex");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse");
        let artifact = build_artifact(&prog);
        let result = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(result, crate::value::Value::Int(15), "0xFF & 0x0F should be 15");
    }

    #[test]
    fn int_shift_left_correct() {
        let src = "public fn main() -> Int { Int.shift_left(1, 4) }";
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize().expect("lex");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse");
        let artifact = build_artifact(&prog);
        let result = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(result, crate::value::Value::Int(16), "1 << 4 should be 16");
    }

    #[test]
    fn int_bit_not_is_minus_one() {
        // !0i64 == -1 in two's complement (i64 全ビット反転)
        let src = "public fn main() -> Int { Int.bit_not(0) }";
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize().expect("lex");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse");
        let artifact = build_artifact(&prog);
        let result = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(result, crate::value::Value::Int(-1), "bit_not(0) should be -1");
    }

    #[test]
    fn changelog_has_v23_2_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v23.2.0]"), "CHANGELOG.md should have [v23.2.0] entry");
    }
}
```

---

## T5: Cargo.toml / CHANGELOG / benchmarks / MDX

### T5-1: `fav/Cargo.toml` バージョン更新

```toml
version = "23.2.0"
```

### T5-2: `CHANGELOG.md` — v23.2.0 エントリ追加（v23.1.0 の上）

```markdown
## [v23.2.0] — 2026-06-21

### Added
- `Int.bit_and`, `Int.bit_or`, `Int.bit_xor`, `Int.bit_not`, `Int.shift_left`, `Int.shift_right`（ビット演算 public API）
- 16 進数リテラル `0xFF` 対応（lexer で `0x`/`0X` prefix を解析 → `TokenKind::Int`）
- `site/content/docs/primitives/bit-ops.mdx` — ビット演算ドキュメント（`bit_not` の i64 挙動を明示）
```

### T5-3: `benchmarks/v23.2.0.json` — 新規作成

`test_count` は `cargo test --bin fav` 実行後の実測値を記入。

### T5-4: `site/content/docs/primitives/bit-ops.mdx` — 新規作成

`site/content/docs/primitives/` ディレクトリを新規作成してから配置する。
内容: ビット演算の使い方・各関数シグネチャ・`bit_not` の i64 挙動（`-1` であり `0xFFFFFFFF` でない旨）・vm.fav での活用例。

---

## 検証手順

```bash
cd /c/Users/yoshi/favnir/fav

# 単体テスト
cargo test v232000 --bin fav

# リグレッションなし確認
cargo test --bin fav
```

期待: v232000_tests 5/5 PASS、全体リグレッションなし。
