# v26.7.0 仕様書 — ストリーミング E2E デモ（nats → postgres）

## 概要

| 項目 | 内容 |
|---|---|
| バージョン | v26.7.0 |
| フェーズ | Streaming Native（v26.1〜v27.0） |
| テーマ | NATS → Postgres IoT センサーデータ蓄積パイプライン E2E デモ |
| 依存関係 | v25.1.0（Postgres Rune）・v26.2.0（NATS Rune）・v26.4.0（Stream.* 操作）完了後 |
| 目標テスト数 | 2094 件（+8 件）|

---

## 背景と目的

v26.5.0 で kafka → elasticsearch、v26.6.0 で kinesis → s3 の E2E デモを実装した。
v26.7.0 では Streaming Native フェーズ第 3（最終）E2E デモとして、
NATS から届く IoT センサーデータを Postgres に蓄積するパイプラインを実装する。

「NATS → Postgres」はマイクロサービス / IoT アーキテクチャの典型的なパターン。
JetStream 対応の nats-server Docker イメージでローカル実行できることを目標とする。

また、v26.7.0 では `examples/streaming/README.md` を新規作成し、
3 本の E2E デモ（kafka → elasticsearch / kinesis → s3 / nats → postgres）の
実行手順を 1 か所にまとめる。

### 利用する Rune

| Rune | import | 名前空間 | 使用関数 | バージョン |
|---|---|---|---|---|
| nats | `import rune "nats"` | `NATS.*` | `connect` / `subscribe` | v26.2.0 |
| postgres | `import rune "postgres"` | `Postgres.*` | `execute` | v25.1.0 |

### ロードマップとの API 設計差異

ロードマップ v26.7 節のデモコードは以下の理想 API を示している:

```favnir
NATS.subscribe[SensorReading]("sensors.>")
|> ValidateSensor
|> EnrichWithMetadata
|> Postgres.insert("sensor_readings")
```

v26.7.0 では実際の NATS/Postgres Rune API を使う:

```
FetchSensorData(Unit -> String)   — NATS.connect("") + NATS.subscribe(conn, "sensors.data")
  |> ValidateSensor(String -> String)  — 空ペイロードチェック（スタブ）
  |> InsertToPostgres(String -> String) — Postgres.execute(INSERT SQL, params)
```

追加の差異:
- ロードマップの `NATS.subscribe[SensorReading]("sensors.>")` はワイルドカード + 型付きの 2 引数（subject, コールバック fn）だが、
  実際の `nats.fav` は `subscribe(conn, subject)` で conn オブジェクトを第 1 引数に取り fn 引数（コールバックモデル）は存在しない。
  v26.x はポーリングモデル。コールバック型 API は v27.x スコープ外。
  戻り値は `Result<String, String>`（JSON 文字列）。
- ロードマップの `EnrichWithMetadata` ステージは `nats.fav` に対応関数がなく省略（v27.x スコープ外）。
- ロードマップの `ValidateSensor` は `Float.to_string(reading.value)` を使うが、
  型付き構造体デシリアライズ（`SensorReading` 型）は v26.x スコープ外のため、
  `String.length(payload) > 0` による空チェックスタブに置き換える。
- ロードマップの `Postgres.insert("sensor_readings")` は高レベル API だが、
  実際は `Postgres.execute(sql, params)` で INSERT SQL を直接記述する。
- subject: ロードマップは `"sensors.>"` ワイルドカードだが実際は `"sensors.data"` を使用。
- params 形式: `Postgres.execute` の第 2 引数は JSON 配列文字列（`"[...]"` 形式）。
  sensor_json が JSON オブジェクト文字列の場合、`"[" ++ sensor_json ++ "]"` として渡す。

---

## 機能仕様

### 1. `examples/streaming/nats_to_postgres.fav`

```favnir
import rune "nats"
import rune "postgres"

// ── NATS → Postgres IoT センサーデータ蓄積デモ (v26.7.0) ──────────────────────
// 前提: docker compose -f examples/streaming/docker-compose.yml up -d
// 実行: fav run examples/streaming/nats_to_postgres.fav
//
// 環境変数:
//   NATS_URL    — NATS サーバー URL（省略: "nats://localhost:4222"）
//   DATABASE_URL — Postgres 接続文字列（省略: "host=localhost port=5432 user=favnir password=favnir dbname=sensors"）

// 1. NATS の sensors.data サブジェクトからセンサー読み取りを受信
// Note: NATS.connect("") の空文字列は VM primitive 内で NATS_URL 環境変数に
//       フォールバックする（backend/vm.rs の NatsConnectRaw 実装参照）。
stage FetchSensorData: Unit -> Result<String, String> !Stream = |_| {
    bind conn <- NATS.connect("")
    NATS.subscribe(conn, "sensors.data")
}

// 2. 空ペイロードをスキップ（スタブ: 実プロジェクトでは値範囲チェックに拡張可能）
stage ValidateSensor: String -> Result<String, String> !Pure = |payload| {
    if String.length(payload) > 0
    then Result.ok(payload)
    else Result.err("empty sensor payload — skipping")
}

// 3. Postgres の sensor_readings テーブルに INSERT
// Note: Postgres.execute は Result<Unit, String> を返す。Unit を _ で捨てて Result<String, String> に変換。
//       params は JSON 配列形式。sensor_json（JSONオブジェクト文字列）を配列で包んで渡す。
stage InsertToPostgres: String -> Result<String, String> !Postgres = |sensor_json| {
    bind _ <- Postgres.execute(
        "INSERT INTO sensor_readings (data, received_at) VALUES ($1::jsonb, NOW())",
        "[" ++ sensor_json ++ "]"
    )
    Result.ok("inserted sensor reading into sensor_readings")
}

seq SensorPipeline = FetchSensorData |> ValidateSensor |> InsertToPostgres
```

### 2. `examples/streaming/docker-compose.yml` 更新

v26.6.0 で更新済みのファイルに **nats** と **postgres** サービスを追加する。

追加するサービス:

```yaml
  nats:
    image: nats:2.10-alpine
    command:
      - -js
      - -m
      - "8222"
    ports:
      - "4222:4222"
      - "8222:8222"
    healthcheck:
      test: ["CMD", "wget", "--quiet", "--tries=1", "--spider", "http://localhost:8222/healthz"]
      interval: 10s
      timeout: 5s
      retries: 10

  postgres:
    image: postgres:16-alpine
    environment:
      - POSTGRES_USER=favnir
      - POSTGRES_PASSWORD=favnir
      - POSTGRES_DB=sensors
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U favnir -d sensors"]
      interval: 10s
      timeout: 5s
      retries: 10
```

> 既存の kafka / elasticsearch / localstack サービスは変更しない。

### 3. `examples/streaming/README.md` 新規作成

3 本の E2E デモ（kafka → elasticsearch / kinesis → s3 / nats → postgres）の
起動手順・環境変数・実行コマンドを 1 か所にまとめる。

### 4. `site/content/docs/streaming/nats-to-postgres.mdx` 新規作成

既存の `site/content/docs/streaming/kafka-to-elasticsearch.mdx` の形式に合わせて作成。

- パイプライン概要（FetchSensorData → ValidateSensor → InsertToPostgres）
- セットアップ手順（docker compose up -d --wait）
- 環境変数一覧（NATS_URL / DATABASE_URL）
- 各ステージ解説
- スコープ外

---

## スコープ外（v27.x 以降）

- 型付きデシリアライズ（`SensorReading` 構造体 → `value: Float` フィールドアクセス）
- `Float.to_string(reading.value)` による数値範囲チェック
- `NATS.subscribe[T]("sensors.>")` ワイルドカード + 型付き消費
- `EnrichWithMetadata` ステージ（センサーメタデータ付与）
- JetStream 消費（`NATS.jetstream_consume`）による ACK / 永続化保証
- Postgres コネクションプール（`Pool.create`）

---

## Rust テスト（v267000_tests、8 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `nats_to_postgres_demo_file_exists` | `examples/streaming/nats_to_postgres.fav` が存在する | assert |
| `nats_to_postgres_demo_has_subscribe` | デモに `subscribe` が含まれる | assert |
| `nats_to_postgres_demo_has_execute` | デモに `Postgres.execute(` が含まれる | assert |
| `nats_to_postgres_demo_has_sensor_pipeline` | デモに `SensorPipeline` が含まれる | assert |
| `nats_to_postgres_demo_has_sensor_readings` | デモに `sensor_readings` が含まれる | assert |
| `streaming_docker_compose_has_nats` | docker-compose.yml に `nats:` が含まれる | assert |
| `streaming_readme_exists` | `examples/streaming/README.md` に 3 本のデモ名が含まれる | assert |
| `changelog_has_v26_7_0` | `CHANGELOG.md` に `[v26.7.0]` が含まれる | assert |

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "26.7.0"` であること
- [ ] `examples/streaming/nats_to_postgres.fav` が存在すること
- [ ] デモに `seq SensorPipeline` が含まれること
- [ ] デモに `stage FetchSensorData` が含まれること
- [ ] デモに `stage ValidateSensor` が含まれること
- [ ] デモに `stage InsertToPostgres` が含まれること
- [ ] デモに `NATS.subscribe` 呼び出しが含まれること
- [ ] デモに `Postgres.execute` 呼び出しが含まれること
- [ ] デモに `"sensor_readings"` が含まれること
- [ ] `examples/streaming/docker-compose.yml` に `nats` サービスが追加されていること
- [ ] `examples/streaming/docker-compose.yml` に `postgres` サービスが追加されていること
- [ ] `examples/streaming/README.md` が存在し、3 本のデモ名（kafka_to_elasticsearch / kinesis_to_s3 / nats_to_postgres）がすべて含まれること
- [ ] `site/content/docs/streaming/nats-to-postgres.mdx` が存在すること
- [ ] `CHANGELOG.md` に `[v26.7.0]` エントリが存在すること
- [ ] `benchmarks/v26.7.0.json` が存在すること（test_count: 2094）
- [ ] `v267000_tests` 8 件すべて PASS
- [ ] 総テスト数 ≥ 2094 件

---

## テスト件数

- v26.6.0 完了時: 2086 件
- v26.7.0 追加: 8 件（v267000_tests）
- **目標**: 2086 + 8 = **2094 件**

> `benchmarks/v26.6.0.json` で `test_count: 2086` を **Step 0 で確認すること**（実装開始前の前提条件）。
