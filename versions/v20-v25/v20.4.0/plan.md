# v20.4.0 実装計画 — DuckDB プッシュダウン最適化パス

## 実装順序

```
T1: src/pushdown/mod.rs — PushdownPlan 型定義 + detect_pushdown エントリ   ← 最初
T2: src/pushdown/pattern.rs — AST パターンマッチャー（5 パターン）          ← T1 完了後
T3: src/pushdown/sql_builder.rs — PushdownPlan → SQL 文字列                ← T1 完了後（T2 と並列可）
T4: compiler.rs — compile_trf_def にプッシュダウン試行を統合               ← T2/T3 完了後
T5: vm.rs — __duckdb_push builtin + PushdownFallback エラー処理             ← T4 完了後
T6: driver.rs — --explain-pushdown フラグ + v204000_tests                  ← T5 完了後
T7: Cargo.toml バージョン更新                                               ← 任意
T8: CHANGELOG.md + benchmarks/v20.4.0.json                                 ← T6 完了後
```

**変更ファイル一覧:**
- `fav/src/pushdown/mod.rs`（T1）— 新規作成
- `fav/src/pushdown/pattern.rs`（T2）— 新規作成
- `fav/src/pushdown/sql_builder.rs`（T3）— 新規作成
- `fav/src/compiler.rs`（T4）
- `fav/src/backend/vm.rs`（T5）
- `fav/src/driver.rs`（T6）
- `fav/src/lib.rs`（T1 — `mod pushdown;` 追加）
- `fav/Cargo.toml`（T7）

---

## T1: `src/pushdown/mod.rs` — PushdownPlan 型定義

`fav/src/pushdown/mod.rs`、`fav/src/pushdown/pattern.rs`（空スタブ）、
`fav/src/pushdown/sql_builder.rs`（空スタブ）を作成する。
`fav/src/lib.rs` に `mod pushdown;` を追加する。

### 型定義

```rust
// fav/src/pushdown/mod.rs

pub mod pattern;
pub mod sql_builder;

use crate::ast::Expr;

/// フィルタ条件の内部表現
#[derive(Debug, Clone)]
pub enum FilterExpr {
    FieldCmp { field: String, op: CmpOp, value: SqlLiteral },
    And(Box<FilterExpr>, Box<FilterExpr>),
    Or(Box<FilterExpr>, Box<FilterExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CmpOp { Gt, Ge, Lt, Le, Eq, Ne }

#[derive(Debug, Clone)]
pub enum SqlLiteral {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}

/// プッシュダウン可能な操作（Phase 1: 5 種）
#[derive(Debug, Clone)]
pub enum PushdownOp {
    Filter(FilterExpr),
    Project(Vec<String>),
    GroupBy(String),
    SumBy(String),
    Count,
}

/// 検出されたプッシュダウン計画
#[derive(Debug, Clone)]
pub struct PushdownPlan {
    pub sql: String,
    pub op: PushdownOp,
}

/// stage body を解析してプッシュダウン計画を返す。
/// マッチしない場合は None。
pub fn detect_pushdown(body: &Expr, param_name: &str) -> Option<PushdownPlan> {
    use pattern::*;
    use sql_builder::build_sql;

    // Count: List.length(param)
    if analyze_count(body, param_name) {
        let op = PushdownOp::Count;
        return Some(PushdownPlan { sql: build_sql(&op), op });
    }
    // Filter: List.filter(param, |r| ...)
    if let Some(filter_expr) = analyze_filter(body, param_name) {
        let op = PushdownOp::Filter(filter_expr);
        return Some(PushdownPlan { sql: build_sql(&op), op });
    }
    // Project: List.map(param, |r| { ... })
    if let Some(fields) = analyze_project(body, param_name) {
        let op = PushdownOp::Project(fields);
        return Some(PushdownPlan { sql: build_sql(&op), op });
    }
    // GroupBy: List.group_by(param, |r| r.key)
    if let Some(key) = analyze_group_by(body, param_name) {
        let op = PushdownOp::GroupBy(key);
        return Some(PushdownPlan { sql: build_sql(&op), op });
    }
    // SumBy: List.sum_by(param, |r| r.val)
    if let Some(field) = analyze_sum_by(body, param_name) {
        let op = PushdownOp::SumBy(field);
        return Some(PushdownPlan { sql: build_sql(&op), op });
    }
    None
}
```

### 完了条件
- `cargo check` でコンパイルエラー 0

---

## T2: `src/pushdown/pattern.rs` — AST パターンマッチャー

`fav/src/pushdown/pattern.rs` を実装する。

### マッチするヘルパー関数群

> **実装注意 — 正式 AST 型名（`ExprKind` は存在しない）:**
> - `Expr::Closure(Vec<String>, Box<Expr>, Span)` — Lambda 相当。`params[0]` がパラメータ名。
> - `Expr::BinOp(BinOp, Box<Expr>, Box<Expr>, Span)` — 二項演算。
> - `BinOp::And / Or / Gt / Ge / Lt / Le / Eq / Ne` （`BinOpKind` は存在しない）。
> - `Expr::Apply(Box<Expr>, Vec<Expr>, Span)` — 関数適用。
> - `Expr::FieldAccess(Box<Expr>, String, Span)` — フィールドアクセス。
> - `Expr::Ident(String, Span)` — 変数参照。
> - リテラルは `Lit`（`Literal` ではない）。実装前に `grep -n "^pub enum Expr" fav/src/ast.rs` で確認。

```rust
use crate::ast::{Expr, BinOp, Lit};
use super::{FilterExpr, CmpOp, SqlLiteral};

/// `List.filter(param, |r| body)` にマッチ
pub fn analyze_filter(expr: &Expr, param: &str) -> Option<FilterExpr> {
    // Expr::Apply(FieldAccess(Ident("List"), "filter"), [Ident(param), Closure(...)], _)
    let args = extract_list_call(expr, "filter")?;
    if args.len() != 2 { return None; }
    is_param_ident(&args[0], param)?;
    // args[1] が Closure
    if let Expr::Closure(params, body, _) = &args[1] {
        let lp = params.first().map(|s| s.as_str())?;
        analyze_filter_expr(body, lp)
    } else {
        None
    }
}

/// フィルタ lambda body を FilterExpr に変換
fn analyze_filter_expr(body: &Expr, lp: &str) -> Option<FilterExpr> {
    match body {
        Expr::BinOp(op, left, right, _) => {
            // AND / OR の複合条件
            if *op == BinOp::And {
                let l = analyze_filter_expr(left, lp)?;
                let r = analyze_filter_expr(right, lp)?;
                return Some(FilterExpr::And(Box::new(l), Box::new(r)));
            }
            if *op == BinOp::Or {
                let l = analyze_filter_expr(left, lp)?;
                let r = analyze_filter_expr(right, lp)?;
                return Some(FilterExpr::Or(Box::new(l), Box::new(r)));
            }
            // 比較演算子: left は lp.field、right はリテラル
            let cmp_op = binop_to_cmp(op)?;
            let field = extract_field_access(left, lp)?;
            let value = extract_literal(right)?;
            Some(FilterExpr::FieldCmp { field, op: cmp_op, value })
        }
        _ => None,
    }
}

/// `List.map(param, |r| { f1: r.f1, f2: r.f2 })` にマッチ → フィールド名リスト
pub fn analyze_project(expr: &Expr, param: &str) -> Option<Vec<String>> {
    let args = extract_list_call(expr, "map")?;
    if args.len() != 2 { return None; }
    is_param_ident(&args[0], param)?;
    if let Expr::Closure(params, body, _) = &args[1] {
        let lp = params.first().map(|s| s.as_str())?;
        extract_projection_fields(body, lp)
    } else {
        None
    }
}

/// `List.group_by(param, |r| r.key)` にマッチ → key フィールド名
pub fn analyze_group_by(expr: &Expr, param: &str) -> Option<String> {
    let args = extract_list_call(expr, "group_by")?;
    if args.len() != 2 { return None; }
    is_param_ident(&args[0], param)?;
    if let Expr::Closure(params, body, _) = &args[1] {
        let lp = params.first().map(|s| s.as_str())?;
        extract_field_access(body, lp)
    } else {
        None
    }
}

/// `List.sum_by(param, |r| r.val)` にマッチ → field 名
pub fn analyze_sum_by(expr: &Expr, param: &str) -> Option<String> {
    let args = extract_list_call(expr, "sum_by")?;
    if args.len() != 2 { return None; }
    is_param_ident(&args[0], param)?;
    if let Expr::Closure(params, body, _) = &args[1] {
        let lp = params.first().map(|s| s.as_str())?;
        extract_field_access(body, lp)
    } else {
        None
    }
}

/// `List.length(param)` にマッチ
pub fn analyze_count(expr: &Expr, param: &str) -> bool {
    if let Some(args) = extract_list_call(expr, "length") {
        args.len() == 1 && matches!(&args[0], Expr::Ident(n, _) if n == param)
    } else {
        false
    }
}

// ──── ヘルパー ────────────────────────────────────────────────

/// `List.method(...)` 形式の Apply を分解し args を返す。
/// Expr::Apply(FieldAccess(Ident("List"), method), args, _) にマッチ。
fn extract_list_call<'a>(expr: &'a Expr, expected_method: &str) -> Option<Vec<Expr>>;

/// `lp.field` 形式（Expr::FieldAccess(Ident(lp), field, _)）から field 名を取得する。
fn extract_field_access(expr: &Expr, lp: &str) -> Option<String>;

/// `Expr::Lit(lit, _)` から SqlLiteral に変換する。
fn extract_literal(expr: &Expr) -> Option<SqlLiteral>;

/// `BinOp` を CmpOp に変換する。And/Or は None。
fn binop_to_cmp(op: &BinOp) -> Option<CmpOp>;

/// expr が `Expr::Ident(param, _)` であることを確認する（Some(()) を返す）。
fn is_param_ident(expr: &Expr, param: &str) -> Option<()>;

/// record 式 `{ f1: lp.f1, f2: lp.f2 }` からフィールド名リストを抽出する。
fn extract_projection_fields(body: &Expr, lp: &str) -> Option<Vec<String>>;
```

### 完了条件
- `cargo check` でコンパイルエラー 0
- `pushdown_detect_filter` ユニットテスト（`src/pushdown/pattern.rs` 内部テスト）

---

## T3: `src/pushdown/sql_builder.rs` — SQL 生成

```rust
// fav/src/pushdown/sql_builder.rs

use super::{PushdownOp, FilterExpr, CmpOp, SqlLiteral};

/// PushdownOp から SQL 文字列を生成する。
/// テーブル名プレースホルダーは実行時に `_batch_{id}` で置換される。
pub fn build_sql(op: &PushdownOp) -> String {
    match op {
        PushdownOp::Count =>
            "SELECT COUNT(*) FROM ?pushdown_table?".to_string(),

        PushdownOp::SumBy(field) =>
            format!("SELECT SUM({field}) FROM ?pushdown_table?"),

        // GroupBy 単体は DISTINCT 相当（COUNT は Phase 2 以降）
        PushdownOp::GroupBy(key) =>
            format!("SELECT DISTINCT {key} FROM ?pushdown_table?"),

        PushdownOp::Project(fields) =>
            format!("SELECT {} FROM ?pushdown_table?", fields.join(", ")),

        PushdownOp::Filter(filter_expr) =>
            format!("SELECT * FROM ?pushdown_table? WHERE {}",
                    build_filter_where(filter_expr)),
    }
}

fn build_filter_where(expr: &FilterExpr) -> String {
    match expr {
        FilterExpr::FieldCmp { field, op, value } =>
            format!("{field} {} {}", build_cmp_op(op), build_literal(value)),

        FilterExpr::And(l, r) =>
            format!("({}) AND ({})", build_filter_where(l), build_filter_where(r)),

        FilterExpr::Or(l, r) =>
            format!("({}) OR ({})", build_filter_where(l), build_filter_where(r)),
    }
}

fn build_cmp_op(op: &CmpOp) -> &'static str {
    match op {
        CmpOp::Gt => ">",  CmpOp::Ge => ">=",
        CmpOp::Lt => "<",  CmpOp::Le => "<=",
        CmpOp::Eq => "=",  CmpOp::Ne => "<>",
    }
}

fn build_literal(lit: &SqlLiteral) -> String {
    match lit {
        SqlLiteral::Int(n)   => n.to_string(),
        SqlLiteral::Float(f) => f.to_string(),
        SqlLiteral::Bool(b)  => if *b { "TRUE".into() } else { "FALSE".into() },
        SqlLiteral::Str(s)   => escape_sql_str(s),
    }
}

/// シングルクォートのエスケープ（SQL インジェクション対策）
fn escape_sql_str(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}
```

### テーブル名プレースホルダー: `?pushdown_table?`

実行時に vm.rs の `execute_duckdb_pushdown` 内で
`sql.replace("?pushdown_table?", &format!("_batch_{batch_id}"))` で置換する。

### 完了条件
- `cargo check` でコンパイルエラー 0
- `pushdown_sql_filter`、`pushdown_sql_count` ユニットテスト

---

## T4: `compiler.rs` — プッシュダウン試行の統合

> **実装注意**: 実際の compiler には `compile_trf_def(&mut self, ...)` というメソッドは存在しない。
> `Item::TrfDef` の処理は `compile_fn_def` 等のフリー関数から呼ばれる。
> 実装前に `grep -n "TrfDef\|trf_def" fav/src/compiler.rs` で処理箇所を確認し、
> 実際のコード構造に合わせて挿入位置を決定する。

### 変更方針

```rust
// compiler.rs — Item::TrfDef の match アームを探して変更する

// 変更前（既存コード）:
Item::TrfDef(trf) => {
    compile_fn_def_inner(trf, ...)?;
}

// 変更後（プッシュダウン試行を追加）:
Item::TrfDef(trf) => {
    let param_name = trf.params.first().map(|p| p.name.as_str()).unwrap_or("");
    if let Some(plan) = crate::pushdown::detect_pushdown(&trf.body.result, param_name) {
        // Step 1: 通常コンパイル（fallback 用）
        compile_fn_def_inner(trf, program, str_table)?;
        let fallback_fn_idx = program.functions.len() - 1;

        // Step 2: プッシュダウンラッパー関数を追加で生成
        compile_pushdown_wrapper(trf, plan, fallback_fn_idx, program, str_table)?;
        return Ok(());
    }
    // 通常コンパイル（pushdown なし）
    compile_fn_def_inner(trf, program, str_table)?;
}
```

### ラッパー関数の生成方針

```rust
fn compile_pushdown_wrapper(
    trf: &TrfDef,
    plan: PushdownPlan,
    fallback_fn_idx: usize,
    program: &mut IRProgram,
    str_table: &mut StrTable,
) -> Result<(), CompileError> {
    // 実際の IRFnDef / Opcode 型に合わせて記述する（実装時にコードを確認）
    // ラッパーは以下の論理構造を持つ:
    //   LoadLocal(0)                        -- rows 引数
    //   Const(Str(plan.sql))               -- SQL テンプレート文字列
    //   Const(Int(fallback_fn_idx as i64)) -- fallback 関数インデックス
    //   CallBuiltin("__duckdb_push", 3)    -- 3 引数
    //   Ret
    Ok(())
}
```

### builtin 名の登録

`"__duckdb_push"` を `compiler.rs` の builtin 定数リスト（`is_known_builtin` または同等）に追加する:
```bash
grep -n "is_known_builtin\|builtin_list\|\"ArrowBatch\"\|\"DuckDb\"" fav/src/compiler.rs
```

### 完了条件
- `cargo check` でコンパイルエラー 0
- プッシュダウン対象 `.fav` を `fav run` で実行して `__duckdb_push` が呼ばれることを確認

---

## T5: `vm.rs` — `__duckdb_push` ビルトイン

> **実装注意 — 正式シグネチャ**:
> - `VMError` は struct（`{ message, fn_name, ip }`）であり enum ではない。variant 追加不可。
> - `vm_call_builtin` の戻り値エラーは `String`（`VMError` ではない）。
> - fallback 伝搬には `Err("__PUSHDOWN_FALLBACK:{fn_idx}")` という特殊文字列を使う。
> - `call_builtin`（VM メソッド）が `vm_call_builtin` の戻り値を受け取り、このパターンを検出する。

### 5-1. 事前確認: `duckdb` バージョンと `register_arrow` の存在

実装開始前に必ず確認する:
```bash
grep duckdb fav/Cargo.toml
# → duckdb = "X.Y.Z" のバージョンを確認（0.9+ なら register_arrow が利用可能）
```

### 5-2. `vm_call_builtin` に `__duckdb_push` ハンドラを追加

```rust
// vm_call_builtin 内（戻り値: Result<VMValue, String>）

"__duckdb_push" if args.len() == 3 => {
    let fallback_idx = match &args[2] {
        VMValue::Int(n) => *n as usize,
        _ => return Err("__duckdb_push: invalid fallback_idx".to_string()),
    };
    let sql_template = match &args[1] {
        VMValue::Str(s) => s.clone(),
        _ => return Err("__duckdb_push: invalid sql arg".to_string()),
    };

    // ArrowBatch 入力なら DuckDB に委譲
    if let VMValue::ArrowBatch(batch_id) = &args[0] {
        match execute_duckdb_pushdown(*batch_id, &sql_template, emit_log) {
            Ok(result) => return Ok(result),
            Err(e) => {
                emit_log.push(VMValue::Str(format!("[pushdown warn] {e}")));
                // 失敗した場合は fallback にフォールスルー
            }
        }
    }
    // ArrowBatch でない、または DuckDB 失敗 → 特殊文字列で fallback 伝搬
    Err(format!("__PUSHDOWN_FALLBACK:{fallback_idx}"))
}
```

### 5-3. `call_builtin` に fallback 文字列の処理を追加

```rust
// call_builtin（VM メソッド）内、vm_call_builtin 呼び出し後

match vm_call_builtin(name, args, &mut temp_log, ...) {
    Ok(result) => { /* ... */ }
    Err(e) if e.starts_with("__PUSHDOWN_FALLBACK:") => {
        // fallback fn_idx を parse して通常関数として実行
        if let Ok(fallback_idx) = e["__PUSHDOWN_FALLBACK:".len()..].parse::<usize>() {
            let table_arg = /* スタックから rows を再取得 */;
            self.push_compiled_frame(artifact, fallback_idx, vec![table_arg])?;
        }
    }
    Err(e) => return Err(self.error(artifact, &e)),
}
```

### 5-4. `execute_duckdb_pushdown` 関数を追加（WASM 除外）

```rust
#[cfg(not(target_arch = "wasm32"))]
fn execute_duckdb_pushdown(
    batch_id: u64,
    sql_template: &str,
    _emit_log: &mut Vec<VMValue>,
) -> Result<VMValue, String> {
    let batch = ARROW_BATCHES.with(|m| m.borrow().get(&batch_id).cloned())
        .ok_or_else(|| format!("pushdown: batch {} not found", batch_id))?;

    let table_name = format!("_batch_{}", batch_id);
    let conn = duckdb::Connection::open_in_memory()
        .map_err(|e| format!("duckdb open: {e}"))?;
    conn.register_arrow(&table_name, batch.clone())
        .map_err(|e| format!("duckdb register: {e}"))?;

    let sql = sql_template.replace("?pushdown_table?", &table_name);
    debug_assert!(!sql.contains("?pushdown_table?"), "placeholder not replaced");

    let result = conn.query_arrow(&sql)
        .map_err(|e| format!("duckdb query: {e}"))?;

    let new_id = next_arrow_batch_id();
    ARROW_BATCHES.with(|m| m.borrow_mut().insert(new_id, result));
    Ok(VMValue::ArrowBatch(new_id))
}

// WASM では常に fallback に委譲（DuckDB 不使用）
#[cfg(target_arch = "wasm32")]
fn execute_duckdb_pushdown(_: u64, _: &str, _: &mut Vec<VMValue>) -> Result<VMValue, String> {
    Err("pushdown: DuckDB not available on wasm32".to_string())
}
```

### 5-5. `is_known_builtin_namespace` に `"__duckdb_push"` を追加

```bash
grep -n "is_known_builtin_namespace\|ArrowBatch" fav/src/backend/vm.rs
# → 追加箇所を確認して "__duckdb_push" を追加
```

### 完了条件
- `cargo check` でコンパイルエラー 0
- ArrowBatch を入力とするプッシュダウンステージが SQL で実行される
- 通常リストを入力とする場合に fallback が呼ばれる
- WASM ビルドがコンパイルエラーなし（`#[cfg(not(wasm32))]` ガード確認）

---

## T6: `driver.rs` — `--explain-pushdown` + v204000_tests

### `--explain-pushdown` フラグ

```rust
// driver.rs の RunOptions に追加
pub struct RunOptions {
    // ... 既存 ...
    pub explain_pushdown: bool,
}
```

```rust
// main.rs の CLI パース箇所に追加
"--explain-pushdown" => { opts.explain_pushdown = true; }
```

実行後、`--explain-pushdown` が true なら `PUSHDOWN_LOG thread-local` の内容を
stderr に出力する。

```rust
// vm.rs: thread-local 宣言
thread_local! {
    pub static PUSHDOWN_EXPLAIN_ENABLED: std::cell::Cell<bool> = std::cell::Cell::new(false);
    pub static PUSHDOWN_LOG: std::cell::RefCell<Vec<String>> = std::cell::RefCell::new(Vec::new());
}
// __duckdb_push ハンドラ内で PUSHDOWN_EXPLAIN_ENABLED が true のときのみ PUSHDOWN_LOG に追記する。
```

### v204000_tests モジュール

```rust
#[cfg(test)]
mod v204000_tests {
    use crate::pushdown::{detect_pushdown, sql_builder::build_sql, PushdownOp};

    #[test]
    fn version_is_20_4_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("20.4.0"), "Cargo.toml should have version 20.4.0");
    }

    #[test]
    fn pushdown_detect_filter() {
        // List.filter(rows, |r| r.amount > 1000.0) の AST を手動構築
        // Expr::Apply(FieldAccess(Ident("List"), "filter"), [Ident("rows"), Closure(["r"], BinOp(Gt, FieldAccess(Ident("r"), "amount"), Lit(Float(1000.0))))], _)
        // このテストは src/pushdown/pattern.rs の内部テスト（#[cfg(test)] mod tests）に配置することで
        // 実際の Expr 型をインポートして手動構築できる
        // driver.rs からは「version_is_20_4_0」と SQL 生成テストのみを実行し、
        // detect_pushdown テストは pushdown/pattern.rs の内部テストで代替する
        let plan = crate::pushdown::detect_pushdown(
            &make_filter_expr(),  // pattern.rs の内部テストヘルパー（pub(crate)）
            "rows"
        );
        assert!(plan.is_some(), "filter should be detected as pushdown");
        assert!(matches!(plan.unwrap().op, PushdownOp::Filter(_)));
    }

    #[test]
    fn pushdown_sql_filter() {
        use crate::pushdown::{FilterExpr, CmpOp, SqlLiteral};
        let op = PushdownOp::Filter(FilterExpr::FieldCmp {
            field: "amount".into(),
            op: CmpOp::Gt,
            value: SqlLiteral::Float(1000.0),
        });
        let sql = build_sql(&op);
        assert!(sql.contains("WHERE amount > 1000"), "sql: {sql}");
        assert!(sql.contains("?pushdown_table?"), "sql: {sql}");
    }

    #[test]
    fn pushdown_sql_count() {
        let sql = build_sql(&PushdownOp::Count);
        assert_eq!(sql, "SELECT COUNT(*) FROM ?pushdown_table?");
    }

    #[test]
    fn pushdown_no_match_complex() {
        // パターン外の body: Expr::Ident("rows") — 単なる変数参照はプッシュダウン対象外
        // detect_pushdown に Expr::Ident を渡すと None が返ることを確認
        use crate::ast::{Expr, Span};
        let non_list_call = Expr::Ident("rows".to_string(), Span::dummy());
        let plan = detect_pushdown(&non_list_call, "rows");
        assert!(plan.is_none(), "bare ident should not be pushed down");
    }
}
```

### 完了条件
- `cargo test v204000` — 5/5 PASS
- `fav run --explain-pushdown` がプッシュダウン適用ステージを stderr に出力する

---

## T7: `fav/Cargo.toml` バージョン更新

`version = "20.3.0"` → `"20.4.0"` に変更。

---

## T8: `CHANGELOG.md` 更新 + `benchmarks/v20.4.0.json`

### CHANGELOG エントリ

```markdown
## [v20.4.0] — 2026-06-XX — DuckDB プッシュダウン最適化パス

### Changed
- `TrfDef` の compile 処理にプッシュダウン試行を追加（`detect_pushdown` が先行チェック）
- プッシュダウン対象ステージは `__duckdb_push` builtin 経由で DuckDB に委譲される

### Added
- `fav/src/pushdown/` モジュール（mod.rs / pattern.rs / sql_builder.rs）
- `__duckdb_push` VM builtin（ArrowBatch → DuckDB SQL 実行、非 ArrowBatch → fallback）
- `fav run --explain-pushdown` フラグ（プッシュダウン適用状況を stderr に出力）
- fallback 伝搬: `Err("__PUSHDOWN_FALLBACK:{fn_idx}")` 特殊文字列（`VMError` struct は変更不要）

### Performance
- `duckdb_query_sum_1m_ms`: +10〜100x（DuckDB ネイティブ集計委譲）
- `record_transform_1m_ms`（DuckDB 由来データの filter ステージ）: +2〜3x
```

### `benchmarks/v20.4.0.json`

プッシュダウン完成後に実測して生成する:
```bash
bash benchmarks/suite/run_all.sh --format json > benchmarks/v20.4.0.json
```

---

## 注意点

### `compile_trf_def_normal` の抽出

既存の `compile_trf_def` 本体を `compile_trf_def_normal` として抽出するが、
関数のシグネチャは変わらない。既存の呼び出しをすべて確認してから変更する。

```bash
grep -n "compile_trf_def" fav/src/compiler.rs
```

### `duckdb` クレートのバージョン確認

vm.rs で `duckdb::Connection::open_in_memory()` と `register_arrow` を使う。
既存の `duckdb` 依存バージョンに `register_arrow` が含まれるか確認する:
```bash
grep duckdb fav/Cargo.toml
```
`0.9.0` 以降であれば `register_arrow` が利用可能。

### SQL インジェクション対策の確認

`build_sql` が生成するリテラルはコンパイル時 AST 由来（ユーザー入力実行時値ではない）。
ただし `escape_sql_str` のテストを必ず追加する（シングルクォートのエスケープ確認）。
