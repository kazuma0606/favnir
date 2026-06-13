# BigQuery E2E Demo (v15.2.0)

CSV → BigQuery の 4 ステージパイプラインデモ。

## アーキテクチャ

```
[ローカル CSV] → LoadCsv → TransformRows → BigQueryInsert → QuerySummary
                                                    ↓
                                           GCP BigQuery (asia-northeast1)
```

## 前提条件

- GCP プロジェクト: `favnir-bigquery-demo`（billing 有効）
- Terraform >= 1.0
- `fav` CLI がパスに通っている

## 手順

### 1. Terraform apply（インフラ + SA キー自動生成）

```bash
cd terraform/gcp/
terraform init
terraform apply -auto-approve
```

出力例:
```
sa_key_path = "../../../../fav/tmp/gcp-sa-key.json"
dataset_id  = "favnir_demo"
table_id    = "users"
```

### 2. E2E 実行

```bash
bash scripts/run.sh favnir-bigquery-demo
```

期待出力:
```
[seed] /tmp/seed.csv を生成しました（3 件）
[1/4] LoadCsv... [1/4] OK
[2/4] TransformRows... [2/4] OK
[3/4] BigQueryInsert... [3/4] OK — 3 rows inserted
[4/4] QuerySummary... [4/4] OK — {"schema":...,"rows":[{"f":[{"v":"3"}]}]}
```

### 3. 件数確認

```bash
GCP_PROJECT_ID=favnir-bigquery-demo bash scripts/verify.sh
# PASS=1 FAIL=0
```

### 4. 後片付け

```bash
cd terraform/gcp/
terraform destroy -auto-approve
```

## ファイル構成

```
bigquery/
├── src/
│   └── demo.fav              # 4 ステージパイプライン
├── terraform/
│   └── gcp/
│       ├── main.tf           # BigQuery dataset/table + SA key
│       ├── variables.tf
│       └── outputs.tf
├── scripts/
│   ├── seed.sh               # /tmp/seed.csv 生成
│   ├── run.sh                # fav run 実行
│   └── verify.sh             # 件数確認
└── README.md
```
