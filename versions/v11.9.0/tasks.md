# Favnir v11.9.0 Tasks

Date: 2026-06-06
Theme: fav2py E2E インフラ (`infra/e2e-demo/fav2py/`)

---

## Phase A — ディレクトリ構築

- [x] A-1: `infra/e2e-demo/fav2py/` 作成
- [x] A-2: `infra/e2e-demo/fav2py/src/` 作成
- [x] A-3: `infra/e2e-demo/fav2py/terraform/` 作成
- [x] A-4: `infra/e2e-demo/fav2py/scripts/` 作成

---

## Phase B — src/pipeline.fav + sample.csv

- [x] B-1: `infra/e2e-demo/fav2py/src/pipeline.fav` 作成
  - `type TxnRow` / `type SummaryRow` 定義
  - `stage LoadAndInsert: String -> Int !IO !Postgres`
  - `stage Aggregate: Int -> List<SummaryRow> !Postgres`
  - `stage SaveResult: List<SummaryRow> -> Unit !IO !AWS`
  - `seq Pipeline = LoadAndInsert |> Aggregate |> SaveResult`
- [x] B-2: `infra/e2e-demo/fav2py/src/sample.csv` 作成（103 行: region × category × amount）

---

## Phase C — terraform/

- [x] C-1: `infra/e2e-demo/fav2py/terraform/main.tf` 作成
  - VPC (10.0.0.0/16) + Public/Private Subnet + IGW + NAT Gateway
  - RDS PostgreSQL db.t3.micro (`fav2py` DB)
  - ECR リポジトリ (`favnir/fav2py`)
  - ECS Cluster + タスク定義 x2 (`fav-native` / `fav-python`)
  - Security Groups (RDS: ECS からの 5432, ECS: outbound 443/5432)
- [x] C-2: `infra/e2e-demo/fav2py/terraform/iam.tf` 作成
  - ECS 実行ロール（ECR pull / CloudWatch Logs）
  - ECS タスクロール（S3 書き込み）
- [x] C-3: `infra/e2e-demo/fav2py/terraform/variables.tf` 作成
  - `aws_region` / `db_password` / `s3_bucket`
- [x] C-4: `infra/e2e-demo/fav2py/terraform/outputs.tf` 作成
  - `rds_endpoint` / `ecr_repository` / `ecs_cluster_arn`
  - `native_task_def` / `python_task_def`
  - `private_subnet_id` / `ecs_security_group_id`

---

## Phase D — scripts/

- [x] D-1: `infra/e2e-demo/fav2py/scripts/upload.sh` 作成
  - Docker build + ECR push + S3 source upload
- [x] D-2: `infra/e2e-demo/fav2py/scripts/run.sh` 作成
  - terraform apply → ECS タスク x2 起動 → 待機 → verify.sh 呼び出し
  - PASS/FAIL カウント + exit code
- [x] D-3: `infra/e2e-demo/fav2py/scripts/verify.sh` 作成
  - S3 最新 2 件取得 → `jq` で region/category/total を比較 → PASS/FAIL
- [x] D-4: 3 スクリプトに実行権限設定（`chmod +x`）

---

## Phase E — Dockerfile

- [x] E-1: `infra/e2e-demo/fav2py/Dockerfile` 作成
  - Ubuntu 22.04 + uv + psycopg2-binary + fav binary + src/

---

## Phase F — README.md

- [x] F-1: `infra/e2e-demo/fav2py/README.md` 作成
  - 事前条件（AWS 認証・DB_PASSWORD 設定）
  - upload.sh → run.sh の実行手順
  - verify.sh による PASS/FAIL 確認方法
  - 期待結果（PASS=5 以上）

---

## Phase G — tasks.md（デモ専用）

- [x] G-1: `infra/e2e-demo/fav2py/tasks.md` 作成（デモ実行チェックリスト）

---

## Phase H — Rust テスト（2 件）

- [x] H-1: `driver.rs` に `v11900_tests` モジュール追加
  - [x] `fav2py_e2e_demo_structure` — 10 ファイルの存在確認
  - [x] `fav2py_pipeline_fav_transpiles` — `pipeline.fav` がパースできる
- [x] H-2: `cargo test v11900` — 2 件通過
- [x] H-3: `cargo test --lib` — 705 件以上通過

---

## Phase I — バージョン更新 + コミット

- [x] I-1: `fav/Cargo.toml` version → `"11.9.0"`
- [x] I-2: `cargo build` で `Cargo.lock` 更新
- [ ] I-3: `git commit & push` — CI 確認

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `infra/e2e-demo/fav2py/` ディレクトリ構造完成 | |
| `pipeline.fav` に LoadAndInsert / Aggregate / SaveResult stage 実装 | |
| Terraform で VPC / RDS / ECS Fargate x2 / ECR 定義 | |
| `upload.sh` / `run.sh` / `verify.sh` スクリプト完成 | |
| `Dockerfile` — fav + uv + psycopg2 イメージ定義 | |
| `cargo test v11900` 2 件通過 | |
| `cargo test --lib` 705 件以上通過 | |
