# Favnir E2E Demo — Airgap 版 アーキテクチャ仕様

Date: 2026-06-05

## 概要

ECS/EKS/Lambda/Snowflake に続く第5のデモ。

**テーマ: IGW なし閉域 EC2 × バイナリドロップ × 決済データ品質チェック ETL**

- IGW・NAT Gateway を持たない完全閉域の Private EC2 に Favnir バイナリを S3 から配置
- システムパスを変更せず `/tmp/fav` として実行（環境無改変の証明）
- 103行の決済 CSV（不良11件混入）を ETL パイプラインで処理
- ドロップ率・理由別件数をログに出力しながら集計
- 証跡を S3 に保存後、EC2 を terraform destroy で後片付け

---

## アーキテクチャ

```
[ローカル / CI]
  scripts/upload.sh
    → s3://favnir-e2e-demo/airgap/binary/fav         (Favnir バイナリ)
    → s3://favnir-e2e-demo/airgap/src/analyze.fav     (パイプライン定義)
    → s3://favnir-e2e-demo/airgap/data/txn_jan.csv    (決済CSV 38行)
    → s3://favnir-e2e-demo/airgap/data/txn_feb.csv    (決済CSV 35行)
    → s3://favnir-e2e-demo/airgap/data/txn_mar.csv    (決済CSV 30行)

        ↓ S3 Gateway Endpoint（IGW なし・無料）

[EC2: Private Subnet — no IGW, no NAT]
  user_data.sh:
    1. aws s3 cp .../binary/fav /tmp/fav && chmod +x /tmp/fav
    2. aws s3 cp .../src/analyze.fav /tmp/analyze.fav
    3. aws s3 cp --recursive .../data/ /tmp/data/
    4. 証跡A: which fav → "not found"（システム未汚染）
    5. /tmp/fav run /tmp/analyze.fav /tmp/data/
       → [INFO] Loaded N rows
       → [WARN] Row XX: reason — skipped
       → [INFO] Valid: 92/103 (89.3%) | Dropped: 11/103 (10.7%)
       → [INFO] Drop reasons: empty_amount=3, negative_amount=2, ...
       → output/summary.json → S3 upload
    6. 証跡B: proof-<timestamp>.txt → S3 upload

        ↓ S3 Gateway Endpoint

[S3: favnir-e2e-demo]
  airgap/output/summary.json   ← ETL 集計結果
  airgap/proof/proof-*.txt     ← 実行証跡（which fav / ログ全文）

[scripts/verify.sh]
  → 証跡チェック 5件
  → PASS=5/FAIL=0 確認後 terraform destroy（EC2 後片付け）
```

---

## CSV スキーマ

```
transaction_id,merchant_id,merchant_name,category,amount,currency,status,timestamp,customer_id,region
```

| フィールド | 型 | 説明 |
|---|---|---|
| transaction_id | String | TXN-YYYYMM-NNNN 形式 |
| merchant_id | String | MRC-NNNN 形式 |
| merchant_name | String | 店舗名 |
| category | String | food/travel/retail/entertainment/utility |
| amount | Float | 決済金額（JPY） |
| currency | String | JPY 固定 |
| status | String | completed/pending/failed |
| timestamp | String | YYYY-MM-DDTHH:MM:SSZ |
| customer_id | String | CUS-NNNN 形式 |
| region | String | tokyo/osaka/nagoya/fukuoka/sapporo（空欄あり） |

### 不良データ 4 種

| reason | 条件 | 件数 |
|---|---|---|
| `empty_amount` | amount = 0.0 | 3件 |
| `negative_amount` | amount < 0.0 | 2件 |
| `below_threshold` | 0.0 < amount < 10.0 | 3件 |
| `empty_region` | region = "" | 3件 |

合計 103 行、不良 11 件（有効率 89.3%）

---

## Favnir パイプライン設計

```favnir
// src/analyze.fav

type TxnRow = {
  transaction_id: String  merchant_id: String  merchant_name: String
  category: String  amount: Float  currency: String
  status: String  timestamp: String  customer_id: String  region: String
}

type DropLog  = { row_num: Int  file: String  reason: String  amount: Float }
type Summary  = { region: String  category: String  total: Float  count: Int }

// Stage 1: CSV ロード（!IO）
stage LoadAll: List<String> -> List<TxnRow> !IO

// Stage 2: バリデーション — 不良行を除外しログ収集
stage Validate: List<TxnRow> -> List<TxnRow> !IO
  → [WARN] 各不良行をログ出力
  → [INFO] Valid: N/M (X%) | Dropped: K/M (Y%)
  → [INFO] Drop reasons: empty_amount=A, negative_amount=B, ...

// Stage 3: 地域×カテゴリで集計
stage Aggregate: List<TxnRow> -> List<Summary>

// Stage 4: S3 に結果を書き込み（!IO !AWS）
stage WriteOutput: List<Summary> -> Unit !IO !AWS

seq AnalyzePipeline = LoadAll |> Validate |> Aggregate |> WriteOutput
```

---

## ネットワーク設計

```
VPC (10.3.0.0/16) — 新規作成
│
├── Private Subnet A (10.3.1.0/24, ap-northeast-1a)
│   └── EC2: t3.small, Amazon Linux 2023
│       Instance Profile: S3 read/write + SSM
│
└── VPC Endpoints
    ├── S3 Gateway (無料) ← binary/CSV/output の送受信
    ├── SSM Interface      ← Session Manager でバスチョンレスアクセス
    ├── SSMMessages Interface
    └── EC2Messages Interface
```

**重要**: IGW なし・NAT Gateway なし。EC2 はインターネットに一切到達できない。
S3 のみ Gateway Endpoint 経由でアクセス可能。

---

## IAM 設計

### EC2 Instance Profile
- `s3:GetObject` — `airgap/binary/*`, `airgap/src/*`, `airgap/data/*`
- `s3:PutObject` — `airgap/output/*`, `airgap/proof/*`
- SSM 用マネージドポリシー: `AmazonSSMManagedInstanceCore`

---

## 証跡設計（verify.sh チェック項目 5 件）

| # | チェック | 確認方法 |
|---|---|---|
| 1 | proof ファイルが S3 に存在する | `aws s3 ls airgap/proof/` |
| 2 | `which fav` → `not found`（システム未汚染） | proof ファイル内容 grep |
| 3 | ドロップ率ログが存在する（`Dropped:` 行） | proof ファイル内容 grep |
| 4 | `airgap/output/summary.json` が S3 に存在 | `aws s3 ls` |
| 5 | EC2 インスタンスが terminated | `aws ec2 describe-instances` |

---

## 他デモとの比較

| 観点 | ECS | EKS | Lambda | Snowflake | Airgap（本デモ） |
|---|---|---|---|---|---|
| 実行環境 | Fargate | K8s Job | サーバーレス | ローカル→SF | **完全閉域 EC2** |
| バイナリ配布 | Docker イメージ | Docker イメージ | Docker イメージ | インストール済み | **S3 → /tmp/** |
| 環境改変 | コンテナ内 | コンテナ内 | コンテナ内 | なし | **なし（/tmp のみ）** |
| データ品質チェック | なし | なし | なし | なし | **あり（4種・ログ付き）** |
| インターネット | あり | あり | あり（VPC in） | あり | **なし** |
| 後片付け | terraform destroy | terraform destroy | terraform destroy | terraform destroy | **terraform destroy** |

---

## コスト概算（デモ 1 回あたり）

| リソース | 備考 | 概算 |
|---|---|---|
| EC2 t3.small | ~10分起動 | ~$0.003 |
| S3 | 既存バケット、数 MB | < $0.01 |
| VPC Endpoints SSM x3 | Interface、~10分 | ~$0.002 |
| S3 Gateway | 無料 | $0 |
| **合計** | | **< $0.02** |
