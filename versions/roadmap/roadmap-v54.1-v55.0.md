# Roadmap v54.1.0 〜 v55.0.0 — Production 3.0

Date: 2026-07-18
Status: 進行中（v54.1.0 COMPLETE）

---

## 前提

- 直前完了: v54.0.0「Integration Sprint」（tests ≥ 3179）
- マスターロードマップ: `roadmap-v50.1-v55.0.md`
- 本文書はマスターの v55.0 スプリント部分の詳細版
- **既存機能の扱い**: `fav explain-error`（`main.rs` `Some("explain-error")`）は既存実装済み。
  v54.1 は全コード網羅と `fav explain --error` への統合が目的。
  `fav run --watch`（v50.7 で実装）の拡張として v54.2 の `--watch-diff/--watch-summary` を追加。
  詳細はマスターロードマップ冒頭「既存機能との位置づけ」テーブルを参照。

---

## 目標

v51〜v54 で積み上げた全機能を最終調整・磨き上げ、
**「現場で選ばれる言語」として Favnir v55.0 — Production 3.0 を宣言する**。

---

## バージョン計画

### v54.1.0 — 全エラーコード `fav explain --error` 対応完備

`error_catalog.rs` に登録された全エラーコードに `fav explain --error` テキストを追加
（`EXPLAIN_CATALOG` を完備）。`explain_error_all_codes_have_collect_text` テストでカバレッジを強制
（E0419 含む将来追加コードも自動カバー）。

```bash
$ fav explain --error E0415
E0415 — Return Type Mismatch

  The return value's type does not match the declared return type.

  Example:
    fn greet() -> Int { return "hello" }  // E0415

  Fix:
    Change the return type annotation to String,
    or return an Int value instead.
```

**完了条件**: Rust テスト 2 件（ベース 3185 + 2 = 3187 tests passed, 0 failed）
- `explain_error_all_codes_have_collect_text`
- `explain_error_e0419_exists`

**実績**: COMPLETE — 3187 tests passed, 0 failed（2026-07-22）

---

### v54.2.0 — `fav run --watch` の高度化（差分表示・サマリー）

`--watch-diff` フラグを追加し、変化量（数値は差分、文字列は before/after）を表示。
複数 stage にまたがった追跡履歴をまとめてサマリー出力する `--watch-summary` オプションを追加。

```bash
$ fav run pipeline.fav --watch order.amount --watch order.status --watch-diff
[watch] order.amount:  0.0  → 99.0   Δ+99.0  (stage: Parse)
[watch] order.status:  None → "ok"            (stage: Validate)
```

**完了条件**: Rust テスト 2 件（ベース 3187 + 2 = 3189 tests passed, 0 failed）
- `run_watch_diff_numeric`
- `run_watch_summary_output`

**実績**: COMPLETE — 3189 tests passed, 0 failed（2026-07-22）

---

### v54.3.0 — パフォーマンスリグレッションスイート CI 統合

既存の `.github/workflows/bench.yml` に `fav bench --fail-on-regression` ステップを追加
（新ワークフロー新設ではなく既存を拡張）。`benchmarks/baseline.json` をリポジトリ管理し、PR ごとに自動比較。

```yaml
# .github/workflows/bench.yml（既存）に以下ステップを追加
- run: cargo test bench_ -- --nocapture
- run: fav bench --all --compare benchmarks/baseline.json --fail-on-regression
```

**完了条件**: Rust テスト 2 件（ベース 3189 + 2 = 3191 tests passed, 0 failed）
- `ci_perf_regression_suite`
- `ci_perf_baseline_recorded`

**実績**: COMPLETE — 3191 tests passed, 0 failed（2026-07-22）

---

### v54.4.0 — `fav dq-report` データ品質レポートコマンド

`fav dq-report` コマンドを追加。`--audit-log` の JSONL ログとスキーマ統計を集計し Markdown レポートを生成。

```bash
$ fav dq-report --audit-log audit.jsonl --schemas schemas/
Schema validation:  12,450 rows checked, 3 errors (0.02%)
  OrderRow:  12,100 / 12,100 OK
  PaymentRow:   350 / 350 OK
SLA violations:  latency >200ms at 2026-07-18T09:15:00Z (stage: Parse)
```

注意: v54.4.0 では `--schemas` フラグは未実装。スキーマ名は audit log の `"schema"` フィールドから直接取得する（将来バージョンで外部スキーマディレクトリ参照を追加予定）。

**完了条件**: Rust テスト 2 件（ベース 3191 + 2 = 3193 tests passed, 0 failed）
- `cmd_dq_report_generates`
- `cmd_dq_report_has_schema_stats`

**実績**: COMPLETE — 3193 tests passed, 0 failed（2026-07-22）

---

### v54.5.0 — `fav doctor` 環境診断コマンド

`fav doctor` コマンドを追加。Rust バージョン・`fav.toml` 有効性・rune インストール状態・
`.fav-cache` 整合性を一括チェック。

```bash
$ fav doctor
[OK]   Rust toolchain: 1.79.0
[OK]   fav version: 54.5.0
[OK]   fav.toml: valid
[WARN] rune kafka: version 2.1.0 declared but not installed
       run: fav install kafka
[OK]   .fav-cache: intact (fingerprints: 42 files)
```

注意: v54.5.0 では rune インストール状態の実チェック（`fav.toml` の rune 宣言解析）は将来バージョンに延期。
`cmd_doctor_detects_missing_rune` テストは `cmd_doctor_collect` の `[WARN]` フォーマット検証のみ実施。
`RUSTUP_TOOLCHAIN` 環境変数からチャンネル名を取得（上記サンプルの `1.79.0` は例示であり、実際はチャンネル名文字列を表示）。

**完了条件**: Rust テスト 2 件（ベース 3193 + 2 = 3195 tests passed, 0 failed）
- `cmd_doctor_passes_clean_env`
- `cmd_doctor_detects_missing_rune`

**実績**: COMPLETE — 3195 tests passed, 0 failed（2026-07-23）

---

### v54.6.0 — README / CONTRIBUTING 最終整備

`README.md` に Production 3.0 への言及・v54.1〜v54.5 機能サマリーを追加。
（v51〜v53 の各マイルストーン宣言は README に既掲載のため、v54.x サブバージョン整備サマリーに特化する）
`CONTRIBUTING.md` のコントリビュート手順を最新化（`fav doctor` / `fav bench` の実行手順追記）。

**完了条件**: Rust テスト 2 件（ベース 3195 + 2 = 3197 tests passed, 0 failed）
- `readme_has_production3_mention`
- `contributing_has_doctor_step`

**実績**: COMPLETE — 3197 tests passed, 0 failed（2026-07-23）

---

### v54.7.0 — ドキュメントサイト Production 3.0 overview ページ

`site/content/docs/production3-overview.mdx` を新規作成。v51〜v55 の全機能を統合した概要ページ。

```
site/content/docs/production3-overview.mdx
  - v51: Developer Experience 3.0（診断統一・インレイヒント・trace/watch）
  - v52: Performance & Scale（par Tokio・バックプレッシャー・bench 回帰）
  - v53: Data Quality & Observability 2.0（assert_schema・lineage 強化・audit-log）
  - v54: Integration Sprint（lineage × LSP・bench × par・E2E デモ）
  → v55: Production 3.0 宣言
```

**完了条件**: Rust テスト 2 件（ベース 3197 + 2 = 3199 tests passed, 0 failed）
- `docs_production3_overview_exists`
- `docs_production3_has_v55`

**実績**: COMPLETE — 3199 tests passed, 0 failed（2026-07-23）

---

### v54.8.0 — MILESTONE.md Production 3.0 エントリ追加

`MILESTONE.md` に `## v55.0.0（予定）— Production 3.0` エントリを追加。v51〜v54 の達成内容を記録。

**完了条件**: Rust テスト 2 件（ベース 3199 + 2 = 3201 tests passed, 0 failed）
- `milestone_has_production3`
- `milestone_has_v55`

**実績**: COMPLETE — 3201 tests passed, 0 failed（2026-07-23）

---

### v54.9.0 — v55.0 前調整・安定化

コードフリーズ。全 lint / clippy クリーン確認。`site/content/docs/production3-overview.mdx` を完成させる。
`cargo test` 全通過を確認して v55.0 へ。

**完了条件**: Rust テスト 2 件（ベース 3201 + 2 = 3203 tests passed, 0 failed）
- `cargo_toml_version_is_54_9_0`
- `production3_overview_doc_complete`

**実績**: COMPLETE — 3203 tests passed, 0 failed（2026-07-23）

---

### v55.0.0 — Production 3.0 宣言 ★クリーンアップ

**宣言文**:

> 「型安全なガード節、スケールする並列パイプライン、
>  保証されたデータ品質、そして考えを助ける開発体験。
>  Favnir はデータエンジニアが現場で選ぶ言語になった。
>
>  これが Favnir v55.0 — Production 3.0 の姿である。」

**完了条件**:
- v54.1〜v54.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3201**）
- `v55000_tests` 4 件 pass:
  - `cargo_toml_version_is_55_0_0`
  - `changelog_has_v55_0_0`
  - `milestone_has_production3`
  - `readme_mentions_production3`
- `MILESTONE.md` に `"Production 3.0"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

**実績**: COMPLETE — 3206 tests passed, 0 failed（2026-07-23）

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v50.1-v55.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v53.1-v54.0.md`
- 達成宣言: `MILESTONE.md`
