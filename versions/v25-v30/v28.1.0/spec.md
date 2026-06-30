# v28.1.0 Spec — prometheus Rune 追加

## 概要

メトリクス収集の標準である Prometheus の Rune を追加する。
`counter / gauge / histogram / push` の 4 関数をスタブ実装し、
`!Io` エフェクトで型安全にメトリクスを送信できるようにする。

> **スコープ注記**: `#[track]` アノテーション（ロードマップ v28.1 記載）は
> ast.rs / parser.rs / コンパイラへの複数ファイル変更が必要なため、
> v28.2+ の独立バージョンとして実装する。v28.1.0 の対象外。

## ロードマップ参照

`versions/roadmap/roadmap-v28.1-v29.0.md` — v28.1 セクション

## 実装内容

### T0 — Cargo.toml バージョン bump
`28.0.0` → `28.1.0`

### T1 — VM primitive 追加（vm.rs）
以下の 4 primitive を `fav/src/backend/vm.rs` に追加（`#[cfg]` ガード付き）:

| primitive | シグネチャ（引数） | 非 WASM 戻り値 | WASM 戻り値 |
|---|---|---|---|
| `Prometheus.counter_raw` | (name: String, value: Float, labels: String) | `ok_vm(Unit)` | `err_vm("Prometheus not supported on wasm32")` |
| `Prometheus.gauge_raw` | (name: String, value: Float) | `ok_vm(Unit)` | `err_vm("Prometheus not supported on wasm32")` |
| `Prometheus.histogram_raw` | (name: String, value: Float) | `ok_vm(Unit)` | `err_vm("Prometheus not supported on wasm32")` |
| `Prometheus.push_raw` | (gateway_url: String) | `ok_vm(Unit)` | `err_vm("Prometheus not supported on wasm32")` |

> `vm_has_prometheus_counter_raw` テスト 1 件で 4 primitive の実装を代表確認する
>（全 primitive が同一ブロック内に存在するため個別テストは冗長）。

### T2 — Rune ファイル作成（runes/prometheus/prometheus.fav）

```favnir
// runes/prometheus/prometheus.fav — Prometheus メトリクス Rune (v28.1.0)
// Pushgateway 経由でメトリクスを送信する。ローカル開発・ETL パイプライン監視に活用する。
// v28.1.0 stub — 実際の HTTP 送信は v28.x 以降
public fn counter(name: String, value: Float, labels: String) -> Result<Unit, String> !Io {
    Prometheus.counter_raw(name, value, labels)
}
public fn gauge(name: String, value: Float) -> Result<Unit, String> !Io {
    Prometheus.gauge_raw(name, value)
}
public fn histogram(name: String, value: Float) -> Result<Unit, String> !Io {
    Prometheus.histogram_raw(name, value)
}
public fn push(gateway_url: String) -> Result<Unit, String> !Io {
    Prometheus.push_raw(gateway_url)
}
```

### T3 — checker.fav 更新（Phase 9a）
`fav/self/checker.fav` の `ns_to_effect` に `"Prometheus" => "Io"` を追加。
SQLite else ブロックの内側（最も深いネスト）に追加。

### T4 — example ファイル作成
`examples/observability/prometheus_demo.fav` — カスタムメトリクス送信デモ:
- `stage ReportMetrics: Unit -> Unit !Io` — counter / gauge / histogram を送信
- `stage PushToGateway: Unit -> Unit !Io` — Pushgateway へ push
- `seq PrometheusDemo = ReportMetrics |> PushToGateway`

### T5 — サイトドキュメント
`site/content/docs/runes/prometheus.mdx` 新規作成。

### T6 — CHANGELOG 更新
`CHANGELOG.md` に `[v28.1.0]` セクション追加。

### T7 — ベンチマーク
`benchmarks/v28.1.0.json` 新規作成（test_count: 2235）。

### T8 — driver.rs テスト（Phase 9b）
`v281000_tests` モジュール（9 件）を `driver.rs` に追加。

### T9 — テスト全通過確認
`cargo test --bin fav` で 2235 tests PASS。

## エフェクト設計

| Rune 関数 | エフェクト | 理由 |
|---|---|---|
| counter / gauge / histogram | `!Io` | Pushgateway への HTTP 送信（ネットワーク I/O） |
| push | `!Io` | 同上 |

## テスト数

- v28.0.0: 2226 tests
- v28.1.0: **2235 tests**（+9）

## 完了条件

- [ ] `Cargo.toml` version = "28.1.0"
- [ ] `runes/prometheus/prometheus.fav` 存在（4 関数、`!Io` エフェクト）
- [ ] `Prometheus.counter_raw` / `gauge_raw` / `histogram_raw` / `push_raw` VM primitive 存在（`#[cfg]` ガード付き）
- [ ] `fav/self/checker.fav` `ns_to_effect` に `ns == "Prometheus"` 条件あり
- [ ] `examples/observability/prometheus_demo.fav` に `PrometheusDemo` seq あり
- [ ] `site/content/docs/runes/prometheus.mdx` 存在
- [ ] `CHANGELOG.md` に `[v28.1.0]` セクションあり
- [ ] `benchmarks/v28.1.0.json` 存在（test_count: 2235）
- [ ] `cargo test --bin fav prometheus` — 7 件以上 PASS（ロードマップ要件 4 件超過）
- [ ] `cargo test --bin fav v281000` — 9/9 PASS
- [ ] `cargo test --bin fav` — 2235 tests PASS

> **checker.fav パスについて**: `include_str!("../../fav/self/checker.fav")` は
> v27.9.0 `checker_has_sqlite_effect` テストで実証済みの正しいパス（2226 tests PASS 確認）。
