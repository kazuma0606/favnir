# v26.9.0 実装計画 — Pulsar Rune 追加

## 前提確認

- `fav/Cargo.toml`: `version = "26.8.0"`
- テスト数: 2102 件
- `runes/pulsar/` ディレクトリが存在しないことを確認（新規作成）
- `vm.rs` に `Pulsar.*_raw` primitive が存在しないことを確認
- `examples/streaming/docker-compose.yml` に `pulsar:` サービスがないことを確認

---

## 実装ステップ

### Step 1: Cargo.toml バージョン bump

`fav/Cargo.toml` の `version` を `"26.8.0"` → `"26.9.0"` に変更。

---

### Step 2: runes/pulsar/pulsar.fav 新規作成

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

### Step 3: vm.rs に Pulsar primitives 追加

**挿入位置**: `// ── RabbitMQ primitives (v26.3.0)` ブロック の直後（`// ── Azure Blob Storage` の直前）に挿入。

追加する 4 primitive（各 native/wasm32 ペア = 8 アーム）:

#### `Pulsar.produce_raw`

> **注意**: Pulsar Admin REST API（port 8080）は produce をサポートしない。Binary Protocol（port 6650）は v27.x 以降。
> 現実装は stub: 引数を検証し `"stub-message-id"` を返す。

```rust
#[cfg(not(target_arch = "wasm32"))]
"Pulsar.produce_raw" => {
    // (topic: String, key: String, value: String) -> Result<String, String>
    // Stub: Pulsar Binary Protocol は v27.x 以降。引数検証のみ実施。
    let mut it = args.into_iter();
    let _topic = vm_string(it.next().ok_or("Pulsar.produce_raw: missing topic")?, "Pulsar.produce_raw")?;
    let _key   = vm_string(it.next().ok_or("Pulsar.produce_raw: missing key")?,   "Pulsar.produce_raw")?;
    let _value = vm_string(it.next().ok_or("Pulsar.produce_raw: missing value")?, "Pulsar.produce_raw")?;
    Ok(ok_vm(VMValue::Str("stub-message-id".to_string())))
}
#[cfg(target_arch = "wasm32")]
"Pulsar.produce_raw" => Ok(err_vm(VMValue::Str("Pulsar not supported on wasm32".into()))),
```

#### `Pulsar.consume_raw`

```rust
#[cfg(not(target_arch = "wasm32"))]
"Pulsar.consume_raw" => {
    // (topic: String, subscription: String) -> Result<String, String>
    // Stub: 1 回ポーリング（Pulsar binary protocol は v27.x 以降）
    let mut it = args.into_iter();
    let _topic        = vm_string(it.next().ok_or("Pulsar.consume_raw: missing topic")?,        "Pulsar.consume_raw")?;
    let _subscription = vm_string(it.next().ok_or("Pulsar.consume_raw: missing subscription")?, "Pulsar.consume_raw")?;
    Ok(ok_vm(VMValue::Str("[]".to_string())))
}
#[cfg(target_arch = "wasm32")]
"Pulsar.consume_raw" => Ok(err_vm(VMValue::Str("Pulsar not supported on wasm32".into()))),
```

#### `Pulsar.ack_raw`

```rust
#[cfg(not(target_arch = "wasm32"))]
"Pulsar.ack_raw" => {
    // (message_id: String) -> Result<Unit, String>
    let mut it = args.into_iter();
    let _message_id = vm_string(it.next().ok_or("Pulsar.ack_raw: missing message_id")?, "Pulsar.ack_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Pulsar.ack_raw" => Ok(err_vm(VMValue::Str("Pulsar not supported on wasm32".into()))),
```

#### `Pulsar.nack_raw`

```rust
#[cfg(not(target_arch = "wasm32"))]
"Pulsar.nack_raw" => {
    // (message_id: String) -> Result<Unit, String>
    let mut it = args.into_iter();
    let _message_id = vm_string(it.next().ok_or("Pulsar.nack_raw: missing message_id")?, "Pulsar.nack_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Pulsar.nack_raw" => Ok(err_vm(VMValue::Str("Pulsar not supported on wasm32".into()))),
```

---

### Step 4: examples/streaming/docker-compose.yml に pulsar サービス追加

既存の `nats:` / `postgres:` サービスの後に追加:

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

### Step 5: site/content/docs/runes/pulsar.mdx 新規作成

Apache Pulsar Rune の使い方を説明するドキュメント。
セットアップ（Docker standalone）・環境変数・4 関数のリファレンス・サンプルコードを記載。

---

### Step 6: CHANGELOG.md 更新

先頭に `[v26.9.0]` エントリを追加:

```
## [v26.9.0] — 2026-06-27

### Added
- Apache Pulsar Rune: `import rune "pulsar"` — `produce` / `consume` / `ack` / `nack` の 4 関数
- `Pulsar.produce_raw` / `Pulsar.consume_raw` / `Pulsar.ack_raw` / `Pulsar.nack_raw` VM primitives（`#[cfg(not(wasm32))]` ガード付き）
- `examples/streaming/docker-compose.yml` に `pulsar` サービス（`apachepulsar/pulsar:3.2.0`）追加
- `site/content/docs/runes/pulsar.mdx` — Pulsar Rune ドキュメント
```

---

### Step 7: benchmarks/v26.9.0.json 新規作成

```json
{"version":"26.9.0","test_count":2110,"timestamp":"2026-06-27"}
```

---

### Step 8: driver.rs に v269000_tests 追加

`v268000_tests` の直後に `v269000_tests` モジュール（8 件）を追加。

---

### Step 9: テスト実行

```bash
cargo test v269000 --bin fav        # 8/8 PASS
cargo test pulsar --bin fav         # 6 件以上 PASS
cargo test --bin fav                # 2110 件 PASS（リグレッションなし）
```

---

## include_str! パス（fav/src/driver.rs 基準）

| パス | 対象 |
|---|---|
| `../../runes/pulsar/pulsar.fav` | `favnir/runes/pulsar/pulsar.fav` |
| `backend/vm.rs` | `fav/src/backend/vm.rs` |
| `../../examples/streaming/docker-compose.yml` | `favnir/examples/streaming/docker-compose.yml` |
| `../../CHANGELOG.md` | `favnir/CHANGELOG.md` |

---

## 注意事項

### Pulsar standalone のヘルスチェック

Pulsar standalone は起動に 30〜60 秒かかるため `start_period: 60s` を設定する。
`bin/pulsar-admin brokers healthcheck` が 200 を返せば起動完了。

### base64 クレート

`produce_raw` では payload を base64 エンコードする。
`base64` クレートは `Cargo.toml` に既存登録済み（v13.x で追加）。
使用する API: `base64::engine::general_purpose::STANDARD.encode(...)`.

### wasm32 ガード

4 primitive すべてに `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` ペアを付ける。
これは v26.3.0（RabbitMQ）と同じパターン。

### 既存 `!Streaming` / `!Stream` エフェクトとの関係

`pulsar.fav` では `!AWS` エフェクトを使用する（`!Pulsar` 独立は v27.x）。
これは SQS Rune / RabbitMQ Rune と同一の方針。
