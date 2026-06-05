# Favnir E2E Demo — Airgap 版 実装計画

Date: 2026-06-05

## 実装フェーズ

### Phase 1 — CSV データ作成
- `src/txn_jan.csv`（38行、不良4件）
- `src/txn_feb.csv`（35行、不良4件）
- `src/txn_mar.csv`（30行、不良3件）

### Phase 2 — Favnir パイプライン
- `src/analyze.fav`
  - `TxnRow` / `DropLog` / `Summary` 型定義
  - `LoadAll` / `Validate` / `Aggregate` / `WriteOutput` ステージ
  - `AnalyzePipeline` seq

### Phase 3 — Terraform: VPC・ネットワーク
- `terraform/main.tf`
  - VPC (10.3.0.0/16)、Private Subnet (10.3.1.0/24)
  - Route Table（S3 Gateway Endpoint ルート）
  - Security Groups（EC2 / VPC Endpoints）
  - VPC Endpoints: S3 Gateway + SSM Interface x3

### Phase 4 — Terraform: IAM
- `terraform/iam.tf`
  - EC2 Instance Profile（S3 read/write + SSM）

### Phase 5 — Terraform: EC2
- `terraform/ec2.tf`
  - `aws_instance.favnir_ec2`（t3.small, AL2023）
  - `user_data.sh` テンプレート（バイナリ取得→実行→証跡→後片付け）

### Phase 6 — Terraform: 変数・出力
- `terraform/variables.tf`
- `terraform/outputs.tf`

### Phase 7 — スクリプト
- `scripts/upload.sh` — バイナリ + ソース + CSV を S3 にアップロード
- `scripts/run.sh` — EC2 起動トリガー（user_data 経由、起動確認）
- `scripts/verify.sh` — 証跡確認（5件）+ terraform destroy

### Phase 8 — 動作確認・README
- terraform apply → upload.sh → run.sh → verify.sh
- PASS=5/FAIL=0 確認
- `README.md` 作成

## ファイル構成

```
infra/e2e-demo/airgap/
├── spec.md
├── plan.md
├── tasks.md
├── README.md               (Phase 8 で作成)
├── src/
│   ├── analyze.fav
│   ├── txn_jan.csv
│   ├── txn_feb.csv
│   └── txn_mar.csv
├── terraform/
│   ├── main.tf             (VPC / Subnet / SG / Endpoints)
│   ├── ec2.tf              (Instance + user_data)
│   ├── iam.tf              (Instance Profile)
│   ├── variables.tf
│   └── outputs.tf
└── scripts/
    ├── upload.sh
    ├── run.sh
    └── verify.sh
```
