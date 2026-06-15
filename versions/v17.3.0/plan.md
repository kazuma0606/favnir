# v17.3.0 — コレクション内包表記 実装計画

## 方針

デシュガー方式（コンパイラが `List.map` / `List.filter` に展開）を採用する。
新 VM opcode は不要。`ListComp` / `MapComp` / `ResultComp` を AST に追加し、
`compiler.rs` の `compile_expr` で既存の builtin 呼び出しに変換する。

パーサーが最も複雑（`[` と `{` の曖昧性、`|` 区切り）。他は機械的な追加で完結する。

---

## 実装ステップ

### Step 1: AST 拡張（`fav/src/ast.rs`）

`Expr` に 3 種の内包表記 variant を追加：

```rust
ListComp { expr: Box<Expr>, clauses: Vec<CompClause>, span: Span }
MapComp  { key: Box<Expr>, val: Box<Expr>, clauses: Vec<CompClause>, span: Span }
ResultComp { expr: Box<Expr>, clauses: Vec<CompClause>, span: Span }
```

`CompClause` enum を新規追加：

```rust
pub enum CompClause {
    For { pat: Pattern, src: Box<Expr>, span: Span },
    Guard(Box<Expr>),
}
```

### Step 2: Lexer（`fav/src/frontend/lexer.rs`）

追加トークンなし。`|` は既存の `Pipe` トークンを使用。
`<-` は既存の `LArrow` トークンを使用。

### Step 3: パーサー（`fav/src/frontend/parser.rs`）

#### `[` のパース分岐

`parse_primary` の `LBracket` ハンドラを拡張：

1. `[` を消費
2. `]` なら空リスト（既存）
3. `?` なら ResultComp モードへ
4. expr を 1 つパース
5. 次トークンを確認：
   - `|` (Pipe) → 内包表記モード（`parse_comp_clauses` を呼ぶ）
   - `,` → リストリテラル（現状は未対応のため、内包表記と区別する必要なし）
   - `]` → 単一要素リスト（既存）

#### `parse_comp_clauses`

```
`|` を消費
clause+ をパース（カンマ区切り）
  clause = `pat <- src` (For) または `expr` (Guard)
`]` を消費
```

`x <- expr` と `guard_expr` の区別:
- トークンを先読みして `<-`（`LArrow`）が現れれば `For` 節
- そうでなければ `Guard` 節

#### `{` のマップ内包

`parse_primary` の `LBrace` ハンドラ：

1. `{` を消費
2. `key_expr: val_expr |` の形を先読みで確認
3. `|` が来れば `parse_comp_clauses` → `MapComp`
4. そうでなければ従来のレコード/マップリテラル処理

### Step 4: 型チェッカー（`fav/src/middle/checker.rs`）

`infer_expr` に `Expr::ListComp` / `MapComp` / `ResultComp` を追加：

```
ListComp:
  For 節ごとにソース型を infer → List<T> でなければ E0327
  パターンを T 型で check_pattern_bindings
  Guard 節を Bool 型で check
  expr を infer → 結果型 List<expr_ty>

ResultComp:
  同上、expr を infer → Result<T, E> でなければ E0328
  全体型: Result<List<T>, E>

MapComp:
  key/val を infer
  全体型: Map<key_ty, val_ty>
```

### Step 5: コンパイラ（`fav/src/middle/compiler.rs`）

`compile_expr` に `Expr::ListComp` / `MapComp` / `ResultComp` を追加。

デシュガー関数 `desugar_list_comp(expr, clauses) -> IRExpr` を実装：

```
clauses を右から左に折り畳む:
  最右 For 節 + その後の Guard 群 → List.map + List.filter 呼び出し
  複数 For 節 → flat_map のネスト
```

`IRExpr` に変換せず、`Expr` レベルでデシュガーして `compile_expr` を再帰呼び出しする方が簡潔。

### Step 6: exhaustive match 対応

以下のファイルの `match expr { }` に `ListComp` / `MapComp` / `ResultComp` / `CompClause` を追加：

- `fav/src/fmt.rs`（pretty print）
- `fav/src/middle/ast_lower_checker.rs`
- `fav/src/emit_python.rs`（簡易対応でよい）
- `fav/src/driver.rs`（`remap_ir_pattern` など）

### Step 7: ドキュメント（`site/content/docs/language/comprehensions.mdx`）

内包表記の全構文・Before/After 比較・型チェック挙動を記述。

### Step 8: テスト（`fav/src/driver.rs`）

`v173000_tests` モジュールを追加（5件）。

---

## パーサーの曖昧性対処

### `[` の曖昧性

| 入力 | 判定 |
|---|---|
| `[x * 2 \| x <- ns]` | ListComp（`\|` が区切り） |
| `[]` | 空リスト |
| `[x]` | 既存の list-pattern / 単一要素 |

`parse_primary` の `LBracket` で：
1. expr をパース（`parse_expr` を呼ぶ、ただし `|` は優先度最低でストップ）
2. 次が `Pipe` なら ListComp、`]` なら終了

### `|` の優先度

内包表記中の `|` は区切り文字として機能するため、`parse_expr` が `|` で止まるよう
優先度テーブルを調整する。具体的には `parse_expr_with_prec` で
`Pipe` の優先度を内包表記コンテキストでは 0 にする（コンテキストフラグ）。

実装上は `parse_comp_body_expr`（`|` の前まで読む専用関数）を新設する方が安全。

### `<-` の区別

`x <- src` の `<-` は `LArrow`（既存トークン）。`bind x <- expr` と同じトークン。
`parse_comp_clauses` 内で `ident/pattern, <-, expr` の順を確認することで Guard と区別。

---

## デシュガー詳細

```
[f(x) | x <- xs]
→ List.map(xs, |x| f(x))

[x | x <- xs, pred(x)]
→ List.filter(xs, |x| pred(x))

[f(x) | x <- xs, pred(x)]
→ List.map(List.filter(xs, |x| pred(x)), |x| f(x))

[f(a, b) | a <- as, b <- bs]
→ List.flat_map(as, |a| List.map(bs, |b| f(a, b)))

[? f(x) | x <- xs]
→ 実装: fold_result として
  List.fold_result(xs, List.empty(), |acc, x|
    bind v <- f(x)
    Result.ok(List.push(acc, v))
  )
  ※ List.fold_result が未実装なら driver.rs の builtin として追加

{ k: v | (k, v) <- Map.entries(m) }
→ Map.from_entries(List.map(Map.entries(m), |(k, v)| Pair(k, v)))
```

---

## List.flat_map の対応

`List.flat_map` が stdlib にない場合は追加が必要：

```rust
// vm.rs call_builtin
"flat_map" => {
    let f = args[0].clone(); // closure
    let list = args[1].clone();
    // 各要素に f を適用して結果 List を concat
}
```

`compiler.rs` / `checker.rs` にも登録。

---

## 実装順序まとめ

1. `ast.rs` — `CompClause` + 3 variant 追加
2. `parser.rs` — `parse_comp_clauses` + `[` / `{` 分岐
3. `checker.rs` — `check_list_comp` / `check_result_comp`
4. `compiler.rs` — `compile_list_comp` デシュガー
5. `vm.rs` — `List.flat_map` / `List.fold_result` 追加（必要なら）
6. exhaustive match 追加（fmt / ast_lower_checker / emit_python / driver）
7. テスト追加
8. ドキュメント

---

## リスク・注意点

- **`|` の曖昧性**: or-pattern（v17.2）・ビット OR・内包表記区切りの 3 用途が `|` を使う。
  コンテキストで判断できるはずだが、パーサーのテストを丁寧に行う。
- **`List.flat_map`**: 未実装の場合はデシュガー先を変更するか追加する。
- **Result 内包のデシュガー**: `List.fold_result` が最も自然だが未実装。
  代替として `List.map` + `List.sequence` 方式もある。
- **マップ内包のタプルパターン**: `(k, v) <- ...` は v17.2 の list-pattern で `[k, v]` としてパースされる可能性あり（`(` はタプル）。Favnir にタプル型がない場合は内包表記専用のペアパターンが必要。
