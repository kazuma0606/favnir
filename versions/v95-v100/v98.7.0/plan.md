# Plan: v98.7.0 — E2E デモ（日次 KPI → SAC → Slack）

## 実装順序

### 前提確認

- Favnir `|>` pipeline stage は環境を共有する（前 stage の `bind` 変数は後続 stage から参照可）
- `snap.kpi.name` の 2 段フィールドアクセスは有効（`analytics.fav` の `kpi.threshold.critical` で実証済み）
- `bind _ <- Result.ok(msg)` は Slack 送信の placeholder（将来 `ctx.slack.post(msg)` に置き換え可）
- `report_to_sac_rows` は `sap_odata.fav` の re-export 経由で利用

---

### Step 1: analytics_demo/ ディレクトリを作成し pipeline_kpi_monitor.fav を新規作成

`infra/e2e-demo/sap-odata/analytics_demo/pipeline_kpi_monitor.fav` を作成。

内容: `kpi_monitor` pipeline（`!SapOData !SapAnalytics`）:
- stage Fetch: `ctx.sap.sales_orders()` で売上データ取得
- stage Evaluate: `build_sales_report` → `make_kpi_snapshot` で KPI 評価
- stage Push: `report_to_sac_rows` → `sac_push_mock` で SAC プッシュ
- stage Alert: `KpiAlert` 生成 → `format_kpi_alert` でメッセージ整形

Favnir コメントは `--` スタイル（`//` 不可）。

---

### Step 2: run.sh を新規作成

`infra/e2e-demo/sap-odata/analytics_demo/run.sh` を作成：

```bash
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# SAP KPI Monitor E2E デモ実行スクリプト（v98.7.0）
fav run "${SCRIPT_DIR}/pipeline_kpi_monitor.fav"
```

---

### Step 3: README.md を新規作成

`infra/e2e-demo/sap-odata/analytics_demo/README.md` を作成：

- デモ概要・前提条件・実行手順・pipeline フロー図を記載
- `workflow_demo/README.md` と同じ構成に従う

---

### Step 4: driver.rs に mod v98700_tests を追加

`mod v98600_tests` の直後に追加：

```rust
#[cfg(test)]
mod v98700_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn analytics_demo_pipeline_exists() {
        std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/analytics_demo/pipeline_kpi_monitor.fav"
        ).expect("analytics_demo/pipeline_kpi_monitor.fav should exist (v98.7.0)");
    }

    #[test]
    fn pipeline_kpi_monitor_has_kpi_alert() {
        let content = std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/analytics_demo/pipeline_kpi_monitor.fav"
        ).expect("analytics_demo/pipeline_kpi_monitor.fav should exist");
        assert!(
            content.contains("KpiAlert"),
            "pipeline_kpi_monitor.fav should reference KpiAlert (v98.7.0)"
        );
    }
}
```

---

### Step 5: テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -- --test-threads=1 2>&1 | grep "test result"
```

期待値: 4,249 tests, 0 failures

---

### Step 6: CHANGELOG.md に v98.7.0 エントリを追加

---

### Step 7: versions/current.md 更新

最新安定版を `v98.7.0` に更新（テスト数 4,249）。

---

### Step 8: CI 事前確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
