# Favnir E2E Demo — Lambda 版 アーキテクチャ仕様

Date: 2026-05-31

## 概要

ECS 版（EC2/Fargate）・EKS 版（Kubernetes Job）に続く第3のデモ。

**テーマ: イベント駆動 × サーバーレス × `!Queue` エフェクト実証**

- S3 への `.fav` 投入をトリガーに Lambda A（compiler）が起動
- compiler が `.fvc` を生成して SQS に通知
- SQS トリガーで Lambda B（executor）が起動し、RDS → S3 パイプラインを実行

ECS/EKS との最大の差分は「手動起動不要・イベント駆動で完結する」点。

---

## アーキテクチャ

```
[S3: source/pipeline.fav 投入]
        ↓ S3 イベント通知（ObjectCreated）
[Lambda A: favnir-compiler]
  イメージ: ECR favnir-toolchain（コンテナ Lambda）
  処理:
    - /app/src/pipeline.fav を fav build → /tmp/pipeline.fvc
    - pipeline.fvc を S3 artifacts/ にアップロード
    - SQS に {artifact_key, timestamp} メッセージ送信
    - 証跡（find / -name "*.fav"）を S3 proof/lambda/ に保存
        ↓ SQS トリガー（EventSourceMapping）
[Lambda B: favnir-executor]
  イメージ: ECR favnir-runtime（コンテナ Lambda）
  処理:
    - SQS メッセージから artifact_key を取得
    - S3 から pipeline.fvc をダウンロード
    - FAV_DB_URL=postgres://... fav exec pipeline.fvc
      → RDS PostgreSQL (orders テーブル) を読み取り
      → AWS.s3_put_object_raw で summary-latest.json を S3 に書き込み
    - 証跡（find / -name "*.fav" → 0 件）を S3 proof/lambda/ に保存
        ↓
[S3: output/summary-latest.json]
[S3: proof/lambda/compiler-*.txt]
[S3: proof/lambda/executor-*.txt]
```

---

## Docker イメージ設計

| イメージ | ベース | 追加ファイル | Lambda エントリポイント |
|---|---|---|---|
| `favnir-toolchain` | EKS 版と同じ | `/app/src/pipeline.fav` + `awscli` | `/bin/sh` でシェルスクリプトを実行 |
| `favnir-runtime` | ECS 版と同じ | なし（`.fav` 0件） | `/bin/sh` でシェルスクリプトを実行 |

Lambda コンテナイメージは `CMD` で Lambda Runtime Interface Emulator（RIE）を使わず、
シェルスクリプトをエントリポイントとして実行する方式（`zip` 型 Lambda と同等）。

> Lambda コンテナは `ENTRYPOINT` + `CMD` で起動コマンドを指定する。
> 既存の ECS/EKS イメージに Lambda Handler 相当のシェルスクリプトを `CMD` として渡す。

---

## 2 イメージ分離の証明（ECS/EKS との連続性）

| デモ | toolchain に `.fav` | runtime に `.fav` | 実行環境 |
|---|---|---|---|
| ECS | ✓ あり | 0 件 | Fargate Task |
| EKS | ✓ あり | 0 件 | Kubernetes Job Pod |
| Lambda（本デモ） | ✓ あり | 0 件 | Lambda コンテナ |

---

## ネットワーク設計

Lambda が RDS（プライベートサブネット）に接続するため VPC 内配置が必要。

```
VPC (10.0.0.0/16) — ECS 版と同じ VPC を流用
│
├── Private Subnet (10.0.2.0/24)  — ECS 版と同じ
│   ├── Lambda A: favnir-compiler（VPC 内配置）
│   ├── Lambda B: favnir-executor（VPC 内配置）
│   └── RDS Aurora PostgreSQL（ECS 版と同じインスタンスを流用）
│
└── VPC Endpoints（ECS 版と同じ）
    ├── S3 Gateway（無料）
    ├── ECR dkr Interface
    ├── ECR api Interface
    ├── CloudWatch Logs Interface
    └── SQS Interface（新規追加）
```

**重要**: Lambda in VPC は SQS へのアクセスに SQS VPC Endpoint が必要
（NAT Gateway なしのため）。

---

## IAM 設計

### Lambda A（compiler）実行ロール
- `s3:GetObject` — `source/*`
- `s3:PutObject` — `artifacts/*`, `proof/lambda/*`
- `sqs:SendMessage` — favnir-pipeline キュー
- `logs:CreateLogGroup/Stream/PutLogEvents`
- `ec2:CreateNetworkInterface` 等（VPC 内 Lambda に必須）

### Lambda B（executor）実行ロール
- `s3:GetObject` — `artifacts/*`
- `s3:PutObject` — `output/*`, `proof/lambda/*`
- `sqs:ReceiveMessage`, `sqs:DeleteMessage`, `sqs:GetQueueAttributes`
- `logs:CreateLogGroup/Stream/PutLogEvents`
- `ec2:CreateNetworkInterface` 等

---

## SQS 設計

```
favnir-pipeline（Standard Queue）
  VisibilityTimeout: 300 秒（Lambda B のタイムアウト + 余裕）
  MessageRetentionPeriod: 3600 秒（1 時間）
  DLQ: favnir-pipeline-dlq（失敗メッセージの保管）
```

メッセージ形式（JSON）:
```json
{
  "artifact_key": "artifacts/pipeline.fvc",
  "timestamp": "20260531-030440",
  "source_key": "source/pipeline.fav"
}
```

---

## 証跡設計（verify.sh チェック項目 6 件）

| # | チェック | S3 パス |
|---|---|---|
| 1 | Compiler Lambda 証跡ファイルが存在 | `proof/lambda/compiler-pod-fav-search-*.txt` |
| 2 | Compiler 証跡に `pipeline.fav` が存在 | 同上 |
| 3 | `artifacts/pipeline.fvc` が存在 | `artifacts/pipeline.fvc` |
| 4 | Executor Lambda 証跡ファイルが存在 | `proof/lambda/executor-pod-fav-search-*.txt` |
| 5 | Executor 証跡に `.fav` が 0 件 | 同上 |
| 6 | `output/summary-latest.json` が存在 | `output/summary-latest.json` |

---

## ECS/EKS との比較

| 観点 | ECS | EKS | Lambda（本デモ） |
|---|---|---|---|
| 起動トリガー | 手動（run-task） | 手動（kubectl apply） | S3 イベント（自動） |
| コンポーネント間通信 | S3 ポーリング | S3 ポーリング | **SQS（プッシュ型）** |
| `!Queue` 実証 | なし | なし | **あり** |
| コールドスタート | なし | あり（Fargate） | あり |
| インフラ管理 | ECS クラスター | EKS クラスター | **なし（サーバーレス）** |
| RDS 接続 | ✓ | △（SQLite 代用） | ✓ |

---

## コスト概算（デモ 1 回あたり）

| リソース | 備考 | 概算 |
|---|---|---|
| Lambda x2 | 無料枠内（100万回/月） | $0 |
| SQS | 無料枠内（100万回/月） | $0 |
| RDS（ECS 版流用） | 既存インスタンス、デモ中のみ起動 | ~$0.06/h |
| S3 | 既存バケット | < $0.01 |
| VPC Endpoint SQS | Interface、~30 分 | ~$0.005 |
| CloudWatch Logs | | < $0.01 |
| **合計** | | **~$0.07** |

Lambda・SQS はほぼ無料枠内のため、ECS/EKS より低コスト。
