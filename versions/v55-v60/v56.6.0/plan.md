# Plan — v56.6.0 — パターンエイリアス（as-patterns `@`）

## ゴール

`name @ sub-pattern` as-pattern 構文を実装し、
全通過テスト数を 3238 → 3240（+2）に引き上げる。

---

## 実装ステップ

### Phase 1: Cargo.toml バージョン更新

`56.5.0` → `56.6.0`

---

### Phase 2: `fav/src/frontend/lexer.rs` — `@` トークン追加

1. `TokenKind` 列挙体に `At, // @ (v56.6.0: as-pattern)` を追加（Symbols セクション末尾）
2. `lex_char` / lex ディスパッチに `'@' => TokenKind::At` を追加
3. `test_unexpected_char` を `@` → `$` に変更（`@` は有効トークンになるため）

---

### Phase 3: `fav/src/ast.rs` — `Pattern::As` 追加

1. `Pattern` 列挙体に `As(String, Box<Pattern>, Span)` を追加（`List` の直後）
2. `Pattern::span()` に `Pattern::As(_, _, s) => s` を追加

---

### Phase 4: `fav/src/frontend/parser.rs` — `@` パース

`parse_pattern()` の `TokenKind::Ident` アームの小文字 Bind 分岐（L2908 付近）に追加:
```rust
if self.peek() == &TokenKind::At {
    self.advance();
    let sub_pattern = self.parse_pattern()?;
    Ok(Pattern::As(name, Box::new(sub_pattern), self.span_from(&start)))
} else {
    Ok(Pattern::Bind(name, self.span_from(&start)))
}
```

---

### Phase 5: `fav/src/middle/ir.rs` — `IRPattern::As` 追加

`IRPattern` 列挙体に `As(u16, Box<IRPattern>)` を追加（`List` の直後）:
```rust
/// `name @ sub_pattern` — as-pattern (v56.6.0)
As(u16, Box<IRPattern>),
```

---

### Phase 6: `fav/src/middle/compiler.rs` — `Pattern::As` 対応

1. `pattern_binds`: `Pattern::As(name, inner, _) => { out.insert(name.clone()); pattern_binds(inner, out); }`
2. `compile_pattern`: `Pattern::As(name, inner, _) => IRPattern::As(ctx.define_local(name.clone()), Box::new(compile_pattern(inner, ctx)))`

---

### Phase 7: `fav/src/backend/codegen.rs` — `IRPattern::As` 対応

`emit_pattern_test` に追加（`IRPattern::List` の後）:
```rust
IRPattern::As(slot, inner) => {
    cg.emit_opcode(Opcode::Dup);
    cg.emit_opcode(Opcode::StoreLocal);
    cg.emit_u16(*slot);
    emit_pattern_test(inner, fail_jumps, cg, depth)  // 末尾セミコロンなし — usize を返す
}
```

**重要**: `emit_pattern_test` は `usize` を返す。末尾に `;` を付けると `()` が返りコンパイルエラーになる。
`emit_pattern_test(inner, ...)` を最後の式として書き、戻り値を上位に伝搬させること。

---

### Phase 8: `fav/src/middle/checker.rs` — `Pattern::As` 対応

1. `check_pattern_bindings`: `Pattern::As(name, inner, _) => { self.env.define(name.clone(), ty.clone()); self.check_pattern_bindings(inner, ty); }`
2. `collect_pattern_variants`: `Pattern::As(_, inner, _) => collect_pattern_variants(inner, covered, has_catch_all),`

---

### Phase 9: `fav/src/middle/ast_lower_checker.rs` — `Pattern::As` 対応

`lower_pat` に追加（`Pattern::List` の後）:
```rust
ast::Pattern::As(_, inner, _) => lower_pat(inner),
```

---

### Phase 10: `fav/src/fmt.rs` — `Pattern::As` フォーマット

`fmt_pattern` に追加（`Pattern::List` の後）:
```rust
Pattern::As(name, inner, _) => format!("{} @ {}", name, fmt_pattern(inner)),
```

---

### Phase 11: `fav/src/emit_python.rs` — `Pattern::As` 対応

`arm_condition` に追加（`Pattern::List` の後）:
```rust
Pattern::As(name, inner, _) => {
    let (cond, mut binds) = self.arm_condition(inner, var);
    binds.push(format!("{} = {}", name, var));
    (cond, binds)
}
```

---

### Phase 12: `fav/src/lint.rs` — `pattern_is_catch_all` 更新

`matches!` マクロを `match` 式に変更し `Pattern::As` を追加:
```rust
fn pattern_is_catch_all(pat: &Pattern) -> bool {
    match pat {
        Pattern::Wildcard(_) | Pattern::Bind(_, _) => true,
        Pattern::As(_, inner, _) => pattern_is_catch_all(inner),
        _ => false,
    }
}
```

---

### Phase 13: `fav/src/driver.rs` — `v56600_tests` 追加

`v56500_tests` の直前に挿入（2 件）。`check_errors` ヘルパーは `v56500_tests` の同名関数と同一パターンを踏襲する:

```rust
mod v56600_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    fn check_errors(src: &str) -> Vec<String> {
        let program = Parser::parse_str(src, "v56600_test.fav").expect("parse");
        Checker::check_program(&program)
            .0
            .iter()
            .map(|e| e.code.to_string())
            .collect()
    }

    #[test]
    fn pattern_alias_binds_whole() { ... }

    #[test]
    fn pattern_alias_with_destructure() { ... }
}
```

---

### Phase 14: `fav/src/driver.rs` — バージョンチェックテスト更新

`v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"56.5.0"` → `"56.6.0"` に更新。
（モジュール名・関数名は変更しない — v56.3.0 以降の慣例として期待値のみ更新する）

---

### Phase 15: ポスト処理

- `CHANGELOG.md` に v56.6.0 エントリを追加
- `versions/current.md` を v56.6.0 / 3240 tests に更新
- `roadmap-v56.1-v57.0.md` の v56.6.0 実績を COMPLETE に更新
- `roadmap-v55.1-v60.0.md` の v56.6.0 実績欄も COMPLETE に更新

---

## テスト数計算

| 操作 | 件数 |
|------|------|
| v56.5.0 実績 | 3238 |
| `test_unexpected_char` 更新（`@` → `$`）— 削除でなく更新 | ±0 |
| `v56600_tests` 新規追加 2 件 | +2 |
| `v56300_tests::cargo_toml_version_is_56_3_0` 期待値更新 | ±0 |
| **目標合計** | **3240** |

---

## リスク管理

| リスク | 対策 |
|--------|------|
| `emit_pattern_test` の `IRPattern::As` スタック計算ミス | `Dup + StoreLocal` は net 0（Bind と同じ動作）。depth は変わらない |
| `Pattern::As` の網羅漏れによるコンパイルエラー | 11 ファイルすべてを Phase 順に実装。`cargo build` で即座に確認 |
| `pattern_is_catch_all` の `v @ _` 偽陰性 | `Pattern::As(_, inner)` を再帰チェックで対応 |
| ロードマップのサンプルコード `[head @ { id } | tail]` が現行 list-pattern 構文と不整合 | テストを単純ケース（`v @ Ok(n)`, `n @ 1`）に限定し、リスト内 as-pattern は将来対応 |
| `test_unexpected_char` の更新忘れによるテスト失敗 | Phase 2 でレクサー変更と同時に更新 |
| `arm_condition` の `Pattern::As` 追加忘れ（exhaustive match） | `cargo build` で E0004 が出るため即座に検出可能 |
