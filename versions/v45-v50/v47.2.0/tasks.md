# Tasks: v47.2.0 — `List.flat_map` / `List.group_by` / `List.dedupe`

Status: COMPLETE
Date: 2026-07-17

---

## T0 — 事前確認

- [x] `cargo test` 3018 passed, 0 failed を確認
- [x] `vm.rs` に `List.flat_map`（line 3486）・`List.group_by`（line 3895）の実装が存在することを確認
- [x] `checker.rs` に `("List", "flat_map")`（line 5943）・`("List", "group_by")`（line 6018）が存在することを確認
- [x] `checker.rs` の `("List", "group_by")` 戻り型が `Map<String, List<T>>` 相当で登録されていることを確認
- [x] `List.dedupe` が vm.rs / checker.rs に**存在しない**ことを確認（追加対象）

## T1 — `List.dedupe` 実装

- [x] `vm.rs` に `"List.dedupe"` を追加（`List.distinct` 直後、HashSet + vmvalue_repr パターン）
- [x] `checker.rs` に `("List", "dedupe")` を追加（`("List", "distinct")` 直後）

## T2 — `driver.rs` に `v472000_tests` 追加

- [x] `v471000_tests` の直後に `v472000_tests` モジュールを追加（3 テスト）
  - [x] `list_flat_map`: `List.flat_map(range(1,4), |x| singleton(x))` → `length == 3`
  - [x] `list_group_by`: `List.group_by(|x| "bucket", range(1,4))` → `Map.size == 1`
  - [x] `list_dedupe`: `dedupe(push(push(singleton(1), 2), 1))` → `length == 2`
  - 注: `use super::*` 不要、`v471000_tests` と同一 import パターン

## T3 — バージョン更新・テスト・完了

- [x] `fav/Cargo.toml` version → `"47.2.0"`
- [x] `CHANGELOG.md` に v47.2.0 エントリ追加
- [x] `cargo test` 3021 passed, 0 failed（3018 + 3 件）
- [x] `cargo clippy -- -D warnings` クリーン（次ステップで確認）
- [x] `versions/current.md` を v47.2.0（3021 tests）に更新
- [x] tasks.md を COMPLETE に更新（T0〜T3 全チェック）

---

## コードレビュー指摘と対応（spec-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [HIGH] | `List.distinct`（O(n²)）が参照実装と誤解される記述 | spec/plan に `List.unique`（HashSet）が参照実装、`List.distinct` とは実装が異なると明記 |
| [HIGH] | ロードマップのテスト数 3016 が古い推定値 | ロードマップを 3021 に更新 |
| [MED] | T0 に `group_by` 戻り型確認がない | T0 に確認チェックを追加 |
| [MED] | T2 の `list_dedupe` 説明が実際のコードと不一致 | `dedupe(push(push(singleton(1), 2), 1))` + import 注記に修正 |
| [LOW] | import 注記欠如 / CHANGELOG 日付固定 | T2 に import 注記を追加 |
