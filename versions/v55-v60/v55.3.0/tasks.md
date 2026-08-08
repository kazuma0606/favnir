# Tasks — v55.3.0 — Exactly-once 意味論（冪等チェックポイント）

## ステータス: COMPLETE（2026-07-24）

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.3.0 セクションを確認
- [x] ベーステスト数 3209（v55.2.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が現在 `55.2.0` であることを確認（更新前）
- [x] `fav/src/backend/vm.rs` の `VM` 構造体で `show_stream_stats` フィールドの直後に追加可能であることを確認（後続に `source_file` 等の非 `#[cfg]` フィールドが続く中間位置）
- [x] `fav/src/backend/vm.rs` の `VM::new_with_db_path` 初期化リストを確認（`show_stream_stats: false,` の直後に追加）
- [x] `fav/src/backend/vm.rs` の `checkpoint_hook` メソッド（L1788〜L1795 付近）を確認（置き換え対象）
- [x] `fav/src/backend/vm.rs` L23 の `use std::collections::{HashMap, HashSet};` が存在することを確認（追加インポート不要の根拠）
- [x] `fav/src/driver.rs` の `v55200_tests` モジュール位置を確認（直前に挿入）
- [x] `v55200_tests` に `cargo_toml_version_is_55_2_0` テストが存在しないことを確認（削除タスク不要）

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `55.3.0` に更新
- [x] T2: `fav/src/backend/vm.rs` の `VM` 構造体に 2 フィールドを追加（`show_stream_stats` の直後）
  - [x] `pub(crate) checkpoint_delivery: Option<String>`
  - [x] `pub(crate) processed_offsets: HashSet<u64>`
- [x] T3: `vm.rs` の `VM::new_with_db_path` 初期化部分に追加（`show_stream_stats: false,` の直後）
  - [x] `checkpoint_delivery: None`
  - [x] `processed_offsets: HashSet::new()`
- [x] T4: `vm.rs` の `checkpoint_hook` を `&self` stub から `&mut self` 実装に置き換え
  - [x] シグネチャを `fn checkpoint_hook(&self, offset: u64)` → `fn checkpoint_hook(&mut self, offset: u64)` に変更
  - [x] `processed_offsets.insert(offset)` の記録ロジックを追加（`delivery == "exactly-once"` 時のみ）
- [x] T5: `vm.rs` に `is_duplicate_offset` メソッドを追加（`checkpoint_hook` の直後）
- [x] T6: `fav/src/driver.rs` に `v55300_tests` モジュールを追加（`v55200_tests` の直前）
  - [x] `exactly_once_checkpoint_saved`（delivery + checkpoint_store 解析）
  - [x] `exactly_once_no_duplicate_on_restart`（delivery + checkpoint_interval_sec 解析）

---

## テスト・検証

- [x] T7: `cargo build` でコンパイルエラーがないことを確認（T8 実施前に実行）
- [x] T8: `cargo test` 全通過（3211 tests passed, 0 failed）
- [x] T9: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T10: `CHANGELOG.md` に v55.3.0 エントリ追加
- [x] T11: `versions/current.md` を v55.3.0 / 3211 tests に更新
- [x] T12: `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.3.0 実績を COMPLETE に更新（3211 tests 確認）
- [x] T13: `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.3.0 実績欄も COMPLETE に更新

---

## コードレビュー

- [x] コードレビュー実施（`/review code`）
- [x] 指摘事項対応
  - [HIGH] `checkpoint_hook` を `is_some()` パターンに変更（v55.7 の永続化拡張時の borrow 競合を構造的に回避）
  - [HIGH] `checkpoint_delivery` 未注入: 仕様通り（spec.md で v55.7 注入と明示）→ 対応不要
  - [HIGH] `exactly_once_no_duplicate_on_restart` テスト名乖離: コメント追加で意図を明確化（TOML パース検証 + v55.7 で VM レベルテスト追加予定）

---

## 完了確認

- [x] `exactly_once_checkpoint_saved` pass
- [x] `exactly_once_no_duplicate_on_restart` pass
- [x] 3211 tests passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン（`is_duplicate_offset` / `checkpoint_delivery` の dead_code 警告なし）
- [x] `CHANGELOG.md` に v55.3.0 エントリが追加されている
- [x] `versions/current.md` が v55.3.0 を反映
- [x] T12 / T13 のロードマップ更新が完了している
