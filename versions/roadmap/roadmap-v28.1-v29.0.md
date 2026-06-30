# Roadmap v28.1.0 〜 v29.0.0 — Observability First

Date: 2026-06-24

## 目標

v28.0「Data Lakehouse」でモダンなデータ基盤に溶け込めるようになった。
バッチ・ストリーミング・レイクハウスの三層がすべて型安全に動く。

しかし「動いている」と「動いていることが確認できる」は別の話だ。
パイプラインが夜間に失敗したとき、どの stage で何のエラーが起きたか、
どこがボトルネックか——これが見えないと本番運用できない。

このフェーズでは、**Favnir のエフェクトシステムを活用して可観測性を言語レベルに組み込む**。
`#[track]` / `#[trace]` / `#[on_error]` アノテーションを加えるだけで、
Prometheus / Datadog / Sentry / Grafana と自動連携できるようにする。

> **Observability First の定義（本プロジェクト固有）**
> 「`#[track(latency, error_rate)]` を stage に付けるだけで
>  Grafana ダッシュボードにメトリクスが現れる」状態を指す。

**完了条件（最終テスト）:**

```bash
# 1. 全 Rust テストが通る
cargo test

# 2. 可観測性 Rune テスト
cargo test prometheus datadog otel sentry grafana

# 3. fav profile フレームグラフ出力
fav profile --format flamegraph tests/fixtures/etl.fav

# 4. E2E デモが動く
docker compose -f examples/observability/docker-compose.yml up -d
fav run examples/observability/prometheus_grafana.fav
fav run examples/observability/datadog_apm.fav
fav run examples/observability/sentry_alerting.fav
```

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| `#[track]` の実装方式 | `parser.rs` / `ast.rs` に `TrackAnnotation` を追加し、コンパイラが stage 前後に計測コードを自動挿入 |
| `#[trace]` の実装方式 | `#[track]` と同じ機構。OTel スパンを自動開始・終了 |
| `#[on_error]` の実装方式 | stage の `Result.err` 時に指定の Rune 関数を自動呼び出し |
| Prometheus のローカル環境 | `prom/prometheus` + `grafana/grafana` Docker |
| Datadog のローカル環境 | `datadog/agent:7` Docker（ローカルモード）|
| Sentry のローカル環境 | `getsentry/sentry` Docker（セルフホスト） |
| フレームグラフの生成方式 | `inferno` クレートで SVG 生成。`perf` / `dtrace` データを入力 |
| `fav profile --compare` の比較対象 | `benchmarks/vX.Y.Z.json` に記録されたステージ別実行時間 |
| 破壊的変更 | なし（STABILITY.md v1.x ポリシーに従う） |

---

## バージョン計画

### v28.1 — prometheus Rune 追加

**テーマ**: メトリクス収集の標準。`#[track]` アノテーションとの統合で
stage の実行時間・成功率を宣言的に計測できるようにする。

**依存関係**: なし

```favnir
import runes/prometheus

// stage の実行時間・エラー率を自動収集
#[track(latency: true, error_rate: true, labels: ["pipeline", "stage"])]
stage TransformOrders: List<RawOrder> -> List<Order> !Io = |orders| {
  orders
  |> List.map(parse_order)
  |> List.filter(|o| o.amount > 0.0)
  |> Result.ok
}

// カスタムメトリクス
stage ReportMetrics: PipelineSummary -> Unit !Io = |summary| {
  bind _ <- Prometheus.counter("rows_processed_total", summary.count, ["pipeline": "etl"])
  bind _ <- Prometheus.histogram("stage_duration_ms", summary.duration_ms)
  bind _ <- Prometheus.gauge("queue_depth", Float.from_int(summary.queue_size))
  Result.ok(unit)
}
```

実装する関数 + アノテーション:

| 関数 / アノテーション | 内容 |
|---|---|
| `Prometheus.counter(name, value, labels)` | カウンタ（単調増加）|
| `Prometheus.gauge(name, value)` | ゲージ（増減あり）|
| `Prometheus.histogram(name, value)` | ヒストグラム（分布計測）|
| `Prometheus.push(gateway_url)` | Pushgateway への送信 |
| `#[track(latency, error_rate, labels)]` | stage 前後に自動計測コードを挿入 |

`cargo test prometheus` で 4 件以上 PASS。
`prom/prometheus`（Docker）で実際にメトリクスが収集されることを確認。

---

### v28.2 — datadog Rune 追加

**テーマ**: APM・ログ・メトリクスの統合プラットフォーム。
ひとつの Rune でトレース・ログ・カスタムメトリクスを一括送信できる。

**依存関係**: v28.1 と並行可能（API 設計の参照のみ）

実装する関数:

| 関数 | 内容 |
|---|---|
| `Datadog.metric(name, value, tags)` | カスタムメトリクス送信（DogStatsD）|
| `Datadog.log(level, message, attrs)` | 構造化ログ送信（JSON Lines 形式）|
| `Datadog.trace(name, fn)` | APM トレース（スパン自動開始・終了）|
| `Datadog.event(title, text, tags)` | イベント通知（デプロイ・アラート等）|
| `Datadog.service_check(name, status)` | サービスチェック（OK / WARN / CRITICAL）|

`datadog/agent:7`（Docker ローカルモード）で `cargo test datadog` が 4 件以上 PASS。

---

### v28.3 — OpenTelemetry Rune 強化

**テーマ**: 既存の `otel.rs`（v22.x で追加）を Rune として公開し、
`#[trace]` アノテーションが Jaeger / Tempo / Honeycomb どのバックエンドでも動くようにする。

**依存関係**: 既存 `fav/src/otel.rs` が存在（Rune としての公開のみ）

```favnir
import runes/otel

// #[trace] アノテーションでスパンが自動生成される
#[trace(name: "load_from_db", service: "etl-pipeline")]
stage LoadFromDb: Config -> List<Order> !Db = |config| {
  // スパン開始 → 処理 → スパン終了 が自動
  bind conn   <- Postgres.connect(config.db)
  bind orders <- Postgres.query[Order](conn, "SELECT * FROM orders WHERE status = 'pending'")
  Result.ok(orders)
}

// 手動でスパンを操作することも可能
stage ManualTrace: String -> Unit !Io = |operation| {
  bind span <- OTel.start_span("custom-operation")
  bind span2 <- OTel.set_attribute(span, "operation.name", operation)
  // ... 処理 ...
  bind _ <- OTel.end_span(span2)
  Result.ok(unit)
}
```

実装内容:
- `fav/src/otel.rs` の機能を `runes/otel/` として Favnir から呼び出せる形に整理
- `OTel.start_span / set_attribute / add_event / end_span` を Rune 関数として公開
- `#[trace(name, service)]` アノテーションが `ast.rs` に追加され、コンパイラがスパンコードを自動挿入
- OTLP エクスポーター（Jaeger / Tempo / Honeycomb）への設定を `fav.toml` で切り替え可能

`cargo test otel` で 3 件以上 PASS。

---

### v28.4 — `fav profile` 強化

**テーマ**: 既存の `fav profile`（v9.9.0 で追加）を stage 別フレームグラフ出力に強化する。
どの stage がどのくらいの時間を使っているかを視覚的に確認できるようにする。

**依存関係**: v28.1（prometheus）完了後推奨（メトリクス基盤の共有）

```bash
# インタラクティブなフレームグラフ（SVG）を生成
fav profile --format flamegraph src/pipeline.fav
# → ./profile.svg（ブラウザで開いてインタラクティブに操作可能）

# 前バージョンとの比較（劣化した stage をハイライト）
fav profile --compare v26.0.0 src/pipeline.fav
# → stdout に比較レポート（劣化 stage は [SLOWER] でハイライト）

# stage ごとの breakdown テーブル
fav profile --format table src/pipeline.fav
# → stage 名 / 実行時間 / 割合 / 呼び出し回数 のテーブル出力
```

実装内容:
- `inferno` クレートを使って folded stack trace → SVG フレームグラフを生成
- `--compare <version>` で `benchmarks/vX.Y.Z.json` の stage 別データと比較
- `--format table` で stage breakdown のテキストテーブルを出力

`fav profile --format flamegraph tests/fixtures/etl.fav` が SVG を生成することを確認。

---

### v28.5 — sentry Rune 追加

**テーマ**: エラートラッキング。`#[on_error]` アノテーションを使うことで、
stage の失敗を自動的に Sentry に報告できるようにする。

**依存関係**: なし

```favnir
import runes/sentry

// stage 失敗時に自動で Sentry へ送信
#[on_error(report_to: "sentry", level: "error")]
stage ProcessPayment: PaymentRequest -> PaymentResult !Http = |req| {
  bind result <- PaymentGateway.charge(req)
  // 失敗した場合、Sentry に req の内容（PII マスク済み）が自動送信される
  Result.ok(result)
}

// 手動でエラーを送信することも可能
stage ManualReport: AppError -> Unit !Io = |err| {
  bind _ <- Sentry.capture_error(err)
  bind _ <- Sentry.set_tag("pipeline", "etl")
  Result.ok(unit)
}
```

実装する関数:

| 関数 | 内容 |
|---|---|
| `Sentry.capture_error(err)` | エラーイベント送信 |
| `Sentry.capture_message(level, msg)` | メッセージイベント送信 |
| `Sentry.set_user(id, email)` | ユーザーコンテキスト設定 |
| `Sentry.set_tag(key, value)` | タグ設定（フィルタリング用）|
| `Sentry.set_extra(key, value)` | 追加情報設定 |
| `#[on_error(report_to, level)]` | stage 失敗時の自動送信アノテーション |

`getsentry/sentry`（Docker セルフホスト）または DSN モックで `cargo test sentry` が 3 件以上 PASS。

---

### v28.6 — grafana Rune 追加

**テーマ**: ダッシュボード管理・更新 API。
パイプライン実行結果を Grafana ダッシュボードにリアルタイムで反映できるようにする。

**依存関係**: v28.1（prometheus）完了後推奨（データソースが prometheus を前提）

実装する関数:

| 関数 | 内容 |
|---|---|
| `Grafana.create_annotation(dashboard_id, text, tags)` | アノテーション作成（デプロイ記録等）|
| `Grafana.push_dashboard(json)` | ダッシュボード定義の更新 |
| `Grafana.snapshot(dashboard_id)` | スナップショット作成（共有 URL 生成）|

`grafana/grafana`（Docker）で `cargo test grafana` が 2 件以上 PASS。

---

### v28.7 — オブザーバビリティ E2E デモ（prometheus + grafana）

**テーマ**: ETL パイプラインのメトリクスを Grafana ダッシュボードにリアルタイム表示する。

**依存関係**: v28.1（prometheus）・v28.6（grafana）完了後

```favnir
// examples/observability/prometheus_grafana.fav
// #[track] を付けるだけで Grafana ダッシュボードにメトリクスが現れる

#[track(latency: true, error_rate: true)]
stage ExtractOrders: Unit -> List<RawOrder> !Db = |_| {
  // ... postgres から読み込み ...
}

#[track(latency: true)]
stage TransformOrders: List<RawOrder> -> List<Order> !Pure = |rows| {
  // ... 変換処理 ...
}

#[track(latency: true, error_rate: true)]
stage LoadToWarehouse: List<Order> -> Unit !Db = |orders| {
  // ... DWH に書き込み ...
}
```

`examples/observability/docker-compose.yml` に prometheus / grafana を定義。
`fav run examples/observability/prometheus_grafana.fav` 後、
`http://localhost:3000` で Grafana ダッシュボードにメトリクスが表示されることを確認。

---

### v28.8 — オブザーバビリティ E2E デモ（datadog APM）

**テーマ**: マイクロサービス連携パイプラインのトレースを Datadog APM で可視化する。

**依存関係**: v28.2（datadog）完了後

```favnir
// examples/observability/datadog_apm.fav
// #[trace] を付けるとサービスマップ・フレームグラフが Datadog に現れる

#[trace(service: "etl-extractor")]
stage Extract: Config -> List<Event> !Db = |config| { ... }

#[trace(service: "etl-transformer")]
stage Transform: List<Event> -> List<ProcessedEvent> !Pure = |events| { ... }

#[trace(service: "etl-loader")]
stage Load: List<ProcessedEvent> -> Unit !Db = |events| { ... }
```

Datadog Agent（ローカルモード）で実際にトレースが送信されることを確認。

---

### v28.9 — オブザーバビリティ E2E デモ（sentry アラート）

**テーマ**: パイプライン失敗時に自動的に Sentry でアラートを受け取れるようにする。

**依存関係**: v28.5（sentry）完了後

```favnir
// examples/observability/sentry_alerting.fav
// エラーが発生した stage を #[on_error] で自動通知

#[on_error(report_to: "sentry", level: "critical")]
stage CriticalLoad: Unit -> List<Order> !Db = |_| {
  // この stage が失敗したら Sentry で critical アラートが上がる
  bind conn  <- Postgres.connect(config.db)
  bind rows  <- Postgres.query[Order](conn, "SELECT * FROM critical_orders")
  Result.ok(rows)
}
```

Sentry（セルフホスト Docker）で実際にエラーイベントが受信されることを確認。

---

## v29.0 — Observability First マイルストーン宣言 ✓ COMPLETE（2026-06-28）

**完了条件:**

| コンポーネント | 完了基準 |
|---|---|
| prometheus Rune | 5 条件クリア + 4 件テスト + `#[track]` 動作確認 |
| datadog Rune | 5 条件クリア + 4 件テスト |
| OpenTelemetry Rune（otel.rs 昇格） | `#[trace]` 動作 + 3 件テスト |
| sentry Rune | 5 条件クリア + 3 件テスト + `#[on_error]` 動作確認 |
| grafana Rune | 2 件テスト |
| `#[track]` / `#[trace]` / `#[on_error]` | コンパイラが自動挿入コードを生成 |
| `fav profile --format flamegraph` | SVG 出力確認 |
| `fav profile --compare <version>` | 劣化 stage ハイライト確認 |
| E2E デモ（prometheus + grafana） | Docker Compose で動作 |
| E2E デモ（datadog APM） | トレース送信確認 |
| E2E デモ（sentry アラート） | エラー受信確認 |

**最終テスト（全件 PASS が完了条件）:**

```bash
# 1. 全 Rust テストが通る
cargo test

# 2. 可観測性 Rune テスト全件
cargo test prometheus datadog otel sentry grafana

# 3. fav profile 強化確認
fav profile --format flamegraph tests/fixtures/etl.fav

# 4. E2E デモ全 3 本
docker compose -f examples/observability/docker-compose.yml up -d
fav run examples/observability/prometheus_grafana.fav
fav run examples/observability/datadog_apm.fav
fav run examples/observability/sentry_alerting.fav
```

> 「`#[track(latency, error_rate)]` を stage に付けるだけで
>  Grafana ダッシュボードにメトリクスが現れる」
> = Observability First の完成を象徴するデモ

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v25.1-v30.0.md`
- 前フェーズ: `versions/roadmap/roadmap-v27.1-v28.0.md`
- 次フェーズ: `versions/roadmap/roadmap-v29.1-v30.0.md`
