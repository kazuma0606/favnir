# v83.5.0 実装計画 — 分散トレーシング強化（`TraceContext`）

## 依存関係

`uuid` クレートは v9.4.0 で導入済み。`TraceContext::new_root()` / `child_span()` で使用する。

## 実装ステップ

### Step 1: `test_framework.rs` に構造体と impl を追加

v83.4.0 追加ブロック（`format_cost_report` 末尾）の後に追加する。

1. `TraceContext` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `trace_id: String`, `span_id: String`, `parent_span_id: Option<String>`

2. `impl TraceContext` ブロック
   - `new_root() -> TraceContext`:
     ```rust
     let trace_uuid = uuid::Uuid::new_v4();
     let span_uuid = uuid::Uuid::new_v4();
     let trace_id = trace_uuid.simple().to_string();
     let span_str = span_uuid.simple().to_string();
     let span_id = span_str[16..].to_string();
     TraceContext { trace_id, span_id, parent_span_id: None }
     ```
   - `child_span(parent: &TraceContext) -> TraceContext`（`self` を取らない関連付け関数、`TraceContext::child_span(&root)` で呼び出す）:
     ```rust
     let span_uuid = uuid::Uuid::new_v4();
     let span_str = span_uuid.simple().to_string();
     let span_id = span_str[16..].to_string();
     TraceContext {
         trace_id: parent.trace_id.clone(),
         span_id,
         parent_span_id: Some(parent.span_id.clone()),
     }
     ```

3. `TraceSpan` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `context: TraceContext`, `name: String`, `start_ms: u64`, `end_ms: u64`,
     `attributes: Vec<(String, String)>`

### Step 2: `format_trace_span` / `compute_span_duration` 関数を追加

```
fn format_trace_span(span: &TraceSpan) -> String
```

出力形式:
```
Span: {name}
Trace: {trace_id}
Span ID: {span_id}
Duration: {duration}ms
Attributes: {key=value, ...}  （attributes が空の場合は行ごと省略）
```
- `attributes` が空の場合は "Attributes:" 行を出力しない
- 複数 attribute は `", "` で結合: `"k1=v1, k2=v2"`

```
fn compute_span_duration(span: &TraceSpan) -> u64
```
- `span.end_ms.saturating_sub(span.start_ms)`（アンダーフローガード）

### Step 3: `driver.rs` に `v83500_tests` を追加

`v83400_tests` の直後に追加する。

```rust
#[cfg(test)]
mod v83500_tests {
    use fav_core::test_framework::*;

    #[test]
    fn trace_context_child_span_created() { ... }

    #[test]
    fn span_duration_computed() { ... }
}
```

### Step 4: `cargo test` で全テスト通過を確認

期待: 3897 tests pass、0 failures

### Step 5: CI チェック

- `cargo clippy --locked -- -D warnings` が pass することを確認
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認
