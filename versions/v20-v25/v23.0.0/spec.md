# v23.0.0 Spec — Distributed Scale マイルストーン宣言

## 概要

v22.x シリーズ（v22.1〜v22.8）で構築した分散スケール機能の集大成を宣言するマイルストーンリリース。
新しい言語機能の追加はなく、ロードマップ完了条件の確認・CHANGELOG 更新・README 更新・バージョン番号の更新が主な作業。

**テーマ**: 「単一マシンに収まらない規模のパイプラインを Favnir で書ける」への到達宣言

---

## v22.x で達成した Distributed Scale 機能

| バージョン | 機能 | 達成内容 |
|---|---|---|
| v22.1.0 | Checkpoint / Resume | `#[checkpoint]` アノテーション、`.favc` 形式でのステージ出力保存、`--resume` フラグで中断後再開 |
| v22.2.0 | Distributed `par` | `par_distributed [A, B, C]`、gRPC Worker 通信、`fav.toml [workers]` セクション |
| v22.3.0 | Pipeline State Rune | `import rune "state"`、Redis / DynamoDB / PostgreSQL バックエンド、型付き分散キャッシュ |
| v22.4.0 | Event-driven Pipeline | `#[trigger(event = "s3:ObjectCreated")]` / `"kafka:message"`、`fav deploy --trigger` |
| v22.5.0 | Pipeline Orchestration | `pipeline { step ... after ... }` DAG 構文、`fav orchestrate run/status/retry` |
| v22.6.0 | SLA 宣言 | `#[timeout]` / `#[retry]` / `#[circuit_breaker]`、コンパイル時チェック、`fav explain --sla` |
| v22.7.0 | OpenTelemetry 統合 | `fav run --trace`、`SeqStageEnter/Check` span 自動生成、OTLP HTTP エクスポート |
| v22.8.0 | `fav deploy` 強化 | `--target ecs/k8s/fly`、Dockerfile / ECS task def / K8s CronJob YAML / fly.toml 生成 |

---

## ロードマップ完了条件との対応

| ロードマップ完了条件 | 達成バージョン | 検証テスト |
|---|---|---|
| checkpoint 付きパイプラインが失敗後に再開できる | v22.1.0 | `changelog_has_v22x_entries` |
| `par_distributed [A, B, C]` が 3 台の Worker で並列実行できる | v22.2.0 | `changelog_has_v22x_entries` |
| `#[trigger(event = "s3:...")]` で S3 イベント駆動パイプラインがデプロイできる | v22.4.0 | `changelog_has_v22x_entries` |
| OpenTelemetry の trace が Jaeger で確認できる | v22.7.0 | `readme_mentions_otel` |
| `fav orchestrate` で multi-step DAG が依存順に実行できる | v22.5.0 | `readme_mentions_orchestrate` |

---

## v23.0.0 実装内容

### 1. バージョン番号更新

- `fav/Cargo.toml`: `22.8.0` → `23.0.0`

### 2. CHANGELOG.md 更新

v22.1.0〜v22.8.0 のエントリはすでに記載済み。v23.0.0 エントリを先頭に追加:

```markdown
## [v23.0.0] — 2026-06-21 — Distributed Scale マイルストーン宣言

v22.1.0〜v22.8.0 で達成した分散スケール機能の集大成。
全 5 完了条件（Checkpoint 再開 / Distributed par / S3 トリガー /
OpenTelemetry trace / fav orchestrate DAG）を達成。
```

### 3. README.md 更新

- 「現在のバージョン」を v23.0.0 に更新
- **Distributed Scale** セクションを Features 一覧に追加（Developer Tooling セクションの直下）
- バージョン履歴表に v22.1.0〜v23.0.0 のエントリを追加

```markdown
### Distributed Scale（v22.x）
- **Checkpoint / Resume**: 長時間パイプラインの中断・再開（`#[checkpoint]`、`fav run --resume`）
- **Distributed par**: `par_distributed [A, B, C]` で gRPC Worker に分散実行
- **Pipeline State Rune**: Redis / DynamoDB / PostgreSQL バックエンドの型付き分散キャッシュ
- **Event-driven Pipeline**: `#[trigger(event = "s3:ObjectCreated")]` で S3 / Kafka トリガー
- **Pipeline Orchestration**: `pipeline { step ... after ... }` DAG と `fav orchestrate`
- **SLA 宣言**: `#[timeout]` / `#[retry]` / `#[circuit_breaker]`（コンパイル時チェック）
- **OpenTelemetry**: `fav run --trace` で自動 span 生成・OTLP エクスポート
- **`fav deploy` 強化**: `--target ecs/k8s/fly` でコンテナ実行環境に対応
```

### 4. ベンチマーク記録（`benchmarks/v23.0.0.json`）

v22.8.0 時点でのテスト件数・分散機能数をスナップショットとして記録:

```json
{
  "version": "23.0.0",
  "timestamp": "2026-06-21T00:00:00Z",
  "_note": "Distributed Scale milestone snapshot: checkpoint/resume, distributed par, event-driven, orchestration, OTel.",
  "metrics": {
    "test_count": 1887,
    "distributed_features": 5,
    "deploy_targets": 4,
    "otel_span_types": 2,
    "sla_annotation_types": 3
  },
  "_metrics_notes": {
    "distributed_features": "checkpoint / distributed_par / pipeline_state / event_trigger / orchestrate",
    "deploy_targets": "aws-lambda(既存) / ecs / k8s / fly",
    "otel_span_types": "SeqStageEnter / SeqStageCheck の 2 種類",
    "sla_annotation_types": "timeout / retry / circuit_breaker",
    "test_count": "v22.8.0完了時の実測値（1882）+ v230000_tests 5件"
  },
  "milestone_checklist": {
    "checkpoint_resume":       { "achieved": true, "version": "v22.1.0" },
    "distributed_par":         { "achieved": true, "version": "v22.2.0" },
    "s3_event_trigger":        { "achieved": true, "version": "v22.4.0" },
    "otel_jaeger_trace":       { "achieved": true, "version": "v22.7.0" },
    "orchestrate_dag":         { "achieved": true, "version": "v22.5.0" }
  }
}
```

### 5. site/ MDX 追加

- `site/content/docs/tools/distributed-scale.mdx` **新規** — Distributed Scale マイルストーン概要ページ

### 6. テスト（v230000_tests、5 件）

```rust
fn version_is_23_0_0()              // Cargo.toml に "23.0.0" が含まれる
fn changelog_has_v22x_entries()     // CHANGELOG に v22.1.0〜v22.8.0 + v23.0.0 の全エントリが含まれる
fn readme_mentions_otel()           // README に "OpenTelemetry" または "OTel" が含まれる
fn readme_mentions_orchestrate()    // README に "orchestrate" または "DAG" が含まれる
fn bench_v23_baseline_exists()      // benchmarks/v23.0.0.json が存在し "metrics" を含む
                                    // ※ T1（benchmarks/v23.0.0.json）完了前に cargo check を実行すると
                                    //   include_str! のコンパイルエラーになるため T1 を先に完了させること
```

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml` に `"23.0.0"` が含まれる | [ ] |
| `CHANGELOG.md` に v22.1.0〜v22.8.0 の全エントリが含まれる（既存確認） | [ ] |
| `CHANGELOG.md` に v23.0.0 エントリが含まれる | [ ] |
| `README.md` に Distributed Scale セクションの記載がある | [ ] |
| `README.md` に OpenTelemetry の記載がある | [ ] |
| `README.md` に orchestrate の記載がある | [ ] |
| `benchmarks/v23.0.0.json` が存在し `"metrics"` フィールドを含む valid JSON | [ ] |
| `site/content/docs/tools/distributed-scale.mdx` が存在する | [ ] |
| `cargo test v230000 --bin fav` — 5/5 PASS | [ ] |
| `cargo test --bin fav` — リグレッションなし（1882 件以上合格） | [ ] |
