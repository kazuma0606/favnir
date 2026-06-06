# fav2py E2E Demo — 実行チェックリスト

## 事前準備

- [ ] AWS CLI 設定済み（`aws sts get-caller-identity` 通過）
- [ ] `fav` バイナリを `infra/e2e-demo/fav2py/fav` にコピー
- [ ] S3 バケット `favnir-e2e-demo` が存在する
- [ ] `export TF_VAR_db_password="..."` を設定
- [ ] `export AWS_DEFAULT_REGION=ap-northeast-1` を設定

## Step 1: Docker ビルド + ECR push

```bash
./scripts/upload.sh
```

- [ ] Docker build 成功
- [ ] ECR push 成功
- [ ] S3 source upload 成功

## Step 2: E2E 実行

```bash
./scripts/run.sh
```

- [ ] `[1/5] terraform apply` — PASS
- [ ] `[2/5] fav-native ECS タスク` — exit 0
- [ ] `[3/5] fav-python ECS タスク` — exit 0
- [ ] `[4/5] verify.sh` — native == python
- [ ] `[5/5] run log upload` — S3 保存成功

## Step 3: 証跡確認

```bash
aws s3 ls s3://favnir-e2e-demo/proof/fav2py/
```

- [ ] 2 件以上の `.json` が存在する
- [ ] `RESULT: PASS=5 FAIL=0` を確認

## Step 4: クリーンアップ（任意）

```bash
cd terraform && terraform destroy -auto-approve
```

- [ ] RDS インスタンス削除完了
- [ ] ECS Cluster 削除完了
- [ ] NAT Gateway 削除完了（課金停止）
