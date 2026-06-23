# v20.4.0 — DuckDB プッシュダウン最適化パス タスク

## ステータス: COMPLETE（T1〜T8 完了）

---

## タスク一覧

### T1: `src/pushdown/mod.rs` — PushdownPlan 型定義 + detect_pushdown エントリ

- [x] `fav/src/pushdown/mod.rs` を新規作成
- [x] `fav/src/pushdown/pattern.rs`（空スタブ）を新規作成
- [x] `fav/src/pushdown/sql_builder.rs`（空スタブ）を新規作成
- [x] `FilterExpr` enum（`FieldCmp / And / Or`）を定義
- [x] `CmpOp` enum（`Gt / Ge / Lt / Le / Eq / Ne`）を定義
- [x] `SqlLiteral` enum（`Int / Float / Str / Bool`）を定義
- [x] `PushdownOp` enum（`Filter / Project / GroupBy / SumBy / Count`）を定義
- [x] `PushdownPlan` struct（`sql: String, op: PushdownOp`）を定義
- [x] `detect_pushdown(body: &Expr, param_name: &str) -> Option<PushdownPlan>` を実装（T2/T3 スタブ呼び出し）
- [x] `fav/src/lib.rs` に `mod pushdown;` を追加（`#[cfg(not(target_arch = "wasm32"))]` 不要か確認）
- [x] `cargo check` でコンパイルエラー 0

---

### T2: `src/pushdown/pattern.rs` — AST パターンマッチャー（5 パターン）

- [x] `analyze_filter(expr, param) -> Option<FilterExpr>` を実装
  - [x] `Expr::Call { func: List.filter, args: [param, Lambda] }` のマッチ
  - [x] lambda body を `analyze_filter_expr` で解析
  - [x] `And / Or / FieldCmp` の再帰的解析
- [x] `analyze_filter_expr(body, lp) -> Option<FilterExpr>` を実装
  - [x] `BinOp::And` → `FilterExpr::And`
  - [x] `BinOp::Or` → `FilterExpr::Or`
  - [x] 比較演算子（Gt/Ge/Lt/Le/Eq/Ne）→ `FilterExpr::FieldCmp`
- [x] `analyze_project(expr, param) -> Option<Vec<String>>` を実装
  - [x] `List.map(param, |r| { f1: r.f1, ... })` のマッチ
  - [x] record リテラルからフィールド名リストを抽出
- [x] `analyze_group_by(expr, param) -> Option<String>` を実装
  - [x] `List.group_by(param, |r| r.key)` のマッチ
- [x] `analyze_sum_by(expr, param) -> Option<String>` を実装
  - [x] `List.sum_by(param, |r| r.val)` のマッチ
- [x] `analyze_count(expr, param) -> bool` を実装
  - [x] `List.length(param)` のマッチ
- [x] ヘルパー `extract_list_call / extract_field_access / extract_literal / binop_to_cmp / is_param_first / extract_projection_fields` を実装
- [x] `cargo check` でコンパイルエラー 0

---

### T3: `src/pushdown/sql_builder.rs` — SQL 文字列生成

- [x] `build_sql(op: &PushdownOp) -> String` を実装
  - [x] `Count` → `"SELECT COUNT(*) FROM ?pushdown_table?"`
  - [x] `SumBy(field)` → `"SELECT SUM(field) FROM ?pushdown_table?"`
  - [x] `GroupBy(key)` → `"SELECT key, COUNT(*) FROM ?pushdown_table? GROUP BY key"`
  - [x] `Project(fields)` → `"SELECT f1, f2 FROM ?pushdown_table?"`
  - [x] `Filter(expr)` → `"SELECT * FROM ?pushdown_table? WHERE ..."`
- [x] `build_filter_where(expr: &FilterExpr) -> String` を実装
  - [x] `FieldCmp` → `"field op literal"`
  - [x] `And` → `"(left) AND (right)"`
  - [x] `Or` → `"(left) OR (right)"`
- [x] `build_cmp_op(op: &CmpOp) -> &'static str` を実装（6 演算子）
- [x] `build_literal(lit: &SqlLiteral) -> String` を実装
- [x] `escape_sql_str(s: &str) -> String` を実装（`'` → `''`）
- [x] `cargo check` でコンパイルエラー 0

---

### T4: `compiler.rs` — プッシュダウン試行の統合

- [x] **事前確認**: `grep -n "TrfDef\|trf_def" fav/src/compiler.rs` で `Item::TrfDef` の処理箇所を特定
- [x] `Item::TrfDef` match アームに `detect_pushdown` 呼び出しを追加:
  - [x] `trf.params.first().map(|p| p.name.as_str())` でパラメータ名を取得
  - [x] `detect_pushdown(&trf.body.result, param_name)` を呼ぶ（`trf.body` が `Block` 型であることを確認）
- [x] `compile_pushdown_wrapper(trf, plan, fallback_fn_idx, program, str_table)` を実装:
  - [x] 実際の `IRFnDef` / `Opcode` 型に合わせてラッパー関数 IR を生成:
    - [x] `LoadLocal(0)` — rows 引数
    - [x] `Const(Str(plan.sql))` — SQL テンプレート文字列
    - [x] `Const(Int(fallback_fn_idx as i64))` — fallback 関数インデックス
    - [x] `CallBuiltin("__duckdb_push", 3)` — 3 引数
    - [x] `Ret`
- [x] compiler.rs の builtin 定数リストに `"__duckdb_push"` を追加:
  - [x] `grep -n "\"ArrowBatch\"\|builtin.*list\|known_builtin" fav/src/compiler.rs` で追加箇所を確認
- [x] `cargo check` でコンパイルエラー 0

---

### T5: `vm.rs` — `__duckdb_push` ビルトイン + fallback 伝搬

- [x] **事前確認**: `grep duckdb fav/Cargo.toml` で `register_arrow` / `query_arrow` が使用可能なバージョン（0.9+）を確認
- [x] `vm_call_builtin` に `"__duckdb_push"` ハンドラを追加（戻り値 `Result<VMValue, String>`）:
  - [x] args[0] が `VMValue::ArrowBatch(id)` なら `execute_duckdb_pushdown` を呼ぶ
  - [x] DuckDB 失敗または非 ArrowBatch の場合は `Err("__PUSHDOWN_FALLBACK:{fallback_idx}")` を返す（`VMError` は変更しない）
- [x] `execute_duckdb_pushdown(batch_id, sql_template, emit_log) -> Result<VMValue, String>` を実装:
  - [x] `#[cfg(not(target_arch = "wasm32"))]` でガード（WASM 版は常に `Err(...)` を返す）
  - [x] `ARROW_BATCHES` から RecordBatch を取得
  - [x] DuckDB インメモリ接続を作成
  - [x] `register_arrow(table_name, batch)` でテーブル登録
  - [x] SQL の `?pushdown_table?` を `_batch_{id}` で置換
  - [x] `debug_assert!(!sql.contains("?pushdown_table?"))` で置換確認
  - [x] `query_arrow(sql)` で実行
  - [x] 結果を新しい ArrowBatch として ARROW_BATCHES に格納して `VMValue::ArrowBatch(new_id)` を返す
- [x] `call_builtin`（VM メソッド）に `"__PUSHDOWN_FALLBACK:"` プレフィックス処理を追加:
  - [x] `push_compiled_frame(artifact, fallback_idx, ...)` にリトライ
- [x] `is_known_builtin_namespace` に `"__duckdb_push"` を追加
- [x] `PUSHDOWN_EXPLAIN_ENABLED` / `PUSHDOWN_LOG` thread-local を追加（`--explain-pushdown` 用）
- [x] `cargo check` でコンパイルエラー 0
- [x] WASM ビルドが `#[cfg(not(wasm32))]` ガードで通過することを確認

---

### T6: `driver.rs` — `--explain-pushdown` + v204000_tests

- [x] `RunOptions` に `explain_pushdown: bool` フィールドを追加
- [x] `main.rs` の CLI パース箇所に `"--explain-pushdown"` を追加
- [x] `cmd_run` 実行後に `explain_pushdown` フラグが true なら `PUSHDOWN_LOG` thread-local の内容を stderr に出力
- [x] `v204000_tests` モジュールを追加:
  - [x] `version_is_20_4_0`
  - [x] `pushdown_detect_filter`（`pattern.rs` 内の `pub(crate) fn make_filter_expr()` を使って Filter 検出を確認）
  - [x] `pushdown_sql_filter`（`FilterExpr::FieldCmp` を直接構築して SQL 生成確認）
  - [x] `pushdown_sql_count`（Count パターンが `SELECT COUNT(*) FROM ?pushdown_table?` を生成）
  - [x] `pushdown_no_match_complex`（`Expr::Ident` を渡して `None` を返すことを確認）
- [x] `cargo test v204000` — 5/5 PASS を確認

---

### T7: `fav/Cargo.toml` バージョン更新

- [x] `version = "20.3.0"` → `"20.4.0"` に変更

---

### T8: `CHANGELOG.md` 更新 + ドキュメント + ベンチマーク

- [x] v20.4.0 エントリを追加（Changed + Added + Performance セクション）
- [x] `benchmarks/v20.4.0.json` を事後計測で生成・保存:
  ```bash
  bash benchmarks/suite/run_all.sh --format json > benchmarks/v20.4.0.json
  ```
- [x] `site/content/docs/cli/run.mdx` に `--explain-pushdown` フラグの説明を追加

---

## テスト（v204000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_20_4_0` | Cargo.toml に `"20.4.0"` が含まれる |
| `pushdown_detect_filter` | `List.filter(rows, \|r\| r.amount > 1000.0)` が Filter パターンを検出 |
| `pushdown_sql_filter` | Filter パターンの SQL 生成が正しい WHERE 句を含む |
| `pushdown_sql_count` | Count パターンが `SELECT COUNT(*) FROM ?pushdown_table?` を生成 |
| `pushdown_no_match_complex` | チェーン操作が `None` を返す（Phase 1 対象外） |

---

## 完了条件チェックリスト

- [x] `detect_pushdown` が 5 パターンを正しく検出する
- [x] 各パターンから正しい SQL が生成される（ユニットテスト）
- [x] `Item::TrfDef` の match アームがプッシュダウン対象ステージを自動変換する
- [x] `__duckdb_push` builtin が ArrowBatch 入力で SQL を実行する
- [x] ArrowBatch 以外の入力で fallback 関数が呼ばれる
- [x] `--explain-pushdown` フラグで適用状況を stderr に出力する
- [x] `cargo test v204000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし（全既存テストが PASS）
- [x] `fav/Cargo.toml` version が `20.4.0`
- [x] `CHANGELOG.md` に v20.4.0 エントリが追加されている
- [x] `benchmarks/v20.4.0.json` が生成されている
- [x] `duckdb_query_sum_1m_ms` が v20.3.0 比 +10x 以上改善（DuckDB 委譲成功時）

---

## 優先度

```
T1（型定義）       ← 他すべての前提
T2（パターン検出） ← T1 完了後
T3（SQL 生成）     ← T1 完了後（T2 と並列可）
T4（compiler 統合）← T2/T3 完了後
T5（VM 実行）      ← T4 完了後（最大工数）
T6（driver）       ← T5 完了後
T7（Cargo.toml）   ← 任意
T8（CHANGELOG）    ← T6 完了後
```

---

## 実装リスク と 対策

| リスク | 対策 |
|---|---|
| Favnir AST の `Expr` 型が想定と異なる | `pattern.rs` 実装前に `grep -n "^pub enum Expr\|Closure\|Apply\|FieldAccess" fav/src/ast.rs` で確認 |
| `Item::TrfDef` の処理箇所が特定できない | `grep -n "TrfDef\|trf_def" fav/src/compiler.rs` で確認 |
| `TrfDef.body` が `Block` でなく `Expr` の場合 | `grep -n "struct TrfDef\|body:" fav/src/ast.rs` で実際の型を確認 |
| DuckDB `register_arrow` が古いバージョンで未対応（事前確認必須） | T5 開始前に `grep duckdb fav/Cargo.toml` でバージョン確認（0.9+ 必要）|
| fallback 文字列 `"__PUSHDOWN_FALLBACK:"` が通常エラーと衝突する | プレフィックスの文字列長を十分長くとる（現状で十分）|
| SQL プレースホルダー置換の漏れ | `debug_assert!(!sql.contains("?pushdown_table?"))` で確認 |
| Phase 1 範囲外パターンで誤検出 | `pushdown_no_match_complex` テストで確認 |
| WASM ビルドで `execute_duckdb_pushdown` がリンクエラー | `#[cfg(not(target_arch = "wasm32"))]` ガードを確認 |
