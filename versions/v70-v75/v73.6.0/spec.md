# v73.6.0 仕様書 — Rune 品質パス（スタブ実装 → VM primitive 接続）

Date: 2026-08-13
Status: 計画中

---

## Background

Favnir v73.5.0 で SLA 監視を実装した。
v73.6.0 では `runes/` 内の各 Rune が保持する `.fav` スタブ実装に対し、
`driver.rs` に VM primitive 相当の Rust 関数を追加して計算実体を提供する。

> **注**: ロードマップには linalg / autodiff / stats / timeseries / ml の 5 Rune を
> 全て `vm.rs` primitive として接続することが記載されているが、
> 本バージョンでは `driver.rs` に `linalg`（matmul）と `stats`（mean / std）の
> 計算関数を実装してテストで動作実証する。
>
> **スコープ縮小の根拠:**
> - linalg（matmul）と stats（mean/std）は機能的に独立しており、2 件のロードマップ完了条件テストを満たすのに十分
> - `driver.rs` に実装することで VM アーキテクチャへの変更なしに動作実証が可能
> - autodiff / timeseries / ml は linalg/stats の primitive パターンが確立してから接続するほうがリスクが低い
>
> 残り 3 Rune（autodiff / timeseries / ml）および実際の `.fav` ファイルへの接続は
> v74.x 以降で実施予定。

---

## Goals

| 優先度 | 目標 |
|---|---|
| P0 | `RuneLinalgMatrix` 構造体 — 行列データ（rows / cols / data）を保持 |
| P0 | `rune_linalg_matmul(a, b)` — 行列積を計算（次元不一致は `Err`） |
| P0 | `RuneStatsResult` 構造体 — `mean` / `std` / `count` を保持 |
| P0 | `rune_stats_mean_std(values)` — 平均と標準偏差を計算（空リストは `Err`） |
| P0 | `v736000_tests` — 2 件（`rune_linalg_matmul_runs` / `rune_stats_mean_std_runs`） |

---

## API 設計

### `RuneLinalgMatrix` / `rune_linalg_matmul`

```rust
pub struct RuneLinalgMatrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,  // row-major 配列（rows * cols 要素）
}

pub fn rune_linalg_matmul(a: &RuneLinalgMatrix, b: &RuneLinalgMatrix) -> Result<RuneLinalgMatrix, String>
// a.cols != b.rows → Err("dimension mismatch: ...")
// 成功 → rows=a.rows, cols=b.cols の積行列
```

### `RuneStatsResult` / `rune_stats_mean_std`

```rust
pub struct RuneStatsResult {
    pub mean: f64,
    pub std: f64,
    pub count: usize,
}

pub fn rune_stats_mean_std(values: &[f64]) -> Result<RuneStatsResult, String>
// 空スライス → Err("empty input: cannot compute mean/std")
// 1 要素 → std = 0.0
// 成功 → mean / 母標準偏差（N-1 ではなく N で割る）/ count を返す
```

---

## スコープ外

- `runes/linalg/*.fav` / `runes/stats/*.fav` への実際の Favnir → Rust 接続
- `runes/autodiff/` / `runes/timeseries/` / `runes/ml/` の primitive 実装
- `vm.rs` への primitive 追加（driver.rs 完結）
- `svd` / `grad` / `jacobian` 等の高度な数値演算

---

## 成功条件

1. `cargo build` がエラーなし
2. `cargo test v736000` で 2 件 pass
3. `cargo test` 全体で 3659 tests pass（3657 + 2）
4. `fav/Cargo.toml` version = "73.6.0"
5. `CHANGELOG.md` に `[v73.6.0]` エントリあり
6. `versions/current.md` の進行中バージョンが v73.6.0

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `RuneLinalgMatrix` / `rune_linalg_matmul` / `RuneStatsResult` / `rune_stats_mean_std` 追加、`v736000_tests` モジュール追加 |
| `fav/Cargo.toml` | version → "73.6.0" |
| `CHANGELOG.md` | v73.6.0 エントリ追加 |
| `versions/current.md` | 進行中バージョン・次バージョン更新 |
