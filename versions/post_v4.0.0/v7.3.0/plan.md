# Favnir v7.3.0 Plan

Date: 2026-05-27
Theme: Rune エコシステム拡充

---

## 実装順序の方針

VM primitive が既存で足りる Rune（fs / slack）を先に実装し、
新規 VM primitive が必要な Rune（queue / cache）は後回しにする。
こうすることで各フェーズが独立してテスト可能になる。

```
Phase A: fs Rune      — IO.read_file_raw など既存 primitive を wrap
Phase B: slack Rune   — Http.post_raw を wrap する Favnir 層
Phase C: queue Rune   — Queue.*_raw 新規追加 → Favnir 層
Phase D: cache Rune   — Cache.*_raw 新規追加 → Favnir 層
Phase E: docs         — 4 Rune のサイトドキュメント
Phase F: final        — cargo test 全件確認 + commit
```

---

## Phase A — fs Rune（3〜4時間）

### A-1: VM primitives 追加（vm.rs / checker.rs）

新規追加：
- `IO.list_dir_raw(path) -> List<String>` — `std::fs::read_dir` を使用
- `IO.file_stat_raw(path) -> Map<String, String>` — exists / size / is_dir を返す

checker.rs にも型シグネチャを追加する。

### A-2: runes/fs/ 作成

- `runes/fs/rune.toml`
- `runes/fs/fs.fav` — read / write / list_dir / exists / is_dir / size / glob / walk

`glob` は `list_dir` + `String.ends_with` でパターンマッチを Favnir 層で実装。
`walk` は再帰が必要なため `walk_inner` ヘルパー関数を使う（`bind inside closure 不可` に注意）。

### A-3: テスト（driver.rs `fs_rune_tests`）

- `fs_exists_test` — `IO.file_exists_raw` / `Fs.exists` で既存ファイルが true
- `fs_read_write_test` — tmp パスに書いて読む
- `fs_list_dir_test` — `list_dir` でファイル数 > 0

---

## Phase B — slack Rune（2〜3時間）

### B-1: runes/slack/ 作成

- `runes/slack/rune.toml`
- `runes/slack/slack.fav` — notify / post_blocks / build_section / build_header / build_divider / build_message

`notify` は `Http.post_raw(url, body, "application/json")` を呼ぶだけ。
`build_*` 関数は JSON 文字列を組み立てる純粋関数。

### B-2: テスト（driver.rs `slack_rune_tests`）

HTTP は実際には叩かないため、ビルダー（純粋関数）のテストのみ：
- `slack_build_section_test` — JSON 文字列に "type":"section" を含む
- `slack_build_header_test` — JSON 文字列に "type":"header" を含む
- `slack_build_message_test` — build_message でラップした JSON を確認

---

## Phase C — queue Rune（4〜5時間）

### C-1: VM primitives 追加（vm.rs / checker.rs）

新規追加（SQS thin wrapper）：
- `Queue.send_raw(url, body) -> Result<Unit, String> !Queue`
- `Queue.recv_raw(url, max) -> Result<List<String>, String> !Queue`
- `Queue.ack_raw(url, receipt) -> Result<Unit, String> !Queue`
- `Queue.delete_raw(url, receipt) -> Result<Unit, String> !Queue`

`!Queue` エフェクトを `BUILTIN_EFFECTS` に追加する。

実装は既存 AWS SQS infra（SendMessage / ReceiveMessage / DeleteMessage API）を使用。
SigV4 は既存の `sign_sqs_request` ヘルパーを再利用する。

### C-2: runes/queue/ 作成

- `runes/queue/rune.toml`
- `runes/queue/queue.fav` — send / send_batch / recv / ack / nack

`send_batch` は `List.map` で send を繰り返し、成功数を返す。
`nack` は `Queue.ack_raw` の代わりに visibility timeout を 0 にリセット（
  実際は SQS の ChangeMessageVisibility — stub で Unit を返す）。

### C-3: テスト（driver.rs `queue_rune_tests`）

VM primitive は SQS 不要の pure 関数のみテスト：
- `queue_fav_check_test` — `fav check runes/queue/queue.fav` passes
- `queue_send_batch_empty_test` — 空リストで send_batch → Ok(0)

---

## Phase D — cache Rune（3〜4時間）

### D-1: VM primitives 追加（vm.rs / checker.rs）

インプロセス `HashMap` を `thread_local!` + `Mutex` で保持：
- `Cache.get_raw(key) -> Option<String>`
- `Cache.set_raw(key, value, ttl_secs) -> Unit`
- `Cache.del_raw(key) -> Unit`
- `Cache.exists_raw(key) -> Bool`

`!Cache` エフェクトを `BUILTIN_EFFECTS` に追加する。
TTL は実際には無視（stub: 永続 HashMap のみ）でよい。

### D-2: runes/cache/ 作成

- `runes/cache/rune.toml`
- `runes/cache/cache.fav` — get / get_or / set / set_ttl / del / invalidate_prefix

`invalidate_prefix` は `Cache.get_raw` で実装できないため、
`Cache.del_prefix_raw(prefix) -> Int` を VM に追加する。

### D-3: テスト（driver.rs `cache_rune_tests`）

- `cache_set_get_test` — set してから get → Some(value)
- `cache_del_test` — set → del → get → None
- `cache_get_or_test` — 存在しないキーで get_or → default 値

---

## Phase E — ドキュメント（2〜3時間）

4 ファイルを作成：
- `site/content/docs/runes/fs.mdx`
- `site/content/docs/runes/slack.mdx`
- `site/content/docs/runes/queue.mdx`
- `site/content/docs/runes/cache.mdx`

各ページ共通構成：
1. 概要（エフェクト・インストール）
2. 主要 API リファレンス（シグネチャ + 例）
3. 使用例（実際のパイプライン）
4. エフェクト一覧表

---

## Phase F — 最終確認

- `cargo test` — 全件通過確認（目標: 1090+ tests）
- `fav check runes/fs/fs.fav` — no errors
- `fav check runes/slack/slack.fav` — no errors
- `fav check runes/queue/queue.fav` — no errors
- `fav check runes/cache/cache.fav` — no errors
- commit & push

---

## 注意点・ハマりポイント

- **`!Queue` / `!Cache` の BUILTIN_EFFECTS 登録忘れ**: checker.rs に追加必須
- **`walk` 再帰**: Favnir は bind-in-closure 不可なので `walk_inner` 関数を別途定義する
- **`glob` パターン**: `*` ワイルドカード完全実装は不要 — 拡張子マッチ（例: ".csv"）を `String.ends_with` で実装
- **Queue VM 実装**: SQS の XML パースは既存の `extract_xml_tags` を使う
- **Cache スレッド安全**: `thread_local!` + `RefCell<HashMap>` が最もシンプル
