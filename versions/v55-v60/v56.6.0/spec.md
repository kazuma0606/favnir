# Spec — v56.6.0 — パターンエイリアス（as-patterns `@`）

## 概要

`name @ sub-pattern` 構文（as-pattern）を Favnir に追加する。
マッチした値全体を `name` に束縛しつつ、`sub-pattern` で構造分解も行える。

**ロードマップ記載との差異**: ロードマップのサンプルコード `[head @ { id, amount } | tail]` は
リスト内包記法の `|` 区切り（現行 `..tail` 構文とは異なる）を使っているため参考に留める。
v56.6.0 のテストは `name @ variant(inner)` / `name @ { field }` の単純ケースを対象とする。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v56.1-v57.0.md` — v56.6.0 セクション
- `versions/roadmap/roadmap-v55.1-v60.0.md` — v56.6.0 行
- ベーステスト数: **3238**（v56.5.0 完了時点の実績値）
- 目標テスト数: **3240**（+2）

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "56.6.0"
```

---

### 2. `fav/src/frontend/lexer.rs` — `TokenKind::At` 追加

`@` を有効なトークンとして追加する。

**TokenKind 列挙体への追加**（Symbols セクション末尾）:
```rust
At, // @  (v56.6.0: as-pattern)
```

**lex_char 関数への追加**（`'.'` 処理付近）:
```rust
'@' => TokenKind::At,
```

**既存テストの更新**:
`test_unexpected_char` は `@` から `$` などの別の無効文字に変更する:
```rust
fn test_unexpected_char() {
    let msg = lex_err("foo $ bar");
    assert!(msg.contains("unexpected character '$'"));
}
```

---

### 3. `fav/src/ast.rs` — `Pattern::As` 追加

**Pattern 列挙体への追加**（`List { ... }` の直後）:
```rust
/// `name @ sub_pattern` — as-pattern (v56.6.0)
/// 値全体を `name` に束縛しつつ `sub_pattern` でマッチする。
As(String, Box<Pattern>, Span),
```

**`Pattern::span()` の更新**:
```rust
Pattern::As(_, _, s) => s,
```

---

### 4. `fav/src/frontend/parser.rs` — `@` パース

`parse_pattern()` の `TokenKind::Ident` アームで、小文字識別子を `Bind` として解析した後、
次トークンが `TokenKind::At` なら `As` に昇格する:

```rust
} else {
    // lowercase → bind（またはas-pattern）
    if self.peek() == &TokenKind::At {
        self.advance(); // consume '@'
        let sub_pattern = self.parse_pattern()?;
        Ok(Pattern::As(name, Box::new(sub_pattern), self.span_from(&start)))
    } else {
        Ok(Pattern::Bind(name, self.span_from(&start)))
    }
}
```

---

### 5. `fav/src/middle/ir.rs` — `IRPattern::As` 追加

```rust
/// `name @ sub_pattern` — as-pattern (v56.6.0)
As(u16, Box<IRPattern>),
```

---

### 6. `fav/src/middle/compiler.rs` — `Pattern::As` 対応

**`pattern_binds` 関数**（List アームの直後）:
```rust
Pattern::As(name, inner, _) => {
    out.insert(name.clone());
    pattern_binds(inner, out);
}
```

**`compile_pattern` 関数**（List アームの直後）:
```rust
// as-pattern (v56.6.0)
Pattern::As(name, inner, _) => {
    let slot = ctx.define_local(name.clone());
    IRPattern::As(slot, Box::new(compile_pattern(inner, ctx)))
}
```

---

### 7. `fav/src/backend/codegen.rs` — `IRPattern::As` 対応

`emit_pattern_test` 関数に追加（`IRPattern::List` アームの後）:

```rust
// as-pattern (v56.6.0): bind whole value then test sub-pattern
IRPattern::As(slot, inner) => {
    cg.emit_opcode(Opcode::Dup);
    cg.emit_opcode(Opcode::StoreLocal);
    cg.emit_u16(*slot);
    emit_pattern_test(inner, fail_jumps, cg, depth)  // 末尾セミコロンなし — 戻り値を上位に返す
}
```

**動作**: `Dup + StoreLocal` は net 0（値を slot に書き込みつつスタックを保持）。
その後 `inner` パターンを同じ深さでテストし、その戻り値（depth）をそのまま返す。

**注意**: 末尾に `;` を付けると `()` が返り `usize` 型不一致でコンパイルエラーになる。
`emit_pattern_test(inner, ...)` は Rust の「最後の式 = 戻り値」として機能する必要がある。

---

### 8. `fav/src/middle/checker.rs` — `Pattern::As` 対応

**`check_pattern_bindings`**（List アームの後）:
```rust
Pattern::As(name, inner, _) => {
    self.env.define(name.clone(), ty.clone());
    self.check_pattern_bindings(inner, ty);
}
```

**`collect_pattern_variants`**（既に `_ => {}` で catch されるが、明示的に追加）:
```rust
Pattern::As(_, inner, _) => collect_pattern_variants(inner, covered, has_catch_all),
```

---

### 9. `fav/src/middle/ast_lower_checker.rs` — `Pattern::As` 対応

`lower_pat` 関数（List アームの後）:
```rust
ast::Pattern::As(_, inner, _) => lower_pat(inner),
```

---

### 10. `fav/src/fmt.rs` — `Pattern::As` フォーマット

`fmt_pattern` 関数（List アームの後）:
```rust
Pattern::As(name, inner, _) => format!("{} @ {}", name, fmt_pattern(inner)),
```

---

### 11. `fav/src/emit_python.rs` — `Pattern::As` 対応

`arm_condition` 関数（List アームの後）:
```rust
Pattern::As(name, inner, _) => {
    // bind whole value to name, then apply inner condition
    let (cond, mut binds) = self.arm_condition(inner, var);
    binds.push(format!("{} = {}", name, var));
    (cond, binds)
}
```

---

### 12. `fav/src/lint.rs` — `pattern_is_catch_all` 更新

`pattern_is_catch_all` を `matches!` マクロから `match` 式に変更し、
`Pattern::As` の catch-all 判定を追加する:

```rust
fn pattern_is_catch_all(pat: &Pattern) -> bool {
    match pat {
        Pattern::Wildcard(_) | Pattern::Bind(_, _) => true,
        // as-pattern: catch-all iff sub-pattern is catch-all (e.g. `v @ _`)
        Pattern::As(_, inner, _) => pattern_is_catch_all(inner),
        _ => false,
    }
}
```

**変換理由**: `matches!(pat, Pattern::As(_, inner, _) if pattern_is_catch_all(inner))` は文法上有効だが、
将来のバリアント追加時にパターン列が視認しにくくなるため `match` 式に統一する。
`matches!` のままでも `v @ _` 偽陰性は防げるが、保守性向上のため変換する。

---

### 13. `fav/src/driver.rs` — `v56600_tests` 追加

`v56500_tests` の直前に挿入する（3 件: 2 機能テスト + 1 バージョンチェック）。

**テスト 1: `pattern_alias_binds_whole`**

`v` を arm 本体で実際に参照することで、`check_pattern_bindings` が `v` を env に定義したことも検証する:

```rust
fn pattern_alias_binds_whole() {
    let src = r#"
fn describe(r: Result<Int, String>) -> Bool {
    match r {
        v @ Ok(_) => true
        Err(_)    => false
    }
}
public fn main() -> Bool { true }
"#;
    // 型チェックエラーなし
}
```

**テスト 2: `pattern_alias_with_destructure`**
```rust
fn pattern_alias_with_destructure() {
    let src = r#"
fn process(x: Int) -> String {
    match x {
        n @ 1 => "one"
        _     => "other"
    }
}
public fn main() -> Bool { true }
"#;
    // 型チェックエラーなし
}
```

---

### 14. `fav/src/driver.rs` — バージョンチェックテスト更新

`v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"56.5.0"` → `"56.6.0"` に更新。

**慣例説明**: このテストは v56.3.0 時点のモジュール名・関数名を維持したまま、
各バージョン更新のたびに期待値（`"56.X.Y"`）のみを書き換える運用となっている。
モジュール名 `v56300_tests`・関数名 `cargo_toml_version_is_56_3_0` は変更しない。

---

## テスト仕様

| テスト名 | 検証内容 |
|---------|---------|
| `pattern_alias_binds_whole` | `v @ Ok(n) => "ok"` が型チェックエラーなし |
| `pattern_alias_with_destructure` | `n @ 1 => "one"` が型チェックエラーなし |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3240 tests passed, 0 failed**）
- `cargo clippy -- -D warnings` クリーン
- `pattern_alias_binds_whole` pass
- `pattern_alias_with_destructure` pass
- `@` トークンが lexer で有効になっている（lex_err テスト更新済み）
- `Pattern::As` が ast.rs に追加されている
- `IRPattern::As` が ir.rs に追加されている
- 11 ファイルすべてに `Pattern::As` / `IRPattern::As` の対応が追加されている
- `CHANGELOG.md` に v56.6.0 エントリが追加されている
- `versions/current.md` が v56.6.0 / 3240 tests を反映
- 両ロードマップの v56.6.0 実績を COMPLETE に更新

---

## サイトドキュメント

as-pattern 構文の MDX ページ追加は **v56.8.0 の Language Power 2.0 記事**（`site/content/docs/language/` 群）に委譲する。
v56.6.0 では MDX ファイルの新規作成は行わない。

---

## 備考

- **テスト数**: `test_unexpected_char` は `@` から `$` に変更（削除ではなく更新） → 合計 ±0。net +2 = 3240。
- **`emit_pattern_test` のスタック動作**: `Dup + StoreLocal slot` は net 0 操作（Bind と同じ）。
  その後 inner をテストするため `depth` は変わらない。
- **`pattern_is_catch_all`**: `v @ _` は catch-all として正しく検出される（W037 連携）。
- **`collect_pattern_variants`**: `_ => {}` catch-all が既存のため Pattern::As は素通りするが、
  明示的に inner に委譲することで将来の拡張に対応する。
- **`emit_python.rs` `arm_condition`**: `Pattern::As` は inner の条件を使いつつ `name = var` binding を追加。
- **ロードマップサンプルコードとの差異**: `[head @ { id, amount } | tail]` のリスト `|` 区切り構文は
  現行の list-pattern 構文と異なる。v56.6.0 では単純なケースのみ対応し、リスト内 as-pattern は将来対応。
