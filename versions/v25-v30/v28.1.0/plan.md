# v28.1.0 Plan — prometheus Rune 追加

## 実装戦略

新 Rune 追加バージョン（v27.8.0 dbt / v27.9.0 sqlite と同パターン）。
- VM primitive stub → Rune ファイル → checker.fav（Phase 9a） → テスト（Phase 9b）

## フェーズ構成

### Phase 1 — Cargo.toml バージョン bump
`28.0.0` → `28.1.0`

### Phase 2 — vm.rs に Prometheus primitive 追加
`fav/src/backend/vm.rs` の SQLite primitive 群の直後に追加:

```rust
// ── Prometheus primitives (v28.1.0) ───────────────────────────────────
#[cfg(not(target_arch = "wasm32"))]
"Prometheus.counter_raw" => {
    // (name: String, value: Float, labels: String) -> Result<Unit, String>
    let _name   = vm_string(...)?;
    let _value  = vm_float(...)?;
    let _labels = vm_string(...)?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Prometheus.counter_raw" => Ok(err_vm(VMValue::Str("Prometheus not supported on wasm32".into()))),
#[cfg(not(target_arch = "wasm32"))]
"Prometheus.gauge_raw" => {
    let _name  = vm_string(...)?;
    let _value = vm_float(...)?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Prometheus.gauge_raw" => Ok(err_vm(VMValue::Str("Prometheus not supported on wasm32".into()))),
#[cfg(not(target_arch = "wasm32"))]
"Prometheus.histogram_raw" => {
    let _name  = vm_string(...)?;
    let _value = vm_float(...)?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Prometheus.histogram_raw" => Ok(err_vm(VMValue::Str("Prometheus not supported on wasm32".into()))),
#[cfg(not(target_arch = "wasm32"))]
"Prometheus.push_raw" => {
    let _gateway_url = vm_string(...)?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Prometheus.push_raw" => Ok(err_vm(VMValue::Str("Prometheus not supported on wasm32".into()))),
```

### Phase 3 — runes/prometheus/prometheus.fav 作成
4 関数（counter / gauge / histogram / push）、エフェクト `!Io`。

### Phase 4 — examples/observability/prometheus_demo.fav 作成
`examples/observability/` ディレクトリを新規作成。
`PrometheusDemo` seq pipeline（2 stage）を実装。

### Phase 5 — site/content/docs/runes/prometheus.mdx 作成

### Phase 6 — CHANGELOG.md 更新
`[v28.1.0]` セクションを先頭に追加。

### Phase 7 — benchmarks/v28.1.0.json 作成

### Phase 8 — driver.rs テスト追加
`v281000_tests` モジュール（9 件）を `v280000_tests` の直前に追加。

### Phase 9a — checker.fav 更新（テスト前に必須）
`fav/self/checker.fav` の `ns_to_effect` に `"Prometheus" => "Io"` 追加。
SQLite の else ブロック内に挿入。

### Phase 9b — テスト実行
`cargo test --bin fav prometheus` で 7 件以上 PASS を確認（ロードマップ要件 4 件超過）。
`cargo test --bin fav v281000` で 9/9 PASS を確認。
`cargo test --bin fav` で 2235 tests PASS を確認。

### Phase 10 — tasks.md COMPLETE 更新

## 実装順序

Phase 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → Phase 9a → Phase 9b → 10

> NOTE: Phase 9a（checker.fav）は Phase 9b（テスト実行）より必ず先に行う。

## vm.rs 挿入位置

SQLite primitives（v27.9.0）の `wasm32` arm の直後に挿入:

```
"SQLite.close_raw" wasm32 arm
↓ ここに Prometheus primitives を挿入
Azure Blob primitives（既存）
```

## checker.fav 挿入位置

現在の ns_to_effect 末端:
```
... SQLite else block ...
  if ns == "SQLite" then "Db"
  else ""   ← ここを書き換え
```
↓
```
  if ns == "SQLite" then "Db"
  else if ns == "Prometheus" then "Io"
  else ""
```
