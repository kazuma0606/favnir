# v26.8.0 仕様書 — SQS Rune 実質化

## 概要

| 項目 | 内容 |
|---|---|
| バージョン | v26.8.0 |
| フェーズ | Streaming Native（v26.1〜v27.0） |
| テーマ | AWS SQS Rune 実質化（`runes/sqs/sqs.fav` スタブ → 完全実装） |
| 依存関係 | v25.2.0（LocalStack AWS 認証基盤）完了後 |
| 目標テスト数 | 2102 件（+8 件）|

---

## 背景と目的

`runes/sqs/sqs.fav` は v24.5.0 で追加したが、コメントのみの空スタブだった。
v26.8.0 では SQS Rune を「実質化」し、`import rune "sqs"` → `SQS.*` で使える
6 関数を実装する。`cargo test sqs` で 6 件以上 PASS させる。

### 名前空間衝突の注意

`runes/aws/sqs.fav`（legacy）と `runes/sqs/sqs.fav` は両方とも `SQS.*` 名前空間を公開する。
同一ファイルで `import rune "aws"` と `import rune "sqs"` を併用した場合の名前解決は未定義（v27.x スコープ外）。
v26.8.0 では両方を同時に import するユースケースは対象外とする。

### 既存リソースの整理

| ファイル | 状態 | 役割 |
|---|---|---|
| `runes/sqs/sqs.fav` | 空スタブ（v24.5.0）| **今バージョンで実装** |
| `runes/aws/sqs.fav` | 実装済み（v4.11.0、legacy）| `import rune "aws"` 経由の既存 API。変更しない |
| `vm.rs` AWS SQS primitives | 実装済み（`AWS.sqs_*_raw` 4 件）| `send_message` / `receive_messages` / `delete_message` / `get_queue_url` |
| `vm.rs` SQS 新 primitives | **未実装**（`SQS.*_raw` 3 件）| `send_message_batch` / `purge` / `consume` |

### ロードマップとの API 設計差異

ロードマップ v26.8 節の理想 API:

| 関数 | ロードマップ | 実装（v26.8.0） |
|---|---|---|
| `send_message` | `SQS.send_message(queue_url, body)`（遅延配信オプション付き） | 基本シグネチャは同一。遅延配信（`DelaySeconds`）は既存 `AWS.sqs_send_message_raw` の制限により省略（v27.x でシグネチャ拡張予定） |
| `send_message_batch` | `SQS.send_message_batch(queue_url, messages)` | 新 `SQS.send_message_batch_raw` primitive（vm.rs に追加） |
| `receive_messages[T]` | 型付き受信 | スタブ。`SQS.receive_messages_raw` → JSON 文字列を返す（型付きデシリアライズは v27.x） |
| `delete_message` | `SQS.delete_message(queue_url, receipt)` | 同一。既存 `AWS.sqs_delete_message_raw` に委譲 |
| `purge` | `SQS.purge(queue_url)` | 新 `SQS.purge_raw` primitive（vm.rs に追加） |
| `consume[T]` | 継続消費ループ（自動削除オプション） | スタブ。`SQS.consume_raw` → 1 回ポーリングの JSON 文字列（ループ化は v27.x） |

---

## 機能仕様

### 1. `runes/sqs/sqs.fav`（既存スタブを実装で上書き）

```favnir
// runes/sqs/sqs.fav — AWS SQS Rune (v26.8.0)
//
// 使い方:
//   import rune "sqs"
//
// 環境変数:
//   AWS_ENDPOINT_URL      — LocalStack エンドポイント（例: "http://localhost:4566"）
//   AWS_ACCESS_KEY_ID     — 認証キー（LocalStack では "test"）
//   AWS_SECRET_ACCESS_KEY — シークレット（LocalStack では "test"）
//   AWS_DEFAULT_REGION    — リージョン（省略: "us-east-1"）

// メッセージ送信。戻り値は MessageId 文字列。
public fn send_message(queue_url: String, body: String) -> Result<String, String> !AWS {
    AWS.sqs_send_message_raw(queue_url, body)
}

// バッチ送信（最大 10 件）。messages は JSON 文字列のリスト。
// Note: 既存 AWS.sqs_send_message_raw に対応する batch 版 primitive を vm.rs に追加。
public fn send_message_batch(queue_url: String, messages: List<String>) -> Result<String, String> !AWS {
    SQS.send_message_batch_raw(queue_url, messages)
}

// メッセージ受信（最大 max 件）。戻り値は受信メッセージの JSON 文字列。
// Note: 型付きデシリアライズ（`receive_messages[T]`）は v27.x スコープ外。
public fn receive_messages(queue_url: String, max: Int) -> Result<String, String> !AWS {
    SQS.receive_messages_raw(queue_url, max)
}

// 受信確認（メッセージ削除）。receipt_handle は receive_messages で取得した値。
public fn delete_message(queue_url: String, receipt_handle: String) -> Result<Unit, String> !AWS {
    AWS.sqs_delete_message_raw(queue_url, receipt_handle)
}

// キュー内の全メッセージを削除（テスト用）。
public fn purge(queue_url: String) -> Result<Unit, String> !AWS {
    SQS.purge_raw(queue_url)
}

// 継続消費スタブ（1 回ポーリング）。戻り値はメッセージ JSON 文字列。
// Note: 継続ループ（自動削除オプション）は v27.x スコープ外。
public fn consume(queue_url: String) -> Result<String, String> !AWS {
    SQS.consume_raw(queue_url)
}
```

> **注意**: `send_message` / `delete_message` は既存 `AWS.sqs_*_raw` primitive を再利用する（vm.rs への追加不要）。
> `send_message_batch` / `receive_messages` / `purge` / `consume` は新 `SQS.*_raw` primitive（vm.rs に 4 件追加）。

### 2. `fav/src/backend/vm.rs` — SQS primitive 4 件追加

追加する primitive（既存 `AWS.sqs_*_raw` ブロックの直後に配置）:

| primitive | 内容 | 戻り値 |
|---|---|---|
| `SQS.send_message_batch_raw` | `SendMessageBatch` AWS アクション（最大 10 件）| `Result<String, String>`（バッチ結果 JSON） |
| `SQS.receive_messages_raw` | `ReceiveMessage` AWS アクション（JSON 文字列で返す） | `Result<String, String>`（メッセージ JSON） |
| `SQS.purge_raw` | `PurgeQueue` AWS アクション | `Result<Unit, String>` |
| `SQS.consume_raw` | `ReceiveMessage`（1 回ポーリング）の JSON 返却版 | `Result<String, String>` |

### 3. `site/content/docs/runes/sqs.mdx` 新規作成

- SQS Rune 概要・LocalStack セットアップ
- 環境変数一覧
- 全 6 関数のシグネチャと使用例
- スコープ外（継続ループ・型付きデシリアライズ）

---

## スコープ外（v27.x 以降）

- `SQS.consume[T](queue_url, fn)` — コールバック型継続消費ループ
- `SQS.receive_messages[T]` — 型付きデシリアライズ（`SQSMessage<T>` 構造体）
- `SQS.receive_messages_raw` の戻り値型を `Result<List<Record>, String>` に移行する互換性変更（v27.x で判断）
- `send_message` の遅延配信オプション（`DelaySeconds` パラメータ）
- Lambda トリガー連携
- Dead Letter Queue（DLQ）操作
- FIFO キュー（`.fifo` サフィックス / MessageGroupId / MessageDeduplicationId）
- `import rune "aws"` + `import rune "sqs"` 同時 import 時の `SQS.*` 名前空間衝突解決

---

## Rust テスト（v268000_tests、8 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `sqs_rune_has_send_message_fn` | `runes/sqs/sqs.fav` に `fn send_message` が含まれる | assert |
| `sqs_rune_has_send_message_batch_fn` | `runes/sqs/sqs.fav` に `fn send_message_batch` が含まれる | assert |
| `sqs_rune_has_receive_messages_fn` | `runes/sqs/sqs.fav` に `fn receive_messages` が含まれる | assert |
| `sqs_rune_has_delete_message_fn` | `runes/sqs/sqs.fav` に `fn delete_message` が含まれる | assert |
| `sqs_rune_has_purge_fn` | `runes/sqs/sqs.fav` に `fn purge` が含まれる | assert |
| `sqs_rune_has_consume_fn` | `runes/sqs/sqs.fav` に `fn consume` が含まれる | assert |
| `sqs_rune_vm_has_send_message_batch_raw` | `fav/src/backend/vm.rs` に `SQS.send_message_batch_raw` が含まれる | assert |
| `changelog_has_v26_8_0` | `CHANGELOG.md` に `[v26.8.0]` が含まれる | assert |

> `cargo test sqs` → テスト 1〜7（`sqs_rune_has_*` 6 件 + `sqs_rune_vm_has_*` 1 件）が検出される（7 件 PASS）。ロードマップ要件「4 件以上」を超える。
> `changelog_has_v26_8_0` はテスト名に "sqs" を含まないため `cargo test sqs` の対象外（`cargo test changelog` で検出）。

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "26.8.0"` であること
- [ ] `runes/sqs/sqs.fav` に `public fn send_message` が含まれること
- [ ] `runes/sqs/sqs.fav` に `public fn send_message_batch` が含まれること
- [ ] `runes/sqs/sqs.fav` に `public fn receive_messages` が含まれること
- [ ] `runes/sqs/sqs.fav` に `public fn delete_message` が含まれること
- [ ] `runes/sqs/sqs.fav` に `public fn purge` が含まれること
- [ ] `runes/sqs/sqs.fav` に `public fn consume` が含まれること
- [ ] `fav/src/backend/vm.rs` に `SQS.send_message_batch_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `SQS.receive_messages_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `SQS.purge_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `SQS.consume_raw` が含まれること
- [ ] `site/content/docs/runes/sqs.mdx` が存在すること
- [ ] `CHANGELOG.md` に `[v26.8.0]` エントリが存在すること
- [ ] `benchmarks/v26.8.0.json` が存在すること（test_count: 2102）
- [ ] `v268000_tests` 8 件すべて PASS
- [ ] `cargo test sqs --bin fav` で 7 件以上 PASS
- [ ] 総テスト数 ≥ 2102 件

---

## ロードマップ v27.0「5 条件クリア」との対応

ロードマップ v27.0 の完了条件テーブル「sqs Rune — 5 条件クリア + 4 件テスト + LocalStack 動作」の
「5 条件クリア」は完了条件チェックリストの以下 5 項目に対応する:
1. `public fn send_message` 存在
2. `public fn send_message_batch` 存在
3. `public fn receive_messages` 存在
4. `public fn delete_message` 存在
5. `public fn purge` 存在

（`consume` は 6 件目として追加。「4 件テスト」は v26.8.0 の 8 件テストで上回る。）

---

## テスト件数

- v26.7.0 完了時: 2094 件
- v26.8.0 追加: 8 件（v268000_tests）
- **目標**: 2094 + 8 = **2102 件**

> `benchmarks/v26.7.0.json` で `test_count: 2094` を **Step 0 で確認すること**。
