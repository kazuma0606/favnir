# Roadmap v22.1.0 〜 v23.0.0 — Distributed Scale

Date: 2026-06-18

## 目標

v22.0「Developer Tooling Complete」で「開発体験が最高」を達成した。
次の壁は「**単一マシンに収まらない規模**」である。

1TB / 1PB のデータ、24 時間動き続けるパイプライン、SLA を持つジョブ——
これらは単一プロセスでは解決できない。
Favnir のパイプライン型（`seq`）が分散実行の記述にそのまま使えることを証明する。

**完了条件:**
1. checkpoint 付きパイプラインが失敗後に再開できる
2. `par_distributed [A, B, C]` が 3 台の Worker で並列実行できる
3. `#[trigger(event = "s3:...")]` で S3 イベント駆動パイプラインがデプロイできる
4. OpenTelemetry の trace が Jaeger で確認できる
5. `fav orchestrate` で multi-step DAG が依存順に実行できる

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| Checkpoint 形式 | stage 出力を `.favc`（Favnir Compiled）形式で保存。SHA-256 で整合性検証 |
| Checkpoint 保存先 | `--checkpoint-dir`（デフォルト `.fav_checkpoint/`） |
| Worker 通信プロトコル | 既存の gRPC Rune（v9.5.0）を流用。stage バイトコードを Worker に転送 |
| Pipeline State バックエンド | Redis / DynamoDB / PostgreSQL（JSONB）の3択 |
| Event-driven デプロイ先 | Lambda + EventBridge（S3トリガー）/ Lambda + MSK（Kafka トリガー） |
| Orchestration モデル | DAG（`after` キーワードで依存宣言）。Airflow 互換 |
| SLA 検証タイミング | コンパイル時（`#[timeout]` 等のアノテーションは静的チェック） |
| OTel エクスポート | OTLP（gRPC）。`fav run --trace` フラグで有効化。Jaeger / Grafana Tempo 対応 |
| deploy ターゲット | ECS Fargate / Kubernetes CronJob / Fly.io の3択 |

---

## バージョン計画

### v22.1 — Checkpoint / Resume（パイプライン永続化）

**テーマ**: 長時間実行パイプラインの中断・再開を安全に行う。

```favnir
// checkpoint を宣言した stage から再開可能
#[checkpoint]
stage ProcessBatch: List<Row> -> List<Result> = |rows| { ... }

seq LongRunning = Load |> ProcessBatch |> Save
```

```bash
fav run --checkpoint-dir /tmp/ckpt pipeline.fav
# 中断後:
fav run --resume /tmp/ckpt/2026-06-18-12345 pipeline.fav
```

#### 内部実装

- checkpoint 時: stage の出力を `.favc` 形式で `.fav_checkpoint/` に保存
- resume 時: checkpoint 済み stage をスキップして次 stage から再開
- 状態の整合性: SHA-256 でデータの同一性を検証

---

### v22.2 — Distributed `par`（複数 Worker への分散）

**テーマ**: 現状のシングルマシン `par [A, B]` を複数マシンに分散する。

```favnir
// worker プール宣言
seq DistributedReport
  [workers: Worker.Pool]
= par_distributed [FetchOrders, FetchPrices, FetchInventory] |> Merge
```

```toml
# fav.toml
[workers]
endpoints = [
  "grpc://worker-1:9090",
  "grpc://worker-2:9090",
  "grpc://worker-3:9090",
]
```

#### 実装

- 既存の gRPC Rune（v9.5.0）を Worker 通信に流用
- stage のバイトコード（`.favc`）を Worker に転送して実行
- 結果の収集と `Merge` stage への受け渡し

---

### v22.3 — Pipeline State Rune（分散状態管理）

**テーマ**: 複数の Worker をまたぐ状態を型安全に管理する。

```favnir
import rune "state"

// 型付き分散キャッシュ
stage DeduplicateRows: List<Row> -> List<Row> = |rows| {
  bind seen <- State.get_set<String>("seen_ids")
  List.filter(rows, |r| State.Set.insert(seen, r.id))
}
```

対応バックエンド: Redis / DynamoDB / PostgreSQL（JSONB）

---

### v22.4 — Event-driven Pipeline（イベントトリガー）

**テーマ**: S3 / SQS / Kafka をトリガーとするパイプラインを Favnir で定義する。

```favnir
// S3 ファイルアップロードをトリガーに
#[trigger(event = "s3:ObjectCreated", bucket = "raw-data")]
seq ProcessUpload = ParseCsv |> Validate |> WriteToWarehouse

// Kafka メッセージをトリガーに
#[trigger(event = "kafka:message", topic = "orders")]
seq ProcessOrder = DeserializeOrder |> EnrichOrder |> SaveOrder
```

```bash
fav deploy --trigger src/pipeline.fav
# → Lambda + EventBridge / Lambda + Kafka trigger として自動デプロイ
```

---

### v22.5 — Pipeline Orchestration（DAG スケジューリング）

**テーマ**: Airflow / Prefect を使わずに Favnir 自体でパイプライン間の依存を管理する。

```favnir
// パイプライン間の依存宣言
pipeline DailyETL {
  step "load_raw"    = seq LoadRaw
  step "transform"   = seq Transform  after "load_raw"
  step "enrich"      = seq Enrich     after "transform"
  step "write"       = seq Write      after "enrich", "load_metadata"
  step "load_meta"   = seq LoadMeta
}
```

```bash
fav orchestrate run DailyETL   # 依存順に自動実行
fav orchestrate status          # 実行状況確認
fav orchestrate retry "enrich"  # 特定 step のみ再実行
```

---

### v22.6 — SLA 宣言（タイムアウト・リトライ・サーキットブレーカー）

**テーマ**: 本番パイプラインの信頼性を型システムレベルで保証する。

```favnir
#[timeout(seconds = 30)]
#[retry(max = 3, backoff = "exponential")]
#[circuit_breaker(threshold = 0.5, window = 60)]
stage CallExternalAPI: Request -> Response = |req| {
  http.post(req)
}
```

SLA 宣言はコンパイル時にチェックされ、
`fav explain --sla` でパイプライン全体の最悪実行時間が計算できる。

---

### v22.7 — OpenTelemetry 統合

**テーマ**: 分散トレーシングを標準で組み込む。

```bash
fav run --trace src/pipeline.fav
# → OpenTelemetry traces を OTLP エンドポイントに送信
# → Jaeger / Grafana Tempo で可視化可能
```

```favnir
// 自動で span を生成
// stage = 1 span、stage の入出力サイズ = span attributes
seq Pipeline = LoadCsv |> Transform |> Save
// trace: Pipeline / LoadCsv / Transform / Save の 4 span
```

環境変数での設定:

```bash
OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317 fav run --trace pipeline.fav
```

---

### v22.8 — `fav deploy` 強化（ECS / EKS 対応）

**テーマ**: 現状の Lambda のみから、コンテナベース実行環境にも対応する。

```bash
fav deploy --target ecs  src/pipeline.fav   # AWS ECS Fargate
fav deploy --target k8s  src/pipeline.fav   # Kubernetes CronJob
fav deploy --target fly  src/pipeline.fav   # Fly.io
```

デプロイ設定は `fav.toml` の `[deploy]` セクションで管理:

```toml
[deploy]
target = "ecs"
region = "ap-northeast-1"
cpu    = "1024"
memory = "2048"
```

---

## v23.0 — Distributed Scale マイルストーン宣言

**完了条件:**
1. checkpoint 付きパイプラインが失敗後に再開できる
2. `par_distributed [A, B, C]` が 3 台の Worker で並列実行できる
3. `#[trigger(event = "s3:...")]` で S3 イベント駆動パイプラインがデプロイできる
4. OpenTelemetry の trace が Jaeger で確認できる
5. `fav orchestrate` で multi-step DAG が依存順に実行できる

---

## 参考リンク

- 前フェーズ: `versions/roadmap/roadmap-v21.1-v22.0.md`
- 次フェーズ: `versions/roadmap/roadmap-v23.1-v24.0.md`
- マスタースケジュール: `versions/roadmap-v20.1-v25.0.md`
