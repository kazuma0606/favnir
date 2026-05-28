# Favnir v7.3.0 Tasks

Date: 2026-05-27
Theme: Rune エコシステム拡充（fs / slack / queue / cache）

---

## Phase A — fs Rune

### A-1: VM primitives（vm.rs / checker.rs）

- [x] A-1-1: IO.list_dir_raw(path: String) -> Result<List<String>, String> !IO
- [x] A-1-2: IO.file_stat_raw(path: String) -> Map<String, String> !IO
- [x] A-1-3: checker.rs に型シグネチャ追加

### A-2: runes/fs/ 作成

- [x] A-2-1: runes/fs/rune.toml
- [x] A-2-2: runes/fs/fs.fav
  - [x] Fs.read(path) -> Result<String, String> !IO
  - [x] Fs.write(path, content) -> Result<Unit, String> !IO
  - [x] Fs.list_dir(path) -> Result<List<String>, String> !IO
  - [x] Fs.exists(path) -> Bool !IO
  - [x] Fs.is_dir(path) -> Bool !IO
  - [x] Fs.size(path) -> Option<Int> !IO
  - [x] Fs.glob(pattern) -> Result<List<String>, String> !IO
  - [x] Fs.walk(dir) -> Result<List<String>, String> !IO
- [x] A-2-3: fav check runes/fs/fs.fav — no errors

### A-3: テスト

- [x] A-3-1: fs_exists_test
- [x] A-3-2: fs_read_write_test
- [x] A-3-3: fs_list_dir_test

---

## Phase B — slack Rune

### B-1: runes/slack/ 作成

- [x] B-1-1: runes/slack/rune.toml
- [x] B-1-2: runes/slack/slack.fav
  - [x] Slack.notify(webhook_url, text) -> Result<Unit, String> !Network
  - [x] Slack.post_blocks(webhook_url, blocks) -> Result<Unit, String> !Network
  - [x] Slack.build_section(text) -> String
  - [x] Slack.build_header(text) -> String
  - [x] Slack.build_divider() -> String
  - [x] Slack.build_message(blocks: List<String>) -> String
- [x] B-1-3: fav check runes/slack/slack.fav — no errors

### B-2: テスト

- [x] B-2-1: slack_build_section_test — JSON に "type":"section" を含む
- [x] B-2-2: slack_build_header_test — JSON に "type":"header" を含む
- [x] B-2-3: slack_build_message_test — build_message でラップした JSON を確認

---

## Phase C — queue Rune

### C-1: VM primitives（vm.rs / checker.rs）

- [x] C-1-1: Queue.send_raw(url, body) -> Result<Unit, String> !Queue
- [x] C-1-2: Queue.recv_raw(url, max) -> Result<List<String>, String> !Queue
- [x] C-1-3: Queue.ack_raw(url, receipt) -> Result<Unit, String> !Queue
- [x] C-1-4: Queue.delete_raw(url, receipt) -> Result<Unit, String> !Queue
- [x] C-1-5: !Queue を BUILTIN_EFFECTS に追加（checker.rs）
- [x] C-1-6: checker.rs に型シグネチャ追加

### C-2: runes/queue/ 作成

- [x] C-2-1: runes/queue/rune.toml
- [x] C-2-2: runes/queue/queue.fav
  - [x] Queue.send(url, body) -> Result<Unit, String> !Queue
  - [x] Queue.send_batch(url, messages) -> Result<Int, String> !Queue
  - [x] Queue.recv(url, max) -> Result<List<String>, String> !Queue
  - [x] Queue.ack(url, receipt) -> Result<Unit, String> !Queue
  - [x] Queue.nack(url, receipt) -> Result<Unit, String> !Queue
- [x] C-2-3: fav check runes/queue/queue.fav — no errors

### C-3: テスト

- [x] C-3-1: queue_fav_check_test
- [x] C-3-2: queue_send_batch_empty_test

---

## Phase D — cache Rune

### D-1: VM primitives（vm.rs / checker.rs）

- [x] D-1-1: Cache.get_raw(key) -> Option<String>
- [x] D-1-2: Cache.set_raw(key, value, ttl_secs) -> Unit
- [x] D-1-3: Cache.del_raw(key) -> Unit
- [x] D-1-4: Cache.exists_raw(key) -> Bool
- [x] D-1-5: Cache.del_prefix_raw(prefix) -> Int
- [x] D-1-6: !Cache を BUILTIN_EFFECTS に追加（checker.rs）
- [x] D-1-7: checker.rs に型シグネチャ追加

### D-2: runes/cache/ 作成

- [x] D-2-1: runes/cache/rune.toml
- [x] D-2-2: runes/cache/cache.fav
  - [x] Cache.get(key) -> Option<String>
  - [x] Cache.get_or(key, default) -> String
  - [x] Cache.set(key, value) -> Unit
  - [x] Cache.set_ttl(key, value, ttl_secs) -> Unit
  - [x] Cache.del(key) -> Unit
  - [x] Cache.invalidate_prefix(prefix) -> Int
- [x] D-2-3: fav check runes/cache/cache.fav — no errors

### D-3: テスト

- [x] D-3-1: cache_set_get_test
- [x] D-3-2: cache_del_test
- [x] D-3-3: cache_get_or_test

---

## Phase E — ドキュメント

- [x] E-1: site/content/docs/runes/fs.mdx
- [x] E-2: site/content/docs/runes/slack.mdx
- [x] E-3: site/content/docs/runes/queue.mdx
- [x] E-4: site/content/docs/runes/cache.mdx

---

## Phase F — 最終確認

- [x] F-1: cargo test — 1070 tests passed; 0 failed
- [x] F-2: fav check 4 Rune すべてエラーなし
- [x] F-3: このファイルを完了状態に更新
- [x] F-4: commit & push

---

## 完了条件

- fs / slack / queue / cache Rune が fav check を通る ✓
- 各 Rune に 3 件以上の統合テスト ✓（fs×3、slack×3、cache×3）
- !IO / !Queue / !Cache エフェクトが checker で追跡される ✓
- 既存テスト 1061 件が全件通る ✓（1070 tests passing）
- サイトドキュメント 4 ページ追加 ✓
