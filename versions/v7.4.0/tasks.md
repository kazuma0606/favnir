# Favnir v7.4.0 Tasks

Date: 2026-05-27
Theme: stdlib 高レベル層（Favnir 化）+ email Rune

---

## Phase A-1: VM primitives 追加（vm.rs / checker.rs / compiler.rs）

- [x] A-1-1: Map.empty() -> Map<K, V> — vm.rs に実装
- [x] A-1-2: String.compare(a, b) -> Int — vm.rs に実装（負/0/正）
- [x] A-1-3: checker.rs に型シグネチャ追加（Map.empty, String.compare）
- [x] A-1-4: compiler.rs 名前空間リストに Map/String が登録済みか確認（追加不要のはず）

---

## Phase A-2: runes/stdlib/list.fav

- [x] A-2-1: runes/stdlib/ ディレクトリ作成 + runes/stdlib/rune.toml
- [x] A-2-2: runes/stdlib/list.fav 作成
  - [x] group_by(xs, key_fn) -> Map<String, List<A>>
  - [x] zip_with(xs, ys, f) -> List<C>
  - [x] sort_by(xs, key_fn) -> List<A>
  - [x] intersperse(xs, sep) -> List<A>
  - [x] tail(xs) -> List<A>
  - [x] head(xs) -> Option<A>
- [x] A-2-3: fav check runes/stdlib/list.fav — no errors

---

## Phase A-3: runes/stdlib/map.fav

- [x] A-3-1: runes/stdlib/map.fav 作成
  - [x] count_by(xs, key_fn) -> Map<String, Int>
  - [x] merge_with(a, b, f) -> Map<String, A>
  - [x] from_entries(entries, key_fn, val_fn) -> Map<String, B>
  - [x] keys(m) -> List<String>
  - [x] values(m) -> List<A>
- [x] A-3-2: fav check runes/stdlib/map.fav — no errors

---

## Phase B: email Rune

### B-1: VM primitive（vm.rs / checker.rs / compiler.rs）

- [x] B-1-1: Email.send_raw(from, to, subject, body) -> Result<Unit, String> — SES 実装
- [x] B-1-2: !Email を BUILTIN_EFFECTS に追加（checker.rs）
- [x] B-1-3: "Email" を compiler.rs 名前空間リスト（2箇所）に追加
- [x] B-1-4: checker.rs に Email.send_raw 型シグネチャ追加

### B-2: runes/email/ 作成

- [x] B-2-1: runes/email/rune.toml
- [x] B-2-2: runes/email/email.fav
  - [x] Email.send(from, to, subject, body) -> Result<Unit, String> !Email
  - [x] Email.send_bulk(from, to_list, subject, body) -> Result<Int, String> !Email
  - [x] Email.notify(from, to, message) -> Result<Unit, String> !Email
- [x] B-2-3: fav check runes/email/email.fav — no errors

---

## Phase C: テスト（driver.rs）

### stdlib_list_tests（5 件）

- [x] C-1: stdlib_list_head_test
- [x] C-2: stdlib_list_tail_test
- [x] C-3: stdlib_list_sort_by_test
- [x] C-4: stdlib_list_intersperse_test
- [x] C-5: stdlib_list_group_by_test

### stdlib_map_tests（3 件）

- [x] C-6: stdlib_map_count_by_test
- [x] C-7: stdlib_map_empty_test
- [x] C-8: stdlib_map_from_entries_test

### email_rune_tests（3 件）

- [x] C-9: email_send_raw_type_checks
- [x] C-10: email_send_bulk_type_checks
- [x] C-11: email_notify_subject_test

---

## Phase D: ドキュメント

- [x] D-1: site/content/docs/stdlib/list.mdx に StdList 高レベル関数セクション追加
- [x] D-2: site/content/docs/runes/email.mdx 作成

---

## Phase E: 最終確認

- [x] E-1: cargo test — 1081 tests passing（+20 新規）
- [x] E-2: fav check runes/stdlib/list.fav, map.fav, runes/email/email.fav — all no errors
- [x] E-3: このファイルを完了状態に更新
- [ ] E-4: commit

---

## 完了条件

- runes/stdlib/list.fav, map.fav が fav check を通る ✓
- runes/email/email.fav が fav check を通る ✓
- group_by / zip_with / sort_by / intersperse / tail / head 動作確認済み ✓
- !Email エフェクトが checker で追跡される ✓
- 1081 テスト全件通る ✓
- ドキュメント 2 ページ追加 ✓
