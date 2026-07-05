# v32.6.0 — Spec: 分散アノテーション 確認・テスト補強

## 概要

v32.6.0 は **分散アノテーション（Variance Annotations）** の確認・テスト補強バージョン。

ロードマップ v32.5〜v32.9 候補「ジェネリクスの `impl` 対応」に隣接する型システム機能として、
`+T`（共変）・`-T`（反変）アノテーションが仕様通りに動作することを確認する。
実際にはすでに v18.6.0 で実装済みである:

| コンポーネント | 実装済み | バージョン |
|---|---|---|
| `Variance::Covariant`（`+T`）/ `Variance::Contravariant`（`-T`）| ✓ | v18.6.0 |
| `GenericParam.variance` フィールド | ✓ | v18.6.0 |
| `parse_type_params` — `<+T>` / `<-T>` パース | ✓ | v18.6.0 |
| E0334 — variance violation（共変パラメータの入力位置使用） | ✓ | v18.6.0 |
| `v186000_tests` — 5 件のテスト（うち 1 件 `#[ignore]`） | ✓ | v18.6.0 |

v32.6.0 では、分散アノテーションの型チェック（E0334）が仕様通りに動作することを
`v326000_tests` で明示的に確認し、バージョンと CHANGELOG を更新する。

---

## 分散アノテーション仕様確認

### 構文

```favnir
// 共変（+T）: T は出力位置にのみ現れる
interface Source<+T> { next: Unit -> Option<T> }

// 反変（-T）: T は入力位置にのみ現れる
interface Sink<-T> { write: T -> Unit }
```

### エラーコード

| エラー | 条件 | コード |
|---|---|---|
| E0334 | 共変パラメータ `+T` が入力（引数）位置に使われている | `"variance violation"` |

---

## 追加するテスト（v326000_tests — 4 件）

`v326000_tests` は v32.1.0〜v32.5.0 と同じパターン:
- `use super::*` **なし**
- モジュール内に独自の `check_errors` を定義
- `use crate::frontend::parser::Parser; use crate::middle::checker::Checker;`

テスト名は v186000_tests（`variance_covariant_parses` / `variance_contravariant_parses` / `variance_subtype_covariant` / `variance_violation_error` / `variance_contravariant_subtype`）と被らないよう `variance_ann_` プレフィックスを使用する。

### テスト 1: バージョン確認

```rust
fn cargo_toml_version_is_32_6_0() {
    let src = include_str!("../Cargo.toml");
    assert!(src.contains("32.6.0"), "Cargo.toml must contain '32.6.0'");
}
```

### テスト 2: ベンチマーク存在確認

```rust
fn benchmark_v32_6_0_exists() {
    let src = include_str!("../../benchmarks/v32.6.0.json");
    assert!(src.contains("32.6.0"), "benchmarks/v32.6.0.json must contain '32.6.0'");
}
```

### テスト 3: 共変パラメータが出力位置にある → E0334 なし（ポジティブ）

```rust
fn variance_ann_covariant_output_pass() {
    // +T が出力位置（返り値）にのみ使われる → E0334 なし
    let errors = check_errors(r#"
interface Source<+T> {
    next: Unit -> Option<T>
}
"#);
    assert!(
        errors.iter().all(|e| e != "E0334"),
        "Covariant +T in output position should not produce E0334: {:?}",
        errors
    );
}
```

### テスト 4: 共変パラメータが入力位置にある → E0334（ネガティブ）

```rust
fn variance_ann_covariant_input_e0334() {
    // +T が入力位置（引数）に使われる → E0334
    let errors = check_errors(r#"
interface BadSource<+T> {
    write: T -> Unit
}
"#);
    assert!(
        errors.iter().any(|e| e == "E0334"),
        "Expected E0334 for covariant +T in input position, got: {:?}",
        errors
    );
}
```

---

## テストモジュールの配置

`v326000_tests` は `v325000_tests` の閉じ括弧（`}`）の直後、
かつ `// ── v31.7.0 tests` コメントの前に挿入する。

---

## 完了条件

- `Cargo.toml` version = `"32.6.0"`
- `cargo_toml_version_is_32_5_0` が空スタブになっていること
- `variance_ann_covariant_output_pass` テストが PASS（E0334 なし）
- `variance_ann_covariant_input_e0334` テストが PASS（E0334 を確認）
- `cargo test --bin fav v326000` — 4/4 PASS
- `cargo test` — 全件 PASS（2480 件、0 failures）
- `CHANGELOG.md` に `[v32.6.0]` セクション
- `benchmarks/v32.6.0.json` 存在かつ `tests_passed` が実測値
- `benchmarks/v32.6.0.json` の `milestone` フィールドが `"Language Power"` であること
- `versions/current.md` を v32.6.0 に更新
- `tasks.md` がすべて `[x]` で COMPLETE に更新されていること
- site/ MDX 更新: 対象外（`variance-annotations.mdx` 等は既に完成）
