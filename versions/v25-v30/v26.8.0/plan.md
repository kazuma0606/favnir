# v26.8.0 実装計画 — SQS Rune 実質化

## 実装方針

- `runes/sqs/sqs.fav` を空スタブから 6 関数の完全実装に上書きする
- `fav/src/backend/vm.rs` に `SQS.*_raw` primitive 4 件を追加する
- `runes/aws/sqs.fav`（legacy）は変更しない
- `site/content/docs/runes/sqs.mdx` を新規作成する

---

## 実装ステップ

### Step 0: 事前確認

```bash
grep 'version = ' fav/Cargo.toml                    # "26.7.0" であること
cat benchmarks/v26.7.0.json                          # "test_count":2094 であること
cargo test --bin fav 2>&1 | tail -3                  # 2094 件 PASS であること
cat runes/sqs/sqs.fav                               # 空スタブであること（関数定義なし）
grep 'SQS.send_message_batch_raw' fav/src/backend/vm.rs || echo "not found"  # 未存在であること
```

### Step 1: `fav/Cargo.toml` bump（26.7.0 → 26.8.0）

```toml
version = "26.8.0"
```

### Step 2: `runes/sqs/sqs.fav` 実装

既存スタブを上書き（Write ツールを使用）。6 関数を実装:

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

### Step 3: `fav/src/backend/vm.rs` に SQS primitive 4 件追加

> **前提**: `vm.rs` を Read してから Edit する。挿入位置は `"AWS.sqs_get_queue_url_raw"` ブロックの直後。

追加する 4 件（既存 `AWS.sqs_*_raw` ブロックとの一貫性を保つ実装）:

```rust
        // ── SQS Rune primitives (v26.8.0) ────────────────────────────────
        "SQS.send_message_batch_raw" => {
            let mut it = args.into_iter();
            let queue_url = vm_string(
                it.next().ok_or("SQS.send_message_batch_raw: missing queue_url")?,
                "SQS.send_message_batch_raw",
            )?;
            let messages = match it.next().ok_or("SQS.send_message_batch_raw: missing messages")? {
                VMValue::List(list) => list,
                _ => return Err("SQS.send_message_batch_raw: messages must be a List".to_string()),
            };
            let config = get_aws_config();
            let mut form = "Action=SendMessageBatch&Version=2012-11-05".to_string();
            for (i, msg) in messages.iter().enumerate() {
                let body = match msg {
                    VMValue::Str(s) => s.clone(),
                    _ => format!("{:?}", msg),
                };
                let n = i + 1; // SQS バッチエントリは 1-indexed（SQS API 仕様）
                form.push_str(&format!(
                    "&SendMessageBatchRequestEntry.{n}.Id=msg{n}&SendMessageBatchRequestEntry.{n}.MessageBody={}",
                    url_encode(&body),
                ));
            }
            Ok(match aws_post(&config, "sqs", &queue_url, &form, "application/x-www-form-urlencoded", None) {
                Ok(xml) => {
                    let ids = extract_xml_tags(&xml, "MessageId");
                    ok_vm(VMValue::Str(format!("{{\"sent\":{}}}", ids.len())))
                }
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }

        "SQS.receive_messages_raw" => {
            // AttributeName.1=All: LocalStack 動作確認済み。本番 AWS の MessageAttribute 取得は v27.x で対応。
            let mut it = args.into_iter();
            let queue_url = vm_string(
                it.next().ok_or("SQS.receive_messages_raw: missing queue_url")?,
                "SQS.receive_messages_raw",
            )?;
            let max = match it.next().ok_or("SQS.receive_messages_raw: missing max")? {
                VMValue::Int(n) => n,
                _ => 1,
            };
            let config = get_aws_config();
            let form = format!(
                "Action=ReceiveMessage&MaxNumberOfMessages={}&AttributeName.1=All&Version=2012-11-05",
                max
            );
            Ok(match aws_post(&config, "sqs", &queue_url, &form, "application/x-www-form-urlencoded", None) {
                Ok(xml) => {
                    let messages = extract_xml_tags(&xml, "Message");
                    let items: Vec<String> = messages.into_iter().map(|msg| {
                        let id = extract_xml_tags(&msg, "MessageId").into_iter().next().unwrap_or_default();
                        let body = extract_xml_tags(&msg, "Body").into_iter().next().unwrap_or_default();
                        let handle = extract_xml_tags(&msg, "ReceiptHandle").into_iter().next().unwrap_or_default();
                        format!("{{\"message_id\":\"{}\",\"body\":\"{}\",\"receipt_handle\":\"{}\"}}", id, body, handle)
                    }).collect();
                    ok_vm(VMValue::Str(format!("[{}]", items.join(","))))
                }
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }

        "SQS.purge_raw" => {
            let mut it = args.into_iter();
            let queue_url = vm_string(
                it.next().ok_or("SQS.purge_raw: missing queue_url")?,
                "SQS.purge_raw",
            )?;
            let config = get_aws_config();
            let form = "Action=PurgeQueue&Version=2012-11-05".to_string();
            Ok(match aws_post(&config, "sqs", &queue_url, &form, "application/x-www-form-urlencoded", None) {
                Ok(_) => ok_vm(VMValue::Unit),
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }

        "SQS.consume_raw" => {
            let mut it = args.into_iter();
            let queue_url = vm_string(
                it.next().ok_or("SQS.consume_raw: missing queue_url")?,
                "SQS.consume_raw",
            )?;
            // Stub: 1 回ポーリングして JSON 文字列で返す（継続ループは v27.x）
            let config = get_aws_config();
            let form = "Action=ReceiveMessage&MaxNumberOfMessages=10&AttributeName.1=All&Version=2012-11-05".to_string();
            Ok(match aws_post(&config, "sqs", &queue_url, &form, "application/x-www-form-urlencoded", None) {
                Ok(xml) => {
                    let messages = extract_xml_tags(&xml, "Message");
                    let items: Vec<String> = messages.into_iter().map(|msg| {
                        let id = extract_xml_tags(&msg, "MessageId").into_iter().next().unwrap_or_default();
                        let body = extract_xml_tags(&msg, "Body").into_iter().next().unwrap_or_default();
                        let handle = extract_xml_tags(&msg, "ReceiptHandle").into_iter().next().unwrap_or_default();
                        format!("{{\"message_id\":\"{}\",\"body\":\"{}\",\"receipt_handle\":\"{}\"}}", id, body, handle)
                    }).collect();
                    ok_vm(VMValue::Str(format!("[{}]", items.join(","))))
                }
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }
```

> **挿入位置**: `"AWS.sqs_get_queue_url_raw"` ブロックの末尾 `}` の直後（同じ `match primitive_name { ... }` ブロック内）。
> **インデントの注意**: `match` アームは 8 スペースインデント（既存 `"AWS.sqs_*_raw"` アームと同じ）。

### Step 4: `site/content/docs/runes/sqs.mdx` 新規作成

既存の `site/content/docs/runes/kafka.mdx` の形式に合わせて作成。以下を含める:
- SQS Rune 概要・LocalStack セットアップ
- 環境変数一覧（AWS_ENDPOINT_URL / AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY / AWS_DEFAULT_REGION）
- 全 6 関数のシグネチャ・使用例
- スコープ外（継続ループ・型付きデシリアライズ・FIFO キュー）

### Step 5: `CHANGELOG.md` 更新

```markdown
## [v26.8.0] — 2026-06-27 — SQS Rune 実質化

### Added
- `runes/sqs/sqs.fav` — SQS Rune 実質化（`send_message` / `send_message_batch` / `receive_messages` / `delete_message` / `purge` / `consume` 6 関数）
- `fav/src/backend/vm.rs` — `SQS.send_message_batch_raw` / `SQS.receive_messages_raw` / `SQS.purge_raw` / `SQS.consume_raw` primitive 追加
- `site/content/docs/runes/sqs.mdx` — SQS Rune ドキュメント
```

### Step 6: `benchmarks/v26.8.0.json` 新規作成

```json
{"version":"26.8.0","test_count":2102,"timestamp":"2026-06-27"}
```

### Step 7: `fav/src/driver.rs` に `v268000_tests` 追加

> **前提**: Step 2（`runes/sqs/sqs.fav` 実装）と Step 3（vm.rs 更新）が完了していること。

`v267000_tests` の直後に追加（8 件）:

```rust
// ── v268000_tests (v26.8.0) — SQS Rune 実質化 ─────────────────────────────
#[cfg(test)]
mod v268000_tests {
    #[test]
    fn sqs_rune_has_send_message_fn() {
        let src = include_str!("../../runes/sqs/sqs.fav");
        assert!(src.contains("fn send_message"), "sqs rune must define send_message");
    }
    #[test]
    fn sqs_rune_has_send_message_batch_fn() {
        let src = include_str!("../../runes/sqs/sqs.fav");
        assert!(src.contains("fn send_message_batch"), "sqs rune must define send_message_batch");
    }
    #[test]
    fn sqs_rune_has_receive_messages_fn() {
        let src = include_str!("../../runes/sqs/sqs.fav");
        assert!(src.contains("fn receive_messages"), "sqs rune must define receive_messages");
    }
    #[test]
    fn sqs_rune_has_delete_message_fn() {
        let src = include_str!("../../runes/sqs/sqs.fav");
        assert!(src.contains("fn delete_message"), "sqs rune must define delete_message");
    }
    #[test]
    fn sqs_rune_has_purge_fn() {
        let src = include_str!("../../runes/sqs/sqs.fav");
        assert!(src.contains("fn purge"), "sqs rune must define purge");
    }
    #[test]
    fn sqs_rune_has_consume_fn() {
        let src = include_str!("../../runes/sqs/sqs.fav");
        assert!(src.contains("fn consume"), "sqs rune must define consume");
    }
    #[test]
    fn sqs_rune_vm_has_send_message_batch_raw() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("SQS.send_message_batch_raw"), "vm.rs must implement SQS.send_message_batch_raw");
    }
    #[test]
    fn changelog_has_v26_8_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("[v26.8.0]"), "CHANGELOG.md must contain '[v26.8.0]'");
    }
}
```

> `include_str!("backend/vm.rs")` — `driver.rs` は `fav/src/driver.rs` に存在するため、
> `backend/vm.rs` は `fav/src/backend/vm.rs` を指す（同一 `src/` ディレクトリ内の相対パス）。

### Step 8: テスト確認

```bash
cd fav && cargo test sqs --bin fav          # 7 件以上 PASS（v268000_tests の sqs_rune_* 7 件）
cd fav && cargo test v268000 --bin fav      # 8/8 PASS
cd fav && cargo test --bin fav -j 8 -- --test-threads=8 2>&1 | tail -4  # 2102 件 PASS
```

---

## ファイル変更一覧

| ファイル | 操作 |
|---|---|
| `fav/Cargo.toml` | version bump 26.7.0 → 26.8.0 |
| `runes/sqs/sqs.fav` | **上書き実装**（空スタブ → 6 関数） |
| `fav/src/backend/vm.rs` | `SQS.*_raw` primitive 4 件追加 |
| `site/content/docs/runes/sqs.mdx` | **新規作成** |
| `CHANGELOG.md` | `[v26.8.0]` エントリ先頭に追加 |
| `benchmarks/v26.8.0.json` | **新規作成** |
| `fav/src/driver.rs` | `v268000_tests`（8 件）追加 |

---

## 注意事項

- `runes/aws/sqs.fav`（legacy）は**変更しない**。`runes/sqs/sqs.fav` のみ更新する。
- `send_message` / `delete_message` は既存 `AWS.sqs_send_message_raw` / `AWS.sqs_delete_message_raw` に委譲。vm.rs 追加不要。
- `send_message_batch_raw` のバッチエントリフォーマット: `SendMessageBatchRequestEntry.N.Id=msgN&SendMessageBatchRequestEntry.N.MessageBody=...`（SQS API 仕様）。
- `SQS.receive_messages_raw` は `AWS.sqs_receive_messages_raw`（List 返却）とは異なり JSON 文字列を返す。型が異なるため別 primitive として追加。
- vm.rs への挿入は `"AWS.sqs_get_queue_url_raw"` ブロック直後（`"AWS.sqs_*"` セクションとまとめて管理）。

## リスクと対応

| リスク | 対応 |
|---|---|
| vm.rs の `include_str!("backend/vm.rs")` パスが誤り | `driver.rs` が `fav/src/driver.rs` にあるため `backend/vm.rs` = `fav/src/backend/vm.rs` で正しい |
| `SQS.send_message_batch_raw` のインデントが既存コードと異なる | Read で周辺行を確認してからインデントを合わせて Edit |
| `url_encode` / `aws_post` / `extract_xml_tags` が未定義 | 既存 `AWS.sqs_*_raw` ブロックと同じスコープ内のため利用可能 |
