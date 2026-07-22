# Current — Favnir 進行状況

最終更新: 2026-07-23 (v55.0.0)

---

## 現行マスターロードマップ

[roadmap/roadmap-v50.1-v55.0.md](roadmap/roadmap-v50.1-v55.0.md)

サブスプリント（完了）: [roadmap/roadmap-v54.1-v55.0.md](roadmap/roadmap-v54.1-v55.0.md)（v55.0.0 宣言完了）

サブスプリント（完了）: [roadmap/roadmap-v53.1-v54.0.md](roadmap/roadmap-v53.1-v54.0.md)（v54.0.0 宣言完了）

前フェーズ（完了）: [roadmap/roadmap-v45.1-v50.0.md](roadmap/roadmap-v45.1-v50.0.md)

---

## 最新安定版

**v55.0.0** — Production 3.0 宣言 — 3206 tests（2026-07-23）

- `cargo install fav --version "55.0.0"`

前バージョン: v54.9.0 — v55.0 前調整・安定化 — 3203 tests

---

## 進行中バージョン

**なし**（v55.0.0 完了）

---

## 次に切る版

**未定**（Production 3.0 完成のため）

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

詳細は [INDEX.md](INDEX.md) / [roadmap/roadmap-v40.1-v45.0.md](roadmap/roadmap-v40.1-v45.0.md) を参照。
