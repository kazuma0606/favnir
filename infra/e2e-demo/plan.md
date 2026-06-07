# Favnir E2E セルフホスト実証デモ 計画書

Date: 2026-05-26

## この計画の位置づけ

`infra/e2e-demo/` 配下のシナリオは、すべてを同じ意味で「Favnir のコアテスト」として扱うものではない。

- `lambda/`, `airgap/`, `fav2py/` の一部は Favnir 近接検証
- `ecs/`, `eks/`, `snowflake/` は reference integration
- `crosscloud/` は architecture / R&D lab

分類の詳細は [classification.md](C:/Users/yoshi/favnir/infra/e2e-demo/classification.md) を参照。

この `plan.md` は「Favnir をどのような実行基盤や運用パターンに載せられるか」を整理した全体計画であり、通常の CI 品質ゲートそのものを定義する文書ではない。

## 目的

Rust コアと Favnir 処理系が「同一プロセス内に混在している」現状では、
セルフホストが本当に機能しているかが視覚的に分かりにくい。

2台の EC2 をネットワーク分離して配置することで：

- Machine A（public）: Favnir toolchain でソースをコンパイル → `.fvc` アーティファクトを生成
- Machine B（private）: `fav` バイナリのみ。ソースコードなしで `.fvc` を実行

ソースコードが物理的に存在しない環境でアーティファクトが動くことを示し、
セルフホストコンパイラが本物のポータブルな bytecode を生成していることを証明する。

加えて、Favnir がデータパイプライン言語として実用的であることを
RDS → S3 サマリー書き出しという実際のユースケースで検証する。

## 前提確認（実装調査済み）

- `fav exec <artifact.fvc>` は `driver.rs:1520` に実装済み
- Machine B に必要なのは `fav` バイナリ 1本のみ（ソースコード不要）
- アーティファクトは `FvcArtifact::from_bytes` でロードして即実行

## アーキテクチャ

```
VPC (10.0.0.0/16)
├── public subnet (10.0.1.0/24)
│   └── Machine A: t3.micro / Ubuntu 24.04 LTS
│       - Favnir toolchain（fav バイナリ + .fav ソース一式）
│       - pipeline.fav をコンパイルして pipeline.fvc を S3 にアップロード
│       - デモ完了後に手動で stop
│
└── private subnet (10.0.2.0/24)
    ├── Machine B: t3.micro / Ubuntu 24.04 LTS
    │   - fav バイナリのみ（.fav ソースコードなし）
    │   - S3 から pipeline.fvc を取得して fav exec 実行
    │   - RDS からデータ読み取り → S3 にサマリー書き出し
    │   - 成功後に自己 stop
    │
    └── RDS: Aurora Serverless v2 (PostgreSQL compatible)
        - デモ用サンプルデータを事前投入

S3 バケット
├── artifacts/pipeline.fvc        (Machine A がアップロード)
├── output/summary_<timestamp>.json (Machine B がアップロード)
└── logs/pipeline_<timestamp>.log  (Machine B がアップロード)

CloudWatch Logs
└── /favnir/e2e-demo/machine-b    (起動ログ・実行ログ)
```

## ネットワーク設計

NAT Gateway は使用しない（コスト削減）。
代わりに VPC Endpoint で private subnet からの外部通信を制御する。

| エンドポイント | タイプ | 料金 | 用途 |
|---|---|---|---|
| S3 | Gateway | 無料 | pipeline.fvc 取得・サマリー書き出し |
| CloudWatch Logs | Interface | $0.01/時 | ログ送信 |
| SSM | Interface | $0.01/時 | SSH 不要のコンソールアクセス |

Machine B は上記 VPC Endpoint 経由のみ外部通信可能。
RDS は同一 private subnet 内なので直接接続。

## 実行フロー

```
1. [事前] RDS にサンプルデータを投入

2. [Machine A] pipeline.fav を fav run でコンパイル
   fav build -o pipeline.fvc pipeline.fav
   aws s3 cp pipeline.fvc s3://your-bucket/artifacts/pipeline.fvc

3. [Machine B: user data で自動実行]
   a. S3 から pipeline.fvc を取得
   b. fav exec /tmp/pipeline.fvc  (RDS 読み取り → S3 書き出し)
   c. 実行ログを S3 + CloudWatch Logs に保存
   d. 成功後に aws ec2 stop-instances --instance-ids $(自分のID)

4. [確認] S3 の output/ にサマリーファイルが存在すること
```

## Machine B ユーザーデータスクリプト（概要）

```bash
#!/bin/bash
set -e

# CloudWatch Agent 起動
/opt/aws/amazon-cloudwatch-agent/bin/amazon-cloudwatch-agent-ctl \
  -a fetch-config -m ec2 -s -c ssm:/favnir/cloudwatch-config

LOG=/var/log/favnir-pipeline.log
exec > >(tee -a $LOG) 2>&1

echo "[$(date)] pipeline start"

# アーティファクト取得
aws s3 cp s3://YOUR_BUCKET/artifacts/pipeline.fvc /tmp/pipeline.fvc

# 実行（環境変数で RDS 接続情報を渡す）
FAV_DB_URL="postgres://..." /usr/local/bin/fav exec /tmp/pipeline.fvc

echo "[$(date)] pipeline done"

# ログを S3 に保存
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
aws s3 cp $LOG s3://YOUR_BUCKET/logs/pipeline-${TIMESTAMP}.log

# 自己 stop
INSTANCE_ID=$(curl -s http://169.254.169.254/latest/meta-data/instance-id)
aws ec2 stop-instances --instance-ids $INSTANCE_ID
```

## Terraform 構成（予定）

```
infra/e2e-demo/
├── main.tf          VPC / subnet / security group / VPC endpoints
├── compute.tf       EC2 x2 (Machine A / B)
├── database.tf      Aurora Serverless v2
├── storage.tf       S3 bucket + bucket policy
├── iam.tf           EC2 Instance Profile（S3 / CloudWatch / SSM 権限）
├── monitoring.tf    CloudWatch Logs group
└── variables.tf
```

## コスト概算（デモ 1 回あたり）

| リソース | 稼働時間 | 費用 |
|---|---|---|
| Machine A (t3.micro) | ~1 時間 | ~$0.01 |
| Machine B (t3.micro) | ~30 分 | ~$0.005 |
| Aurora Serverless v2 | ~1 時間 | ~$0.06 |
| S3 | - | < $0.01 |
| CloudWatch Logs | - | < $0.01 |
| VPC Interface Endpoints x2 | ~2 時間 | ~$0.04 |
| **合計** | | **~$0.13** |

## 実施タイミング

v7 後を推奨。理由：

- v7 でランタイムや artifact 形式が変わる可能性があり、
  その場合 pipeline.fav の書き直しが発生する
- v7 後の方が「完成形の Favnir」でデモできる

ただし、v7 がランタイムに影響しない場合は v7 前でも成立する。

## 完了条件

- [ ] S3 の `output/` にサマリーファイルが存在する
- [ ] Machine B のログに `.fav` ソースコードが一切ない状態で実行成功の記録がある
- [ ] CloudWatch Logs に起動から終了までのログが残っている
- [ ] Machine B が自動 stop されている

---

# Favnir E2E セルフホスト実証デモ — EKS 版 計画書

## 位置づけ

EC2 版（上記）が「動く証明」であるのに対し、EKS 版は
**「本番 Kubernetes 環境でも動く」** ことを示す発展デモ。

OSS 公開・正式発表のタイミングに合わせて実施することで、
エンタープライズ採用判断者へのデモとして有効。

## EC2 版との比較

| 観点 | EC2 版 | EKS 版 |
|---|---|---|
| 分離の強さ | subnet レベル | Pod レベル（namespace + cgroup） |
| 証明方法 | ネットワーク分離 | コンテナイメージで証明 |
| プロダクトストーリー | PoC | 本番運用パターン |
| スケーリング | 手動 | Pod 単位で水平スケール可能 |
| コスト（1回） | ~$0.13 | ~$0.25 |

## Docker イメージ設計

2種類のイメージを用意することが、分離の証明そのものになる。

```
favnir/toolchain          favnir/runtime
─────────────────         ────────────────
fav バイナリ              fav バイナリのみ
pipeline.fav              （ソースコードなし）
その他 .fav ファイル      数 MB の軽量イメージ
```

`docker inspect favnir/runtime` で `.fav` ファイルが存在しないことが確認できる。

## アーキテクチャ

```
EKS Cluster (Fargate ノード)
│
├── Namespace: favnir-demo
│   │
│   ├── Compiler Job Pod  [favnir/toolchain イメージ]
│   │   - pipeline.fav を fav build → pipeline.fvc を生成
│   │   - S3 にアップロードして終了
│   │
│   └── Executor Job Pod  [favnir/runtime イメージ]
│       - S3 から pipeline.fvc を取得
│       - fav exec pipeline.fvc
│       - RDS 読み取り → S3 サマリー書き出し
│       - 完了後に Pod 終了（Job として自然に終了）
│
├── private subnet: Aurora Serverless v2
└── S3 VPC Endpoint (Gateway, 無料)

S3 バケット
├── artifacts/pipeline.fvc
├── output/summary_<timestamp>.json
└── logs/
```

## Pod 間の artifact 受け渡し方式

S3 経由を採用。理由：

- 証跡が残る（S3 に `.fvc` ファイルが存在することが確認できる）
- Pod 間の直接通信が不要でシンプル
- EC2 版と同じフローを再現できる

shared PersistentVolumeClaim 方式も可能だが、証跡の観点から S3 が適切。

## Kubernetes Job 設計

```yaml
# Compiler Job（概要）
apiVersion: batch/v1
kind: Job
metadata:
  name: favnir-compiler
spec:
  template:
    spec:
      containers:
        - name: compiler
          image: favnir/toolchain:latest
          command:
            - /bin/sh
            - -c
            - |
              fav build -o /tmp/pipeline.fvc /app/pipeline.fav
              aws s3 cp /tmp/pipeline.fvc s3://BUCKET/artifacts/pipeline.fvc
      restartPolicy: Never

---
# Executor Job（概要）
apiVersion: batch/v1
kind: Job
metadata:
  name: favnir-executor
spec:
  template:
    spec:
      initContainers:
        - name: wait-for-artifact
          image: amazon/aws-cli
          command:
            - aws
            - s3
            - cp
            - s3://BUCKET/artifacts/pipeline.fvc
            - /tmp/pipeline.fvc
      containers:
        - name: executor
          image: favnir/runtime:latest
          command:
            - fav
            - exec
            - /tmp/pipeline.fvc
          env:
            - name: FAV_DB_URL
              valueFrom:
                secretKeyRef:
                  name: favnir-secrets
                  key: db-url
      restartPolicy: Never
```

## ネットワーク設計

NAT Gateway は使用しない。

| エンドポイント | タイプ | 料金 | 用途 |
|---|---|---|---|
| S3 | Gateway | 無料 | artifact + サマリー |
| CloudWatch Logs | Interface | $0.01/時 | Pod ログ |
| ECR | Interface | $0.01/時 | イメージ pull（private subnet） |

Fargate ノードは private subnet に配置し、
ECR VPC Endpoint 経由でイメージを取得する。

## Terraform 構成（予定）

```
infra/e2e-demo/
├── ec2/                     EC2 版（既存計画）
│   ├── main.tf
│   ├── compute.tf
│   ├── database.tf
│   ├── storage.tf
│   ├── iam.tf
│   ├── monitoring.tf
│   └── variables.tf
│
└── eks/                     EKS 版（追加）
    ├── main.tf              VPC / subnet / VPC endpoints（EC2 版と共有可）
    ├── eks.tf               EKS クラスター + Fargate profile
    ├── database.tf          Aurora Serverless v2（EC2 版と共有可）
    ├── storage.tf           S3 bucket（EC2 版と共有可）
    ├── iam.tf               IRSA (IAM Roles for Service Accounts)
    ├── k8s/
    │   ├── compiler-job.yaml
    │   └── executor-job.yaml
    └── variables.tf
```

## コスト概算（デモ 1 回あたり）

| リソース | 稼働時間 | 費用 |
|---|---|---|
| EKS コントロールプレーン | ~2 時間 | ~$0.20 |
| Fargate Pod x2 (0.25vCPU/0.5GB) | ~30 分合計 | ~$0.01 |
| Aurora Serverless v2 | ~1 時間 | ~$0.06 |
| S3 | - | < $0.01 |
| CloudWatch Logs | - | < $0.01 |
| VPC Interface Endpoints x2 | ~2 時間 | ~$0.04 |
| **合計** | | **~$0.25** |

デモ後にクラスター削除で維持コストゼロ。

## 実施タイミング

**EC2 版の後、OSS 公開・正式発表（v7 以降）に合わせて実施**を推奨。

理由：
- EC2 版で「動く」を証明してから EKS 版で「本番パターンでも動く」と積み上げる
- OSS 公開時のデモとして完成度が高い
- v7 後の方が artifact 形式・pipeline API が安定している

## 完了条件

- [ ] `favnir/runtime` イメージに `.fav` ソースファイルが存在しないことを確認
- [ ] Executor Job Pod が `.fav` なしで `fav exec` を完走する
- [ ] S3 の `output/` にサマリーファイルが存在する
- [ ] CloudWatch Logs に Compiler Job / Executor Job 両方のログが残っている
- [ ] デモ後にクラスターを削除してコストゼロに戻る

---

# インフラ全体ロードマップ

## 位置づけ整理

| デモ | 目的 | 対象 |
|---|---|---|
| EC2 版 E2E | セルフホスト PoC | 技術検証 |
| EKS 版 E2E | 本番 K8s パターン実証 | エンタープライズ採用判断者 |
| イベント駆動 | サーバーレスパイプライン実証 | データエンジニア全般 |

## Phase 1 — v7 後・OSS 公開前

**目標: OSS 公開時に最低限のインフラ基盤を整える**

### 1-A: GitHub Actions CI/CD

`.fav` → `.fvc` を CI でビルドして S3 / ECR に配置するパイプライン。
OSS 公開時の `README.md` に載せるために必須。

```
push to main
  └── GitHub Actions
        ├── cargo test
        ├── fav build pipeline.fav → pipeline.fvc
        └── aws s3 cp pipeline.fvc s3://BUCKET/artifacts/
```

```
infra/ci/
└── .github/workflows/
    ├── test.yml        cargo test
    └── build.yml       .fvc artifact ビルド + S3 アップロード
```

### 1-B: Secrets Manager 統合

EC2 / EKS / Lambda どのデモでも DB 接続情報が必要。
先に整備しておくことで後続デモの実装が共通化できる。

```
AWS Secrets Manager
├── /favnir/demo/db-url       RDS 接続文字列
└── /favnir/demo/aws-region   リージョン設定

EC2: IAM Instance Profile → Secrets Manager GetSecretValue
EKS: IRSA (IAM Roles for Service Accounts) → 同上
Lambda: 実行ロール → 同上
```

```
infra/secrets/
├── main.tf       Secrets Manager リソース定義
└── iam.tf        各サービス用アクセスポリシー
```

## Phase 2 — EC2 / EKS デモと同時（v7 後）

**目標: E2E デモにスケジューラーを組み合わせて DE ユースケースを完成させる**

### 2-A: EC2 版 E2E デモ

→ 本書「EC2 版」参照

### 2-B: EventBridge Scheduler

「毎晩 2 時にパイプライン実行」はデータエンジニアが最もよく使うパターン。
EC2/EKS デモのバッチ版として自然に組み合わさる。

```
EventBridge Scheduler (cron: 0 2 * * ? *)
  └── Lambda または ECS Task 起動
        └── fav exec pipeline.fvc
              └── RDS → S3 サマリー
```

```
infra/scheduler/
├── main.tf       EventBridge Scheduler + ターゲット設定
└── iam.tf        スケジューラー実行ロール
```

コスト: EventBridge Scheduler は月 14,000,000 回まで無料枠あり。

### 2-C: EKS 版 E2E デモ

→ 本書「EKS 版」参照

## Phase 3 — OSS 公開後

**目標: 公開後の追加デモとしてイベント駆動・ストリーミングを段階的に追加**

### 3-A: SQS + Lambda イベント駆動

rune-registry と同じ Lambda custom runtime 基盤で作れるため工数が少ない。
「データ到着を検知してパイプラインを起動する」ユースケースを実証する。

```
データ投入 → SQS キュー → Lambda (fav exec pipeline.fvc)
                                  └── RDS 更新 / S3 書き出し
```

```
infra/event-driven/
├── main.tf       SQS キュー + Lambda 関数 + イベントソースマッピング
├── iam.tf        Lambda 実行ロール（SQS / S3 / RDS 権限）
└── lambda/
    └── bootstrap  fav バイナリ（Lambda custom runtime エントリポイント）
```

コスト概算（デモ 1 回）:
- SQS: 最初の 100 万リクエスト/月は無料
- Lambda: 最初の 100 万リクエスト/月は無料
- 実質 $0

### 3-B: Kinesis Streams（検討候補）

リアルタイムストリーミング処理の実証。
3-A が完成してからニーズがあれば追加する。

```
Producer → Kinesis Data Stream → Lambda (fav exec stream-processor.fvc)
                                         └── リアルタイム集計 → S3
```

難易度・コストが高いため、OSS 公開後にユーザーの反応を見てから判断。

## 全体タイムライン

```
現在
 │
 ├── v7 開発中
 │
 ├── v7 完了
 │    ├── Phase 1-A: GitHub Actions CI/CD
 │    └── Phase 1-B: Secrets Manager 統合
 │
 ├── OSS 公開準備
 │    ├── Phase 2-A: EC2 版 E2E デモ
 │    ├── Phase 2-B: EventBridge Scheduler
 │    └── Phase 2-C: EKS 版 E2E デモ
 │
 └── OSS 公開後
      ├── Phase 3-A: SQS + Lambda イベント駆動
      └── Phase 3-B: Kinesis Streams（要検討）

```

## Terraform ディレクトリ全体構成（予定）

```
infra/
├── registry/               既存: rune-registry Lambda
├── site/                   既存: リファレンスサイト S3 + CloudFront
├── e2e-demo/
│   ├── ec2/                Phase 2-A: EC2 版 E2E
│   └── eks/                Phase 2-C: EKS 版 E2E
├── ci/                     Phase 1-A: GitHub Actions
├── secrets/                Phase 1-B: Secrets Manager
├── scheduler/              Phase 2-B: EventBridge Scheduler
└── event-driven/           Phase 3-A: SQS + Lambda
```
