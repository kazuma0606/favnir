# Observability E2E Demo — Prometheus + Grafana

Favnir ETL パイプラインのメトリクスを Prometheus で収集し、Grafana ダッシュボードで表示する E2E デモです。

## セットアップ

### 1. Docker Compose でインフラを起動する

`prometheus.yml` が `examples/observability/` に含まれています。
カスタムスクレイプ設定が必要な場合は編集してから起動してください。

> ⚠️ `docker-compose.yml` の `GF_SECURITY_ADMIN_PASSWORD=admin` はデモ用です。
> 本番環境では環境変数 `${GF_ADMIN_PASSWORD}` に置き換えてください。

```bash
docker compose -f examples/observability/docker-compose.yml up -d
```

起動後、以下のサービスが利用可能になります:
- Prometheus: http://localhost:9090
- Grafana: http://localhost:3000 (admin / admin)

### 2. Favnir パイプラインを実行する

```bash
fav run examples/observability/prometheus_grafana.fav
```

### 3. Grafana UI でメトリクスを確認する

ブラウザで http://localhost:3000 を開き、admin / admin でログインします。
Prometheus データソースを追加し、ダッシュボードで以下のメトリクスを確認できます:

- `rows_extracted_total` — 抽出行数カウンタ
- `transform_duration_ms` — 変換処理時間ヒストグラム
- `warehouse_rows` — ウェアハウス行数ゲージ

## 構成

| サービス | URL | 説明 |
|---|---|---|
| Prometheus | http://localhost:9090 | メトリクス収集・クエリ |
| Grafana | http://localhost:3000 | ダッシュボード表示 |

## 停止

```bash
docker compose -f examples/observability/docker-compose.yml down
```

## 注記

- `#[track]` アノテーションによる自動メトリクス挿入は v29.0+ 実装予定
- v28.7.0 時点では `Prometheus.*` / `Grafana.*` 関数を明示的に呼び出す形式
- Prometheus のスクレイプ設定（`prometheus.yml`）は別途作成が必要（v28.9+ で自動生成予定）
