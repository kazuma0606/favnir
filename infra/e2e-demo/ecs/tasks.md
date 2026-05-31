# Favnir E2E Demo — ECS 版 タスクリスト

Date: 2026-05-30

## Phase 1 — 事前準備

### 1-A: favnir/runtime Docker イメージ（multi-stage build）
- [ ] `docker/runtime/Dockerfile` を作成
  - Stage 1 (builder): `rust:slim-bookworm` で `cargo build --release`
  - Stage 2 (runtime): `debian:bookworm-slim` に `fav` バイナリのみコピー
  - `.fav` ファイルは一切含まない
  - `awscli` をインストール（S3 アクセス用）
  - Alpine (musl) は不使用（duckdb bundled / wasmtime の glibc 依存があるため）
- [ ] リポジトリルートから `docker build -f infra/e2e-demo/ecs/docker/runtime/Dockerfile -t favnir-runtime .` を実行
- [ ] `docker run --rm favnir-runtime find / -name "*.fav"` の出力が 0 件であることを確認
- [ ] `docker run --rm favnir-runtime fav --version` で動作確認

### 1-B: ECR リポジトリ + プッシュ + EC2 用バイナリ配置
- [ ] `scripts/build-and-push.sh` を作成
  - `docker build` でイメージをビルド
  - `docker create` + `docker cp` でバイナリを抽出 → `s3://favnir-e2e-demo/bootstrap/fav` にアップロード
  - ECR にイメージをプッシュ
- [ ] スクリプトを実行し以下を確認
  - ECR にイメージが登録されていること
  - `aws s3 ls s3://favnir-e2e-demo/bootstrap/fav` でバイナリが存在すること

---

## Phase 2 — ETL Favnir ソース作成

### 2-A: etl.fav の実装
- [ ] `src/etl.fav` を作成
  - `ExtractOrders` ステージ（postgres rune → List<Order>）
  - `Summarize` ステージ（顧客別集計、純粋関数）
  - `SaveSummary` ステージ（aws rune → S3 書き出し）
  - `seq EtlPipeline = ExtractOrders |> Summarize |> SaveSummary`
- [ ] `fav check src/etl.fav` で型チェック通過を確認

### 2-B: pipeline.fav の実装（Machine B 用デモ）
- [ ] `src/pipeline.fav` を作成
  - Machine B が `fav exec` で実行するシンプルなパイプライン
  - RDS 読み取り → サマリー → S3 書き出し
- [ ] `fav check src/pipeline.fav` で型チェック通過を確認

### 2-C: ローカルビルド確認
- [ ] `fav build src/etl.fav -o /tmp/etl.fvc` でアーティファクト生成
- [ ] `fav build src/pipeline.fav -o /tmp/pipeline.fvc` でアーティファクト生成

---

## Phase 3 — Terraform: ネットワーク

### 3-A: VPC / Subnet
- [ ] `terraform/main.tf` を作成
  - VPC (10.0.0.0/16)
  - Public Subnet (10.0.1.0/24) — Machine A 用
  - Private Subnet (10.0.2.0/24) — Machine B / ECS / RDS 用
  - Internet Gateway + Route Table (Public のみ)
  - Private Route Table

### 3-B: VPC Endpoints
- [ ] S3 Gateway Endpoint（無料）
- [ ] CloudWatch Logs Interface Endpoint
- [ ] SSM Interface Endpoint
- [ ] ECR dkr Interface Endpoint
- [ ] ECR api Interface Endpoint

### 3-C: Security Groups
- [ ] `machine_a` SG（SSH: 開発者 IP のみ、Egress: 全許可）
- [ ] `machine_b` SG（Inbound なし、Egress: 全許可）
- [ ] `ecs` SG（Inbound なし、Egress: 全許可）
- [ ] `rds` SG（Inbound: machine_b + ecs から 5432 のみ）
- [ ] `endpoints` SG（Inbound: VPC 内から 443 のみ）

---

## Phase 4 — Terraform: ストレージ / IAM

### 4-A: S3 バケット（storage.tf）
- [ ] バケット `favnir-e2e-demo` を作成
  - パブリックアクセスブロック: 全有効
  - バケットポリシー: EC2 Instance Profile / ECS Task Role のみアクセス許可
- [ ] ディレクトリ構造確認（artifacts / proof / output / logs / bootstrap）

### 4-B: IAM（iam.tf）
- [ ] EC2 Instance Profile 用 Role を作成
  - S3: GetObject / PutObject（バケット限定）
  - CloudWatch Logs: PutLogEvents
  - SSM: SSM Session Manager 用ポリシー
  - EC2: StopInstances（Machine B 自己 stop 用）
- [ ] ECS Task Execution Role を作成
  - ECR: GetAuthorizationToken / BatchGetImage / GetDownloadUrlForLayer
  - CloudWatch Logs: CreateLogStream / PutLogEvents
- [ ] ECS Task Role を作成
  - S3: GetObject / PutObject（バケット限定）
  - Secrets Manager: GetSecretValue

---

## Phase 5 — Terraform: EC2

### 5-A: Machine A（Public EC2）
- [ ] `compute.tf` に `aws_instance.machine_a` を定義
  - AMI: Ubuntu 24.04 LTS（最新）
  - Instance type: t3.micro
  - Public Subnet + Public IP
  - IAM Instance Profile 付与
  - `machine-a-userdata.sh` を template file で渡す
- [ ] `scripts/machine-a-userdata.sh` を作成
  - `fav` バイナリをインストール（S3 から取得）
  - `.fav` ソースを配置（`/app/src/`）
  - 証跡収集: `find /app -type f | sort` → S3 upload
  - `fav build` で `.fvc` 生成
  - アーティファクトを S3 にアップロード

### 5-B: Machine B（Private EC2）
- [ ] `compute.tf` に `aws_instance.machine_b` を定義
  - Private Subnet（Public IP なし）
  - SSM 経由でのみアクセス
  - `machine-b-userdata.sh` を template file で渡す
- [ ] `scripts/machine-b-userdata.sh` を作成
  - `fav` バイナリをインストール（S3 から取得）
  - `.fav` ファイルは一切インストールしない
  - 証跡収集: `find / -name "*.fav"` → S3 upload（0 件であることを記録）
  - pipeline.fvc が S3 に存在するまでポーリング待機
  - `fav exec /tmp/pipeline.fvc` を実行
  - 自己 stop

---

## Phase 6 — Terraform: ECS

### 6-A: ECS Cluster
- [ ] `ecs.tf` に `aws_ecs_cluster.demo` を定義

### 6-B: ECS Task Definition
- [ ] `aws_ecs_task_definition.etl` を定義（Fargate, awsvpc）
  - Container 1: `proof-collector`（init container 相当）
    - `find / -name "*.fav"` → S3/proof/ecs/ にアップロード
    - `ls -la /usr/local/bin/` もアップロード
  - Container 2: `etl-runner`（proof-collector の完了後に起動）
    - `aws s3 cp s3://artifacts/etl.fvc /tmp/etl.fvc`
    - `fav exec /tmp/etl.fvc`
    - Secrets Manager 経由で `DB_URL` を注入

### 6-C: CloudWatch Logs
- [ ] `/favnir/e2e-demo/ecs` ロググループを作成（保持 7 日）

---

## Phase 7 — Terraform: データベース

### 7-A: Aurora Serverless v2（database.tf）
- [ ] `aws_rds_cluster.demo` を定義
  - Engine: aurora-postgresql
  - min_capacity: 0.5、max_capacity: 1.0
  - Private Subnet Group（Machine B / ECS と同一 private subnet）
- [ ] `aws_rds_cluster_instance.demo` を定義（1 インスタンス）
- [ ] Secrets Manager にDB接続情報を保存

### 7-B: サンプルデータ投入
- [ ] `orders` テーブルを作成するマイグレーション SQL を用意
  ```sql
  CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    customer VARCHAR(100),
    amount NUMERIC(10,2),
    created_at TIMESTAMP DEFAULT NOW()
  );
  INSERT INTO orders (customer, amount) VALUES
    ('Alice', 1200.00), ('Bob', 800.50), ('Alice', 350.00),
    ('Carol', 2100.00), ('Bob', 450.75), ('Carol', 600.00);
  ```
- [ ] Machine A 経由（またはローカル VPN）でデータを投入

---

## Phase 8 — デプロイと検証

### 8-A: Terraform apply
- [ ] `terraform init` を実行
- [ ] `terraform plan` で差分確認
- [ ] `terraform apply` でリソース構築

### 8-C: Machine A の動作確認
- [ ] Machine A の `/app/src/` に `.fav` ファイルが存在することを確認
- [ ] `s3://favnir-e2e-demo/proof/machine-a/filelist-*.txt` が存在することを確認
- [ ] `s3://favnir-e2e-demo/artifacts/etl.fvc` が存在することを確認
- [ ] `s3://favnir-e2e-demo/artifacts/pipeline.fvc` が存在することを確認

### 8-D: Machine B の動作確認
- [ ] `s3://favnir-e2e-demo/proof/machine-b/fav-search-*.txt` の内容確認
  - `.fav` 検索結果が **0 件** であること
  - `/usr/local/bin/fav` のみが存在すること
- [ ] Machine B が自動 stop されていること
- [ ] CloudWatch Logs に実行ログが残っていること

### 8-E: ECS Task の動作確認
- [ ] ECS Task を手動で起動
  ```bash
  aws ecs run-task \
    --cluster favnir-e2e-demo \
    --task-definition favnir-etl \
    --launch-type FARGATE \
    --network-configuration "awsvpcConfiguration={subnets=[PRIVATE_SUBNET_ID],securityGroups=[ECS_SG_ID]}"
  ```
- [ ] `s3://favnir-e2e-demo/proof/ecs/fav-search-*.txt` の内容確認
  - `.fav` 検索結果が **0 件** であること
- [ ] `s3://favnir-e2e-demo/output/summary-*.json` が存在することを確認
- [ ] CloudWatch Logs の `/favnir/e2e-demo/ecs` にログが残っていること

---

## Phase 9 — 証跡確認（完了条件）

### 9-A: 証跡ファイルの確認
- [ ] `aws s3 ls s3://favnir-e2e-demo/proof/ --recursive` で3サーバ分の証跡が存在
- [ ] Machine A 証跡: `.fav` ファイルが `/app/src/` に存在する
- [ ] Machine B 証跡: `.fav` ファイルが **0 件**
- [ ] ECS 証跡: `.fav` ファイルが **0 件**

### 9-B: ETL 出力の確認
- [ ] `aws s3 cp s3://favnir-e2e-demo/output/summary-*.json .` でダウンロード
- [ ] 顧客別集計データが正しく出力されていること（Alice: 2件, Bob: 2件, Carol: 2件）

### 9-C: ログの確認
- [ ] CloudWatch Logs: Machine A / Machine B / ECS の全ログが残っている

---

## Phase 10 — クリーンアップ

- [ ] `terraform destroy` でリソース全削除
- [ ] S3 バケットのオブジェクトを削除（terraform が空バケット削除を要求する場合）
- [ ] ECR リポジトリのイメージを削除（コスト節約）

---

## 完了条件サマリー

| 確認項目 | 担当 | 状態 |
|---|---|---|
| Machine A: `.fav` ソースが存在 + ビルド成功 | compute.tf + machine-a-userdata.sh | - |
| Machine B: `.fav` 0 件（証跡 S3 保存済み） | compute.tf + machine-b-userdata.sh | - |
| ECS: `.fav` 0 件（証跡 S3 保存済み） | ecs.tf + proof-collector container | - |
| ECS: ETL が `.fvc` のみで完走 | etl.fav + ecs.tf | - |
| S3 `output/` にサマリー JSON が存在 | etl.fav SaveSummary | - |
| CloudWatch Logs に全ログ残存 | monitoring.tf | - |
| `terraform destroy` 後コストゼロ | - | - |
