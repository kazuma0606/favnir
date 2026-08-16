# v71.1.0 Plan — 依存型の基礎 `Vec<T>[N]`

Date: 2026-08-09
Status: 計画中

---

## 実装ステップ（依存順）

### Step 1: 現行パーサー・チェッカーの `Vec<T>[N]` 対応状況を確認

```bash
# Vec<Float>[1536] が現行パーサーでパースできるか確認
cargo test -- --nocapture 2>&1 | grep -i "vec"
```

実際にテストソースを書いて `Parser::parse_str` を試し、エラーが出るか確認する。

---

### Step 2: パーサー対応（必要な場合）

現行パーサーが `Vec<Float>[N]` を解析できない場合、`src/frontend/parser.rs` に対応を追加:

- `parse_type_expr` の `Vec<T>` パース後に `[N]` の省略可能サフィックスを追加
- `TypeExpr` に `VecDim` バリアント追加（または `Generic` に次元フィールドを追加）

---

### Step 3: チェッカーに E0420 追加（次元不一致エラー）

`src/middle/checker.rs` に E0420 を追加:
- `Vec<Float>[1536]` と `Vec<Float>[768]` の型統合（unify）時に次元を比較
- 次元が定数かつ不一致 → `TypeError { code: "E0420", ... }`

---

### Step 4: driver.rs に `v711000_tests` を追加

`v71000_tests` の直後（driver.rs 末尾）に追加:

```rust
// ── v71.1.0: 依存型の基礎 Vec<T>[N] ─────────────────────────────────────────

#[cfg(test)]
mod v711000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    #[test]
    fn dependent_type_vec_dim_param() {
        // Vec<Float>[1536] の型注釈が parse + typecheck で通ることを確認
        let src = concat!(
            "fn process(v: Vec<Float>[1536]) -> Int { 1536 }\n",
            "public fn main() -> Bool { true }\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse should succeed");
        let (errors, _) = Checker::check_program(&prog);
        assert!(
            !errors.iter().any(|e| e.code == "E9999"),
            "Vec<Float>[N] should typecheck cleanly; errors: {:?}",
            errors
        );
    }

    #[test]
    fn dependent_type_dim_mismatch_error() {
        // Vec<Float>[1536] と Vec<Float>[768] を同じ引数に渡す → E0420
        let src = concat!(
            "fn dot(a: Vec<Float>[1536], b: Vec<Float>[1536]) -> Float { 0.0 }\n",
            "fn bad(v: Vec<Float>[768]) -> Float { dot(v, v) }\n",
            "public fn main() -> Bool { true }\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse should succeed");
        let (errors, _) = Checker::check_program(&prog);
        assert!(
            errors.iter().any(|e| e.code == "E0420"),
            "dimension mismatch should produce E0420; errors: {:?}",
            errors
        );
    }
}
```

Note: Step 2/3 でパーサー・チェッカーを改修した後に、このテストが通るか確認する。

---

### Step 5: Cargo.toml バージョン更新

- `fav/Cargo.toml` の `version = "71.0.0"` → `"71.1.0"`
- driver.rs 内の全バージョン文字列を一括更新（replace_all）

---

### Step 6: CHANGELOG.md 更新

ヘッダー形式: `## [v71.1.0] — 2026-08-09 — 依存型の基礎 Vec<T>[N]`

---

### Step 7: 最終確認

- `cargo test v711000` で 2 件 pass
- `cargo test` 全体で 3586 tests pass（0 failures）
- `versions/current.md` を v71.1.0 に更新
