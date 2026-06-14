# v16.6.0 Plan — モジュールシステム強化（Namespace Alias）

Date: 2026-06-14

---

## 実装フェーズ一覧

| Phase | 内容 | 主要ファイル |
|---|---|---|
| A | Cargo バージョン更新 | `Cargo.toml` |
| B | Lexer: `as` キーワード追加 | `lexer.rs` |
| C | AST: `Item::UseAlias` 追加 | `ast.rs` |
| D | Parser: `use X as Y` パース | `parser.rs` |
| E | Compiler: `namespace_aliases` + エイリアス解決 | `compiler.rs` |
| F | Checker: `namespace_aliases` + exhaustive match | `checker.rs` |
| G | exhaustive match 対応（各ファイル） | `driver.rs` 等 |
| H | テスト追加（v166000_tests — 5 件） | `driver.rs` |
| I | サイトドキュメント | `site/content/docs/language/modules.mdx` |
| J | テスト確認とコミット | — |

---

## Phase A — Cargo バージョン更新

```toml
[package]
version = "16.6.0"
```

**確認:** `cargo build` でコンパイルエラーなし。

---

## Phase B — Lexer: `as` キーワード追加（lexer.rs）

### B-1: `TokenKind` に `As` 追加

```rust
// Keywords
...
Alias,
As,      // ← 追加
```

### B-2: キーワードマッピングに追加

```rust
"as" => TokenKind::As,
```

**確認:** `cargo build` でコンパイルエラーなし。

---

## Phase C — AST: `Item::UseAlias` 追加（ast.rs）

### C-1: `Item` enum に追加

```rust
/// `use String as S` — ネームスペースエイリアス（v16.6.0）
UseAlias {
    original: String,  // 元のネームスペース名
    alias: String,     // エイリアス名
    span: Span,
},
```

### C-2: `Item::span()` に追加

```rust
Item::UseAlias { span, .. } => span,
```

**確認:** `cargo build` → exhaustive match エラーを確認（Phase G で対処）。

---

## Phase D — Parser: `use X as Y` パース（parser.rs）

### D-1: `parse_item` の `TokenKind::Use` 分岐を拡張

現在の `TokenKind::Use` 分岐:
```rust
TokenKind::Use => {
    // `use X.{ a, b }` or `use X.*` — rune-internal file import
    let span = self.peek_span().clone();
    self.advance(); // consume 'use'
    let (module, _) = self.expect_ident()?;
    self.expect(&TokenKind::Dot)?;      // ← ここで Dot を expect
    ...
}
```

拡張後 — `as` が続く場合を先に検出:

```rust
TokenKind::Use => {
    let span = self.peek_span().clone();
    self.advance(); // consume 'use'
    let (name, _) = self.expect_ident()?;
    // `use X as Y` パターン
    if self.peek() == &TokenKind::As {
        self.advance(); // consume 'as'
        let (alias, _) = self.expect_ident()?;
        return Ok(Item::UseAlias { original: name, alias, span });
    }
    // 既存: `use X.{ ... }` or `use X.*`
    self.expect(&TokenKind::Dot)?;
    ...
}
```

**確認:** `cargo build` でコンパイルエラーなし（exhaust エラーは Phase G で対処）。

---

## Phase E — Compiler: エイリアス解決（compiler.rs）

### E-1: `CompileCtx` に `namespace_aliases` フィールド追加

```rust
pub struct CompileCtx<'a> {
    ...
    /// Namespace aliases: alias_name → real_namespace (v16.6.0)
    pub namespace_aliases: HashMap<String, String>,
}
```

### E-2: `CompileCtx::new` で初期化

```rust
namespace_aliases: HashMap::new(),
```

### E-3: アイテム収集フェーズで `UseAlias` を処理

`compile_program` の先頭（グローバル登録フェーズ）に追加:

```rust
Item::UseAlias { original, alias, .. } => {
    ctx.namespace_aliases.insert(alias.clone(), original.clone());
}
```

### E-4: `compile_expr` の `Expr::FieldAccess` でエイリアス解決

`Expr::FieldAccess { obj, field }` を処理する際、`obj` が `Expr::Ident(ns)` で
`namespace_aliases` に登録されている場合、`ns` を実際のネームスペース名に置換:

```rust
Expr::FieldAccess(obj, field, _) => {
    // エイリアス解決: use String as S → S を String に解決
    let effective_obj = if let Expr::Ident(ns, span) = obj.as_ref() {
        if let Some(real_ns) = ctx.namespace_aliases.get(ns.as_str()) {
            Cow::Owned(Expr::Ident(real_ns.clone(), span.clone()))
        } else {
            Cow::Borrowed(obj.as_ref())
        }
    } else {
        Cow::Borrowed(obj.as_ref())
    };
    // effective_obj を使って既存の FieldAccess コンパイルを実行
    ...
}
```

**注**: `Cow` を使わずに、既存の FieldAccess コンパイルロジックを分岐させる形でも可。
実装時にコードを読んで最も自然な挿入点を選ぶこと。

**確認:** `cargo build` でコンパイルエラーなし。

---

## Phase F — Checker: エイリアス登録 + exhaustive match（checker.rs）

### F-1: `Checker` struct に `namespace_aliases` 追加

```rust
/// Namespace aliases: alias → real_namespace (v16.6.0)
namespace_aliases: HashMap<String, String>,
```

### F-2: `new()` / `new_with_resolver()` で初期化

```rust
namespace_aliases: HashMap::new(),
```

### F-3: `register_item_signatures` に `UseAlias` 処理を追加

```rust
Item::UseAlias { original, alias, .. } => {
    self.namespace_aliases.insert(alias.clone(), original.clone());
}
```

### F-4: `check_item` に `UseAlias` 追加（no-op）

```rust
Item::UseAlias { .. } => {} // registered in register_item_signatures
```

### F-5: 型チェック時のエイリアス解決

`check_expr` で `Expr::FieldAccess` を処理する際に、`obj` の識別子名が
`namespace_aliases` に登録されている場合は実際のネームスペース名に置換してから型チェック。

具体的には、`check_apply` / `builtin_ret_ty` の namespace 引数が渡される箇所で解決する。

**確認:** `cargo build` でコンパイルエラーなし。

---

## Phase G — exhaustive match 対応

`Item::UseAlias` を追加したことで `non-exhaustive patterns` エラーが出るファイルを修正。
`cargo build` のエラーメッセージで対象ファイルを特定し、各ファイルに:

```rust
Item::UseAlias { .. } => {} // or appropriate formatting
```

を追加。

**予想対象:** `driver.rs`, `fmt.rs`（前バージョンで同様のパターン）

**確認:** `cargo build` でコンパイルエラーなし。

---

## Phase H — テスト追加（driver.rs）

`fav/src/driver.rs` に `v166000_tests` モジュールを追加:

```rust
#[cfg(test)]
mod v166000_tests {
    // version_is_16_6_0
    // namespace_alias_string   -- use String as S → S.concat が動作
    // namespace_alias_list     -- use List as L → L.length が動作
    // namespace_alias_math     -- use Math as M → M.abs が動作
    // namespace_alias_multi    -- 複数エイリアスが共存して動作
}
```

### テスト 1: version_is_16_6_0

```rust
#[test]
fn version_is_16_6_0() {
    let cargo = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(cargo.contains("version = \"16.6.0\""), "...");
}
```

### テスト 2: namespace_alias_string

```fav
use String as S

public fn main() -> String {
    S.concat("hello", " world")
}
```

期待値: `"hello world"`

### テスト 3: namespace_alias_list

```fav
use List as L

public fn main() -> Int {
    bind xs <- L.push(L.push(L.push(L.empty(), 1), 2), 3)
    L.length(xs)
}
```

期待値: `3`

### テスト 4: namespace_alias_math

```fav
use Math as M

public fn main() -> Int {
    M.abs(-42)
}
```

期待値: `42`

### テスト 5: namespace_alias_multi

```fav
use String as S
use List as L

public fn main() -> Int {
    bind parts <- S.split("a,b,c", ",")
    L.length(parts)
}
```

期待値: `3`

**テスト実行:** `cargo test v166000` → 5/5 PASS 確認。

---

## Phase I — サイトドキュメント

`site/content/docs/language/modules.mdx` を新規作成:
- `use` 構文の説明（既存 + 新 alias 構文）
- ネームスペースエイリアスの使用例
- プロジェクトモジュールのインポート（`use` / `import`）

---

## 実装の注意点

1. **`as` キーワードとの競合**: 将来の `match` アームや型キャストで `as` を使う可能性がある。
   `TokenKind::As` は `parse_item` の `use` 分岐でのみ消費し、他の文脈では識別子として扱える設計にする。
   （`parse_item` の外では `as` をキーワードとして予約しない — `_` と同様の扱い）

2. **Checker のエイリアス解決箇所**: `check_expr` 内の `Expr::FieldAccess` もしくは
   `check_apply` で namespace を引数として渡す箇所を特定し、そこでエイリアス解決を挿入する。
   `grep "builtin_ret_ty\|check_apply\|call_builtin_fn"` で特定すること。

3. **CompileCtx の初期化タイミング**: `UseAlias` アイテムは最初の走査フェーズ（グローバル登録）で
   処理する必要がある。2 パス目（compile_item）の前に収集が完了していること。

4. **ネームスペース名の大文字小文字**: 組み込みネームスペースは大文字始まり（`String`, `List` 等）。
   エイリアス名も大文字始まりを推奨するが、小文字も許可する（ユーザーの自由）。
