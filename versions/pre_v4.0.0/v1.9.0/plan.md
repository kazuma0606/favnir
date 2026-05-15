# Favnir v1.9.0 実装プラン

作成日: 2026-05-09

---

## Phase 0 — バージョン更新

```toml
# Cargo.toml
version = "1.9.0"
```

```rust
// main.rs
const HELP: &str = "fav v1.9.0 ...";
```

---

## Phase 1 — `for` 式

### 字句解析 (`src/frontend/lexer.rs`)

```rust
// TokenKind に追加
For,
In,

// キーワードマッピング
"for" => TokenKind::For,
"in"  => TokenKind::In,
```

### AST (`src/ast.rs`)

```rust
// Stmt に追加
ForIn {
    var:  String,
    iter: Expr,
    body: Block,
    span: Span,
},
```

### パーサー (`src/frontend/parser.rs`)

```rust
// parse_stmt 内
TokenKind::For => {
    Ok(Stmt::ForIn(self.parse_for_in()?))
}

fn parse_for_in(&mut self) -> Result<(String, Expr, Block, Span), ParseError> {
    let start = self.peek_span().clone();
    self.expect(&TokenKind::For)?;
    let var = self.expect_ident()?;
    self.expect(&TokenKind::In)?;
    let iter = self.parse_expr()?;
    let body = self.parse_block()?;
    Ok(ForIn { var, iter, body, span: self.span_from(&start) })
}
```

### 型検査 (`src/middle/checker.rs`)

```rust
// check_stmt 内
Stmt::ForIn { var, iter, body, span } => {
    let iter_ty = self.check_expr(iter);
    // E067: collect 内の for は未対応
    if self.in_collect {
        self.error(span, "E067", "for in collect block not supported in v1.9.0");
        return;
    }
    // E065: イテレータが List<T> でなければエラー
    let elem_ty = match iter_ty {
        Type::List(inner) => *inner,
        _ => {
            self.error(span, "E065", "for iterator must be List<T>");
            Type::Error
        }
    };
    self.env.push();
    self.env.define(var, elem_ty);
    let body_ty = self.check_block(body);
    // E066: ボディが Unit でなければエラー
    if !matches!(body_ty, Type::Unit | Type::Error) {
        self.error(span, "E066", "for body must return Unit");
    }
    self.env.pop();
}
```

### コンパイラ (`src/middle/compiler.rs`)

`for x in list { body }` を以下にデシュガーしてコンパイル:

```
List.fold(list, Unit, |_, x| { body; Unit })
```

具体的には:
1. `iter` を評価して List を得る
2. `List.fold` の IRExpr を構築
3. クロージャ引数 `_` (Unit) と `x` (elem_ty) を env に登録
4. `body` を IRStmt として展開
5. クロージャ返り値は Unit

### フォーマッタ (`src/fmt.rs`)

```rust
// Stmt::ForIn
Stmt::ForIn { var, iter, body, .. } => {
    let iter_s = self.expr(iter);
    let body_s = self.block(body);
    format!("for {} in {} {}", var, iter_s, body_s)
}
```

### リント (`src/lint.rs`)

- `ForIn` の `iter` と `body` を使用収集・バインド収集に追加

---

## Phase 2 — `??` 演算子

### 字句解析 (`src/frontend/lexer.rs`)

```rust
// TokenKind に追加
QuestionQuestion,

// lex_char 内: '?' を見たとき、次も '?' なら QuestionQuestion
'?' => {
    if self.pos < self.source.len() && self.source[self.pos] == '?' {
        self.pos += 1;
        tokens.push(Token { kind: TokenKind::QuestionQuestion, span });
    } else {
        tokens.push(Token { kind: TokenKind::Question, span });
    }
}
```

### AST (`src/ast.rs`)

```rust
// BinOp に追加
NullCoalesce,  // ??
```

### パーサー (`src/frontend/parser.rs`)

`??` は最低優先順位の二項演算子として追加:

```rust
// parse_binop_expr の最外ループに追加（最後に確認）
if self.peek() == &TokenKind::QuestionQuestion {
    self.advance();
    let rhs = self.parse_binop_expr()?;
    lhs = Expr::BinOp(BinOp::NullCoalesce, Box::new(lhs), Box::new(rhs), span);
}
```

### 型検査 (`src/middle/checker.rs`)

```rust
// check_binop 内
BinOp::NullCoalesce => {
    // lhs: Option<T>、rhs: T → 結果: T
    match &lhs_ty {
        Type::Option(inner) => {
            let t = *inner.clone();
            self.unify(&rhs_ty, &t, span, "E069");
            t
        }
        _ => {
            self.error(span, "E068", "?? left-hand side must be Option<T>");
            Type::Error
        }
    }
}
```

### コンパイラ (`src/middle/compiler.rs`)

`expr1 ?? expr2` を以下にデシュガー:

```
Option.unwrap_or(expr1, expr2)
```

具体的には IRExpr::Call(FieldAccess(Global("Option"), "unwrap_or"), [expr1, expr2]) を生成。

### フォーマッタ (`src/fmt.rs`)

```rust
BinOp::NullCoalesce => format!("{} ?? {}", lhs_s, rhs_s),
```

---

## Phase 3 — `stage`/`seq` エイリアス

### 字句解析 (`src/frontend/lexer.rs`)

```rust
// TokenKind に追加
Stage,  // trf のエイリアス
Seq,    // flw のエイリアス

// キーワードマッピング
"stage" => TokenKind::Stage,
"seq"   => TokenKind::Seq,
```

### パーサー (`src/frontend/parser.rs`)

```rust
// parse_item 内
TokenKind::Trf | TokenKind::Stage => {
    Ok(Item::TrfDef(self.parse_trf_def()?))
}
TokenKind::Flw | TokenKind::Seq => {
    Ok(Item::FlwDef(self.parse_flw_def()?))
}
```

`parse_trf_def` / `parse_flw_def` は変更なし。
AST に keyword フィールドは追加しない（セマンティクスは完全同一）。

### abstract stage/seq (`src/frontend/parser.rs`)

```rust
// parse_abstract_item 内
TokenKind::Trf | TokenKind::Stage => { /* abstract trf / abstract stage */ }
TokenKind::Flw | TokenKind::Seq   => { /* abstract flw / abstract seq   */ }
```

### 変更が不要なもの

- checker.rs: `Item::TrfDef` / `Item::FlwDef` をそのまま使う（キーワードは関係なし）
- compiler.rs: 同上
- vm.rs: 同上
- lint.rs: 同上

### フォーマッタ (`src/fmt.rs`)

フォーマット時は常に `trf` / `flw` を使う（v2.0.0 で `stage`/`seq` に切り替える）。

---

## Phase 4 — Coverage HTML 出力

### `src/driver.rs`

```rust
/// HTML index ページを生成する
fn format_coverage_html_index(
    file_reports: &[(String, usize, usize, f64)],  // (path, covered, total, pct)
) -> String {
    // シンプルな HTML テーブル
}

/// ソースファイルの HTML 注釈ページを生成する
fn format_coverage_html_file(
    path: &str,
    source: &str,
    executed: &HashSet<u32>,
) -> String {
    // 各行を <div class="line covered/uncovered"> で囲む
}

/// ファイル名を HTML ファイル名に変換 (path separators → underscore)
fn sanitize_html_filename(path: &str) -> String {
    path.replace(['/', '\\', ':'], "_") + ".html"
}
```

`cmd_test` の coverage_report_dir ブランチを拡張:

```rust
if let Some(dir) = coverage_report_dir {
    std::fs::create_dir_all(dir)?;
    // 既存: coverage.txt
    // NEW: index.html
    let html_index = format_coverage_html_index(&file_reports);
    std::fs::write(Path::new(dir).join("index.html"), html_index)?;
    // NEW: per-file HTML
    for (path, source, executed) in &file_data {
        let html = format_coverage_html_file(path, source, executed);
        let fname = sanitize_html_filename(path);
        std::fs::write(Path::new(dir).join(&fname), html)?;
    }
}
```

---

## Phase 5 — `fav bench` 統計強化

### `src/driver.rs`

```rust
pub struct BenchStats {
    pub mean_us:   f64,
    pub min_us:    f64,
    pub max_us:    f64,
    pub stddev_us: f64,
    pub p50_us:    f64,
    pub iters:     u64,
}

fn compute_bench_stats(samples: &[f64]) -> BenchStats {
    let n = samples.len() as f64;
    let mean = samples.iter().sum::<f64>() / n;
    let variance = samples.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / n;
    let stddev = variance.sqrt();
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = sorted[sorted.len() / 2];
    BenchStats {
        mean_us: mean,
        min_us: *sorted.first().unwrap(),
        max_us: *sorted.last().unwrap(),
        stddev_us: stddev,
        p50_us: p50,
        iters: samples.len() as u64,
    }
}
```

`exec_bench_case` の戻り値を `f64`（mean）から `Vec<f64>`（全サンプル）に変更:

```rust
fn exec_bench_case(prog: &ast::Program, description: &str, iters: u64) -> Result<Vec<f64>, String> {
    // warmup 1 回
    // timed: 各イテレーションを個別計測して Vec<f64> に push
}
```

`cmd_bench` に `compact: bool` / `json_output: bool` を追加。

### `src/main.rs`

```rust
// bench コマンドに --compact と --json フラグを追加
Some("bench") => {
    let mut compact = false;
    let mut json_output = false;
    // ... parse flags
    cmd_bench(file.as_deref(), filter.as_deref(), iters, compact, json_output);
}
```

---

## Phase 6 — テスト・ドキュメント

### 追加するテスト

新規テストは `src/driver.rs` の `tests` モジュールに追加する。

- `for_in_io_context_iterates_list`
- `for_in_pure_context_unit_result`
- `for_non_list_iter_errors_e065`
- `for_in_collect_block_errors_e067`
- `null_coalesce_returns_value_when_some`
- `null_coalesce_returns_default_when_none`
- `null_coalesce_chained`
- `null_coalesce_lhs_non_option_errors_e068`
- `stage_keyword_parses_like_trf`
- `seq_keyword_parses_like_flw`
- `trf_and_stage_coexist`
- `coverage_html_index_created`
- `coverage_html_file_created`
- `coverage_html_contains_percentage`
- `bench_stats_compute_mean`
- `bench_stats_compute_stddev`
- `bench_stats_compute_p50`
- `bench_compact_format`

### example ファイル

- `examples/for_demo.fav`
- `examples/coalesce_demo.fav`
- `examples/stage_seq_demo.fav`

### ドキュメント

- `versions/v1.9.0/langspec.md` 新規作成
- `README.md` に v1.9.0 セクション追加

---

## ファイル変更一覧

| ファイル | Phase | 変更内容 |
|---|---|---|
| `Cargo.toml` | 0 | version = "1.9.0" |
| `src/main.rs` | 0, 5 | HELP v1.9.0; bench --compact/--json フラグ追加 |
| `src/ast.rs` | 1, 2 | `Stmt::ForIn`; `BinOp::NullCoalesce` |
| `src/frontend/lexer.rs` | 1, 2, 3 | For/In/QuestionQuestion/Stage/Seq トークン追加 |
| `src/frontend/parser.rs` | 1, 2, 3 | parse_for_in; ?? 演算子; stage/seq → trf/flw |
| `src/middle/checker.rs` | 1, 2 | ForIn/NullCoalesce 型検査 |
| `src/middle/compiler.rs` | 1, 2 | ForIn/NullCoalesce デシュガー |
| `src/fmt.rs` | 1, 2 | ForIn/NullCoalesce フォーマット |
| `src/lint.rs` | 1 | ForIn バインド/使用収集 |
| `src/driver.rs` | 4, 5 | HTML生成; BenchStats; exec_bench_case変更; cmd_bench変更 |
| `examples/for_demo.fav` | 6 | 新規 |
| `examples/coalesce_demo.fav` | 6 | 新規 |
| `examples/stage_seq_demo.fav` | 6 | 新規 |
| `versions/v1.9.0/langspec.md` | 6 | 新規 |
| `README.md` | 6 | v1.9.0 セクション追加 |

---

## 実装順序と依存関係

```
Phase 0 (バージョン更新)
  ↓
Phase 1 (for 式) ── lexer → parser → checker → compiler → fmt → lint
Phase 2 (?? 演算子) ── lexer → parser → checker → compiler → fmt
Phase 3 (stage/seq) ── lexer → parser のみ (checker/compiler は変更なし)
  ↓ (3つは並列可能)
Phase 4 (Coverage HTML) ── driver.rs のみ
Phase 5 (bench 統計) ── driver.rs のみ (exec_bench_case の型変更に注意)
  ↓
Phase 6 (テスト・ドキュメント)
```

Phase 1〜3 は互いに独立しているため並列で実装できる。
Phase 4/5 は driver.rs を共有するが、変更箇所が異なるため競合しにくい。
