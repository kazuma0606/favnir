# Roadmap v73.1.0 〜 v74.0.0 — Production Proven 宣言

Date: 2026-08-08
Status: 未着手（v73.0.0 完了後に開始）

マスターロードマップ: [roadmap-v70.1-v75.0.md](roadmap-v70.1-v75.0.md)

---

## 前提

- 直前完了: v73.0.0「Developer Experience 2.0」（tests = 3646）
- 本スプリントは Phase 4「Production Proven」の詳細計画
- 目標: v74.0.0「Production Proven 宣言」（tests = 3668）

### スプリントの性格

Phase 4 は「実際のチームが本番で Favnir を動かしている」を実証するスプリントである。
データコントラクト・品質スコアリング・PII 保護・SLA 監視——
企業のデータパイプラインが要求する非機能要件を整備する。
さらに Paper Rune を実装に昇格させ、ドッグフーディングで実証する。
C（実証・品質）85% + B（機能完成）15% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v73.1.0 | データコントラクト | 3646 + 2 = 3648 | 未着手 |
| v73.2.0 | データ品質スコアリング（`fav quality`） | 3648 + 2 = 3650 | 未着手 |
| v73.3.0 | PII 検出・マスキング Rune（`Rune.privacy`） | 3650 + 3 = 3653 | 完了 |
| v73.4.0 | 監査ログ + OpenLineage エクスポート | 3653 + 2 = 3655 | 未着手 |
| v73.5.0 | SLA 監視 + アラート統合 | 3655 + 2 = 3657 | 未着手 |
| v73.6.0 | Rune 品質パス（スタブ実装 → VM primitive 接続） | 3657 + 2 = 3659 | 未着手 |
| v73.7.0 | ドッグフーディング Sprint（Favnir で Favnir を運用） | 3659 + 2 = 3661 | 未着手 |
| v73.8.0 | GitHub Actions 公式 Action | 3661 + 2 = 3663 | 未着手 |
| v73.9.0 | 安定化・コードフリーズ（Production Proven 前調整） | 3663 + 2 = 3665 | 未着手 |
| v74.0.0 | Production Proven 宣言 ★クリーンアップ | 3665 + 4 = 3669 | 未着手 |

---

## v73.1.0 — データコントラクト

パイプラインのステージ境界にスキーマ・SLA・品質条件を宣言する。
違反はコンパイル時（スキーマ不一致）または実行時（SLA 超過）に検出される。

```favnir
// データコントラクトの宣言
contract OrderPipelineContract {
    input: {
        order_id: String where String.length(self) > 0
        amount:   PositiveFloat
        status:   "pending" | "paid" | "cancelled"
    }
    output: {
        inserted: Int where self >= 0
        skipped:  Int where self >= 0
    }
    sla: {
        max_latency_ms:  5000
        min_throughput:  1000  // rows/sec
        max_error_rate:  0.01  // 1%
    }
    quality: {
        min_completeness: 0.99
        max_null_ratio:   0.01
    }
}

// コントラクトをステージに適用
stage ProcessOrders: OrderPipelineContract.Input -> OrderPipelineContract.Output = |rows| {
    // 違反は実行前にコンパイルエラーまたは実行時例外
    ...
}
```

**実装内容:**
- `contract` キーワードのパース・AST 追加
- checker: コントラクトのスキーマ境界をステージ型と照合
- VM: 実行時 SLA / 品質条件の監視フック

**完了条件**: Rust テスト 2 件（3646 + 2 = 3648）
- `data_contract_schema_mismatch_error`
- `data_contract_sla_monitoring`

---

## v73.2.0 — データ品質スコアリング（`fav quality`）

```bash
$ fav quality report pipeline.fav --input data.csv
Favnir Data Quality Report
==========================
Overall Score: 87/100

Dimension        Score   Detail
──────────────── ─────── ────────────────────────────────
Completeness      94%    58/1000 rows have null fields
Validity          89%    112 schema violations (amount < 0)
Consistency       78%    220 potential duplicates
Freshness         92%    8% of records older than 24h
Referential       95%    52 orphaned foreign keys

Recommendations:
  1. Add `where self > 0.0` constraint to `amount` field
  2. Enable dedup stage: fav add rune dedup
  3. Add freshness filter: Filter.by_age(max_hours: 24)
```

**実装内容:**
- `cmd_quality_report(path, input)` — completeness / validity / consistency / freshness / referential の 5 次元スコアリング
- `--min-score <n> --fail-below` フラグ（CI 品質ゲート用）
- スコアに基づく推奨アクション生成

**完了条件**: Rust テスト 2 件（3648 + 2 = 3650）
- `quality_report_completeness_score`
- `quality_report_recommendations`

---

## v73.3.0 — PII 検出・マスキング Rune（`Rune.privacy`）

```favnir
import rune "privacy"

// PII フィールドを自動検出してマスク
stage MaskPII: CustomerRecord -> CustomerRecord = |r| {
    Rune.privacy.mask(r, strategy: Hash, fields: ["email", "phone", "ssn"])
}

// 正規表現ベースの PII スキャン
stage ScanPII: String -> PiiReport = |text| {
    Rune.privacy.scan(text, rules: [EmailPattern, PhonePattern, CreditCardPattern])
}

// GDPR 削除要求対応
fn handle_erasure_request(ctx: AppCtx, user_id: UserId) -> Result<Unit, String> {
    Rune.privacy.gdpr_erase(ctx, user_id, tables: ["orders", "sessions", "events"])
}
```

**実装内容:**
- `runes/privacy/privacy.fav` — mask / scan / gdpr_erase の Favnir スタブ実装
- VM primitive 接続（v73.6.0 で実装予定）
- `rune.toml` と統合テスト

**完了条件**: Rust テスト 3 件（3650 + 3 = 3653）
- `privacy_rune_mask_pii_fields`
- `privacy_rune_gdpr_erase`
- `truncate_boundary_values`（コードレビュー指摘によりバグ修正と同時に追加）

---

## v73.4.0 — 監査ログ + OpenLineage エクスポート

```bash
# すべてのパイプライン実行を追跡
$ fav run pipeline.fav --audit-log audit.jsonl

# OpenLineage 形式でエクスポート
$ fav lineage --export openlineage --output lineage.json
# → Marquez / DataHub / OpenMetadata に送信可能
```

**実装内容:**
- `--audit-log <path>` フラグ: 実行開始・完了・エラーを JSONL に記録
- `fav lineage --export openlineage`: 静的リネージ解析 → OpenLineage JSON
- 実行ごとの runId・parentRunId でパイプライン系譜を追跡

**完了条件**: Rust テスト 2 件（3653 + 2 = 3655）
- `audit_log_records_run_start_end`
- `lineage_export_openlineage_format`

---

## v73.5.0 — SLA 監視 + アラート統合

```toml
# fav.toml
[sla]
max_latency_ms   = 5000
min_throughput   = 1000
max_error_rate   = 0.01

[sla.alerts]
slack     = "https://hooks.slack.com/..."
pagerduty = "${PAGERDUTY_KEY}"
```

```bash
$ fav run pipeline.fav --enforce-sla
[SLA] Latency: 4823ms (< 5000ms OK)
[SLA] Throughput: 1243 rows/sec (> 1000 OK)
[SLA] Error rate: 0.3% (< 1% OK)
All SLA conditions met.
```

**実装内容:**
- `[sla]` / `[sla.alerts]` セクションのパース（toml.rs 拡張）
- `--enforce-sla` フラグ: 実行中に SLA 条件を監視
- SLA 違反時に Slack / PagerDuty に通知

**完了条件**: Rust テスト 2 件（3655 + 2 = 3657）
- `sla_violation_triggers_alert`
- `sla_toml_config_parsed`

---

## v73.6.0 — Rune 品質パス（スタブ実装 → VM primitive 接続）

`runes/` ディレクトリ内の各 Rune は `.fav` 実装ファイルが存在するものの、
関数本体が VM primitive（`Rune.linalg.dot(...)` 等）を呼び出していない
**スタブ実装**の状態にある。本バージョンで VM primitive を接続し、
実際にデータを処理できる本番品質の実装に昇格させる。

**対象 Rune（優先順）:**

| Rune | 現状 | 実装内容 |
|---|---|---|
| `runes/linalg/` | .fav スタブあり、VM primitive 未接続 | `dot`, `matmul`, `transpose`, `svd` の vm.rs primitive 追加 + .fav 接続 |
| `runes/autodiff/` | .fav スタブあり、VM primitive 未接続 | `grad`, `jacobian` の vm.rs primitive 追加 + .fav 接続 |
| `runes/stats/` | .fav スタブあり、VM primitive 未接続 | `mean`, `std`, `median`, `t_test` の vm.rs primitive 追加 + .fav 接続 |
| `runes/timeseries/` | .fav スタブあり、VM primitive 未接続 | `rolling_mean`, `ewm`, `decompose` の vm.rs primitive 追加 + .fav 接続 |
| `runes/ml/` | .fav スタブあり、VM primitive 未接続 | `knn_predict`, `random_forest_fit` の vm.rs primitive 追加 + .fav 接続 |

各 Rune に統合テストを追加する。

**完了条件**: Rust テスト 2 件（3657 + 2 = 3659）
- `rune_linalg_matmul_runs`
- `rune_stats_mean_std_runs`

---

## v73.7.0 — ドッグフーディング Sprint（Favnir で Favnir を運用）

Favnir 自身の開発ワークフローに Favnir パイプラインを使う実証スプリント。

**実装するパイプライン:**

| パイプライン | 内容 |
|---|---|
| `pipelines/benchmark_analytics.fav` | bench JSON を集計してトレンド可視化 |
| `pipelines/coverage_report.fav` | テストカバレッジ → Slack 通知 |
| `pipelines/changelog_lint.fav` | CHANGELOG.md の形式を検証 |
| `pipelines/rune_catalog_sync.fav` | `runes/` ディレクトリ → catalog.mdx 自動更新 |
| `pipelines/doc_link_check.fav` | MDX ファイルの broken link を検出 |

全パイプラインのスタブ作成と構造検証（`fav run` による実行確認は v73.9.0 で実施）。

**完了条件**: Rust テスト 2 件（3659 + 2 = 3661）
- `dogfooding_benchmark_pipeline_runs`
- `dogfooding_doc_link_check_runs`

---

## v73.8.0 — GitHub Actions 公式 Action

```yaml
# .github/workflows/favnir-ci.yml
steps:
  - uses: favnir/setup-fav@v1
    with:
      version: "75.0.0"

  - name: Type Check
    run: fav check pipeline.fav

  - name: Test
    run: fav test pipeline.fav

  - name: Quality Gate
    run: fav quality report pipeline.fav --min-score 80 --fail-below

  - name: Audit
    run: fav audit --deny-high
```

**実装内容:**
- `.github/actions/setup-fav/` ディレクトリ — `action.yml` / `README.md`
- GitHub Releases から OS 別 fav バイナリをダウンロードしてパスに追加
- 使用例・バッジ・マトリックスビルドサンプルを同梱

**完了条件**: Rust テスト 2 件（3661 + 2 = 3663）
- `github_action_setup_fav_action_yml_valid`
- `github_action_fav_binary_url_format`

---

## v73.9.0 — 安定化・コードフリーズ（Production Proven 前調整）

v73.1〜v73.8 の全機能が正常動作することを確認する安定化バージョン。
ドッグフーディング 5 パイプラインの全 pass を確認する。

**完了条件**: Rust テスト 2 件（3663 + 2 = 3665）
- `production_proven_all_stable`
- `dogfooding_all_5_pipelines_pass`

---

## v74.0.0 — Production Proven 宣言 ★クリーンアップ

**宣言文**:

> 「データコントラクトがスキーマ境界を守り、品質スコアが劣化を警告する。
>  PII が型で保護され、監査ログが法的要件を満たす。
>  Favnir が Favnir 自身を運用し、GitHub Action が CI に溶け込む。
>
>  これが Favnir v74.0 — Production Proven の姿である。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `74.0.0` に更新
- `CHANGELOG.md` に v74.0.0 エントリを追加
- `MILESTONE.md` に「Production Proven」を追記
- `README.md` に v74.0 達成を追記
- `versions/current.md` を更新（進行中 → v74.1.0）

**完了条件**: `v74000_tests` 4 件（3665 + 4 = 3669）
- `cargo_toml_version_is_74_0_0`
- `changelog_has_v74_0_0`
- `milestone_has_production_proven`
- `readme_mentions_production_proven`

---

## テスト数推移（本スプリント）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v73.0.0（ベース） | 3,646 | — |
| v73.1.0 | 3,648 | +2 |
| v73.2.0 | 3,650 | +2 |
| v73.3.0 | 3,653 | +3 |
| v73.4.0 | 3,655 | +2 |
| v73.5.0 | 3,657 | +2 |
| v73.6.0 | 3,659 | +2 |
| v73.7.0 | 3,661 | +2 |
| v73.8.0 | 3,663 | +2 |
| v73.9.0 | 3,665 | +2 |
| v74.0.0（宣言） | 3,669 | +4 |

**本スプリント合計**: +23 tests（3,646 → 3,669）

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v70.1-v75.0.md`
- 前スプリント（完了予定）: `versions/roadmap/roadmap-v72.1-v73.0.md`
- 次スプリント: `versions/roadmap/roadmap-v74.1-v75.0.md`
- 進行状況: `versions/current.md`
