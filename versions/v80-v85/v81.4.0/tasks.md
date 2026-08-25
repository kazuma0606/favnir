# Tasks: v81.4.0 — 品質スコアリング（`QualityScore` / `QualityDimension`）

> COMPLETE — 2026-08-19
> 3851 tests, 0 failures（+2 from 3849）

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3849 tests, 0 failures を確認する
- [x] `Cargo.toml` の `version` フィールドが `81.0.0` であることを確認する
  （`grep '^version = ' fav/Cargo.toml` または `cargo_toml_version_is_81_0_0` テストで確認）
  （v81.x マイナーバージョンは Cargo.toml を更新しない慣例）
- [x] `fav/src/driver.rs` に `mod v81300_tests` が存在することを確認する（v81.3.0 完了済みの証拠）
- [x] `fav/src/test_framework.rs` に `format_drift_report` が定義済みであることを確認する

## T1: `fav/src/test_framework.rs` に追記

- [x] `QualityDimension` enum（`#[derive(Debug, Clone, PartialEq)]`）を追加する
  - バリアント: `Completeness`, `Consistency`, `Timeliness`, `Accuracy`, `Validity`
- [x] `DimensionScore` 構造体（`#[derive(Debug, Clone)]`）を追加する
  - フィールド: `dimension: QualityDimension`, `score: f64`, `weight: f64`
- [x] `QualityScore` 構造体（`#[derive(Debug, Clone)]`）を追加する
  - フィールド: `dimensions: Vec<DimensionScore>`, `overall: f64`
- [x] `compute_quality_score(dimensions: &[DimensionScore]) -> QualityScore` を実装する
  - `total_weight = Σ weight`
  - `overall = Σ(score * weight) / total_weight`（`total_weight == 0.0` のとき `0.0`）
  - `dimensions.to_vec()` で clone して格納する
- [x] `format_quality_score(score: &QualityScore) -> String` を実装する
  - 形式: `"overall={:.3} grade={grade} dimensions={count}"`
  - 内部で `quality_grade(score)` を呼ぶ
- [x] `quality_grade(score: &QualityScore) -> &'static str` を実装する
  - A: overall >= 0.90 / B: >= 0.80 / C: >= 0.70 / D: >= 0.60 / F: < 0.60

## T2: `fav/src/driver.rs` に `mod v81400_tests` を追加

- [x] `mod v81300_tests { ... }` の直後に `#[cfg(test)] mod v81400_tests { ... }` を追加する
- [x] `use fav_core::test_framework::*;` でインポートする
- [x] `quality_score_weighted_average` テストを実装する
  - `Completeness(score=0.9, weight=2.0)` + `Accuracy(score=0.8, weight=1.0)` で `overall ≈ 0.8667`（誤差 1e-9）
  - `dimensions.len() == 2` を確認する
  - `format_quality_score` の出力に `"overall=0.867"`, `"grade=B"`, `"dimensions=2"` が含まれることを確認する
- [x] `quality_grade_a_when_perfect` テストを実装する
  - `overall = 1.0` → `"A"`
  - `overall = 0.9` → `"A"`（境界値 inclusive）
  - `overall = 0.89` → `"B"`
  - `overall = 0.0` → `"F"`

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3851 tests, 0 failures であることを確認する

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v81.4.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
