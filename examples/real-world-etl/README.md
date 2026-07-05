# real-world-etl

Favnir で構築した実案件規模の ETL パイプライン。
S3 から CSV をロードし、バリデーションを行い、Postgres / BigQuery に書き込み、
Slack に通知し、OTel でトレースを記録する 5 ステージ構成のパイプラインです。

## 処理フロー

```
LoadCsv      CSV ファイルを読み込み Order 型にマッピング
    |> Validate      欠損値・範囲・重複チェック
    |> WritePostgres 有効な注文を Postgres に INSERT
    |> SyncBigQuery  BigQuery にも同期
    |> Notify        処理結果を Slack に通知（OTel でトレース記録）
```

## ファイル構成

```
examples/real-world-etl/
├── fav.toml                  プロジェクト定義
├── src/
│   ├── types.fav             Order / OrderStatus / ValidationError / LoadResult 型
│   ├── validators.fav        ビジネスルールバリデーション
│   ├── stages.fav            load_csv / write_postgres / sync_bigquery
│   ├── notifications.fav     Slack 通知
│   └── main.fav              RealWorldEtl pipeline + エントリポイント
├── data/
│   └── orders_sample.csv     サンプルデータ（5 行）
└── README.md
```

## 30 分で動かす手順

### 前提

- Favnir CLI がインストール済みであること（`fav --version` で確認）
- Docker が起動済みであること

### Step 1: Postgres を起動（約 2 分）

```bash
docker run --rm -d \
  --name favnir-pg \
  -e POSTGRES_PASSWORD=password \
  -p 5432:5432 \
  postgres:16-alpine

# テーブルを作成
docker exec -i favnir-pg psql -U postgres <<'SQL'
CREATE TABLE IF NOT EXISTS orders (
    order_id TEXT PRIMARY KEY,
    customer TEXT,
    product  TEXT,
    quantity INT,
    price    NUMERIC
);
SQL
```

### Step 2: 環境変数を設定（約 1 分）

```bash
export DATABASE_URL="postgres://postgres:password@localhost:5432/postgres"
export SLACK_WEBHOOK_URL="https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4318"
```

### Step 3: 型チェックを実行（約 1 分）

```bash
fav check src/main.fav
# Expected: No errors
```

### Step 4: パイプラインを実行（約 2 分）

```bash
fav run src/main.fav
```

実行結果:

```
[LoadCsv]      Loaded 5 rows from data/orders_sample.csv
[Validate]     Valid: 5, Errors: 0
[WritePostgres] Inserted 5 rows
[SyncBigQuery] Synced 5 rows
[Notify]       Slack notification sent to #data-pipeline
```

### Step 5: 結果を確認（約 1 分）

```bash
docker exec -i favnir-pg psql -U postgres -c "SELECT * FROM orders;"
```

### 合計: 約 7 分（環境構築済みなら 2 分）

実データ（10,000 行以上）で試す場合は `data/orders_sample.csv` を差し替えてください。
CSV フォーマット: `order_id,customer,product,quantity,price,status,created_at`

## テスト

```bash
fav test
```

## 本番デプロイ

AWS Lambda へのデプロイ:

```bash
fav build --target native
# 生成されたバイナリを Lambda にアップロード
```

Lambda 関数には以下の環境変数を設定してください:
- `DATABASE_URL`
- `SLACK_WEBHOOK_URL`
- `OTEL_EXPORTER_OTLP_ENDPOINT`
