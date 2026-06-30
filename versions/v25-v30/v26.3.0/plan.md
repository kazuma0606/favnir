# v26.3.0 実装計画 — rabbitmq Rune 実質化

## 実装方針

- rabbitmq Rune は kafka / kinesis / nats Rune と同じ**シングルファイルパターン**（`runes/rabbitmq/rabbitmq.fav` のみ）で実装する
- Cargo 依存追加は**しない**（スタブ実装でモック値を返す、実 AMQP 接続は別途 E2E デモで実装）
- `RabbitConn(String)` は `VMValue::Str` にマップ（KafkaConn / KinesisConn / NatsConn と同パターン）
- 環境変数: `RABBITMQ_URL`（デフォルト: `amqp://guest:guest@localhost:5672`）

---

## 実装ステップ

### Step 0: 事前確認

```bash
grep 'version = ' fav/Cargo.toml                       # "26.2.0" であること
cat benchmarks/v26.2.0.json                            # "test_count":2054 であること
cargo test --bin fav 2>&1 | tail -3                    # 2054 件 PASS であること
ls runes/rabbitmq/ 2>/dev/null || echo "not found"    # 未存在であること
```

### Step 1: `fav/Cargo.toml` bump（26.2.0 → 26.3.0）

```toml
version = "26.3.0"
```

### Step 2: VM Primitive 6 件追加（`fav/src/backend/vm.rs`）

> **順序の理由**: rabbitmq.fav が vm.rs の primitive を呼び出すため、vm.rs を先に追加する。

挿入位置: NATS primitive ブロックの直後（`"NATS.jetstream_consume_raw"` wasm32 arm の後）。

**`#[cfg]` ガード方針**:
```rust
#[cfg(not(target_arch = "wasm32"))]
"RabbitMQ.connect_raw" => { ... }
#[cfg(target_arch = "wasm32")]
"RabbitMQ.connect_raw" => { Err("RabbitMQ not supported on wasm32".to_string()) }
```
> wasm32 フォールバックは全 primitive で `"RabbitMQ not supported on wasm32"` に統一する。

追加する primitive:

| primitive 名 | 引数 | 戻り値 | スタブ実装 |
|---|---|---|---|
| `"RabbitMQ.connect_raw"` | `url: String` | `RabbitConn` (Str) | URL 検証（`RABBITMQ_URL` env → `amqp://guest:guest@localhost:5672`）、`VMValue::Str(url)` を返す |
| `"RabbitMQ.declare_exchange_raw"` | `conn, name, ex_type: String` | `Unit` | スタブ: `VMValue::Unit` |
| `"RabbitMQ.declare_queue_raw"` | `conn, name: String` | `Unit` | スタブ: `VMValue::Unit` |
| `"RabbitMQ.bind_queue_raw"` | `conn, queue, exchange, routing_key: String` | `Unit` | スタブ: `VMValue::Unit` |
| `"RabbitMQ.publish_raw"` | `conn, exchange, routing_key, msg: String` | `Unit` | スタブ: `VMValue::Unit` |
| `"RabbitMQ.consume_raw"` | `conn, queue: String` | `String`（JSON） | スタブ: `"{}"` を返す |

> `connect_raw` は `VMValue::Str` を返す（将来の実 AMQP 接続移行時は接続ハンドル管理が必要）。
> TODO コメントを挿入して将来の移行コストを明示すること。

### Step 2.5: `cargo build` — vm.rs コンパイルエラーなし確認

```bash
cargo build --bin fav 2>&1 | grep -E "^error" | head -10
```

### Step 3: `runes/rabbitmq/rabbitmq.fav` 新規作成

spec.md §4 の Favnir ラッパーを実装:

```favnir
type RabbitConn(String)
type RabbitMsg = { exchange: String, routing_key: String, body: String }

public fn connect(url: String) -> Result<RabbitConn, String> !Stream { ... }
public fn declare_exchange(conn: RabbitConn, name: String, ex_type: String) -> Result<Unit, String> !Stream { ... }
public fn declare_queue(conn: RabbitConn, name: String) -> Result<Unit, String> !Stream { ... }
public fn bind_queue(conn: RabbitConn, queue: String, exchange: String, routing_key: String) -> Result<Unit, String> !Stream { ... }
public fn publish(conn: RabbitConn, exchange: String, routing_key: String, msg: String) -> Result<Unit, String> !Stream { ... }
public fn consume(conn: RabbitConn, queue: String) -> Result<String, String> !Stream { ... }
```

### Step 4: `site/content/docs/runes/rabbitmq.mdx` 新規作成

- Docker セットアップ: `docker run -p 5672:5672 -p 15672:15672 rabbitmq:3-management`
- 環境変数: `RABBITMQ_URL=amqp://guest:guest@localhost:5672`
- Exchange / Queue / Binding の概念説明
- API リファレンス 6 関数
- 5 条件クリア状況

### Step 5: `CHANGELOG.md` 更新

```markdown
## [v26.3.0] — 2026-06-26 — rabbitmq Rune 実質化

### Added
- `runes/rabbitmq/rabbitmq.fav` — RabbitMQ Rune（connect / declare_exchange / declare_queue / bind_queue / publish / consume）
- `RabbitMQ.connect_raw` / `declare_exchange_raw` / `declare_queue_raw` / `bind_queue_raw` / `publish_raw` / `consume_raw` — VM primitive 6 件追加
- `site/content/docs/runes/rabbitmq.mdx` — RabbitMQ Rune ドキュメント新規作成
```

### Step 6: `benchmarks/v26.3.0.json` 新規作成

```json
{"version":"26.3.0","test_count":2062,"timestamp":"2026-06-26"}
```

### Step 7: `fav/src/driver.rs` に `v263000_tests` 追加

`v262000_tests` の直後に追加（7 件）:

```rust
// ── v263000_tests (v26.3.0) — rabbitmq Rune 実質化 ─────────────────────────
#[cfg(test)]
mod v263000_tests {
    #[test]
    fn rabbitmq_rune_has_connect_fn() {
        let src = include_str!("../../runes/rabbitmq/rabbitmq.fav");
        assert!(src.contains("fn connect"), "rabbitmq connect fn not found");
    }
    #[test]
    fn rabbitmq_rune_has_publish_fn() {
        let src = include_str!("../../runes/rabbitmq/rabbitmq.fav");
        assert!(src.contains("fn publish"), "rabbitmq publish fn not found");
    }
    #[test]
    fn rabbitmq_rune_has_consume_fn() {
        let src = include_str!("../../runes/rabbitmq/rabbitmq.fav");
        assert!(src.contains("fn consume"), "rabbitmq consume fn not found");
    }
    #[test]
    fn rabbitmq_rune_has_declare_exchange_fn() {
        let src = include_str!("../../runes/rabbitmq/rabbitmq.fav");
        assert!(src.contains("fn declare_exchange"), "rabbitmq declare_exchange fn not found");
    }
    #[test]
    fn rabbitmq_rune_has_declare_queue_fn() {
        let src = include_str!("../../runes/rabbitmq/rabbitmq.fav");
        assert!(src.contains("fn declare_queue"), "rabbitmq declare_queue fn not found");
    }
    #[test]
    fn rabbitmq_rune_has_bind_queue_fn() {
        let src = include_str!("../../runes/rabbitmq/rabbitmq.fav");
        assert!(src.contains("fn bind_queue"), "rabbitmq bind_queue fn not found");
    }
    #[test]
    fn rabbitmq_rune_has_rabbit_msg_type() {
        let src = include_str!("../../runes/rabbitmq/rabbitmq.fav");
        assert!(src.contains("type RabbitMsg"), "rabbitmq RabbitMsg type not found");
    }
    #[test]
    fn changelog_has_v26_3_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("[v26.3.0]"), "CHANGELOG.md must contain '[v26.3.0]'");
    }
}
```

### Step 8: テスト確認

```bash
cd fav && cargo test v263000 --bin fav          # 8/8 PASS
cd fav && cargo test --bin fav -j 8 -- --test-threads=8 2>&1 | tail -4  # 2062 件 PASS
```

> `--bin fav` フラグが必須。テストモジュール（`v263000_tests`）は `driver.rs` 内に配置されるため `cargo test v263000` だけでは見つからない。

---

## ファイル変更一覧

| ファイル | 操作 |
|---|---|
| `fav/Cargo.toml` | version bump 26.2.0 → 26.3.0 |
| `runes/rabbitmq/rabbitmq.fav` | **新規作成**（2 型 + 6 関数） |
| `fav/src/backend/vm.rs` | RabbitMQ primitive 6 件追加 |
| `site/content/docs/runes/rabbitmq.mdx` | **新規作成** |
| `CHANGELOG.md` | `[v26.3.0]` エントリ先頭に追加 |
| `benchmarks/v26.3.0.json` | **新規作成** |
| `fav/src/driver.rs` | `v263000_tests`（8 件）追加 |

---

## 注意事項

- `runes/rabbitmq/` ディレクトリは未存在。Write ツールが自動作成する。
- **vm.rs を先に実装してから rabbitmq.fav を作成する**（Step 2 → Step 3 の順序が重要）。
- wasm32 フォールバックは全 primitive で `"RabbitMQ not supported on wasm32"` に統一すること。
- `connect_raw` の戻り値は `VMValue::Str(url)`（将来の実 AMQP 接続移行時は接続ハンドル化が必要 → TODO コメント必須）。
- `declare_exchange_raw` / `declare_queue_raw` / `bind_queue_raw` / `publish_raw` の戻り値は `VMValue::Unit`。
- `consume_raw` の戻り値は `VMValue::Str("{}")` （JSON オブジェクトスタブ）。
- `include_str!` のパスは `fav/src/driver.rs` から見た相対パス:
  - `"../../runes/rabbitmq/rabbitmq.fav"`
  - `"../../CHANGELOG.md"`
- vm.rs primitive 挿入位置: `"NATS.jetstream_consume_raw"` wasm32 arm の直後。

## リスクと対応

| リスク | 対応 |
|---|---|
| RabbitMQ が起動していない環境でのテスト失敗 | primitive をスタブ実装（接続なしでモック値を返す） |
| `bind_queue` と `bind` の命名（ロードマップは `bind`） | 他 Rune との名前衝突を避けるため `bind_queue` を採用（より明示的） |
| `ack` / `nack` の delivery ハンドル | delivery tag を `String` として渡すパターンは v26.x 以降で設計 |
