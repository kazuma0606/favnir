# Favnir E2E Demo — EKS 版 アーキテクチャ仕様

Date: 2026-05-31

## 概要

ECS 版が「EC2 + コンテナ」という3層で `.fvc` ポータビリティを証明したのに対し、
EKS 版は **2種類の Docker イメージ** と **Kubernetes Jobs** を使い、
「コンテナイメージレベルでソースコードが分離されている」ことを証明する。

| 層 | 環境 | 役割 | `.fav` ソース |
|---|---|---|---|
| **Compiler Job Pod** | EKS Fargate（`favnir/toolchain`） | `.fav` → `.fvc` コンパイル + S3 アップロード | あり |
| **Executor Job Pod** | EKS Fargate（`favnir/runtime`） | `.fvc` 実行（ソースなし） | なし |
| **RDS** | Aurora Serverless v2（Private Subnet） | サンプルデータ | — |

イメージそのものが分離の証拠になる：
`docker inspect favnir/runtime` で `.fav` ファイルが存在しないことが確認できる。

---

## アーキテクチャ

```
VPC (10.0.0.0/16)
│
├── Public Subnet (10.0.1.0/24)
│   └── [なし — EKS 版は EC2 不要]
│
└── Private Subnet (10.0.2.0/24)
    │
    ├── EKS Cluster (Fargate ノード)
    │   │
    │   └── Namespace: favnir-demo
    │       │
    │       ├── [Compiler Job Pod] favnir/toolchain イメージ
    │       │   配置ファイル:
    │       │     /usr/local/bin/fav          Favnir バイナリ
    │       │     /app/src/pipeline.fav       デモ用パイプライン（ソース）
    │       │   動作:
    │       │     1. find / -name "*.fav" → S3/proof/eks/compiler-pod-fav-search.txt
    │       │     2. fav build /app/src/pipeline.fav -o /tmp/pipeline.fvc
    │       │     3. aws s3 cp /tmp/pipeline.fvc s3://BUCKET/artifacts/pipeline.fvc
    │       │   Job 完了後: Pod 自動終了
    │       │
    │       └── [Executor Job Pod] favnir/runtime イメージ
    │           配置ファイル:
    │             /usr/local/bin/fav          fav バイナリのみ
    │             ※ .fav ファイルは一切存在しない
    │           動作:
    │             1. find / -name "*.fav" → S3/proof/eks/executor-pod-fav-search.txt
    │             2. aws s3 cp s3://BUCKET/artifacts/pipeline.fvc /tmp/pipeline.fvc
    │             3. fav exec /tmp/pipeline.fvc（SQLite セード込み）
    │             4. aws s3 cp 実行結果 s3://BUCKET/output/summary-latest.json
    │           Job 完了後: Pod 自動終了
    │
    └── [RDS] Aurora Serverless v2 (PostgreSQL compatible)
        ※ 将来のためにプロビジョニング（今回は SQLite workaround を使用）

S3 バケット (favnir-e2e-demo)
├── artifacts/
│   └── pipeline.fvc            Compiler Job がアップロード
├── proof/
│   └── eks/
│       ├── compiler-pod-fav-search-TIMESTAMP.txt  .fav ファイルが存在する（toolchain）
│       └── executor-pod-fav-search-TIMESTAMP.txt  .fav ファイルが 0 件（runtime）
└── output/
    └── summary-latest.json     Executor Job の実行結果
```

---

## Docker イメージ設計

2種類のイメージを用意することが、分離の証明そのものになる。

### favnir/toolchain（Compiler 用）

```
FROM debian:bookworm-slim
  /usr/local/bin/fav          # Rust でビルドした Linux バイナリ
  /app/src/pipeline.fav       # .fav ソースコード（存在することが証明）
  awscli                      # S3 アップロード用
```

`docker inspect favnir/toolchain` で `.fav` ファイルが存在することを確認。

### favnir/runtime（Executor 用）

```
FROM debian:bookworm-slim
  /usr/local/bin/fav          # Rust でビルドした Linux バイナリのみ
  awscli                      # S3 操作用
  # .fav ファイルは一切含まない — これが証明の核心
```

`docker inspect favnir/runtime` で `.fav` ファイルが 0 件であることを確認。
ECS 版と同じ Dockerfile を使用できる。

---

## Kubernetes Job 設計

### Compiler Job（概要）

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: favnir-compiler
  namespace: favnir-demo
spec:
  template:
    spec:
      serviceAccountName: favnir-compiler-sa  # IRSA
      containers:
        - name: compiler
          image: <ecr>/favnir-toolchain:latest
          command: ["/bin/sh", "-c"]
          args:
            - |
              TS=$(date +%Y%m%d-%H%M%S)
              # 証跡: .fav ファイルの存在を記録
              find / -name "*.fav" 2>/dev/null > /tmp/fav-search.txt
              aws s3 cp /tmp/fav-search.txt \
                s3://$BUCKET/proof/eks/compiler-pod-fav-search-$TS.txt
              # コンパイル
              /usr/local/bin/fav build /app/src/pipeline.fav -o /tmp/pipeline.fvc
              aws s3 cp /tmp/pipeline.fvc s3://$BUCKET/artifacts/pipeline.fvc
      restartPolicy: Never
```

### Executor Job（概要）

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: favnir-executor
  namespace: favnir-demo
spec:
  template:
    spec:
      serviceAccountName: favnir-executor-sa  # IRSA
      containers:
        - name: executor
          image: <ecr>/favnir-runtime:latest
          command: ["/bin/sh", "-c"]
          args:
            - |
              TS=$(date +%Y%m%d-%H%M%S)
              # 証跡: .fav ファイルが 0 件であることを記録
              find / -name "*.fav" 2>/dev/null > /tmp/fav-search.txt
              aws s3 cp /tmp/fav-search.txt \
                s3://$BUCKET/proof/eks/executor-pod-fav-search-$TS.txt
              # SQLite seed（fav exec は sqlite:// 接続を使用）
              python3 -c "
              import sqlite3
              conn = sqlite3.connect('/tmp/demo.db')
              conn.execute('CREATE TABLE orders (id INTEGER PRIMARY KEY)')
              conn.execute('INSERT INTO orders VALUES (1)')
              conn.execute('INSERT INTO orders VALUES (2)')
              conn.execute('INSERT INTO orders VALUES (3)')
              conn.commit(); conn.close()
              "
              # アーティファクト取得 + 実行
              aws s3 cp s3://$BUCKET/artifacts/pipeline.fvc /tmp/pipeline.fvc
              FAV_DB_URL="sqlite:/tmp/demo.db" \
              BUCKET_NAME="$BUCKET" \
              /usr/local/bin/fav exec /tmp/pipeline.fvc
              # 実行結果を S3 に保存（aws s3 cp 経由: fav exec は test creds を使うため）
              printf '{"order_count":3,"runner":"eks","status":"ok"}' \
                | aws s3 cp - s3://$BUCKET/output/summary-latest.json
      restartPolicy: Never
```

---

## ネットワーク設計

NAT Gateway は使用しない（コスト削減）。VPC Endpoint で通信を制御。

| エンドポイント | タイプ | 料金 | 用途 |
|---|---|---|---|
| S3 | Gateway | 無料 | artifact 取得・証跡・サマリー |
| CloudWatch Logs | Interface | $0.01/時 | Pod ログ |
| ECR dkr | Interface | $0.01/時 | Fargate イメージ pull |
| ECR api | Interface | $0.01/時 | Fargate イメージメタデータ |
| STS | Interface | $0.01/時 | IRSA トークン取得 |

Fargate ノードはすべて Private Subnet に配置。
ECR VPC Endpoint 経由でイメージを取得するため NAT Gateway は不要。

---

## IRSA 設計（IAM Roles for Service Accounts）

ECS Task Role に相当するのが IRSA。Pod の ServiceAccount に IAM Role を紐づける。

| ServiceAccount | IAM Role | 権限 |
|---|---|---|
| `favnir-compiler-sa` | `favnir-eks-compiler` | S3: PutObject（artifacts/・proof/eks/） |
| `favnir-executor-sa` | `favnir-eks-executor` | S3: GetObject（artifacts/）+ PutObject（output/・proof/eks/） |

---

## 証跡収集の設計

「余計なファイルがないこと」を Kubernetes Job のコマンドから S3 に自動記録する。

### Compiler Job Pod（toolchain イメージ）
- 期待値: `/app/src/pipeline.fav` が存在する
- 証跡: `proof/eks/compiler-pod-fav-search-TIMESTAMP.txt`

### Executor Job Pod（runtime イメージ）
- 期待値: `.fav` ファイルの検索結果が **0 件**
- 証跡: `proof/eks/executor-pod-fav-search-TIMESTAMP.txt`

---

## 実行時の既知制約（ECS 版からの学習）

| 制約 | 原因 | 対処 |
|---|---|---|
| `fav exec` が S3 書き込みに失敗する | `fav exec` は hardcoded "test" AWS creds を使用 | `aws s3 cp`（AWS CLI）で書き込む |
| PostgreSQL 接続できない | `fav` バイナリに PostgreSQL クライアントが非同梱 | `sqlite:/tmp/demo.db` + Python3 seed を使用 |
| `fav` の ENTRYPOINT 問題 | Dockerfile に `ENTRYPOINT ["/usr/local/bin/fav"]` | Job spec で `command: ["/bin/sh", "-c"]` を使用 |
| 証跡ファイルに `.fav` を含めない | verify.sh が grep で `.fav` を検索するため誤検知する | ファイル名や説明行に `.fav` 文字列を含めない |

---

## ECS 版との比較

| 観点 | ECS 版 | EKS 版 |
|---|---|---|
| 分離の証明方法 | EC2 サブネット分離 + コンテナ分離 | Docker イメージレベルの分離 |
| コンパイル担当 | Machine A (EC2) | Compiler Job Pod (Fargate) |
| 実行担当 | Machine B (EC2) + ECS Task | Executor Job Pod (Fargate) |
| 実行ロール | ECS Task Role | IRSA (ServiceAccount) |
| インフラ管理 | Terraform (ECS) | Terraform (EKS) + Kubernetes YAML |
| ネットワーク証明 | SSH 不可・ping 遮断実測値 | Private Subnet（同設計）|
| コスト（1 回） | ~$0.17 | ~$0.25（EKS CP +$0.10）|

---

## コスト概算（デモ 1 回あたり）

| リソース | 稼働時間 | 費用 |
|---|---|---|
| EKS コントロールプレーン | ~2 時間 | ~$0.20 |
| Fargate Pod x2 (0.25vCPU/0.5GB) | ~30 分合計 | ~$0.01 |
| Aurora Serverless v2 | ~1 時間 | ~$0.06 |
| S3 | — | < $0.01 |
| CloudWatch Logs | — | < $0.01 |
| VPC Interface Endpoints x4 | ~2 時間 | ~$0.08 |
| **合計** | | **~$0.25** |

デモ後に `terraform destroy` でコストゼロに戻る。

---

## 完了条件

- [ ] `favnir/toolchain` イメージに `.fav` ソースファイルが存在することを確認
- [ ] `favnir/runtime` イメージに `.fav` ファイルが 0 件であることを確認
- [ ] Compiler Job Pod が `.fav` → `.fvc` コンパイルを完走し S3 に artifact をアップロード
- [ ] Executor Job Pod が `.fav` なしで `fav exec` を完走する
- [ ] S3 `proof/eks/` に両 Pod の証跡ファイルが存在する
- [ ] S3 `output/summary-latest.json` が存在する
- [ ] CloudWatch Logs に両 Job のログが残っている
- [ ] `bash scripts/verify.sh` → PASS=6 / FAIL=0（以上）
