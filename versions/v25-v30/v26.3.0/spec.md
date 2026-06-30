# v26.3.0 仕様書 — rabbitmq Rune 実質化

## 概要

| 項目 | 内容 |
|---|---|
| バージョン | v26.3.0 |
| フェーズ | Streaming Native（v26.1〜v27.0） |
| テーマ | rabbitmq Rune の「動く Rune」5 条件達成 |
| 依存関係 | v26.2.0（nats Rune — API 設計参照）、v25.7.0（kafka Rune — connect パターン参照） |
| 目標テスト数 | 2061 件（+7 件）|

---

## 背景と目的

v26.2.0 で nats Rune が「動く Rune の 5 条件」を満たした。

v26.3.0 では RabbitMQ Rune を実質化する。
RabbitMQ は AMQP プロトコルを使うエンタープライズ MQ の定番であり、
Exchange / Queue / Binding の概念を Favnir の型システムで表現する。

### ロードマップとの API 設計差異

> ロードマップ v26.3 節では以下の 8 関数が記述されている:
> `connect(config)` / `declare_exchange(name, type, opts)` / `declare_queue(name, opts)` / `bind(queue, exchange, routing_key)` / `publish(exchange, routing_key, msg)` / `consume[T](queue, fn)` / `ack(delivery)` / `nack(delivery, requeue)`
>
> v26.3.0 では以下の変更を加える:
> - **`conn` 引数を追加**: kafka / kinesis / nats との API 一貫性を優先し、全関数に `conn: RabbitConn` を第 1 引数として追加
> - **`opts` を省略**: `declare_exchange` / `declare_queue` の `opts` パラメータはスタブ段階では省略（durable/exclusive 等は v26.x 以降）
> - **`fn` コールバック省略**: `consume[T](queue, fn)` のコールバックスタイルは v26.x スコープ外、`consume(conn, queue)` → `Result<String, String>`（JSON 文字列）に変更
> - **`ack` / `nack` をスコープ外**: delivery ハンドル管理が必要なため v26.x 以降に延期
> - **`bind` → `bind_queue` に変更**: ロードマップは `bind(queue, exchange, routing_key)` だが、他 Rune で `bind` が予約語として衝突する可能性を避けるため、より明示的な `bind_queue` を採用（kafka の `create_topic` 等の動詞+目的語パターンと一致）

### 「動く Rune」5 条件

| # | 条件 | 対象 |
|---|---|---|
| 1 | connect | `connect(url)` — RabbitConn（URL ラッパー）を返す |
| 2 | read | `consume(conn, queue)` — メッセージを JSON 文字列で取得 |
| 3 | write | `publish(conn, exchange, routing_key, msg)` — メッセージ発行 |
| 4 | error | `Result<T, String>` 統一、接続失敗時に適切な err を返す |
| 5 | test | `v263000_tests` 8 件 PASS + `runes/rabbitmq/rabbitmq.fav` に 6 関数実装済み |

---

## 機能仕様

### 1. 型定義

`runes/rabbitmq/rabbitmq.fav` に定義する型:

```favnir
// AMQP URL ラッパー型
// "" の場合: RABBITMQ_URL 環境変数 → "amqp://guest:guest@localhost:5672"（デフォルト）
type RabbitConn(String)

// RabbitMQ メッセージ
type RabbitMsg = {
    exchange:    String
    routing_key: String
    body:        String
}
```

### 2. 実装する関数（6 件）

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `connect` | `(url: String) -> Result<RabbitConn, String>` | AMQP URL を検証して RabbitConn を返す |
| `declare_exchange` | `(conn: RabbitConn, name: String, ex_type: String) -> Result<Unit, String>` | Exchange 宣言（`"direct"` / `"fanout"` / `"topic"`） |
| `declare_queue` | `(conn: RabbitConn, name: String) -> Result<Unit, String>` | Queue 宣言 |
| `bind_queue` | `(conn: RabbitConn, queue: String, exchange: String, routing_key: String) -> Result<Unit, String>` | Queue を Exchange にバインド |
| `publish` | `(conn: RabbitConn, exchange: String, routing_key: String, msg: String) -> Result<Unit, String>` | メッセージ発行 |
| `consume` | `(conn: RabbitConn, queue: String) -> Result<String, String>` | メッセージ 1 件取得（JSON 文字列） |

> `consume` はスタブとして空 JSON オブジェクト `"{}"` を返す。
> `ack` / `nack` は delivery ハンドルが必要なため v26.x スコープ外。

### 3. VM Primitive（6 件）

`fav/src/backend/vm.rs` に追加（NATS primitive の直後）:

| primitive 名 | 処理内容 |
|---|---|
| `"RabbitMQ.connect_raw"` | URL 検証（`RABBITMQ_URL` 環境変数 → `amqp://guest:guest@localhost:5672` fallback）して `VMValue::Str(url)` を返す |
| `"RabbitMQ.declare_exchange_raw"` | conn + name + ex_type を受け取り Exchange 宣言（スタブ: `VMValue::Unit`） |
| `"RabbitMQ.declare_queue_raw"` | conn + name を受け取り Queue 宣言（スタブ: `VMValue::Unit`） |
| `"RabbitMQ.bind_queue_raw"` | conn + queue + exchange + routing_key を受け取りバインド（スタブ: `VMValue::Unit`） |
| `"RabbitMQ.publish_raw"` | conn + exchange + routing_key + msg を受け取り発行（スタブ: `VMValue::Unit`） |
| `"RabbitMQ.consume_raw"` | conn + queue を受け取りメッセージ取得（スタブ: `"{}"` を返す） |

各 primitive は `#[cfg(not(target_arch = "wasm32"))]` ガード + `#[cfg(target_arch = "wasm32")]` フォールバックのペアで実装する。
wasm32 フォールバックメッセージは `"RabbitMQ not supported on wasm32"` で統一する。

### 4. `runes/rabbitmq/rabbitmq.fav` — Favnir ラッパー

```favnir
// runes/rabbitmq/rabbitmq.fav — RabbitMQ Rune (v26.3.0)
//
// 使い方:
//   import rune "rabbitmq"
//
// 環境変数:
//   RABBITMQ_URL    — AMQP URL（省略時: "amqp://guest:guest@localhost:5672"）
//
// ローカル開発:
//   docker run -p 5672:5672 -p 15672:15672 rabbitmq:3-management

type RabbitConn(String)
type RabbitMsg = { exchange: String, routing_key: String, body: String }

public fn connect(url: String) -> Result<RabbitConn, String> !Stream {
    RabbitMQ.connect_raw(url)
}

public fn declare_exchange(conn: RabbitConn, name: String, ex_type: String) -> Result<Unit, String> !Stream {
    RabbitMQ.declare_exchange_raw(conn, name, ex_type)
}

public fn declare_queue(conn: RabbitConn, name: String) -> Result<Unit, String> !Stream {
    RabbitMQ.declare_queue_raw(conn, name)
}

public fn bind_queue(conn: RabbitConn, queue: String, exchange: String, routing_key: String) -> Result<Unit, String> !Stream {
    RabbitMQ.bind_queue_raw(conn, queue, exchange, routing_key)
}

public fn publish(conn: RabbitConn, exchange: String, routing_key: String, msg: String) -> Result<Unit, String> !Stream {
    RabbitMQ.publish_raw(conn, exchange, routing_key, msg)
}

public fn consume(conn: RabbitConn, queue: String) -> Result<String, String> !Stream {
    RabbitMQ.consume_raw(conn, queue)
}
```

### 5. `site/content/docs/runes/rabbitmq.mdx` — 新規作成

5 条件クリア状況・API ドキュメント・Docker 実行手順を記載。

---

## エラー処理

すべての primitive は `Result<T, String>` を返す:
- 接続失敗 → `Result.err("RabbitMQ.connect_raw: ...")`
- Exchange 宣言失敗 → `Result.err("RabbitMQ.declare_exchange_raw: ...")`
- Queue 宣言失敗 → `Result.err("RabbitMQ.declare_queue_raw: ...")`
- バインド失敗 → `Result.err("RabbitMQ.bind_queue_raw: ...")`
- 発行失敗 → `Result.err("RabbitMQ.publish_raw: ...")`
- 取得失敗 → `Result.err("RabbitMQ.consume_raw: ...")`

---

## スコープ外（v26.x 以降）

- `ack(delivery)` / `nack(delivery, requeue)` — delivery ハンドル管理が必要（v26.x 以降予定）
- `fn` コールバックスタイルの継続消費ループ
- `declare_exchange` の `opts`（durable / exclusive / auto-delete）パラメータ
- `declare_queue` の `opts`（durable / exclusive / auto-delete）パラメータ
- TLS 接続サポート
- Exchange / Queue バインディングの詳細制御（argument ヘッダー）
- rabbitmq-to-postgres E2E デモ（v27.x 以降）

---

## Rust テスト（v263000_tests、8 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `rabbitmq_rune_has_connect_fn` | `runes/rabbitmq/rabbitmq.fav` に `fn connect` が含まれる | assert |
| `rabbitmq_rune_has_publish_fn` | `runes/rabbitmq/rabbitmq.fav` に `fn publish` が含まれる | assert |
| `rabbitmq_rune_has_consume_fn` | `runes/rabbitmq/rabbitmq.fav` に `fn consume` が含まれる | assert |
| `rabbitmq_rune_has_declare_exchange_fn` | `runes/rabbitmq/rabbitmq.fav` に `fn declare_exchange` が含まれる | assert |
| `rabbitmq_rune_has_declare_queue_fn` | `runes/rabbitmq/rabbitmq.fav` に `fn declare_queue` が含まれる | assert |
| `rabbitmq_rune_has_bind_queue_fn` | `runes/rabbitmq/rabbitmq.fav` に `fn bind_queue` が含まれる | assert |
| `rabbitmq_rune_has_rabbit_msg_type` | `runes/rabbitmq/rabbitmq.fav` に `type RabbitMsg` が含まれる | assert |
| `changelog_has_v26_3_0` | `CHANGELOG.md` に `[v26.3.0]` が含まれる | assert |

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "26.3.0"` であること
- [ ] `runes/rabbitmq/rabbitmq.fav` が存在し、6 関数（connect / declare_exchange / declare_queue / bind_queue / publish / consume）が定義されていること
- [ ] `fav/src/backend/vm.rs` に 6 件の RabbitMQ primitive（`connect_raw` / `declare_exchange_raw` / `declare_queue_raw` / `bind_queue_raw` / `publish_raw` / `consume_raw`）が追加されていること
- [ ] 各 primitive に `#[cfg(not(target_arch = "wasm32"))]` + wasm32 フォールバックがあること
- [ ] `site/content/docs/runes/rabbitmq.mdx` が存在すること
- [ ] `CHANGELOG.md` に `[v26.3.0]` エントリが存在すること
- [ ] `benchmarks/v26.3.0.json` が存在すること（test_count: 2061）
- [ ] `v263000_tests` 8 件すべて PASS
- [ ] 総テスト数 ≥ 2062 件

---

## テスト件数

- v26.2.0 完了時: 2054 件
- v26.3.0 追加: 8 件（v263000_tests）
- **目標**: 2054 + 8 = **2062 件**

> ロードマップの v27.0 完了条件は「4 件以上」を最低基準として記載。本バージョンでは 8 件を実装する。
