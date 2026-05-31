# Favnir EKS E2E Demo

EKS Fargate 上で Favnir の「toolchain/runtime イメージ分離」を実証するデモ。

## 結果

```
=== Favnir EKS E2E Demo — 証跡確認 ===

[PASS] Compiler Pod: 証跡ファイルが S3 に存在する
[PASS] Compiler Pod: pipeline.fav が存在する（toolchain イメージ）
[PASS] S3: artifacts/pipeline.fvc が存在する
[PASS] Executor Pod: 証跡ファイルが S3 に存在する
[PASS] Executor Pod: .fav ファイルが 0 件（runtime イメージ）
[PASS] サマリー JSON が S3/output/ に存在する

結果: PASS=6 / FAIL=0
```

実行日: 2026-05-31

## アーキテクチャ

```
[favnir/toolchain] ──────────── Compiler Job Pod ──────────────→ S3: artifacts/pipeline.fvc
  └─ fav binary                                                    S3: proof/eks/compiler-pod-*
  └─ /app/src/pipeline.fav  (証拠: .fav が存在する)

[favnir/runtime]  ──────────── Executor Job Pod ───────────────→ S3: output/summary-latest.json
  └─ fav binary                                                    S3: proof/eks/executor-pod-*
  (証拠: .fav が 0 件)
```

### 2 イメージ分離の証明

| イメージ | `.fav` ファイル | 役割 |
|---|---|---|
| `favnir/toolchain` | `/app/src/pipeline.fav` あり | ソースのコンパイル専用 |
| `favnir/runtime` | **0 件** | バイトコード（.fvc）の実行専用 |

Executor Pod はソースコードを持たず、S3 から取得した `.fvc` バイトコードのみで `fav exec` を実行する。

## 構成

```
infra/e2e-demo/eks/
├── docker/
│   └── toolchain/Dockerfile       # fav + awscli + pipeline.fav
├── src/
│   └── pipeline.fav               # SQLite 対応パイプライン定義
├── terraform/
│   ├── main.tf                    # VPC / Subnet / VPC Endpoints
│   ├── eks.tf                     # EKS Cluster / Fargate Profiles / OIDC
│   ├── iam.tf                     # Cluster Role / Fargate Role / IRSA Roles
│   ├── storage.tf                 # S3 バケット（ECS 版と共用）
│   ├── variables.tf
│   └── outputs.tf
├── k8s/
│   ├── namespace.yaml
│   ├── serviceaccount.yaml        # IRSA アノテーション付き
│   ├── compiler-job.yaml          # toolchain イメージで fav build
│   └── executor-job.yaml          # runtime イメージで fav exec
└── scripts/
    ├── build-and-push.sh          # ECR へビルド・プッシュ
    ├── run-jobs.sh                # Kubernetes Job 実行
    └── verify.sh                  # 証跡検証（6 チェック）
```

## インフラ構成

- **EKS クラスター**: `favnir-eks-demo`（Kubernetes 1.31）
- **Fargate Profile**: `favnir-demo`（アプリ）+ `kube-system`（CoreDNS）
- **VPC**: `10.1.0.0/16`、Private Subnet x2（AZ: 1a / 1c）、NAT Gateway なし
- **VPC Endpoints**: S3 Gateway + ECR dkr/api + CloudWatch Logs + STS（各 AZ）
- **IRSA**: Compiler / Executor それぞれ最小権限の S3 ポリシー

## 実行手順

### 前提

- AWS CLI 設定済み（`ap-northeast-1`）
- Docker、kubectl、Terraform インストール済み
- `favnir-e2e-demo` S3 バケットが存在する

### 1. Docker イメージのビルドと ECR プッシュ

リポジトリルート（`favnir/`）から実行：

```bash
bash infra/e2e-demo/eks/scripts/build-and-push.sh
```

### 2. Terraform でインフラ構築

```bash
cd infra/e2e-demo/eks/terraform
terraform init
terraform apply -var="db_password=<任意のパスワード>"
```

### 3. CoreDNS の Fargate 対応（初回のみ）

Fargate-only クラスターでは CoreDNS pods を再スケジュールする必要がある：

```bash
kubectl delete pod -n kube-system -l k8s-app=kube-dns
kubectl wait --for=condition=Ready pod -l k8s-app=kube-dns -n kube-system --timeout=300s
```

### 4. Kubernetes Job 実行

```bash
cd infra/e2e-demo/eks
bash scripts/run-jobs.sh
```

### 5. 証跡検証

```bash
bash scripts/verify.sh
```

期待出力: `結果: PASS=6 / FAIL=0`

### 6. クリーンアップ

```bash
cd terraform
terraform destroy -var="db_password=dummy"
```

## 既知の制約

| 制約 | 詳細 |
|---|---|
| `fav exec` の DB 接続 | PostgreSQL client 未搭載のため SQLite を使用 |
| `fav exec` の AWS 認証 | IRSA 経由の env var は `fav exec` 内では使用できないため `aws s3 cp` で S3 書き込み |
| CoreDNS の初期化 | Fargate-only クラスターでは `kube-system` Fargate Profile と pods 再起動が必要 |
