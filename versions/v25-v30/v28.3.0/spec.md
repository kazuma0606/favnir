# v28.3.0 Spec — OpenTelemetry Rune 強化

## 概要

既存の `fav/src/otel.rs`（v22.7.0 で追加）を参照しつつ、
Favnir コードから `OTel.start_span / set_attribute / add_event / end_span` を
型安全に呼び出せる Rune を追加する。
エフェクトは `!Io`（OTLP エクスポーター経由でトレースバックエンドに HTTP 送信）。

> **`otel.rs` との関係**: v28.3.0 では VM primitive はスタブ（`Ok(unit)` 返し）として実装する。
> `otel.rs` の `OTEL_SPANS` thread-local / `otel_export_http` 等との接続は v28.x 以降に実装予定。
> ロードマップ v28.3 の `ManualTrace` 擬似コードは span オブジェクトを返す設計を示しているが、
> v28.3.0 では span ID を返さないスタブとして実装する（span ID 追跡は v28.x 以降）。

## ロードマップ参照

`versions/roadmap/roadmap-v28.1-v29.0.md` — v28.3 セクション

## 実装内容

### T1 — VM primitive 追加（vm.rs）

以下の 4 primitive を `fav/src/backend/vm.rs` に追加（`#[cfg]` ガード付き）。
Datadog primitives の直後に挿入。

| primitive | 引数 | 非 WASM 戻り値 | WASM 戻り値 |
|---|---|---|---|
| `OTel.start_span_raw` | (name: String, service: String) | `ok_vm(Unit)` | `err_vm("OTel not supported on wasm32")` |
| `OTel.set_attribute_raw` | (key: String, value: String) | `ok_vm(Unit)` | `err_vm("OTel not supported on wasm32")` |
| `OTel.add_event_raw` | (name: String, attrs: String) | `ok_vm(Unit)` | `err_vm("OTel not supported on wasm32")` |
| `OTel.end_span_raw` | (status: String) | `ok_vm(Unit)` | `err_vm("OTel not supported on wasm32")` |

> `vm_has_otel_start_span_raw` テスト 1 件で 4 primitive の実装を代表確認する。

### T2 — Rune ファイル作成（runes/otel/otel.fav）

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

### T3 — checker.fav 更新（Phase 9a）

`fav/self/checker.fav` の `ns_to_effect` に `"OTel" => "IO"` を追加。
Datadog else ブロックの内側（最も深いネスト）に追加。

> **重要**: v28.1.0 / v28.2.0 の教訓から `"IO"`（全大文字）を使用すること（`"Io"` は誤り）。

### T4 — example ファイル作成

`examples/observability/otel_tracing.fav` — OTel トレース送信デモ:
- `stage StartTrace: Unit -> Result<Unit, String> !Io` — スパン開始
- `stage EndTrace: Unit -> Result<Unit, String> !Io` — 属性設定 + イベント + スパン終了
- `seq OTelTracingDemo = StartTrace |> EndTrace`

> **スコープ注記**: `#[trace(name, service)]` アノテーション（ロードマップ v28.3 記載）は
> ast.rs / parser.rs / コンパイラへの複数ファイル変更が必要なため v28.3+ で実装予定。
> v28.3.0 ではアノテーションなしスタブとして OTel Rune 関数の公開のみを行う。
> `fav.toml` への OTLP エクスポーター設定（`[otel]` セクション）も同様に v28.4+ で追加予定。

### T5 — サイトドキュメント

`site/content/docs/runes/otel.mdx` 新規作成。

### T6 — CHANGELOG 更新

`CHANGELOG.md` に `[v28.3.0]` セクション追加。

### T7 — ベンチマーク

`benchmarks/v28.3.0.json` 新規作成（test_count: 2253）。

### T8 — driver.rs テスト（Phase 9b）

`v283000_tests` モジュール（9 件）を `driver.rs` に追加。

### T9 — テスト全通過確認

`cargo test --bin fav` で 2253 tests PASS。

## エフェクト設計

| Rune 関数 | エフェクト | 理由 |
|---|---|---|
| start_span / set_attribute / add_event / end_span | `!Io` | OTLP エクスポーター経由でトレースバックエンドに HTTP 送信（ネットワーク I/O） |

## テスト数

- v28.2.0: 2244 tests
- v28.3.0: **2253 tests**（+9）

## 完了条件

- [ ] `Cargo.toml` version = "28.3.0"
- [ ] `runes/otel/otel.fav` 存在（4 関数、`!Io` エフェクト）
- [ ] `OTel.*_raw` 4 VM primitive 存在（`#[cfg]` ガード付き）
- [ ] `fav/self/checker.fav` `ns_to_effect` に `ns == "OTel"` → `"IO"` あり
- [ ] `examples/observability/otel_tracing.fav` に `OTelTracingDemo` seq あり
- [ ] `site/content/docs/runes/otel.mdx` 存在
- [ ] `CHANGELOG.md` に `[v28.3.0]` または `## v28.3.0` セクションあり
- [ ] `benchmarks/v28.3.0.json` 存在（test_count: 2253）
- [ ] `cargo test --bin fav otel` — 7 件以上 PASS（`v283000_tests` のうち `otel` 含む 7 件がマッチ。`checker_has_otel_effect` / `changelog_has_v28_3_0` は `v283000` フィルタでのみヒット）
- [ ] `cargo test --bin fav v283000` — 9/9 PASS
- [ ] `cargo test --bin fav` — 2253 tests PASS
