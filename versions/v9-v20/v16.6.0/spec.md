# v16.6.0 Spec — モジュールシステム強化（Namespace Alias / Import Ergonomics）

Date: 2026-06-14

---

## 概要

現在の `use module.fn` は 1 関数ずつのインポートのみ。
v16.6.0 では「よく使うモジュール名を短く書く」ための **ネームスペースエイリアス** を中心に、
インポートの書き心地を改善する。

```fav
-- ネームスペースエイリアス: 組み込み名前空間に短縮名をつける
use String   as S
use List     as L
use Math     as M
use DateTime as DT

-- 以降、短縮名で呼び出せる
S.concat("Hello, ", name)      -- String.concat と同じ
L.length(rows)                  -- List.length と同じ
M.round_to(score, 2)           -- Math.round_to と同じ
DT.now_unix()                   -- DateTime.now_unix と同じ
```

---

## ユースケース

### 1. 名前が長い組み込みモジュールの短縮

```fav
use DateTime as DT

fn log(msg: String) -> Unit {
    bind ts <- DT.format_iso(DT.now_unix())
    IO.println(S.concat(ts, S.concat(": ", msg)))
}
```

### 2. 複数エイリアスの組み合わせ

```fav
use String   as S
use List     as L
use Math     as M

fn summarize(nums: List<Float>) -> String {
    bind total  <- L.sum_by(nums, |n| n)
    bind count  <- L.length(nums)
    bind avg    <- total / count
    S.concat("avg=", S.from_float(M.round_to(avg, 2)))
}
```

### 3. プロジェクトモジュールへのエイリアス（将来拡張向け構文、構文解析のみ）

```fav
use very.long.module.path as M
use json as J
```

---

## 構文仕様

```
UseAlias ::= "use" Ident "as" Ident
           | "use" DottedIdent "as" Ident   -- プロジェクトモジュール（将来）
```

---

## 実装仕様

### 1. Lexer（lexer.rs）

`As` トークンを追加（`as` キーワード）。

```rust
As,  // as
```

```rust
"as" => TokenKind::As,
```

### 2. AST（ast.rs）

`Item::UseAlias` variant を追加:

```rust
UseAlias {
    original: String,  // 元のネームスペース名（"String", "List" 等）
    alias: String,     // 短縮名（"S", "L" 等）
    span: Span,
}
```

`Item::span()` にも追加。

### 3. Parser（parser.rs）

`parse_item` の `TokenKind::Use` 分岐を拡張:

```
use IDENT as IDENT → Item::UseAlias
use IDENT.{ fn1, fn2 } → 既存 Item::RuneUse
use IDENT.* → 既存 Item::RuneUse
```

- `use` の直後に識別子、その後 `as` が続く場合 → `UseAlias`
- `use` の直後に識別子、その後 `.` が続く場合 → 既存の `RuneUse` パス

### 4. Compiler（compiler.rs）

`CompileCtx` に `namespace_aliases: HashMap<String, String>` フィールドを追加。

`use` アイテムの収集フェーズで `UseAlias` を処理:

```rust
Item::UseAlias { original, alias, .. } => {
    ctx.namespace_aliases.insert(alias.clone(), original.clone());
}
```

`compile_expr` の `Expr::FieldAccess` 処理で、`obj` が `Expr::Ident` の場合にエイリアス解決:

```rust
if let Expr::Ident(ns, _) = obj.as_ref() {
    let real_ns = ctx.namespace_aliases.get(ns).cloned().unwrap_or(ns.clone());
    // real_ns を使って既存の namespace 解決ロジックを呼ぶ
}
```

### 5. Checker（checker.rs）

`Checker` に `namespace_aliases: HashMap<String, String>` を追加。

`register_item_signatures` で `UseAlias` を処理:

```rust
Item::UseAlias { original, alias, .. } => {
    self.namespace_aliases.insert(alias.clone(), original.clone());
}
```

`check_item` に `Item::UseAlias { .. } => {}` 追加（no-op）。

型解決時: `Expr::FieldAccess` の `obj` が `Ident(alias)` の場合、
`namespace_aliases` でエイリアスを解決してからビルトイン型チェックを行う。

### 6. 対象外（v16.6.0 スコープ外）

- `pub use ...` re-export — 構文パース対応のみ（実行は将来）
- `use module.*` ワイルドカード（プロジェクトモジュール） — 既存 RuneUse で対応済み
- 非 public 関数への E0324 エラー — プロジェクトモード専用（将来）

---

## エラーコード

| コード | 説明 |
|---|---|
| E0330 | `use X as Y` で X が未知のネームスペース（組み込みでもモジュールでもない）|

---

## テスト（v166000_tests — 5 件）

| # | テスト名 | 確認内容 |
|---|---|---|
| 1 | `version_is_16_6_0` | Cargo.toml バージョンが 16.6.0 |
| 2 | `namespace_alias_string` | `use String as S` → `S.concat("a","b")` = "ab" |
| 3 | `namespace_alias_list` | `use List as L` → `L.length(xs)` = 3 |
| 4 | `namespace_alias_math` | `use Math as M` → `M.abs(-42)` = 42 |
| 5 | `namespace_alias_multi` | `use String as S; use List as L` — 複数エイリアスが共存 |

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "16.6.0"` | [ ] |
| `as` キーワードがレキサーで認識される | [ ] |
| `Item::UseAlias` が AST に追加されている | [ ] |
| `use String as S` → `S.concat` が `String.concat` として動作する | [ ] |
| `use List as L` → `L.length` が `List.length` として動作する | [ ] |
| `use Math as M` → `M.abs` が `Math.abs` として動作する | [ ] |
| 複数エイリアスが共存して動作する | [ ] |
| `cargo test v166000` 全テストパス（5/5） | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
| `site/content/docs/language/modules.mdx` が存在する | [ ] |
