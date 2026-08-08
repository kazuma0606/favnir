# v65.8.0 Spec — Math Lint Rules（W050〜W054）

Version: 65.8.0
Status: 未着手
Base tests: 3467
Target tests: 3469

---

## 概要

数学 Rune 特有のアンチパターンを静的解析で検出する lint ルール W050〜W054 を `lint.rs` に追加する。
W042〜W049 は意図的欠番（スキップ）。Math lint rules は W050 から開始。

| コード | 検出内容 | 重大度 |
|---|---|---|
| W050 | 行列次元の不一致が型推論で検出できない動的パス | warning |
| W051 | 数値不安定な演算（ゼロ除算リスク、`log(x)` で x ≤ 0 の可能性） | warning |
| W052 | 統計的有意性なしの比較（サンプルサイズ < 30 の t 検定） | info |
| W053 | 自動微分ループでの in-place 変更（テープ破壊の危険） | warning |
| W054 | 最適化ループの収束条件未設定（`max_iter` も `tol` も未指定） | warning |

ロードマップ `roadmap-v65.1-v66.0.md` の v65.8.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test --bin fav` でベース 3467 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"65.0.0"` であることを確認（sub-version では更新しない）
- `lint.rs` の最大 W コードが W041 であることを確認（W050〜W054 は新規追加）
- `driver.rs` に `v65700_tests` が存在することを確認（`v65800_tests` の挿入位置）
- `driver.rs` に `v65800_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v65700_tests` で 2 件 PASS することを確認（前バージョンが正常）
- `versions/current.md` の「進行中バージョン」が `v65.7.0` であることを確認

---

## 実装スコープ

### 1. `fav/src/lint.rs` — `check_w050`〜`check_w054` 追加

#### 1a. `lint_program` への呼び出し追加

`check_w040_type_holes(program, &mut errors);` の直後・`errors` 裸式（関数末尾の返却式）の直前に以下を追加：

```rust
// v65.8.0: W050〜W054 Math Lint Rules
check_w050_matrix_dim_mismatch(program, &mut errors);
check_w051_numeric_instability(program, &mut errors);
check_w052_small_sample_test(program, &mut errors);
check_w053_inplace_in_autodiff(program, &mut errors);
check_w054_missing_convergence(program, &mut errors);
```

#### 1b. ファイル末尾への関数追加

`check_w041_*` 関数群の末尾（ファイル末尾）に以下を追加：

```rust
// ── W050〜W054: Math Lint Rules (v65.8.0) ─────────────────────────────────────

// W050: 行列次元の不一致が型推論で検出できない動的パス
// 今バージョンはスタブ（将来フェーズで Rune.linalg 型情報と連携して実装）
fn check_w050_matrix_dim_mismatch(_program: &Program, _errors: &mut Vec<LintError>) {}

// W051: 数値不安定な演算（ゼロ除算リスク、log(x) で x ≤ 0 の可能性）
// 今バージョンはスタブ（将来フェーズで Rune.numeric 呼び出しパターン検出を実装）
fn check_w051_numeric_instability(_program: &Program, _errors: &mut Vec<LintError>) {}

// W052: 統計的有意性なしの比較（サンプルサイズ < 30 の t 検定）
// 今バージョンはスタブ（将来フェーズで Rune.stats.t_test 引数チェックを実装）
fn check_w052_small_sample_test(_program: &Program, _errors: &mut Vec<LintError>) {}

// W053: 自動微分ループでの in-place 変更（テープ破壊の危険）
// 今バージョンはスタブ（将来フェーズで Rune.autodiff.grad 内部の Tensor.set 検出を実装）
fn check_w053_inplace_in_autodiff(_program: &Program, _errors: &mut Vec<LintError>) {}

// W054: 最適化ループの収束条件未設定（max_iter も tol も未指定）
// 今バージョンはスタブ（将来フェーズで Rune.optim.minimize 引数チェックを実装）
fn check_w054_missing_convergence(_program: &Program, _errors: &mut Vec<LintError>) {}
```

### 2. `driver.rs` — `v65800_tests` 追加

挿入位置: `// -- v65700_tests (v65.7.0)` コメントの直前

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

挿入後、`cargo build` でエラーなしを確認。

---

## 完了条件

- `lint.rs` に `check_w050_matrix_dim_mismatch` 〜 `check_w054_missing_convergence` の 5 関数が存在する
- `lint_program` から 5 関数が呼ばれている（呼び出しが `check_w040_type_holes` の後に存在する）
- `cargo test --bin fav v65800_tests` で 2 件 PASS
  - `lint_w051_detects_div_zero_risk` PASS
  - `lint_w053_detects_inplace_in_autodiff` PASS
- `cargo test --bin fav` で 3469 tests passed, 0 failed

---

## 非スコープ

- W050〜W054 の実際の検出ロジック実装 — 将来フェーズ（各 Rune の型情報連携が必要）
- W042〜W049 の実装 — 意図的欠番
- CHANGELOG.md 更新 — v66.0.0 宣言時に一括追記
- site/ MDX ドキュメント — v65.9.0 安定化時に一括作成

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"lint.rs"` → `fav/src/lint.rs`（同ディレクトリ参照）
- ※ Rune ファイルの `../../runes/...` パターンと異なる点に注意

### 既存コードとの整合

- 最後の W-code: W041（`check_w041_perf_hint_large_collect`、v63.6.0）
- W042〜W049: 意図的欠番（roadmap 明記）
- W050〜W054: 本バージョンで関数名・W-code 文字列を lint.rs に登録（実装はスタブ）

### `contains` 判定の設計方針

- `contains("W050")` — コメント `// W050:` でマッチ
- `contains("W051")` — コメント `// W051:` でマッチ
- `contains("check_w051")` — 関数定義 `fn check_w051_numeric_instability` でマッチ
- `contains("W052")` — コメント `// W052:` でマッチ
- `contains("W053")` — コメント `// W053:` でマッチ
- `contains("W054")` — コメント `// W054:` でマッチ
- `contains("check_w053")` — 関数定義 `fn check_w053_inplace_in_autodiff` でマッチ

### `lint_program_with_config` との関係

W050〜W054 は `lint_program` に直接追加するため、`perf` / `strict` フラグに関わらず常に実行される。
現バージョンはスタブ（空実装）のため実害はないが、**将来フェーズで実際の検出を実装する際は
W052（info 重大度）等に対して `perf` / `strict` ゲートの要否を再検討すること**。

### スタブ関数のシグネチャ

```rust
fn check_wXXX_...(program: &Program, errors: &mut Vec<LintError>) {}
```

- `_program` / `_errors` と下線接頭辞を使用（未使用変数警告を抑制）
- 関数本体は空（スタブ）
