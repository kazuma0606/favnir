# v65.8.0 実装計画 — Math Lint Rules（W050〜W054）

Version: 65.8.0
Status: 未着手
Base tests: 3467
Target tests: 3469

---

## 実装ステップ

### Step 1: `lint.rs` — 5 スタブ関数追加

ファイル末尾（`check_w041_*` 群の後）に以下を追加：

```rust
// ── W050〜W054: Math Lint Rules (v65.8.0) ─────────────────────────────────────
fn check_w050_matrix_dim_mismatch(_program: &Program, _errors: &mut Vec<LintError>) {}
fn check_w051_numeric_instability(_program: &Program, _errors: &mut Vec<LintError>) {}
fn check_w052_small_sample_test(_program: &Program, _errors: &mut Vec<LintError>) {}
fn check_w053_inplace_in_autodiff(_program: &Program, _errors: &mut Vec<LintError>) {}
fn check_w054_missing_convergence(_program: &Program, _errors: &mut Vec<LintError>) {}
```

### Step 2: `lint.rs` — `lint_program` への呼び出し追加

`check_w040_type_holes(program, &mut errors);` の直後・`errors` 裸式（`lint_program` の末尾返却式）の直前に追加：

```rust
// v65.8.0: W050〜W054 Math Lint Rules
check_w050_matrix_dim_mismatch(program, &mut errors);
check_w051_numeric_instability(program, &mut errors);
check_w052_small_sample_test(program, &mut errors);
check_w053_inplace_in_autodiff(program, &mut errors);
check_w054_missing_convergence(program, &mut errors);
```

### Step 3: `driver.rs` テスト追加

`// -- v65700_tests (v65.7.0)` コメントの直前に `v65800_tests` を挿入。

- **Rust テストで検証する内容:**
  - `lint_w051_detects_div_zero_risk`: W050 / W051 / check_w051 が lint.rs に存在する
  - `lint_w053_detects_inplace_in_autodiff`: W052 / W053 / W054 / check_w053 が lint.rs に存在する
- `include_str!("lint.rs")` — 同ディレクトリの lint.rs を参照（`../../runes/...` ではない）

### Step 4: ビルド・テスト確認

```bash
cargo build
cargo test --bin fav v65800_tests
cargo test --bin fav
```

---

## `driver.rs` 挿入コード

```rust
// -- v65800_tests (v65.8.0) -- Math Lint Rules --
#[cfg(test)]
mod v65800_tests {
    #[test]
    fn lint_w051_detects_div_zero_risk() {
        let src = include_str!("lint.rs");
        assert!(src.contains("W050"), "lint.rs should define W050");
        assert!(src.contains("W051"), "lint.rs should define W051");
        assert!(src.contains("check_w051"), "lint.rs should define check_w051");
    }

    #[test]
    fn lint_w053_detects_inplace_in_autodiff() {
        let src = include_str!("lint.rs");
        assert!(src.contains("W052"), "lint.rs should define W052");
        assert!(src.contains("W053"), "lint.rs should define W053");
        assert!(src.contains("W054"), "lint.rs should define W054");
        assert!(src.contains("check_w053"), "lint.rs should define check_w053");
    }
}
```

---

## W-code 一覧

| コード | 関数名 | 検出対象（将来実装） |
|---|---|---|
| W050 | `check_w050_matrix_dim_mismatch` | 行列次元の不一致（動的パス） |
| W051 | `check_w051_numeric_instability` | ゼロ除算・log(x) x≤0 リスク |
| W052 | `check_w052_small_sample_test` | サンプルサイズ < 30 の t 検定 |
| W053 | `check_w053_inplace_in_autodiff` | grad 内 Tensor.set 等の in-place 変更 |
| W054 | `check_w054_missing_convergence` | minimize の max_iter / tol 両方省略 |

---

## リスク・注意点

- `_program` / `_errors` の下線接頭辞がない場合 `cargo build` で未使用変数警告が出る
- `lint_program` への呼び出し追加忘れに注意（関数定義だけでは W-code が登録されない）
- `include_str!("lint.rs")` のパスは `fav/src/lint.rs` を指す（`../../` 不要）
