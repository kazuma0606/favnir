# Roadmap v84.1.0 〜 v85.0.0 — Favnir 4.0 宣言

Date: 2026-08-16
Status: 完了（v85.0.0 宣言 2026-08-22）

マスターロードマップ: [roadmap-v80.1-v85.0.md](roadmap-v80.1-v85.0.md)

---

## 前提

- 直前完了: v84.0.0「Observability 2.0 宣言」（tests = 3,897）
- 本スプリントは Quality-First Era の最終スプリント（Sprint 5）
- 目標: v85.0.0「Favnir 4.0 宣言」（tests = 3,919）
- **着手前確認**: `versions/current.md` の現行マスターロードマップが `roadmap-v80.1-v85.0.md` を指していること、最新安定版が v84.0.0 になっていることを確認してから開始する

### スプリントの性格

v80.1〜v84.9 で積み上げた 4 つの Quality 柱（Test-Driven Data / Data Quality 2.0 / Pipeline Contracts / Observability 2.0）を
統合ショーケース・ドキュメント・OSS 公開強化で完成させ、Favnir 4.0 として宣言する。
B（統合）40% + C（宣言・ドキュメント）60% の構成。

---

## バージョン一覧

> **テスト数注記**: ロードマップ策定時の計画ベースは 3,897 だったが、
> code-reviewer 指摘対応の累積により実際のベースは v84.0.0 完了時点で 3,909 に増加。
> 以下のテスト数は実績値（計画値からの累積乖離 +12 を反映済み）。

| バージョン | 内容 | テスト数（実績） | 状態 |
|---|---|---|---|
| v84.1.0 | E2E ショーケース基盤（`infra/e2e-demo/favnir4-showcase/`） | 3909 + 2 = 3911 | 完了 |
| v84.2.0 | テスト統合ショーケース（`fav test` E2E） | 3911 + 2 = 3913 | 完了 |
| v84.3.0 | 品質統合ショーケース（`fav quality` E2E） | 3913 + 2 = 3915 | 完了 |
| v84.4.0 | 契約統合ショーケース（`fav verify --contract` E2E） | 3915 + 2 = 3917 | 完了 |
| v84.5.0 | 可観測性統合ショーケース（`fav observe` E2E） | 3917 + 2 = 3919 | 完了 |
| v84.6.0 | ドキュメント完全化（`site/content/docs/v4/`） | 3919 + 2 = 3921 | 完了 |
| v84.7.0 | OSS 公開強化・コミュニティ整備 v2 | 3921 + 2 = 3923 | 完了 |
| v84.8.0 | パフォーマンス最終調整 | 3923 + 2 = 3925 | 完了 |
| v84.9.0 | 安定化・コードフリーズ | 3925 + 2 = 3927 | 完了 |
| v85.0.0 | Favnir 4.0 宣言 ★クリーンアップ | 3927 + 4 = 3931 | 完了 |

---

## v84.1.0 — E2E ショーケース基盤

4 スプリントすべての機能を網羅するショーケースの骨格を作成する。
v79.1.0 の `infra/e2e-demo/favnir3-showcase/` と同構造。

```
infra/e2e-demo/favnir4-showcase/
├── pipeline.fav     # 4 Quality 柱を統合したパイプライン
├── fav.toml         # QualityGate / IoContract / AlertRule 設定
├── contract.fav     # IoContract + SlaContract 宣言
└── README.md        # 概要・実行手順
```

**実装内容:**
- `infra/e2e-demo/favnir4-showcase/pipeline.fav` — 骨格（各ステージのプレースホルダ）。Favnir 構文サンプル:
  ```favnir
  fn load_stage(ctx: AppCtx) -> Result<List<Row>, String> {
      bind rows <- ctx.io.read_csv("data/input.csv")
      Result.ok(rows)
  }
  ```
- `infra/e2e-demo/favnir4-showcase/fav.toml` — `[quality]` / `[contract]` / `[observe]` 設定
- `infra/e2e-demo/favnir4-showcase/contract.fav` — `Favnir4ShowcaseContract` 宣言
- `infra/e2e-demo/favnir4-showcase/README.md` — 実行手順

**完了条件**: Rust テスト 2 件（3897 + 2 = 3899）
- `favnir4_showcase_structure_exists`
- `favnir4_showcase_contract_valid`

---

## v84.2.0 — テスト統合ショーケース（`fav test` E2E）

ショーケースパイプラインに `fav test` を組み込み、型付きテストが通ることを示す。

**実装内容:**
- `infra/e2e-demo/favnir4-showcase/pipeline.fav` に `TestSuite` セクション追加
- `GoldenDataset` 比較・`StageTestCase` 単体テストを組み込み
- `SchemaSnapshot` 比較を追加

**完了条件**: Rust テスト 2 件（3899 + 2 = 3901）
- `showcase_test_suite_passes`
- `showcase_golden_dataset_comparison`

---

## v84.3.0 — 品質統合ショーケース（`fav quality` E2E）

ショーケースに品質チェックを組み込み、`QualityGate` で停止条件を設定する。

**実装内容:**
- `infra/e2e-demo/favnir4-showcase/pipeline.fav` に `QualityCheck` セクション追加
- `AnomalyDetector` による外れ値検知を組み込み
- `QualityGate.strict()` でパイプライン停止条件を設定

**完了条件**: Rust テスト 2 件（3913 + 2 = 3915）
- `showcase_quality_gate_passes`
- `showcase_anomaly_detector_integrated`

---

## v84.4.0 — 契約統合ショーケース（`fav verify --contract` E2E）

ショーケースの `contract.fav` を `fav verify --contract` で検証し、
`ContractRegistry` への登録を示す。

**実装内容:**
- `infra/e2e-demo/favnir4-showcase/contract.fav` を完成版に更新
- `SlaContract` + `ContractDependency` 宣言を追加
- `ContractRegistry` への登録フローを示す

**完了条件**: Rust テスト 2 件（3915 + 2 = 3917）
- `showcase_contract_verified`
- `showcase_contract_registry_registered`

---

## v84.5.0 — 可観測性統合ショーケース（`fav observe` E2E）

ショーケースに `fav observe` を組み込み、`HealthDashboard` が出力されることを示す。

**実装内容:**
- `infra/e2e-demo/favnir4-showcase/pipeline.fav` に `PipelineMetrics` 収集セクション追加
- `AlertRule` によるアラート評価を組み込み
- `SloTarget` 設定と `HealthDashboard` 出力を追加

**完了条件**: Rust テスト 2 件（3917 + 2 = 3919）
- `showcase_observe_metrics_collected`
- `showcase_health_dashboard_generated`

---

## v84.6.0 — ドキュメント完全化（`site/content/docs/v4/`）

4 スプリントで追加した全機能のドキュメントを `site/content/docs/v4/` に完全化する。

```
site/content/docs/v4/
├── test-driven-data.mdx    # fav test / TestSuite / GoldenDataset / SchemaSnapshot
├── data-quality.mdx        # QualityRule / QualityGate / AnomalyDetector
├── pipeline-contracts.mdx  # IoContract / SlaContract / ContractRegistry
├── observability.mdx       # PipelineMetrics / AlertRule / SloStatus / HealthDashboard
└── migration-v3-v4.mdx     # v3（v80.0）→ v4（v85.0）移行ガイド
```

**実装内容:**
- `site/content/docs/v4/test-driven-data.mdx`（`fav test` / `TestSuite` / `GoldenDataset` / `SchemaSnapshot` の解説）
- `site/content/docs/v4/data-quality.mdx`（`QualityRule` / `QualityGate` / `AnomalyDetector` の解説）
- `site/content/docs/v4/pipeline-contracts.mdx`（`IoContract` / `SlaContract` / `ContractRegistry` の解説）
- `site/content/docs/v4/observability.mdx`（`PipelineMetrics` / `AlertRule` / `SloStatus` / `HealthDashboard` の解説）
- `site/content/docs/v4/migration-v3-v4.mdx`（v3（v80.0）→ v4（v85.0）移行ガイド）
- 注: test-driven-data.mdx と migration-v3-v4.mdx は初回コミットで追加、残り 3 ファイルは同スプリント内の後続コミットで追加

**完了条件**: Rust テスト 2 件（3919 + 2 = 3921）
- `docs_v4_test_driven_data_exists`
- `docs_v4_migration_guide_exists`

---

## v84.7.0 — OSS 公開強化・コミュニティ整備 v2

Quality-First 機能に対応した OSS コントリビュートガイドと RFC プロセスを整備する。

**実装内容:**
- `CONTRIBUTING.md` を v4 対応に更新（QualityRule / IoContract 追加手順）
- `.github/ISSUE_TEMPLATE/quality-feedback.md` 新規作成（品質フィードバックテンプレート）
- `SECURITY.md` を最新に更新
- `CODE_OF_CONDUCT.md` を確認（内容に変更がなければ確認のみ、必要な場合のみ更新）

**完了条件**: Rust テスト 2 件（3921 + 2 = 3923）
- `oss_contributing_v4_exists`
- `oss_issue_template_quality_exists`

---

## v84.8.0 — パフォーマンス最終調整

4 スプリント分の型追加によるコンパイル時間・実行時間の劣化を確認・最適化する。

**実装内容:**
- `cargo test --release` で全テスト通過確認
- `PipelineMetrics` / `QualityCheck` / `ContractRegistry` の実行パス最適化（不要な Clone 削減）
- `fav bench --all` でベースラインとの乖離確認（`benchmarks/compare.fav` を使用）

**完了条件**: Rust テスト 2 件（3923 + 2 = 3925）
- `perf_cargo_test_release_passes`（`benchmarks/v80.0.0.json` が存在することを確認するテスト）
- `perf_no_regression_from_v80_baseline`（`benchmarks/v80.0.0.json` に `duration_ms` フィールドと `"80.0.0"` バージョン文字列が含まれることを確認するテスト）

---

## v84.9.0 — 安定化・コードフリーズ

v84.1〜v84.8 の全機能・v80.1〜v84.8 の全スプリントを通しで確認する最終安定化スプリント。

**実装内容:**
- v84.1〜v84.8 の全テスト通過確認（`cargo test` 全 pass）
- `infra/e2e-demo/favnir4-showcase/` の E2E ショーケース実行確認
- v80〜v84 の全スプリント統合動作確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（3925 + 2 = 3927）
- `favnir4_full_sprint_all_stable`
- `favnir4_e2e_showcase_runs`

---

## v85.0.0 — Favnir 4.0 宣言 ★クリーンアップ

**宣言文**:
> 「テストが型となり、品質が型となり、契約が型となり、観測が型となった。
>
>  `fav test` がパイプラインの正しさを証明し、
>  `QualityGate` が品質基準を守り、
>  `IoContract` がチームを安全に繋ぎ、
>  `AlertRule` が壊れる前に教えてくれる。
>
>  Favnir 4.0 は、データパイプラインの品質を
>  コードと同じ言語で語れる、唯一の言語である。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `85.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` の現行マスターロードマップが `roadmap-v80.1-v85.0.md` を指していることを確認してから更新
- `roadmap-v80.1-v85.0.md` の Sprint 5 バージョン一覧テーブルを全行「完了」に更新

**完了条件**: `v85000_tests` 4 件（3927 + 4 = 3931）
- `cargo_toml_version_is_85_0_0`
- `changelog_has_v85_0_0`
- `milestone_has_favnir_4`
- `readme_mentions_favnir_4`（`README.md` に `"Favnir 4.0"` という文字列が含まれていることを確認）

---

## テスト数推移（本スプリント）

> code-reviewer 対応の累積により実際のベースは 3,909（計画値 3,897 から +12）。

| バージョン | テスト数（実績） | 増加 |
|---|---|---|
| v84.0.0（ベース） | 3,909 | — |
| v84.1.0 | 3,911 | +2 |
| v84.2.0 | 3,913 | +2 |
| v84.3.0 | 3,915 | +2 |
| v84.4.0 | 3,917 | +2 |
| v84.5.0 | 3,919 | +2 |
| v84.6.0 | 3,921 | +2 |
| v84.7.0 | 3,923 | +2 |
| v84.8.0 | 3,925 | +2 |
| v84.9.0 | 3,927 | +2 |
| v85.0.0（宣言） | 3,931 | +4 |

**本スプリント合計**: +22 tests（3,909 → 3,931）

---

## Quality-First Era 全スプリント総括

| スプリント | 期間 | テーマ | テスト増 | 到達値 |
|---|---|---|---|---|
| Test-Driven Data 1.0 | v80.1〜v81.0 | テストが型になる | +22 | 3,831 |
| Data Quality 2.0 | v81.1〜v82.0 | 品質が型になる | +22 | 3,853 |
| Pipeline Contracts 1.0 | v82.1〜v83.0 | 契約が型になる | +22 | 3,875 |
| Observability 2.0 | v83.1〜v84.0 | 観測が型になる | +22 | 3,897 |
| **Favnir 4.0 宣言** | **v84.1〜v85.0** | **統合・宣言** | **+22** | **3,919** |
| **合計** | | | **+110** | **3,919** |

---

## 参考リンク

- マスターロードマップ: [roadmap-v80.1-v85.0.md](roadmap-v80.1-v85.0.md)
- 前スプリント: [roadmap-v83.1-v84.0.md](roadmap-v83.1-v84.0.md)
- 次フェーズ: （未計画 — v85.0.0 宣言後に策定）
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
