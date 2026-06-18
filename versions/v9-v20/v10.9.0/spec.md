# Favnir v10.9.0 仕様書

Date: 2026-06-04
Theme: E2E テスト（実 Snowflake インスタンス）

---

## 概要

`infra/e2e-demo`（ECS / EKS / Lambda）と同じ構造で Snowflake E2E デモを実装する。
v10.1.0〜v10.8.0 で構築した Snowflake 統合（VM Primitive / Effect / Rune / infer）を
**実際の Snowflake インスタンス**に対して証明する。

---

## デモシナリオ

```
CSV ファイル → TransformRows（純粋変換）→ Snowflake INSERT → クエリ集計 → S3 サマリー出力
```

4 ステージの Favnir pipeline:

```favnir
import rune "snowflake"
import rune "csv"
import rune "aws"

type OrderRow   = { order_id: Int  customer: String  amount: Float  region: String }
type SummaryRow = { region: String  total: Float  count: Int }

// Stage 1: CSV 読み込み
stage LoadCsv: String -> List<OrderRow> !IO = |path| {
  csv.read<OrderRow>(path)
}

// Stage 2: 行変換（純粋関数）
stage TransformRows: List<OrderRow> -> List<OrderRow> = |rows| {
  List.filter(rows, |r| r.amount > 0.0)
}

// Stage 3: Snowflake に INSERT
stage SnowflakeInsert: List<OrderRow> -> Int !Snowflake = |rows| {
  bind sql <- Json.encode_raw(rows)
  snowflake.execute($"INSERT INTO DEMO_DB.PUBLIC.ORDERS SELECT * FROM TABLE(FLATTEN(PARSE_JSON('{sql}')))")
}

// Stage 4: クエリ集計 → S3 保存
stage QuerySummary: Int -> Unit !Snowflake !AWS = |_| {
  bind rows <- snowflake.query<SummaryRow>(
    "SELECT region, SUM(amount) AS total, COUNT(*) AS count FROM DEMO_DB.PUBLIC.ORDERS GROUP BY region"
  )
  bind ts  <- aws.timestamp()
  aws.s3_put_json($"favnir-e2e-demo/proof/snowflake/summary-{ts}.json", rows)
}

seq DemoPipeline = LoadCsv |> TransformRows |> SnowflakeInsert |> QuerySummary
```

**PASS = 4（ステージ数）/ FAIL = 0**

---

## ディレクトリ構造

```
infra/e2e-demo/snowflake/
├── spec.md          本仕様書（コピー）
├── plan.md          実装計画
├── tasks.md         タスクリスト
├── README.md        実行手順
├── src/
│   └── demo.fav     デモパイプライン（上記 Favnir ソース）
├── terraform/
│   ├── main.tf      provider / backend / locals
│   ├── variables.tf
│   ├── outputs.tf
│   ├── warehouse.tf Snowflake ウェアハウス（DEMO_WH）
│   ├── table.tf     Snowflake テーブル（DEMO_DB.PUBLIC.ORDERS）
│   └── iam.tf       AWS IAM（S3 proof 書き込み用ロール）
└── scripts/
    └── run.sh       E2E 実行スクリプト
```

---

## Terraform リソース

### Snowflake リソース（snowflake-labs/snowflake provider）

| リソース | 名前 | 備考 |
|---|---|---|
| `snowflake_warehouse` | `DEMO_WH` | X-Small、auto-suspend 60s |
| `snowflake_database` | `DEMO_DB` | |
| `snowflake_schema` | `DEMO_DB.PUBLIC` | |
| `snowflake_table` | `DEMO_DB.PUBLIC.ORDERS` | order_id/customer/amount/region |

### AWS リソース

| リソース | 備考 |
|---|---|
| S3 prefix `s3://favnir-e2e-demo/proof/snowflake/` | 既存バケット流用 |
| IAM Role | S3 PutObject（proof/snowflake/ のみ） |

---

## 証跡

S3 に以下のファイルを保存:

```
s3://favnir-e2e-demo/proof/snowflake/
├── summary-<TIMESTAMP>.json   クエリ集計結果（region 別集計）
└── run-<TIMESTAMP>.txt        実行ログ（PASS/FAIL カウント）
```

---

## Rust テスト（+1）

`driver.rs` に `v10900_tests::snowflake_e2e_demo_structure` を 1 件追加:
- `infra/e2e-demo/snowflake/` ディレクトリが存在する
- `infra/e2e-demo/snowflake/src/demo.fav` が存在する
- `infra/e2e-demo/snowflake/README.md` が存在する

実接続不要。ファイル存在確認のみ。

---

## 完了条件

| 条件 | 状態 |
|---|---|
| `demo.fav` が `fav check` を通過する | - |
| `demo.fav` が実 Snowflake に対して PASS=4 / FAIL=0 | - |
| 証跡が `s3://favnir-e2e-demo/proof/snowflake/` に保存されている | - |
| `infra/e2e-demo/snowflake/README.md` に実行手順が記載されている | - |
| `cargo test v10900` 1 件通過 | - |
| `cargo test` 全件通過（1283 件） | - |
