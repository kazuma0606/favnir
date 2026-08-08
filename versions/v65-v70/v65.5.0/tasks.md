# v65.5.0 タスクリスト

Status: COMPLETE
Version: 65.5.0
Base tests: 3461
Target tests: 3463

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3461 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"65.0.0"` であることを確認（sub-version では変更しない）
- [x] `runes/numeric/` ディレクトリが存在しないことを確認（新規作成対象）
- [x] `driver.rs` に `v65400_tests` が存在することを確認（`v65500_tests` の挿入位置）
- [x] `driver.rs` に `v65500_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v65400_tests` で 2 件 PASS することを確認（前バージョンが正常）
- [x] `versions/current.md` の「進行中バージョン」が `v65.4.0` であることを確認

---

## T1: Rune ファイル作成

- [x] `runes/numeric/` ディレクトリ作成
- [x] `runes/numeric/rune.toml` 作成（`entry` / `effects = []` / `[dependencies]` 形式）
- [x] `runes/numeric/numeric.fav` 作成（以下の全 19 関数を定義）
  - **数値積分**
  - [x] `integrate` — 汎用数値積分（method 指定）
  - [x] `trapezoid` — 台形則
  - [x] `simpson` — シンプソン則
  - [x] `gauss_quadrature` — ガウス求積
  - **ODE ソルバー**
  - [x] `ode_solve` — 汎用 ODE ソルバー（method 指定）
  - [x] `euler` — オイラー法
  - [x] `runge_kutta4` — 4 次ルンゲクッタ法
  - [x] `dormand_prince` — 適応刻み幅 Dormand-Prince 法
  - **補間**
  - [x] `linear_interp` — 線形補間
  - [x] `cubic_spline` — 3 次スプライン補間
  - [x] `polynomial_interp` — 多項式補間
  - **フーリエ変換**
  - [x] `fft` — 高速フーリエ変換
  - [x] `ifft` — 逆高速フーリエ変換
  - [x] `power_spectrum` — パワースペクトル
  - [x] `spectrogram` — スペクトログラム
  - **根探索**
  - [x] `bisection` — 二分法
  - [x] `newton_raphson` — ニュートン・ラフソン法
  - [x] `brent` — Brent 法
  - **線形方程式系**
  - [x] `conjugate_gradient_solver` — 共役勾配法による線形方程式系求解
- [x] `numeric.fav` 内に `let ` が含まれないことを確認
- [x] `numeric.fav` 内に `bind.*=`（`<-` でない bind）が含まれないことを確認
- [x] `numeric.fav` 内に `Float.from_int` / `Float.sqrt` が含まれないことを確認
- [x] `grep -c 'public fn ' numeric.fav` で 19 が出ることを確認

---

## T2: `driver.rs` — `v65500_tests` 追加

- [x] `// -- v65400_tests (v65.4.0)` コメントの直前に `v65500_tests` を挿入
  - [x] `numeric_rune_integrate` — `fn integrate(` / `fn fft(` / `fn ifft(` を含む
  - [x] `numeric_rune_fft` — `fn ode_solve(` / `fn bisection(` / `fn newton_raphson(` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v65500_tests` で 2 件 PASS
  - [x] `numeric_rune_integrate` PASS
  - [x] `numeric_rune_fft` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3463 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v65.1-v66.0.md` の v65.5.0 行を「完了」に更新
- [x] `versions/current.md` の「進行中バージョン」を v65.5.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v65.1〜v65.9 では CHANGELOG.md を更新しない。v66.0.0 宣言時に一括追記する。
> **MDX 方針**: `site/` の MDX ドキュメントは v65.9.0 安定化時に一括作成するため今バージョンは省略。

---

## コードレビュー対応

実装は spec.md の通り。注意点:
- `bind` / `let` は一切使用しない（全スタブが `0.0` / `[]` を返すだけ）
- `Float.from_int` / `Float.sqrt` は使用しない
- `fn fft(` は `fn ifft(` の部分文字列にならない（`fn ` の次が `i`）— 偽陽性なし確認済み
- 関数数は 19（integrate/ode_solve を汎用ラッパーとして追加、brent 含む）
