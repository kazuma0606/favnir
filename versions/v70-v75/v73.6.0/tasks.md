# v73.6.0 タスクリスト — Rune 品質パス（スタブ実装 → VM primitive 接続）

Date: 2026-08-13
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `73.5.0` であることを確認
- [x] `cargo test` が 3657 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v735000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v736000_tests` が未存在であることを確認

---

## T1: `RuneLinalgMatrix` 構造体 + `rune_linalg_matmul` 追加

- [x] `RuneLinalgMatrix { rows: usize, cols: usize, data: Vec<f64> }` を `driver.rs` に追加した
- [x] `pub struct` であることを確認
- [x] `#[derive(Debug)]` を付与した（`.expect()` が Debug を要求するため）
- [x] `pub fn rune_linalg_matmul(a: &RuneLinalgMatrix, b: &RuneLinalgMatrix) -> Result<RuneLinalgMatrix, String>` を実装した
  - `a.cols != b.rows` → `Err("dimension mismatch: a.cols=X != b.rows=Y")`
  - 成功 → `rows=a.rows, cols=b.cols` の積行列（row-major）
- [x] `cargo build` でエラーがないことを確認

---

## T2: `RuneStatsResult` 構造体 + `rune_stats_mean_std` 追加

- [x] `RuneStatsResult { mean: f64, std: f64, count: usize }` を `driver.rs` に追加した
- [x] `pub struct` であることを確認
- [x] `#[derive(Debug)]` を付与した
- [x] `pub fn rune_stats_mean_std(values: &[f64]) -> Result<RuneStatsResult, String>` を実装した
  - 空スライス → `Err("empty input: cannot compute mean/std")`
  - 1 要素 → `std = 0.0`
  - 成功 → `mean` / 母標準偏差（N で割る）/ `count`
- [x] `cargo build` でエラーがないことを確認

---

## T3: `v736000_tests` モジュール追加

- [x] `v735000_tests` の直後に `v736000_tests` モジュールを追加した
- [x] `use super::{RuneLinalgMatrix, rune_linalg_matmul, rune_stats_mean_std}` を追加した
- [x] `rune_linalg_matmul_runs` テストを実装した
  - 2×2 行列積 → 各要素を `< 1e-9` 精度で assert
  - 次元不一致 → `Err` で `"dimension mismatch"` を含むことを assert
- [x] `rune_stats_mean_std_runs` テストを実装した
  - `[1.0..5.0]` → `mean=3.0` / `std=sqrt(2.0)` を `< 1e-9` 精度で assert
  - 1 要素 → `std=0.0` を assert
  - 空リスト → `Err` で `"empty input"` を含むことを assert
- [x] `cargo test v736000` で 2 件 pass することを確認

---

## T4: バージョン更新

- [x] `fav/Cargo.toml` の `version = "73.5.0"` → `version = "73.6.0"` に変更した
- [x] `driver.rs` 内の `"73.5.0"` を `"73.6.0"` に replace_all した（バージョン検証テスト文字列を含む）
- [x] 残存 `73.5.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `cargo build` でエラーがないことを確認
- [x] `fav/Cargo.lock` が `version = "73.6.0"` を含むことを確認

---

## T5: 部分テスト確認

- [x] T4 のバージョン更新後も `cargo test v736000` で引き続き 2 件 pass することを確認

---

## T6: 全体テスト確認

- [x] `cargo test` 全体で 3659 tests pass（0 failures）であることを確認

---

## T7: `CHANGELOG.md` 更新

- [x] `## [v73.6.0]` エントリを先頭に追加した
  - Added: `RuneLinalgMatrix` / `rune_linalg_matmul` / `RuneStatsResult` / `rune_stats_mean_std`
  - Tests: 2 件、合計テスト数 3659（+2）

---

## T8: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-13 (v73.6.0)` に更新した
- [x] 「進行中バージョン」を `v73.6.0` に更新した
- [x] 「次に切る版」を `v73.7.0` に更新した

---

## T9: 最終確認（T7・T8 完了後）

- [x] `cargo test v736000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3659 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `73.6.0` であることを確認
- [x] `RuneLinalgMatrix` / `rune_linalg_matmul` / `RuneStatsResult` / `rune_stats_mean_std` が pub で存在することを確認
- [x] `CHANGELOG.md` に `[v73.6.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v73.6.0` であることを確認

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [HIGH] | `rune_linalg_matmul` で `data.len()` 未検証 → パニックリスク | 関数冒頭に `a.data.len() != a.rows * a.cols` / `b.data.len() != b.rows * b.cols` チェックを追加、Err を返す |
| [MED] | `rune_stats_mean_std` が母標準偏差であること未明示 | `///` doc コメントで「N で割る母標準偏差」を明記 |
| [LOW] | 非正方行列テスト欠如 | 2×3 × 3×2 = 2×2 の手計算検証テストを追加 |
| [LOW] | `data.len()` エラーのテスト欠如 | `data=vec![1.0]`（rows=2,cols=2）で `Err("does not match")` を assert するテストを追加 |
| [LOW] | 負の値テスト欠如 | `[-1.0, 0.0, 1.0]` → `mean=0.0`, `std=sqrt(2/3)` のテストを追加 |
| [LOW] | i-k-j ループ最適化 | 現スコープでは許容範囲 — 対応不要 |

実装時に発覚したビルドエラー:
- `RuneLinalgMatrix` / `RuneStatsResult` に `#[derive(Debug)]` 未付与 → `.expect()` が Debug を要求 → 追加

---

## スコープ外（明示的除外）

- `runes/linalg/*.fav` / `runes/stats/*.fav` への Favnir → Rust 接続
- `runes/autodiff/` / `runes/timeseries/` / `runes/ml/` の primitive 実装
- `vm.rs` への primitive 追加
- `svd` / `grad` / `jacobian` 等の高度な数値演算
