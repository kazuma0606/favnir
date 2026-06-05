# Favnir E2E Demo — Airgap 版 タスクリスト

Date: 2026-06-05

---

## Phase 1 — CSV データ作成

- [ ] 1-A: `src/txn_jan.csv` を作成（38行、不良4件）
  - 不良: empty_amount x1, negative_amount x1, below_threshold x1, empty_region x1
- [ ] 1-B: `src/txn_feb.csv` を作成（35行、不良4件）
  - 不良: empty_amount x1, negative_amount x1, below_threshold x1, empty_region x1
- [ ] 1-C: `src/txn_mar.csv` を作成（30行、不良3件）
  - 不良: empty_amount x1, below_threshold x1, empty_region x1

---

## Phase 2 — Favnir パイプライン

- [ ] 2-A: `src/analyze.fav` を作成
  - `TxnRow` / `DropLog` / `Summary` / `Stats` 型定義
  - `load_csv_file(path)` ヘルパー
  - `validate_row(row, idx, file)` — 4種バリデーション
  - `stage LoadAll: List<String> -> List<TxnRow> !IO`
  - `stage Validate: List<TxnRow> -> List<TxnRow> !IO` — [WARN]/[INFO] ログ付き
  - `stage Aggregate: List<TxnRow> -> List<Summary>` — region×category 集計
  - `stage WriteOutput: List<Summary> -> Unit !IO !AWS` — S3 書き込み
  - `seq AnalyzePipeline = LoadAll |> Validate |> Aggregate |> WriteOutput`

---

## Phase 3 — Terraform: VPC・ネットワーク

- [ ] 3-A: `terraform/main.tf` を作成
  - provider "aws" (ap-northeast-1)
  - `aws_vpc` (10.3.0.0/16, DNS enabled)
  - `aws_subnet.private_a` (10.3.1.0/24, ap-northeast-1a)
  - `aws_route_table` + `aws_route_table_association`
  - `aws_security_group.ec2` — egress: VPC endpoints のみ（443/tcp）
  - `aws_security_group.endpoints` — ingress: 443/tcp from EC2 SG
  - `aws_vpc_endpoint.s3` — Gateway 型（無料）
  - `aws_vpc_endpoint.ssm` — Interface 型
  - `aws_vpc_endpoint.ssmmessages` — Interface 型
  - `aws_vpc_endpoint.ec2messages` — Interface 型

---

## Phase 4 — Terraform: IAM

- [ ] 4-A: `terraform/iam.tf` を作成
  - `aws_iam_role.ec2_role`（EC2 assume role）
  - `aws_iam_role_policy.s3_policy`
    - `s3:GetObject` — `airgap/binary/*`, `airgap/src/*`, `airgap/data/*`
    - `s3:PutObject` — `airgap/output/*`, `airgap/proof/*`
  - `aws_iam_role_policy_attachment.ssm`（AmazonSSMManagedInstanceCore）
  - `aws_iam_instance_profile.ec2_profile`

---

## Phase 5 — Terraform: EC2

- [ ] 5-A: `terraform/ec2.tf` を作成
  - `aws_instance.favnir_ec2`
    - ami: Amazon Linux 2023 (ap-northeast-1)
    - instance_type: t3.small
    - subnet_id: private_a
    - vpc_security_group_ids: ec2 SG
    - iam_instance_profile: ec2_profile
    - user_data: `<<-EOF ... EOF` — 以下の手順を実行
      1. `aws s3 cp s3://${BUCKET}/airgap/binary/fav /tmp/fav`
      2. `chmod +x /tmp/fav`
      3. `aws s3 cp s3://${BUCKET}/airgap/src/analyze.fav /tmp/analyze.fav`
      4. `aws s3 cp --recursive s3://${BUCKET}/airgap/data/ /tmp/data/`
      5. `which fav 2>/dev/null || echo "not found"` → proof
      6. `/tmp/fav run /tmp/analyze.fav /tmp/data/txn_jan.csv /tmp/data/txn_feb.csv /tmp/data/txn_mar.csv`
      7. `aws s3 cp /tmp/proof-*.txt s3://${BUCKET}/airgap/proof/`

---

## Phase 6 — Terraform: 変数・出力

- [ ] 6-A: `terraform/variables.tf` を作成
  - `aws_region` (default: ap-northeast-1)
  - `aws_account` (sensitive)
  - `bucket_name` (default: favnir-e2e-demo)
- [ ] 6-B: `terraform/outputs.tf` を作成
  - `instance_id`, `instance_private_ip`, `vpc_id`

---

## Phase 7 — スクリプト

- [ ] 7-A: `scripts/upload.sh` を作成
  - Favnir バイナリを `airgap/binary/fav` にアップロード
  - `analyze.fav` / CSV 3件 を S3 にアップロード
- [ ] 7-B: `scripts/run.sh` を作成
  - `terraform apply -auto-approve`
  - EC2 instance ID 取得
  - SSM で起動完了まで待機（cloud-init done チェック）
  - user_data ログを CloudWatch / SSM 経由で確認
- [ ] 7-C: `scripts/verify.sh` を作成
  - [1] `airgap/proof/proof-*.txt` が S3 に存在する
  - [2] proof に `which fav: not found` が含まれる
  - [3] proof に `Dropped:` 行が含まれる（品質チェックログ）
  - [4] `airgap/output/summary.json` が S3 に存在する
  - [5] PASS 確認後 `terraform destroy -auto-approve`（EC2 後片付け）

---

## Phase 8 — 動作確認・README

- [ ] 8-A: `terraform init` + `scripts/upload.sh` 実行
- [ ] 8-B: `scripts/run.sh` 実行（EC2 起動 → user_data 完了待ち）
- [ ] 8-C: `scripts/verify.sh` 実行 → **PASS=5 / FAIL=0**
- [ ] 8-D: `README.md` 作成
  - 実行結果サマリー（PASS=5/FAIL=0）
  - アーキテクチャ図
  - 他デモとの比較表
  - 実行手順

---

## 完了条件サマリー

| 確認項目 | 担当 | 状態 |
|---|---|---|
| proof ファイルが S3 に存在する | scripts/verify.sh | |
| `which fav` → not found（システム未汚染） | user_data + verify | |
| ドロップ率ログが proof に存在する | analyze.fav + verify | |
| `airgap/output/summary.json` が S3 に存在 | analyze.fav + verify | |
| EC2 インスタンスが terminated | scripts/verify.sh | |
