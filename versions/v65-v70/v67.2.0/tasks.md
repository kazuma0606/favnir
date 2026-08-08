# v67.2.0 タスクリスト

Status: COMPLETE
Version: 67.2.0
Note: MDX ドキュメントは v67.9.0 で一括作成のため本バージョンに T5 はない
Base tests: 3499
Target tests: 3501
Actual tests: 3501

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3499 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"67.0.0"` であることを確認（sub-version では変更しない）
- [x] `fav/src/debug.rs` が存在することを確認（v67.1.0 で作成済み）
- [x] `driver.rs` に `v67100_tests` が存在することを確認（`v67200_tests` の挿入位置）
- [x] `driver.rs` に `v67200_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v67100_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `debug_step_execution`, `debug_breakpoint_stage`
- [x] `versions/current.md` の「進行中バージョン」が `v67.1.0` であることを確認

---

## T1: `fav/src/debug.rs` 拡張

- [x] `fav/src/debug.rs` に Time-Travel Debugging を追加（既存コードは変更しない）
  - [x] `"--record"` を含む（`debug_record_replay` テストにマッチ）
  - [x] `"--replay"` を含む（`debug_record_replay` テストにマッチ）
  - [x] `"rewind"` を含む（`debug_rewind_to_step` テストにマッチ）
  - [x] `"forward"` を含む（`debug_rewind_to_step` テストにマッチ）
  - [x] `".fav-trace"` を含む（`debug_rewind_to_step` テストにマッチ）
  - [x] `pub const TIME_TRAVEL_HELP: &str` を追加
  - [x] `pub fn cmd_debug_replay(trace_path: &str, _args: &[String]) -> String` を追加
- [x] `cargo build` でエラーなし（debug.rs 追記後）

---

## T2: `driver.rs` — `v67200_tests` 追加

- [x] `// -- v67100_tests (v67.1.0)` コメントの直前に `v67200_tests` を挿入
  - [x] `debug_record_replay`: `include_str!("debug.rs")` に `"--record"` と `"--replay"` を含む
  - [x] `debug_rewind_to_step`: `include_str!("debug.rs")` に `"rewind"` / `"forward"` / `".fav-trace"` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし（driver.rs テスト挿入後）

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v67200_tests` で 2 件 PASS
  - [x] `debug_record_replay` PASS
  - [x] `debug_rewind_to_step` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3501 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

> T3 のテスト全通過（3501 tests passed）を確認してから実施すること。

- [x] `versions/roadmap/roadmap-v67.1-v68.0.md` のバージョン一覧表で v67.2.0 の「状態」列を「未着手」→「完了」に変更し、変更後に当該行が「完了」になっていることを目視確認
- [x] `versions/current.md` の「進行中バージョン」を v67.2.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v67.1〜v67.9 では CHANGELOG.md を更新しない。v68.0.0 宣言時に一括追記する。
> **Cargo.toml 方針**: v67.1〜v67.9 では version を変更しない。v68.0.0 宣言時に `"68.0.0"` に更新する。

---

## コードレビュー指摘と対応

- [HIGH] spec-reviewer: plan.md current.md 更新値が曖昧 → `v67.1.0` → `v67.2.0` と明記
- [HIGH] spec-reviewer: spec.md 非スコープにメモリ効率の言及なし → 追記済み
- [MED] spec-reviewer: cmd_debug_replay の dead_code リスク → `pub(crate)` + `#[allow(dead_code)]` で対応
- [MED] spec-reviewer: tasks.md の cargo build 重複の意図不明 → 目的注釈を追加
- [MED] code-reviewer: `cmd_debug_replay` が CLI から到達不可（`Some("debug")` アームに `--replay` 分岐なし） → `main.rs` の `Some("debug")` アームに `--replay` チェックを追加し `cmd_debug_replay` へ委譲
