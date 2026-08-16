# v71.5.0 実装計画 — Generic Constraints（`impl Trait` 風の境界）

---

## Step 1: 事前確認

- `fav/Cargo.toml` のバージョンが `71.4.0` であることを確認
- `cargo test` が全 pass（3594 tests）であることを確認
- `parse_type_bounds` 関数の現在の実装を確認（`parser.rs` ~line 1707）
- `TypeConstraint::Interface` の利用箇所を確認
- `TokenKind::Colon` が Lexer で定義されているかを確認
- `Ampersand`（`&`）の TokenKind 名称を確認（`TokenKind::And` か `TokenKind::Ampersand` かを grep）

---

## Step 2: パーサー — `parse_type_bounds` に `:` 記法追加

`fav/src/frontend/parser.rs` の `parse_type_bounds` 関数末尾（`while self.peek() == &TokenKind::With...` の直後）に `:` 記法を追加:

```rust
// v71.5.0: colon-style bounds: `<T: A & B>` and `<T: impl A>`
if self.peek() == &TokenKind::Colon {
    self.advance(); // consume `:`
    loop {
        // `impl` is sugar — skip it
        if self.peek_ident_text("impl") {
            self.advance();
        }
        let (bound_name, _) = self.expect_ident()?;
        bounds.push(TypeConstraint::Interface(bound_name));
        // check for `&` to continue with next bound
        if /* Ampersand token */ {
            self.advance(); // consume `&`
        } else {
            break;
        }
    }
}
```

**注意**: `&` の `TokenKind` 名は実際のコード（`lexer.rs` または `token.rs`）を確認してから使うこと。

---

## Step 3: `cargo build` + 既存テスト通過確認

- `cargo build` でエラーがないことを確認
- `cargo test` で既存 3594 件が全 pass であることを確認

---

## Step 4: `v715000_tests` 追加（`driver.rs`）

```rust
#[cfg(test)]
mod v715000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    /// fn<T: A & B> — 複数境界の colon 記法
    #[test]
    fn generic_constraint_multi_interface() {
        let src = concat!(
            "interface Serializable { serialize: Self -> String }\n",
            "interface Comparable { compare: Self -> Self -> Int }\n",
            "fn serialize_all<T: Serializable & Comparable>(item: T) -> String {\n",
            "    T.serialize(item)\n",
            "}\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse should succeed");
        let (errors, _) = Checker::check_program(&prog);
        assert!(
            errors.is_empty(),
            "multi-interface constraint should typecheck; errors: {:?}",
            errors
        );
    }

    /// fn<T: impl A> — impl Trait 糖衣構文
    #[test]
    fn generic_constraint_impl_trait() {
        let src = concat!(
            "interface Printable { print: Self -> String }\n",
            "fn display<T: impl Printable>(item: T) -> String {\n",
            "    T.print(item)\n",
            "}\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse should succeed");
        let (errors, _) = Checker::check_program(&prog);
        assert!(
            errors.is_empty(),
            "impl Trait sugar should typecheck; errors: {:?}",
            errors
        );
    }
}
```

`cargo test v715000` で 2 件 pass を確認。

---

## Step 5: Cargo.toml バージョン更新

- `fav/Cargo.toml` の `version` を `"71.4.0"` → `"71.5.0"` に更新
- `driver.rs` 内の `"71.4.0"` 文字列を `"71.5.0"` に一括更新

---

## Step 6: CHANGELOG.md 更新

```markdown
## [v71.5.0] — 2026-08-09 — Generic Constraints（`impl Trait` 風の境界）

### Added
- `v715000_tests`: 2 件追加（3594 → 3596 tests）
  - `generic_constraint_multi_interface`
  - `generic_constraint_impl_trait`
- パーサー: `<T: A & B>` 型パラメータ境界記法を追加（既存 `<T with A with B>` の代替）
- パーサー: `<T: impl A>` 糖衣構文を追加（`impl` キーワードをスキップ）
- 既存 `<T with A>` 構文との後方互換性を維持
- 境界違反は既存 E0422 で検出（新規エラーコードなし）
```

---

## Step 7: versions/current.md 更新

- 「進行中バージョン」を `v71.5.0`（Generic Constraints）に更新
- 「次に切る版」を `v71.6.0` に更新

---

## Step 8: 最終確認

- `cargo test v715000` で 2 件 pass
- `cargo test` 全体で 3596 件 pass（0 failures）
- `fav/Cargo.toml` が `71.5.0`
- `versions/current.md` が正しく更新されている
