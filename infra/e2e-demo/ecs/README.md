# Favnir E2E Demo — ECS 版

セルフホストコンパイラが本物のポータブルなバイトコードを生成していることを
3層の物理分離で証明するデモ。

| サーバ / コンテナ | 役割 | ソースコード |
|---|---|---|
| **Public EC2** (Machine A) | Favnir 処理系 | `.fav` あり |
| **Private EC2** (Machine B) | Rust VM のみ | `.fav` なし |
| **ECS Fargate** | ETL 実行 | `.fav` なし |

証跡（各サーバの `find / -name "*.fav"` 結果）は S3 に自動保存される。

---

## 実行結果サマリー（2026-05-31）

`bash scripts/verify.sh` の結果：**PASS=8 / FAIL=0**

### 証跡ファイル一覧

| ファイル | 内容 |
|---|---|
| `s3://favnir-e2e-demo/proof/machine-a/proof-latest.txt` | `/app/src/etl.fav`, `pipeline.fav` の存在確認・コンパイル済み `.fvc` サイズ |
| `s3://favnir-e2e-demo/proof/machine-b/proof-latest.txt` | `.fav` ファイル 0 件・`fav exec pipeline.fvc` 実行結果 |
| `s3://favnir-e2e-demo/proof/machine-b/network-isolation.txt` | ネットワーク分離の実測証跡（下記参照） |
| `s3://favnir-e2e-demo/proof/ecs/fav-search-*.txt` | ECS コンテナ上の `.fav` ファイル 0 件 |
| `s3://favnir-e2e-demo/output/report-latest.json` | Machine B の実行出力 `{"order_count":3,"runner":"machine-b"}` |
| `s3://favnir-e2e-demo/output/summary-latest.json` | ECS の実行出力 `{"order_count":3,"runner":"ecs"}` |

### ネットワーク分離の証跡（Machine B）

`proof/machine-b/network-isolation.txt` に記録。実測で確認した内容：

| 確認項目 | 結果 |
|---|---|
| パブリック IP | なし（プライベートサブネット `10.0.2.180` のみ） |
| ルートテーブル | IGW・NAT なし。VPC ローカル + S3 Gateway Endpoint のみ |
| `curl https://google.com` | **BLOCKED** |
| `ping 8.8.8.8` | **100% packet loss** |
| SG インバウンドルール | **0 件**（外部からの接続を一切受け付けない） |
| S3 アクセス（VPC Endpoint 経由） | **OK**（`.fvc` アーティファクト取得のみ可能） |

`.fav` ソースが Machine B に届く経路は構造的に存在しない。

---

## 実行手順

### 前提条件

- AWS CLI 設定済み（`aws configure`）
- Docker インストール済み
- Terraform インストール済み
- EC2 Key Pair 作成済み

### Step 1 — Docker イメージをビルドして ECR にプッシュ

```bash
# リポジトリルートから実行
cd C:\Users\yoshi\favnir
bash infra/e2e-demo/ecs/scripts/build-and-push.sh
```

完了すると以下が出力される：
- `ECR image : <account>.dkr.ecr.ap-northeast-1.amazonaws.com/favnir-runtime:latest`
- `EC2 binary: s3://favnir-e2e-demo/bootstrap/fav`

### Step 2 — Terraform でインフラを構築

```bash
cd infra/e2e-demo/ecs/terraform
terraform init
terraform apply \
  -var="my_ip_cidr=$(curl -s ifconfig.me)/32" \
  -var="key_pair_name=<your-key-pair>" \
  -var="db_password=<your-password>" \
  -var="ecr_uri=$(aws sts get-caller-identity --query Account --output text).dkr.ecr.ap-northeast-1.amazonaws.com/favnir-runtime"
```

### Step 3 — RDS にサンプルデータを投入

Machine A に SSH して seed スクリプトを実行する。

```bash
# terraform output で接続情報を取得
RDS_HOST=$(terraform output -raw rds_endpoint)
MACHINE_A_IP=$(terraform output -raw machine_a_public_ip)

# Machine A に SSH
ssh -i <your-key.pem> ubuntu@$MACHINE_A_IP

# Machine A 上で実行
DB_HOST=<rds_endpoint> DB_PASS=<your-password> bash /tmp/seed-db.sh
```

### Step 4 — Machine A のビルド完了を確認

Machine A の user-data が完了すると S3 に証跡とアーティファクトが置かれる。

```bash
# アーティファクトの存在確認
aws s3 ls s3://favnir-e2e-demo/artifacts/
# → etl.fvc と pipeline.fvc が存在すること

# Machine A の証跡確認
aws s3 ls s3://favnir-e2e-demo/proof/machine-a/
```

### Step 5 — Machine B の完了を確認

Machine B は user-data で pipeline.fvc を実行後に自動 stop する。

```bash
# Machine B の証跡確認（.fav が 0 件であること）
aws s3 ls s3://favnir-e2e-demo/proof/machine-b/
```

### Step 6 — ECS Task を起動

```bash
cd infra/e2e-demo/ecs/terraform
bash ../scripts/run-ecs-task.sh
```

### Step 7 — 証跡を確認

```bash
cd infra/e2e-demo/ecs/terraform
bash ../scripts/verify.sh
```

期待される出力：
```
[PASS] Machine A: 証跡ファイル存在
[PASS] Machine A: .fav ソースファイルが /app/src/ に存在する
[PASS] Machine B: 証跡ファイル存在
[PASS] Machine B: .fav ファイルが 0 件（ソースコードなし）
[PASS] ECS: 証跡ファイル存在
[PASS] ECS: .fav ファイルが 0 件（ソースコードなし）
[PASS] サマリー JSON が S3/output/ に存在する
[PASS] レポート JSON が S3/output/ に存在する (Machine B)

結果: PASS=8 / FAIL=0
```

### Step 8 — クリーンアップ

```bash
# S3 オブジェクトを削除してから destroy
aws s3 rm s3://favnir-e2e-demo --recursive
cd infra/e2e-demo/ecs/terraform
terraform destroy
```

---

## S3 ディレクトリ構造

```
s3://favnir-e2e-demo/
├── bootstrap/
│   └── fav                              EC2 起動時に取得する fav バイナリ
├── artifacts/
│   ├── etl.fvc                          ECS が実行する ETL アーティファクト
│   └── pipeline.fvc                     Machine B が実行するアーティファクト
├── proof/
│   ├── machine-a/proof-latest.txt         /app/src/ のファイル一覧・fav バージョン
│   ├── machine-b/proof-latest.txt         .fav 検索結果（0件）・実行結果
│   ├── machine-b/network-isolation.txt    ネットワーク分離実測証跡
│   └── ecs/fav-search-TIMESTAMP.txt       .fav 検索結果（0件）
├── output/
│   ├── summary-TIMESTAMP.json           ETL 集計結果（ECS）
│   └── report-TIMESTAMP.json            パイプライン実行結果（Machine B）
└── logs/
    └── machine-b-TIMESTAMP.log          Machine B の実行ログ
```

---

## コスト概算（デモ 1 回）

| リソース | 費用 |
|---|---|
| Machine A (t3.micro, ~1h) | ~$0.01 |
| Machine B (t3.micro, ~30m) | ~$0.005 |
| ECS Fargate (0.25vCPU/0.5GB, ~15m) | ~$0.003 |
| Aurora Serverless v2 (~1h) | ~$0.06 |
| VPC Interface Endpoints x4 (~2h) | ~$0.08 |
| S3 / CloudWatch Logs | < $0.01 |
| **合計** | **~$0.17** |

デモ後は `terraform destroy` でコストゼロに戻る。
