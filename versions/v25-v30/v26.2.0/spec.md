# v26.2.0 仕様書 — nats Rune 実質化

## 概要

| 項目 | 内容 |
|---|---|
| バージョン | v26.2.0 |
| フェーズ | Streaming Native（v26.1〜v27.0） |
| テーマ | nats Rune の「動く Rune」5 条件達成 |
| 依存関係 | v26.1.0（kinesis Rune — API 設計参照）、v25.7.0（kafka Rune — connect パターン参照） |
| 目標テスト数 | 2053 件（+6 件）|

---

## 背景と目的

v26.1.0 で kinesis Rune が「動く Rune の 5 条件」を満たした。

v26.2.0 では NATS Rune を実質化する。
NATS は軽量・高速メッセージングの定番であり、IoT / マイクロサービス・エッジコンピューティングで広く使われる。
JetStream（永続メッセージ機能）まで含めて実装し、v26.4 の `Stream.*` 操作との統合基盤を整える。

### kafka / kinesis Rune との API 設計対応

| kafka Rune（v25.7.0）/ kinesis Rune（v26.1.0） | nats Rune（v26.2.0） | 備考 |
|---|---|---|
| `KafkaConn(String)` / `KinesisConn(String)` — 名目型ラッパー | `NatsConn(String)` — 名目型ラッパー | 同パターン |
| `!Stream` エフェクト | `!Stream` エフェクト | 共通 |
| `fn connect(brokers/endpoint: String)` | `fn connect(url: String)` | 形式統一 |
| `fn produce(conn, topic, key, value)` | `fn publish(conn, subject, payload)` | NATS API 名に準拠 |
| `fn consume_batch(conn, topic, ...)` → JSON 配列 | `fn subscribe(conn, subject)` → JSON 文字列 | 単発取得（スタブ） |

### 「動く Rune」5 条件

| # | 条件 | 対象 |
|---|---|---|
| 1 | connect | `connect(url)` — NatsConn（URL ラッパー）を返す |
| 2 | read | `subscribe(conn, subject)` — メッセージを 1 件取得（JSON 文字列） |
| 3 | write | `publish(conn, subject, payload)` — メッセージ発行 |
| 4 | error | `Result<T, String>` 統一、接続失敗時に適切な err を返す |
| 5 | test | `v262000_tests` 6 件 PASS + `runes/nats/nats.fav` に 5 関数実装済み |

---

## 機能仕様

### 1. 型定義

`runes/nats/nats.fav` に定義する型:

```favnir
// NATS サーバー URL ラッパー型
// "" の場合: NATS_URL 環境変数 → "nats://localhost:4222"（デフォルト）
type NatsConn(String)

// NATS メッセージ
type NatsMsg = {
    subject: String
    payload: String
    reply:   String
}
```

### 2. 実装する関数（5 件）

> **ロードマップのシグネチャ差異について**: ロードマップ `v26.2` 節では `NATS.publish(subject, payload)` と `conn` なしで記述されているが、kafka / kinesis Rune との API 一貫性を優先して `conn: NatsConn` を第 1 引数に追加した。同様に `subscribe[T](subject, fn)` / `jetstream_consume[T](stream, consumer, fn)` の `fn` コールバック引数と型パラメータは v26.2.0 では省略し、スタブとして JSON 文字列を返す単純な設計とした（kafka の `consume_batch_raw` / kinesis の `get_records_raw` と同パターン）。

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `connect` | `(url: String) -> Result<NatsConn, String>` | URL を検証して NatsConn を返す |
| `publish` | `(conn: NatsConn, subject: String, payload: String) -> Result<Unit, String>` | メッセージ発行（コア NATS） |
| `subscribe` | `(conn: NatsConn, subject: String) -> Result<String, String>` | 単発メッセージ取得（JSON 文字列として返す） |
| `jetstream_publish` | `(conn: NatsConn, stream: String, payload: String) -> Result<String, String>` | JetStream 永続メッセージ送信（シーケンス番号を返す） |
| `jetstream_consume` | `(conn: NatsConn, stream: String, consumer: String) -> Result<String, String>` | JetStream メッセージ取得（JSON 配列文字列を返す） |

> `subscribe` はスタブとして空の JSON オブジェクト `"{}"` を返す（コネクション未接続時）。
> `request` 関数（リクエスト/レスポンスパターン）は v26.2.0 スコープ外（v26.9 以降で実装予定）。
> `fn` コールバックスタイルの継続購読ループも v26.2.0 スコープ外。

### 3. VM Primitive（5 件）

`fav/src/backend/vm.rs` に追加（Kinesis primitive の直後）:

| primitive 名 | 処理内容 |
|---|---|
| `"NATS.connect_raw"` | URL 文字列を検証して NatsConn を返す（`NATS_URL` 環境変数 → `nats://localhost:4222` fallback） |
| `"NATS.publish_raw"` | conn + subject + payload を受け取りメッセージ送信（スタブ: `VMValue::Unit` を返す） |
| `"NATS.subscribe_raw"` | conn + subject を受け取り単発メッセージ取得（スタブ: `"{}"` を返す） |
| `"NATS.jetstream_publish_raw"` | conn + stream + payload を受け取り JetStream 送信（スタブ: `"seq-js-0001"` を返す） |
| `"NATS.jetstream_consume_raw"` | conn + stream + consumer を受け取りメッセージ取得（スタブ: `"[]"` を返す） |

各 primitive は `#[cfg(not(target_arch = "wasm32"))]` ガード + `#[cfg(target_arch = "wasm32")]` フォールバックのペアで実装する（Kinesis と同パターン）。

### 4. `runes/nats/nats.fav` — Favnir ラッパー

kafka.fav / kinesis.fav と同じシングルファイルパターン:

```favnir
// runes/nats/nats.fav — NATS Rune (v26.2.0)
//
// 使い方:
//   import rune "nats"
//
// 環境変数:
//   NATS_URL    — NATS サーバー URL（省略時: "nats://localhost:4222"）
//
// ローカル開発（JetStream 有効）:
//   docker run -p 4222:4222 nats:latest -js

type NatsConn(String)
type NatsMsg = { subject: String, payload: String, reply: String }

public fn connect(url: String) -> Result<NatsConn, String> !Stream {
    NATS.connect_raw(url)
}

public fn publish(conn: NatsConn, subject: String, payload: String) -> Result<Unit, String> !Stream {
    NATS.publish_raw(conn, subject, payload)
}

public fn subscribe(conn: NatsConn, subject: String) -> Result<String, String> !Stream {
    NATS.subscribe_raw(conn, subject)
}

public fn jetstream_publish(conn: NatsConn, stream: String, payload: String) -> Result<String, String> !Stream {
    NATS.jetstream_publish_raw(conn, stream, payload)
}

public fn jetstream_consume(conn: NatsConn, stream: String, consumer: String) -> Result<String, String> !Stream {
    NATS.jetstream_consume_raw(conn, stream, consumer)
}
```

### 5. `site/content/docs/runes/nats.mdx` — 新規作成

5 条件クリア状況・API ドキュメント・`nats-server` Docker 実行手順を記載。

---

## エラー処理

すべての primitive は `Result<T, String>` を返す:
- 接続失敗 → `Result.err("NATS.connect_raw: ...")`
- 発行失敗 → `Result.err("NATS.publish_raw: ...")`
- 購読失敗 → `Result.err("NATS.subscribe_raw: ...")`
- JetStream 送信失敗 → `Result.err("NATS.jetstream_publish_raw: ...")`
- JetStream 取得失敗 → `Result.err("NATS.jetstream_consume_raw: ...")`

---

## スコープ外（v26.x 以降）

- `request[T](subject, payload, timeout)` リクエスト/レスポンスパターン — v26.9 以降予定
- `fn` コールバックスタイルの継続購読ループ
- TLS / mTLS 接続サポート
- NATS クラスタ接続（複数サーバー）
- JetStream Key-Value Store / Object Store
- NATS 2.0 分散セキュリティ（NKey / JWT 認証）
- nats_to_postgres E2E デモ — v26.7 で実装予定

---

## Rust テスト（v262000_tests、7 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `nats_rune_has_connect_fn` | `runes/nats/nats.fav` に `fn connect` が含まれる | assert |
| `nats_rune_has_publish_fn` | `runes/nats/nats.fav` に `fn publish` が含まれる | assert |
| `nats_rune_has_subscribe_fn` | `runes/nats/nats.fav` に `fn subscribe` が含まれる | assert |
| `nats_rune_has_jetstream_publish_fn` | `runes/nats/nats.fav` に `fn jetstream_publish` が含まれる | assert |
| `nats_rune_has_jetstream_consume_fn` | `runes/nats/nats.fav` に `fn jetstream_consume` が含まれる | assert |
| `nats_rune_has_nats_msg_type` | `runes/nats/nats.fav` に `type NatsMsg` が含まれる | assert |
| `changelog_has_v26_2_0` | `CHANGELOG.md` に `[v26.2.0]` が含まれる | assert |

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "26.2.0"` であること
- [ ] `runes/nats/nats.fav` が存在し、5 関数（connect / publish / subscribe / jetstream_publish / jetstream_consume）が定義されていること
- [ ] `fav/src/backend/vm.rs` に 5 件の NATS primitive（`connect_raw` / `publish_raw` / `subscribe_raw` / `jetstream_publish_raw` / `jetstream_consume_raw`）が追加されていること
- [ ] 各 primitive に `#[cfg(not(target_arch = "wasm32"))]` + wasm32 フォールバックがあること
- [ ] `site/content/docs/runes/nats.mdx` が存在すること
- [ ] `CHANGELOG.md` に `[v26.2.0]` エントリが存在すること
- [ ] `benchmarks/v26.2.0.json` が存在すること（test_count: 2053）
- [ ] `v262000_tests` 7 件すべて PASS
- [ ] 総テスト数 ≥ 2054 件

---

## テスト件数

- v26.1.0 完了時: 2047 件
- v26.2.0 追加: 7 件（v262000_tests）
- **目標**: 2047 + 7 = **2054 件**
