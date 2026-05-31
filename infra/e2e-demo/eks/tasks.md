# Favnir E2E Demo — EKS 版 タスクリスト

Date: 2026-05-31

## Phase 1 — Docker イメージ

### 1-A: favnir/runtime（ECS 版と共用）
- [x] ECS 版 `docker/runtime/Dockerfile` で既にビルド済みのイメージを確認
  - `docker run --rm --entrypoint /bin/sh favnir-runtime -c 'find / -name "*.fav" 2>/dev/null'` → 0 件
  - `docker run --rm --entrypoint /bin/sh favnir-runtime -c 'fav --version'` → バージョン表示
- [x] ECR に `favnir-runtime` リポジトリが存在することを確認

### 1-B: favnir/toolchain（EKS 版 新規）
- [x] `docker/toolchain/Dockerfile` を作成
  - Stage 1 (builder): `rust:slim-bookworm` で `cargo build --release`
  - Stage 2: `debian:bookworm-slim` に `fav` バイナリ + `awscli` + `pipeline.fav` をコピー
  - `/app/src/pipeline.fav` が存在すること（これが証明の核心）
- [x] `src/pipeline.fav` を作成（ECS 版 `src/pipeline.fav` を参考に SQLite 対応版）
- [x] ECR に `favnir-toolchain` リポジトリを作成

### 1-C: ビルド + ECR プッシュスクリプト
- [x] `scripts/build-and-push.sh` を作成
  - runtime + toolchain 両方をビルド・プッシュ
  - `docker inspect` で `.fav` 有無を確認するログを出力
- [x] スクリプトを実行して確認
  - toolchain: `find / -name "*.fav"` で `/app/src/pipeline.fav` が出力される
  - runtime: `find / -name "*.fav"` で **0 件**

---

## Phase 2 — Terraform: VPC / ネットワーク

### 2-A: VPC + Subnet
- [x] `terraform/main.tf` を作成
  - VPC (10.1.0.0/16) — ECS 版（10.0.0.0/16）と競合しないよう変更
  - Private Subnet x2（AZ: ap-northeast-1a / ap-northeast-1c）
  - EKS は複数 AZ の subnet が必要
  - Private Route Table（IGW なし）

### 2-B: VPC Endpoints（NAT Gateway 不使用）
- [x] S3 Gateway Endpoint（無料）
- [x] ECR dkr Interface Endpoint（Fargate イメージ pull 用）— 両 AZ に配置
- [x] ECR api Interface Endpoint — 両 AZ に配置
- [x] CloudWatch Logs Interface Endpoint — 両 AZ に配置
- [x] STS Interface Endpoint（IRSA トークン取得用）— 両 AZ に配置

### 2-C: Security Groups
- [x] `eks_nodes` SG（Egress 全許可）
- [x] `endpoints` SG（Inbound: VPC 内から 443 のみ）

---

## Phase 3 — Terraform: EKS クラスター

### 3-A: EKS Cluster
- [x] `terraform/eks.tf` に `aws_eks_cluster.demo` を定義
  - Kubernetes version: 1.31
  - Public + Private Endpoint（`endpoint_public_access = true`）
  - Private Subnet x2 を指定

### 3-B: Fargate Profile
- [x] `aws_eks_fargate_profile.demo` を定義（`favnir-demo` namespace）
- [x] `aws_eks_fargate_profile.kube_system` を定義（`kube-system` namespace — CoreDNS 用）
  - Pod Execution Role を付与
  - Private Subnet x2 を指定

### 3-C: OIDC Provider（IRSA 前提）
- [x] `data.tls_certificate.eks` で EKS OIDC エンドポイントの証明書を取得
- [x] `aws_iam_openid_connect_provider.eks` を作成

### 3-D: CloudWatch Logs
- [x] `/favnir/e2e-demo/eks` ロググループを作成（保持 7 日）

---

## Phase 4 — Terraform: IAM / IRSA

### 4-A: EKS クラスターロール
- [x] `aws_iam_role.eks_cluster` を作成
  - Assume Policy: `eks.amazonaws.com`
  - Attach: `AmazonEKSClusterPolicy`

### 4-B: Fargate 実行ロール
- [x] `aws_iam_role.fargate_execution` を作成
  - Assume Policy: `eks-fargate-pods.amazonaws.com`
  - Attach: `AmazonEKSFargatePodExecutionRolePolicy`

### 4-C: IRSA — Compiler Job 用
- [x] `aws_iam_role.eks_compiler` を作成
  - Assume Policy: OIDC + `system:serviceaccount:favnir-demo:favnir-compiler-sa`
  - S3 PutObject: `artifacts/*` / `proof/eks/*`

### 4-D: IRSA — Executor Job 用
- [x] `aws_iam_role.eks_executor` を作成
  - Assume Policy: OIDC + `system:serviceaccount:favnir-demo:favnir-executor-sa`
  - S3 GetObject: `artifacts/*`
  - S3 PutObject: `output/*` / `proof/eks/*`

---

## Phase 5 — Kubernetes マニフェスト

### 5-A: Namespace + ServiceAccount
- [x] `k8s/namespace.yaml` を作成（`favnir-demo`）
- [x] `k8s/serviceaccount.yaml` を作成
  - `favnir-compiler-sa`（annotation: `eks.amazonaws.com/role-arn`）
  - `favnir-executor-sa`（annotation: `eks.amazonaws.com/role-arn`）

### 5-B: Compiler Job
- [x] `k8s/compiler-job.yaml` を作成（favnir/toolchain イメージ）
  - 証跡収集: `find / -name "*.fav"` → S3/proof/eks/compiler-pod-fav-search-TIMESTAMP.txt
  - ビルド: `fav build /app/src/pipeline.fav -o /tmp/pipeline.fvc`
  - S3 アップロード: `aws s3 cp /tmp/pipeline.fvc s3://BUCKET/artifacts/pipeline.fvc`
  - `backoffLimit: 0`（失敗時にリトライしない）
  - `restartPolicy: Never`

### 5-C: Executor Job
- [x] `k8s/executor-job.yaml` を作成（favnir/runtime イメージ）
  - 証跡収集: `find / -name "*.fav"` → S3/proof/eks/executor-pod-fav-search-TIMESTAMP.txt
  - SQLite seed: `python3 -c "..."` で `/tmp/demo.db` を作成（orders 3件）
  - S3 ポーリング: `pipeline.fvc` が存在するまで最大 30 回 × 10 秒待機
  - 実行: `FAV_DB_URL=sqlite:/tmp/demo.db fav exec /tmp/pipeline.fvc`
  - S3 書き込み: `aws s3 cp - s3://BUCKET/output/summary-latest.json`（aws cli 経由）
  - `backoffLimit: 0`
  - `restartPolicy: Never`

---

## Phase 6 — src/pipeline.fav（SQLite 対応版）

### 6-A: pipeline.fav の作成
- [x] `src/pipeline.fav` を作成
  - ECS 版の `src/pipeline.fav` を参考に SQLite（`DB.connect` / `DB.query_raw`）を使用
  - Executor Job の `fav exec` で実行可能なことを確認

### 6-B: ローカル確認
- [x] `fav build src/pipeline.fav -o /tmp/pipeline.fvc` でビルド成功を確認（Pod 内）
- [x] 型チェック通過を確認

---

## Phase 7 — Terraform: ストレージ

### 7-A: S3 バケット（storage.tf）
- [x] ECS 版と同じ `favnir-e2e-demo` バケットを使用（既存）
  - `proof/eks/` プレフィックスに Executor / Compiler Pod の証跡を保存

---

## Phase 8 — スクリプト

### 8-A: scripts/run-jobs.sh
- [x] `scripts/run-jobs.sh` を作成
  - `aws eks update-kubeconfig` で kubeconfig を更新
  - ECR URI と IAM Role ARN をマニフェストに注入して `kubectl apply`
  - `kubectl wait --for=condition=complete job/favnir-compiler` で Compiler 完了を待機
  - Executor Job を順次起動・待機

### 8-B: scripts/verify.sh
- [x] `scripts/verify.sh` を作成（チェック項目 6 件）
  1. Compiler Pod 証跡ファイルが S3 に存在する
  2. Compiler Pod 証跡に `pipeline.fav` が存在する（toolchain イメージ確認）
  3. `artifacts/pipeline.fvc` が S3 に存在する
  4. Executor Pod 証跡ファイルが S3 に存在する
  5. Executor Pod 証跡に `.fav` ファイルが **0 件**（runtime イメージ確認）
  6. `output/summary-latest.json` が S3 に存在する

---

## Phase 9 — デプロイと検証

### 9-A: Terraform apply
- [x] `terraform init`
- [x] `terraform apply` — 合計 29 リソース作成
  - EKS クラスター、Fargate Profile x2（favnir-demo + kube-system）
  - VPC + Subnet x2 + VPC Endpoints（S3/ECR/CWLogs/STS）各 AZ
  - IAM Roles（cluster / fargate execution / compiler IRSA / executor IRSA）

### 9-B: CoreDNS の Fargate 対応
- [x] Fargate-only クラスターでは `kube-system` Fargate Profile が必要
- [x] CoreDNS pods delete + 再スケジュール → Running

### 9-C: Kubernetes Job 実行
- [x] `bash scripts/run-jobs.sh` を実行
- [x] Compiler Job Completed
  - `pipeline.fav` 証跡 → S3
  - `pipeline.fvc`（1914 bytes）→ S3
- [x] Executor Job Completed
  - `.fav` 0 件証跡 → S3
  - SQLite seed 3 rows
  - `fav exec` 完走
  - `summary-latest.json` → S3

### 9-D: 証跡確認
- [x] `bash scripts/verify.sh` → **PASS=6 / FAIL=0** ✓

---

## Phase 10 — README + クリーンアップ

### 10-A: README.md の作成
- [x] `infra/e2e-demo/eks/README.md` を作成
  - 実行結果サマリー（PASS=6/FAIL=0）
  - 証跡ファイル一覧
  - 実行手順

### 10-B: クリーンアップ（任意）
- [ ] S3 証跡オブジェクト（proof/eks/）を削除
- [ ] `terraform destroy` でリソース全削除
- [ ] ECR リポジトリ（favnir-toolchain）のイメージを削除（コスト節約）

---

## 完了条件サマリー

| 確認項目 | 担当 | 状態 |
|---|---|---|
| toolchain イメージに `.fav` が存在 | docker/toolchain/Dockerfile | ✓ PASS |
| runtime イメージに `.fav` が 0 件 | docker/runtime/Dockerfile（ECS 版流用） | ✓ PASS |
| Compiler Pod: `.fav` → `.fvc` ビルド成功 | compiler-job.yaml | ✓ PASS |
| Executor Pod: `.fav` なしで `fav exec` 完走 | executor-job.yaml | ✓ PASS |
| Executor Pod 証跡: `.fav` が 0 件（S3 保存済み） | executor-job.yaml | ✓ PASS |
| S3 `output/summary-latest.json` 存在 | executor-job.yaml | ✓ PASS |
| `bash scripts/verify.sh` → PASS=6/FAIL=0 | verify.sh | ✓ PASS |
| `terraform destroy` 後コストゼロ | — | 未実施（任意） |
