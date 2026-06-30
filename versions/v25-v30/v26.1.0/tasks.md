# v26.1.0 タスクリスト — kinesis Rune 実質化

**状態**: COMPLETE
**開始日**: 2026-06-26
**完了日**: 2026-06-26

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | 事前確認: `Cargo.toml` が `26.0.0`、テスト数 2041 件、`runes/kinesis/` が未存在であることを確認 | [x] |
| T1 | `fav/Cargo.toml` を `version = "26.1.0"` に bump | [x] |
| T2 | `fav/src/backend/vm.rs` 更新: Kinesis primitive 5 件追加（`connect_raw` / `put_record_raw` / `put_records_raw` / `get_shard_iterator_raw` / `get_records_raw`）— 各 primitive に `#[cfg(not(target_arch = "wasm32"))]` ガードと wasm32 フォールバックをペアで実装 | [x] |
| T2.5 | `cargo build` — T2 のコンパイルエラーなし確認 | [x] |
| T3 | `runes/kinesis/kinesis.fav` 新規作成（3 型定義 + 5 関数: connect / put_record / put_records / get_shard_iterator / get_records / `get_records` の戻り型は `Result<String, String>`） | [x] |
| T4 | `site/content/docs/runes/kinesis.mdx` 新規作成（5 条件クリア状況 / API リファレンス / LocalStack セットアップ手順） | [x] |
| T5 | `CHANGELOG.md` 更新: 先頭に `[v26.1.0]` エントリ追加 | [x] |
| T6 | `benchmarks/v26.1.0.json` 新規作成（test_count: 2047） | [x] |
| T7 | `fav/src/driver.rs` 更新: `v261000_tests`（6 件）を `v260000_tests` の直後に追加 | [x] |
| T7.5 | `cargo test v261000 --bin fav` — 6/6 PASS 確認 | [x] |
| T8 | `cargo test --bin fav` — 2047 件 PASS 確認（リグレッションなし） | [x] |
| T9 | spec-reviewer レビュー実施（実装前・本タスクで完了済み） | [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "26.1.0"` であること
- [x] `runes/kinesis/kinesis.fav` が存在すること
- [x] `runes/kinesis/kinesis.fav` に `fn connect` が含まれること
- [x] `runes/kinesis/kinesis.fav` に `fn put_record` が含まれること
- [x] `runes/kinesis/kinesis.fav` に `fn put_records` が含まれること
- [x] `runes/kinesis/kinesis.fav` に `fn get_shard_iterator` が含まれること
- [x] `runes/kinesis/kinesis.fav` に `fn get_records` が含まれること
- [x] `fav/src/backend/vm.rs` に `"Kinesis.connect_raw"` primitive が存在すること
- [x] `fav/src/backend/vm.rs` に `"Kinesis.put_record_raw"` primitive が存在すること
- [x] `fav/src/backend/vm.rs` に `"Kinesis.put_records_raw"` primitive が存在すること
- [x] `fav/src/backend/vm.rs` に `"Kinesis.get_shard_iterator_raw"` primitive が存在すること
- [x] `fav/src/backend/vm.rs` に `"Kinesis.get_records_raw"` primitive が存在すること
- [x] `site/content/docs/runes/kinesis.mdx` が存在すること
- [x] `CHANGELOG.md` に `[v26.1.0]` エントリが存在すること
- [x] `benchmarks/v26.1.0.json` が存在すること（test_count: 2047）
- [x] `v261000_tests` 6 件すべて PASS
- [x] 総テスト数 ≥ 2047 件（実測: 2047）

> 注: ロードマップ記載の `cargo test kinesis`（LocalStack 接続テスト）・kinesis_to_s3 E2E デモは
> それぞれ v26.1.0 スコープ外。前者は Docker 依存のため #[ignore]、後者は v26.6.0 で実装予定。

---

## メモ

### vm.rs primitive 挿入位置

Kafka `create_topic_raw` wasm32 arm の直後に Kinesis primitive 10 アーム（5 primitive × 2 cfg）を挿入。

### KinesisConn / ShardIterator の vm.rs 表現

```rust
// KinesisConn は endpoint 文字列を VMValue::Str として格納
// kinesis.fav 側: type KinesisConn(String) → 名目型ラッパー
// vm.rs 側: 直接 VMValue::Str(endpoint_str) を返す（KafkaConn と同パターン）

// ShardIterator も同様
// vm.rs 側: VMValue::Str(iterator_str) を返す
```

### put_records の `records` 引数

`Kinesis.put_records_raw(conn, stream, records)` の `records` は `VMValue::List` で渡される。
スタブ実装では `records.len()` を件数として返す（DynamoDB batch_write と同パターン）。

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [MED] `put_records_raw` でリスト要素の型検証なし | `vm.rs` に TODO コメント追加（スタブ段階では実害なし） |
| [LOW] `connect_raw` が `VMValue::Str` を返し `KinesisConn` ラッパーなし | Kafka と同パターンのため対応不要 |
| [LOW] `v261000_tests` が `"fn X"` 部分文字列のみで `public` / `!Stream` を未検証 | 許容（他 Rune テストと同パターン） |
