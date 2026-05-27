# Favnir v7.3.0 Spec

Date: 2026-05-27
Theme: Rune エコシステム拡充

---

## 概要

v7.2.0（SQL Rune）で確立した「VM primitive = 薄い接続層 / Favnir 層 = 意味のある操作」パターンを
他の Rune にも適用し、エコシステムを拡充する。

| Rune    | VM primitive（Rust）                         | Favnir 層の追加価値                              |
|---------|---------------------------------------------|--------------------------------------------------|
| `fs`    | `IO.read_file_raw`（既存）                   | ディレクトリ walk・glob・ファイル stat            |
| `slack` | `Http.post_raw`（既存）                      | Slack Webhook 通知・メッセージビルダー           |
| `queue` | `Queue.send_raw` / `Queue.recv_raw`（新規）  | バッチ送信・dead letter・ack 管理                |
| `cache` | `Cache.get_raw` / `Cache.set_raw`（新規）    | TTL 管理・invalidation パターン                  |

`email` Rune は `Smtp.send_raw` が必要で VM 実装コストが高いため v7.4.0 に延期。
まず既存 VM primitive で完結できる `fs` / `slack` を先に実装し、
次に VM に軽量 primitive を追加する `queue` / `cache` を実装する。

---

## Phase A — fs Rune

### 追加する VM primitives（vm.rs / checker.rs）

```
IO.list_dir_raw(path: String) -> List<String>  !IO
IO.file_stat_raw(path: String) -> Map<String, String>  !IO
  // returns: { "exists": "true"/"false", "size": "<bytes>", "is_dir": "true"/"false" }
```

### runes/fs/ 構造

```
runes/fs/
  rune.toml
  fs.fav        — public API: read, write, list_dir, exists, stat, glob, walk
```

### 主要 API

```favnir
// ファイル読み書き（IO.read_file_raw / IO.write_file_raw を wrap）
public fn read(path: String) -> Result<String, String> !IO
public fn write(path: String, content: String) -> Result<Unit, String> !IO

// ディレクトリ一覧
public fn list_dir(path: String) -> Result<List<String>, String> !IO

// stat
public fn exists(path: String) -> Bool !IO
public fn is_dir(path: String) -> Bool !IO
public fn size(path: String) -> Option<Int> !IO

// glob — path パターン（例: "data/*.csv"）でファイル一覧取得
public fn glob(pattern: String) -> Result<List<String>, String> !IO

// walk — ディレクトリを再帰的にたどり全ファイルパスを返す
public fn walk(dir: String) -> Result<List<String>, String> !IO
```

---

## Phase B — slack Rune

### 使用する VM primitive

`Http.post_raw`（既存）— JSON ボディを Slack Webhook URL に POST するだけ。

### runes/slack/ 構造

```
runes/slack/
  rune.toml
  slack.fav     — public API: notify, post_blocks, build_section, build_header
```

### 主要 API

```favnir
// シンプル通知（テキストのみ）
public fn notify(webhook_url: String, text: String) -> Result<Unit, String> !Network

// Block Kit メッセージ（blocks: JSON 文字列）
public fn post_blocks(webhook_url: String, blocks: String) -> Result<Unit, String> !Network

// Block Kit ビルダー（純粋）
public fn build_section(text: String) -> String
public fn build_header(text: String) -> String
public fn build_divider() -> String
public fn build_message(blocks: List<String>) -> String
```

---

## Phase C — queue Rune（VM primitive 新規追加）

### 追加する VM primitives

```
Queue.send_raw(url: String, body: String) -> Result<Unit, String>  !Queue
Queue.recv_raw(url: String, max: Int) -> Result<List<String>, String>  !Queue
Queue.ack_raw(url: String, receipt: String) -> Result<Unit, String>  !Queue
Queue.delete_raw(url: String, receipt: String) -> Result<Unit, String>  !Queue
```

バックエンドは SQS（既存の AWS infra を再利用）。
`Queue.*_raw` は SQS の thin wrapper。`!Queue` エフェクトを checker に登録。

### runes/queue/ 構造

```
runes/queue/
  rune.toml
  queue.fav     — public API: send, send_batch, recv, ack, nack
```

### 主要 API

```favnir
// 1件送信
public fn send(url: String, body: String) -> Result<Unit, String> !Queue

// バッチ送信（最大10件）
public fn send_batch(url: String, messages: List<String>) -> Result<Int, String> !Queue

// 受信（最大 max 件、デフォルト 1）
public fn recv(url: String, max: Int) -> Result<List<String>, String> !Queue

// Ack（処理完了、キューから削除）
public fn ack(url: String, receipt: String) -> Result<Unit, String> !Queue

// Nack（再試行させる — visibility timeout をリセット）
public fn nack(url: String, receipt: String) -> Result<Unit, String> !Queue
```

---

## Phase D — cache Rune（VM primitive 新規追加）

### 追加する VM primitives

```
Cache.get_raw(key: String) -> Option<String>  !Cache
Cache.set_raw(key: String, value: String, ttl_secs: Int) -> Unit  !Cache
Cache.del_raw(key: String) -> Unit  !Cache
Cache.exists_raw(key: String) -> Bool  !Cache
```

バックエンドはインプロセス HashMap（開発用）。
本番では環境変数 `CACHE_BACKEND=redis` を参照して Redis に転送する設計とする（stub 実装でよい）。
`!Cache` エフェクトを checker に登録。

### runes/cache/ 構造

```
runes/cache/
  rune.toml
  cache.fav     — public API: get, get_or, set, set_ttl, del, invalidate_prefix
```

### 主要 API

```favnir
// 取得
public fn get(key: String) -> Option<String> !Cache

// 取得（なければデフォルト値）
public fn get_or(key: String, default: String) -> String !Cache

// 保存（TTL なし = 永続）
public fn set(key: String, value: String) -> Unit !Cache

// 保存（TTL 秒指定）
public fn set_ttl(key: String, value: String, ttl_secs: Int) -> Unit !Cache

// 削除
public fn del(key: String) -> Unit !Cache

// プレフィックス一括無効化（例: "user:42:" 以下を全削除）
public fn invalidate_prefix(prefix: String) -> Int !Cache
```

---

## Phase E — ドキュメント

各 Rune に対してサイトドキュメントを追加する。

| ファイル | 内容 |
|---------|------|
| `site/content/docs/runes/fs.mdx` | fs Rune リファレンス |
| `site/content/docs/runes/slack.mdx` | slack Rune リファレンス |
| `site/content/docs/runes/queue.mdx` | queue Rune リファレンス |
| `site/content/docs/runes/cache.mdx` | cache Rune リファレンス |

---

## 完了条件

- 4 Rune（fs / slack / queue / cache）の `fav check` がエラーなし
- 各 Rune に最低 3 件の統合テスト
- `!Queue` / `!Cache` エフェクトが checker で追跡される
- 既存テスト 1061 件が全件通る（新規テスト込みで 1090+ 目標）
- サイトドキュメント 4 ページ追加
