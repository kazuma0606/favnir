# Tasks: v81.6.0 — 品質ゲート（`QualityGate` / パイプライン停止条件）

> COMPLETE — 2026-08-19
> 3855 tests, 0 failures（+2 from 3853）

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3853 tests, 0 failures を確認する
- [x] `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "81.0.0"` であることを確認する
  （v81.x マイナーバージョンは Cargo.toml を更新しない慣例。このバージョン完了後も `81.0.0` のまま変更しない）
- [x] `fav/src/driver.rs` に `mod v81500_tests` が存在することを確認する（v81.5.0 完了済みの証拠）
- [x] `fav/src/test_framework.rs` に `QualityScore` / `QualityDimension` / `DimensionScore` が定義済みであることを確認する

## T1: `fav/src/test_framework.rs` に追記

- [x] `GateDecision` enum（`#[derive(Debug, Clone, PartialEq)]`）を追加する
  - バリアント: `Pass`, `Fail(String)`, `Warn(String)`
- [x] `QualityGate` 構造体（`#[derive(Debug, Clone)]`）を追加する
  - フィールド: `min_overall_score: f64`, `required_dimensions: Vec<QualityDimension>`, `min_dimension_score: f64`
- [x] `impl QualityGate` ブロックを追加する
  - `strict()`: `min_overall_score = 0.9`, 全 5 次元, `min_dimension_score = 0.9`
  - `permissive()`: `min_overall_score = 0.6`, `required_dimensions = vec![]`, `min_dimension_score = 0.6`
- [x] `evaluate_quality_gate(gate: &QualityGate, score: &QualityScore) -> GateDecision` を実装する
  - `score.overall < gate.min_overall_score` → `Fail("overall score X.XXX below minimum Y.YYY")`
  - 各 required 次元が見つからない → `Fail("dimension X score not found")`
  - 各 required 次元スコア < `min_dimension_score` → `Fail("dimension X score Y.YYY below minimum Z.ZZZ")`
  - すべて通過 → `Pass`
- [x] `format_gate_decision(decision: &GateDecision) -> String` を実装する
  - `Pass` → `"PASS"` / `Fail(msg)` → `"FAIL: {msg}"` / `Warn(msg)` → `"WARN: {msg}"`

## T2: `fav/src/driver.rs` に `mod v81600_tests` を追加

- [x] `mod v81500_tests { ... }` の直後に `#[cfg(test)] mod v81600_tests { ... }` を追加する
- [x] `use fav_core::test_framework::*;` でインポートする
- [x] `quality_gate_fails_below_threshold` テストを実装する
  - `QualityGate::permissive()` に `overall = 0.5` を渡すと `Fail` を返すことを確認する
  - `format_gate_decision` の出力に `"FAIL"` / `"0.500"` / `"0.600"` が含まれることを確認する
- [x] `quality_gate_passes_above_threshold` テストを実装する
  - `QualityGate::permissive()` に `overall = 0.8` → `Pass` を確認する
  - `format_gate_decision(&GateDecision::Pass)` が `"PASS"` を返すことを確認する
  - `QualityGate::strict()` に全次元 0.95 + `overall = 0.95` → `Pass` を確認する

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3855 tests, 0 failures であることを確認する

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v81.6.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
