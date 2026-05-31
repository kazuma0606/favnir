# Favnir E2E Demo — ECS 版 アーキテクチャ仕様

Date: 2026-05-30

## 概要

EC2 版（plan.md）の「2台の EC2 で分離」という証明に対し、ECS 版は：

- **Public EC2**: Favnir 処理系（compiler.fav / checker.fav / .fav ソース一式）
- **Private EC2**: Rust VM のみ（`fav` バイナリ。ソースコードなし）
- **ECS Fargate Task**: 純粋な Favnir で書いた ETL パイプライン（`.fvc` アーティファクトのみ実行）

の3層で「何がどこにあるか」を物理的に分離し、それぞれのサーバ・コンテナに
**余計なファイルが存在しないこと**をファイルリスト（証跡）として S3 に保存する。

---

## アーキテクチャ

```
VPC (10.0.0.0/16)
│
├── Public Subnet (10.0.1.0/24)
│   └── [Machine A] Public EC2 — t3.micro / Ubuntu 24.04 LTS
│       役割: Favnir 処理系サーバ
│       配置ファイル:
│         /usr/local/bin/fav          Favnir バイナリ（ビルド成果物）
│         /app/src/pipeline.fav       デモ用パイプライン（ソース）
│         /app/src/etl.fav            ECS ETL パイプライン（ソース）
│         /app/self/compiler.fav      セルフホストコンパイラ
│         /app/self/checker.fav       セルフホスト型チェッカー
│       動作:
│         fav build /app/src/pipeline.fav -o /tmp/pipeline.fvc
│         fav build /app/src/etl.fav     -o /tmp/etl.fvc
│         aws s3 cp /tmp/pipeline.fvc s3://BUCKET/artifacts/pipeline.fvc
│         aws s3 cp /tmp/etl.fvc      s3://BUCKET/artifacts/etl.fvc
│       証跡:
│         find /app -type f | sort → S3/proof/machine-a/filelist-TIMESTAMP.txt
│
└── Private Subnet (10.0.2.0/24)
    │
    ├── [Machine B] Private EC2 — t3.micro / Ubuntu 24.04 LTS
    │   役割: Rust VM サーバ（.fav ソース一切なし）
    │   配置ファイル:
    │     /usr/local/bin/fav          fav バイナリのみ
    │     ※ .fav ファイルは一切存在しない
    │   動作:
    │     S3 から pipeline.fvc を取得して fav exec 実行
    │     RDS からデータ読み取り → S3 にサマリー書き出し
    │   証跡:
    │     find / -name "*.fav" 2>/dev/null → S3/proof/machine-b/fav-search-TIMESTAMP.txt
    │     ls -la /usr/local/bin/      → S3/proof/machine-b/binlist-TIMESTAMP.txt
    │
    ├── [ECS Fargate Task] ETL Pipeline Runner
    │   役割: 純粋 Favnir ETL（コンテナ内に .fav ソースなし）
    │   イメージ: favnir/runtime（fav バイナリのみ）
    │   動作:
    │     S3 から etl.fvc を取得して fav exec 実行
    │     RDS 読み取り → 集計 → S3 サマリー書き出し
    │   証跡（init container で実行）:
    │     find / -name "*.fav" 2>/dev/null → S3/proof/ecs/fav-search-TIMESTAMP.txt
    │     ls -la /usr/local/bin/          → S3/proof/ecs/binlist-TIMESTAMP.txt
    │
    └── [RDS] Aurora Serverless v2 (PostgreSQL compatible)
        デモ用サンプルデータ（orders テーブル）を事前投入

S3 バケット (favnir-e2e-demo)
├── artifacts/
│   ├── pipeline.fvc             Machine A がビルド・アップロード
│   └── etl.fvc                  Machine A がビルド・アップロード
├── proof/
│   ├── machine-a/
│   │   └── filelist-TIMESTAMP.txt     /app 配下の全ファイル一覧
│   ├── machine-b/
│   │   ├── fav-search-TIMESTAMP.txt   .fav ファイルが0件であること
│   │   └── binlist-TIMESTAMP.txt      /usr/local/bin/ の内容
│   └── ecs/
│       ├── fav-search-TIMESTAMP.txt   .fav ファイルが0件であること
│       └── binlist-TIMESTAMP.txt      /usr/local/bin/ の内容
└── output/
    └── summary-TIMESTAMP.json         ETL の出力サマリー
```

---

## ETL パイプライン仕様（Favnir）

ECS で実行される ETL は以下の3ステージで構成する：

```favnir
import rune "postgres"
import rune "aws"

type Order   = { id: Int  customer: String  amount: Float  created_at: String }
type Summary = { customer: String  total: Float  count: Int }

// Extract: RDS から受注データを取得
stage ExtractOrders: String -> List<Order> !Db = |conn_str| {
  bind conn <- postgres.connect(conn_str)
  postgres.query<Order>(conn, "SELECT id, customer, amount, created_at FROM orders")
}

// Transform: 顧客別に集計
stage Summarize: List<Order> -> List<Summary> = |orders| {
  List.map(
    List.group_by(orders, |o| o.customer),
    |group| Summary {
      customer: group.key
      total:    List.sum(List.map(group.items, |o| o.amount))
      count:    List.length(group.items)
    }
  )
}

// Load: S3 に JSON として書き出し
stage SaveSummary: List<Summary> -> Unit !AWS = |summaries| {
  bind ts <- aws.timestamp()
  aws.s3_put_json($"favnir-e2e-demo/output/summary-{ts}.json", summaries)
}

// パイプライン定義
seq EtlPipeline = ExtractOrders |> Summarize |> SaveSummary
```

---

## Docker イメージ設計

```
favnir/runtime
  FROM ubuntu:24.04
  COPY fav /usr/local/bin/fav        # Rust でビルドした Linux バイナリのみ
  # .fav ファイルは一切含まない
  # docker inspect で確認可能
```

---

## ネットワーク設計

NAT Gateway は使用しない（コスト削減）。VPC Endpoint で通信を制御。

| エンドポイント     | タイプ    | 料金       | 用途                         |
|--------------------|-----------|------------|------------------------------|
| S3                 | Gateway   | 無料       | artifact 取得・証跡・サマリー |
| CloudWatch Logs    | Interface | $0.01/時   | ログ送信                     |
| SSM                | Interface | $0.01/時   | SSH 不要コンソールアクセス   |
| ECR (dkr/api)      | Interface | $0.01/時   | ECS イメージ pull            |

Machine B・ECS Task はすべて Private Subnet に配置。
RDS は同一 Private Subnet 内のため直接接続。

---

## 証跡収集の設計

「余計なファイルがないこと」を自動的に S3 に記録する。

### Machine A（Favnir 処理系）
```bash
# 起動スクリプト内で実行
find /app -type f | sort > /tmp/machine-a-filelist.txt
aws s3 cp /tmp/machine-a-filelist.txt \
  s3://favnir-e2e-demo/proof/machine-a/filelist-$(date +%Y%m%d-%H%M%S).txt
```

期待値: `/app/src/*.fav`, `/app/self/*.fav`, `/usr/local/bin/fav` が存在する。

### Machine B（Rust VM のみ）
```bash
# user-data スクリプト内で実行
find / -name "*.fav" 2>/dev/null > /tmp/fav-search.txt
ls -la /usr/local/bin/ >> /tmp/fav-search.txt
aws s3 cp /tmp/fav-search.txt \
  s3://favnir-e2e-demo/proof/machine-b/fav-search-$(date +%Y%m%d-%H%M%S).txt
```

期待値: `.fav` ファイルの検索結果が**0件**。`/usr/local/bin/fav` のみ存在。

### ECS Task（runtime コンテナ）
```yaml
# init container として実行（favnir/runtime と同じイメージ）
- name: proof-collector
  image: favnir/runtime:latest
  command:
    - /bin/sh
    - -c
    - |
      find / -name "*.fav" 2>/dev/null > /tmp/ecs-fav-search.txt
      ls -la /usr/local/bin/ >> /tmp/ecs-fav-search.txt
      aws s3 cp /tmp/ecs-fav-search.txt \
        s3://favnir-e2e-demo/proof/ecs/fav-search-$(date +%Y%m%d-%H%M%S).txt
```

期待値: `.fav` ファイルの検索結果が**0件**。`/usr/local/bin/fav` のみ存在。

---

## 完了条件

- [ ] Machine A の `/app/src/` に `.fav` ソースが存在し、ビルドが成功する
- [ ] Machine B で `find / -name "*.fav"` の結果が 0 件（証跡 S3 に保存済み）
- [ ] ECS コンテナ内で `find / -name "*.fav"` の結果が 0 件（証跡 S3 に保存済み）
- [ ] ECS Task が `.fvc` アーティファクトのみで ETL を完走する
- [ ] S3 `output/` にサマリー JSON が出力されている
- [ ] CloudWatch Logs に Machine A・Machine B・ECS の全ログが残っている
- [ ] Machine B が ETL 完了後に自動 stop する

---

## コスト概算（デモ 1 回あたり）

| リソース                    | 稼働時間  | 費用    |
|-----------------------------|-----------|---------|
| Machine A (t3.micro)        | ~1 時間   | ~$0.01  |
| Machine B (t3.micro)        | ~30 分    | ~$0.005 |
| ECS Fargate (0.25vCPU/0.5GB)| ~15 分    | ~$0.003 |
| Aurora Serverless v2        | ~1 時間   | ~$0.06  |
| S3                          | -         | < $0.01 |
| CloudWatch Logs             | -         | < $0.01 |
| VPC Interface Endpoints x4  | ~2 時間   | ~$0.08  |
| **合計**                    |           | **~$0.17** |
