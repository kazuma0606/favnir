# v17.2.0 Spec — パターンマッチ拡張

Date: 2026-06-15

---

## 概要

`match` 式でデータの形を完全に記述できるようにする。
or-pattern / バインディングパターン / guard / list-pattern の 4 種類を追加する。
既存の `Pattern::Bind` / `Pattern::Variant` を拡張し、後方互換を維持しながら実装する。

---

## 構文

### or-pattern（縦棒で複数パターンを OR）

```fav
match status {
  "active" | "pending" => process(row)
  "deleted" | "archived" | "cancelled" => skip(row)
  _ => Result.err(f"unknown status: {status}")
}

// バリアントにも適用
match event {
  Event.Created(x) | Event.Updated(x) => handle_upsert(x)
  Event.Deleted(id)                   => handle_delete(id)
}
```

### バインディングパターン（レコード分解）

```fav
match event {
  Event.Created({ id: i, name: n }) => handle_created(i, n)
  Event.Updated({ id: i, ..rest }) => handle_updated(i, rest)
  Event.Deleted(id) => handle_deleted(id)
}
```

### guard 条件（if）

```fav
match row {
  { amount: a } if a > 1000.0 => high_value(row)
  { amount: a } if a > 0.0   => normal(row)
  _                           => Result.err("negative amount")
}
```

### list-pattern（リストの先頭・末尾分解）

```fav
match rows {
  []              => Result.err("empty list")
  [single]        => process_single(single)
  [head, ..tail]  => process_many(head, tail)
}

// 複数先頭要素
match rows {
  [a, b, ..rest] => compare_and_process(a, b, rest)
  _              => Result.err("need at least 2 rows")
}
```

---

## 実装ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | `version` を `"17.2.0"` に更新 |
| `fav/src/ast.rs` | `Pattern::Or(Vec<Pattern>)` / `Pattern::List { head, tail }` 追加、`MatchArm.guard: Option<Expr>` 追加 |
| `fav/src/frontend/parser.rs` | `parse_pattern` で Or / List / Guard を解析 |
| `fav/src/middle/checker.rs` | Or / List パターンの型チェック・網羅性チェック拡張 |
| `fav/src/middle/compiler.rs` | `compile_or_pattern` / `compile_list_pattern` 実装 |
| `fav/src/fmt.rs` | Or / List / Guard の pretty-print 対応 |
| `fav/src/driver.rs` | `v172000_tests` モジュール（5 件） |
| `fav/src/middle/ast_lower_checker.rs` | 新 Pattern バリアント exhaustive match 対応 |
| `site/content/docs/language/patterns.mdx` | パターンマッチ全集（新規作成） |

---

## AST 変更詳細

### Pattern への追加

```rust
pub enum Pattern {
    // 既存
    Wildcard(Span),
    Bind(String, Span),
    Lit(Lit, Span),
    Variant(String, Option<Box<Pattern>>, Span),
    Record(Vec<PatternField>, Span),
    // 新規追加
    Or(Vec<Pattern>, Span),                          // "a" | "b" | "c"
    List { head: Vec<Pattern>, tail: Option<String>, span: Span },  // [a, b, ..rest]
}
```

### MatchArm への追加

```rust
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,   // 追加: `if condition`
    pub body: Expr,
    pub span: Span,
}
```

---

## エラーコード

既存エラーコードを使用（新規追加なし）:
- **E0010**: 非網羅的パターン（or-pattern / list-pattern 追加後も既存ロジックを拡張）

---

## テスト（v172000_tests）

| # | テスト名 | 内容 |
|---|---|---|
| 1 | `version_is_17_2_0` | `Cargo.toml` に `"17.2.0"` が含まれる |
| 2 | `or_pattern_string` | `"active" \| "pending" => ...` が動作する |
| 3 | `list_pattern_head_tail` | `[head, ..tail]` で先頭と残りに分解できる |
| 4 | `list_pattern_empty_single` | `[]` / `[x]` パターンが動作する |
| 5 | `match_guard` | `{ amount: a } if a > 0.0 => ...` guard 条件が動作する |

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `"a" \| "b" => ...` or-pattern が動作する | [ ] |
| `[head, ..tail]` list-pattern が動作する | [ ] |
| `[]` / `[x]` パターンが動作する | [ ] |
| `if guard` 条件が動作する | [ ] |
| or-pattern / list-pattern を含む match の網羅性チェックが正しく動作する | [ ] |
| `cargo test v172000` → 5/5 PASS | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
