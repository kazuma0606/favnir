# v28.2.0 Spec — datadog Rune 追加

## 概要

APM・ログ・メトリクスの統合プラットフォームである Datadog の Rune を追加する。
`metric / log / trace / event / service_check` の 5 関数をスタブ実装し、
`!Io` エフェクトで型安全にメトリクス・ログ・トレースを送信できるようにする。

## ロードマップ参照

`versions/roadmap/roadmap-v28.1-v29.0.md` — v28.2 セクション

## 実装内容

### T0 — Cargo.toml バージョン bump
`28.1.0` → `28.2.0`

### T1 — VM primitive 追加（vm.rs）
以下の 5 primitive を `fav/src/backend/vm.rs` に追加（`#[cfg]` ガード付き）。
Prometheus primitives の直後に挿入。

| primitive | 引数 | 非 WASM 戻り値 | WASM 戻り値 |
|---|---|---|---|
| `Datadog.metric_raw` | (name: String, value: Float, tags: String) | `ok_vm(Unit)` | `err_vm("Datadog not supported on wasm32")` |
| `Datadog.log_raw` | (level: String, message: String, attrs: String) | `ok_vm(Unit)` | `err_vm("Datadog not supported on wasm32")` |
| `Datadog.trace_raw` | (name: String, fn_body: String) | `ok_vm(Unit)` | `err_vm("Datadog not supported on wasm32")` |
| `Datadog.event_raw` | (title: String, text: String, tags: String) | `ok_vm(Unit)` | `err_vm("Datadog not supported on wasm32")` |
| `Datadog.service_check_raw` | (name: String, status: String) | `ok_vm(Unit)` | `err_vm("Datadog not supported on wasm32")` |

> `vm_has_datadog_metric_raw` テスト 1 件で 5 primitive の実装を代表確認する
>（全 primitive が同一ブロック内に存在するため個別テストは冗長）。

### T2 — Rune ファイル作成（runes/datadog/datadog.fav）

```favnir
// runes/datadog/datadog.fav — Datadog APM/Metrics/Logs Rune (v28.2.0)
// APM・ログ・メトリクスを DogStatsD / Datadog API 経由で送信する。
// v28.2.0 stub — 実際の HTTP 送信は v28.x 以降
public fn metric(name: String, value: Float, tags: String) -> Result<Unit, String> !Io {
    Datadog.metric_raw(name, value, tags)
}
public fn log(level: String, message: String, attrs: String) -> Result<Unit, String> !Io {
    Datadog.log_raw(level, message, attrs)
}
public fn trace(name: String, fn_body: String) -> Result<Unit, String> !Io {
    Datadog.trace_raw(name, fn_body)
}
public fn event(title: String, text: String, tags: String) -> Result<Unit, String> !Io {
    Datadog.event_raw(title, text, tags)
}
public fn service_check(name: String, status: String) -> Result<Unit, String> !Io {
    Datadog.service_check_raw(name, status)
}
```

### T3 — checker.fav 更新（Phase 9a）
`fav/self/checker.fav` の `ns_to_effect` に `"Datadog" => "IO"` を追加。
Prometheus else ブロックの内側（最も深いネスト）に追加。

> **重要**: v28.1.0 の教訓から `"IO"`（全大文字）を使用すること（`"Io"` は誤り）。

### T4 — example ファイル作成
`examples/observability/datadog_apm.fav` — APM トレース + メトリクス送信デモ:
- `stage TraceExtract: Unit -> Result<Unit, String> !Io` — トレース開始
- `stage ReportToDatadog: Unit -> Result<Unit, String> !Io` — メトリクス + ログ送信
- `seq DatadogApmDemo = TraceExtract |> ReportToDatadog`

> **スコープ注記**: `#[trace]` アノテーション（ロードマップ v28.2 記載、v28.8 の E2E デモが前提）は
> ast.rs / parser.rs / コンパイラへの複数ファイル変更が必要なため v28.3+ で実装予定。
> v28.8 ロードマップの `datadog_apm.fav`（`#[trace]` 使用版）は v28.2 時点では
> アノテーションなしスタブとして作成し、v28.8 で `#[trace]` 付きに置き換える。

### T5 — サイトドキュメント
`site/content/docs/runes/datadog.mdx` 新規作成。

### T6 — CHANGELOG 更新
`CHANGELOG.md` に `[v28.2.0]` セクション追加。

### T7 — ベンチマーク
`benchmarks/v28.2.0.json` 新規作成（test_count: 2244）。

### T8 — driver.rs テスト（Phase 9b）
`v282000_tests` モジュール（9 件）を `driver.rs` に追加。

### T9 — テスト全通過確認
`cargo test --bin fav` で 2244 tests PASS。

## エフェクト設計

| Rune 関数 | エフェクト | 理由 |
|---|---|---|
| metric / log / trace / event / service_check | `!Io` | DogStatsD / Datadog API への HTTP 送信（ネットワーク I/O） |

## テスト数

- v28.1.0: 2235 tests
- v28.2.0: **2244 tests**（+9）

## 完了条件

- [ ] `Cargo.toml` version = "28.2.0"
- [ ] `runes/datadog/datadog.fav` 存在（5 関数、`!Io` エフェクト）
- [ ] `Datadog.*_raw` 5 VM primitive 存在（`#[cfg]` ガード付き）
- [ ] `fav/self/checker.fav` `ns_to_effect` に `ns == "Datadog"` → `"IO"` あり
- [ ] `examples/observability/datadog_apm.fav` に `DatadogApmDemo` seq あり
- [ ] `site/content/docs/runes/datadog.mdx` 存在
- [ ] `CHANGELOG.md` に `[v28.2.0]` または `## v28.2.0` セクションあり
- [ ] `benchmarks/v28.2.0.json` 存在（test_count: 2244）
- [ ] `cargo test --bin fav datadog` — 8 件以上 PASS（`changelog_has_v28_2_0` は `v282000` フィルタでのみヒット）
- [ ] `cargo test --bin fav v282000` — 9/9 PASS
- [ ] `cargo test --bin fav` — 2244 tests PASS
