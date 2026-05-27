# Favnir v7.3.0 Tasks

Date: 2026-05-27
Theme: Rune エコシステム拡充（fs / slack / queue / cache）

---

## Phase A — fs Rune

### A-1: VM primitives（vm.rs / checker.rs）

- [ ] A-1-1: IO.list_dir_raw(path: String) -> List<String> !IO
- [ ] A-1-2: IO.file_stat_raw(path: String) -> Map<String, String> !IO
- [ ] A-1-3: checker.rs に型シグネチャ追加

### A-2: runes/fs/ 作成

- [ ] A-2-1: runes/fs/rune.toml
- [ ] A-2-2: runes/fs/fs.fav
  - [ ] Fs.read(path) -> Result<String, String> !IO
  - [ ] Fs.write(path, content) -> Result<Unit, String> !IO
  - [ ] Fs.list_dir(path) -> Result<List<String>, String> !IO
  - [ ] Fs.exists(path) -> Bool !IO
  - [ ] Fs.is_dir(path) -> Bool !IO
  - [ ] Fs.size(path) -> Option<Int> !IO
  - [ ] Fs.glob(pattern) -> Result<List<String>, String> !IO
  - [ ] Fs.walk(dir) -> Result<List<String>, String> !IO
- [ ] A-2-3: fav check runes/fs/fs.fav — no errors

### A-3: テスト

- [ ] A-3-1: fs_exists_test
- [ ] A-3-2: fs_read_write_test
- [ ] A-3-3: fs_list_dir_test

---

## Phase B — slack Rune

### B-1: runes/slack/ 作成

- [ ] B-1-1: runes/slack/rune.toml
- [ ] B-1-2: runes/slack/slack.fav
  - [ ] Slack.notify(webhook_url, text) -> Result<Unit, String> !Network
  - [ ] Slack.post_blocks(webhook_url, blocks) -> Result<Unit, String> !Network
  - [ ] Slack.build_section(text) -> String
  - [ ] Slack.build_header(text) -> String
  - [ ] Slack.build_divider() -> String
  - [ ] Slack.build_message(blocks: List<String>) -> String
- [ ] B-1-3: fav check runes/slack/slack.fav — no errors

### B-2: テスト

- [ ] B-2-1: slack_build_section_test — JSON に "type":"section" を含む
- [ ] B-2-2: slack_build_header_test — JSON に "type":"header" を含む
- [ ] B-2-3: slack_build_message_test — build_message でラップした JSON を確認

---

## Phase C — queue Rune

### C-1: VM primitives（vm.rs / checker.rs）

- [ ] C-1-1: Queue.send_raw(url, body) -> Result<Unit, String> !Queue
- [ ] C-1-2: Queue.recv_raw(url, max) -> Result<List<String>, String> !Queue
- [ ] C-1-3: Queue.ack_raw(url, receipt) -> Result<Unit, String> !Queue
- [ ] C-1-4: Queue.delete_raw(url, receipt) -> Result<Unit, String> !Queue
- [ ] C-1-5: !Queue を BUILTIN_EFFECTS に追加（checker.rs）
- [ ] C-1-6: checker.rs に型シグネチャ追加

### C-2: runes/queue/ 作成

- [ ] C-2-1: runes/queue/rune.toml
- [ ] C-2-2: runes/queue/queue.fav
  - [ ] Queue.send(url, body) -> Result<Unit, String> !Queue
  - [ ] Queue.send_batch(url, messages) -> Result<Int, String> !Queue
  - [ ] Queue.recv(url, max) -> Result<List<String>, String> !Queue
  - [ ] Queue.ack(url, receipt) -> Result<Unit, String> !Queue
  - [ ] Queue.nack(url, receipt) -> Result<Unit, String> !Queue
- [ ] C-2-3: fav check runes/queue/queue.fav — no errors

### C-3: テスト

- [ ] C-3-1: queue_fav_check_test
- [ ] C-3-2: queue_send_batch_empty_test

---

## Phase D — cache Rune

### D-1: VM primitives（vm.rs / checker.rs）

- [ ] D-1-1: Cache.get_raw(key) -> Option<String>
- [ ] D-1-2: Cache.set_raw(key, value, ttl_secs) -> Unit
- [ ] D-1-3: Cache.del_raw(key) -> Unit
- [ ] D-1-4: Cache.exists_raw(key) -> Bool
- [ ] D-1-5: Cache.del_prefix_raw(prefix) -> Int
- [ ] D-1-6: !Cache を BUILTIN_EFFECTS に追加（checker.rs）
- [ ] D-1-7: checker.rs に型シグネチャ追加

### D-2: runes/cache/ 作成

- [ ] D-2-1: runes/cache/rune.toml
- [ ] D-2-2: runes/cache/cache.fav
  - [ ] Cache.get(key) -> Option<String>
  - [ ] Cache.get_or(key, default) -> String
  - [ ] Cache.set(key, value) -> Unit
  - [ ] Cache.set_ttl(key, value, ttl_secs) -> Unit
  - [ ] Cache.del(key) -> Unit
  - [ ] Cache.invalidate_prefix(prefix) -> Int
- [ ] D-2-3: fav check runes/cache/cache.fav — no errors

### D-3: テスト

- [ ] D-3-1: cache_set_get_test
- [ ] D-3-2: cache_del_test
- [ ] D-3-3: cache_get_or_test

---

## Phase E — ドキュメント

- [ ] E-1: site/content/docs/runes/fs.mdx
- [ ] E-2: site/content/docs/runes/slack.mdx
- [ ] E-3: site/content/docs/runes/queue.mdx
- [ ] E-4: site/content/docs/runes/cache.mdx

---

## Phase F — 最終確認

- [ ] F-1: cargo test — 全件通過（目標: 1090+ tests）
- [ ] F-2: fav check 4 Rune すべてエラーなし
- [ ] F-3: このファイルを完了状態に更新
- [ ] F-4: commit & push

---

## 完了条件

- fs / slack / queue / cache Rune が fav check を通る
- 各 Rune に 3 件以上の統合テスト
- !Queue / !Cache エフェクトが checker で追跡される
- 既存テスト 1061 件が全件通る
- サイトドキュメント 4 ページ追加
