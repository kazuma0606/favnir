# v14.3.0 Plan — 技術実装設計

Date: 2026-06-12

---

## 実装順序（Phase A → E）

```
A: fav/src/ast.rs — AzureStorage effect variant 追加
    ↓
B: fav/src/lineage.rs — AzureBlob 基盤 + CrossCloud 出力改善
    ↓
C: fav/src/middle/checker.rs — AzureStorage エフェクト登録
    ↓
D: fav/src/frontend/parser.rs — "!AzureStorage" パース対応
    ↓
E: fav/src/driver.rs — v143000_tests + Cargo.toml バンプ
```

---

## Phase A: `fav/src/ast.rs`

### A-1: `Effect` enum に `AzureStorage` 追加

`AzureDb` の直後（ast.rs:32 付近）に追加:

```rust
AzureDb,
AzureStorage,  // Azure Blob Storage (v14.3.0 infra, v14.5.0 primitives)
```

### A-2: `cargo build` でコンパイルエラーなし確認

`Effect` enum を追加すると match 網羅性エラーが出る可能性がある。
`format_effects` や `combined_effects` など全 match を確認して `_ => {}` または
明示的なアーム追加でカバーする。

---

## Phase B: `fav/src/lineage.rs`

### B-1: `format_effects` に `AzureStorage` 追加

`AzureDb => "!AzureDb".into()` の直後に:

```rust
AzureStorage => "!AzureStorage".into(),
```

### B-2: `collect_azure_blob_call_kinds` 関数追加

`collect_azure_call_kinds`（line ~318）の直後に追加:

```rust
// ── AzureBlob read/write classification (v14.3.0) ─────────────────────────

fn is_azure_blob_read_method(method: &str) -> bool {
    matches!(method, "get_raw" | "list_raw")
}

fn is_azure_blob_write_method(method: &str) -> bool {
    matches!(method, "put_raw" | "delete_raw")
}

/// Walk an expression tree and return `(has_read, has_write)` for AzureBlob calls.
/// - `AzureBlob.get_raw(...)`  / `AzureBlob.list_raw(...)`   → has_read
/// - `AzureBlob.put_raw(...)`  / `AzureBlob.delete_raw(...)` → has_write
pub fn collect_azure_blob_call_kinds(expr: &ast::Expr) -> (bool, bool) {
    let mut has_read = false;
    let mut has_write = false;
    collect_azure_blob_kinds_inner(expr, &mut has_read, &mut has_write);
    (has_read, has_write)
}

fn collect_azure_blob_kinds_inner(expr: &ast::Expr, r: &mut bool, w: &mut bool) {
    match expr {
        ast::Expr::Apply(func, args, _) => {
            if let ast::Expr::FieldAccess(obj, method, _) = func.as_ref() {
                if matches!(
                    obj.as_ref(),
                    ast::Expr::Ident(n, _) if n == "AzureBlob"
                ) {
                    if is_azure_blob_read_method(method) { *r = true; }
                    if is_azure_blob_write_method(method) { *w = true; }
                }
            }
            for a in args { collect_azure_blob_kinds_inner(a, r, w); }
            collect_azure_blob_kinds_inner(func, r, w);
        }
        ast::Expr::Block(blk) => {
            for s in &blk.stmts { collect_azure_blob_kinds_stmt(s, r, w); }
            collect_azure_blob_kinds_inner(&blk.expr, r, w);
        }
        ast::Expr::If(cond, then_blk, else_blk, _) => {
            collect_azure_blob_kinds_inner(cond, r, w);
            for s in &then_blk.stmts { collect_azure_blob_kinds_stmt(s, r, w); }
            collect_azure_blob_kinds_inner(&then_blk.expr, r, w);
            if let Some(b) = else_blk {
                for s in &b.stmts { collect_azure_blob_kinds_stmt(s, r, w); }
                collect_azure_blob_kinds_inner(&b.expr, r, w);
            }
        }
        ast::Expr::Match(scrutinee, arms, _) => {
            collect_azure_blob_kinds_inner(scrutinee, r, w);
            for arm in arms { collect_azure_blob_kinds_inner(&arm.body, r, w); }
        }
        ast::Expr::Pipeline(exprs, _) => {
            for e in exprs { collect_azure_blob_kinds_inner(e, r, w); }
        }
        ast::Expr::Closure(_, body, _) => { collect_azure_blob_kinds_inner(body, r, w); }
        ast::Expr::Collect(blk, _) => {
            for s in &blk.stmts { collect_azure_blob_kinds_stmt(s, r, w); }
            collect_azure_blob_kinds_inner(&blk.expr, r, w);
        }
        ast::Expr::BinOp(_, l, r2, _) => {
            collect_azure_blob_kinds_inner(l, r, w);
            collect_azure_blob_kinds_inner(r2, r, w);
        }
        ast::Expr::FieldAccess(obj, _, _) | ast::Expr::TypeApply(obj, _, _) => {
            collect_azure_blob_kinds_inner(obj, r, w);
        }
        ast::Expr::RecordConstruct(_, fields, _) => {
            for (_, v) in fields { collect_azure_blob_kinds_inner(v, r, w); }
        }
        ast::Expr::Unary(_, e, _)
        | ast::Expr::Return(e, _)
        | ast::Expr::AssertMatches(e, _, _)
        | ast::Expr::Question(e, _) => {
            collect_azure_blob_kinds_inner(e, r, w);
        }
        ast::Expr::Lit(_, _) | ast::Expr::Ident(_, _) | ast::Expr::FString(_, _) => {}
        _ => {}
    }
}

fn collect_azure_blob_kinds_stmt(stmt: &ast::Stmt, r: &mut bool, w: &mut bool) {
    match stmt {
        ast::Stmt::Bind(b) => collect_azure_blob_kinds_inner(&b.expr, r, w),
        ast::Stmt::Expr(e) => collect_azure_blob_kinds_inner(e, r, w),
        ast::Stmt::Chain(c) => collect_azure_blob_kinds_inner(&c.expr, r, w),
        ast::Stmt::Yield(y) => collect_azure_blob_kinds_inner(&y.expr, r, w),
        ast::Stmt::ForIn(f) => {
            collect_azure_blob_kinds_inner(&f.iter, r, w);
            for s in &f.body.stmts { collect_azure_blob_kinds_stmt(s, r, w); }
            collect_azure_blob_kinds_inner(&f.body.expr, r, w);
        }
    }
}
```

### B-3: `azure_storage_effects` 関数追加

`azure_db_effects` の直後に追加（パターン踏襲）:

```rust
/// Replace `!AzureStorage` with `!AzureStorage(read)` / `!AzureStorage(write)` where known.
fn azure_storage_effects(
    effects: &[ast::Effect],
    blob_read: bool,
    blob_write: bool,
) -> Vec<ast::Effect> {
    effects
        .iter()
        .flat_map(|e| {
            if matches!(e, ast::Effect::AzureStorage) && (blob_read || blob_write) {
                let mut out = Vec::new();
                if blob_read  { out.push(ast::Effect::AzureStorageRead); }
                if blob_write { out.push(ast::Effect::AzureStorageWrite); }
                out
            } else {
                vec![e.clone()]
            }
        })
        .collect()
}
```

**注意**: `AzureStorageRead` / `AzureStorageWrite` は別 variant として ast.rs に追加するか、
`AzureStorage(bool, bool)` のような enum にするか、または `combined_effects` の文字列出力で
`"!AzureStorage(read)"` / `"!AzureStorage(write)"` として処理するかを選択する。

→ **推奨**: 既存の `AzureDb` パターン（`AzureDbRead/Write` は enum variant ではなく文字列表現）と
同じ設計にする。`combined_effects` 内で文字列として出力する。

### B-4: `combined_effects` を 6 引数に拡張

現行シグネチャ:
```rust
fn combined_effects(
    effects: &[ast::Effect],
    sf_read: bool, sf_write: bool,
    az_read: bool, az_write: bool,
) -> Vec<String>
```

新シグネチャ:
```rust
fn combined_effects(
    effects: &[ast::Effect],
    sf_read: bool, sf_write: bool,
    az_db_read: bool, az_db_write: bool,
    az_blob_read: bool, az_blob_write: bool,
) -> Vec<String>
```

`!AzureStorage` の処理を追加:
```rust
if matches!(e, ast::Effect::AzureStorage) {
    if az_blob_read  { out.push("!AzureStorage(read)".into()); }
    if az_blob_write { out.push("!AzureStorage(write)".into()); }
    if !az_blob_read && !az_blob_write { out.push("!AzureStorage".into()); }
    continue;
}
```

`lineage_analysis` 内の 2 箇所（transformation + fn def）で `combined_effects` 呼び出しを更新:

```rust
let has_azure_blob = trf.effects.iter().any(|e| matches!(e, ast::Effect::AzureStorage));
let (az_blob_read, az_blob_write) = if has_azure_blob {
    collect_azure_blob_call_kinds(&ast::Expr::Block(Box::new(trf.body.clone())))
} else {
    (false, false)
};
// ...
effects: combined_effects(&trf.effects, sf_read, sf_write, az_read, az_write, az_blob_read, az_blob_write),
```

### B-5: `render_lineage_text` に CrossCloud Flow セクション追加

ソース/シンクラベルの強化と CrossCloud Flow セクションを追加する。
`Pipelines:` セクションの直前に追加:

```rust
// CrossCloud Flow: !Postgres(read) + !AzureDb(...) が共存する場合に出力
let has_aws_db = report.transformations.iter().any(|e| {
    e.effects.iter().any(|eff| {
        eff.contains("!Postgres") || eff.contains("!Db(read)") || eff.contains("!Snowflake")
    })
});
let has_azure_db = report.transformations.iter().any(|e| {
    e.effects.iter().any(|eff| eff.contains("!AzureDb"))
});

if has_aws_db && has_azure_db {
    out.push_str("CrossCloud Flow:\n");
    // パイプラインステップを順番に並べる
    let stages: Vec<String> = if !report.pipelines.is_empty() {
        report.pipelines[0].steps.clone()
    } else {
        report.transformations.iter().map(|e| e.name.clone()).collect()
    };
    let stages_str = stages.join(" → ");
    out.push_str(&format!("  [AWS RDS] → {} → [Azure Postgres]\n", stages_str));
    out.push('\n');
}
```

`LineageEntry` の `effects` フィールドが `Vec<String>` であることに注意。
`render_lineage_text` は `report` を参照するので、`LineageReport` の既存フィールドを使う。

---

## Phase C: `fav/src/middle/checker.rs`

### C-1: `BUILTIN_EFFECTS` に `AzureStorage` 追加

既存の `"!AzureDb"` の直後に:
```rust
"!AzureStorage" | "!AzureStorage(read)" | "!AzureStorage(write)",
```

パターンを確認して適切な場所に追加する。

### C-2: `str_to_effect` / エフェクト変換に `AzureStorage` 追加

checker.rs の `str_to_effect` または対応する match で:
```rust
"AzureStorage" | "!AzureStorage" => Some(ast::Effect::AzureStorage),
```

---

## Phase D: `fav/src/frontend/parser.rs` または `lexer.rs`

### D-1: `"!AzureStorage"` のパース対応

既存の `"!AzureDb"` パースと同じパターンで `AzureStorage` を追加。
`parse_effect` 関数（または対応する match）に追加:

```rust
"AzureStorage" => ast::Effect::AzureStorage,
```

---

## Phase E: `fav/src/driver.rs`

### E-1: `v143000_tests` モジュール追加（`v142000_tests` の直後推奨）

```rust
#[cfg(test)]
mod v143000_tests {
    use crate::lineage::{lineage_analysis, render_lineage_text};
    use crate::frontend::parser::Parser;

    #[test]
    fn version_is_14_3_0() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "14.3.0");
    }

    #[test]
    fn azure_db_lineage_collected() {
        // AzureDb エフェクトを持つ関数がリネージに収集されることを確認
        let src = r#"
public fn load_to_azure(conn_str: String) -> Result<Int, String> !AzureDb {
    AzurePostgres.execute_raw(conn_str, "INSERT INTO t VALUES ($1)", "[42]")
}
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let report = lineage_analysis(&prog);
        let entry = report.transformations.iter()
            .find(|e| e.name == "load_to_azure")
            .expect("load_to_azure not found in lineage");
        let has_azure_db = entry.effects.iter().any(|e| e.contains("AzureDb"));
        assert!(has_azure_db, "expected !AzureDb effect, got: {:?}", entry.effects);
    }

    #[test]
    fn crosscloud_lineage_format() {
        // !Postgres + !AzureDb が共存する場合に CrossCloud Flow が出力されることを確認
        let src = r#"
public fn extract(conn_str: String) -> Result<String, String> !Postgres {
    Postgres.query_raw(conn_str, "SELECT * FROM src", "[]")
}

public fn load(conn_str: String) -> Result<Int, String> !AzureDb {
    AzurePostgres.execute_raw(conn_str, "INSERT INTO dst VALUES ($1)", "[42]")
}
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let report = lineage_analysis(&prog);
        let text = render_lineage_text(&report, "test.fav");
        assert!(
            text.contains("CrossCloud Flow"),
            "expected CrossCloud Flow section, got:\n{}", text
        );
        assert!(
            text.contains("[AWS RDS]") && text.contains("[Azure Postgres]"),
            "expected crosscloud markers, got:\n{}", text
        );
    }
}
```

### E-2: `fav/Cargo.toml` バージョンバンプ

```toml
version = "14.3.0"
```

---

## 実装上の注意点

1. **`combined_effects` シグネチャ変更**: 2 箇所の呼び出し元（trf ループ + fn def ループ）を両方更新する。

2. **`LineageEntry.effects` の型**: `Vec<String>`（既存）。`render_lineage_text` はこの文字列を
   `contains("!AzureDb")` で判定するシンプルな方法で OK。

3. **ast.rs の `Effect` 追加による網羅性エラー**: `match e { ... }` パターンがある箇所
   （`format_effects`、`checker.rs`、`lineage.rs` など）で全追加が必要。
   コンパイルエラーを頼りに修正する。

4. **`collect_azure_blob_call_kinds` の `_ => {}`**: `ast::Expr` の variant は多い。
   既存 `collect_azure_kinds_inner` のパターンをそのまま流用し、`_ => {}` で残りをカバー。

5. **parser.rs の `AzureStorage` パース**: `.fav` ファイルに `!AzureStorage` を書いた場合に
   正しく `ast::Effect::AzureStorage` に変換されること。
   `runes/azure-blob/rune.fav`（v14.5.0 作成予定）でこの effect が使われる。

---

## 参照先ファイル（実装時に確認すること）

| ファイル | 参照目的 |
|---|---|
| `fav/src/ast.rs:28-40` | `Effect` enum — `AzureStorage` 追加箇所 |
| `fav/src/lineage.rs:60-75` | `format_effects` — `AzureDb` パターン |
| `fav/src/lineage.rs:304-420` | `collect_azure_call_kinds` — コピー元パターン |
| `fav/src/lineage.rs:590-620` | `combined_effects` — 拡張対象 |
| `fav/src/lineage.rs:620-750` | `lineage_analysis` — `combined_effects` 呼び出し箇所 |
| `fav/src/lineage.rs:793-886` | `render_lineage_text` — CrossCloud セクション追加箇所 |
| `fav/src/middle/checker.rs` | `BUILTIN_EFFECTS` + `str_to_effect` |
