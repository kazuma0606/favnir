# fav2py E2E Demo

Favnir v11.9.0 — **Fav ネイティブ実行** と **Python トランスパイル実行** が
同一 RDS PostgreSQL に対して同じ集計結果を出力することを実証する E2E デモ。

## アーキテクチャ

```
sample.csv (103 rows)
    │
    ├─► ECS Task: fav-native   ─► fav run pipeline.fav        ─► RDS PostgreSQL ─► S3 (result-native.json)
    │
    └─► ECS Task: fav-python   ─► fav transpile + uv run      ─► RDS PostgreSQL ─► S3 (result-python.json)
                                                                                        │
                                                                               verify.sh (比較 PASS/FAIL)
```

## パイプライン（pipeline.fav）

```
LoadAndInsert  : String -> Int          !IO !Postgres
    |> Aggregate   : Int -> List<SummaryRow>  !Postgres
    |> SaveResult  : List<SummaryRow> -> Unit !IO !AWS
```

## 事前条件

- AWS CLI 設定済み（`aws sts get-caller-identity` が通る）
- Terraform >= 1.5 インストール済み
- Docker インストール済み
- `jq` インストール済み
- S3 バケット `favnir-e2e-demo` が存在する
- `fav` バイナリがビルド済み（`fav/target/release/fav`）

## セットアップ

```bash
# 1. fav binary を Dockerfile と同じディレクトリに配置
cp ../../fav/target/release/fav .

# 2. Docker build & ECR push
export AWS_DEFAULT_REGION=ap-northeast-1
export TF_VAR_db_password="<your-db-password>"
./scripts/upload.sh

# 3. Terraform apply + ECS タスク起動 + 結果検証
./scripts/run.sh
```

## 期待結果

```
=== RESULT: PASS=5 FAIL=0 ===
```

| ステップ | 内容 |
|---|---|
| PASS 1 | terraform apply 成功 |
| PASS 2 | fav-native ECS タスク exit 0 |
| PASS 3 | fav-python ECS タスク exit 0 |
| PASS 4 | verify.sh: 出力一致 |
| PASS 5 | run log S3 アップロード成功 |

## 証跡の確認

```bash
aws s3 ls s3://favnir-e2e-demo/proof/fav2py/
```

## クリーンアップ

```bash
cd terraform
terraform destroy -auto-approve
```
