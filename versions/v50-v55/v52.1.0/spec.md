# Spec: v52.1.0 — `assert_schema` Phase 1（型チェック）

Date: 2026-07-20
Status: 設計中

---

## 目的

`assert_schema<T>(value)` を VM primitive として追加し、実行時に `Map<String, Any>` の
フィールド名・型を型 `T` のスキーマと照合する。不一致時は `Err` を返す。
これにより「型なし外部データ → 型安全な Favnir 型」の境界を実行時に保証できる。

---

## 使用例

```favnir
type OrderRow = { id: Int, amount: Float, status: String }

stage ValidateSchema: Map<String, Any> -> Result<OrderRow> = |row| {
  bind validated <- assert_schema<OrderRow>(row)
  Ok(validated)
}
```

---

## 実装対象

### 1. `ast.rs` — `Expr::AssertSchema` ノード追加

```rust
AssertSchema {
    ty_name: String,   // "OrderRow"
    arg: Box<Expr>,    // 検証対象の式
    span: Span,
},
```

### 2. `middle/ir.rs` — `IRExpr::AssertSchema` ノード追加

```rust
AssertSchema {
    ty_name: String,
    arg: Box<IRExpr>,
    ty: Type,
},
```

### 3. `middle/compiler.rs` — `AssertSchema` のコンパイル

`Expr::AssertSchema` を `IRExpr::AssertSchema` に変換する。

### 4. `backend/vm.rs` — `AssertSchema` の実行時評価

`IRExpr::AssertSchema` の VM ハンドラを追加:
- `arg` を評価して `VMValue::Map` を取得
- `ty_name` に対応するスキーマ（フィールド名 → 期待型）を参照
- フィールド型が不一致なら `VMValue::Err(E0419 メッセージ)` を返す
- 一致すれば `VMValue::Ok(record)` を返す

### 5. `error_catalog.rs` — E0419 定義

既存の「予約（将来拡張用）」コメントを実際のエラーエントリに置き換える:

```rust
ErrorEntry {
    code: "E0419",
    title: "assert_schema type mismatch",
    category: "runtime",
    description: "assert_schema<T> found a field whose runtime type does not match the schema T.",
    example: "// expected { id: Int } but got { id: \"hello\" }",
    fix: "Ensure the input map contains fields matching the schema T.",
    suggestion: Some("Check the upstream data source for type mismatches."),
},
```

### 6. `driver.rs` — `v52100_tests` 追加（2 件）

---

## テスト仕様

### `assert_schema_type_ok`

正しいフィールド型を持つマップに対して `assert_schema` が `Ok` を返すことを検証する。

```rust
// ast.rs に AssertSchema が存在することを確認
let src = include_str!("ast.rs");
assert!(src.contains("AssertSchema"), "ast.rs must define AssertSchema node");
// vm.rs に AssertSchema ハンドラが存在することを確認
let vm = include_str!("backend/vm.rs");
assert!(vm.contains("AssertSchema"), "vm.rs must handle AssertSchema");
```

### `assert_schema_type_fail`

E0419 が `error_catalog.rs` に実装されていることを検証する。

```rust
let src = include_str!("error_catalog.rs");
assert!(src.contains("E0419"), "error_catalog.rs must define E0419");
assert!(src.contains("assert_schema type mismatch"),
    "E0419 must have title 'assert_schema type mismatch'");
```

`include_str!` のパス（`fav/src/driver.rs` 起点）:
- `include_str!("ast.rs")` → `fav/src/ast.rs`
- `include_str!("backend/vm.rs")` → `fav/src/backend/vm.rs`
- `include_str!("error_catalog.rs")` → `fav/src/error_catalog.rs`

---

## テスト数

- ベース: 3135（v52.0.0 完了時点）
- `cargo_toml_version_is_52_0_0` 削除: -1
- 新規追加: +2（`assert_schema_type_ok` + `assert_schema_type_fail`）
- **完了後合計: 3136 tests passed, 0 failed**（実装後 `cargo test` 実測値で確定）

---

## 完了条件

- `ast.rs` に `Expr::AssertSchema` ノードが追加されている
- `middle/ir.rs` に `IRExpr::AssertSchema` ノードが追加されている
- `middle/compiler.rs` で `AssertSchema` が IR に変換される
- `backend/vm.rs` で `AssertSchema` の実行時評価が実装されている
- `error_catalog.rs` の E0419 が「予約」から実際のエントリに置き換えられている
- `fav/Cargo.toml` version → `"52.1.0"`
- `cargo test` 3136 passed, 0 failed（実測値で確定）
- `cargo clippy -- -D warnings` クリーン
- `CHANGELOG.md` に v52.1.0 エントリ追加
- `versions/current.md` を v52.1.0（実測テスト数）に更新
- `roadmap-v52.1-v53.0.md` の v52.1.0 実績欄を更新
- `site/` MDX ドキュメントは v52.8.0 で対応（このバージョンの対象外）

## 注意事項

- `frontend/parser.rs` の `assert_schema<T>(...)` 構文パース実装は Phase 2 以降（v52.2.0〜）の対象。本バージョンのテスト 2 件は `include_str!` による静的ファイル検査のみのため、パーサー対応なしでも pass する。

- `wasm_dce.rs` の `collect_expr_fns` は `IRExpr` を網羅 match している。`IRExpr::AssertSchema` 追加時は `collect_expr_fns` に `AssertSchema` アームを追加すること（exhaustive match エラー防止）

- `Expr::AssertSchema` 追加時に exhaustive match エラーが発生しうるファイル（漏れなく対応すること）:
  - `fav/src/fmt.rs`
  - `fav/src/lint.rs`
  - `fav/src/emit_python.rs`
  - `fav/src/middle/checker.rs`（`collect_helpers_in_expr` / `check_expr` 等）
  - `fav/src/middle/compiler.rs`（`collect_free_vars_expr` 等）
  - `fav/src/lineage.rs`（`Expr` を match する複数の関数）
  - `fav/src/lsp/references.rs`（`collect_in_expr` 等）
  - `cargo build` で exhaustive match エラーが出たファイルを都度対応する方針でも可
