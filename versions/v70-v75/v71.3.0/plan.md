# v71.3.0 Plan — Phantom Types（型タグによる誤使用防止）

Date: 2026-08-09
Status: 計画中

---

## 実装ステップ（依存順）

### Step 1: 事前確認

```bash
# phantom が識別子として扱われることを確認
grep -rn '"phantom"' fav/src/frontend/
# TypeDef.is_opaque の初期化箇所を確認（is_phantom と同じ箇所に追加が必要）
grep -n 'is_opaque:' fav/src/frontend/parser.rs
```

---

### Step 2: AST — `is_phantom: bool` 追加

`fav/src/ast.rs` の `TypeDef` に `is_opaque` の直後に追加:

```rust
pub is_opaque: bool,   // v43.11.0
pub is_phantom: bool,  // v71.3.0: phantom type キーワード（デフォルト false）
```

---

### Step 3: パーサー — 全 TypeDef 初期化に `is_phantom: false` 追加

`fav/src/frontend/parser.rs` の TypeDef を返す全箇所（4箇所）に `is_phantom: false` を追加:
1. line ~1549 — Wrapper body
2. line ~1580 — Record body
3. line ~1603 — Alias body（既存）
4. line ~1615 — Sum body

また、`opaque` 文脈キーワード処理（line ~692）では `is_phantom` が `false` のままになる（parse_type_def 後に `is_opaque = true` をセットするだけなので自動的に OK）。

---

### Step 4: パーサー — `phantom` 文脈キーワードの解析

`parse_type_def` 内の `self.expect(&TokenKind::Eq)?` の直後、alias body 解析（`else { let target = self.parse_type_expr()?; ...`）の**前**に追加:

```rust
// v71.3.0: phantom type: `type Name = phantom InnerType`
if matches!(self.peek(), &TokenKind::Ident(ref n) if n == "phantom") {
    self.advance(); // consume "phantom"
    let inner = self.parse_type_expr()?;
    return Ok(TypeDef {
        visibility,
        name,
        type_params,
        with_interfaces,
        invariants: vec![],
        is_opaque: false,
        is_phantom: true,
        body: TypeBody::Alias(inner),
        span: self.span_from(&start),
    });
}
```

**注意**: `TokenKind::Ident(ref n)` のパターンは `peek()` が `&TokenKind` を返すため `ref` が必要。または `self.peek().clone()` して match する形でも可。

---

### Step 5: チェッカー — pre-pass から phantom を除外

`register_item_signatures` の先頭 pre-pass（v71.2.0 追加）に `is_phantom` 除外を追加:

```rust
// v71.2.0 pre-pass: non-opaque, non-phantom alias invariants only
if !td.is_opaque && !td.is_phantom && !td.invariants.is_empty() {
    self.type_invariants.insert(td.name.clone(), td.invariants.clone());
}
```

---

### Step 6: チェッカー — phantom コンストラクタ登録

`register_item_signatures` の `TypeBody::Alias` ブランチを更新:

```rust
if let TypeBody::Alias(target) = &td.body {
    if td.is_opaque {
        // v45.5.0: 既存コード
        let inner_ty = self.resolve_type_expr(target);
        self.opaque_alias_inner.insert(td.name.clone(), inner_ty);
    } else if td.is_phantom {
        // v71.3.0: phantom type — register constructor, do NOT add to type_aliases
        let inner_ty = self.resolve_type_expr(target);
        let parent = Type::Named(td.name.clone(), vec![]);
        self.env.define(
            td.name.clone(),
            Type::Fn(vec![inner_ty], Box::new(parent)),
        );
        if !td.type_params.is_empty() {
            self.type_arity.insert(td.name.clone(), td.type_params.len());
        }
        continue; // skip type_aliases registration
    } else {
        self.type_aliases.insert(td.name.clone(), target.clone());
    }
    if !td.type_params.is_empty() {
        self.type_arity.insert(td.name.clone(), td.type_params.len());
    }
    // Define in env so the name resolves
    self.env.define(td.name.clone(), Type::Named(td.name.clone(), vec![]));
    continue;
}
```

**重要**: 既存コードでは `is_opaque` の場合でも最後の `env.define` と `continue` が実行されている（`is_opaque` ブランチは `type_aliases` をスキップするだけで `env.define` は共有）。`is_phantom` では専用 `env.define`（Fn型）を使い、その後 `continue` する。

---

### Step 7: fmt.rs — phantom 型のフォーマット

`fav/src/fmt.rs` の `type_def` 関数で `TypeBody::Alias` 部分を更新:

```rust
TypeBody::Alias(target) => {
    if td.is_phantom {
        format!("type {} = phantom {}", td.name, self.type_expr(target))
    } else if td.is_opaque {
        format!("type {} = opaque {}", td.name, self.type_expr(target))
    } else {
        format!("type {} = {}", td.name, self.type_expr(target))
    }
}
```

または既存のフォーマット処理に `phantom` prefix を挿入。

---

### Step 8: driver.rs に `v713000_tests` を追加

`v712000_tests` の直後に追加:

```rust
// ── v71.3.0: Phantom Types（型タグによる誤使用防止） ────────────────────────

#[cfg(test)]
mod v713000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    #[test]
    fn phantom_type_explicit_cast() {
        let src = concat!(
            "type UserId = phantom String\n",
            "fn get_user(id: UserId) -> Bool { true }\n",
            "fn good() -> Bool { get_user(UserId(\"u-123\")) }\n",
            "public fn main() -> Bool { true }\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse should succeed");
        let (errors, _) = Checker::check_program(&prog);
        assert!(
            errors.is_empty(),
            "UserId(\"u-123\") should typecheck cleanly; errors: {:?}",
            errors
        );
    }

    #[test]
    fn phantom_type_prevents_id_confusion() {
        let src = concat!(
            "type UserId  = phantom String\n",
            "type OrderId = phantom String\n",
            "fn get_user(id: UserId) -> Bool { true }\n",
            "fn bad() -> Bool { get_user(OrderId(\"x\")) }\n",
            "public fn main() -> Bool { true }\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse should succeed");
        let (errors, _) = Checker::check_program(&prog);
        assert!(
            !errors.is_empty(),
            "passing OrderId where UserId expected should produce a compile error; errors: {:?}",
            errors
        );
    }
}
```

---

### Step 9: Cargo.toml バージョン更新

- `fav/Cargo.toml` の `version = "71.2.0"` → `"71.3.0"`
- driver.rs 内の `"71.2.0"` 文字列リテラルを `"71.3.0"` に一括更新（replace_all）

---

### Step 10: CHANGELOG.md 更新

ヘッダー形式: `## [v71.3.0] — 2026-08-09 — Phantom Types（型タグによる誤使用防止）`

---

### Step 11: 最終確認

- `cargo test v713000` で 2 件 pass
- `cargo test` 全体で 3591 tests pass（0 failures）
- `versions/current.md` を v71.3.0 に更新

---

## 注意事項

### `is_opaque` との構造的類似

`is_phantom` は `is_opaque` と同様のパターンで実装する。ただし:
- `is_opaque`: コンストラクタなし、`opaque_alias_inner` に登録してE0413で内部型への直接代入を禁止
- `is_phantom`: コンストラクタあり（Fn型で env 登録）、type_aliases に登録しない（透過しない）

### `check_type_def` での Alias ブランチ

v71.2.0 で追加した `check_type_def` の `TypeBody::Alias` ブランチは `!td.is_opaque && !td.invariants.is_empty()` の条件でチェックするため、phantom 型（`td.invariants` が空）は自動的にスキップされる。変更不要。

### lint.rs の `collect_refinement_aliases` について

`lint.rs` の `collect_refinement_aliases` は `TypeBody::Alias(_)` かつ `!td.invariants.is_empty()` の両方が真の場合のみ処理する。phantom 型は `invariants: vec![]` で生成されるため自動的にスキップされる。変更不要。

### fmt.rs の現状確認と注意

現行 `fmt.rs` の `TypeBody::Alias` ブランチには `is_opaque` の出力分岐が存在しない（opaque 型も `type Name = Inner` と出力される既存バグ）。Step 7 では `is_phantom` と合わせて `is_opaque` の出力も修正することで、既存 opaque 型の round-trip 正確性も改善する。
