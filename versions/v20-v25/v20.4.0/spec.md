# v20.4.0 Spec — DuckDB プッシュダウン最適化パス

## 概要

v20.4.0 は Favnir パイプラインの **DuckDB プッシュダウン最適化**を実装する。
コンパイラが AST レベルで「DuckDB に委譲できる操作」を静的解析し、
対象ステージを Favnir VM 実行から DuckDB SQL 実行に自動的に切り替える。

`fav explain --lineage` の静的解析基盤を活用し、5 つの操作パターン（Phase 1）を
コンパイル時に検出する。実行時は入力が `ArrowBatch`（DuckDB 由来）であれば SQL で、
それ以外であれば通常の Favnir VM コードにフォールバックする二重パス戦略を採用する。

**テーマ**: Runtime Excellence シリーズ第4弾 — データ処理の DuckDB への委譲

---

## 動機と期待効果

| ベンチマーク | v20.3.0 基準 | 期待改善 |
|---|---|---|
| `duckdb_query_sum_1m_ms` | ~44ms（VM 実行） | **+10〜100x**（DuckDB ネイティブ集計） |
| `record_transform_1m_ms` | ~165ms | **+2〜3x**（DuckDB フィルタ委譲） |

DuckDB の集計エンジンは VM ループより 10〜100 倍速い。
特に `GROUP BY` + `SUM` パターンはベクトル化実行が効くため効果が大きい。

---

## アーキテクチャ

### 二重パス戦略

```
                    ┌─ compile-time ──────────────────────────┐
AST (TrfDef)  ─→   │  analyze_pushdown()                      │
                    │   ↓ match pattern                        │
                    │  PushdownPlan { sql, fallback_fn_idx }  │
                    └─────────────────────────────────────────┘
                            ↓ emit
                    Opcode::CallBuiltin("__duckdb_push")
                    args: [table, sql_const, fallback_fn_idx]

                    ┌─ runtime ───────────────────────────────┐
vm_call_builtin     │  is ArrowBatch? → execute SQL (fast)    │
"__duckdb_push"     │  otherwise?     → invoke fallback fn    │
                    └─────────────────────────────────────────┘
```

### なぜ AST レベルで検出するか

IR/bytecode レベルではラムダが別関数にコンパイルされており、
「`|r| r.amount > 1000.0`」というパターンが読み取れない。
AST レベルでは `Expr::Lambda { param, body: Expr::BinOp { op: Gt, ... } }` として
構造が保たれているため、パターンマッチが可能。

---

## 検出パターン（Phase 1 — 5 種）

| パターン名 | Favnir AST | 生成 SQL |
|---|---|---|
| **Filter** | `List.filter(rows, \|r\| r.field op lit)` | `WHERE field op lit` |
| **Project** | `List.map(rows, \|r\| { field1: r.field1 })` | `SELECT field1` |
| **GroupBy** | `List.group_by(rows, \|r\| r.key)` | `GROUP BY key` |
| **SumBy** | `List.sum_by(rows, \|r\| r.val)` | `SUM(val)` |
| **Count** | `List.length(rows)` | `COUNT(*)` |

### Filter 式の詳細

```favnir
// 単純比較
List.filter(rows, |r| r.amount > 1000.0)
  → WHERE amount > 1000.0

// AND 複合
List.filter(rows, |r| r.amount > 1000.0 && r.status == "active")
  → WHERE amount > 1000.0 AND status = 'active'

// OR 複合
List.filter(rows, |r| r.category == "A" || r.category == "B")
  → WHERE category = 'A' OR category = 'B'
```

#### サポートする比較演算子

| Favnir | SQL |
|---|---|
| `>` | `>` |
| `>=` | `>=` |
| `<` | `<` |
| `<=` | `<=` |
| `==` | `=` |
| `!=` | `<>` |

#### サポートするリテラル型

- `Int` → SQL integer リテラル
- `Float` → SQL real リテラル
- `Str` → SQL `'...'`（シングルクォート、エスケープ処理あり）
- `Bool` → `TRUE` / `FALSE`

### Pipeline チェーン（Phase 1 スコープ外）

```favnir
// この形のチェーンは Phase 1 対象外（v20.5 以降）
List.filter(rows, |r| r.amount > 1000.0)
|> |filtered| List.group_by(filtered, |r| r.category)
```

Phase 1 は **単一操作**（1 ステージ = 1 SQL 変換）のみ対象。

---

## 新規モジュール: `src/pushdown/`

### ファイル構成

```
fav/src/pushdown/
├── mod.rs          — PushdownPattern, PushdownPlan, detect_pushdown エントリポイント
├── pattern.rs      — AST パターンマッチャー（5 パターン）
└── sql_builder.rs  — PushdownPlan → SQL 文字列生成
```

### 型定義

```rust
// src/pushdown/mod.rs

/// フィルタ条件の AST 表現
#[derive(Debug, Clone)]
pub enum FilterExpr {
    /// r.field op literal
    FieldCmp { field: String, op: CmpOp, value: SqlLiteral },
    And(Box<FilterExpr>, Box<FilterExpr>),
    Or(Box<FilterExpr>, Box<FilterExpr>),
}

#[derive(Debug, Clone)]
pub enum CmpOp { Gt, Ge, Lt, Le, Eq, Ne }

#[derive(Debug, Clone)]
pub enum SqlLiteral { Int(i64), Float(f64), Str(String), Bool(bool) }

/// プッシュダウン可能な操作（Phase 1: 5 種）
#[derive(Debug, Clone)]
pub enum PushdownOp {
    Filter(FilterExpr),
    Project(Vec<String>),     // SELECT field1, field2, ...
    GroupBy(String),           // GROUP BY key
    SumBy(String),             // SUM(field)
    Count,                     // COUNT(*)
}

/// 検出されたプッシュダウン計画
#[derive(Debug, Clone)]
pub struct PushdownPlan {
    /// 生成された SQL（`?pushdown_table?` は実行時にテーブル名で置換）
    pub sql: String,
    /// 操作の種類（ログ/--explain-pushdown 用）
    pub op: PushdownOp,
}
```

### エントリポイント

```rust
// src/pushdown/mod.rs

/// stage 関数の AST body（Block の末尾式）を解析し、プッシュダウン可能なら PushdownPlan を返す。
/// param_name: stage 関数の先頭パラメータ名（trf.params.first().map(|p| &p.name) で取得）
/// 呼び出し側: detect_pushdown(&trf.body.result, trf.params.first().map(|p| p.name.as_str()).unwrap_or(""))
pub fn detect_pushdown(body: &Expr, param_name: &str) -> Option<PushdownPlan>;
```

---

## pattern.rs — AST パターンマッチャー

> **実装注意 — Favnir AST の正式型名**:
> - `Expr` は `ExprKind` ネスト型を持たないフラット enum
> - Lambda は `Expr::Closure(Vec<String>, Box<Expr>, Span)` — `param` は `Vec<String>` の先頭要素
> - 二項演算子は `BinOp`（`BinOpKind` は存在しない）: `BinOp::And / Or / Gt / Ge / Lt / Le / Eq / Ne`
> - Literal は `Lit`（`Literal` ではない）
> - 関数適用は `Expr::Apply(Box<Expr>, Vec<Expr>, Span)`
> - フィールドアクセスは `Expr::FieldAccess(Box<Expr>, String, Span)`
> - 変数参照は `Expr::Ident(String, Span)`
>
> `TrfDef.body` は `Block { stmts: Vec<Stmt>, result: Expr, span: Span }` 型。
> `detect_pushdown` には `&trf.body.result` を渡す（末尾式のみ対象）。

```rust
// src/pushdown/pattern.rs

use crate::ast::{Expr, BinOp, Lit};
use super::{FilterExpr, CmpOp, SqlLiteral};

/// Expr が `List.filter(param, |r| ...)` にマッチするか解析
pub fn analyze_filter(expr: &Expr, param: &str) -> Option<FilterExpr>;

/// Expr が `List.map(param, |r| { f1: r.f1, ... })` にマッチするか解析
pub fn analyze_project(expr: &Expr, param: &str) -> Option<Vec<String>>;

/// Expr が `List.group_by(param, |r| r.key)` にマッチするか解析
pub fn analyze_group_by(expr: &Expr, param: &str) -> Option<String>;

/// Expr が `List.sum_by(param, |r| r.val)` にマッチするか解析
pub fn analyze_sum_by(expr: &Expr, param: &str) -> Option<String>;

/// Expr が `List.length(param)` にマッチするか解析
pub fn analyze_count(expr: &Expr, param: &str) -> bool;

/// フィルタ lambda body（`r.field op lit`）を FilterExpr に変換
fn analyze_filter_expr(body: &Expr, lambda_param: &str) -> Option<FilterExpr>;

/// `Lit` を `SqlLiteral` に変換
fn literal_to_sql(lit: &Lit) -> Option<SqlLiteral>;

/// `BinOp` を CmpOp に変換（And/Or は None）
fn binop_to_cmp(op: &BinOp) -> Option<CmpOp>;
```

**マッチ条件（`List.filter` の例 — 実際の Favnir AST 型）:**

```
// List.filter(rows, |r| r.amount > 1000.0) の AST
Expr::Apply(
    Box<Expr::FieldAccess(Box<Expr::Ident("List", _)>, "filter", _)>,
    vec![
        Expr::Ident(param_name, _),        // 入力パラメータと一致
        Expr::Closure(
            vec![lambda_param],            // params: Vec<String>
            Box<filter_body_expr>,
            _span
        )
    ],
    _span
)
```

---

## sql_builder.rs — SQL 生成

```rust
// src/pushdown/sql_builder.rs

use super::{PushdownOp, FilterExpr, CmpOp, SqlLiteral};

/// PushdownOp から SQL 文字列を生成する。
/// テーブル名プレースホルダー `?pushdown_table?` は実行時に `_batch_{id}` で置換する。
pub fn build_sql(op: &PushdownOp) -> String;

fn build_filter_where(expr: &FilterExpr) -> String;
fn build_cmp_op(op: &CmpOp) -> &'static str;
fn build_literal(lit: &SqlLiteral) -> String;
fn escape_sql_str(s: &str) -> String;  // ' → '' のエスケープ
```

**生成例:**

```rust
// Filter
build_sql(&PushdownOp::Filter(FilterExpr::FieldCmp {
    field: "amount".into(), op: CmpOp::Gt, value: SqlLiteral::Float(1000.0)
}))
// → "SELECT * FROM ?pushdown_table? WHERE amount > 1000.0"

// Count
build_sql(&PushdownOp::Count)
// → "SELECT COUNT(*) FROM ?pushdown_table?"

// SumBy
build_sql(&PushdownOp::SumBy("amount".into()))
// → "SELECT SUM(amount) FROM ?pushdown_table?"

// Project
build_sql(&PushdownOp::Project(vec!["category".into(), "amount".into()]))
// → "SELECT category, amount FROM ?pushdown_table?"

// GroupBy（Phase 1: DISTINCT 相当 — COUNT は含まない）
build_sql(&PushdownOp::GroupBy("category".into()))
// → "SELECT DISTINCT category FROM ?pushdown_table?"

// GroupBy + SumBy の複合は Phase 1 対象外（単一操作のみ）
```

---

## compiler.rs — プッシュダウン統合

> **実装注意**: 実際の compiler は `Item::TrfDef(td)` の `match` アームで処理する構成。
> `compile_trf_def` という self メソッドは存在しない。
> `trf.params.first()` でパラメータ名を取得し、`trf.body.result` を detect_pushdown に渡す。
> また、`"__duckdb_push"` を compiler.rs の builtin 名リストと `is_known_builtin_namespace`
> に追加する必要がある（変更ファイル一覧参照）。

### `Item::TrfDef` の処理箇所への追加

```rust
// compiler.rs（Item::TrfDef の match アーム内）

Item::TrfDef(trf) => {
    // --- 新規: プッシュダウン試行 ---
    let param_name = trf.params.first().map(|p| p.name.as_str()).unwrap_or("");
    if let Some(plan) = crate::pushdown::detect_pushdown(&trf.body.result, param_name) {
        // fallback 用に通常コンパイルを先に実行
        let fallback_fn_idx = compile_trf_def_inner(trf, program, str_table)?;
        // プッシュダウンラッパーを追加で生成
        compile_pushdown_wrapper(trf, plan, fallback_fn_idx, program, str_table)?;
        return Ok(());
    }
    // --- 既存: 通常コンパイル ---
    compile_trf_def_inner(trf, program, str_table)
}
```

### `compile_pushdown_wrapper` の実装方針

```rust
// プッシュダウンラッパー関数を生成する（フリー関数）
fn compile_pushdown_wrapper(
    trf: &TrfDef,
    plan: PushdownPlan,
    fallback_fn_idx: usize,
    program: &mut IRProgram,
    str_table: &mut StrTable,
) -> Result<(), CompileError> {
    // SQL 文字列を str_table に追加
    let sql_idx = str_table.intern(&plan.sql);

    // ラッパー関数の IR を生成:
    //   LoadLocal(0)              -- rows 引数
    //   Const(Str(sql_idx))      -- SQL 文字列
    //   Const(Int(fallback_idx)) -- fallback 関数インデックス
    //   CallBuiltin("__duckdb_push", 3)
    //   Ret
    // ...（実際の IRFnDef 構築に合わせる）
    Ok(())
}
```

---

## vm.rs — `__duckdb_push` ビルトイン

> **実装注意 — VMError / vm_call_builtin の正式型**:
> - `VMError` は struct（`{ message: String, fn_name: String, ip: usize }`）であり enum ではない。
>   variant を追加できないため、fallback 伝搬には **特殊エラー文字列** を使用する。
> - `vm_call_builtin` のシグネチャ: `fn vm_call_builtin(name: &str, args: Vec<VMValue>, emit_log: &mut Vec<VMValue>, ...) -> Result<VMValue, String>`
>   戻り値エラーは `String`（`VMError` ではない）。
> - `call_builtin` （VM メソッド）は `vm_call_builtin` を呼び出す wrapper。

### `vm_call_builtin` への追加

```rust
// vm.rs / vm_call_builtin（戻り値: Result<VMValue, String>）

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
                // DuckDB 失敗 → fallback にフォールスルー（警告のみ）
                emit_log.push(VMValue::Str(format!("[pushdown warn] {e}")));
            }
        }
    }
    // ArrowBatch でない、または DuckDB 失敗 → 特殊文字列で fallback 伝搬
    Err(format!("__PUSHDOWN_FALLBACK:{fallback_idx}"))
}
```

> **fallback 伝搬**: `vm_call_builtin` は `String` エラーを返す。
> `"__PUSHDOWN_FALLBACK:{fn_idx}"` という特殊プレフィックスを使い、
> `call_builtin` メソッド内でこのパターンを検出して `push_compiled_frame(fallback_idx, ...)` にリトライする。
> `VMError` struct には手を加えない。

### `execute_duckdb_pushdown` の実装

```rust
// #[cfg(not(target_arch = "wasm32"))] で WASM ビルドから除外する
#[cfg(not(target_arch = "wasm32"))]
fn execute_duckdb_pushdown(
    batch_id: u64,
    sql_template: &str,
    _emit_log: &mut Vec<VMValue>,
) -> Result<VMValue, String> {
    // ARROW_BATCHES thread-local から RecordBatch を取得
    let batch = ARROW_BATCHES.with(|m| m.borrow().get(&batch_id).cloned())
        .ok_or_else(|| format!("pushdown: batch {} not found", batch_id))?;

    // DuckDB インメモリ DB を作成
    let conn = duckdb::Connection::open_in_memory()
        .map_err(|e| format!("duckdb open: {e}"))?;

    // RecordBatch を DuckDB に登録（API: duckdb 0.9+ が必要）
    // 実装前に register_arrow / query_arrow の存在を確認すること
    let table_name = format!("_batch_{}", batch_id);
    conn.register_arrow(&table_name, batch.clone())
        .map_err(|e| format!("duckdb register: {e}"))?;

    // SQL プレースホルダーを実テーブル名で置換
    let sql = sql_template.replace("?pushdown_table?", &table_name);
    debug_assert!(!sql.contains("?pushdown_table?"), "placeholder not replaced");

    // クエリ実行
    let result = conn.query_arrow(&sql)
        .map_err(|e| format!("duckdb query '{sql}': {e}"))?;

    // 結果を新しい ArrowBatch として格納
    let new_id = next_arrow_batch_id();
    ARROW_BATCHES.with(|m| m.borrow_mut().insert(new_id, result));
    Ok(VMValue::ArrowBatch(new_id))
}
```

---

## `--explain-pushdown` フラグ

```bash
fav run pipeline.fav --explain-pushdown
```

実行時、プッシュダウンが適用されたステージの情報を標準エラーに出力:

```
[pushdown] stage Filter → SQL: SELECT * FROM ? WHERE amount > 1000.0
[pushdown] stage Aggregate → fallback (pattern not matched)
```

実装: `driver.rs` の `RunOptions` に `explain_pushdown: bool` を追加。
`__duckdb_push` builtin 内で `PUSHDOWN_LOG thread-local` に書き込み、
実行後にフラッシュする。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/pushdown/mod.rs` | **新規作成** — PushdownPattern / PushdownPlan / detect_pushdown |
| `fav/src/pushdown/pattern.rs` | **新規作成** — AST パターンマッチャー（5 パターン） |
| `fav/src/pushdown/sql_builder.rs` | **新規作成** — SQL 文字列生成 |
| `fav/src/compiler.rs` | `Item::TrfDef` match アームにプッシュダウン試行を追加 + builtin 名リストに `"__duckdb_push"` 追加 |
| `fav/src/backend/vm.rs` | `__duckdb_push` builtin（`vm_call_builtin`） + `execute_duckdb_pushdown`（`#[cfg(not(wasm32))]`） + `call_builtin` の fallback 文字列処理 |
| `fav/src/driver.rs` | `--explain-pushdown` フラグ + v204000_tests |
| `fav/src/lib.rs` | `mod pushdown;` 追加 |
| `fav/Cargo.toml` | version `20.3.0` → `20.4.0` |

---

## テスト（v204000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_20_4_0` | Cargo.toml に `"20.4.0"` が含まれる |
| `pushdown_detect_filter` | `List.filter(rows, \|r\| r.amount > 1000.0)` の AST を手動構築して Filter パターンを検出（`pattern.rs` 内部テスト） |
| `pushdown_sql_filter` | Filter パターンの SQL 生成に `WHERE amount > 1000.0` と `?pushdown_table?` が含まれる |
| `pushdown_sql_count` | Count パターンが `SELECT COUNT(*) FROM ?pushdown_table?` を生成 |
| `pushdown_no_match_complex` | パターン外の body（Ident でない第1引数）が `None` を返す |

---

## 完了条件

- [ ] `detect_pushdown` が 5 パターンを正しく検出する
- [ ] 各パターンから正しい SQL が生成される
- [ ] `compile_trf_def` がプッシュダウン対象ステージを自動変換する
- [ ] `__duckdb_push` builtin が ArrowBatch 入力で SQL を実行する
- [ ] ArrowBatch 以外の入力で fallback 関数が呼ばれる
- [ ] `--explain-pushdown` フラグで適用状況を出力する
- [ ] `cargo test v204000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
- [ ] `benchmarks/v20.4.0.json` が生成されている
- [ ] `duckdb_query_sum_1m_ms` が v20.3.0 比 +10x 以上改善（DuckDB 委譲成功時）

---

## 技術ノート

### SQL インジェクション対策

`build_sql` が生成する SQL に含まれるリテラル値は、
Favnir のコンパイル時 AST から取得した値のみ（ユーザー入力実行時値ではない）。
ただし `Str` リテラルに含まれるシングルクォートは `escape_sql_str` でエスケープする:
```rust
fn escape_sql_str(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}
```

### `register_arrow` の事前確認

`execute_duckdb_pushdown` の実装開始前に、既存の `duckdb` クレートバージョンで
`Connection::register_arrow` と `Connection::query_arrow` が利用可能かを確認する:
```bash
grep duckdb fav/Cargo.toml   # 0.9+ であれば register_arrow が利用可能
```
利用不可の場合は `CREATE TABLE ... AS SELECT ...` 経由でバッチをシリアライズする代替策を検討する。

### GroupBy 単体の SQL セマンティクス

Phase 1 の `PushdownOp::GroupBy` は **DISTINCT 相当**として実装する（COUNT は含まない）:
```sql
SELECT DISTINCT key FROM ?pushdown_table?
```
`GroupBy + SumBy` の組み合わせ（`GROUP BY key ... SUM(val)`）は Phase 2（v20.5 以降）。

### WASM ビルドへの影響

`execute_duckdb_pushdown` は `#[cfg(not(target_arch = "wasm32"))]` でガードする。
WASM ビルドでは `__duckdb_push` builtin が常に fallback 文字列を返すため、
pushdown 機能は無効化されるが（通常の Favnir VM 実行にフォールバック）、
コンパイルは通る。`vm.rs` の `is_known_builtin_namespace` への `"__duckdb_push"` 追加は
WASM でも必要（parse/compile は共通）。

### スコープ外（v20.5 以降）

- Pipeline チェーン（Filter → GroupBy 等の結合）
- `JOIN` パターン（複数テーブル）
- 集計 + フィルタの組み合わせ（`HAVING` 句）
- 実行時プロファイルに基づく適応的プッシュダウン
