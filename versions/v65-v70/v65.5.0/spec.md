# v65.5.0 Spec — Numerical Methods Rune（`Rune.numeric`）

Version: 65.5.0
Status: 未着手
Base tests: 3461
Target tests: 3463

---

## 概要

数値積分・ODE ソルバー・補間・フーリエ変換・根探索などの数値解析ツール群を提供する Rune `Rune.numeric` を実装する。
信号処理・科学計算・センサーデータパイプラインの基盤となる。

```favnir
// 利用例（用途のイメージ）
// ※ ロードマップ例は擬似コード。実際の Favnir 構文は技術ノートを参照。
public stage IntegrateSignal: List<Float> -> Float = |samples| {
    Rune.numeric.integrate(samples, Simpson, 0.01)
}

public stage FrequencyAnalysis: List<Float> -> Spectrum = |signal| {
    Rune.numeric.fft(signal)
}
```

ロードマップ `roadmap-v65.1-v66.0.md` の v65.5.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3461 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"65.0.0"` であることを確認（sub-version では Cargo.toml は更新しない）
- `runes/numeric/` ディレクトリが存在しないことを確認（新規作成対象）
- `driver.rs` に `v65400_tests` が存在することを確認（`v65500_tests` の挿入位置）
- `driver.rs` に `v65500_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v65400_tests` で 2 件 PASS することを確認（前バージョンが正常）
- `versions/current.md` の「進行中バージョン」が `v65.4.0` であることを確認

---

## 実装スコープ

### 1. `runes/numeric/rune.toml` — Rune メタデータ

```toml
[rune]
name        = "numeric"
version     = "0.1.0"
description = "Numerical methods Rune for Favnir — integration, ODE solvers, interpolation, FFT, root finding"
entry       = "numeric.fav"
effects     = []

[dependencies]
```

### 2. `runes/numeric/numeric.fav` — Rune 実装スタブ

以下の全関数定義を含むスタブファイルを作成する。
今バージョンでは**関数シグネチャの確立**が目的。実際の数値計算は将来フェーズ。

```favnir
// Numeric Rune — Rune.numeric
// Numerical integration, ODE solvers, interpolation, FFT, root finding
//
// NOTE: IntegMethod, OdeMethod, Spectrum 等の型は将来フェーズで型システムに登録する。
//       今バージョンは include_str! テストのみ（型チェックエラーは無視する）。

// --- 数値積分 ---

// 汎用数値積分（method パラメータで手法を切り替え）
public fn integrate(samples: List<Float>, method: IntegMethod, dx: Float) -> Float {
    0.0
}

public fn trapezoid(xs: List<Float>, ys: List<Float>) -> Float {
    0.0
}

public fn simpson(xs: List<Float>, ys: List<Float>) -> Float {
    0.0
}

public fn gauss_quadrature(f: Float -> Float, a: Float, b: Float, n: Int) -> Float {
    0.0
}

// --- ODE ソルバー ---

// 汎用 ODE ソルバー（method パラメータで手法を切り替え）
public fn ode_solve(f: Float -> Float, y0: Float, t_end: Float, method: OdeMethod, dt: Float) -> List<Float> {
    []
}

public fn euler(f: Float -> Float, y0: Float, dt: Float, steps: Int) -> List<Float> {
    []
}

public fn runge_kutta4(f: Float -> Float, y0: Float, dt: Float, steps: Int) -> List<Float> {
    []
}

public fn dormand_prince(f: Float -> Float, y0: Float, t_end: Float, tol: Float) -> List<Float> {
    []
}

// --- 補間 ---

public fn linear_interp(xs: List<Float>, ys: List<Float>, x: Float) -> Float {
    0.0
}

public fn cubic_spline(xs: List<Float>, ys: List<Float>, x: Float) -> Float {
    0.0
}

public fn polynomial_interp(xs: List<Float>, ys: List<Float>, x: Float) -> Float {
    0.0
}

// --- フーリエ変換 ---

public fn fft(signal: List<Float>) -> Spectrum {
    Spectrum { frequencies: [], amplitudes: [], phase: [] }
}

public fn ifft(spectrum: Spectrum) -> List<Float> {
    []
}

public fn power_spectrum(signal: List<Float>) -> List<Float> {
    []
}

public fn spectrogram(signal: List<Float>, window_size: Int, hop: Int) -> List<List<Float>> {
    []
}

// --- 根探索 ---

public fn bisection(f: Float -> Float, a: Float, b: Float, tol: Float) -> Float {
    0.0
}

public fn newton_raphson(f: Float -> Float, df: Float -> Float, x0: Float, tol: Float) -> Float {
    0.0
}

public fn brent(f: Float -> Float, a: Float, b: Float, tol: Float) -> Float {
    0.0
}

// --- 線形方程式系 ---

public fn conjugate_gradient_solver(a: List<List<Float>>, b: List<Float>, tol: Float) -> List<Float> {
    []
}
```

### 3. `driver.rs` — `v65500_tests` 追加

挿入位置: `// -- v65400_tests (v65.4.0)` コメントの直前

```rust
// -- v65500_tests (v65.5.0) -- Numerical Methods Rune --
#[cfg(test)]
mod v65500_tests {
    #[test]
    fn numeric_rune_integrate() {
        let content = include_str!("../../runes/numeric/numeric.fav");
        assert!(!content.is_empty(), "numeric.fav should not be empty");
        assert!(content.contains("fn integrate("), "numeric.fav should define integrate");
        assert!(content.contains("fn fft("), "numeric.fav should define fft");
        assert!(content.contains("fn ifft("), "numeric.fav should define ifft");
    }

    #[test]
    fn numeric_rune_fft() {
        let content = include_str!("../../runes/numeric/numeric.fav");
        assert!(content.contains("fn ode_solve("), "numeric.fav should define ode_solve");
        assert!(content.contains("fn bisection("), "numeric.fav should define bisection");
        assert!(
            content.contains("fn newton_raphson("),
            "numeric.fav should define newton_raphson"
        );
    }
}
```

挿入後、`cargo build` でエラーなしを確認。

---

## 完了条件

- `runes/numeric/numeric.fav` が存在し空でない
- `runes/numeric/rune.toml` が存在する
- `numeric.fav` に全 19 関数が定義されている:
  - `integrate`, `trapezoid`, `simpson`, `gauss_quadrature`（数値積分）
  - `ode_solve`, `euler`, `runge_kutta4`, `dormand_prince`（ODE ソルバー）
  - `linear_interp`, `cubic_spline`, `polynomial_interp`（補間）
  - `fft`, `ifft`, `power_spectrum`, `spectrogram`（フーリエ）
  - `bisection`, `newton_raphson`, `brent`（根探索）
  - `conjugate_gradient_solver`（線形方程式系）
  - ※ テストで検証するのはうち 6 関数（`integrate`/`fft`/`ifft`/`ode_solve`/`bisection`/`newton_raphson`）。残り 13 関数は tasks.md T1 チェックボックスで確認する
  - ※ `integrate` と `ode_solve` はロードマップの汎用エントリポイント（`Rune.numeric.integrate(samples, Simpson, 0.01)` 等）として追加。ロードマップの実装内容列挙（trapezoid/simpson 等）に加えた形
- `cargo test --bin fav v65500_tests` で 2 件 PASS
  - `numeric_rune_integrate` PASS
  - `numeric_rune_fft` PASS
- `cargo test -j 8 -- --test-threads=8` で 3463 tests passed, 0 failed

---

## 非スコープ

- 実際の数値計算実装 — 将来フェーズ
- `IntegMethod` / `OdeMethod` / `Spectrum` の型システム登録 — 将来フェーズ
- `fav check` での型チェック通過 — 今バージョンは `include_str!` テストのみ（型未定義エラーは無視する）
- CHANGELOG.md 更新 — v66.0.0 宣言時に一括追記
- site/ MDX ドキュメント作成 — v65.9.0 安定化時に一括作成するため今バージョンは省略

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../../runes/numeric/numeric.fav"` → `favnir/runes/numeric/numeric.fav`

### `contains` 判定の設計方針

- `contains("fn integrate(")` — `public fn integrate(` にマッチ。`fn integrate` を含む他関数名はない（偽陽性なし）
- `contains("fn fft(")` — `public fn fft(` にマッチ。`"fn ifft("` の中で `"fn fft("` を探すと `fn ` の直後が `i` であり `f` でないため部分文字列にならず、偽陽性なし
- `contains("fn ifft(")` — `public fn ifft(` にマッチ。`fn fft(` とは独立（偽陰性なし）
- `contains("fn ode_solve(")` — 長い名前で一意
- `contains("fn bisection(")` — 一意
- `contains("fn newton_raphson(")` — 長い名前で一意

### 未定義型について

`Spectrum { frequencies: [], amplitudes: [], phase: [] }` は `Spectrum` が型未登録のためエラーになるが、
`include_str!` テストは文字列読み込みのみなので影響なし。
将来フェーズで `Spectrum` 型定義を型システムに登録する際にスタブ本体も整備する。

### v65.1.0〜v65.4.0 レビューで確立した構文ルール

- `bind x <- expr` は Result/Option を返す式にのみ使用する（スタブでは `bind` 不要）
- `let` は使わない
- `Math.sqrt` を使う（`Float.sqrt` は VM に存在しない）
- `Float.from_int` は VM に存在しない

### rune.toml フォーマット

- `entry = "numeric.fav"`（`main` ではなく `entry`）
- `effects = []` を明示
- `[dependencies]` セクションを含める
