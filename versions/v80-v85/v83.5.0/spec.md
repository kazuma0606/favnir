# v83.5.0 仕様書 — 分散トレーシング強化（OpenTelemetry `TraceContext`）

## Background

v83.4.0 でコスト追跡が導入された。次のステップとして、
v29.0「Observability First」の OTel 統合を強化し、
スパン伝播（W3C Trace Context 形式）を型で扱えるようにする。

本バージョンは Observability 2.0 スプリント（v83.1〜v84.0）の第 5 段階。

ロードマップ参照: `versions/roadmap/roadmap-v83.1-v84.0.md` — v83.5.0 セクション

## Goals

1. `TraceContext` 構造体を追加する（trace_id / span_id / parent_span_id）
2. `TraceSpan` 構造体を追加する（context / name / start_ms / end_ms / attributes）
3. `TraceContext::new_root() -> TraceContext` を追加する
4. `TraceContext::child_span(parent: &TraceContext) -> TraceContext` を追加する
5. `format_trace_span(span: &TraceSpan) -> String` を追加する
6. `compute_span_duration(span: &TraceSpan) -> u64` を追加する

## 型定義・API

```rust
/// OTel W3C Trace Context 形式のコンテキスト。
#[derive(Debug, Clone, PartialEq)]
pub struct TraceContext {
    pub trace_id: String,           // 32 桁 hex（W3C traceparent の trace-id）
    pub span_id: String,            // 16 桁 hex（W3C traceparent の parent-id）
    pub parent_span_id: Option<String>,  // ルートスパンは None
}

impl TraceContext {
    /// 新しいルートコンテキストを作成する。
    /// `trace_id` と `span_id` は uuid::Uuid::new_v4() で生成し
    /// `parent_span_id = None`。
    pub fn new_root() -> TraceContext

    /// 子スパンのコンテキストを作成する（`self` を取らない関連付け関数）。
    /// `TraceContext::child_span(&root)` の形で呼び出す。
    /// 親と同じ `trace_id` を引き継ぎ、新しい `span_id` を生成し、
    /// `parent_span_id = Some(parent.span_id.clone())`。
    pub fn child_span(parent: &TraceContext) -> TraceContext
}

/// 単一スパンの実行記録。
#[derive(Debug, Clone, PartialEq)]
pub struct TraceSpan {
    pub context: TraceContext,
    pub name: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub attributes: Vec<(String, String)>,
}

/// スパン情報のテキストを返す。
///
/// 例:
/// "Span: load_stage\nTrace: <trace_id>\nSpan ID: <span_id>\nDuration: 150ms\nAttributes: env=prod"
pub fn format_trace_span(span: &TraceSpan) -> String

/// スパンの実行時間（ミリ秒）を返す。
/// `end_ms >= start_ms` を前提とし、差分を返す（`end_ms - start_ms`）。
pub fn compute_span_duration(span: &TraceSpan) -> u64
```

## ID 生成

- `TraceContext::new_root()` と `TraceContext::child_span()` は `uuid::Uuid::new_v4()` を使用する
- `trace_id` は UUID を小文字 hex 32 桁（ハイフンなし）で表現: `uuid.simple().to_string()`
- `span_id` は UUID の下位 16 桁を使用（変数束縛してからスライス）:
  ```rust
  let s = uuid.simple().to_string();
  let span_id = s[16..].to_string();
  ```

## テスト（v83.5.0 で追加）

実際のテスト数ベース（※ drift 補正後）: **3895 + 2 = 3897**

（ロードマップ記載値 3883 + 2 = 3885 は旧バージョン到達時点のドリフト。
 実際の v83.4.0 完了テスト数は 3895。）

### `trace_context_child_span_created`

```rust
let root = TraceContext::new_root();
assert!(root.parent_span_id.is_none(), "root should have no parent");
assert_eq!(root.trace_id.len(), 32, "trace_id should be 32 hex chars");
assert_eq!(root.span_id.len(), 16, "span_id should be 16 hex chars");

let child = TraceContext::child_span(&root);
assert_eq!(child.trace_id, root.trace_id, "child should share trace_id");
assert_ne!(child.span_id, root.span_id, "child should have different span_id");
assert_eq!(
    child.parent_span_id.as_deref(),
    Some(root.span_id.as_str()),
    "child parent_span_id should match root span_id"
);
```

### `span_duration_computed`

```rust
let root = TraceContext::new_root();
let span = TraceSpan {
    context: root,
    name: "load_stage".into(),
    start_ms: 1000,
    end_ms: 1150,
    attributes: vec![("env".into(), "prod".into())],
};
let duration = compute_span_duration(&span);
assert_eq!(duration, 150, "duration should be 150ms");
let fmt = format_trace_span(&span);
assert!(fmt.contains("Span:"), "format should contain 'Span:'");
assert!(fmt.contains("Duration:"), "format should contain 'Duration:'");
```

## Success Criteria

- `cargo test` が 3897 tests pass（+2）、0 failures
- `child_span` が親の `trace_id` を引き継ぎ、異なる `span_id` を持つことをテストで確認
- `compute_span_duration` が `end_ms - start_ms` を正しく返す

## Files to Modify

- `fav/src/test_framework.rs` — 型定義・impl・関数追加
- `fav/src/driver.rs` — `v83500_tests` モジュール追加
- `CHANGELOG.md` — v83.5.0 エントリ追加
