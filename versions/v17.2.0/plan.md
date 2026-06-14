# v17.2.0 Plan — パターンマッチ拡張

Date: 2026-06-15

---

## Phase A — Cargo バージョン更新

`fav/Cargo.toml` の `version` を `"17.2.0"` に更新。
`cargo build` → コンパイルエラーなし確認。

---

## Phase B — AST 拡張

`fav/src/ast.rs` を編集：

### Pattern に追加するバリアント

```rust
// or-pattern: "a" | "b" | "c"
Or(Vec<Pattern>, Span),

// list-pattern: [] / [x] / [head, ..tail] / [a, b, ..rest]
List { head: Vec<Pattern>, tail: Option<String>, span: Span },
```

### MatchArm に guard フィールド追加

```rust
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,   // ← 追加
    pub body: Expr,
    pub span: Span,
}
```

変更に伴い `MatchArm` の構築箇所を全て更新（`guard: None` をデフォルト値に）。

---

## Phase C — パーサー拡張

`fav/src/frontend/parser.rs` を編集：

### or-pattern

`parse_match_arm` の `parse_pattern` 呼び出し後に `|` が続く場合に複数パターンを収集：

```
pattern1 | pattern2 | pattern3 => body
→ Pattern::Or(vec![pattern1, pattern2, pattern3])
```

注意: `|` はビット OR / パイプライン演算子と重複しないよう、`match` アーム文脈でのみ OR として解釈する。

### list-pattern

`[` で始まるパターンを解析：
- `[]` → `Pattern::List { head: [], tail: None }`
- `[x]` → `Pattern::List { head: [Bind("x")], tail: None }`
- `[head, ..tail]` → `Pattern::List { head: [Bind("head")], tail: Some("tail") }`
- `[a, b, ..rest]` → `Pattern::List { head: [Bind("a"), Bind("b")], tail: Some("rest") }`

`..name` は rest binding として `tail: Some("name")` に設定。

### guard 条件

`parse_match_arm` でパターン解析後、`if` キーワードがある場合に guard 式を解析：

```
Pattern if Expr => Body
```

---

## Phase D — 型チェッカー拡張

`fav/src/middle/checker.rs` を編集：

### Or パターンの型チェック

`check_pattern(Pattern::Or(pats), scrutinee_ty)`:
- 各パターンが `scrutinee_ty` と適合することを確認
- Or 内の全パターンが同じバインディング変数セットを持つことを確認

### List パターンの型チェック

`check_pattern(Pattern::List { head, tail }, scrutinee_ty)`:
- `scrutinee_ty` が `List<T>` であることを確認
- `head` の各要素が型 `T` にマッチすることを確認
- `tail` がある場合、その変数の型を `List<T>` として登録

### Guard の型チェック

`check_match_arm` で `arm.guard` がある場合:
- guard 式の型が `Bool` であることを確認

### 網羅性チェック拡張

`check_match_exhaustiveness` で Or / List パターンを考慮:
- `Pattern::Or(pats)` は各 `pat` を個別に展開して網羅性判定
- `Pattern::List { head: [], tail: None }` は空リストをカバー
- `Pattern::List { tail: Some(_) }` は 1 要素以上のリストをカバー

---

## Phase E — コンパイラ拡張

`fav/src/middle/compiler.rs` を編集：

### Or パターンのコンパイル

`compile_pattern(Pattern::Or(pats), ...)`:
各パターンを順次試行し、いずれかがマッチすれば成功とする opcode 列を生成。

```
Try pattern1:
  if match → jump to body
Try pattern2:
  if match → jump to body
...
→ no match
```

既存の `compile_match_arm` のフォールスルーロジックを拡張して実装。

### List パターンのコンパイル

`compile_pattern(Pattern::List { head, tail }, ...)`:

```
GetLength(scrutinee)
if tail is None:
  BranchIfNe(expected_len)    // 長さが head.len() でなければスキップ
else:
  BranchIfLt(head.len())      // 長さが head.len() 未満なら スキップ
GetIndex(0) → bind head[0]
GetIndex(1) → bind head[1]
...
if tail is Some(name):
  SliceFrom(head.len()) → bind tail
```

VM に `GetLength` / `GetIndex` / `SliceFrom` のプリミティブが必要か確認。
既存の `List.length` / リストアクセスを利用して実装する。

### Guard のコンパイル

`compile_match_arm` でパターンマッチ後に `arm.guard` がある場合:
guard 式を評価し、`false` なら次のアームへ飛ぶ分岐を追加。

---

## Phase F — exhaustive match 対応

新 Pattern バリアントへの exhaustive match 更新:
- `fav/src/middle/checker.rs` の全 `match pattern { ... }` を更新
- `fav/src/middle/compiler.rs` の全 `match pattern { ... }` を更新
- `fav/src/fmt.rs` の `fmt_pattern` を更新
- `fav/src/middle/ast_lower_checker.rs` の `lower_pattern` を更新

`cargo build` でエラーなし確認。

---

## Phase G — テスト追加（v172000_tests）

`fav/src/driver.rs` に `v172000_tests` モジュール追加（5 件）:

```rust
fn version_is_17_2_0()         // Cargo.toml に "17.2.0" が含まれる
fn or_pattern_string()         // "active" | "pending" => ... が動作
fn list_pattern_head_tail()    // [head, ..tail] 分解が動作
fn list_pattern_empty_single() // [] / [x] パターンが動作
fn match_guard()               // { amount: a } if a > 0.0 => ... が動作
```

`cargo test v172000` → 5/5 PASS 確認。

---

## Phase H — サイトドキュメント作成

`site/content/docs/language/patterns.mdx` を新規作成:
- or-pattern 構文説明と例
- list-pattern 構文説明（`[]` / `[x]` / `[head, ..tail]` / `[a, b, ..rest]`）
- guard 条件の構文説明
- Before / After 比較（煩雑な if-else vs パターンマッチ）
- 網羅性チェックの動作説明

---

## Phase I — 最終確認 + コミット

- `cargo test v172000` → 5/5 PASS 最終確認
- `cargo test` → 全件 PASS（リグレッションなし）
- コミット: `feat: v17.2.0 — パターンマッチ拡張（or-pattern / list-pattern / guard）`

---

## 依存関係

```
A（Cargo）→ G（テスト: version check）
B（AST）→ C（Parser）→ D（Checker）→ E（Compiler）→ F（exhaustive match）
F → G（テスト）
G → H（ドキュメント）→ I（コミット）
```

Phase A は独立して先行可能。
Phase B〜F は順次実施（後続が前段に依存）。

---

## 技術メモ

- **`|` の曖昧性**: `|` は Favnir でパイプライン演算子・ビット OR・クロージャ区切り・or-pattern に使われる。`parse_match_arm` 内でパターンを解析した後 `|` が来た場合にのみ or-pattern として扱う。式コンテキストでは従来通り。
- **`..tail` の字句解析**: `..` は既存の `DotDot` / `DotDotDot` トークンと重複する可能性がある。パターン内では `..Ident` の連続として認識する。
- **List パターンのコンパイル**: `GetIndex` は既存の VM 命令を活用。`SliceFrom` は `List.drop(n)` 相当のプリミティブとして実装するか、既存の `List.tail` / `List.drop` を利用する。
- **Guard のフォールスルー**: guard が `false` を返した場合、次のアームに進む（panic しない）。コンパイラはガード失敗時に次アームの先頭へジャンプする命令を生成する。
- **MatchArm.guard のデフォルト**: 既存の `MatchArm` 構築箇所すべてに `guard: None` を追加。
- **`v172000_tests` の `#[ignore]` 戦略**: 旧 `version_is_xxx` テストには `#[ignore = "historical version check"]` を付ける（v17.1.0 の前例に従う）。
