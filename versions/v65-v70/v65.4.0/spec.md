# v65.4.0 Spec — Optimization Rune（`Rune.optim`）

Version: 65.4.0
Status: 未着手
Base tests: 3459
Target tests: 3461

---

## 概要

勾配ベースの最適化アルゴリズム群を型安全に提供する Rune `Rune.optim` を実装する。
`Rune.autodiff`（v65.3.0）と組み合わせて ML 訓練ループを型安全に記述する基盤となる。

```favnir
// 利用例（用途のイメージ）
// ※ ロードマップ例は擬似コード。実際の Favnir 構文は技術ノートを参照。
public stage TrainModel: Dataset -> ModelParams = |data| {
    Rune.optim.minimize(
        |params| { cross_entropy(model(params, data), data.labels) },
        ModelParams.random(),
        Rune.optim.adam(lr: 0.001, beta1: 0.9, beta2: 0.999),
        1000,
        1.0e-6
    )
}
```

ロードマップ `roadmap-v65.1-v66.0.md` の v65.4.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3459 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"65.0.0"` であることを確認（sub-version では Cargo.toml は更新しない）
- `runes/optim/` ディレクトリが存在しないことを確認（新規作成対象）
- `driver.rs` に `v65300_tests` が存在することを確認（`v65400_tests` の挿入位置）
- `driver.rs` に `v65400_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v65300_tests` で 2 件 PASS することを確認（前バージョンが正常）

---

## 実装スコープ

### 1. `runes/optim/rune.toml` — Rune メタデータ

```toml
[rune]
name        = "optim"
version     = "0.1.0"
description = "Optimization Rune for Favnir — gradient-based optimizers, schedulers, constrained optimization"
entry       = "optim.fav"
effects     = []

[dependencies]
```

### 2. `runes/optim/optim.fav` — Rune 実装スタブ

以下の全関数定義を含むスタブファイルを作成する。
今バージョンでは**関数シグネチャの確立**が目的。実際の最適化計算は将来フェーズ。

```favnir
// Optimization Rune — Rune.optim
// Gradient-based optimizers, learning rate schedulers, constrained optimization
//
// NOTE: Optimizer, Scheduler, Tensor<Float>, Constraint 等の型は将来フェーズで型システムに登録する。
//       今バージョンは include_str! テストのみ（型チェックエラーは無視する）。

// --- ファーストオーダー最適化 ---

public fn sgd(lr: Float) -> Optimizer {
    Optimizer { kind: "sgd", lr: lr, state: [] }
}

public fn adam(lr: Float, beta1: Float, beta2: Float) -> Optimizer {
    Optimizer { kind: "adam", lr: lr, beta1: beta1, beta2: beta2, state: [] }
}

public fn adamw(lr: Float, beta1: Float, beta2: Float, weight_decay: Float) -> Optimizer {
    Optimizer { kind: "adamw", lr: lr, beta1: beta1, beta2: beta2, weight_decay: weight_decay, state: [] }
}

public fn adagrad(lr: Float) -> Optimizer {
    Optimizer { kind: "adagrad", lr: lr, state: [] }
}

public fn rmsprop(lr: Float) -> Optimizer {
    Optimizer { kind: "rmsprop", lr: lr, state: [] }
}

// --- 二次法 ---

public fn l_bfgs(max_iter: Int) -> Optimizer {
    Optimizer { kind: "l_bfgs", max_iter: max_iter, state: [] }
}

public fn conjugate_gradient(tol: Float) -> Optimizer {
    Optimizer { kind: "conjugate_gradient", tol: tol, state: [] }
}

// --- 汎用最適化 ---

// 損失関数を最小化する汎用エントリポイント（収束判定・早期終了付き）
public fn minimize(loss_fn: Tensor<Float> -> Float, initial: Tensor<Float>, optimizer: Optimizer, max_iter: Int, tol: Float) -> Tensor<Float> {
    initial
}

// --- 学習率スケジューラ ---

public fn step_decay(initial_lr: Float, drop: Float, epochs_drop: Int) -> Scheduler {
    Scheduler { kind: "step_decay", initial_lr: initial_lr }
}

public fn cosine_annealing(t_max: Int, eta_min: Float) -> Scheduler {
    Scheduler { kind: "cosine_annealing", t_max: t_max, eta_min: eta_min }
}

public fn warmup_cosine(warmup_steps: Int, total_steps: Int) -> Scheduler {
    Scheduler { kind: "warmup_cosine", warmup_steps: warmup_steps, total_steps: total_steps }
}

// --- 制約付き最適化 ---

public fn minimize_constrained(loss_fn: Tensor<Float> -> Float, initial: Tensor<Float>, constraints: List<Constraint>, max_iter: Int) -> Tensor<Float> {
    initial
}
```

### 3. `driver.rs` — `v65400_tests` 追加

挿入位置: `// -- v65300_tests (v65.3.0)` コメントの直前

```rust
// -- v65400_tests (v65.4.0) -- Optimization Rune --
#[cfg(test)]
mod v65400_tests {
    #[test]
    fn optim_rune_adam_converges() {
        let content = include_str!("../../runes/optim/optim.fav");
        assert!(!content.is_empty(), "optim.fav should not be empty");
        assert!(content.contains("fn adam("), "optim.fav should define adam");
        assert!(content.contains("fn sgd("), "optim.fav should define sgd");
        assert!(content.contains("fn minimize("), "optim.fav should define minimize");
    }

    #[test]
    fn optim_rune_minimize_quadratic() {
        let content = include_str!("../../runes/optim/optim.fav");
        assert!(content.contains("fn l_bfgs("), "optim.fav should define l_bfgs");
        assert!(
            content.contains("fn conjugate_gradient("),
            "optim.fav should define conjugate_gradient"
        );
        assert!(content.contains("fn step_decay("), "optim.fav should define step_decay scheduler");
    }
}
```

挿入後、`cargo build` でエラーなしを確認。

---

## 完了条件

- `runes/optim/optim.fav` が存在し空でない
- `runes/optim/rune.toml` が存在する
- `optim.fav` に全 12 関数が定義されている:
  - `sgd`, `adam`, `adamw`, `adagrad`, `rmsprop`（ファーストオーダー）
  - `l_bfgs`, `conjugate_gradient`（二次法）
  - `minimize`（汎用）
  - `step_decay`, `cosine_annealing`, `warmup_cosine`（スケジューラ）
  - `minimize_constrained`（制約付き）
  - ※ テストで検証するのはうち 6 関数（`adam`/`sgd`/`minimize`/`l_bfgs`/`conjugate_gradient`/`step_decay`）。残り 6 関数は tasks.md T1 の個別チェックボックスで確認する
- `cargo test --bin fav v65400_tests` で 2 件 PASS
  - `optim_rune_adam_converges` PASS
  - `optim_rune_minimize_quadratic` PASS
- `cargo test -j 8 -- --test-threads=8` で 3461 tests passed, 0 failed

---

## 非スコープ

- 実際の最適化計算実装（数値アルゴリズム）— 将来フェーズ
- `Optimizer` / `Scheduler` / `Tensor<Float>` / `Constraint` の型システム登録 — 将来フェーズ
- `fav check` での型チェック通過 — 今バージョンは `include_str!` テストのみ（型未定義エラーは無視する）
- CHANGELOG.md 更新 — v66.0.0 宣言時に一括追記
- site/ MDX ドキュメント作成 — v65.9.0 安定化時に一括作成するため今バージョンは省略

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../../runes/optim/optim.fav"` → `favnir/runes/optim/optim.fav`

### 型未定義エラーについて

`optim.fav` を `fav check` した場合、`Optimizer` / `Scheduler` / `Tensor<Float>` / `Constraint` が
型システムに未登録のためエラーになる。これは想定内で今バージョンのスコープ外。
`driver.rs` のテストは `include_str!` で文字列として読み込むだけなので型チェックなしで動作する。

### v65.1.0〜v65.3.0 レビューで判明した正しい構文（必ず守ること）

- `bind x <- expr` は Result/Option を返す式にのみ使用する（スタブでは `bind` 不要）
- 非 Result/Option の中間値はネスト呼び出しまたは `|>` パイプラインで表現する
- `Math.sqrt` を使う（`Float.sqrt` は VM に存在しない）
- `Float.from_int` は VM に存在しない
- `let` は使わない

### スタブ本体のレコードコンストラクタについて

`optim.fav` のスタブは `Optimizer { kind: "adam", ... }` のようなレコードコンストラクタを使っている。
`autodiff.fav`（v65.3.0）のスタブが引数をそのまま返す最小形式だったのと対照的。
どちらも include_str! テストでは問題ないが、将来 `fav check` を通す際は
`Optimizer` / `Scheduler` の型定義が必要になる（型システム登録は将来フェーズ）。
スタブ本体の書き換えも将来フェーズのタスクとして記録する。

### `contains` 判定の設計方針

- `contains("fn adam(")` — `public fn adam(` にマッチ（偽陽性なし）
- `contains("fn l_bfgs(")` — `l` から始まる短い名前だが `fn l_bfgs(` で一意
- `contains("fn conjugate_gradient(")` — 長い名前なので偽陽性リスクなし
- `contains("fn step_decay(")` — スケジューラの代表として `step_decay` を選択

### rune.toml フォーマット

- `entry = "optim.fav"`（`main` ではなく `entry`）
- `effects = []` を明示
- `[dependencies]` セクションを含める
