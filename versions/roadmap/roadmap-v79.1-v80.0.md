# Roadmap v79.1.0 〜 v80.0.0 — Favnir 3.0 宣言

Date: 2026-08-14
Status: 未着手（v79.0.0 完了後に開始）

マスターロードマップ: [roadmap-v75.1-v80.0.md](roadmap-v75.1-v80.0.md)

---

## 前提

- 直前完了: v79.0.0「Execution Effects 1.0 宣言」（tests = 3787）
- 本スプリントは Phase 6「Favnir 3.0 宣言」の最終スプリント
- 目標: v80.0.0「Favnir 3.0 宣言」（tests = 3809）

### スプリントの性格

4スプリント（Temporal / Provenance / Verifiable / Execution Effects）を統合し、
Favnir 3.0 として世界に宣言する。新規機能は最小限に抑え、
統合ショーケース・ドッグフーディング・ドキュメント完全化・宣言が中心。
B（統合・磨き上げ）40% + C（宣言・ドキュメント）60% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v79.1.0 | 統合ショーケース基盤（`infra/e2e-demo/favnir3-showcase/`） | 3787 + 2 = 3789 | 完了 |
| v79.2.0 | Temporal showcase パイプライン | 3789 + 2 = 3791 | 未着手 |
| v79.3.0 | Provenance showcase パイプライン | 3791 + 2 = 3793 | 未着手 |
| v79.4.0 | Verifiable showcase パイプライン | 3793 + 2 = 3795 | 未着手 |
| v79.5.0 | Execution Effects showcase パイプライン | 3795 + 2 = 3797 | 未着手 |
| v79.6.0 | ドッグフーディング強化 | 3797 + 2 = 3799 | 未着手 |
| v79.7.0 | OSS 公開強化・コミュニティ整備 | 3799 + 2 = 3801 | 未着手 |
| v79.8.0 | ドキュメント完全化（v3 リファレンス） | 3801 + 2 = 3803 | 未着手 |
| v79.9.0 | 安定化・コードフリーズ（Favnir 3.0 前最終調整） | 3803 + 2 = 3805 | 未着手 |
| v80.0.0 | Favnir 3.0 宣言 ★クリーンアップ | 3805 + 4 = 3809 | 未着手 |

---

## v79.1.0 — 統合ショーケース基盤

4スプリントすべての機能を網羅するショーケースの骨格を作成する。
v74.8.0 の `infra/e2e-demo/favnir2-showcase/` と同構造。

```
infra/e2e-demo/favnir3-showcase/
├── pipeline.fav     # 4スプリント全機能統合パイプライン
├── fav.toml         # !Cached / !Adaptive / !Parallel 設定
├── contract.fav     # Temporal + Provenance + Verifiable コントラクト
└── README.md        # 概要・実行手順
```

**実装内容:**
- `infra/e2e-demo/favnir3-showcase/pipeline.fav` — 骨格（各ステージのプレースホルダ）
- `infra/e2e-demo/favnir3-showcase/fav.toml` — `[effects.cached]` / `[effects.adaptive]` 設定
- `infra/e2e-demo/favnir3-showcase/contract.fav` — `ShowcaseContract3` 宣言
- `infra/e2e-demo/favnir3-showcase/README.md` — 実行手順

**完了条件**: Rust テスト 2 件（3787 + 2 = 3789）
- `favnir3_showcase_structure_exists`
- `favnir3_showcase_contract_valid`

---

## v79.2.0 — Temporal showcase パイプライン

ショーケースの `pipeline.fav` に Temporal 機能を組み込む。

```favnir
// pipeline.fav — Temporal セクション
fn load_with_freshness(ctx: AppCtx) -> Result<List<Row>, String> {
    bind snapshot <- AsOfQuery { table: "orders", as_of_ts: ctx.run_ts }
    bind _        <- FreshnessPolicy.check(snapshot, max_age: Duration.hours(1))
    bind history  <- apply_scd2_update(existing_customers, new_data, ctx.run_ts)
    Result.ok(history)
}
```

**実装内容:**
- `infra/e2e-demo/favnir3-showcase/pipeline.fav` を更新（Temporal ステージ追加）

**完了条件**: Rust テスト 2 件（3789 + 2 = 3791）
- `showcase_temporal_freshness_check`
- `showcase_temporal_scd2_applied`

---

## v79.3.0 — Provenance showcase パイプライン

ショーケースに Provenance 機能（来歴追跡・OpenLineage）を組み込む。

```favnir
// pipeline.fav — Provenance セクション
fn load_with_provenance(ctx: AppCtx, rows: List<Row>) -> Result<TracedData, String> {
    bind source <- DataSource {
        name: "snowflake-crm",
        uri: "snowflake://warehouse/crm/users",
        source_type: Snowflake
    }
    bind raw    <- TracedData.wrap(rows, source)
    bind masked <- raw |> TracedData.map(mask_pii, label: "mask_pii")
    bind facet  <- OpenLineage.from_provenance(masked.provenance)
    Result.ok(masked)
}
```

**実装内容:**
- `infra/e2e-demo/favnir3-showcase/pipeline.fav` を更新（Provenance ステージ追加）

**完了条件**: Rust テスト 2 件（3791 + 2 = 3793）
- `showcase_provenance_traced`
- `showcase_provenance_openlineage_generated`

---

## v79.4.0 — Verifiable showcase パイプライン

ショーケースに不変条件検証を組み込む。

```favnir
// contract.fav — Verifiable セクション
contract Favnir3ShowcaseContract {
    input:     { rows: List<Row> }
    output:    { processed: List<Row> }
    invariant: output.row_count <= input.row_count
    invariant: SUM(output.amount) >= 0.0
    probabilistic_invariant score_dist:
        confidence: 0.95, sample_size: 1000,
        property: AVG(score) BETWEEN 40.0 AND 60.0
}
```

**実装内容:**
- `infra/e2e-demo/favnir3-showcase/contract.fav` を更新（不変条件追加）

**完了条件**: Rust テスト 2 件（3793 + 2 = 3795）
- `showcase_verifiable_invariants_declared`
- `showcase_verifiable_probabilistic_contract`

---

## v79.5.0 — Execution Effects showcase パイプライン

ショーケースに実行戦略エフェクトを組み込む。

```favnir
// pipeline.fav — Execution Effects セクション
fn join_stage(ctx: AppCtx, customers: List<Row>, orders: List<Row>) -> Result<List<Row>, String> !Adaptive !Cached {
    bind joined <- customers |> join(orders, on: "id")
    // !Adaptive → row 数に応じて broadcast/hash を自動選択
    // !Cached   → TTL 内は同じ入力に対してキャッシュを返す
    Result.ok(joined)
}
```

**実装内容:**
- `infra/e2e-demo/favnir3-showcase/pipeline.fav` を更新（Execution Effects ステージ追加）
- `infra/e2e-demo/favnir3-showcase/fav.toml` に `[effects.cached]` / `[effects.adaptive]` 設定を確認

**完了条件**: Rust テスト 2 件（3795 + 2 = 3797）
- `showcase_execution_cached_effect`
- `showcase_execution_adaptive_effect`

---

## v79.6.0 — ドッグフーディング強化

Favnir 自身のリリースパイプラインを Favnir で記述する（セルフホスト精神の継続）。

```
fav/pipelines/
├── release.fav        # バージョンバンプ・CHANGELOG 生成・tag push（v79.6.0 で実装）
└── health-check.fav   # cargo test + fav verify の CI ヘルスチェック（v79.6.0 で実装）
# benchmark.fav は後続スプリントで追加予定
```

**実装内容:**
- `fav/pipelines/release.fav` — バージョン文字列更新・CHANGELOG 先頭挿入のロジック
- `fav/pipelines/health-check.fav` — `fav verify` コマンド呼び出しのラッパー

**完了条件**: Rust テスト 2 件（3797 + 2 = 3799）
- `dogfood_release_pipeline_exists`
- `dogfood_health_check_pipeline_exists`

---

## v79.7.0 — OSS 公開強化・コミュニティ整備

Rune マーケットプレイスと OSS コントリビュートフローを強化する。

```
- CONTRIBUTING.md v2（v3 対応・新エフェクトの追加手順・invariant 追加手順）
- COMMUNITY.md（ディスカッションチャンネル・RFC プロセス）
- .github/CODEOWNERS 更新
- Rune validate ガイド（validate_rune_score 利用手順）
```

**実装内容:**
- `CONTRIBUTING.md` を v3 対応に更新（Execution Effects 追加手順・`fav verify` の使い方）
- `COMMUNITY.md` 新規作成（RFC プロセス・ディスカッション場所）

**完了条件**: Rust テスト 2 件（3799 + 2 = 3801）
- `oss_contributing_v2_exists`
- `oss_community_md_exists`

---

## v79.8.0 — ドキュメント完全化（v3 リファレンス）

4スプリントで追加した全機能のドキュメントを `site/content/docs/v3/` に完全化する。

```
site/content/docs/v3/
├── temporal.mdx          # FreshnessPolicy / AsOfQuery / SCD / time-travel
├── provenance.mdx        # TracedData / ProvenanceTag / lineage graph
├── verifiable.mdx        # PipelineInvariant / fav verify / counter-examples
├── execution-effects.mdx # !Cached / !Adaptive / !Parallel / explain plan
└── migration-v2-v3.mdx   # v2（v75.0）→ v3（v80.0）移行ガイド
```

**実装内容:**
- `site/content/docs/v3/temporal.mdx`
- `site/content/docs/v3/migration-v2-v3.mdx`
- （他 3 ファイルは後続コミットで追加）

**完了条件**: Rust テスト 2 件（3801 + 2 = 3803）
- `docs_v3_temporal_exists`
- `docs_v3_migration_guide_exists`

---

## v79.9.0 — 安定化・コードフリーズ（Favnir 3.0 前最終調整）

v79.1〜v79.8 の全機能・v75.1〜v79.8 の全スプリントを通しで確認する最終安定化スプリント。

**実装内容:**
- v79.1〜v79.8 の全テスト通過確認（`cargo test` 全 pass）
- `infra/e2e-demo/favnir3-showcase/` の E2E ショーケース実行確認
- v75〜v79 の全スプリント統合動作確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（3803 + 2 = 3805）
- `favnir3_full_sprint_all_stable`
- `favnir3_e2e_showcase_runs`

---

## v80.0.0 — Favnir 3.0 宣言 ★クリーンアップ（完了 2026-08-16）

**宣言文**:
> 「時間が型となり、来歴が型となり、正しさが型となり、実行戦略が型となった。
>
>  FreshnessPolicy がデータの鮮度を保証し、ProvenanceTag が来歴を追い、
>  PipelineInvariant が不変条件を証明し、!Adaptive がコストを最適化する。
>
>  Favnir 3.0 は、データパイプラインが「何を・どこから・どう正しく・どう速く」
>  処理するかを、すべて型で語れる言語である。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `80.0.0` に更新
- `CHANGELOG.md` に v80.0.0 エントリを追加
- `MILESTONE.md` に「Favnir 3.0 宣言」を追記
- `README.md` に v80.0 達成（Favnir 3.0）を追記
- `versions/current.md` を更新（v80.0.0 完了・次フェーズ計画へ）
- マスターロードマップの本スプリントを「完了」に更新

**完了条件**: `v80000_tests` 4 件（3805 + 4 = 3809）
- `cargo_toml_version_is_80_0_0`
- `changelog_has_v80_0_0`
- `milestone_has_favnir_3`
- `readme_mentions_favnir_3`

---

## テスト数推移（本スプリント）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v79.0.0（ベース） | 3,787 | —（v79.0.0 宣言時点での実測値）|
| v79.1.0 | 3,789 | +2 |
| v79.2.0 | 3,791 | +2 |
| v79.3.0 | 3,793 | +2 |
| v79.4.0 | 3,795 | +2 |
| v79.5.0 | 3,797 | +2 |
| v79.6.0 | 3,799 | +2 |
| v79.7.0 | 3,801 | +2 |
| v79.8.0 | 3,803 | +2 |
| v79.9.0 | 3,805 | +2 |
| v80.0.0（宣言） | 3,809 | +4 |

**本スプリント合計**: +22 tests（3,787 → 3,809）

---

## v75.1〜v80.0 全スプリント総括

| スプリント | 期間 | テーマ | テスト増 | 到達値 |
|---|---|---|---|---|
| Temporal Data Native | v75.1〜v76.0 | 時間軸型 | +22 | 3,714 |
| Data Provenance 1.0 | v76.1〜v77.0 | 来歴型 | +22 | 3,736 |
| Verifiable Pipelines | v77.1〜v78.0 | 証明可能型 | +22 | 3,758 |
| Execution Effects 1.0 | v78.1〜v79.0 | 実行戦略エフェクト | +27 | 3,787 |
| **Favnir 3.0 宣言** | **v79.1〜v80.0** | **統合・宣言** | **+22** | **3,809** |
| **合計** | | | **+115** | **3,809** |

---

## 参考リンク

- マスターロードマップ: [roadmap-v75.1-v80.0.md](roadmap-v75.1-v80.0.md)
- 前スプリント: [roadmap-v78.1-v79.0.md](roadmap-v78.1-v79.0.md)
- 次フェーズ: （未計画 — v80.0.0 宣言後に策定）
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
