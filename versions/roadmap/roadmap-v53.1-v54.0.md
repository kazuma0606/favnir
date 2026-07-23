# Roadmap v53.1.0 〜 v54.0.0 — Integration Sprint

Date: 2026-07-18
Status: 計画中（v53.0 完了後に開始）

---

## 前提

- 直前完了: v53.0.0「Data Quality & Observability 2.0」（tests ≥ 3157）
- マスターロードマップ: `roadmap-v50.1-v55.0.md`
- 本文書はマスターの v54.0 スプリント部分の詳細版
- **既存機能の扱い**: v51〜v53 で実装した機能（インレイヒント・par 並列・assert_schema 等）を
  統合・連携させるスプリント。新規コマンドより既存機能間の接続が主目的。
  詳細はマスターロードマップ冒頭「既存機能との位置づけ」テーブルを参照。

---

## 目標

v51.0（DX 3.0）・v52.0（Performance & Scale）・v53.0（Data Quality 2.0）の 3 機能を
**統合・連携させ、大規模 E2E デモを完成させる**。
v54.0「Integration Sprint」として宣言する。

---

## バージョン計画

### v53.1.0 — lineage × LSP 統合（リネージをエディタで表示）

`lsp/hover.rs` のホバー応答に lineage 情報（upstream / downstream stage・スキーマ名）を追加。
`lineage.rs` の `lineage_analysis` 結果を LSP サーバー（`CheckedDoc.lineage`）にキャッシュし、ホバー時に参照。
（注: 旧記述 `lsp/references.rs` / `collect_lineage` は誤記 — 実際のホバー実装は `hover.rs`、関数名は `lineage_analysis`）

```
// Validate stage にホバーした場合
stage Validate
  type:       Order -> Result<Order>
  upstream:   Parse
  downstream: snowflake.insert
  effects:    !Snowflake(write)
  schema:     OrderRow
```

**完了条件**: Rust テスト 2 件（実績推定 3159 tests passed, 0 failed）
- `lsp_hover_shows_lineage`
- `lsp_hover_lineage_upstream`

**実績**: COMPLETE — 3165 tests passed（2026-07-22）

---

### v53.2.0 — bench × par 統合（par stage 個別計測）

`fav bench` が `par` ブロック内の各 stage を個別に計測し最遅 stage（ボトルネック）を明示。
`benchmarks/<version>.json` のスキーマに `par_stages` フィールドを追加。

```bash
$ fav bench pipeline.fav
par.Enrich:   12.3ms
par.Validate: 18.7ms  ← ボトルネック
par.total:    18.9ms  (limited by Validate)
```

**完了条件**: Rust テスト 2 件（実績推定 3161 tests passed, 0 failed）
- `bench_par_stage_individual`
- `bench_par_stage_total`

**実績**: COMPLETE — 3167 tests passed（2026-07-22）
ベース 3165（v53.1.0 完了時実績）+ 2 件 = 3167。推定値 3161 との差 +6 は v53.1.0 コードレビューで +3 件追加されたことに起因。

---

### v53.3.0 — DX × DQ 統合（`assert_schema` 失敗時の詳細 suggestion）

`assert_schema` 失敗時に v50.1〜v50.2 で整備した統一診断フォーマット（span + suggestion）を適用。
`fav explain --error E0419` でフィールド差分の詳細説明を参照できるよう `EXPLAIN_CATALOG` を更新。

```
E0419: schema validation failed for OrderRow
  --> src/pipeline.fav:8:3
  expected: { id: Int, amount: Float, status: String }
  got:      { id: "abc", amount: 99.0, status: "ok" }
  field `id`: expected Int, got String
  help: use Int.parse(row["id"]) to convert
```

**完了条件**: Rust テスト 2 件（実績推定 3163 tests passed, 0 failed）
- `assert_schema_error_has_suggestion`
- `assert_schema_diff_shown`

**実績**: COMPLETE — 3169 tests passed（2026-07-22）
ベース 3167（v53.2.0 完了時実績）+ 2 件 = 3169。推定値 3163 との差 +6 は v53.1.0 コードレビューで +3 件追加されたことに起因（v53.2.0 完了時点と同じ累積差）。

---

### v53.4.0 — E2E 統合デモ Phase 1（Kafka → par transform → Snowflake）

`examples/v55-demo/` ディレクトリに大規模パイプラインデモを作成。
`par` 並列 stage・新 import 構文を活用。

```favnir
// examples/v55-demo/pipeline.fav
import kafka
import snowflake
import "./stages/enrich" as enrich
import "./stages/validate" as validate

pipeline OrderIngestion {
  stage Consume: Stream<RawOrder> -> Stream<Order> = |_raw| {
    bind order <- kafka.consume("orders")
    Ok(order)
  }
  stage Process: Order -> Result<EnrichedOrder> = |order| {
    par [enrich.run(order), validate.run(order)] |> Merge.ordered
  }
  stage Store: EnrichedOrder -> Unit = |enriched| {
    bind _ <- snowflake.insert("orders_v2", enriched)
    Ok(Unit)
  }
}
```

**完了条件**: Rust テスト 2 件（実績推定 3171 tests passed, 0 failed）
- `e2e_integration_demo_structure`
- `e2e_integration_demo_uses_par`

**実績**: COMPLETE — 3171 tests passed（2026-07-22）
ベース 3169（v53.3.0 完了時実績）+ 2 件 = 3171。

---

### v53.5.0 — E2E 統合デモ Phase 2（assert_schema + audit-log + OTel）

E2E デモに `assert_schema`・`--audit-log` 対応・OTel span を統合。デモ用 `run.sh` を更新。

```favnir
stage SchemaCheck: Order -> Result<ValidOrder> = |order| {
  bind checked <- assert_schema<ValidOrder>(order)
  Ok(checked)
  // fav run --audit-log 実行時: アクセスログに自動記録
  // OTel span に schema.name = "ValidOrder" が付与
}
```

**完了条件**: Rust テスト 2 件（実績推定 3167 tests passed, 0 failed）
- `e2e_integration_demo_has_schema`
- `e2e_integration_demo_has_audit_log`

**実績**: COMPLETE — 3173 tests passed（2026-07-22）
ベース 3171（v53.4.0 完了時実績）+ 2 件 = 3173。推定値 3167 との差 +6 は v53.1.0 コードレビュー起因累積差（+4）+ 今回追加（+2）。

---

### v53.6.0 — cookbook 更新

`site/content/cookbook/parallel-pipeline.mdx` — `par` stage・マージモードのレシピ（v24.7.0 で作成済み、今回はテスト追加のみ）。
`site/content/cookbook/schema-validation.mdx` — `assert_schema` + nullable + OTel のレシピ。

**完了条件**: Rust テスト 2 件（実績推定 3169 tests passed, 0 failed）
- `cookbook_parallel_pipeline_exists`
- `cookbook_schema_validation_exists`

**実績**: COMPLETE — 3175 tests passed（2026-07-22）
ベース 3173（v53.5.0 完了時実績）+ 2 件 = 3175。推定値 3169 との差 +6 は累積差 +4（v53.1.0 コードレビュー起因）+ 今回追加 +2。

---

### v53.7.0 — ドキュメントサイト全体最終チェック

全 MDX ページのリンク切れ修正・用語統一（`rune` / `stage` / `pipeline` の表記統一）。
`site/content/docs/glossary.mdx` を v51〜v53 新語彙で更新。

**完了条件**: Rust テスト 2 件（実績推定 3171 tests passed, 0 failed）
- `docs_no_broken_links`
- `docs_glossary_updated`

**実績**: COMPLETE — 3177 tests passed（2026-07-22）
ベース 3175（v53.6.0 完了時実績）+ 2 件 = 3177。推定値 3171 との差 +6 は累積差（v53.1.0 コードレビュー起因 +4 + 今回追加 +2）。

---

### v53.8.0 — CHANGELOG / MILESTONE 整理（v51〜v53 まとめ）

CHANGELOG の v51.0〜v53.0 エントリを整理・補完。MILESTONE.md に v51〜v53 の達成サマリーを追記。

**完了条件**: Rust テスト 2 件（実績推定 3173 tests passed, 0 failed）
- `changelog_has_v51_to_v53_summary`
- `milestone_integration_sprint_noted`

**実績**: COMPLETE — 3179 tests passed（2026-07-22）
ベース 3177（v53.7.0 完了時実績）+ 2 件 = 3179。推定値 3173 との差 +6 は累積差（v53.1.0 コードレビュー起因 +4 + 今回追加 +2）。

---

### v53.9.0 — 安定化・コードフリーズ（Integration Sprint 前調整）

全 lint / clippy クリーン確認。`site/content/docs/integration-overview.mdx` 骨子作成。

**完了条件**: Rust テスト 2 件（実績推定 3175 tests passed, 0 failed）
- `cargo_toml_version_is_53_9_0`
- `integration_overview_doc_exists`

**実績**: COMPLETE — 3181 tests passed（2026-07-22）
ベース 3179（v53.8.0 完了時実績）+ 2 件 = 3181。推定値 3175 との差 +6 は累積差（v53.1.0 コードレビュー起因 +4 + 今回追加 +2）。

---

### v54.0.0 — Integration Sprint 宣言 ★クリーンアップ

**宣言文**:

> 「エディタはデータの来歴を示し、並列パイプラインの性能は
>  計測可能で、スキーマ違反は即座に修正できる。
>  Favnir の 3 つの柱が一体となった。
>
>  これが Favnir v54.0 — Integration の姿である。」

**完了条件**:
- v53.1〜v53.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3179**）
- `v54000_tests` 4 件 pass:
  - `cargo_toml_version_is_54_0_0`
  - `changelog_has_v54_0_0`
  - `milestone_has_integration_sprint`
  - `readme_mentions_integration_sprint`
- `MILESTONE.md` に `"Integration Sprint"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

**実績**: COMPLETE — 3185 tests passed（2026-07-22）
ベース 3181（v53.9.0 完了時実績）+ 4 件 = 3185。テスト数 ≥ 3179 の条件を満たす。
`cargo clean`（26895 ファイル / 26.9 GiB 削除）完了後も全テスト通過を確認。

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v50.1-v55.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v52.1-v53.0.md`
- 達成宣言: `MILESTONE.md`
