# v65.3.0 Spec — Autodiff Rune（`Rune.autodiff`）

Version: 65.3.0
Status: 未着手
Base tests: 3457
Target tests: 3459

---

## 概要

reverse-mode 自動微分（AD）を Favnir の型システムと統合する Rune `Rune.autodiff` を実装する。
勾配計算・バックプロパゲーションを型安全に表現し、`Rune.optim`（v65.4.0）の基盤となる。

```favnir
// 利用例（用途のイメージ）
// ※ ロードマップ例は擬似コード。実際の Favnir 構文は技術ノートを参照。
public stage GradientStep: Tensor<Float> -> Tensor<Float> = |params| {
    Rune.autodiff.grad(|p| { model_loss(p) }, params)
}
```

ロードマップ `roadmap-v65.1-v66.0.md` の v65.3.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3457 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"65.0.0"` であることを確認（sub-version では Cargo.toml は更新しない）
- `runes/autodiff/` ディレクトリが存在しないことを確認（新規作成対象）
- `driver.rs` に `v65200_tests` が存在することを確認（`v65300_tests` の挿入位置）
- `driver.rs` に `v65300_tests` が存在しないことを確認（新規追加）

---

## 実装スコープ

### 1. `runes/autodiff/rune.toml` — Rune メタデータ

既存 `runes/stat/rune.toml` の形式（`entry` / `effects = []` / `[dependencies]`）に合わせる。

```toml
[rune]
name        = "autodiff"
version     = "0.1.0"
description = "Automatic differentiation Rune for Favnir — reverse-mode AD, grad, jacobian, hessian"
entry       = "autodiff.fav"
effects     = []

[dependencies]
```

### 2. `runes/autodiff/autodiff.fav` — Rune 実装スタブ

以下の全関数・型定義を含むスタブファイルを作成する。
今バージョンでは**関数シグネチャの確立**が目的。実際の計算グラフ構築は将来フェーズ。

```favnir
// Autodiff Rune — Rune.autodiff
// Reverse-mode automatic differentiation for Favnir
//
// NOTE: Tensor<Float>, Tape 等の型は将来フェーズで型システムに登録する。
//       今バージョンは include_str! テストのみ（型チェックエラーは無視する）。

// --- 計算テープ（reverse-mode AD の中核） ---

public fn tape() -> Tape {
    Tape { nodes: [], ops: [] }
}

// --- 微分演算 ---

// スカラー値関数の勾配 — f: Tensor<Float> -> Float の ∂f/∂x（戻り型は入力と同型）
public fn grad(f: Tensor<Float> -> Float, x: Tensor<Float>) -> Tensor<Float> {
    x
}

// ベクトル値関数のヤコビアン行列
public fn jacobian(f: Tensor<Float> -> Tensor<Float>, x: Tensor<Float>) -> Tensor<Float> {
    x
}

// 二階微分行列（ヘッシアン）
public fn hessian(f: Tensor<Float> -> Float, x: Tensor<Float>) -> Tensor<Float> {
    x
}

// --- 勾配追跡制御 ---

// 勾配追跡を無効化するコンテキスト（推論・評価時に使用）
public fn no_grad(f: () -> Tensor<Float>) -> Tensor<Float> {
    f()
}

// --- プリミティブ（チェーンルール対応） ---

public fn prim_exp(x: Tensor<Float>) -> Tensor<Float> {
    x
}

public fn prim_log(x: Tensor<Float>) -> Tensor<Float> {
    x
}

public fn prim_sin(x: Tensor<Float>) -> Tensor<Float> {
    x
}

public fn prim_cos(x: Tensor<Float>) -> Tensor<Float> {
    x
}

public fn prim_tanh(x: Tensor<Float>) -> Tensor<Float> {
    x
}

public fn relu(x: Tensor<Float>) -> Tensor<Float> {
    x
}
```

### 3. `driver.rs` — `v65300_tests` 追加

挿入位置: `// -- v65200_tests (v65.2.0)` コメントの直前

```rust
// -- v65300_tests (v65.3.0) -- Autodiff Rune --
#[cfg(test)]
mod v65300_tests {
    #[test]
    fn autodiff_rune_grad_scalar() {
        let content = include_str!("../../runes/autodiff/autodiff.fav");
        assert!(!content.is_empty(), "autodiff.fav should not be empty");
        assert!(content.contains("fn grad("), "autodiff.fav should define grad");
        assert!(content.contains("fn jacobian("), "autodiff.fav should define jacobian");
        assert!(content.contains("fn hessian("), "autodiff.fav should define hessian");
    }

    #[test]
    fn autodiff_rune_chain_rule() {
        let content = include_str!("../../runes/autodiff/autodiff.fav");
        assert!(content.contains("Tape"), "autodiff.fav should reference Tape");
        assert!(content.contains("fn no_grad("), "autodiff.fav should define no_grad");
        assert!(content.contains("fn relu("), "autodiff.fav should define relu primitive");
    }
}
```

挿入後、`cargo build` でエラーなしを確認。

---

## 完了条件

- `runes/autodiff/autodiff.fav` が存在し空でない
- `runes/autodiff/rune.toml` が存在する
- `autodiff.fav` に `grad`, `jacobian`, `hessian`, `no_grad`, `relu`, `Tape` が定義されている
- `cargo test --bin fav v65300_tests` で 2 件 PASS
  - `autodiff_rune_grad_scalar` PASS
  - `autodiff_rune_chain_rule` PASS
- `cargo test -j 8 -- --test-threads=8` で 3459 tests passed, 0 failed

---

## 非スコープ

- 実際の計算グラフ構築・逆伝播アルゴリズム実装 — 将来フェーズ
- `+`, `-`, `*`, `/` 演算子のチェーンルール登録 — Favnir の組み込み演算子と重複するため関数スタブの対象外。将来フェーズで演算子オーバーロード機構と統合する
- `Tensor<Float>` / `Tape` / `GradFn` の型システム登録 — 将来フェーズ
- `fav check` での型チェック通過 — 今バージョンは `include_str!` テストのみ（型未定義エラーは無視する）
- WASM 対応 — 将来フェーズ
- CHANGELOG.md 更新 — v66.0.0 宣言時に一括追記
- site/ MDX ドキュメント作成 — v65.9.0 安定化時に一括作成するため今バージョンは省略

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../../runes/autodiff/autodiff.fav"` → `favnir/runes/autodiff/autodiff.fav`

### 型未定義エラーについて

`autodiff.fav` を `fav check` した場合、`Tensor<Float>` / `Tape` 等が型システムに未登録のためエラーになる。
これは想定内で今バージョンのスコープ外。
`driver.rs` のテストは `include_str!` で文字列として読み込むだけなので型チェックなしで動作する。

### v65.1.0 / v65.2.0 レビューで判明した正しい構文（必ず守ること）

- `bind x <- expr` は Result/Option を返す式にのみ使用する
- 非 Result/Option の中間値はネスト呼び出しまたは `|>` パイプラインで表現する
- `Math.sqrt` を使う（`Float.sqrt` は VM に存在しない）
- `Float.from_int` は VM に存在しない（`Int / Int` の除算は Float を自動昇格）
- `List.zip_with(f, xs, ys)` — クロージャが第1引数

### 既知問題: `runes/linalg/linalg.fav` の `bind <- Float` パターン

`runes/linalg/linalg.fav` の `cosine_similarity` 関数では `bind n_a <- norm(a)` のように
`Float`（非 Result/Option）に対して `bind x <- expr` を使っている。
これは v65.1.0 のコードレビューで確立した構文ルール（「Result/Option のみ」）に反するが、
v65.1.0 の include_str! テストは実行を伴わないため実害はなく、既存実装として放置されている。
本バージョンの `autodiff.fav` はスタブのみで `bind` を一切使わないため直接の影響はない。
将来フェーズで linalg.fav を実行するようになった際に修正対象となる既知問題として記録する。

### `Tape` の `contains` 判定について

テストの `content.contains("Tape")` は `-> Tape` または `Tape {` のいずれかにマッチする。
`tape()` 関数の戻り型 `-> Tape` と `Tape { nodes: [], ops: [] }` の両方で満たされる。

### rune.toml フォーマット

- `entry = "autodiff.fav"`（`main` ではなく `entry`）
- `effects = []` を明示
- `[dependencies]` セクションを含める
