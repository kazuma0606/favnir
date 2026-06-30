# v28.7.0 Spec — オブザーバビリティ E2E デモ（prometheus + grafana）

## 概要

v28.1（prometheus Rune）・v28.6（grafana Rune）が揃ったので、
ETL パイプラインのメトリクスを Grafana ダッシュボードにリアルタイム表示する
E2E デモを `examples/observability/` に追加する。

**新規コンパイラ機能なし**。追加するのはデモファイル・Docker 設定・ドキュメントのみ。
`#[track]` アノテーションは v28.7.0 時点では **コメント形式**（`// #[track...]`）で記述し、
実際のメトリクス送信は `Prometheus.*` 関数の明示的な呼び出しで行う。
（`#[track]` のコンパイラ自動挿入は v29.0+ 実装予定）

> **ロードマップとの差分**: `roadmap-v28.1-v29.0.md` の設計決定事項テーブルでは
> `#[track]` の `parser.rs` / `ast.rs` 追加を v28.x の作業として記載しているが、
> v28.7.0 ではデモ優先とし、コンパイラ実装は v29.0 マイルストーンに後送りする。
> ロードマップ v28.7 セクションのコード例は有効アノテーション形式で書かれているが、
> v28.7.0 の実装ではコメント形式に変更している。

---

## 追加ファイル

| ファイル | 内容 |
|---|---|
| `examples/observability/prometheus_grafana.fav` | ETL デモ（3 stage + seq、Prometheus 明示呼び出し）|
| `examples/observability/docker-compose.yml` | prometheus / grafana Docker 定義 |
| `examples/observability/README.md` | セットアップ手順（docker compose up → fav run → Grafana UI）|
| `site/content/docs/tools/observability-e2e.mdx` | E2E デモ解説ドキュメント |
| `benchmarks/v28.7.0.json` | `{"version":"28.7.0","test_count":2289}` |
| `CHANGELOG.md` | `[v28.7.0]` セクション追加 |

---

## prometheus_grafana.fav の設計

> **シグネチャ簡略化**: ロードマップの v28.7 コード例では `ExtractOrders: Unit -> List<RawOrder> !Db`
> のようにステージ間でデータを受け渡すが、v28.7.0 はコンパイラ機能なしのデモのため
> 全 stage を `Unit -> Result<Unit, String> !Io` に統一し副作用をチェーンする形式に簡略化した。

```favnir
import runes/prometheus
import runes/grafana

// #[track(latency: true, error_rate: true)] — v29.0+ でコンパイラが自動挿入予定
stage ExtractOrders: Unit -> Result<Unit, String> !Io = |_| {
    bind _ <- Prometheus.counter("rows_extracted_total", 100.0, "pipeline:etl")
    Result.ok(unit)
}

// #[track(latency: true)]
stage TransformOrders: Unit -> Result<Unit, String> !Io = |_| {
    bind _ <- Prometheus.histogram("transform_duration_ms", 42.5)
    Result.ok(unit)
}

// #[track(latency: true, error_rate: true)]
stage LoadToWarehouse: Unit -> Result<Unit, String> !Io = |_| {
    bind _ <- Prometheus.gauge("warehouse_rows", 100.0)
    bind _ <- Grafana.create_annotation("etl-dashboard", "ETL cycle complete", "pipeline,favnir")
    Result.ok(unit)
}

seq PrometheusGrafanaDemo = ExtractOrders |> TransformOrders |> LoadToWarehouse
```

- `#[track]` はコメント形式で記述（ファイル内に `#[track` 文字列が含まれることをテストで確認）
- 実際のメトリクス送信は `Prometheus.*` / `Grafana.*` 明示呼び出し
- seq 名: `PrometheusGrafanaDemo`

---

## docker-compose.yml の設計

```yaml
version: "3.8"
services:
  prometheus:
    image: prom/prometheus:v2.45.0
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
  grafana:
    image: grafana/grafana:10.0.0
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    depends_on:
      - prometheus
```

---

## テスト一覧（driver.rs v287000_tests）

| # | テスト名 | 確認内容 |
|---|---|---|
| 1 | `prometheus_grafana_example_has_extract_stage` | `prometheus_grafana.fav` に `stage ExtractOrders` を含む |
| 2 | `prometheus_grafana_example_has_seq` | `prometheus_grafana.fav` に `seq PrometheusGrafanaDemo` を含む |
| 3 | `prometheus_grafana_example_has_track_annotation` | `prometheus_grafana.fav` に `#[track` を含む |
| 4 | `docker_compose_has_prometheus` | `docker-compose.yml` に `prometheus` を含む |
| 5 | `docker_compose_has_grafana` | `docker-compose.yml` に `grafana` を含む |
| 6 | `observability_readme_has_docker_compose` | `examples/observability/README.md` に `docker compose` を含む |
| 7 | `observability_e2e_doc_exists` | `site/content/docs/tools/observability-e2e.mdx` に `PrometheusGrafanaDemo` を含む |
| 8 | `changelog_has_v28_7_0` | `CHANGELOG.md` に `[v28.7.0]` または `## v28.7.0` を含む |

合計 8 テスト。test_count: **2289**（2281 + 8）

`cargo test v287000` — 8/8 PASS。

---

## 完了条件チェックリスト

- [ ] `Cargo.toml` version = `28.7.0`
- [ ] `examples/observability/prometheus_grafana.fav` 存在（`PrometheusGrafanaDemo` seq、`#[track` コメント、`Prometheus.*` / `Grafana.*` 呼び出し）
- [ ] `examples/observability/docker-compose.yml` 存在（prometheus / grafana サービス定義）
- [ ] `examples/observability/README.md` 存在（`docker compose` 手順含む）
- [ ] `site/content/docs/tools/observability-e2e.mdx` 存在（`PrometheusGrafanaDemo` 言及）
- [ ] `CHANGELOG.md` に `[v28.7.0]` セクションあり
- [ ] `benchmarks/v28.7.0.json` 存在（test_count: 2289）
- [ ] `cargo test --bin fav v287000` — 8/8 PASS
- [ ] `cargo test --bin fav` — 2289 tests PASS
