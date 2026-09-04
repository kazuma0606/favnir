# Favnir 4.0 Showcase

## 概要

Favnir 4.0 の 4 Quality 柱を統合するデモパイプラインです。

- **Sprint 1** — Test-Driven Data 1.0（`fav test` / TestSuite / GoldenDataset）
- **Sprint 2** — Data Quality 2.0（QualityRule / QualityGate / AnomalyDetector）
- **Sprint 3** — Pipeline Contracts 1.0（IoContract / SlaContract / ContractRegistry）
- **Sprint 4** — Observability 2.0（PipelineMetrics / AlertRule / SloStatus / HealthDashboard）

## 前提

- Favnir コンパイラ v84.0.0 以上
- `fav` CLI がインストール済みであること

## 実行方法

```sh
fav run pipeline.fav
```

`pipeline.fav` は 4 ステージのプレースホルダ実装を含みます。
各ステージを実装して `fav test` / `fav check` / `fav observe` で動作を確認してください。
