# v71.9.0 実装計画 — 安定化・コードフリーズ（Type System 2.0 前調整）

Date: 2026-08-11

---

## 依存関係

```
T0（事前確認）
  └→ T1（v719000_tests 追加）
       └→ T2（cargo_toml_version テスト更新）
            └→ T3（Cargo.toml バージョン更新）
                 └→ T4（cargo test v719000 確認）
                      └→ T5（CHANGELOG.md 更新）
                           └→ T6（versions/current.md 更新）
                                └→ T7（最終確認）
```

---

## 実装ステップ

### Step 0: 事前確認

- `fav/Cargo.toml` のバージョンが `71.8.0` であることを確認
- `cargo test` が 3606 tests pass であることを確認
- `driver.rs` に `v718000_tests` モジュールが存在し、`v719000_tests` が未存在であることを確認

### Step 1: `v719000_tests` モジュール追加（`driver.rs`）

`v718000_tests` モジュールの直後に以下を追加:

```rust
// ── v71.9.0 tests — 安定化・コードフリーズ（Type System 2.0 前調整） ─────────
#[cfg(test)]
mod v719000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    /// v71.1〜v71.8 の全機能が一つのプログラムで共存してエラーなしであることを確認。
    #[test]
    fn type_system_2_all_stable() {
        let src = concat!(
            // v71.1: 依存型 Vec<T>[N]
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
        let program = Parser::parse_str(src, "test.fav").expect("parse should succeed");
        let (errors, _) = Checker::check_program(&program);
        assert!(
            errors.is_empty(),
            "all v71.x features should coexist without errors: {:?}",
            errors
        );
    }

    /// 依存型 + refined type + phantom type を組み合わせた E2E テスト。
    #[test]
    fn dependent_refined_phantom_e2e() {
        let src = concat!(
            "const VEC_DIM: Int = 384\n",
            "type UserId = phantom String\n",
            "type Score = Float where self >= 0.0\n",
            "fn similarity(a: Vec<Float>[384], b: Vec<Float>[384]) -> Float { 0.0 }\n",
            "fn get_user(id: UserId) -> Bool { true }\n",
            "fn good_user() -> Bool { get_user(UserId(\"u-123\")) }\n",
            "public fn main() -> Bool { true }\n",
        );
        let program = Parser::parse_str(src, "test.fav").expect("parse should succeed");
        let (errors, _) = Checker::check_program(&program);
        assert!(
            errors.is_empty(),
            "dependent+refined+phantom combined should typecheck without errors: {:?}",
            errors
        );
    }
}
```

### Step 2: `cargo_toml_version` テスト文字列を更新

`driver.rs` 内の `"71.8.0"` バージョンアサーション文字列を `"71.9.0"` に一括更新。

### Step 3: `fav/Cargo.toml` バージョン更新

`version = "71.8.0"` → `version = "71.9.0"`

### Step 4: テスト確認

`cargo test v719000` で 2 件 pass を確認。

### Step 5: `CHANGELOG.md` 更新

先頭に `## [v71.9.0]` エントリを追加。

### Step 6: `versions/current.md` 更新

- 進行中: `v71.9.0`（安定化・コードフリーズ）
- 次: `v72.0.0`

### Step 7: 最終確認

`cargo test` 全体で 3608 tests pass（0 failures）を確認。

---

## 注意事項

- `type_system_2_all_stable` のソースは v71.6（AOT）・v71.7（WASM）を直接テストしない
  — これらは実行バイナリを要求するため別途テスト済み（`aot_native_binary_runs_hello`、`wasm_target_runs_simple_pipeline`）
- `interface Sortable { key: Self -> Int }` はフィールド記法（`name: TypeExpr`）を使う。`fn` キーワード付きの `fn key(...)` は parser が Colon を要求するためパースエラーになる
- `>=` 演算子は `BinOp::GtEq` として `checker.rs` line 4799 に実装済み。`where self >= 0.0` は問題なく動作する（確認不要）

