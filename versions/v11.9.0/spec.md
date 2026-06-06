# Favnir v11.9.0 仕様書

作成日: 2026-06-06
テーマ: fav2py E2E インフラ (`infra/e2e-demo/fav2py/`)

---

## 背景と目的

v11.8.0 で Python トランスパイラの機能が完成した。
v11.9.0 では `infra/e2e-demo/fav2py/` に E2E 実証インフラを構築し、
**Fav ネイティブ実行** と **Python トランスパイル実行** が同じ RDS Postgres に対して
同一結果を出力することを実証する。

---

## ディレクトリ構造

```
infra/e2e-demo/fav2py/
├── src/
│   ├── pipeline.fav     # Fav ネイティブパイプライン
│   └── sample.csv       # デモ用サンプルデータ（103 行）
├── terraform/
│   ├── main.tf          # VPC / RDS PostgreSQL / ECS Fargate / ECR
│   ├── iam.tf           # IAM ロール（ECS タスク実行・RDS 接続・S3 書き込み）
│   ├── variables.tf     # 変数定義
│   └── outputs.tf       # RDS endpoint / ECR URI / ECS cluster ARN
├── scripts/
│   ├── upload.sh        # Docker ビルド + ECR push + S3 ソースアップロード
│   ├── run.sh           # terraform apply → ECS タスク x2 起動 → 待機
│   └── verify.sh        # S3 出力比較（Fav 版 vs Python 版）
├── tasks.md             # このデモ専用タスクチェックリスト
└── README.md
```

---

## pipeline.fav 仕様

```
import rune "postgres"
import rune "aws"
import rune "csv"

type TxnRow = {
  id: Int
  region: String
  category: String
  amount: Float
}

type SummaryRow = {
  region: String
  category: String
  total: Float
  count: Int
}

// Stage 1: CSV 読み込み → Postgres INSERT
stage LoadAndInsert: String -> Int !IO !Postgres = |path| {
  bind rows <- csv.read<TxnRow>(path)
  postgres.execute(
    "INSERT INTO txn(id,region,category,amount) SELECT * FROM json_populate_recordset(NULL::txn,$1)",
    rows
  )
}

// Stage 2: 集計クエリ
stage Aggregate: Int -> List<SummaryRow> !Postgres = |_| {
  postgres.query<SummaryRow>(
    "SELECT region, category, SUM(amount) AS total, COUNT(*) AS count FROM txn GROUP BY region, category ORDER BY region, category",
    []
  )
}

// Stage 3: S3 に JSON 保存
stage SaveResult: List<SummaryRow> -> Unit !IO !AWS = |rows| {
  bind ts <- aws.timestamp()
  aws.s3_put_json($"favnir-e2e-demo/proof/fav2py/{ts}.json", rows)
}

seq Pipeline = LoadAndInsert |> Aggregate |> SaveResult
```

---

## インフラ設計

### VPC

| リソース | 設定 |
|---|---|
| CIDR | 10.0.0.0/16 |
| Public Subnet | 10.0.1.0/24 (ap-northeast-1a) |
| Private Subnet | 10.0.2.0/24 (ap-northeast-1a) |
| NAT Gateway | Public Subnet に 1 つ |

### RDS PostgreSQL

| 設定 | 値 |
|---|---|
| Engine | PostgreSQL 16 |
| Instance class | db.t3.micro |
| Subnet | Private Subnet |
| Security Group | ECS タスクからのみ 5432 許可 |
| Initial database | `fav2py` |

### ECS Fargate タスク x2

| タスク | イメージ | コマンド | 環境変数 |
|---|---|---|---|
| `fav-native` | `<ECR>/fav2py:latest` | `fav run pipeline.fav sample.csv` | `DATABASE_URL`, `AWS_*` |
| `fav-python` | `<ECR>/fav2py:latest` | `fav transpile --target python pipeline.fav --out-dir /out && uv run /out/main.py sample.csv` | 同上 |

同一 Docker イメージに `fav` バイナリ + `uv` + `psycopg2` をインストール。

### ECR

- リポジトリ名: `favnir/fav2py`

---

## scripts/

### `upload.sh`
1. `docker build -t fav2py .` （Dockerfile: Ubuntu + fav binary + uv）
2. `docker tag fav2py:latest <ECR_URI>/fav2py:latest`
3. `docker push <ECR_URI>/fav2py:latest`
4. `aws s3 cp src/ s3://favnir-e2e-demo/fav2py/src/ --recursive`

### `run.sh`
1. `terraform init && terraform apply -auto-approve`
2. ECS タスク `fav-native` を起動 → 完了まで待機
3. ECS タスク `fav-python` を起動 → 完了まで待機
4. 両タスクの exit code を確認
5. `verify.sh` を呼び出す

### `verify.sh`
1. S3 から最新 2 件の JSON を取得（native / python それぞれ）
2. `jq` で region+category+total を比較
3. 差異がなければ `PASS=5` / 差異あれば `FAIL` を表示
4. exit code: 0 (PASS) / 1 (FAIL)

---

## テスト設計（v11900_tests）

Rust テスト（ディレクトリ存在確認 + pipeline.fav パース + トランスパイル確認）:

| テスト名 | 検証内容 |
|---|---|
| `fav2py_e2e_demo_structure` | ディレクトリ・ファイルが存在する |
| `fav2py_pipeline_fav_transpiles` | `pipeline.fav` が Python にトランスパイルできる |

---

## バージョン更新

- `fav/Cargo.toml`: `version = "11.9.0"`
