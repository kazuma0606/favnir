# v23.0.0 実装計画 — Distributed Scale マイルストーン宣言

## 実装方針

v22.0.0（Developer Tooling Complete 宣言）と同じパターン。
新機能・Rust コード変更は最小限。主な作業はドキュメント整備とバージョン更新。

---

## タスク順序

| タスク | 内容 | 依存 |
|---|---|---|
| T1 | `benchmarks/v23.0.0.json` 作成 | なし |
| T2 | `fav/Cargo.toml` バージョン更新（22.8.0 → 23.0.0） | なし |
| T3 | `CHANGELOG.md` 更新（v23.0.0 エントリ追加） | なし |
| T4 | `README.md` 更新（Distributed Scale セクション追加） | なし |
| T5 | `site/content/docs/tools/distributed-scale.mdx` 新規作成 | なし |
| T6 | `fav/src/driver.rs` — `v230000_tests` 追加 | T1, T2, T3, T4, T5 |

**Rust コードへの変更は T2（バージョン）と T6（テスト）のみ。**

---

## T1: `benchmarks/v23.0.0.json` — マイルストーンスナップショット

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
    "checkpoint_resume":   { "achieved": true, "version": "v22.1.0" },
    "distributed_par":     { "achieved": true, "version": "v22.2.0" },
    "s3_event_trigger":    { "achieved": true, "version": "v22.4.0" },
    "otel_jaeger_trace":   { "achieved": true, "version": "v22.7.0" },
    "orchestrate_dag":     { "achieved": true, "version": "v22.5.0" }
  }
}
```

> `metrics` キーに `"metrics"` 文字列が含まれることが `bench_v23_baseline_exists` テストで確認される。

---

## T2: `fav/Cargo.toml` バージョン更新

```toml
version = "23.0.0"
```

---

## T3: `CHANGELOG.md` 更新

v22.1.0〜v22.8.0 のエントリはすでに存在する（要確認）。先頭に v23.0.0 エントリを追加:

```markdown
## [v23.0.0] — 2026-06-21 — Distributed Scale マイルストーン宣言

v22.1.0〜v22.8.0 で達成した分散スケール機能の集大成。
全 5 完了条件（Checkpoint 再開 / Distributed par / S3 トリガー /
OpenTelemetry trace / fav orchestrate DAG）を達成。
```

既存の v22.8.0 エントリの直上に挿入する。

---

## T4: `README.md` 更新

### 変更箇所

1. バージョンバッジ / 「現在のバージョン」を v23.0.0 に更新

2. **Distributed Scale** セクションを Features 一覧に追加（Developer Tooling セクションの直下）:

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

3. バージョン履歴表に v22.1.0〜v23.0.0 のエントリを追加:

```markdown
| v22.1.0 | Checkpoint / Resume |
| v22.2.0 | Distributed `par` |
| v22.3.0 | Pipeline State Rune |
| v22.4.0 | Event-driven Pipeline（`#[trigger]`）|
| v22.5.0 | Pipeline Orchestration（`fav orchestrate`）|
| v22.6.0 | SLA 宣言（`#[timeout]` / `#[retry]` / `#[circuit_breaker]`）|
| v22.7.0 | OpenTelemetry 統合（`fav run --trace`）|
| v22.8.0 | `fav deploy` 強化（ECS / K8s / Fly.io）|
| v23.0.0 | Distributed Scale マイルストーン宣言 |
```

---

## T5: `site/content/docs/tools/distributed-scale.mdx`

```mdx
---
title: Distributed Scale
description: v22.x シリーズで達成した分散スケール機能の全体像
---

# Distributed Scale

v22.x シリーズ（v22.1.0〜v22.8.0）で達成した分散スケール機能の全体像。

> **Distributed Scale マイルストーン（v23.0.0）**: 全 5 完了条件達成

## 達成した完了条件

| 完了条件 | 達成バージョン |
|---|---|
| checkpoint 付きパイプラインが失敗後に再開できる | v22.1.0 ✅ |
| `par_distributed [A, B, C]` が複数 Worker で並列実行できる | v22.2.0 ✅ |
| `#[trigger(event = "s3:...")]` で S3 イベント駆動パイプラインがデプロイできる | v22.4.0 ✅ |
| OpenTelemetry の trace が Jaeger で確認できる | v22.7.0 ✅ |
| `fav orchestrate` で multi-step DAG が依存順に実行できる | v22.5.0 ✅ |

## 機能一覧

- [Checkpoint / Resume](/docs/cli/checkpoint) — v22.1.0
- [Distributed par](/docs/cli/par-distributed) — v22.2.0
- [Pipeline State Rune](/docs/runes/state) — v22.3.0
- [Event-driven Pipeline](/docs/cli/trigger) — v22.4.0
- [Pipeline Orchestration](/docs/cli/orchestrate) — v22.5.0
- [SLA 宣言](/docs/cli/sla) — v22.6.0
- [OpenTelemetry](/docs/cli/otel) — v22.7.0
- [`fav deploy`](/docs/cli/deploy) — v22.8.0
```

---

## T6: `fav/src/driver.rs` — `v230000_tests` 追加

### 事前: `v228000_tests::version_is_22_8_0` に `#[ignore]` を追加

```rust
#[test]
#[ignore]
fn version_is_22_8_0() { ... }
```

### テストコード

```rust
#[cfg(test)]
mod v230000_tests {
    use super::*;

    fn repo_path(rel: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join(rel)
    }

    #[test]
    fn version_is_23_0_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("\"23.0.0\""), "Cargo.toml should have version 23.0.0");
    }

    #[test]
    fn changelog_has_v22x_entries() {
        let cl = include_str!("../../CHANGELOG.md");
        for v in &["v22.1.0", "v22.2.0", "v22.3.0", "v22.4.0",
                   "v22.5.0", "v22.6.0", "v22.7.0", "v22.8.0", "v23.0.0"] {
            assert!(cl.contains(v), "CHANGELOG should have {} entry", v);
        }
    }

    #[test]
    fn readme_mentions_otel() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("OpenTelemetry") || readme.contains("OTel"),
            "README should mention OpenTelemetry"
        );
    }

    #[test]
    fn readme_mentions_orchestrate() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("orchestrate") || readme.contains("DAG"),
            "README should mention fav orchestrate or DAG"
        );
    }

    #[test]
    fn bench_v23_baseline_exists() {
        let content = include_str!("../../benchmarks/v23.0.0.json");
        assert!(content.contains("\"metrics\""),
            "v23.0.0.json should contain metrics field");
    }
}
```

---

## 実装上の注意点

### T6 は T1 完了前に `cargo check` を実行しないこと

`bench_v23_baseline_exists` テストは `include_str!("../../benchmarks/v23.0.0.json")` を使用する。
`include_str!` はコンパイル時にファイルが存在しないとビルドエラーになる。
**T1（benchmarks/v23.0.0.json 作成）を完了させてから T6 の実装・`cargo check` を実行すること。**

### v228000_tests::version_is_22_8_0 に `#[ignore]` を追加するタイミング

T2（Cargo.toml を 23.0.0 に更新）より前に `#[ignore]` を付ける。
順序を逆にすると `version_is_22_8_0` がバージョン不一致で失敗する。

### CHANGELOG の既存確認

v22.1.0〜v22.8.0 のエントリはすでに存在している（`grep "v22\." CHANGELOG.md` で確認）。
T3 では v23.0.0 エントリの追加のみ行う。

---

## リスクと対策

| リスク | 対策 |
|---|---|
| CHANGELOG に v22.x エントリが欠けている | T6 の `changelog_has_v22x_entries` でコンパイル時に全バージョン確認 |
| README に OTel / orchestrate が未記載 | `readme_mentions_otel` / `readme_mentions_orchestrate` テストで確認 |
| `benchmarks/v23.0.0.json` の JSON 形式が不正 | `include_str!` + `contains("\"metrics\"")` でシンプルに確認 |
| T2 前に `#[ignore]` を付け忘れる | tasks.md の T6 チェックリストに「#[ignore] は T2 より先に実施」の注意を明記 |
