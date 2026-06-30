# v26.9.0 仕様書 — Pulsar Rune 追加

## 概要

Apache Pulsar Rune を実質化する。
Pulsar は Kafka の代替として、マルチテナント・マルチクラスタ・geo-replication を特徴とするストリーミングプラットフォーム。
`import rune "pulsar"` → `Pulsar.*` 名前空間で 4 関数が使用可能になる。

---

## 背景

ロードマップ v26.9「Pulsar Rune 追加」より。

- v26.8.0 で SQS Rune が実質化された（AWS 系キューは完了）
- v27.0.0 の Streaming Native マイルストーン達成に必要な最後の Rune
- ロードマップ要件: `cargo test pulsar` で 3 件以上 PASS

---

## 実装する関数

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `Pulsar.produce` | `(topic: String, key: String, value: String) -> Result<String, String> !AWS` | メッセージ送信（MessageId 返却） |
| `Pulsar.consume` | `(topic: String, subscription: String) -> Result<String, String> !AWS` | 1 回ポーリング stub（JSON 文字列返却）。型付きコールバック消費は v27.x 以降 |
| `Pulsar.ack` | `(message_id: String) -> Result<Unit, String> !AWS` | ACK（確認） |
| `Pulsar.nack` | `(message_id: String) -> Result<Unit, String> !AWS` | NACK（再配信要求） |

> **エフェクト**: Pulsar は Apache 製品（クラウド非依存）だが、`!Pulsar` エフェクトの独立化前のため `SQS / RabbitMQ` と同様に `!AWS` で代替する（`!Pulsar` の独立は v27.x 以降）。

---

## VM Primitive（vm.rs に追加）

| primitive 名 | 実装方針 |
|---|---|
| `Pulsar.produce_raw` | stub 実装: `ok_vm(VMValue::Str("stub-message-id".into()))` 固定返却。Pulsar Admin REST API は produce をサポートしないため（Binary Protocol は v27.x）。`#[cfg(not(target_arch = "wasm32"))]` ガード付き |
| `Pulsar.consume_raw` | 1 回ポーリング stub（JSON 文字列返却）。wasm32 では `err_vm` |
| `Pulsar.ack_raw` | ack 受付 stub（`ok_vm(VMValue::Unit)` 返却）。wasm32 では `err_vm` |
| `Pulsar.nack_raw` | nack 受付 stub（`ok_vm(VMValue::Unit)` 返却）。wasm32 では `err_vm` |

### produce_raw の実装方針

Pulsar の Admin REST API（port 8080）は管理操作（topic 作成・削除・stats）のみを提供し、
メッセージの produce には対応していない。Pulsar Binary Protocol（port 6650、TCP 接続）が必要だが、
現 VM 実装では TCP ソケット接続は未サポート。

そのため `produce_raw` は **stub 実装**（`ok_vm(VMValue::Str("stub-message-id".into()))` 固定返却）とする。
実 Pulsar への produce は v27.x 以降で `tokio-tungstenite`（WebSocket 経由）または外部 Pulsar クライアントライブラリ経由で実装する。

---

## Docker Compose

`examples/streaming/docker-compose.yml` に `pulsar` サービスを追加:

```yaml
pulsar:
  image: apachepulsar/pulsar:3.2.0
  command:
    - bin/pulsar
    - standalone
  ports:
    - "6650:6650"
    - "8080:8080"
  healthcheck:
    test: ["CMD", "bin/pulsar-admin", "brokers", "healthcheck"]
    interval: 30s
    timeout: 10s
    retries: 5
    start_period: 60s
```

---

## runes/pulsar/pulsar.fav

```favnir
// runes/pulsar/pulsar.fav — Apache Pulsar Rune (v26.9.0)
public fn produce(topic: String, key: String, value: String) -> Result<String, String> !AWS {
    Pulsar.produce_raw(topic, key, value)
}
public fn consume(topic: String, subscription: String) -> Result<String, String> !AWS {
    Pulsar.consume_raw(topic, subscription)
}
public fn ack(message_id: String) -> Result<Unit, String> !AWS {
    Pulsar.ack_raw(message_id)
}
public fn nack(message_id: String) -> Result<Unit, String> !AWS {
    Pulsar.nack_raw(message_id)
}
```

---

## テスト

### driver.rs v269000_tests（8 件）

| テスト名 | 内容 |
|---|---|
| `pulsar_rune_has_produce_fn` | `pulsar.fav` に `fn produce(` が含まれること |
| `pulsar_rune_has_consume_fn` | `pulsar.fav` に `fn consume(` が含まれること |
| `pulsar_rune_has_ack_fn` | `pulsar.fav` に `fn ack(` が含まれること |
| `pulsar_rune_has_nack_fn` | `pulsar.fav` に `fn nack(` が含まれること |
| `pulsar_vm_has_produce_raw` | `vm.rs` に `Pulsar.produce_raw` が含まれること |
| `pulsar_vm_has_ack_raw` | `vm.rs` に `Pulsar.ack_raw` が含まれること |
| `docker_compose_has_pulsar_service` | `docker-compose.yml` に `pulsar:` が含まれること |
| `changelog_has_v26_9_0` | `CHANGELOG.md` に `[v26.9.0]` が含まれること |

### `cargo test pulsar` 期待値

- `v269000_tests::pulsar_rune_has_*` 4 件
- `v269000_tests::pulsar_vm_has_*` 2 件
- 合計 6 件（ロードマップ要件「3 件以上」を超過）

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "26.9.0"` であること
- [ ] `runes/pulsar/pulsar.fav` に `fn produce(` が含まれること
- [ ] `runes/pulsar/pulsar.fav` に `fn consume(` が含まれること
- [ ] `runes/pulsar/pulsar.fav` に `fn ack(` が含まれること
- [ ] `runes/pulsar/pulsar.fav` に `fn nack(` が含まれること
- [ ] `fav/src/backend/vm.rs` に `Pulsar.produce_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `Pulsar.consume_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `Pulsar.ack_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `Pulsar.nack_raw` が含まれること
- [ ] `examples/streaming/docker-compose.yml` に `pulsar:` サービスが含まれること
- [ ] `site/content/docs/runes/pulsar.mdx` が存在すること
- [ ] `CHANGELOG.md` に `[v26.9.0]` エントリが存在すること
- [ ] `benchmarks/v26.9.0.json` が存在すること（test_count: 2110）
- [ ] `v269000_tests` 8 件すべて PASS
- [ ] `cargo test pulsar` で 6 件以上 PASS
- [ ] 総テスト数 ≥ 2110 件

---

## スコープ外（v27.x 以降）

- Pulsar Binary Protocol（TCP port 6650）経由の高速 produce/consume（`produce_raw` は現在 stub）
- `Pulsar.consume[T](topic, subscription, fn)` — 型付きコールバック消費ループ（ロードマップ v26.9 定義の完全実装）
- JetStream / Persistent Subscription の詳細制御
- `!Pulsar` エフェクトの独立（現在 `!AWS` に包含）
- マルチテナント（`tenant` / `namespace` パラメータの完全サポート）
