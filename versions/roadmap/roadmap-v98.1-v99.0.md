# Roadmap v98.1.0 〜 v99.0.0 — SAP Analytics 1.0

Date: 2026-08-30
Status: 未着手

マスターロードマップ: [roadmap-v95.1-v100.0.md](roadmap-v95.1-v100.0.md)

---

## 前提

- 直前完了: v98.0.0「SAP Workflow 1.0 宣言」（tests = 4,235）※ code-reviewer 対応累積により計画値 4,230 から +5
- 本スプリントは SAP Platform Era の第 4 スプリント
- 目標: v99.0.0「SAP Analytics 1.0 宣言」（tests = 4,257）

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v98.0.0 になっていることを確認する
- `runes/sap-odata/workflow.fav` が存在することを確認する（v97.1.0 完了済みの証拠）
- `fav/src/driver.rs` に `mod v98000_tests` が存在することを確認する（v98.0.0 完了済みの証拠）
- `fav/Cargo.toml` の version が `98.0.0` であることを確認する

### スプリントの性格

SAP Platform Era の**アナリティクス・BI スプリント**。

SAP Analytics Cloud（SAC）へのデータプッシュ、BW/4HANA クエリ、
KPI 型定義とモニタリング、`fav report` コマンドによるローカルレポート生成を実装する。
「SAP データから洞察を得る」を Favnir pipeline で自動化する。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v98.1.0 | `KpiDefinition<T>` / `KpiSnapshot<T>` 型定義 | 4235 + 2 = 4237 | 未着手 |
| v98.2.0 | `BwQuery<T>` 型 + `ctx.sap.bw_query()` — BW/4HANA クエリインターフェース | 4237 + 2 = 4239 | 未着手 |
| v98.3.0 | SAP Analytics Cloud データプッシュ API（`ctx.sap.sac_push()`） | 4239 + 2 = 4241 | 未着手 |
| v98.4.0 | レポート自動生成 pipeline（Favnir → SAC ダッシュボードデータ） | 4241 + 2 = 4243 | 未着手 |
| v98.5.0 | KPI 閾値アラート（`KpiAlert` 型 + Slack / メール通知 pipeline） | 4243 + 2 = 4245 | 未着手 |
| v98.6.0 | `fav report --sap`（ローカル HTML レポート生成コマンド） | 4245 + 2 = 4247 | 未着手 |
| v98.7.0 | E2E デモ（日次売上 KPI → SAC プッシュ → Slack アラート） | 4247 + 2 = 4249 | 未着手 |
| v98.8.0 | サイトドキュメント（Analytics / KPI パターンガイド） | 4249 + 2 = 4251 | 未着手 |
| v98.9.0 | 安定化・コードフリーズ | 4251 + 2 = 4253 | 未着手 |
| v99.0.0 | SAP Analytics 1.0 宣言 ★クリーンアップ | 4253 + 4 = 4257 | 未着手 |

---

## v98.1.0 — `KpiDefinition<T>` / `KpiSnapshot<T>` 型定義

KPI を型で定義し、計測結果をスナップショットとして保持する型を追加する。

```favnir
type KpiThreshold = {
    warning:  Float,
    critical: Float
}

type KpiDefinition<T> = {
    name:      String,
    unit:      String,
    threshold: KpiThreshold,
    extract:   fn(T) -> Float
}

type KpiStatus =
    | Ok
    | Warning(Float)
    | Critical(Float)

type KpiSnapshot<T> = {
    kpi:       KpiDefinition<T>,
    value:     Float,
    status:    KpiStatus,
    measured_at: String
}
```

**修正ファイル**: `runes/sap-odata/analytics.fav`（新規）、`fav/src/driver.rs`

---

## v98.2.0 — `BwQuery<T>` + `ctx.sap.bw_query()`

BW/4HANA の InfoProvider / Query に対応する型安全クエリインターフェースを追加する。

```favnir
type BwQuery<T> = {
    info_provider: String,
    characteristics: List<String>,
    key_figures:    List<String>,
    filters:        List<String>
}

type BwResult<T> = {
    rows:  List<T>,
    total: Int
}

-- BW クエリ実行
bind result <- ctx.sap.bw_query<SalesKpi>(BwQuery {
    info_provider:   "0SD_C03",
    characteristics: ["0CALMONTH", "0SOLD_TO"],
    key_figures:     ["0NET_VAL_S"],
    filters:         ["0CALMONTH = 202608"]
})
```

**修正ファイル**: `runes/sap-odata/analytics.fav`、`fav/src/driver.rs`

---

## v98.3.0 — SAP Analytics Cloud データプッシュ API

SAC の Data Import Service API に接続し、Favnir から SAC ダッシュボードデータを更新する。

```favnir
type SacDataset = {
    model_id: String,
    rows:     List<String>    -- CSV 形式の行データ
}

-- SAC にデータをプッシュ
bind _ <- ctx.sap.sac_push(SacDataset {
    model_id: "SAP__FI_GL_IM_GLACCOUNTS",
    rows:     csv_rows
})
```

**修正ファイル**: `runes/sap-odata/sac.fav`（新規）、`runes/sap-odata/sap_odata.fav`（追記）、`fav/src/driver.rs`

---

## v98.4.0 — レポート自動生成 pipeline

SAP データから SAC ダッシュボード用のデータを生成し、プッシュする pipeline を実装する。

```favnir
-- !SapOData: ctx.sap.* アクセス / !SapAnalytics: ctx.sap.sac_push() アクセスのマーカー
-- （!SapAnalytics は v98.3.0 で Effect::SapAnalytics として Rust 側に追加する）
pipeline daily_sales_report !SapOData !SapAnalytics {
    stage Fetch {
        bind orders <- ctx.sap.sales_orders(SalesOrderFilter {
            date_from: Option.some(today()),
            date_to:   Option.none(),
            top:       Option.some(5000)
        })
    }
    |> stage Aggregate {
        bind report <- build_sales_report(today(), orders)
    }
    |> stage Push {
        bind rows <- report_to_sac_rows(report)
        bind _    <- ctx.sap.sac_push(SacDataset {
            model_id: "FAVNIR_DAILY_SALES",
            rows:     rows
        })
    }
}
```

**修正ファイル**: `infra/e2e-demo/sap-odata/pipeline_analytics.fav`（新規）、`fav/src/driver.rs`
**Rust 側（v98.4.0 で追加 ※延期）**: `Effect::SapAnalytics` を `Effect` enum に追加、`checker.fav` の exhaustive match 更新
（当初 v98.3.0 で予定していたが、スプリント調整により v98.4.0 に延期）

---

## v98.5.0 — KPI 閾値アラート + Slack / メール通知

KPI スナップショットを評価し、閾値超えを検出したら Slack / メールで通知する pipeline を追加する。

```favnir
type KpiAlert = {
    kpi_name: String,
    status:   KpiStatus,
    message:  String
}

-- KPI チェック + アラート送信
bind snaps  <- List.map(kpi_defs, fn(kpi) { measure_kpi(kpi, orders) })
bind alerts <- List.filter(snaps, fn(s) { s.status != Ok })
bind _      <- List.map(alerts, fn(alert) {
    ctx.slack.post("#sap-alerts", format_kpi_alert(alert))
})
```

**修正ファイル**: `runes/sap-odata/analytics.fav`、`fav/src/driver.rs`

---

## v98.6.0 — `fav report --sap`

SAP データからローカル HTML レポートを生成する CLI コマンドを追加する。

```
$ fav report --sap --entity SalesOrder --from 2026-08-01 --to 2026-08-31 --output report.html
Fetching SalesOrder from SAP... 1,234 records
Generating report...
Saved: report.html
```

**修正ファイル**: `fav/src/main.rs`、`fav/src/driver.rs`

---

## v98.7.0 — E2E デモ（日次 KPI → SAC → Slack）

日次売上 KPI 計算 → SAC プッシュ → 閾値超え Slack アラートの完全な E2E デモを実装する。

```
infra/e2e-demo/sap-odata/
  analytics_demo/
    README.md
    run.sh
    pipeline_kpi_monitor.fav
```

**修正ファイル**: `infra/e2e-demo/sap-odata/analytics_demo/`（新規）、`fav/src/driver.rs`

---

## v98.8.0 — サイトドキュメント

`site/content/docs/guides/sap-analytics.mdx` を新規作成する。

**内容**:
- KPI 定義パターン（`KpiDefinition<T>` / `KpiSnapshot<T>`）
- BW/4HANA クエリの使い方
- SAC データプッシュの設定
- `fav report --sap` コマンドリファレンス

**修正ファイル**: `site/content/docs/guides/sap-analytics.mdx`（新規）、`fav/src/driver.rs`

---

## v98.9.0 — 安定化・コードフリーズ

- 全テスト通過確認（4,253 tests, 0 failures）
- `cargo clippy --locked -- -D warnings` 通過
- `./target/debug/fav fmt --check self/compiler.fav` 通過
- `./target/debug/fav fmt --check self/checker.fav` 通過

---

## v99.0.0 — SAP Analytics 1.0 宣言

**宣言文**:

> 「SAP のデータが、洞察になった。
>
>  `KpiDefinition<SalesOrder>` が売上の健全性を測り、
>  BW クエリの結果が SAC に流れ、
>  閾値を超えた瞬間に Slack が鳴る。
>
>  それが、Favnir SAP Analytics 1.0 である。」

**v99000_tests（4 テスト）**:
- `cargo_toml_version_is_99_0_0`
- `changelog_has_v99_0_0`
- `milestone_has_sap_analytics`
- `readme_mentions_sap_analytics`

---

## スプリント終了時の確認

- [ ] 4,257 tests, 0 failures
- [ ] `cargo clean` を実施する（★クリーンアップ）
- [ ] `cargo test` で 4,257 tests, 0 failures を再確認する（cargo clean 後）
- [ ] `cargo clippy --locked -- -D warnings` pass
- [ ] `./target/debug/fav fmt --check self/compiler.fav` pass
- [ ] `./target/debug/fav fmt --check self/checker.fav` pass
- [ ] `versions/current.md` を v99.0.0 に更新
- [ ] `MILESTONE.md` に v99.0.0 エントリを追加
- [ ] `README.md` に `## v99.0 — SAP Analytics 1.0` セクションを追加
