# v84.6.0 タスクリスト

Status: COMPLETE

---

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,919 tests, 0 failures を確認する（前提: v84.5.0 完了済み）
- [x] `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "84.0.0"` であることを確認する
  （v84.x マイナーバージョンは Cargo.toml を更新しない慣例。v85.0.0 宣言時に一括更新する）
- [x] `fav/src/driver.rs` に `mod v84500_tests` が存在することを確認する（v84.5.0 完了済みの証拠）
- [x] `site/content/docs/v4/` ディレクトリが存在しないことを確認する（新規作成の前提）

## T1: `site/content/docs/v4/` ディレクトリ作成と MDX ファイル追加

- [x] `site/content/docs/v4/` ディレクトリを作成する
- [x] `test-driven-data.mdx` を作成する
  - `TestSuite` / `StageTestCase` / `GoldenDataset` / `SchemaSnapshot` / `compare_golden` / `compare_schema_snapshots` の説明を含める
- [x] `data-quality.mdx` を作成する
  - `QualityRule` / `QualityCheck` / `run_quality_check` / `QualityGate` / `evaluate_quality_gate` / `AnomalyDetector` / `detect_anomaly` の説明を含める
- [x] `pipeline-contracts.mdx` を作成する
  - `IoContract` / `SlaContract` / `ContractRegistry` / `ContractRegistryEntry` / `registered_at` の説明を含める
- [x] `observability.mdx` を作成する
  - `PipelineMetrics` / `StageMetrics` / `AlertRule` / `evaluate_alert_rules` / `SloTarget` / `SloMeasurement` / `compute_slo_status` / `HealthDashboard` / `format_health_dashboard` の説明を含める
- [x] `migration-v3-v4.mdx` を作成する
  - v3（v80.0）→ v4（v85.0）の機能比較表と移行手順 4 ステップを含める

## T2: `fav/src/driver.rs` に `v84600_tests` を追加

- [x] `mod v84500_tests { ... }` の直後に `#[cfg(test)] mod v84600_tests { ... }` を追加する
  - パス起点は `fav/`（`cargo test` の CWD）
- [x] `docs_v4_test_driven_data_exists` テストを実装する
  - `../site/content/docs/v4/test-driven-data.mdx` が存在すること（メッセージ付き）
- [x] `docs_v4_migration_guide_exists` テストを実装する
  - `../site/content/docs/v4/migration-v3-v4.mdx` が存在すること（メッセージ付き）

## T3: テスト通過確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,921 tests, 0 failures（+2）であることを確認する

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v84.6.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
