# v17.3.0 — コレクション内包表記 Spec

Date: 2026-06-15

## 概要

リスト変換・フィルタリングを `List.map` / `List.filter` のネストなしで書けるようにする。
Python の list comprehension に近い構文を Favnir に導入する。
実装はコンパイラが `List.filter` + `List.map` にデシュガーする方式。新 VM opcode は不要。

---

## 構文

### 基本（map）

```fav
let doubled = [x * 2 | x <- numbers]
```

`numbers` の各要素 `x` に `x * 2` を適用したリストを返す。`List.map(numbers, |x| x * 2)` に等価。

### フィルタ付き

```fav
let evens = [x | x <- numbers, x % 2 == 0]
```

ガード条件（カンマ区切り）を複数指定できる。

### 変換 + フィルタ

```fav
let valid_emails = [String.trim(s) | s <- raw_emails, String.contains(s, "@")]
```

### 複数ソース（直積）

```fav
let pairs = [Pair(a, b) | a <- as, b <- bs]
```

複数の `<-` 節は左から右にネストした `List.flat_map` に展開される。

### ネストしたリストの平坦化

```fav
let flat = [item | row <- matrix, item <- row]
```

### マップ内包

```fav
let counts = { k: List.length(v) | (k, v) <- Map.entries(grouped) }
```

`{ key_expr: val_expr | ... }` 形式。`Map.from_entries` への展開。

### Result 内包（エラー伝播）

```fav
let results: Result<List<Output>, String> = [? transform(row) | row <- rows]
```

`[? expr | ...]` 形式。いずれかの `expr` が `Result.err` なら全体が `Result.err` になる。

---

## Before / After 比較

```fav
// Before（v17.2 まで）
let valid_names =
  List.map(
    List.filter(rows, |r| String.length(String.trim(r.name)) > 0),
    |r| String.trim(r.name)
  )

// After（v17.3 以降）
let valid_names = [String.trim(r.name) | r <- rows, String.length(String.trim(r.name)) > 0]
```

---

## AST

### 追加 Node

```rust
// ast.rs に追加
Expr::ListComp {
    expr: Box<Expr>,
    clauses: Vec<CompClause>,
    span: Span,
}

Expr::MapComp {
    key: Box<Expr>,
    val: Box<Expr>,
    clauses: Vec<CompClause>,
    span: Span,
}

Expr::ResultComp {
    expr: Box<Expr>,
    clauses: Vec<CompClause>,
    span: Span,
}

enum CompClause {
    For { pat: Pattern, src: Box<Expr>, span: Span },
    Guard(Box<Expr>),
}
```

---

## デシュガー戦略

コンパイラ（`compiler.rs`）が AST → IR 変換時に展開する。

### 単一ソース + ガードなし → List.map

```fav
[f(x) | x <- xs]
// ↓
List.map(xs, |x| f(x))
```

### 単一ソース + ガード → List.filter + List.map（別 expr）または List.filter_map

```fav
[f(x) | x <- xs, pred(x)]
// ↓
List.map(List.filter(xs, |x| pred(x)), |x| f(x))
```

ガードのみ（`[x | x <- xs, pred(x)]`）は `List.filter` に展開。

### 複数ソース → flat_map のネスト

```fav
[f(a, b) | a <- as, b <- bs]
// ↓
List.flat_map(as, |a| List.map(bs, |b| f(a, b)))
```

### Result 内包 → fold + Result.map

```fav
[? f(x) | x <- xs]
// ↓
// 各 f(x) を評価し、最初の err で短絡する fold
// 実装: List.fold_result(xs, |x| f(x))
```

### マップ内包 → List.map + Map.from_entries

```fav
{ k: v | (k, v) <- Map.entries(m) }
// ↓
Map.from_entries(List.map(Map.entries(m), |(k, v)| (k, v)))
```

---

## 型チェック

`checker.rs` の `check_list_comp`:

1. 各 `For` 節のソース型を推論（`List<T>` を期待）
2. パターン変数の型を `T` に束縛してスコープに追加
3. `Guard` 節を `Bool` 型として型チェック
4. `expr` の型を推論 → 全体の型は `List<expr_type>`

`check_result_comp`:

1. 同上で各変数をスコープに追加
2. `expr` の型を `Result<T, E>` として型チェック
3. 全体の型は `Result<List<T>, E>`

---

## エラーコード

| コード | 意味 |
|---|---|
| E0327 | 内包表記のソースが List 型でない |
| E0328 | Result 内包の expr が Result 型でない |
| E0329 | マップ内包の節にタプルパターンが必要 |

---

## テスト（v173000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_17_3_0` | バージョン文字列が "17.3.0" であること |
| `list_comp_map` | `[x * 2 \| x <- ns]` が `List.map` 相当の結果を返す |
| `list_comp_filter` | `[x \| x <- ns, x > 0]` が `List.filter` 相当の結果を返す |
| `list_comp_multi_source` | `[Pair(a, b) \| a <- as, b <- bs]` が直積を返す |
| `result_comp_propagation` | `[? f(x) \| x <- xs]` のエラー伝播が動作する |

---

## 完了条件（PASS=5）

1. `[x * 2 | x <- numbers]` が `List.map` 相当の結果を返す
2. `[x | x <- numbers, x > 0]` が `List.filter` 相当の結果を返す
3. 複数ソース `[Pair(a, b) | a <- as, b <- bs]` が動作する
4. `[? transform(row) | row <- rows]` のエラー伝播が動作する
5. マップ内包 `{ k: v | (k, v) <- ... }` が動作する

---

## 非対応（スコープ外）

- `[x | x <- xs]` において `x` が `let` 束縛された変数の場合のスコープ解決（v17.4 の `let` と組み合わせて後続対応）
- 内包表記のネスト（`[[x | x <- row] | row <- matrix]`）— 動作はするが公式サポート外
- 非同期内包（`async/await` は未対応）
