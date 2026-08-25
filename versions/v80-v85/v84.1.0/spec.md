# spec: v84.1.0 — E2E ショーケース基盤

## Background

> **テスト数注記**: ロードマップ策定時の計画値は 3,897/3,899 だったが、
> code-reviewer 指摘対応（各バージョンへの追加テスト）により実際のベースは
> **3,909 tests**（v84.0.0 完了時点）となっている。
> v84.1.0 完了目標は **3,911 tests**（+2）。

v84.0.0「Observability 2.0 宣言」により、Quality-First Era の 4 スプリントが完成した:
- Sprint 1 (v80.1〜v81.0): Test-Driven Data 1.0 — `fav test` / TestSuite / GoldenDataset
- Sprint 2 (v81.1〜v82.0): Data Quality 2.0 — QualityRule / QualityGate / AnomalyDetector
- Sprint 3 (v82.1〜v83.0): Pipeline Contracts 1.0 — IoContract / SlaContract / ContractRegistry
- Sprint 4 (v83.1〜v84.0): Observability 2.0 — PipelineMetrics / AlertRule / SloStatus / HealthDashboard

v84.1.0 は Sprint 5「Favnir 4.0 宣言」の第一歩として、4 スプリントすべての機能を網羅する
E2E ショーケースの骨格を作成する。`infra/e2e-demo/favnir3-showcase/`（v79.1.0 相当）と
同構造で、Favnir 4.0 時代のショーケースを `favnir4-showcase/` として整備する。

## Goals

1. `infra/e2e-demo/favnir4-showcase/` ディレクトリを作成し、4 柱統合パイプラインの骨格を配置する
2. `pipeline.fav` — 各ステージのプレースホルダを Favnir 構文で記述（実際の IO は省略可）
3. `fav.toml` — `[quality]` / `[contract]` / `[observe]` セクションの設定例を含める
4. `contract.fav` — `Favnir4ShowcaseContract`（IoContract + SlaContract）を宣言
5. `README.md` — ショーケースの概要・実行手順を記述
6. Rust テスト 2 件でディレクトリ構造と contract.fav の内容を検証する

## Syntax / API Examples

### pipeline.fav（骨格）

```favnir
-- Favnir 4.0 Showcase Pipeline
-- 4 Quality 柱（Test / Quality / Contract / Observe）を統合するデモパイプライン

fn load_stage(ctx: AppCtx) -> Result<List<Row>, String> {
    bind rows <- ctx.io.read_csv("data/input.csv")
    Result.ok(rows)
}

fn transform_stage(rows: List<Row>) -> Result<List<Row>, String> {
    -- 変換ロジックのプレースホルダ
    Result.ok(rows)
}

fn quality_stage(rows: List<Row>) -> Result<List<Row>, String> {
    -- QualityCheck を適用する（Sprint 2）
    Result.ok(rows)
}

fn observe_stage(rows: List<Row>) -> Result<List<Row>, String> {
    -- PipelineMetrics を収集する（Sprint 4）
    Result.ok(rows)
}
```

### fav.toml（設定例）

```toml
[package]
name = "favnir4-showcase"
version = "1.0.0"

[quality]
gate = "permissive"
rules = ["not_null:0", "range:1:0:100"]

[contract]
input = "Favnir4ShowcaseContract"
sla_target_ms = 500

[observe]
alert_threshold_ms = 1000
slo_objective_pct = 99.0
```

### contract.fav（型宣言のみ）

> **スコープ注記**: v84.1.0 では `Favnir4ShowcaseContract` 型の宣言のみ行う。
> `IoContract` / `SlaContract` インタフェースの実装は v84.4.0 以降で追加する予定。

```favnir
-- Favnir 4.0 Showcase Contract

type Favnir4ShowcaseContract {
    input_fields: List<String>,
    output_fields: List<String>,
    sla_ms: Int,
}
```

## Success Criteria

- `infra/e2e-demo/favnir4-showcase/pipeline.fav` が存在し Favnir 構文で記述されていること
- `infra/e2e-demo/favnir4-showcase/fav.toml` が存在し `[quality]`・`[contract]`・`[observe]` セクションを含むこと
- `infra/e2e-demo/favnir4-showcase/contract.fav` が存在し `Favnir4ShowcaseContract` を含むこと
- `infra/e2e-demo/favnir4-showcase/README.md` が存在すること
- `cargo test` が 3,911 tests pass（+2）、0 failures であること

## Error Codes

なし（本バージョンはファイル配置のみ。構文エラーは fav build で確認）

## Files to Modify / Create

### 新規作成
- `infra/e2e-demo/favnir4-showcase/pipeline.fav`
- `infra/e2e-demo/favnir4-showcase/fav.toml`
- `infra/e2e-demo/favnir4-showcase/contract.fav`
- `infra/e2e-demo/favnir4-showcase/README.md`

### 追記
- `fav/src/driver.rs` — `v84100_tests` モジュール追加（2 テスト）
- `CHANGELOG.md` — v84.1.0 エントリ追加

### パス起点の違いについて

テスト内でのパス指定は 2 種類ある:

| マクロ / 関数 | パス起点 | 理由 |
|---|---|---|
| `std::path::Path::new("../infra/...")` | `fav/`（`cargo test` の CWD） | ランタイムで解決 |
| `include_str!("../../infra/...")` | `fav/src/`（ソースファイルの位置） | コンパイル時マクロ |

`favnir4_showcase_structure_exists` は `Path::new` を使うため `"../infra/..."` 形式。
`favnir4_showcase_contract_valid` は `include_str!` を使うため `"../../infra/..."` 形式。
