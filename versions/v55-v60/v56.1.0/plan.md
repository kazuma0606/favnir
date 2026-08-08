# Plan — v56.1.0 — 境界付きジェネリクス本番品質化（where T: Interface 拡張）

## ステップ

### Step 1: 事前確認

- `fav/Cargo.toml` が `56.0.0` であることを確認（更新前）
- `error_catalog.rs` に E0421 が存在し、E0422 が存在しないことを確認
- `checker.rs` の `TypeConstraint::Interface` ブランチが `"E0325"` を emit していることを確認
- driver.rs に `v56100_tests` が存在しないことを確認
- `v55.5.0` 完了（E0421 追加済み）を確認
- `fav/self/checker.fav` に `E0325` 文字列参照がないことを確認（セルフホスト側への影響確認）→ 参照なし（確認済み）

---

### Step 2: `fav/Cargo.toml` バージョン更新

```toml
[package]
version = "56.1.0"
```

---

### Step 3: `error_catalog.rs` — E0422 追加

E0421 エントリ（v55.5.0 追加）の直後、E05xx セクションコメントの直前に挿入する。

```rust
// v56.1.0: where clause interface constraint (正式カタログ登録)
ErrorEntry {
    code: "E0422",
    title: "where clause interface constraint not satisfied",
    category: "types",
    description: "A generic function was called with a type argument that does not satisfy \
                  the required `where T: Interface` constraint. The type must implement \
                  the specified interface to be used in this position.",
    example: "fn pick<T with Ord>(a: T, b: T) -> T { if a > b { a } else { b } }\nfn main() -> Bool { pick(true, false) }  // E0422: Bool does not implement Ord",
    fix: "Use a type that implements the required interface, or add an `impl` for the interface on your type.",
    suggestion: Some("Ensure the type argument implements the required interface bound."),
},
```

（完全な文字列は spec.md §2 にも記載。両文書で一致していることを確認すること。）

---

### Step 4: `checker.rs` — E0325 → E0422 変更

変更箇所（3箇所）:
1. call-site 検証コメント（`// Bounded generics call-site check`）
2. `TypeConstraint::Interface` ブランチのエラーコード文字列
3. `type_implements_bound` 関数の doc コメント

変更は `TypeConstraint::Interface` ブランチのみ。`TypeConstraint::HasField`（E0337）は変更しない。

---

### Step 5: `driver.rs` — 既存テスト更新 + `v56100_tests` 追加

#### 5a. 既存テスト更新（E0325 → E0422）

更新対象（2テスト × 2モジュール）:
- `v171000_tests::bounded_generic_violation_e0325` → `bounded_generic_violation_e0422`（関数名 + assertion + コメント）
- `v321000_tests::bounded_generics_hash_violation_e0325` → `bounded_generics_hash_violation_e0422`（同上）
- `v321000_tests::bounded_generics_display_and_hash_bounds` のコメント `E0325 なし` → `E0422 なし`（2箇所）

#### 5b. `v56100_tests` を `v56000_tests` の直前に挿入

```rust
// -- v56100_tests (v56.1.0) -- 境界付きジェネリクス本番品質化 --
#[cfg(test)]
mod v56100_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    fn check_errors(src: &str) -> Vec<String> { ... }

    #[test]
    fn where_clause_e0422_emitted() { ... }  // E0422 が emitted される

    #[test]
    fn where_clause_stdlib_fn() { ... }       // errors.is_empty()
}
```

---

### Step 6: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo build 2>&1 | tail -5
```

期待結果: `Finished`

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep -E "^test result|v56100|FAILED"
```

期待結果: `3229 tests passed, 0 failed`、v56100 2 件 ok

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -5
```

期待結果: クリーン

---

### Step 7: ポスト処理

```
CHANGELOG.md               → v56.1.0 エントリ追加
versions/current.md        → v56.1.0 / 3229 tests に更新
roadmap-v56.1-v57.0.md    → v56.1.0 実績を COMPLETE に更新
roadmap-v55.1-v60.0.md    → v56.1.0 実績欄も COMPLETE に更新
```

---

## テスト数の変化

| 操作 | 件数 |
|------|------|
| v56.0.0 完了時点ベース | 3227 |
| `v56100_tests` 追加（`where_clause_e0422_emitted` / `where_clause_stdlib_fn`） | +2 |
| **合計（目標）** | **3229** |

---

## 注意事項

- `TypeConstraint::HasField`（E0337）は変更対象外。`TypeConstraint::Interface` のみ変更する。
- 既存テスト関数名に `_e0325` が含まれるものは `_e0422` に改名する（名前と実際の検証内容を一致させる）。
- `where_clause_stdlib_fn` の assert は `errors.is_empty()` を使用する（部分一致では不十分）。
