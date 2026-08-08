# Spec — v56.1.0 — 境界付きジェネリクス本番品質化（where T: Interface 拡張）

## 概要

v33.0「Language Power」で実装した `where T: Interface`（`T with Ord` 形式）を本番品質化する。
`error_catalog.rs` に E0422 を正式カタログ登録し、`checker.rs` の Interface 境界違反エラーコードを
E0325（未カタログ）から E0422（正式カタログ）へ変更する。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v56.1-v57.0.md` — v56.1.0 セクション
- `versions/roadmap/roadmap-v55.1-v60.0.md` — v56.1.0 行
- ベーステスト数: **3227**（v56.0.0 完了時点の実績値）
- 目標テスト数: **3229**（+2）

---

## 既存実装との関係

| 要素 | バージョン | 状態 |
|------|-----------|------|
| `where T: Interface`（`T with Ord` 形式）解析・評価 | v33.0 | 実装済み |
| `GenericParam.bounds: Vec<TypeConstraint>` | v17.1.0 | 実装済み |
| `TypeConstraint::Interface(String)` | v18.2.0 | 実装済み |
| `fn_bounds_registry` / call-site 検証 | v17.1.0 | 実装済み |
| E0421（`!State` エフェクトエラー） | v55.5.0 | 実装済み（E0422 の直前） |
| E0422 正式カタログ登録 | v56.1.0 | **本バージョンで追加** |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "56.1.0"
```

（既存コードの変更は `error_catalog.rs` と `checker.rs` のみ。新機能追加なし。）

---

### 2. `fav/src/error_catalog.rs` — E0422 エントリ追加

E0421 エントリの直後（E05xx セクションの直前）に挿入する。

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

---

### 3. `fav/src/middle/checker.rs` — E0325 → E0422 変更

`TypeConstraint::Interface` ブランチの `"E0325"` を `"E0422"` に変更する。
コメント（2箇所）も更新する。

変更箇所:
- call-site 検証コメント: `// Bounded generics call-site check (E0422 for Interface, E0337 for HasField).`
- エラーコード文字列: `"E0422"` （`TypeConstraint::Interface` ブランチ）
- docコメント: `/// Used for bounded generics call-site checking (E0422 for Interface, E0337 for HasField).`

> **注意**: `TypeConstraint::HasField` ブランチ（E0337）は変更しない。

---

### 4. `fav/src/driver.rs` — `v56100_tests` モジュール追加 + 既存テスト更新

#### 4a. 既存テスト assertion 更新（E0325 → E0422）

以下のテスト関数を更新する（関数名改名 + assertion 変更）:

| 変更前 | 変更後 |
|--------|--------|
| `fn bounded_generic_violation_e0325()` | `fn bounded_generic_violation_e0422()` |
| `fn bounded_generics_hash_violation_e0325()` | `fn bounded_generics_hash_violation_e0422()` |

各コメント内の `E0325` → `E0422` も更新する。

#### 4b. `v56000_tests` の直前に `v56100_tests` を挿入

```rust
// -- v56100_tests (v56.1.0) -- 境界付きジェネリクス本番品質化（where T: Interface 拡張）--
#[cfg(test)]
mod v56100_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    fn check_errors(src: &str) -> Vec<String> {
        let program = Parser::parse_str(src, "v56100_test.fav").expect("parse");
        Checker::check_program(&program)
            .0
            .iter()
            .map(|e| e.code.to_string())
            .collect()
    }

    #[test]
    fn where_clause_e0422_emitted() {
        // Interface 境界違反で E0422 が出ることを確認（v56.1.0 で正式カタログ登録）
        let errors = check_errors(r#"
fn pick_larger<T with Ord>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
fn main() -> Bool {
    pick_larger(true, false)
}
"#);
        assert!(
            errors.iter().any(|e| e == "E0422"),
            "Expected E0422 for Bool not implementing Ord, got: {:?}",
            errors
        );
    }

    #[test]
    fn where_clause_stdlib_fn() {
        // 正しい型で constrained generic を呼んだ場合はエラーなし
        let errors = check_errors(r#"
fn pick_larger<T with Ord>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
fn main() -> Int {
    pick_larger(3, 7)
}
"#);
        assert!(
            errors.is_empty(),
            "pick_larger<Int> should not emit any errors (Int implements Ord), got: {:?}",
            errors
        );
    }
}
```

---

## テスト仕様

| テスト名 | 検証内容 |
|---------|---------|
| `where_clause_e0422_emitted` | `Bool with Ord` 境界違反で E0422 が emitted される |
| `where_clause_stdlib_fn` | `Int with Ord` は境界を満たすためエラーなし（`errors.is_empty()`）|

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3229 tests passed, 0 failed**）
- `cargo clippy -- -D warnings` クリーン
- `where_clause_e0422_emitted` pass
- `where_clause_stdlib_fn` pass
- `error_catalog.rs` に E0422 エントリが含まれる
- `checker.rs` の Interface 境界違反が E0422 を emit する（E0325 は使用されていない）
- 既存テスト `bounded_generic_violation_e0422` / `bounded_generics_hash_violation_e0422` が pass
- `CHANGELOG.md` に v56.1.0 エントリが追加されている
- `versions/current.md` が v56.1.0 / 3229 tests を反映
- `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.1.0 実績を COMPLETE に更新
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v56.1.0 実績欄も COMPLETE に更新

---

## 備考

- E0325 は元々 `error_catalog.rs` に登録されていなかった（checker 内部のみで使用）。
  v56.1.0 で E0422 として正式カタログ化し、エラーコードの一貫性を確保する。
- `TypeConstraint::HasField`（E0337）は別の制約体系であり変更しない。
- **ロードマップ記載「stdlib 各関数定義への `where` 節付与」について**:
  ロードマップ（roadmap-v56.1-v57.0.md L38）では「stdlib の各関数定義に `where` 節を適切に付与」と記載されているが、
  v56.1.0 のスコープはエラーコード正式カタログ化（E0422）のみとする。
  stdlib への `where` 節付与は v56.2.0 以降のスプリントで対応予定。
- **`fav/self/checker.fav` への影響**:
  `checker.fav` 内に `E0325` 文字列参照がないことを確認済み（セルフホスト側への影響なし）。
- コードレビュー対応（[LOW]×3）:
  - テスト関数名を `_e0422` に改名（関数名と検証コードの一致）
  - `where_clause_stdlib_fn` の assert を `errors.is_empty()` に強化
  - コメント内の `E0325 なし` → `E0422 なし` に更新（2箇所）
