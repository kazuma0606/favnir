# Roadmap v27.1.0 〜 v28.0.0 — Data Lakehouse

Date: 2026-06-24

## 目標

v27.0「Streaming Native」でリアルタイムパイプラインが型安全に書けるようになった。
バッチ ETL（v26.0）とストリーミング（v27.0）の両方が動く状態になった。

しかし現代のデータ基盤の中心は「データレイクハウス」だ。
Delta Lake / Apache Iceberg がデータレイクの標準になり、
Databricks / Snowflake / BigQuery / Redshift が DWH の主要選択肢になっている。
dbt がデータ変換の標準ツールとして普及している。

このフェーズでは、**現代のデータ基盤アーキテクチャに Favnir を溶け込ませる**。
Delta Lake・Iceberg のテーブルを読み書きし、dbt モデルを参照し、
主要 DWH（BigQuery / Redshift / ClickHouse）に接続できるようにする。

> **Data Lakehouse の定義（本プロジェクト固有）**
> 「Delta Lake / Iceberg テーブルの読み書きができ、dbt モデルを参照でき、
>  主要 DWH 3 本（BigQuery / Redshift / ClickHouse）に接続できる」状態を指す。

**完了条件（最終テスト）:**

```bash
# 1. 全 Rust テストが通る
cargo test

# 2. Delta Lake テーブル操作
fav run examples/delta_lake_etl.fav

# 3. dbt モデル参照
cargo test dbt

# 4. fav infer でスキーマ自動生成
fav infer --from delta --path /tmp/test_delta_table

# 5. 各 DWH テスト
cargo test clickhouse bigquery redshift jsonl sqlite
```

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| Delta Lake の実装方式 | `delta-rs`（Rust 実装）クレートを Rust バックエンドで利用。Pure Favnir 実装はしない |
| Iceberg の実装方式 | `iceberg-rust`（Apache 公式 Rust 実装）を利用 |
| Delta / Iceberg のローカル環境 | ローカルファイルシステム（`/tmp/`）で動作確認。S3 は LocalStack |
| `fav infer --from delta/iceberg` の出力形式 | Favnir 型定義コード（`type Row = { ... }` 形式）を標準出力 |
| dbt 連携の仕組み | `dbt/manifest.json` を解析して SQL を動的生成・実行。dbt CLI の直接呼び出しはしない |
| ClickHouse のローカル環境 | `clickhouse/clickhouse-server` Docker イメージ |
| BigQuery のローカル環境 | BigQuery Emulator（`ghcr.io/goccy/bigquery-emulator`） |
| Redshift のローカル環境 | Redshift は postgres 互換 → ローカルは postgres で代替 |
| sqlite のライブラリ | `rusqlite`（bundled feature）。依存ゼロで動く |
| 破壊的変更 | なし（STABILITY.md v1.x ポリシーに従う） |

---

## バージョン計画

### v27.1 — delta-lake Rune 追加

**テーマ**: 現代のデータレイクハウスの事実上の標準。
`delta-rs` を活用し、Favnir から Delta テーブルを型安全に読み書きできるようにする。

**依存関係**: v25.2（s3 Rune）完了後（Delta テーブルは S3 / ローカル FS 上に存在）

```favnir
import runes/delta-lake

// Delta Lake テーブルから型安全に読み込む
stage LoadOrders: Unit -> List<Order> !Io = |_| {
  bind rows <- DeltaLake.read[Order]("s3://my-bucket/orders")
  Result.ok(rows)
}

// 処理結果を Delta Lake に書き戻す（upsert）
stage SaveResult: List<ProcessedOrder> -> Unit !Io = |orders| {
  bind _ <- DeltaLake.merge[ProcessedOrder](
    "s3://my-bucket/processed-orders",
    orders,
    "source.id = target.id"
  )
  Result.ok(unit)
}
```

実装する関数:

| 関数 | 内容 |
|---|---|
| `DeltaLake.read[T](path)` | Delta テーブル全件読み込み（型付き、S3 / ローカル対応） |
| `DeltaLake.read_with_filter[T](path, predicate)` | 述語プッシュダウン付き読み込み |
| `DeltaLake.write(path, data, mode)` | 書き込み（append / overwrite） |
| `DeltaLake.merge[T](path, data, condition)` | MERGE（upsert / delete-when-matched） |
| `DeltaLake.history(path)` | トランザクションログ取得（バージョン一覧） |
| `DeltaLake.vacuum(path, retention_hours)` | 古いファイル削除（最小 168h = 7 日） |
| `DeltaLake.optimize(path)` | コンパクション（小さいファイルをまとめる） |

`cargo test delta_lake` で 5 件以上 PASS。
`fav run examples/delta_lake_etl.fav`（ローカルパス）が動くことを確認する。

---

### v27.2 — iceberg Rune 追加

**テーマ**: Apache Iceberg。Snowflake / AWS Glue / Spark との親和性が高く、
マルチエンジン対応のデータレイクテーブル形式として急速に普及している。

**依存関係**: v27.1（delta-lake）と並行可能（独立した API 体系）

実装する関数:

| 関数 | 内容 |
|---|---|
| `Iceberg.read[T](catalog, table)` | テーブル全件読み込み（REST / Glue カタログ対応） |
| `Iceberg.append(catalog, table, data)` | データ追加（新しいスナップショット作成） |
| `Iceberg.overwrite(catalog, table, data, filter)` | 条件を満たすデータを上書き |
| `Iceberg.time_travel[T](catalog, table, snapshot_id)` | スナップショット ID 指定読み込み |
| `Iceberg.schema_evolution(catalog, table, new_schema)` | スキーマ追加・型昇格（互換変更のみ） |
| `Iceberg.list_snapshots(catalog, table)` | スナップショット一覧取得 |

REST カタログ（ローカル）で `cargo test iceberg` が 4 件以上 PASS。

---

### v27.3 — clickhouse Rune 追加

**テーマ**: 分析特化の列指向 DB。リアルタイム集計クエリが高速。
ストリームから直接高速バルク挿入できる点で Rune Foundation との相性が良い。

**依存関係**: v25.1（postgres）完了後（クエリ API の設計参照）

```favnir
import runes/clickhouse

// Kafka から受信したイベントを ClickHouse にバルク挿入
stage InsertEvents: List<ClickEvent> -> Unit !Db = |events| {
  bind conn <- ClickHouse.connect(config.clickhouse)
  bind _    <- ClickHouse.insert(conn, "events", events)
  Result.ok(unit)
}

// リアルタイム集計クエリ
stage QueryHourlyStats: Unit -> List<HourlyStat> !Db = |_| {
  bind conn  <- ClickHouse.connect(config.clickhouse)
  bind stats <- ClickHouse.query[HourlyStat](conn, """
    SELECT
      toStartOfHour(timestamp) AS hour,
      count()                   AS event_count,
      uniq(user_id)             AS unique_users
    FROM events
    WHERE timestamp >= now() - INTERVAL 24 HOUR
    GROUP BY hour
    ORDER BY hour DESC
  """)
  Result.ok(stats)
}
```

実装する関数:

| 関数 | 内容 |
|---|---|
| `ClickHouse.connect(config)` | 接続確立（HTTP / native プロトコル） |
| `ClickHouse.query[T](conn, sql)` | 型付きクエリ（SELECT） |
| `ClickHouse.insert(conn, table, rows)` | バルク挿入（Native format） |
| `ClickHouse.async_insert(conn, table, rows)` | 非同期バルク挿入（ClickHouse v21.11+） |

`clickhouse/clickhouse-server`（Docker）で `cargo test clickhouse` が 3 件以上 PASS。

---

### v27.4 — bigquery Rune 実質化

**テーマ**: Google BigQuery。v15.2 の部分実装を完全実装に昇格。
GCS 経由のバルクロード（LOAD DATA）まで対応する。

**依存関係**: v25.2（s3 / GCS 接続設定の参照）完了後

実装する関数:

| 関数 | 内容 |
|---|---|
| `BigQuery.connect(config)` | 接続確立（service account JSON 対応） |
| `BigQuery.query[T](conn, sql)` | 型付きクエリ（SELECT） |
| `BigQuery.insert(conn, table, rows)` | streaming insert（即時反映、小量向き） |
| `BigQuery.load_from_gcs(conn, table, gcs_uri, format)` | GCS からバルクロード（大量向き） |
| `BigQuery.create_table(conn, table, schema)` | テーブル作成（スキーマ定義） |

BigQuery Emulator（`ghcr.io/goccy/bigquery-emulator`）で `cargo test bigquery` が 4 件以上 PASS。

---

### v27.5 — redshift Rune 追加

**テーマ**: AWS の分析 DWH。postgres 互換 API + COPY コマンドによる S3 からの高速ロード。

**依存関係**: v25.1（postgres API 設計の再利用）・v25.2（s3 COPY との連携）完了後

実装する関数:

| 関数 | 内容 |
|---|---|
| `Redshift.connect(config)` | 接続確立（postgres ドライバ利用） |
| `Redshift.query[T](conn, sql)` | 型付きクエリ |
| `Redshift.execute(conn, sql, params)` | DDL / DML 実行 |
| `Redshift.copy_from_s3(conn, table, s3_uri, opts)` | S3 COPY（CSV / Parquet / JSON） |
| `Redshift.unload_to_s3(conn, query, s3_uri, opts)` | UNLOAD（SELECT 結果を S3 に出力） |

ローカル postgres で API 互換テスト。`cargo test redshift` で 3 件以上 PASS。

---

### v27.6 — jsonl Rune 追加

**テーマ**: JSON Lines。LLM ファインチューニングデータ・構造化ログ・イベントストアの現代的標準。
ストリーミング処理との親和性が高く、`Stream.*` と組み合わせて大容量を扱える。

**依存関係**: なし

```favnir
import runes/jsonl

// 大容量 JSONL をストリーミング処理
#[streaming]
stage ProcessLlmData: String -> Stream<TrainingExample> !Io = |path| {
  JSONL.stream[TrainingExample](path, |example| {
    // フィルタ・変換
    if String.length(example.text) > 10
    then Result.ok(example)
    else Result.err("too short")
  })
}
```

実装する関数:

| 関数 | 内容 |
|---|---|
| `JSONL.read[T](path)` | 型付き全件読み込み（メモリに全ロード） |
| `JSONL.write(path, rows)` | 書き込み（追記 / 上書きオプション） |
| `JSONL.stream[T](path, fn)` | ストリーミング処理（1 行ずつ変換・フィルタ） |
| `JSONL.append(path, row)` | 1 行追記 |

`cargo test jsonl` で 3 件以上 PASS。

---

### v27.7 — `fav infer --from delta` / `--from iceberg`

**テーマ**: データレイクのスキーマから Favnir の型定義を自動生成する。
「既存の Delta テーブルに繋ぎたい」という最初の 5 分の体験を改善する。

**依存関係**: v27.1（delta-lake）/ v27.2（iceberg）完了後

```bash
# Delta Lake テーブルからスキーマを自動生成
fav infer --from delta --path s3://my-bucket/orders
# → 出力:
# type OrderRow = {
#   id:         Int,
#   user_id:    String,
#   amount:     Float,
#   created_at: DateTime,
#   status:     String,
# }

# Iceberg テーブルから（REST カタログ）
fav infer --from iceberg \
  --catalog http://localhost:8181 \
  --table warehouse.orders
```

実装内容:
- `fav infer` コマンドに `--from delta` / `--from iceberg` サブコマンドを追加
- Delta / Iceberg のスキーマ（列名・型）を読み取り、Favnir の `type` 定義に変換
- 型マッピング: `long → Int` / `double → Float` / `string → String` / `timestamp → DateTime` / `boolean → Bool`

`fav infer --from delta --path /tmp/test_delta_table` でコード出力を確認する。

---

### v27.8 — dbt 連携

**テーマ**: dbt（data build tool）はデータ変換の事実上の標準。
`dbt ref()` で参照するモデルを Favnir パイプラインから型安全に読み込めるようにする。

**依存関係**: v25.1（postgres）または v27.4（bigquery）いずれか完了後

```favnir
import runes/dbt

// dbt モデルの出力を Favnir パイプラインで後処理
stage LoadCustomerSummary: Unit -> List<CustomerSummary> !Db = |_| {
  // manifest.json を解析して "customer_summary" モデルのクエリを生成
  bind result <- Dbt.ref[CustomerSummary](config.dbt, "customer_summary")
  Result.ok(result)
}

// dbt の source 定義を参照
stage LoadRawEvents: Unit -> List<RawEvent> !Db = |_| {
  bind rows <- Dbt.source[RawEvent](config.dbt, "raw", "events")
  Result.ok(rows)
}
```

実装内容:
- `runes/dbt/` を新規作成
- `Dbt.ref[T](config, model_name)` — `manifest.json` を解析して compiled SQL を実行し型付き結果を返す
- `Dbt.source[T](config, source_name, table_name)` — dbt source 定義から SQL を生成して実行
- `DbtConfig { project_dir: String, profiles_dir: String, target: String }` 型

`cargo test dbt` で 3 件以上 PASS（manifest.json フィクスチャを使ったモックテスト）。

---

### v27.9 — sqlite Rune 追加

**テーマ**: 依存ゼロ・ローカル動作の組み込み DB。
ローカル開発・軽量 ETL・テスト用 DB として広く使われる。
Favnir のテスト基盤（MockDb の代替）としても有用。

**依存関係**: なし（`rusqlite --features bundled` で依存ゼロ）

```favnir
import runes/sqlite

// ローカル SQLite で軽量 ETL
stage ProcessCsvToSqlite: String -> Unit !Io = |csv_path| {
  bind rows <- Csv.read[DataRow](csv_path)
  bind db   <- SQLite.open("./output.db")
  bind _    <- SQLite.execute(db, """
    CREATE TABLE IF NOT EXISTS data (
      id INTEGER PRIMARY KEY,
      name TEXT,
      value REAL
    )
  """)
  bind _ <- SQLite.execute_many(db, "INSERT INTO data VALUES (?, ?, ?)", rows)
  Result.ok(unit)
}
```

実装する関数:

| 関数 | 内容 |
|---|---|
| `SQLite.open(path)` | DB ファイルを開く（なければ作成） |
| `SQLite.open_memory()` | インメモリ DB（テスト用） |
| `SQLite.query[T](db, sql, params)` | 型付きクエリ |
| `SQLite.execute(db, sql, params)` | DDL / DML 実行 |
| `SQLite.execute_many(db, sql, rows)` | バッチ実行 |
| `SQLite.close(db)` | DB クローズ |

`cargo test sqlite` で 4 件以上 PASS（`SQLite.open_memory()` でインメモリテスト）。

---

## v28.0 — Data Lakehouse マイルストーン宣言

**完了条件:**

| コンポーネント | 完了基準 |
|---|---|
| delta-lake Rune | 5 条件クリア + 5 件テスト + examples/delta_lake_etl.fav 動作 |
| iceberg Rune | 5 条件クリア + 4 件テスト |
| clickhouse Rune | 5 条件クリア + 3 件テスト |
| bigquery Rune | 5 条件クリア + 4 件テスト（BigQuery Emulator）|
| redshift Rune | 5 条件クリア + 3 件テスト |
| jsonl Rune | 5 条件クリア + 3 件テスト |
| sqlite Rune | 5 条件クリア + 4 件テスト |
| `fav infer --from delta/iceberg` | コード出力が正しいこと |
| dbt 連携 Rune | 3 件テスト PASS |

**最終テスト（全件 PASS が完了条件）:**

```bash
# 1. 全 Rust テストが通る
cargo test

# 2. Delta Lake ETL デモ
fav run examples/delta_lake_etl.fav

# 3. fav infer でスキーマ自動生成
fav infer --from delta --path /tmp/test_table

# 4. 各 Rune テスト
cargo test delta_lake iceberg clickhouse bigquery redshift jsonl sqlite dbt
```

> 「Delta Lake テーブルを Favnir から型安全に読み書きし、
>  dbt モデルの結果を次のステージに渡す」
> = Data Lakehouse の完成を象徴するデモ

---

## 完了マーク

**v28.0.0 COMPLETE** — 2026-06-27
全コンポーネント（v27.1〜v27.9）実装完了。`cargo test --bin fav` 2226 tests PASS。
Data Lakehouse マイルストーン宣言済み。詳細は `MILESTONE.md` を参照。

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v25.1-v30.0.md`
- 前フェーズ: `versions/roadmap/roadmap-v26.1-v27.0.md`
- 次フェーズ: `versions/roadmap/roadmap-v28.1-v29.0.md`
