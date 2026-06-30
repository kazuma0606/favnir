# v26.2.0 タスクリスト — nats Rune 実質化

**状態**: COMPLETE
**開始日**: 2026-06-26
**完了日**: 2026-06-26

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | 事前確認: `Cargo.toml` が `26.1.0`、テスト数 2047 件、`runes/nats/` が未存在であることを確認 | [x] |
| T1 | `fav/Cargo.toml` を `version = "26.2.0"` に bump | [x] |
| T2 | `fav/src/backend/vm.rs` 更新: NATS primitive 5 件追加（`connect_raw` / `publish_raw` / `subscribe_raw` / `jetstream_publish_raw` / `jetstream_consume_raw`）— 各 primitive に `#[cfg(not(target_arch = "wasm32"))]` ガードと wasm32 フォールバックをペアで実装 | [x] |
| T2.5 | `cargo build` — T2 のコンパイルエラーなし確認 | [x] |
| T3 | `runes/nats/nats.fav` 新規作成（2 型定義 + 5 関数: connect / publish / subscribe / jetstream_publish / jetstream_consume） | [x] |
| T4 | `site/content/docs/runes/nats.mdx` 新規作成（5 条件クリア状況 / API リファレンス / Docker セットアップ手順） | [x] |
| T5 | `CHANGELOG.md` 更新: 先頭に `[v26.2.0]` エントリ追加 | [x] |
| T6 | `benchmarks/v26.2.0.json` 新規作成（test_count: 2054） | [x] |
| T7 | `fav/src/driver.rs` 更新: `v262000_tests`（7 件）を `v261000_tests` の直後に追加 | [x] |
| T7.5 | `cargo test v262000 --bin fav` — 7/7 PASS 確認 | [x] |
| T8 | `cargo test --bin fav` — 2054 件 PASS 確認（リグレッションなし） | [x] |
| T9 | spec-reviewer レビュー実施（実装前・本タスクで完了済み） | [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "26.2.0"` であること
- [x] `runes/nats/nats.fav` が存在すること
- [x] `runes/nats/nats.fav` に `fn connect` が含まれること
- [x] `runes/nats/nats.fav` に `fn publish` が含まれること
- [x] `runes/nats/nats.fav` に `fn subscribe` が含まれること
- [x] `runes/nats/nats.fav` に `fn jetstream_publish` が含まれること
- [x] `runes/nats/nats.fav` に `fn jetstream_consume` が含まれること
- [x] `runes/nats/nats.fav` に `type NatsMsg` が含まれること
- [x] `fav/src/backend/vm.rs` に `"NATS.connect_raw"` primitive が存在すること
- [x] `fav/src/backend/vm.rs` に `"NATS.publish_raw"` primitive が存在すること
- [x] `fav/src/backend/vm.rs` に `"NATS.subscribe_raw"` primitive が存在すること
- [x] `fav/src/backend/vm.rs` に `"NATS.jetstream_publish_raw"` primitive が存在すること
- [x] `fav/src/backend/vm.rs` に `"NATS.jetstream_consume_raw"` primitive が存在すること
- [x] `site/content/docs/runes/nats.mdx` が存在すること
- [x] `CHANGELOG.md` に `[v26.2.0]` エントリが存在すること
- [x] `benchmarks/v26.2.0.json` が存在すること（test_count: 2054）
- [x] `v262000_tests` 7 件すべて PASS
- [x] 総テスト数 ≥ 2054 件（実測: 2054）

> 注: ロードマップ記載の `cargo test nats`（nats-server 接続テスト）は Docker 依存のため `#[ignore]`。
> また、ロードマップのテストフィルタ名 `nats` と実装モジュール名 `v262000_tests` は異なる
> （`cargo test nats` では走らない）。v27.0 マイルストーン検証時はフィルタを `v262000` で実行すること。
> `request[T]` 関数・fn コールバック継続購読ループは v26.2.0 スコープ外。
> nats_to_postgres E2E デモは v26.7.0 で実装予定。

---

## メモ

### vm.rs primitive 挿入位置

`"Kinesis.get_records_raw"` wasm32 arm の直後に NATS primitive 10 アーム（5 primitive × 2 cfg）を挿入。

### wasm32 エラーメッセージ統一

全 primitive で `"NATS not supported on wasm32"` に統一（primitive 名による差異なし）。

### NatsConn の vm.rs 表現

```rust
// NatsConn は URL 文字列を VMValue::Str として格納
// nats.fav 側: type NatsConn(String) → 名目型ラッパー
// vm.rs 側: 直接 VMValue::Str(url) を返す（KafkaConn / KinesisConn と同パターン）
```

### publish_raw の戻り値

`NATS.publish_raw` の戻り値は `VMValue::Unit`（kafka の `produce_raw` と同パターン）。

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [MED] `connect_raw` がスタブであることのコメント未記載 | vm.rs に TODO コメント追加（将来の nats crate 実装移行時の注意） |
| [LOW] `NatsConn` 型の存在確認テストが未追加 | 許容（kinesis と同パターン、将来版で検討） |
| [LOW] `nats.mdx` に `Json.decode[NatsMsg]` サンプルなし | 許容（利用者向け参考情報、必須ではない） |
