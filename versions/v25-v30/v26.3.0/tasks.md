# v26.3.0 タスクリスト — rabbitmq Rune 実質化

**状態**: COMPLETE
**開始日**: 2026-06-26
**完了日**: 2026-06-26

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | 事前確認: `Cargo.toml` が `26.2.0`、テスト数 2054 件、`runes/rabbitmq/` が未存在であることを確認 | [x] |
| T1 | `fav/Cargo.toml` を `version = "26.3.0"` に bump | [x] |
| T2 | `fav/src/backend/vm.rs` 更新: RabbitMQ primitive 6 件追加（`connect_raw` / `declare_exchange_raw` / `declare_queue_raw` / `bind_queue_raw` / `publish_raw` / `consume_raw`）— 各 primitive に `#[cfg(not(target_arch = "wasm32"))]` ガードと wasm32 フォールバックをペアで実装 | [x] |
| T2.5 | `cargo build` — T2 のコンパイルエラーなし確認 | [x] |
| T3 | `runes/rabbitmq/rabbitmq.fav` 新規作成（2 型定義 + 6 関数: connect / declare_exchange / declare_queue / bind_queue / publish / consume） | [x] |
| T4 | `site/content/docs/runes/rabbitmq.mdx` 新規作成（5 条件クリア状況 / API リファレンス / Docker セットアップ手順） | [x] |
| T5 | `CHANGELOG.md` 更新: 先頭に `[v26.3.0]` エントリ追加 | [x] |
| T6 | `benchmarks/v26.3.0.json` 新規作成（test_count: 2062） | [x] |
| T7 | `fav/src/driver.rs` 更新: `v263000_tests`（8 件）を `v262000_tests` の直後に追加 | [x] |
| T7.5 | `cargo test v263000 --bin fav` — 8/8 PASS 確認 | [x] |
| T8 | `cargo test --bin fav` — 2062 件 PASS 確認（リグレッションなし） | [x] |
| T9 | spec-reviewer レビュー実施（実装前・本タスクで完了済み） | [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "26.3.0"` であること
- [x] `runes/rabbitmq/rabbitmq.fav` が存在すること
- [x] `runes/rabbitmq/rabbitmq.fav` に `fn connect` が含まれること
- [x] `runes/rabbitmq/rabbitmq.fav` に `fn publish` が含まれること
- [x] `runes/rabbitmq/rabbitmq.fav` に `fn consume` が含まれること
- [x] `runes/rabbitmq/rabbitmq.fav` に `fn declare_exchange` が含まれること
- [x] `runes/rabbitmq/rabbitmq.fav` に `fn declare_queue` が含まれること
- [x] `runes/rabbitmq/rabbitmq.fav` に `fn bind_queue` が含まれること
- [x] `runes/rabbitmq/rabbitmq.fav` に `type RabbitMsg` が含まれること
- [x] `fav/src/backend/vm.rs` に `"RabbitMQ.connect_raw"` primitive が存在すること
- [x] `fav/src/backend/vm.rs` に `"RabbitMQ.declare_exchange_raw"` primitive が存在すること
- [x] `fav/src/backend/vm.rs` に `"RabbitMQ.declare_queue_raw"` primitive が存在すること
- [x] `fav/src/backend/vm.rs` に `"RabbitMQ.bind_queue_raw"` primitive が存在すること
- [x] `fav/src/backend/vm.rs` に `"RabbitMQ.publish_raw"` primitive が存在すること
- [x] `fav/src/backend/vm.rs` に `"RabbitMQ.consume_raw"` primitive が存在すること
- [x] `site/content/docs/runes/rabbitmq.mdx` が存在すること
- [x] `CHANGELOG.md` に `[v26.3.0]` エントリが存在すること
- [x] `benchmarks/v26.3.0.json` が存在すること（test_count: 2062）
- [x] `v263000_tests` 8 件すべて PASS
- [x] 総テスト数 ≥ 2062 件（実測: 2062）

> 注: ロードマップ記載の `cargo test rabbitmq`（Docker 接続テスト）は環境依存のため `#[ignore]`。
> `ack` / `nack` / fn コールバック継続消費ループは v26.3.0 スコープ外。
> `bind` はロードマップ記載名だが `bind_queue` に変更（他 Rune との名前衝突回避）。

---

## メモ

### vm.rs primitive 挿入位置

`"NATS.jetstream_consume_raw"` wasm32 arm の直後に RabbitMQ primitive 12 アーム（6 primitive × 2 cfg）を挿入。

### wasm32 フォールバック統一

全 6 primitive で `"RabbitMQ not supported on wasm32"` に統一。

### RabbitConn の vm.rs 表現

```rust
// RabbitConn は URL 文字列を VMValue::Str として格納
// TODO: 実 AMQP 接続移行時は接続ハンドルのレジストリ管理が必要
```

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| — | — |
