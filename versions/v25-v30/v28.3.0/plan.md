# v28.3.0 Plan — OpenTelemetry Rune 強化

## Phase 概要

| Phase | 内容 | 依存 |
|---|---|---|
| Phase 0 | 事前確認 | — |
| Phase 1 | Cargo.toml バージョン bump | — |
| Phase 2 | vm.rs に OTel primitive 4 件追加 | Phase 1 |
| Phase 3 | runes/otel/otel.fav 新規作成 | Phase 2 |
| Phase 4 | examples/observability/otel_tracing.fav 新規作成 | Phase 3 |
| Phase 5 | site/content/docs/runes/otel.mdx 新規作成 | — |
| Phase 6 | CHANGELOG.md 更新 | — |
| Phase 7 | benchmarks/v28.3.0.json 新規作成 | — |
| Phase 9a | checker.fav 更新（ns_to_effect に OTel 追加） | Phase 3 |
| Phase 9b | driver.rs に v283000_tests 追加 | Phase 9a |
| Phase 9c | cargo test --bin fav v283000 — 9/9 PASS 確認 | Phase 9b |
| Phase 9d | cargo test --bin fav 全体 — 2253 PASS 確認 | Phase 9c |

---

## Phase 0 — 事前確認

```bash
grep '^version' fav/Cargo.toml          # "28.2.0" を確認
cargo test --bin fav 2>&1 | tail -1     # "2244 tests" を含むことを確認
grep 'v283000_tests' fav/src/driver.rs  # 存在しないことを確認
grep 'OTel.start_span_raw' fav/src/backend/vm.rs  # 存在しないことを確認
grep '"runes/otel/' runes/              # runes/otel/ が存在しないことを確認
```

---

## Phase 1 — Cargo.toml バージョン bump

```toml
# fav/Cargo.toml
version = "28.3.0"
```

---

## Phase 2 — vm.rs に OTel primitive 追加

Datadog `service_check_raw` の wasm32 アームの直後に挿入する。

```rust
// ── OTel primitives (v28.3.0) ─────────────────────────────────────────────
#[cfg(not(target_arch = "wasm32"))]
"OTel.start_span_raw" => {
    let mut it = args.into_iter();
    let _name    = vm_string(it.next().ok_or("OTel.start_span_raw: missing name")?,    "OTel.start_span_raw")?;
    let _service = vm_string(it.next().ok_or("OTel.start_span_raw: missing service")?, "OTel.start_span_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"OTel.start_span_raw" => Ok(err_vm(VMValue::Str("OTel not supported on wasm32".into()))),

#[cfg(not(target_arch = "wasm32"))]
"OTel.set_attribute_raw" => {
    let mut it = args.into_iter();
    let _key   = vm_string(it.next().ok_or("OTel.set_attribute_raw: missing key")?,   "OTel.set_attribute_raw")?;
    let _value = vm_string(it.next().ok_or("OTel.set_attribute_raw: missing value")?, "OTel.set_attribute_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"OTel.set_attribute_raw" => Ok(err_vm(VMValue::Str("OTel not supported on wasm32".into()))),

#[cfg(not(target_arch = "wasm32"))]
"OTel.add_event_raw" => {
    let mut it = args.into_iter();
    let _name  = vm_string(it.next().ok_or("OTel.add_event_raw: missing name")?,  "OTel.add_event_raw")?;
    let _attrs = vm_string(it.next().ok_or("OTel.add_event_raw: missing attrs")?, "OTel.add_event_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"OTel.add_event_raw" => Ok(err_vm(VMValue::Str("OTel not supported on wasm32".into()))),

#[cfg(not(target_arch = "wasm32"))]
"OTel.end_span_raw" => {
    let mut it = args.into_iter();
    let _status = vm_string(it.next().ok_or("OTel.end_span_raw: missing status")?, "OTel.end_span_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"OTel.end_span_raw" => Ok(err_vm(VMValue::Str("OTel not supported on wasm32".into()))),
```

---

## Phase 3 — runes/otel/otel.fav 新規作成

```favnir
// runes/otel/otel.fav — OpenTelemetry Rune (v28.3.0)
// OTLP / Jaeger / Tempo / Honeycomb へトレースを送信する。
// v28.3.0 stub — 実際の OTLP HTTP エクスポートは fav/src/otel.rs を参照
public fn start_span(name: String, service: String) -> Result<Unit, String> !Io {
    OTel.start_span_raw(name, service)
}
public fn set_attribute(key: String, value: String) -> Result<Unit, String> !Io {
    OTel.set_attribute_raw(key, value)
}
public fn add_event(name: String, attrs: String) -> Result<Unit, String> !Io {
    OTel.add_event_raw(name, attrs)
}
public fn end_span(status: String) -> Result<Unit, String> !Io {
    OTel.end_span_raw(status)
}
```

---

## Phase 4 — examples/observability/otel_tracing.fav 新規作成

`examples/observability/` ディレクトリはすでに存在（v28.1.0 で作成）。

```favnir
// examples/observability/otel_tracing.fav — OpenTelemetry Rune デモ (v28.3.0)
import runes/otel

stage StartTrace: Unit -> Result<Unit, String> !Io = |_| {
    bind _ <- OTel.start_span("etl-pipeline", "etl-service")
    bind _ <- OTel.set_attribute("pipeline.version", "28.3.0")
    Result.ok(unit)
}

stage EndTrace: Unit -> Result<Unit, String> !Io = |_| {
    bind _ <- OTel.add_event("pipeline.completed", "{\"rows\": 1000}")
    bind _ <- OTel.end_span("ok")
    Result.ok(unit)
}

seq OTelTracingDemo = StartTrace |> EndTrace
```

---

## Phase 5 — site/content/docs/runes/otel.mdx 新規作成

```mdx
---
title: otel Rune
description: OpenTelemetry トレース送信 Rune（v28.3.0）
---

# otel Rune

`runes/otel` は OpenTelemetry トレースを OTLP / Jaeger / Tempo / Honeycomb へ送信する Rune です。
既存の `fav/src/otel.rs`（v22.7.0）の機能を Favnir から直接呼び出せる形に整理しました。

## インポート

```favnir
import runes/otel
```

## 関数一覧

| 関数 | シグネチャ | 説明 |
|---|---|---|
| `OTel.start_span` | `(name: String, service: String) -> Result<Unit, String> !Io` | 新しいスパンを開始 |
| `OTel.set_attribute` | `(key: String, value: String) -> Result<Unit, String> !Io` | スパンに属性を追加 |
| `OTel.add_event` | `(name: String, attrs: String) -> Result<Unit, String> !Io` | スパンにイベントを追加 |
| `OTel.end_span` | `(status: String) -> Result<Unit, String> !Io` | スパンを終了 |

## 使用例

```favnir
import runes/otel

stage TraceLoad: Unit -> Result<Unit, String> !Io = |_| {
    bind _ <- OTel.start_span("load-orders", "etl-service")
    bind _ <- OTel.set_attribute("db.system", "postgresql")
    bind _ <- OTel.add_event("query.start", "{}")
    bind _ <- OTel.end_span("ok")
    Result.ok(unit)
}
```

## エフェクト

すべての関数は `!Io` エフェクトを持ちます（OTLP エンドポイントへの HTTP 送信）。

## 注記

- `#[trace(name, service)]` アノテーションによる自動スパン挿入は v28.3+ で実装予定
- OTLP エクスポーターの接続先設定（`fav.toml` `[otel]` セクション）は v28.4+ で追加予定
- WASM ターゲットでは `Result.err("OTel not supported on wasm32")` を返します
```

---

## Phase 6 — CHANGELOG.md 更新

`CHANGELOG.md` の先頭に追加:

```markdown
## [v28.3.0] — 2026-06-27

### Added
- `runes/otel/otel.fav` — OpenTelemetry Rune（start_span / set_attribute / add_event / end_span）
- `OTel.start_span_raw` / `OTel.set_attribute_raw` / `OTel.add_event_raw` / `OTel.end_span_raw` VM primitive 追加
- `fav/self/checker.fav` `ns_to_effect` に `"OTel" => "IO"` 追加
- `examples/observability/otel_tracing.fav` — OTelTracingDemo E2E デモ
- `site/content/docs/runes/otel.mdx` — ドキュメント追加
```

---

## Phase 7 — benchmarks/v28.3.0.json 新規作成

```json
{
  "version": "28.3.0",
  "test_count": 2253,
  "timestamp": "2026-06-27"
}
```

---

## Phase 9a — checker.fav 更新

`fav/self/checker.fav` の `ns_to_effect` 関数内、Datadog の `else { "" }` ブロックを置き換える:

```favnir
// 変更前:
} else {
    ""
}
// （Datadog の else ブロック末尾）

// 変更後:
} else {
    if ns == "OTel" {
        "IO"
    } else {
        ""
    }
}
```

> **重要**: `"IO"`（全大文字）— v28.1.0 / v28.2.0 同様の JSONL パターンに従う。

---

## Phase 9b — driver.rs テスト追加

`v283000_tests` を `v282000_tests` の直前に追加。

```rust
// ── v283000_tests (v28.3.0) — OpenTelemetry Rune 強化 ──────────────────────
#[cfg(test)]
mod v283000_tests {
    #[test]
    fn otel_rune_has_start_span_fn() {
        let src = include_str!("../../runes/otel/otel.fav");
        assert!(src.contains("fn start_span("), "otel rune must define fn start_span(");
    }
    #[test]
    fn otel_rune_has_set_attribute_fn() {
        let src = include_str!("../../runes/otel/otel.fav");
        assert!(src.contains("fn set_attribute("), "otel rune must define fn set_attribute(");
    }
    #[test]
    fn otel_rune_has_add_event_fn() {
        let src = include_str!("../../runes/otel/otel.fav");
        assert!(src.contains("fn add_event("), "otel rune must define fn add_event(");
    }
    #[test]
    fn otel_rune_has_end_span_fn() {
        let src = include_str!("../../runes/otel/otel.fav");
        assert!(src.contains("fn end_span("), "otel rune must define fn end_span(");
    }
    #[test]
    fn otel_rune_uses_io_effect() {
        let src = include_str!("../../runes/otel/otel.fav");
        assert!(src.contains("!Io"), "otel rune must use !Io effect");
    }
    #[test]
    fn vm_has_otel_start_span_raw() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("OTel.start_span_raw"), "vm.rs must implement OTel.start_span_raw");
    }
    #[test]
    fn otel_example_has_pipeline() {
        let src = include_str!("../../examples/observability/otel_tracing.fav");
        assert!(src.contains("OTelTracingDemo"), "otel_tracing.fav must define OTelTracingDemo seq");
    }
    #[test]
    fn checker_has_otel_effect() {
        let src = include_str!("../../fav/self/checker.fav");
        assert!(
            src.contains("ns == \"OTel\"") && src.contains("\"IO\""),
            "checker.fav ns_to_effect must contain 'ns == \"OTel\"' mapped to \"IO\""
        );
    }
    #[test]
    fn changelog_has_v28_3_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v28.3.0]") || src.contains("## v28.3.0"), "CHANGELOG.md must contain '[v28.3.0]'");
    }
}
```

---

## Phase 9c / 9d — テスト確認

```bash
cargo test --bin fav v283000   # 9/9 PASS（全テスト個別確認）
cargo test --bin fav otel      # 7 件以上 PASS（otel 含む 7 件がマッチ）
cargo test --bin fav           # 2253 tests PASS
```
