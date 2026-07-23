# Tasks: v49.2.0 — パフォーマンス計測 + ボトルネック修正

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3071 passed, 0 failed を確認（ベース確認）
- [x] `benchmarks/v49.2.0.json` が存在しないことを確認（新規作成対象）

## T1 — ベンチマーク結果ファイル作成

- [x] `benchmarks/v49.2.0.json` 新規作成（`favnir/benchmarks/` 直下・フラット命名）
  - [x] `version`: `"49.2.0"`
  - [x] `milestone`: `"Production 2.0"`
  - [x] `test_count`: `3073`
  - [x] `metrics.checker_ms` フィールドを含む
  - [x] `metrics.compiler_ms` フィールドを含む
  - [x] `metrics.total_pipeline_ms` フィールドを含む
  - [x] `"regression": false`（スペース 1 個・テスト assert `"\"regression\": false"` と一致）
  - [x] `notes` フィールドを含む

## T2 — `driver.rs` テスト追加

- [x] `v492000_tests` モジュールを `v491000_tests` の直前に追加（2テスト）
  - [x] `bench_all_result_recorded`: `"checker_ms"` と `"49.2.0"` が含まれることを確認
  - [x] `checker_perf_regression_none`: `"regression": false` が含まれることを確認

## T3 — バージョン更新・完了

- [x] site/ MDX 更新: 不要（ベンチマーク記録のみ・新構文なし）
- [x] `fav/Cargo.toml` version → `"49.2.0"`
- [x] `CHANGELOG.md` に v49.2.0 エントリ追加（`metrics.checker_ms` / `compiler_ms` / `total_pipeline_ms` 記録を明記）
- [x] `cargo test` 3073 passed, 0 failed（3071 + 2 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v49.2.0（3073 tests）に更新、進行中バージョンを `v49.3.0` に更新
- [x] `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.2.0 実績を 3073 に記入
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）

---

> **注記**: `cargo clean` はこのバージョンのスコープ外（v50.0.0 で実施）
> **注記**: checker.rs / compiler.rs への実際のコード変更はなし（ホットパス改善は v49.3.0 以降に持ち越し）
