# Current — Favnir 進行状況

最終更新: 2026-08-18 (v83.0.0)

---

## 現行マスターロードマップ

[roadmap/roadmap-v80.1-v85.0.md](roadmap/roadmap-v80.1-v85.0.md)

前フェーズ（完了）: [roadmap/roadmap-v75.1-v80.0.md](roadmap/roadmap-v75.1-v80.0.md)

前フェーズ（完了）: [roadmap/roadmap-v70.1-v75.0.md](roadmap/roadmap-v70.1-v75.0.md)

前フェーズ（完了）: [roadmap/roadmap-v65.1-v70.0.md](roadmap/roadmap-v65.1-v70.0.md)

前フェーズ（完了）: [roadmap/roadmap-v60.1-v65.0.md](roadmap/roadmap-v60.1-v65.0.md)

サブスプリント（完了）: [roadmap/roadmap-v54.1-v55.0.md](roadmap/roadmap-v54.1-v55.0.md)（v55.0.0 宣言完了）

サブスプリント（完了）: [roadmap/roadmap-v53.1-v54.0.md](roadmap/roadmap-v53.1-v54.0.md)（v54.0.0 宣言完了）

前フェーズ（完了）: [roadmap/roadmap-v45.1-v50.0.md](roadmap/roadmap-v45.1-v50.0.md)

---

## 最新安定版

**v83.0.0** — Pipeline Contracts 1.0 宣言 — 3875 tests（2026-08-18）

- `cargo install fav --version "83.0.0"`

前バージョン: v80.0.0 — Favnir 3.0 宣言 — 3809 tests

---

## 進行中バージョン

**v83.1.0〜v84.0.0**（Observability 2.0 スプリント — Quality-First Era Sprint 4）

---

## 次に切る版

**v84.1.0**（Favnir 4.0 宣言スプリント開始）

---

## !Effect 廃止ロードマップ

| バージョン | スプリント | 内容 |
|---|---|---|
| v35.3.0 | v34.7A | examples/ + infra/ から !Effect 除去 ✅ |
| v35.4.0 | v34.8A | parser で !Effect を E0374 ハードエラー化 ✅ |
| v35.6.0 | v34.9A | Effect enum + effects フィールドの完全削除 ✅ |
| v35.6.0 | v35.0A | サイト MDX 125 件を ctx 構文に統一 + v35.0 Production Ready 宣言 ✅ |

---

## マイルストーン進捗

| マイルストーン | 状態 | 備考 |
|---|---|---|
| v26.0 — Rune Foundation | **完了** | コア Rune 実質化 |
| v27.0 — Streaming Native | **完了** | Kafka / Kinesis ストリーム |
| v28.0 — Data Lakehouse | **完了** | Delta Lake / Iceberg / DuckDB |
| v29.0 — Observability First | **完了** | OTel / Prometheus / Datadog |
| v30.0 — Ecosystem Maturity | **完了** | Rune Registry / コミュニティ Rune |
| v31.0 — Real-World Readiness | **完了** | v30.1〜v30.9 完了後に宣言（2026-07-02） |
| v32.0 — Language Polish | **完了** | v31.1〜v31.9 完了後に宣言（2026-07-03） |
| v33.0 — Language Power | **完了** | v32.1〜v32.9 完了後に宣言（2026-07-03） |
| v34.0 — Performance & Tooling | **完了** | v33.x 完了後（2026-07-04）|
| v35.0 — Production Ready | **完了** | v34.x 完了後（2026-07-04）|
| v36.0 — Deployment Story | **完了** | v35.1〜v35.9 完了後に宣言（2026-07-08） |
| v37.0 — Data Quality First | **完了** | v36.1〜v36.9 完了後に宣言（2026-07-09） |
| v38.0 — Multi-Source ETL Power | **完了** | v37.1〜v37.9 完了後に宣言（2026-07-10） |
| v39.0 — Intelligence & Assistance | **完了** | v38.1〜v38.9 完了後に宣言（2026-07-10） |
| v40.0 — Enterprise Governance | **完了** | v39.1〜v39.9 完了後に宣言（2026-07-11） |
| v65.0 — Performance 1.0 | **完了** | v60.1〜v64.9 完了後（2026-08-04） |
| v70.0 — Intelligent ETL 1.0 | **完了** | v65.1〜v69.9 完了後（2026-08-08） |
| v71.0 — Language Complete 1.0 | **完了** | v70.1〜v70.9（積み残し解消 + compiler.fav 完全化）（2026-08-09） |
| v72.0 — Type System 2.0 | **完了** | v71.1〜v71.9（依存型・refined type・AOT・WASM・phantom type）（2026-08-11） |
| v73.0 — Developer Exp 2.0 | **完了** | v72.1〜v72.9（VS Code・AI アシスタント・REPL・Playground・fav learn）（2026-08-13） |
| v74.0 — Production Proven | **完了** | v73.1〜v73.9 完了後（2026-08-13） |
| v75.0 — Favnir 2.0 | **完了** | v74.1〜v74.9（統合・Rune マーケット・宣言）（2026-08-14） |
| v80.0 — Favnir 3.0 | **完了** | v75.1〜v79.9（時間型・来歴型・証明可能型・実行戦略）（2026-08-16） |
| v81.0 — Test-Driven Data 1.0 | **完了** | v80.1〜v80.9（fav test・GoldenDataset・SchemaSnapshot）（2026-08-16） |
| v82.0 — Data Quality 2.0 | **完了** | v81.1〜v81.9（QualityRule・QualityGate・AnomalyDetector）（2026-08-16） |
| v83.0 — Pipeline Contracts 1.0 | **完了** | v82.1〜v82.9（IoContract・SlaContract・ContractRegistry）（2026-08-18） |

詳細は [INDEX.md](INDEX.md) / [roadmap/roadmap-v70.1-v75.0.md](roadmap/roadmap-v70.1-v75.0.md) を参照。
