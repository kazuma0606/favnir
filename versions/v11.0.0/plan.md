# Favnir v11.0.0 実装計画

Date: 2026-06-05
Theme: Snowflake 統合完成 + リネージ可視化 + ドキュメント整備

---

## Phase A: lineage.rs — `!Snowflake(read/write)` 区別

### A-1: `collect_snowflake_call_kinds` 追加

`lineage.rs` に追加（`collect_sql_literals_inner` の後）:

```rust
/// Inspect an expression tree and return (has_read, has_write) for Snowflake calls.
/// - `snowflake.query(...)` / `snowflake.query_raw(...)` → has_read
/// - `snowflake.execute(...)` / `snowflake.execute_raw(...)` → has_write
pub fn collect_snowflake_call_kinds(expr: &ast::Expr) -> (bool, bool) {
    let mut has_read = false;
    let mut has_write = false;
    collect_snowflake_kinds_inner(expr, &mut has_read, &mut has_write);
    (has_read, has_write)
}

fn is_snowflake_read_method(name: &str) -> bool {
    name == "query" || name == "query_raw"
}

fn is_snowflake_write_method(name: &str) -> bool {
    name == "execute" || name == "execute_raw"
}

fn collect_snowflake_kinds_inner(expr: &ast::Expr, r: &mut bool, w: &mut bool) {
    match expr {
        ast::Expr::Apply(func, args, _) => {
            if let ast::Expr::FieldAccess(obj, method, _) = func.as_ref() {
                let is_sf = matches!(obj.as_ref(),
                    ast::Expr::Ident(n, _) if n == "snowflake" || n == "Snowflake"
                );
                if is_sf {
                    if is_snowflake_read_method(method)  { *r = true; }
                    if is_snowflake_write_method(method) { *w = true; }
                }
            }
            for a in args { collect_snowflake_kinds_inner(a, r, w); }
            collect_snowflake_kinds_inner(func, r, w);
        }
        ast::Expr::Block(blk) => {
            for s in &blk.stmts { collect_snowflake_kinds_stmt(s, r, w); }
            collect_snowflake_kinds_inner(&blk.expr, r, w);
        }
        ast::Expr::If(cond, then_blk, else_blk, _) => {
            collect_snowflake_kinds_inner(cond, r, w);
            for s in &then_blk.stmts { collect_snowflake_kinds_stmt(s, r, w); }
            collect_snowflake_kinds_inner(&then_blk.expr, r, w);
            if let Some(b) = else_blk {
                for s in &b.stmts { collect_snowflake_kinds_stmt(s, r, w); }
                collect_snowflake_kinds_inner(&b.expr, r, w);
            }
        }
        ast::Expr::Match(scrutinee, arms, _) => {
            collect_snowflake_kinds_inner(scrutinee, r, w);
            for arm in arms { collect_snowflake_kinds_inner(&arm.body, r, w); }
        }
        ast::Expr::Pipeline(exprs, _) => {
            for e in exprs { collect_snowflake_kinds_inner(e, r, w); }
        }
        ast::Expr::Closure(_, body, _) => {
            collect_snowflake_kinds_inner(body, r, w);
        }
        ast::Expr::Collect(blk, _) => {
            for s in &blk.stmts { collect_snowflake_kinds_stmt(s, r, w); }
            collect_snowflake_kinds_inner(&blk.expr, r, w);
        }
        ast::Expr::BinOp(_, l, r2, _) => {
            collect_snowflake_kinds_inner(l, r, w);
            collect_snowflake_kinds_inner(r2, r, w);
        }
        ast::Expr::FieldAccess(obj, _, _) | ast::Expr::TypeApply(obj, _, _) => {
            collect_snowflake_kinds_inner(obj, r, w);
        }
        ast::Expr::RecordConstruct(_, fields, _) => {
            for (_, v) in fields { collect_snowflake_kinds_inner(v, r, w); }
        }
        ast::Expr::EmitExpr(e, _) | ast::Expr::AssertMatches(e, _, _) | ast::Expr::Question(e, _) => {
            collect_snowflake_kinds_inner(e, r, w);
        }
        ast::Expr::Lit(_, _) | ast::Expr::Ident(_, _) | ast::Expr::FString(_, _) => {}
    }
}

fn collect_snowflake_kinds_stmt(stmt: &ast::Stmt, r: &mut bool, w: &mut bool) {
    match stmt {
        ast::Stmt::Bind(b)  => collect_snowflake_kinds_inner(&b.expr, r, w),
        ast::Stmt::Expr(e)  => collect_snowflake_kinds_inner(e, r, w),
        ast::Stmt::Chain(c) => collect_snowflake_kinds_inner(&c.expr, r, w),
        ast::Stmt::Yield(y) => collect_snowflake_kinds_inner(&y.expr, r, w),
        ast::Stmt::ForIn(f) => {
            collect_snowflake_kinds_inner(&f.iter, r, w);
            for s in &f.body.stmts { collect_snowflake_kinds_stmt(s, r, w); }
            collect_snowflake_kinds_inner(&f.body.expr, r, w);
        }
    }
}
```

### A-2: `lineage_analysis` の TrfDef ループを更新

現状の TrfDef 処理（line ~280）に以下を追加:

```rust
// Snowflake read/write 区別
let has_snowflake = trf.effects.iter().any(|e| matches!(e, ast::Effect::Snowflake));
let (sf_read, sf_write) = if has_snowflake {
    collect_snowflake_call_kinds(&ast::Expr::Block(Box::new(trf.body.clone())))
} else {
    (false, false)
};
if sf_read  { sources.push(format!("({}:snowflake-read)",  trf.name)); }
if sf_write { sinks.push(format!("({}:snowflake-write)", trf.name)); }
```

effects の文字列生成も更新（`!Snowflake` を `!Snowflake(read)` / `!Snowflake(write)` に置換）:

```rust
effects: trf.effects.iter().flat_map(|e| {
    if matches!(e, ast::Effect::Snowflake) && (sf_read || sf_write) {
        let mut v = Vec::new();
        if sf_read  { v.push("!Snowflake(read)".to_string()); }
        if sf_write { v.push("!Snowflake(write)".to_string()); }
        v
    } else {
        vec![format_effects(std::slice::from_ref(e))]
    }
}).collect(),
```

同様の更新を FnDef ループにも適用。

### A-3: テスト追加

`lineage.rs` 末尾の `#[cfg(test)] mod tests` に 3 件追加:

```rust
#[test]
fn lineage_snowflake_write_stage_shows_write_label() {
    let src = r#"
stage Insert: List<String> -> Int !Snowflake = |rows| {
  snowflake.execute("INSERT INTO T VALUES (?)")
}
"#;
    let prog = crate::frontend::parser::parse(src).unwrap();
    let report = lineage_analysis(&prog);
    let entry = report.transformations.iter().find(|e| e.name == "Insert").unwrap();
    assert!(entry.effects.contains(&"!Snowflake(write)".to_string()));
    assert!(entry.sinks.iter().any(|s| s.contains("snowflake-write")));
}

#[test]
fn lineage_snowflake_read_stage_shows_read_label() {
    let src = r#"
stage Query: String -> List<String> !Snowflake = |sql| {
  snowflake.query(sql)
}
"#;
    let prog = crate::frontend::parser::parse(src).unwrap();
    let report = lineage_analysis(&prog);
    let entry = report.transformations.iter().find(|e| e.name == "Query").unwrap();
    assert!(entry.effects.contains(&"!Snowflake(read)".to_string()));
    assert!(entry.sources.iter().any(|s| s.contains("snowflake-read")));
}

#[test]
fn lineage_snowflake_undistinguished_falls_back() {
    let src = r#"
stage Sf: String -> String !Snowflake = |x| { x }
"#;
    let prog = crate::frontend::parser::parse(src).unwrap();
    let report = lineage_analysis(&prog);
    let entry = report.transformations.iter().find(|e| e.name == "Sf").unwrap();
    assert!(entry.effects.contains(&"!Snowflake".to_string()));
}
```

---

## Phase B: CHANGELOG.md 更新

`CHANGELOG.md` の先頭（`[v10.0.0]` の前）に v10.1.0〜v11.0.0 を追記:

```markdown
## [v11.0.0] — 2026-06-05

### Added
- `fav explain --lineage` で `!Snowflake(read)` / `!Snowflake(write)` を区別表示
- `site/content/docs/runes/snowflake.mdx` — Snowflake Rune リファレンスページ

### Changed
- README.md の Rune エコシステム表に `snowflake`（`!Snowflake` エフェクト）を追加
- CHANGELOG に v10.1.0〜v10.9.0 全履歴を追記

### Notes
- テスト: 1286 件（lineage Snowflake 区別テスト 3 件追加）

---

## [v10.9.0] — 2026-06-05
（以下 v10.1.0 まで順に追記）
```

---

## Phase C: README.md 更新

```markdown
| **Rune エコシステム** | AWS / DuckDB / SQL / DB / fs / Parquet | ✓ |
| | http / grpc / graphql（`!Http` エフェクト） | ✓ |
| | llm（`!Llm` エフェクト、Claude / OpenAI） | ✓ |
| | snowflake（`!Snowflake` エフェクト） | ✓ |  ← 追加
```

ロードマップ表末尾に追記:

```markdown
| v10.1.0〜v10.9.0 | Snowflake ネイティブ対応（インフラ〜E2E デモ） | 完了 |
| v11.0.0 | Snowflake 統合完成宣言・リネージ可視化・サイトドキュメント | 完了 |
```

---

## Phase D: site/content/docs/runes/snowflake.mdx

`aws.mdx` と同構造で新規作成。主なセクション:

1. **概要** — Snowflake SQL API v2 (REST) / JWT RS256 認証
2. **インストール** — `import rune "snowflake"`
3. **fav.toml 設定** — `[snowflake]` セクション（account / user / warehouse / role / database / schema）
4. **環境変数** — `SNOWFLAKE_ACCOUNT` / `SNOWFLAKE_USER` / `SNOWFLAKE_PRIVATE_KEY` / `SNOWFLAKE_ROLE` / `SNOWFLAKE_WAREHOUSE`
5. **API リファレンス**
   - `execute(sql: String) -> Int !Snowflake` — DML 実行（影響行数を返す）
   - `query<T>(sql: String) -> List<T> !Snowflake` — SELECT クエリ
6. **fav infer** — `fav infer --from snowflake --table <TABLE_NAME>`
7. **fav explain --lineage** — `!Snowflake(read)` / `!Snowflake(write)` の出力例
8. **完全なコード例** — CSV → Snowflake INSERT → 集計クエリ → S3 出力

---

## Phase E: バージョン更新

- `fav/Cargo.toml`: `version = "11.0.0"`
- `fav/self/cli.fav`: `IO.println("favnir 11.0.0 (self-host CLI)")`

---

## Phase F: self-check + cargo test

1. `fav check --legacy-check self/compiler.fav` — エラーなし
2. `cargo test bootstrap` — 通過
3. `cargo test` — 1286 件全件通過
