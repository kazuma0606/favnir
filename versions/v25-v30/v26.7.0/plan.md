# v26.7.0 実装計画 — ストリーミング E2E デモ（nats → postgres）

## 実装方針

- 新規 Cargo 依存・Rust コードは追加しない（既存の nats / postgres Rune を使用）
- `examples/streaming/nats_to_postgres.fav` を新規作成する
- `examples/streaming/docker-compose.yml` に `nats` / `postgres` サービスを**追加**（既存の kafka / elasticsearch / localstack サービスは変更しない）
- `examples/streaming/README.md` を新規作成する（3 本デモの実行手順まとめ）
- `site/content/docs/streaming/nats-to-postgres.mdx` を新規作成する

---

## 実装ステップ

### Step 0: 事前確認

```bash
grep 'version = ' fav/Cargo.toml                              # "26.6.0" であること
cat benchmarks/v26.6.0.json                                   # "test_count":2086 であること
cargo test --bin fav 2>&1 | tail -3                           # 2086 件 PASS であること
ls examples/streaming/nats_to_postgres.fav 2>/dev/null || echo "not found"  # 未存在であること
grep 'nats' examples/streaming/docker-compose.yml || echo "not found"       # 未存在であること
```

### Step 1: `fav/Cargo.toml` bump（26.6.0 → 26.7.0）

```toml
version = "26.7.0"
```

### Step 2: `examples/streaming/nats_to_postgres.fav` 新規作成

spec.md §1 の内容を実装。3 ステージ + `seq SensorPipeline`:

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

> **NATS Rune の名前空間確認**: `import rune "nats"` → `NATS.*`（`nats.fav` の `public fn connect/subscribe` が対応）
> **Postgres Rune の名前空間確認**: `import rune "postgres"` → `Postgres.*`（`client.fav` の `public fn execute` が対応）

### Step 3: `examples/streaming/docker-compose.yml` に nats / postgres 追加

既存ファイルを Read してから Edit で `nats` と `postgres` サービスを末尾に追加する:

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

> 既存の kafka / elasticsearch / localstack サービスは変更しない。**Edit（追記）** のみ行う。

### Step 4: `examples/streaming/README.md` 新規作成

3 本の E2E デモ（kafka→elasticsearch / kinesis→s3 / nats→postgres）の
実行手順を 1 ファイルにまとめる。各デモの起動コマンド・環境変数・実行例を記載する。

### Step 5: `site/content/docs/streaming/nats-to-postgres.mdx` 新規作成

既存の `site/content/docs/streaming/kafka-to-elasticsearch.mdx` の形式（見出し構成・コードブロック言語指定）に合わせて作成する。

- パイプライン概要（FetchSensorData → ValidateSensor → InsertToPostgres）
- セットアップ手順（`docker compose ... up -d --wait`）
- 環境変数一覧（NATS_URL / DATABASE_URL）
- 各ステージ解説
- スコープ外

### Step 6: `CHANGELOG.md` 更新

```markdown
## [v26.7.0] — 2026-06-27 — ストリーミング E2E デモ（nats → postgres）

### Added
- `examples/streaming/nats_to_postgres.fav` — NATS → Postgres IoT センサーデータ蓄積デモ（FetchSensorData / ValidateSensor / InsertToPostgres + `seq SensorPipeline`）
- `examples/streaming/docker-compose.yml` に `nats` / `postgres` サービス追加
- `examples/streaming/README.md` — 3 本の E2E デモ実行手順まとめ
- `site/content/docs/streaming/nats-to-postgres.mdx` — E2E デモドキュメント
```

### Step 7: `benchmarks/v26.7.0.json` 新規作成

```json
{"version":"26.7.0","test_count":2094,"timestamp":"2026-06-27"}
```

### Step 8: `fav/src/driver.rs` に `v267000_tests` 追加

> **前提**: Step 2（`nats_to_postgres.fav` 作成）・Step 3（docker-compose.yml 更新）・Step 4（README.md 作成）が完了していること。
> `include_str!` マクロはコンパイル時にファイルを要求するため、ファイルが存在しない状態でこのステップを実行するとコンパイルエラーになる。

`v266000_tests` の直後に追加（8 件）:

```rust
// ── v267000_tests (v26.7.0) — nats → postgres E2E デモ ───────────────────────
#[cfg(test)]
mod v267000_tests {
    #[test]
    fn nats_to_postgres_demo_file_exists() {
        let src = include_str!("../../examples/streaming/nats_to_postgres.fav");
        assert!(!src.is_empty(), "nats_to_postgres.fav must not be empty");
    }
    #[test]
    fn nats_to_postgres_demo_has_subscribe() {
        let src = include_str!("../../examples/streaming/nats_to_postgres.fav");
        assert!(src.contains("subscribe"), "demo must call NATS.subscribe");
    }
    #[test]
    fn nats_to_postgres_demo_has_execute() {
        let src = include_str!("../../examples/streaming/nats_to_postgres.fav");
        assert!(src.contains("Postgres.execute("), "demo must call Postgres.execute");
    }
    #[test]
    fn nats_to_postgres_demo_has_sensor_pipeline() {
        let src = include_str!("../../examples/streaming/nats_to_postgres.fav");
        assert!(src.contains("SensorPipeline"), "demo must define SensorPipeline");
    }
    #[test]
    fn nats_to_postgres_demo_has_sensor_readings() {
        let src = include_str!("../../examples/streaming/nats_to_postgres.fav");
        assert!(src.contains("sensor_readings"), "demo must reference sensor_readings table");
    }
    #[test]
    fn streaming_docker_compose_has_nats() {
        let src = include_str!("../../examples/streaming/docker-compose.yml");
        assert!(src.contains("nats:"), "docker-compose.yml must define nats service");
    }
    #[test]
    fn streaming_readme_exists() {
        let src = include_str!("../../examples/streaming/README.md");
        assert!(src.contains("kafka_to_elasticsearch") && src.contains("kinesis_to_s3") && src.contains("nats_to_postgres"), "README.md must reference all three demos");
    }
    #[test]
    fn changelog_has_v26_7_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("[v26.7.0]"), "CHANGELOG.md must contain '[v26.7.0]'");
    }
}
```

### Step 9: テスト確認

```bash
cd fav && cargo test v267000 --bin fav          # 8/8 PASS
cd fav && cargo test --bin fav -j 8 -- --test-threads=8 2>&1 | tail -4  # 2094 件 PASS
```

---

## ファイル変更一覧

| ファイル | 操作 |
|---|---|
| `fav/Cargo.toml` | version bump 26.6.0 → 26.7.0 |
| `examples/streaming/nats_to_postgres.fav` | **新規作成**（3 ステージ + seq） |
| `examples/streaming/docker-compose.yml` | `nats` / `postgres` サービス追記（Edit） |
| `examples/streaming/README.md` | **新規作成**（3 本デモ実行手順） |
| `site/content/docs/streaming/nats-to-postgres.mdx` | **新規作成** |
| `CHANGELOG.md` | `[v26.7.0]` エントリ先頭に追加 |
| `benchmarks/v26.7.0.json` | **新規作成** |
| `fav/src/driver.rs` | `v267000_tests`（8 件）追加 |

---

## 注意事項

- `NATS.connect("")` — 空文字列渡しは既存 `Kafka.connect("")` と同じ慣用パターン。VM primitive が `NATS_URL` 環境変数をフォールバックで読む。
- `Postgres.execute(sql, params)` — params は JSON 配列文字列。`sensor_json` が JSON オブジェクトの場合 `"[" ++ sensor_json ++ "]"` で配列に包む。
- NATS の起動フラグ — `-js`（JetStream 有効化）と `-m "8222"`（HTTP monitoring ポート有効化）の両方が必要。`-m` がないと `8222` ポートが開かず healthcheck が永続失敗する。
- `nats:2.10-alpine` の healthcheck には `wget`（alpine に標準搭載）を使用。`curl` は alpine イメージに含まれないため注意。
- docker-compose.yml は **Edit（追記）** する。既存サービス（kafka / elasticsearch / localstack）は変更しないこと。
- README.md と MDX は kafka-to-elasticsearch.mdx の形式に合わせること。

## リスクと対応

| リスク | 対応 |
|---|---|
| `nats:2.10-alpine` の healthcheck で `wget` が使えない | nats:2.10-alpine には wget が同梱されている（alpine 標準パッケージ） |
| `Postgres.execute` の params 形式不正 | `"[" ++ sensor_json ++ "]"` は sensor_json が有効な JSON の場合に正しい配列形式になる |
| docker-compose.yml の Edit で既存サービスが壊れる | 末尾への追記のみ。既存サービスは変更しない |
| `include_str!` パスの誤り | `fav/src/driver.rs` から `../../examples/streaming/` — 既存テストと同パターン |
