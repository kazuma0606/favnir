# Plan: v99.0.0 — SAP Analytics 1.0 宣言

## 実装順序

> **重要**: `changelog_has_v99_0_0` / `milestone_has_sap_analytics` / `readme_mentions_sap_analytics` テストが通るには、CHANGELOG / MILESTONE / README をテストモジュール追加より**先**に更新する必要がある。

---

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version = "98.0.0"` を `version = "99.0.0"` に変更する。

---

### Step 2: MILESTONE.md に v99.0.0 エントリを追加

先頭（`## v98.0.0` の前）に追加：

```markdown
## v99.0.0（2026-09-03）— SAP Analytics 1.0 宣言

> 「SAP のデータが、洞察になった。
>
>  `KpiDefinition<SalesOrder>` が売上の健全性を測り、
>  BW クエリの結果が SAC に流れ、
>  閾値を超えた瞬間に Slack が鳴る。
>
>  それが、Favnir SAP Analytics 1.0 である。」

**SAP Analytics 1.0** の宣言バージョン。v98.1.0〜v98.9.0 で実装した
KPI 型定義・BW クエリ・SAC プッシュ・KPI アラート・CLI・E2E デモ・サイトドキュメントの完成を宣言した。
テスト数: 4,257。

**SAP Analytics 1.0（v98.1〜v98.9）達成内容:**
- **`KpiDefinition<T>` / `KpiSnapshot<T>`**: KPI を型で定義し計測結果をスナップショットとして保持
- **`BwQuery<T>` / `BwResult<T>`**: BW/4HANA クエリの型安全なインターフェース
- **`SacDataset` / `sac_push_mock`**: SAC へのデータプッシュ API
- **`report_to_sac_rows`**: `SalesReport` → SAC CSV 行リスト変換
- **`KpiAlert` / `format_kpi_alert`**: 閾値超えアラートの型と整形関数
- **`fav report --sap`**: HTML レポート生成 CLI コマンド
- **`analytics_demo/`**: 日次売上 KPI → SAC → アラートの E2E デモ
- **`sap-analytics.mdx`**: KPI 定義・BW クエリ・SAC プッシュの完全ガイド

---
```

---

### Step 3: README.md に v99.0 セクションを追加

`## v98.0 — SAP Workflow 1.0 宣言` の前に追加：

```markdown
## v99.0 — SAP Analytics 1.0 宣言（2026-09-03）

Favnir v99.0 で **SAP Analytics 1.0** を宣言しました。

`KpiDefinition<SalesOrder>` が売上の健全性を測り、BW クエリの結果が SAC に流れ、
閾値を超えた瞬間に Slack が鳴る。

\`\`\`favnir
-- 日次売上 KPI を定義し、SAC プッシュ → アラート送信する pipeline
pipeline kpi_monitor !SapOData !SapAnalytics {
    stage Fetch {
        bind orders <- ctx.sap.sales_orders(SalesOrderFilter {
            date_from: Option.some("2026-09-03"),
            date_to:   Option.none(),
            top:       Option.some(5000)
        })
    }
    |> stage Evaluate {
        bind report  <- build_sales_report("2026-09-03", orders)
        bind kpi_def <- Result.ok(KpiDefinition {
            name:      "DailyRevenue",
            unit:      "JPY",
            threshold: KpiThreshold { warning: 500000.0, critical: 1000000.0 },
            extract:   |_| 0.0
        })
        bind snap    <- Result.ok(make_kpi_snapshot(kpi_def, report.total_amount, "2026-09-03"))
    }
    |> stage Alert {
        bind alert <- Result.ok(KpiAlert {
            kpi_name: snap.kpi.name,
            status:   snap.status,
            message:  Float.to_string(snap.value)
        })
        bind msg   <- Result.ok(format_kpi_alert(alert))
        -- "[CRITICAL] DailyRevenue: 1200000.0" のような文字列を生成
        bind _     <- Result.ok(msg)
    }
}
\`\`\`

---
```

---

### Step 4: CHANGELOG.md に v99.0.0 エントリを追加

先頭に追加（`[v98.9.0]` の前）。

---

### Step 5: driver.rs に mod v99000_tests を追加

`mod v98900_tests` の直後に追加：

```rust
#[cfg(test)]
mod v99000_tests {
    // use super::* は不要（外部シンボル未使用）
    #[test]
    fn cargo_toml_version_is_99_0_0() {
        let content = std::fs::read_to_string("../fav/Cargo.toml")
            .unwrap_or_else(|_| std::fs::read_to_string("Cargo.toml").unwrap());
        assert!(
            content.contains("version = \"99.0.0\""),
            "Cargo.toml should declare version 99.0.0"
        );
    }

    #[test]
    fn changelog_has_v99_0_0() {
        let content = std::fs::read_to_string("../CHANGELOG.md").unwrap();
        assert!(
            content.contains("[v99.0.0]"),
            "CHANGELOG.md should have v99.0.0 entry"
        );
    }

    #[test]
    fn milestone_has_sap_analytics() {
        let content = std::fs::read_to_string("../MILESTONE.md").unwrap();
        assert!(
            content.contains("SAP Analytics"),
            "MILESTONE.md should mention SAP Analytics 1.0"
        );
    }

    #[test]
    fn readme_mentions_sap_analytics() {
        let content = std::fs::read_to_string("../README.md").unwrap();
        assert!(
            content.contains("SAP Analytics"),
            "README.md should mention SAP Analytics 1.0"
        );
    }
}
```

---

### Step 6: cargo test 実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -- --test-threads=1 2>&1 | grep "test result"
```

期待値: 4,257 tests, 0 failures

---

### Step 7: cargo clean（★クリーンアップ）

```bash
cargo clean
```

cargo clean 後に `fav/tmp/hello.fav` が消えるため、以下の内容で復元する:

```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

---

### Step 8: cargo test（cargo clean 後）

```bash
cargo test -- --test-threads=1 2>&1 | grep "test result"
```

期待値: 4,257 tests, 0 failures

---

### Step 9: versions/current.md 更新

最新安定版を `v99.0.0` に更新（テスト数 4,257）。

---

### Step 10: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
