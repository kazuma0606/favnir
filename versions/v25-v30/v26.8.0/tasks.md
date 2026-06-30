# v26.8.0 タスクリスト — SQS Rune 実質化

**状態**: COMPLETE
**開始日**: 2026-06-27
**完了日**: 2026-06-27

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | 事前確認: `Cargo.toml` が `26.7.0`、テスト数 2094 件、`sqs.fav` が空スタブ、vm.rs に `SQS.send_message_batch_raw` がないことを確認 | [x] |
| T1 | `fav/Cargo.toml` を `version = "26.8.0"` に bump | [x] |
| T2 | `runes/sqs/sqs.fav` を上書き実装（空スタブ → 6 関数） | [x] |
| T3 | `fav/src/backend/vm.rs` に `SQS.*_raw` primitive 4 件追加（`AWS.sqs_get_queue_url_raw` ブロックの直後） | [x] |
| T4 | `site/content/docs/runes/sqs.mdx` 新規作成 | [x] |
| T5 | `CHANGELOG.md` 更新: 先頭に `[v26.8.0]` エントリ追加 | [x] |
| T6 | `benchmarks/v26.8.0.json` 新規作成（test_count: 2102） | [x] |
| T7 | `fav/src/driver.rs` 更新: `v268000_tests`（8 件）を `v267000_tests` の直後に追加 | [x] |
| T7.5 | `cargo test v268000 --bin fav` — 8/8 PASS 確認 | [x] |
| T7.6 | `cargo test sqs --bin fav` — 11 件 PASS 確認（ロードマップ要件「4 件以上」を大幅超過） | [x] |
| T8 | `cargo test --bin fav` — 2102 件 PASS 確認（リグレッションなし） | [x] |
| T9 | spec-reviewer レビュー実施（実装前）| [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "26.8.0"` であること
- [x] `runes/sqs/sqs.fav` に `public fn send_message` が含まれること
- [x] `runes/sqs/sqs.fav` に `public fn send_message_batch` が含まれること
- [x] `runes/sqs/sqs.fav` に `public fn receive_messages` が含まれること
- [x] `runes/sqs/sqs.fav` に `public fn delete_message` が含まれること
- [x] `runes/sqs/sqs.fav` に `public fn purge` が含まれること
- [x] `runes/sqs/sqs.fav` に `public fn consume` が含まれること
- [x] `fav/src/backend/vm.rs` に `SQS.send_message_batch_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `SQS.receive_messages_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `SQS.purge_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `SQS.consume_raw` が含まれること
- [x] `site/content/docs/runes/sqs.mdx` が存在すること
- [x] `CHANGELOG.md` に `[v26.8.0]` エントリが存在すること
- [x] `benchmarks/v26.8.0.json` が存在すること（test_count: 2102）
- [x] `v268000_tests` 8 件すべて PASS
- [x] `cargo test sqs --bin fav` で 7 件以上 PASS（実績: 11 件）
- [x] 総テスト数 ≥ 2102 件

---

## メモ

### vm.rs の挿入位置

`"AWS.sqs_get_queue_url_raw"` ブロックの末尾（`}` の後）、`"AWS.dynamo_get_item_raw"` の直前に挿入した。

### primitive の使い分け

| 関数 | 委譲先 | 理由 |
|---|---|---|
| `send_message` | `AWS.sqs_send_message_raw`（既存） | 同一シグネチャ |
| `send_message_batch` | `SQS.send_message_batch_raw`（新規） | 既存 primitive なし |
| `receive_messages` | `SQS.receive_messages_raw`（新規） | 既存は `List<Record>` 返却だが SQS Rune は JSON String で統一 |
| `delete_message` | `AWS.sqs_delete_message_raw`（既存） | 同一シグネチャ |
| `purge` | `SQS.purge_raw`（新規） | 既存 primitive なし |
| `consume` | `SQS.consume_raw`（新規） | 既存 primitive なし |

### `include_str!` パス（`fav/src/driver.rs` 基準）

```rust
include_str!("../../runes/sqs/sqs.fav")  // fav/src/ → ../../ → favnir/runes/sqs/sqs.fav
include_str!("backend/vm.rs")            // fav/src/ → fav/src/backend/vm.rs
include_str!("../../CHANGELOG.md")       // fav/src/ → ../../ → favnir/CHANGELOG.md
```

### `cargo test sqs` の検出件数

`cargo test sqs --bin fav` で検出されたテスト（11 件）:
- `v268000_tests::sqs_rune_has_*` 6 件
- `v268000_tests::sqs_rune_vm_has_send_message_batch_raw` 1 件
- 既存テスト（`emit_python` / `vm_stdlib_tests` / `duckdb_rune_tests`）4 件

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [HIGH] JSON インジェクション: `receive_messages_raw` / `consume_raw` の `body`/`receipt_handle` が無エスケープで JSON 埋め込み | `serde_json::to_string(&s)` でエスケープするよう修正（vm.rs） |
| [MED] `receive_messages_raw` の `max` が SQS 上限（1〜10）を超えても無検証 | `max < 1 \|\| max > 10` チェックを追加し `err_vm` 返却（vm.rs） |
| [MED] `send_message_batch_raw` の 10 件上限チェック欠落 | `messages.len() > 10` ガードを追加（vm.rs） |
| [MED] `max` 型不一致でサイレントフォールバック（`_ => 1`） | `_ => return Err(...)` に修正（vm.rs） |
| [LOW] `fn consume` テストが false positive リスク | `fn consume(` に修正（driver.rs）— `purge`・`delete_message` も同様修正 |
| [MED] `send_message_batch` ドキュメントに `bind` 欠落 | `bind result <-` を追加（sqs.mdx） |
| [LOW] `receive_messages` の `max` 上限（1〜10）が docs に未記載 | `max` の説明に SQS 上限注記を追加（sqs.mdx） |
