# v71.8.0 実装計画 — 型推論強化（型注釈省略可能範囲の拡大）

Date: 2026-08-11

---

## 依存関係

```
T0（事前確認）
  └→ T1（v718000_tests 追加）
       └→ T2（cargo_toml_version テスト更新）
            └→ T3（Cargo.toml バージョン更新）
                 └→ T4（cargo test v718000 確認）
                      └→ T5（CHANGELOG.md 更新）
                           └→ T6（versions/current.md 更新）
                                └→ T7（最終確認）
```

---

## 実装ステップ

### Step 0: 事前確認

- `fav/Cargo.toml` のバージョンが `71.7.0` であることを確認
- `cargo test` が 3604 tests pass であることを確認
- `driver.rs` に `v717000_tests` モジュールが存在し、`v718000_tests` が未存在であることを確認
- `checker.rs` の `Stmt::Bind` ハンドラ（line ~4301）で `annotated_ty = None` 時に
  `check_pattern_bindings(&b.pattern, &effective_ty)` が呼ばれることを確認（型推論済み）
- `checker.rs` の `Expr::Closure` ハンドラ（line ~5471）でパラメータが `Type::Unknown` に
  初期化されていることを確認

### Step 1: `v718000_tests` モジュール追加（`driver.rs`）

`v717000_tests` モジュールの直後に以下を追加:

```rust
// ── v71.8.0 tests — 型推論強化（型注釈省略可能範囲の拡大） ──────────────────
#[cfg(test)]
mod v718000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    /// bind 束縛の型注釈省略: RHS の戻り型から型を推論できることを確認。
    /// checker.rs line ~4330: annotated_ty = None の場合 effective_ty で check_pattern_bindings。
    #[test]
    fn type_infer_local_var_omit_annotation() {
        let src = r#"
fn get_values() -> List<Int> { List.of(1, 2, 3) }
fn main() -> Int {
    bind items <- get_values()
    List.length(items)
}
"#;
        let program = Parser::parse_str(src, "test.fav").expect("parse should succeed");
        let (errors, _) = Checker::check_program(&program);
        assert!(
            errors.is_empty(),
            "bind without type annotation should infer type from RHS: {:?}",
            errors
        );
    }

    /// クロージャ引数の型注釈省略: Unknown → unify 互換でエラーなしであることを確認。
    /// checker.rs line ~5473: params は Type::Unknown で初期化。
    /// unify line ~503: Unknown は任意の型と互換。
    #[test]
    fn type_infer_closure_arg_omit() {
        let src = r#"
fn main() -> Int {
    bind items <- List.of(1, 2, 3)
    bind total <- List.fold(items, 0, |acc, x| acc + x)
    total
}
"#;
        let program = Parser::parse_str(src, "test.fav").expect("parse should succeed");
        let (errors, _) = Checker::check_program(&program);
        assert!(
            errors.is_empty(),
            "closure args without type annotation should type-check: {:?}",
            errors
        );
    }
}
```

### Step 2: `cargo_toml_version` テスト文字列を更新

`driver.rs` 内の `"71.7.0"` バージョンアサーション文字列を `"71.8.0"` に一括更新。

### Step 3: `fav/Cargo.toml` バージョン更新

`version = "71.7.0"` → `version = "71.8.0"`

### Step 4: `cargo test v718000` で 2 件 pass 確認

`cargo test` 全体で 3606 tests pass を確認。

### Step 5: `CHANGELOG.md` 更新

先頭に `## [v71.8.0]` エントリを追加。

### Step 6: `versions/current.md` 更新

- 進行中: `v71.8.0`（型推論強化）
- 次: `v71.9.0`

---

## 注意事項

- `v718000_tests` は `#[cfg(not(target_arch = "wasm32"))]` ガード不要（`wasm_exec` を使わない）
- テストは `Checker::check_program` を直接呼ぶため、`use crate::middle::checker::Checker` が必要
- `List.fold`（checker.rs line ~6416）と `List.length`（line ~6353）は `check_builtin_apply` の match アームで処理される
- `List.of` は `check_builtin_apply` の専用アームがなく `Type::Unknown` を返すが、`expect_list_arg`（line ~7861）が `Unknown` を受け入れるためエラーにならない
- `register_builtins` は名前空間を登録するだけで `List.of` の型は登録しない（誤解注意）
- ソース内で W001（型未解決 Unknown 警告）が出ないことも確認したい場合は `warnings.is_empty()` も assert できるが、v71.8.0 のスコープは errors のみ
