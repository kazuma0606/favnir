# v26.7.0 タスクリスト — ストリーミング E2E デモ（nats → postgres）

**状態**: COMPLETE
**開始日**: 2026-06-27
**完了日**: 2026-06-27

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | 事前確認: `Cargo.toml` が `26.6.0`、テスト数 2086 件、`nats_to_postgres.fav` 未存在、docker-compose.yml に `nats` がないことを確認 | [x] |
| T1 | `fav/Cargo.toml` を `version = "26.7.0"` に bump | [x] |
| T2 | `examples/streaming/nats_to_postgres.fav` 新規作成（FetchSensorData / ValidateSensor / InsertToPostgres + `seq SensorPipeline`） | [x] |
| T3 | `examples/streaming/docker-compose.yml` を Edit: `nats` / `postgres` サービスを末尾に追記（既存サービスは変更しない） | [x] |
| T4 | `examples/streaming/README.md` 新規作成（3 本デモ実行手順まとめ） | [x] |
| T5 | `site/content/docs/streaming/nats-to-postgres.mdx` 新規作成 | [x] |
| T6 | `CHANGELOG.md` 更新: 先頭に `[v26.7.0]` エントリ追加 | [x] |
| T7 | `benchmarks/v26.7.0.json` 新規作成（test_count: 2094） | [x] |
| T8 | `fav/src/driver.rs` 更新: `v267000_tests`（8 件）を `v266000_tests` の直後に追加 | [x] |
| T8.5 | `cargo test v267000 --bin fav` — 8/8 PASS 確認 | [x] |
| T9 | `cargo test --bin fav` — 2094 件 PASS 確認（リグレッションなし） | [x] |
| T10 | spec-reviewer レビュー実施（実装前・本タスクで完了済み） | [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "26.7.0"` であること
- [x] `examples/streaming/nats_to_postgres.fav` が存在すること
- [x] デモに `seq SensorPipeline` が含まれること
- [x] デモに `stage FetchSensorData` が含まれること
- [x] デモに `stage ValidateSensor` が含まれること
- [x] デモに `stage InsertToPostgres` が含まれること
- [x] デモに `NATS.subscribe` 呼び出しが含まれること
- [x] デモに `Postgres.execute` 呼び出しが含まれること
- [x] デモに `"sensor_readings"` が含まれること
- [x] `examples/streaming/docker-compose.yml` に `nats` サービスが追加されていること
- [x] `examples/streaming/docker-compose.yml` に `postgres` サービスが追加されていること
- [x] `examples/streaming/README.md` が存在し、3 本のデモ名（kafka_to_elasticsearch / kinesis_to_s3 / nats_to_postgres）がすべて含まれること
- [x] `site/content/docs/streaming/nats-to-postgres.mdx` が存在すること
- [x] `CHANGELOG.md` に `[v26.7.0]` エントリが存在すること
- [x] `benchmarks/v26.7.0.json` が存在すること（test_count: 2094）
- [x] `v267000_tests` 8 件すべて PASS
- [x] 総テスト数 ≥ 2094 件

---

## メモ

### NATS Rune の API（`runes/nats/nats.fav` より）

```
connect(url: String) -> Result<NatsConn, String> !Stream
subscribe(conn: NatsConn, subject: String) -> Result<String, String> !Stream
jetstream_publish(conn: NatsConn, stream: String, payload: String) -> Result<String, String> !Stream
jetstream_consume(conn: NatsConn, stream: String, consumer: String) -> Result<String, String> !Stream
```

`import rune "nats"` → `NATS.*` として使用。

### Postgres Rune の API（`runes/postgres/client.fav` より）

```
execute(sql: String, params: String) -> Result<Unit, String> !Postgres
```

`import rune "postgres"` → `Postgres.*`（`use client.{ ..., execute, ... }` で公開済み）。

params は JSON 配列文字列。`sensor_json` が JSON オブジェクトの場合 `"[" ++ sensor_json ++ "]"` で渡す。

### `include_str!` パス（`fav/src/driver.rs` 基準）

```rust
include_str!("../../examples/streaming/nats_to_postgres.fav")
include_str!("../../examples/streaming/docker-compose.yml")
include_str!("../../examples/streaming/README.md")
include_str!("../../CHANGELOG.md")
```

`fav/src/driver.rs` から `../` で `fav/`、さらに `../` でプロジェクトルート（`favnir/`）に出る。

### docker-compose.yml の Edit 方針

`localstack` サービスの後に `nats` と `postgres` の 2 サービスを**末尾追記**した。
kafka / elasticsearch / localstack サービスは変更しなかった。

### NATS の起動フラグと healthcheck

`nats:2.10-alpine` の起動フラグは `-js`（JetStream）と `-m "8222"`（HTTP monitoring）の両方が必要。
`-m` がないと 8222 ポートが開かず healthcheck の `wget http://localhost:8222/healthz` が永続失敗する。
`wget` コマンドを使用（alpine 標準搭載。`curl` は含まれないため注意）。

### `v267000_tests` に `use super::*` は不要

`include_str!` のみ使用のため `use super::*` は必要ない（v266000_tests と同じ）。

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [MED] `postgres` サービスに `volumes` マウントがなく、`down` 時データ消失の旨が README に未記載 | `README.md` 停止セクションに「`down` でデータ消失、永続化には `stop` を使用」注記を追加 |
| [LOW] `InsertToPostgres` の `"[" ++ sensor_json ++ "]"` は invalid JSON 時に Postgres キャスト例外が発生する可能性 | v27.x スコープ外（spec.md にも記載）として変更なし |
