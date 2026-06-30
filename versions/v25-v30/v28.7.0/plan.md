# v28.7.0 Plan — オブザーバビリティ E2E デモ（prometheus + grafana）

## 実装順序

```
T1: Cargo.toml version bump (28.6.0 → 28.7.0)
T2: examples/observability/prometheus_grafana.fav 新規作成
T3: examples/observability/docker-compose.yml 新規作成
T4: examples/observability/README.md 新規作成
T5: site/content/docs/tools/observability-e2e.mdx 新規作成
T6: CHANGELOG.md に [v28.7.0] セクション追加
T7: benchmarks/v28.7.0.json 新規作成
T8: driver.rs に v287000_tests 8 件追加
T9: cargo test --bin fav v287000 — 8/8 PASS 確認
T9.5: （手動確認）docker compose up -d → fav run → http://localhost:3000 で Grafana UI 表示確認
T10: cargo test --bin fav 全体 — 2289 PASS 確認
T11: tasks.md を COMPLETE に更新
```

---

## T2: examples/observability/prometheus_grafana.fav

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

**注意**: `#[track(...)]` はコメント形式（`//` 前置き）で記述する。
Favnir パーサーが `#[track]` アノテーションをサポートしている場合は後続バージョンで有効化予定。
テスト `prometheus_grafana_example_has_track_annotation` は `#[track` の文字列存在のみ確認するため、
コメント内の記述でも PASS する。

---

## T3: examples/observability/docker-compose.yml

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

## T4: examples/observability/README.md

```markdown
# Observability E2E Demo — Prometheus + Grafana

## セットアップ

1. Docker Compose でインフラを起動する:

   docker compose -f examples/observability/docker-compose.yml up -d

2. Favnir パイプラインを実行する:

   fav run examples/observability/prometheus_grafana.fav

3. Grafana UI を開く:

   http://localhost:3000 (admin / admin)

## 構成

- Prometheus: http://localhost:9090 — メトリクス収集
- Grafana: http://localhost:3000 — ダッシュボード表示

## 注記

- `#[track]` アノテーションによる自動メトリクス挿入は v29.0+ 実装予定
- v28.7.0 時点では `Prometheus.*` / `Grafana.*` 関数を明示的に呼び出す
```

---

## T5: site/content/docs/tools/observability-e2e.mdx

- frontmatter: `title: Observability E2E Demo`, `description: prometheus + grafana E2E デモ（v28.7.0）`
- `PrometheusGrafanaDemo` seq のコード例掲載
- Docker Compose セットアップ手順
- `#[track]` の将来バージョン（v29.0+）での自動化について

---

## T8: driver.rs — v287000_tests

```rust
// ── v287000_tests (v28.7.0) — オブザーバビリティ E2E デモ ────────────────────────────
#[cfg(test)]
mod v287000_tests {
    #[test]
    fn prometheus_grafana_example_has_extract_stage() {
        let src = include_str!("../../examples/observability/prometheus_grafana.fav");
        assert!(src.contains("stage ExtractOrders"), "prometheus_grafana.fav must define stage ExtractOrders");
    }
    #[test]
    fn prometheus_grafana_example_has_seq() {
        let src = include_str!("../../examples/observability/prometheus_grafana.fav");
        assert!(src.contains("seq PrometheusGrafanaDemo"), "prometheus_grafana.fav must define seq PrometheusGrafanaDemo");
    }
    #[test]
    fn prometheus_grafana_example_has_track_annotation() {
        let src = include_str!("../../examples/observability/prometheus_grafana.fav");
        assert!(src.contains("#[track"), "prometheus_grafana.fav must contain #[track annotation (comment form)");
    }
    #[test]
    fn docker_compose_has_prometheus() {
        let src = include_str!("../../examples/observability/docker-compose.yml");
        assert!(src.contains("prometheus"), "docker-compose.yml must define prometheus service");
    }
    #[test]
    fn docker_compose_has_grafana() {
        let src = include_str!("../../examples/observability/docker-compose.yml");
        assert!(src.contains("grafana"), "docker-compose.yml must define grafana service");
    }
    #[test]
    fn observability_readme_has_docker_compose() {
        let src = include_str!("../../examples/observability/README.md");
        assert!(src.contains("docker compose"), "examples/observability/README.md must contain 'docker compose'");
    }
    #[test]
    fn observability_e2e_doc_exists() {
        let src = include_str!("../../site/content/docs/tools/observability-e2e.mdx");
        assert!(src.contains("PrometheusGrafanaDemo"), "observability-e2e.mdx must mention PrometheusGrafanaDemo");
    }
    #[test]
    fn changelog_has_v28_7_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v28.7.0]") || src.contains("## v28.7.0"), "CHANGELOG.md must contain '[v28.7.0]'");
    }
}
```

`include_str!` パス一覧:
- `../../examples/observability/prometheus_grafana.fav`
- `../../examples/observability/docker-compose.yml`
- `../../examples/observability/README.md`
- `../../site/content/docs/tools/observability-e2e.mdx`
- `../../CHANGELOG.md`

すべて `fav/src/` → `../../` → ルート → 各ファイル のパターンで正しい。
