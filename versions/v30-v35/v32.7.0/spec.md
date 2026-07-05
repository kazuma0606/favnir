# v32.7.0 — Spec: 定数ジェネリクス 確認・テスト補強

## 概要

v32.7.0 は **定数ジェネリクス（Const Generics）** の確認・テスト補強バージョン。

ロードマップ v32.5〜v32.9 候補のうち「ジェネリクスの `impl` 対応」に隣接する機能として、
`<const N: Int>` 構文および N > 0 などの制約（E0335）が仕様通りに動作することを確認する。
実際にはすでに v18.7.0 で実装済みである:

| コンポーネント | 実装済み | バージョン |
|---|---|---|
| `GenericParam.is_const` / `const_ty` / `const_constraint` | ✓ | v18.7.0 |
| `parse_type_param` — `<const N: Int>` / `<const N: Int where { N > 0 }>` パース | ✓ | v18.7.0 |
| `const_generics_registry` — 関数登録 | ✓ | v18.7.0 |
| E0335 — const constraint violation | ✓ | v18.7.0 |
| `v187000_tests` — 6 件のテスト（うち 2 件 `#[ignore]`） | ✓ | v18.7.0 |

v32.7.0 では、定数ジェネリクスの制約チェック（E0335）が仕様通りに動作することを
`v327000_tests` で明示的に確認し、バージョンと CHANGELOG を更新する。

---

## 定数ジェネリクス仕様確認

### 構文

```favnir
// const パラメータ（制約なし）
fn first<const N: Int>(items: Int) -> Int { 0 }

// const パラメータ（制約あり）
fn safe_chunk<const N: Int where { N > 0 }>(items: Int) -> Int { 0 }

// 呼び出し
safe_chunk<100>(42)  // OK — 制約 N > 0 を満たす
safe_chunk<0>(42)    // E0335 — 制約 N > 0 を違反
```

### エラーコード

| エラー | 条件 | コード |
|---|---|------|
| E0335 | const ジェネリクス制約（`where { N > 0 }` 等）に違反する引数が渡された | `"E0335"` |

---

## 追加するテスト（v327000_tests — 4 件）

`v327000_tests` は v32.1.0〜v32.6.0 と同じパターン:
- `use super::*` **なし**
- モジュール内に独自の `check_errors` を定義
- `use crate::frontend::parser::Parser; use crate::middle::checker::Checker;`

テスト名は v187000_tests（`const_generic_parses` / `const_generic_constraint_parses` / `const_generic_violation` / `const_generic_valid`）と被らないよう `const_gen_` プレフィックスを使用する。

### テスト 1: バージョン確認

```rust
fn cargo_toml_version_is_32_7_0() {
    let src = include_str!("../Cargo.toml");
    assert!(src.contains("32.7.0"), "Cargo.toml must contain '32.7.0'");
}
```

### テスト 2: ベンチマーク存在確認

```rust
fn benchmark_v32_7_0_exists() {
    let src = include_str!("../../benchmarks/v32.7.0.json");
    assert!(src.contains("32.7.0"), "benchmarks/v32.7.0.json must contain '32.7.0'");
}
```

### テスト 3: 制約を満たす const 引数 → E0335 なし（ポジティブ）

```rust
fn const_gen_chunk_size_valid() {
    // N = 5 は N > 0 を満たす → E0335 なし
    // (テスト名は v187000_tests::const_generic_valid と異なる)
    let errors = check_errors(r#"
fn safe_chunk<const N: Int where { N > 0 }>(items: Int) -> Int { 0 }
fn main() -> Int { safe_chunk<5>(100) }
"#);
    assert!(
        errors.iter().all(|e| e != "E0335"),
        "const N=5 satisfies N>0, should not produce E0335: {:?}",
        errors
    );
}
```

### テスト 4: 制約を違反する const 引数 → E0335（ネガティブ）

```rust
fn const_gen_chunk_size_zero_e0335() {
    // N = 0 は N > 0 を違反 → E0335
    // (テスト名は v187000_tests::const_generic_violation と異なる)
    let errors = check_errors(r#"
fn safe_chunk<const N: Int where { N > 0 }>(items: Int) -> Int { 0 }
fn main() -> Int { safe_chunk<0>(100) }
"#);
    assert!(
        errors.iter().any(|e| e == "E0335"),
        "Expected E0335 for const N=0 violating N>0, got: {:?}",
        errors
    );
}
```

---

## テストモジュールの配置

`v327000_tests` は `v326000_tests` の閉じ括弧（`}`）の直後、
かつ `// ── v31.7.0 tests` コメントの前に挿入する。

---

## 完了条件

- `Cargo.toml` version = `"32.7.0"`
- `cargo_toml_version_is_32_6_0` が空スタブになっていること
- `const_gen_chunk_size_valid` テストが PASS（E0335 なし）
- `const_gen_chunk_size_zero_e0335` テストが PASS（E0335 を確認）
- `cargo test --bin fav v327000` — 4/4 PASS
- `cargo test` — 全件 PASS（2484 件、0 failures）
- `CHANGELOG.md` に `[v32.7.0]` セクション
- `benchmarks/v32.7.0.json` 存在かつ `tests_passed` が実測値
- `benchmarks/v32.7.0.json` の `milestone` フィールドが `"Language Power"` であること
- `versions/current.md` を v32.7.0 に更新
- `tasks.md` がすべて `[x]` で COMPLETE に更新されていること
- site/ MDX 更新: 対象外（const generics は v18.7.0 で完成済み）
