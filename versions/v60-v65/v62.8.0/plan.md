# v62.8.0 Plan — AOT エラーコード E0427（AOT 未サポート機能検出）

Version: 62.8.0
Status: 未着手

---

## 実装順序

### Step 1: `cranelift_aot.rs` — `contains_aot_unsupported` + `validate_aot_compat` 追加

`CraneliftBackend` の `impl` ブロック閉じ括弧（`analyze_for_inlining` 定義行の後）の **外側** に
private fn `contains_aot_unsupported` と pub fn `validate_aot_compat` を追加する。

- `analyze_for_inlining` は `impl CraneliftBackend` 内にある（`pub(crate)` 関連メソッド）。
- `validate_aot_compat` は impl 外の standalone `pub fn` として配置し、
  呼び出しを `crate::backend::cranelift_aot::validate_aot_compat(&ir)` で統一する。
- `contains_aot_unsupported` の `IRStmt::Block` アームには `Chain / Yield / Return / SeqChain` を含める
  （`_ => false` を残さず exhaustive に処理することで検出漏れを防ぐ）。

`cargo build` でエラーなし確認。

### Step 2: `error_catalog.rs` — E0427 エントリ追加

`E0426` エントリの直後（`// E05xx: モジュール` セクションコメントの直前）に
`// ── E0427: AOT 未サポート機能 (v62.8.0)` セクションとして E0427 エントリを追加。
`long_description` は詳細な Markdown 形式で記述（`E0103` のパターンに倣う）。
`cargo build` でエラーなし確認。

### Step 3: `driver.rs` — `cmd_build_aot_validate` 追加

`cmd_build_aot_stats` の直後に追加。
`cargo build` でエラーなし確認。

### Step 4: `driver.rs` — `v62800_tests` 追加

`v62700_tests` の直前（ファイル先頭方向）に挿入。
`cargo test v62800` で 2 件 PASS 確認。

### Step 5: 全テスト

`cargo test -j 8 -- --test-threads=8` で 3399 tests passed, 0 failed を確認。

### Step 6: ドキュメント更新

roadmap / current.md / CHANGELOG.md / tasks.md を更新。

---

## 設計メモ

### `validate_aot_compat` の検出範囲

v62.8.0 では `IRExpr::Emit` のみを AOT 非対応としてフラグする。
`is_aot_pure` が `false` を返すケース（`IRExpr::Call` 等）は「AOT で最適化されない」ことを意味するが、
実際には VM dispatch にフォールバックするため「エラー」ではない。
E0427 は「実行自体が AOT では不可能な機能」に限定する。

### `contains_aot_unsupported` の実装方針

`is_aot_pure` とは独立した関数として実装する（関心の分離）。
`is_aot_pure` は「インライン展開できるか」の判定、
`contains_aot_unsupported` は「AOT で実行できないか」の判定という異なる目的を持つ。

### テスト用ソースコードの選択

`emit "hello"` は `!Emit<E>` 型注釈なしでパースとコンパイルが通る（v34.8A 以降）。
`Expr::EmitExpr` → `IRExpr::Emit` のコンパイルパスは `middle/compiler.rs:2324` で確認済み。
`fn main() -> Bool { true }` を同じソースに含めることで
「`main` は E0427 フリー、`f` は E0427」という状態を作り、
`errors.join("\n")` の出力に `"f"` が含まれることをアサートできる。

### `error_catalog_has_e0427` テストの実装

```rust
let entry = crate::error_catalog::ERROR_CATALOG
    .iter()
    .find(|e| e.code == "E0427")
    .expect("E0427 should be in error catalog");
assert_eq!(entry.category, "build");
```

`ERROR_CATALOG` は `pub const` なのでどのモジュールからも `crate::error_catalog::ERROR_CATALOG` で参照可能。

### `#[cfg(not(target_arch = "wasm32"))]` の要否

`validate_aot_compat` は `compile_program` を呼ばず IR を受け取るだけなので
`std::process::Command` 等の WASM 非対応 API は使わない → `#[cfg]` 不要。
`cmd_build_aot_validate` は `compile_program` を使うが `std::process::Command` は使わない → `#[cfg]` 不要。

### ロードマップとの乖離

- ベーステスト数: ロードマップ記載 3396 → 実際 3397（v62.7.0 code-reviewer 対応 +1）
- ターゲット: 3397 + 2 = 3399（ロードマップ記載 3398 より +1）
- `aot.rs` 記載: ロードマップは `aot.rs` と書いているが実装は `cranelift_aot.rs`（既存ファイル構造に従う）
