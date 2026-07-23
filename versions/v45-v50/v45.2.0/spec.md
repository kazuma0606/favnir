# Spec: v45.2.0 — `return` 型チェック + E0415

Date: 2026-07-15
Sprint: Language Refinement (v45.1〜v46.0)

---

## 概要

v45.1.0 で AST + parser を追加した `return` 構文に、型チェックを実装する。
`checker.rs` で `ReturnStmt` の式型と宣言戻り型を照合し、不一致時に **E0415** を発行する。
あわせて `seq` ボディ内での `return` 使用を E0415 として検出・拒否する。

## 動機

```favnir
fn bad() -> Int {
  return "hello"  // E0415: return type mismatch — expected Int, got String
}

fn ok() -> Int {
  if some_condition { return 0 }
  42
}

stage ValidateOrder: Order -> Result<Order> = |order| {
  if order.amount <= 0.0 { return Err("invalid amount") }  // OK: Result<Order>
  Ok(order)
}
```

v45.1.0 の時点では `return` は checker.rs で stub（スキップ）されており、型の整合性が保証されない。

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/middle/checker.rs` | `Stmt::Return` stub → 型チェック実装（`check_return_stmt` ヘルパー追加）、`Checker` 構造体に `current_return_ty: Option<Type>` フィールド追加 |
| `fav/src/error_catalog.rs` | E0415 `ReturnTypeMismatch` エラーコード追加（E0414〜E0419 予約範囲コメントを更新して E0415 を正式登録） |
| `fav/src/driver.rs` | `v452000_tests` テストモジュール追加（3件）|
| `fav/Cargo.toml` | version `45.1.0` → `45.2.0` |
| `CHANGELOG.md` | v45.2.0 エントリ追加 |

## 設計詳細

### checker.rs — `check_return_stmt`

1. **戻り型の取得**: `checker` は現在のスコープ（`fn` / `stage`）の宣言戻り型を
   コンテキストとして保持する。`fn foo() -> T` の `T`、
   `stage S: A -> B` の `B` を参照する。

2. **seq ボディ内での return 禁止**:
   - `seq` コンテキストでは宣言戻り型が存在しない
   - `Stmt::Return` を検出した場合 E0415 (`return is not allowed in seq body`) を発行

3. **型照合**:
   - `ReturnStmt.expr` の型を推論（`infer_expr`）
   - 宣言戻り型と `unify` → 不一致なら E0415 を発行

### error_catalog.rs — E0415

`error_catalog.rs` には `E0414〜E0419: 予約（将来拡張用）` コメントが存在する。
本バージョンで E0415 を `ReturnTypeMismatch` として正式登録し、予約コメントを更新する。

```
E0415: return type mismatch
  expected: Int
  got:      String
  --> src/main.fav:3:10
  note: function is declared to return Int
```

`seq` ボディでの禁止時:

```
E0415: `return` is not allowed in `seq` body
  --> src/pipeline.fav:8:5
  note: use stage composition instead
```

### checker.rs — コンテキスト構造

`check_return_stmt` は `?` ではなく既存の `self.errors.push` / `type_error` メソッド方式で実装する
（checker.rs 全体がエラーを collect してから返す設計のため即時伝播は使わない）。

- `check_fn_def` 内: `ret_resolved` を `self.current_return_ty` にセット、終了時にリセット
- `check_trf_def`（stage）内: `output_ty` を `self.current_return_ty` にセット、終了時にリセット
- `check_flw_def`（seq）内: `self.current_return_ty = None` にセット（return 禁止コンテキスト）

## 完了条件

- `cargo test` 全通過（**2972 tests** passed, 0 failed）
- `v452000_tests` の 4 件が pass:
  - `return_type_ok`
  - `return_type_mismatch_e0415`
  - `return_in_stage_ok`
  - `return_in_closure_no_false_e0415`
- `cargo clippy --locked -D warnings` クリーン
- `CHANGELOG.md` に v45.2.0 エントリ追加
