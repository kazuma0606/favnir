# v28.2.0 Plan — datadog Rune 追加

## 実装戦略

新 Rune 追加バージョン（v28.1.0 prometheus と同パターン）。
- VM primitive stub → Rune ファイル → checker.fav（Phase 9a） → テスト（Phase 9b）

## フェーズ構成

### Phase 1 — Cargo.toml バージョン bump
`28.1.0` → `28.2.0`

### Phase 2 — vm.rs に Datadog primitive 追加
Prometheus primitives ブロックの直後に挿入:

```rust
// ── Datadog primitives (v28.2.0) ──────────────────────────────────────
#[cfg(not(target_arch = "wasm32"))]
"Datadog.metric_raw" => {
    // (name: String, value: Float, tags: String) -> Result<Unit, String>
    let mut it = args.into_iter();
    let _name  = vm_string(it.next().ok_or("Datadog.metric_raw: missing name")?,  "Datadog.metric_raw")?;
    let _value = vm_float( it.next().ok_or("Datadog.metric_raw: missing value")?, "Datadog.metric_raw")?;
    let _tags  = vm_string(it.next().ok_or("Datadog.metric_raw: missing tags")?,  "Datadog.metric_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Datadog.metric_raw" => Ok(err_vm(VMValue::Str("Datadog not supported on wasm32".into()))),
#[cfg(not(target_arch = "wasm32"))]
"Datadog.log_raw" => {
    // (level: String, message: String, attrs: String) -> Result<Unit, String>
    let mut it = args.into_iter();
    let _level   = vm_string(it.next().ok_or("Datadog.log_raw: missing level")?,   "Datadog.log_raw")?;
    let _message = vm_string(it.next().ok_or("Datadog.log_raw: missing message")?, "Datadog.log_raw")?;
    let _attrs   = vm_string(it.next().ok_or("Datadog.log_raw: missing attrs")?,   "Datadog.log_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Datadog.log_raw" => Ok(err_vm(VMValue::Str("Datadog not supported on wasm32".into()))),
#[cfg(not(target_arch = "wasm32"))]
"Datadog.trace_raw" => {
    // (name: String, fn_body: String) -> Result<Unit, String>
    let mut it = args.into_iter();
    let _name    = vm_string(it.next().ok_or("Datadog.trace_raw: missing name")?,    "Datadog.trace_raw")?;
    let _fn_body = vm_string(it.next().ok_or("Datadog.trace_raw: missing fn_body")?, "Datadog.trace_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Datadog.trace_raw" => Ok(err_vm(VMValue::Str("Datadog not supported on wasm32".into()))),
#[cfg(not(target_arch = "wasm32"))]
"Datadog.event_raw" => {
    // (title: String, text: String, tags: String) -> Result<Unit, String>
    let mut it = args.into_iter();
    let _title = vm_string(it.next().ok_or("Datadog.event_raw: missing title")?, "Datadog.event_raw")?;
    let _text  = vm_string(it.next().ok_or("Datadog.event_raw: missing text")?,  "Datadog.event_raw")?;
    let _tags  = vm_string(it.next().ok_or("Datadog.event_raw: missing tags")?,  "Datadog.event_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Datadog.event_raw" => Ok(err_vm(VMValue::Str("Datadog not supported on wasm32".into()))),
#[cfg(not(target_arch = "wasm32"))]
"Datadog.service_check_raw" => {
    // (name: String, status: String) -> Result<Unit, String>
    let mut it = args.into_iter();
    let _name   = vm_string(it.next().ok_or("Datadog.service_check_raw: missing name")?,   "Datadog.service_check_raw")?;
    let _status = vm_string(it.next().ok_or("Datadog.service_check_raw: missing status")?, "Datadog.service_check_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Datadog.service_check_raw" => Ok(err_vm(VMValue::Str("Datadog not supported on wasm32".into()))),
```

### Phase 3 — runes/datadog/datadog.fav 作成
5 関数（metric / log / trace / event / service_check）、エフェクト `!Io`。

### Phase 4 — examples/observability/datadog_apm.fav 作成
`examples/observability/` ディレクトリは v28.1.0 で作成済み（prometheus_demo.fav が存在）。
`DatadogApmDemo` seq pipeline（2 stage）を実装。

### Phase 5 — site/content/docs/runes/datadog.mdx 作成

### Phase 6 — CHANGELOG.md 更新

### Phase 7 — benchmarks/v28.2.0.json 作成

### Phase 8 — driver.rs テスト追加
`v282000_tests` モジュール（9 件）を `v281000_tests` の直前に追加。

### Phase 9a — checker.fav 更新（テスト前に必須）
`fav/self/checker.fav` の `ns_to_effect` に `"Datadog" => "IO"` 追加。
Prometheus の else ブロック内に挿入:

```
// 変更前:
if ns == "Prometheus" {
    "IO"
} else {
    ""
}

// 変更後:
if ns == "Prometheus" {
    "IO"
} else {
    if ns == "Datadog" {
        "IO"
    } else {
        ""
    }
}
```

### Phase 9b — テスト実行
`cargo test --bin fav datadog` — 8 件以上 PASS 確認（`changelog_has_v28_2_0` は v282000 フィルタでのみヒット）。
`cargo test --bin fav v282000` — 9/9 PASS 確認。
`cargo test --bin fav` — 2244 tests PASS 確認。

### Phase 10 — tasks.md COMPLETE 更新

## 実装順序

Phase 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → Phase 9a → Phase 9b → 10

> NOTE: Phase 9a（checker.fav）は Phase 9b（テスト実行）より必ず先に行う。
> NOTE: `"IO"`（全大文字）を使用すること。v28.1.0 で `"Io"` バグを修正済み。
