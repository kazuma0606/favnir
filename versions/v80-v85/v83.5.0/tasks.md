# v83.5.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,895 tests pass、0 failures であることを確認する（前提: v83.4.0 完了済み）

## T1: `test_framework.rs` に構造体と impl を追加

- [x] `TraceContext` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `trace_id: String`, `span_id: String`, `parent_span_id: Option<String>`
- [x] `impl TraceContext` ブロックを追加する
  - `new_root() -> TraceContext`（uuid::Uuid::new_v4() で trace_id/span_id 生成、parent_span_id = None）
  - `child_span(parent: &TraceContext) -> TraceContext`（trace_id 引き継ぎ、新 span_id、parent_span_id = Some(parent.span_id)）
- [x] `TraceSpan` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `context: TraceContext`, `name: String`, `start_ms: u64`, `end_ms: u64`, `attributes: Vec<(String, String)>`

## T2: `format_trace_span` / `compute_span_duration` 関数を追加

- [x] `format_trace_span(span: &TraceSpan) -> String` を追加する
  - "Span:"、"Trace:"、"Span ID:"、"Duration:" を含む
  - `attributes` が空の場合は "Attributes:" 行を省略
- [x] `compute_span_duration(span: &TraceSpan) -> u64` を追加する
  - `end_ms.saturating_sub(start_ms)`

## T3: `driver.rs` に `v83500_tests` を追加

- [x] `v83400_tests` の直後に `#[cfg(test)] mod v83500_tests` を追加する
  - `trace_context_child_span_created`
  - `span_duration_computed`（`format_trace_span` スモークテスト含む）

## T4: `CHANGELOG.md` 更新

- [x] `CHANGELOG.md` の先頭に v83.5.0 エントリを追加する（`TraceContext` / `TraceSpan` 追加内容を記載）

## T5: テスト通過確認

- [x] `cargo test` が 3,897 tests pass（+2）、0 failures であることを確認する

## T6: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 対応

- [x] [MED] `span_str[16..]` を `span_str[16..32]`（上限明示）に変更し、「常に 32 文字の ASCII hex」である旨のコメントを追加
- [LOW] `child_span` associated function の設計: 呼び出し側テストが `TraceContext::child_span(&root)` 形式で正しく動作しており対応不要
- [LOW] `format_trace_span` の attributes エスケープ: 内部テスト用途のため対応不要
