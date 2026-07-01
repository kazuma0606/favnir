# Current — Favnir 進行状況

最終更新: 2026-07-01

---

## 現行マスターロードマップ

[roadmap/roadmap-v30.1-v35.0.md](roadmap/roadmap-v30.1-v35.0.md)

---

## 最新安定版

**v30.0.0** — Ecosystem Maturity 宣言（2026-07-01）

- 2372 tests pass
- `cargo install fav --version "30.0.0"`

---

## 進行中バージョン

なし（v30.0.0 完了直後）

---

## 次に切る版

**v30.1.0** — ビルド軽量化

- spec/plan/tasks: 未作成 → [テンプレート](_templates/version/spec.md) を使用
- 依存関係: なし
- 内容: `[profile.dev] debug = 0` + `.cargo/config.toml` でビルド生成物を軽量化

---

## マイルストーン進捗

| マイルストーン | 状態 | 備考 |
|---|---|---|
| v26.0 — Rune Foundation | **完了** | コア Rune 実質化 |
| v27.0 — Streaming Native | **完了** | Kafka / Kinesis ストリーム |
| v28.0 — Data Lakehouse | **完了** | Delta Lake / Iceberg / DuckDB |
| v29.0 — Observability First | **完了** | OTel / Prometheus / Datadog |
| v30.0 — Ecosystem Maturity | **完了** | Rune Registry / コミュニティ Rune |
| v31.0 — Real-World Readiness | planned | v30.1〜v30.9 完了後に宣言 |
| v32.0 — Language Polish | planned | v31.x 完了後 |
| v33.0 — Language Power | planned | v32.x 完了後（詳細はドッグフード後確定）|
| v34.0 — Performance & Tooling | planned | v33.x 完了後（同上）|
| v35.0 — Production Ready | planned | v34.x 完了後（同上）|

詳細は [INDEX.md](INDEX.md) / [roadmap/roadmap-v30.1-v35.0.md](roadmap/roadmap-v30.1-v35.0.md) を参照。
