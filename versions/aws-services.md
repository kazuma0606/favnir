# Favnir — AWS サービス統合アイデア集

作成日: 2026-05-20
前提: **v6.0.0 セルフホスト完成後**に着手する候補
位置づけ: v7.x 以降のロードマップ候補、または v5〜v6 と並行する PoC

---

## 全体像

```
現在の AWS 統合
  Lambda + API Gateway  ← Rune Registry ホスティング
  S3 + CloudFront       ← リファレンスサイト配信
  DynamoDB              ← Registry メタデータ
  aws Rune              ← S3/SQS/DynamoDB を Favnir から型安全に操作

追加候補（本文書）
  A. Bedrock Rune       ← 言語 AI アシスタント
  B. Step Functions     ← パイプライン → ステートマシン コンパイル
  C. Athena Rune        ← サーバーレス SQL on S3
  D. Glue Rune          ← Data Catalog スキーマ探索
  E. Kinesis Rune       ← ストリーム処理
  F. RDS Oracle PoC     ← Oracle 対応検証
```

---

## A. Bedrock Rune — 言語 AI アシスタント

### 概要

AWS Bedrock（Claude/Titan）を使い、Favnir の学習・開発体験を向上させる。

### ユースケース

```bash
# ターミナルからの問い合わせ
fav ask "CSV をヘッダー付きで読む方法は？"
# → Bedrock が Favnir コードを生成して返す

# エラーの詳細説明（既存の explain-error を拡張）
fav explain-error E0213 --context src/main.fav:42
# → コードの文脈を含めて Bedrock が原因・修正案を提示
```

```favnir
// Rune として使う場合
import rune "bedrock"

bind reply <- bedrock.ask("CSVを読んで最大値を返すコードを書いて")
IO.println(reply)
```

### サイト統合（Playground 拡張）

- Playground にチャット欄を追加
- Bedrock API を呼ぶサーバーサイドエンドポイントを Lambda に追加
- エラーカタログ・Rune カタログをシステムプロンプトに注入してコンテキスト化

### 実装方針

```
Lambda (fav-bedrock-api)
  POST /ask  { question: String, context?: String }
  → bedrock.invoke_model() → 生成テキストを返す

Rust VM primitive
  Bedrock.invoke_raw(model_id, prompt) -> Result<String, String> !AWS
  （aws Rune と同じ SigV4 署名パターン）
```

### 評価

| 項目 | 評価 |
|------|------|
| ユーザー価値 | ★★★ 学習コストを直接削減 |
| 実装コスト | ★★☆ API 呼び出しのみ、VM primitive 1本 |
| AWS 活用度 | ★★☆ Bedrock + Lambda |
| 差別化 | ★★☆ 言語専用コンテキストを持つ点が差別化 |

---

## B. Step Functions 統合 — パイプライン → ステートマシン コンパイル

### 概要

`fav deploy --target step-functions` で `.fav` ファイルを AWS Step Functions の
ステートマシン定義（Amazon States Language JSON）にコンパイルする。

### コンセプト

```favnir
// この Favnir パイプラインを…
public fn pipeline() -> Unit !AWS !Db !Io {
  bind rows  <- aws.s3.read_parquet<Order>("my-bucket", "input/")
  bind clean <- validate(rows)
  bind _     <- db.insert_all(clean)
  bind _     <- aws.s3.write_json("my-bucket", "output/summary.json", summarize(clean))
}
```

```json
// …Step Functions JSON にコンパイル
{
  "StartAt": "ReadParquet",
  "States": {
    "ReadParquet": { "Type": "Task", "Resource": "arn:aws:lambda:...:fav-s3-read", "Next": "Validate" },
    "Validate":   { "Type": "Task", "Resource": "arn:aws:lambda:...:fav-validate", "Next": "DbInsert" },
    "DbInsert":   { "Type": "Task", "Resource": "arn:aws:lambda:...:fav-db-insert", "Next": "WriteOutput" },
    "WriteOutput": { "Type": "Task", "Resource": "arn:aws:lambda:...:fav-s3-write", "End": true }
  }
}
```

### エフェクト → ステートマッピング

Favnir のエフェクト型がステートマシンの型付けに自然に対応する:

| Favnir エフェクト | Step Functions ステート |
|-----------------|----------------------|
| `!AWS` | Task（Lambda 経由） |
| `!Db` | Task（Lambda 経由） |
| `!Io` | Task or Pass |
| `match` 分岐 | Choice |
| エラー伝播（`Result`） | Catch |
| 並列処理（将来） | Parallel |

### 実装方針

```
fav/src/backend/step_functions_codegen.rs  ← 新規バックエンド
fav deploy --target step-functions main.fav
  → main.asl.json (ステートマシン定義)
  → CDK / Terraform に取り込み可能

Terraform モジュール（infra/modules/step-functions/）
  aws_sfn_state_machine リソース定義テンプレート
```

### 評価

| 項目 | 評価 |
|------|------|
| ユーザー価値 | ★★★ 型安全パイプラインの実行基盤として最強 |
| 実装コスト | ★★★ コンパイラ新バックエンドが必要（大きめ） |
| AWS 活用度 | ★★★ Step Functions + Lambda のフル活用 |
| 差別化 | ★★★ 「型チェック済みパイプラインが本番で動く」が証明できる |

> v6.0.0 以降の大きな柱候補。v7.x ロードマップの Orchestration Rune（DAG 実行）とも統合できる。

---

## C. Athena Rune — サーバーレス SQL on S3

### 概要

AWS Athena への型安全クエリを DuckDB Rune と同じ API で提供する。
「ローカル分析は DuckDB、本番 S3 データは Athena」の使い分けが自然にできる。

### 使用例

```favnir
import rune "athena"

type Order = { customer: String  amount: Float }
type Summary = { customer: String  total: Float }

public fn main() -> Unit !Io !AWS {
  bind conn   <- athena.connect("my-glue-database", "s3://my-bucket/athena-results/")
  bind result <- athena.query<Summary>(conn,
    "SELECT customer, SUM(amount) AS total
     FROM orders
     GROUP BY customer ORDER BY total DESC")
  IO.println(result)
}
```

### DuckDB Rune との差異

| | DuckDB Rune | Athena Rune |
|-|------------|------------|
| 実行場所 | ローカル / Lambda | AWS マネージド |
| データ | ローカルファイル / S3（DL） | S3（直接クエリ） |
| エフェクト | `!Db` | `!AWS` |
| コスト | 無料 | スキャン量課金 |
| 向き先 | 開発・小規模 | 本番・大規模 |

### 実装方針

```
VM primitive (Rust)
  AWS.athena_start_query_raw(db, output_s3, sql) -> Result<String, String> !AWS
  AWS.athena_get_result_raw(query_id) -> Result<List<Map>, String> !AWS

runes/athena/ (Favnir)
  connect(database, output_location) -> AthenaConn
  query<T>(conn, sql) -> Result<List<T>, String> !AWS
  （DuckDB Rune とほぼ同じ構造）
```

### 評価

| 項目 | 評価 |
|------|------|
| ユーザー価値 | ★★★ データエンジニアが日常的に使う |
| 実装コスト | ★☆☆ DuckDB Rune の横展開でコスト低 |
| AWS 活用度 | ★★★ Athena のフル活用 |
| 差別化 | ★★☆ DuckDB との対比で「ローカル↔本番」切り替えを訴求 |

---

## D. Glue Rune — Data Catalog スキーマ探索

### 概要

AWS Glue Data Catalog に登録されたテーブルのスキーマを Favnir 型として取得・生成する。

### ユースケース

```bash
# Glue カタログからスキーマを自動生成
fav schema infer --glue my_database --table orders
```

```favnir
// 自動生成された型定義
type Order = {
  order_id:   Int
  customer:   String
  amount:     Float
  created_at: String
}
```

```favnir
// Rune として使う場合（動的にスキーマを取得）
import rune "glue"

bind schema <- glue.describe_table("my_database", "orders")
IO.println(schema)
```

### 実装方針

```
VM primitive (Rust)
  AWS.glue_get_table_raw(database, table) -> Result<Map, String> !AWS
  AWS.glue_list_tables_raw(database) -> Result<List<String>, String> !AWS

fav schema infer --glue コマンド (driver.rs)
  Glue API のスキーマ定義 → Favnir type 定義文字列に変換
  → src/types/ 以下にファイル生成
```

### v7.2.0「DB スキーマ推論」との関係

v7.2.0 で計画している `fav schema infer --db` の Glue 版。
実装パターンが共通なので v7.2.0 と同時着手が効率的。

### 評価

| 項目 | 評価 |
|------|------|
| ユーザー価値 | ★★☆ 既存 Glue 資産を持つ企業に有効 |
| 実装コスト | ★★☆ v7.2.0 と共通化できれば低め |
| AWS 活用度 | ★★☆ Glue 活用 |
| 差別化 | ★★☆ 「スキーマは書かなくていい」訴求 |

---

## E. Kinesis Rune — ストリーム処理

### 概要

AWS Kinesis Data Streams を使ったリアルタイムパイプラインを Favnir で記述する。

### 使用例

```favnir
import rune "kinesis"

type ClickEvent = { user_id: String  page: String  ts: Int }

public fn producer() -> Unit !Io !AWS {
  bind stream <- kinesis.connect("my-stream")
  bind event  <- ClickEvent { user_id: "u123" page: "/top" ts: 1234567890 }
  bind _      <- kinesis.produce(stream, event)
  IO.println("sent")
}

public fn consumer() -> Unit !Io !AWS {
  bind stream <- kinesis.connect("my-stream")
  kinesis.consume<ClickEvent>(stream, |event| {
    IO.println($"user={event.user_id} page={event.page}")
  })
}
```

### 既存との関係

現在の `aws` Rune は SQS（メッセージキュー）対応済み。Kinesis はより大規模な
ストリーム処理向けで、SQS Rune の上位互換となる位置づけ。

### 実装方針

```
VM primitive (Rust)
  AWS.kinesis_put_record_raw(stream, data, partition_key) -> Result<String, String> !AWS
  AWS.kinesis_get_records_raw(stream, shard_id) -> Result<List<Map>, String> !AWS

runes/kinesis/ (Favnir)
  connect(stream_name) -> KinesisConn
  produce<T>(conn, record) -> Result<Unit, String> !AWS
  consume<T>(conn, handler) -> Unit !AWS
```

### 評価

| 項目 | 評価 |
|------|------|
| ユーザー価値 | ★★☆ ストリーム処理ユーザーに刺さる |
| 実装コスト | ★★☆ SQS Rune に近いが Shard 管理が追加 |
| AWS 活用度 | ★★★ Kinesis フル活用 |
| 差別化 | ★★☆ バッチ → ストリームへの自然な拡張 |

---

## F. RDS Oracle PoC — Oracle 対応検証

### 概要

AWS RDS for Oracle を使い、既存の `db` Rune（PostgreSQL/SQLite 対応）を
Oracle に拡張できるかを検証する。

### PoC スコープ

```
- RDS Oracle インスタンス起動（検証用 db.t3.small）
- Rust oracle driver（oracle crate または jdbc-rs）の評価
- Db.connect_raw("oracle://...") の接続文字列対応
- SELECT / INSERT が db Rune 経由で動くことの確認
```

### 位置づけ

エンタープライズ（銀行・製造業）でのレガシー Oracle 統合を狙う PoC。
v7.2.0「スキーマ推論」と組み合わせると「Oracle スキーマ → Favnir 型自動生成」が実現。

### 評価

| 項目 | 評価 |
|------|------|
| ユーザー価値 | ★★☆ エンタープライズ向け |
| 実装コスト | ★★★ oracle crate の成熟度に依存 |
| AWS 活用度 | ★☆☆ RDS を使うだけ（AWS 固有性は低い） |
| 差別化 | ★★☆ エンタープライズ訴求に効く |

---

## 優先度まとめ

| 優先 | 案 | 推奨タイミング | 理由 |
|------|---|--------------|------|
| 1位 | **A. Bedrock** | v5.5〜5.6 と並行 PoC | 実装コスト低・すぐ見える成果 |
| 2位 | **C. Athena** | v7.x DuckDB 横展開時 | コスト低・DuckDB と対比で訴求 |
| 3位 | **B. Step Functions** | v7.x Orchestration 時 | 差別化最大・v7 Orchestration Rune と統合 |
| 4位 | **D. Glue** | v7.2.0 スキーマ推論と同時 | v7.2.0 実装の自然な拡張 |
| 5位 | **E. Kinesis** | v7.4.0 Rune 拡充時 | SQS Rune の延長 |
| 6位 | **F. Oracle PoC** | エンタープライズ需要が出たとき | AWS 固有性低・需要次第 |

---

## v6.0.0 完成後のロードマップへの組み込みイメージ

```
v6.0.0  セルフホスト完成（言語機能一区切り）
v7.0.0  db rune 本格稼働 + DbRead/DbWrite/DbAdmin エフェクト
v7.1.0  fav explain --lineage（データリネージ）
v7.2.0  スキーマ推論（DB → Favnir 型）
           ↳ D. Glue Rune もここで着手
v7.3.0  SQL Rune（型安全クエリビルダ）
           ↳ C. Athena Rune もここで着手（DuckDB/Athena 共通 API）
v7.4.0  Rune エコシステム拡充（queue / cache / fs）
           ↳ E. Kinesis Rune もここで着手
v7.5.0  Orchestration Rune（DAG 実行）
           ↳ B. Step Functions 統合の本命タイミング

PoC（随時）
  A. Bedrock Rune  ← v6.0.0 完成を待たずに PoC 可能
  F. Oracle PoC   ← エンタープライズ需要が出たタイミングで
```
