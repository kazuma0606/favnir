# v71.9.0 spec — 安定化・コードフリーズ（Type System 2.0 前調整）

Date: 2026-08-11

---

## Background

v71.1〜v71.8 で Type System 2.0 の各機能を実装した:
- v71.1: 依存型 `Vec<T>[N]`（`E0421` 次元不一致エラー）
- v71.2: Refined Types（`type X = T where self > expr`、E0425）
- v71.3: Phantom Types（`type X = phantom T`、ID 混用防止）
- v71.4: Const/Compile-Time Evaluation（`const N: Int = expr`）
- v71.5: Generic Constraints（`<T: A & B>`、`<T: impl A>`）
- v71.6: AOT Native Compilation 本番品質化（`--arch arm64`、strip）
- v71.7: WebAssembly テストカバレッジ確立
- v71.8: 型推論強化（`bind` / クロージャ引数の型注釈省略）

v71.9.0 はこれらの全機能が干渉なく共存できることを確認する安定化バージョン。
特に依存型・refined type・phantom type を組み合わせた E2E テストにより v72.0 への移行準備を完了する。

---

## Goals

1. `type_system_2_all_stable`: v71.1〜v71.8 の全機能を一つのプログラムに組み合わせ、
   `Checker::check_program` でエラーなしであることを確認
2. `dependent_refined_phantom_e2e`: 依存型 + refined type + phantom type を組み合わせた
   E2E プログラムが `Checker::check_program` でエラーなしであることを確認
3. Cargo.toml バージョンを `71.9.0` に更新

---

## テスト詳細

### `type_system_2_all_stable`

v71.1〜v71.8 の各機能を網羅したプログラムが型チェックを通過することを確認。

```rust
let src = concat!(
    // v71.1: 依存型
    "fn dot_product(a: Vec<Float>[1536], b: Vec<Float>[1536]) -> Float { 0.0 }\n",
    // v71.2: refined types
    "type PositiveFloat = Float where self > 0.0\n",
    "fn safe_log(x: PositiveFloat) -> Float { 1.0 }\n",
    // v71.3: phantom types
    "type UserId = phantom String\n",
    "fn get_user(id: UserId) -> Bool { true }\n",
    // v71.4: const eval
    "const EMBED_DIM: Int = 1536\n",
    "fn get_dim() -> Int { EMBED_DIM }\n",
    // v71.5: generic constraints
    "interface Sortable { key: Self -> Int }\n",
    "fn top_item<T: Sortable>(a: T) -> T { a }\n",
    // v71.8: bind type inference
    "fn current_count() -> Int { 42 }\n",
    "fn main() -> Int { bind n <- current_count() n }\n",
);
let program = Parser::parse_str(src, "test.fav").expect("parse");
let (errors, _) = Checker::check_program(&program);
assert!(errors.is_empty(), "all v71.x features should coexist without errors: {:?}", errors);
```

### `dependent_refined_phantom_e2e`

依存型 + refined type + phantom type の 3 機能を組み合わせた E2E プログラム。

```rust
let src = concat!(
    "const VEC_DIM: Int = 384\n",
    "type UserId = phantom String\n",
    "type Score = Float where self >= 0.0\n",
    "fn similarity(a: Vec<Float>[384], b: Vec<Float>[384]) -> Float { 0.0 }\n",
    "fn get_user(id: UserId) -> Bool { true }\n",
    "fn good_user() -> Bool { get_user(UserId(\"u-123\")) }\n",
    "public fn main() -> Bool { true }\n",
);
let program = Parser::parse_str(src, "test.fav").expect("parse");
let (errors, _) = Checker::check_program(&program);
assert!(errors.is_empty(), "dependent+refined+phantom combined should typecheck: {:?}", errors);
```

---

## 使用する内部 API

```rust
// driver.rs 内 mod — crate:: パスで参照
use crate::frontend::parser::Parser;
use crate::middle::checker::Checker;
```

---

## Success Criteria

- `cargo test v719000` で 2 件 pass（0 failures）
- `cargo test` 全体で 3608 tests pass（3606 + 2）
- `fav/Cargo.toml` のバージョンが `71.9.0`

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v719000_tests` モジュール追加（2 テスト）+ cargo_toml_version 更新 |
| `fav/Cargo.toml` | バージョン `71.8.0` → `71.9.0` |
| `CHANGELOG.md` | `## [v71.9.0]` エントリ追加 |
| `versions/current.md` | 進行中: v71.9.0 / 次: v72.0.0 |

---

## スコープ外

- v71.6.0（AOT）のバイナリ実行テスト: `aot_native_binary_runs_hello` で既にカバー済み
- v71.7.0（WASM）の実行テスト: `wasm_target_runs_simple_pipeline` で既にカバー済み
- `v72.0.0` 宣言・`MILESTONE.md` 更新: v72.0.0 スコープ
- `site/` MDX 追加: 機能追加なしのため対象外
