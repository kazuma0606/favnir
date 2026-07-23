# Roadmap v52.1.0 〜 v53.0.0 — Data Quality & Observability 2.0

Date: 2026-07-18
Status: 計画中（v52.0 完了後に開始）

---

## 前提

- 直前完了: v52.0.0「Performance & Scale」（tests ≥ 3135）
- マスターロードマップ: `roadmap-v50.1-v55.0.md`
- 本文書はマスターの v53.0 スプリント部分の詳細版
- **既存機能の扱い**: `fav explain --lineage --format mermaid/dot/svg` は既存実装済み
  （`main.rs` `cmd_explain_lineage`）。v52.3〜v52.4 は「新コマンド追加」ではなく「拡張」。
  `fav audit`（`main.rs` `Some("audit")`）は Enterprise Governance 用として別途存在するため、
  データアクセスログは `fav run --audit-log` として実装する（名称衝突回避）。
  詳細はマスターロードマップ冒頭「既存機能との位置づけ」テーブルを参照。

---

## 目標

v52.0 で並列実行・スケールを確立した。このスプリントでは
**「スキーマ検証・リネージ可視化・SLA 監視・アクセスログ」**
を実装して Favnir v53.0 を宣言する。

---

## バージョン計画

### v52.1.0 — `assert_schema` Phase 1（型チェック）

`assert_schema<T>(value)` を VM primitive として追加。`T` のフィールド名・型を実行時に検証し、
不一致時 `Err` を返す。`ast.rs` / `compiler.rs` に `AssertSchema` ノードを追加。E0419 エラーコードを追加。

```favnir
type OrderRow = { id: Int, amount: Float, status: String }

stage ValidateSchema: Map<String, Any> -> Result<OrderRow> = |row| {
  bind validated <- assert_schema<OrderRow>(row)
  Ok(validated)
}
```

**完了条件**: Rust テスト 2 件（実績推定 3137 tests passed, 0 failed）
- `assert_schema_type_ok`
- `assert_schema_type_fail`

**実績（2026-07-20 COMPLETE）**: 3136 tests passed, 0 failed — `Expr::AssertSchema` / `IRExpr::AssertSchema` / `Opcode::AssertSchema = 0x64` / VM ハンドラ追加、E0419「assert_schema type mismatch」定義、全 exhaustive match サイト対応（wasm_dce / wasm_codegen / fmt / lint / emit_python / checker / compiler / lineage / references / driver）
- `assert_schema_type_ok`
- `assert_schema_type_fail`

---

### v52.2.0 — `assert_schema` Phase 2（nullable・追加フィールド対応）

nullable フィールド（`field?: Type`）を `assert_schema` で許容。想定外フィールドを W036 警告として追加。
`--strict-schema` フラグで W036 をエラー化する。

```favnir
type FlexRow = { id: Int, name: String, note?: String }

// note フィールドが存在しなくても OK（nullable）
// 想定外フィールドがあれば W036 警告（--strict-schema でエラー化）
bind row <- assert_schema<FlexRow>(raw_map)
```

**完了条件**: Rust テスト 2 件（実績推定 3138 tests passed, 0 failed）
- `assert_schema_nullable_field`
- `assert_schema_extra_field_warn`

**実績（2026-07-20 COMPLETE）**: 3138 tests passed, 0 failed — `FieldMeta.optional` 追加、TMET bit-flag 方式（bit1=optional）後方互換更新、`Vm.strict_schema` フラグ追加、`AssertSchema` ハンドラ更新（nullable 許容 + W036 警告）、`--strict-schema` CLI フラグ追加、`lint.rs` W036 スタブ登録
- `assert_schema_nullable_field`
- `assert_schema_extra_field_warn`

---

### v52.3.0 — `fav explain --lineage` 表示強化（スキーマ情報付加）

既存の `fav explain --lineage --format mermaid/dot/svg`（`main.rs` `cmd_explain_lineage` として実装済み）に
`--with-schema` オプションを追加。`assert_schema` で検証されたスキーマ名をノードラベルに表示。

```bash
$ fav explain --lineage pipeline.fav --format mermaid --with-schema
```

```
flowchart LR
  kafka.consume["kafka.consume\nStream<RawOrder>"] --> Parse["Parse\nRawOrder → Order"]
  Parse --> Validate["Validate\nOrder → Result<Order>"]
  Validate --> snowflake.insert["snowflake.insert\n!Snowflake(write)"]
```

**完了条件**: Rust テスト 2 件（実績推定 3141 tests passed, 0 failed）
- `lineage_mermaid_with_schema`
- `lineage_dot_with_schema`

**実績（2026-07-21 COMPLETE）**: 3141 tests passed, 0 failed — `LineageEntry.schema: Option<String>` 追加、`collect_assert_schema_name`（Apply(TypeApply(Ident("assert_schema"), ..)) パターン検出を含む exhaustive 実装）/ `_stmt` / `_block` 実装、`lineage_analysis` でスキーマ収集、`render_lineage_mermaid_with_schema` / `render_lineage_dot_with_schema` 追加、`cmd_explain_lineage` に `with_schema: bool` 引数追加、`main.rs` に `--with-schema` フラグ追加、動作テスト `lineage_schema_field_detected_from_assert_schema` 追加
- `lineage_mermaid_with_schema`
- `lineage_dot_with_schema`

---

### v52.4.0 — `fav explain --lineage` インタラクティブ HTML レポート

`--format html` オプションを追加。依存グラフを SVG でレンダリングし、クリックで stage 詳細
（型・エフェクト・スキーマ）を表示できる自己完結型 HTML を生成。

```bash
$ fav explain --lineage pipeline.fav --format html -o lineage.html
# → ブラウザで開けるインタラクティブな SVG + テーブル
```

**完了条件**: Rust テスト 2 件（実績推定 3143 tests passed, 0 failed）
- `lineage_html_output`
- `lineage_html_has_stage_detail`

**実績（2026-07-21 COMPLETE）**: 3144 tests passed, 0 failed — `render_lineage_html` 追加（クリック可能 SVG + JS `showDetail` + `<div id="detail">` パネル）、`cmd_explain_lineage` に `output: Option<&str>` 引数追加、`--format html` アーム追加、`main.rs` に `-o <file>` フラグ追加、`v52400_tests` 3 件追加
- `lineage_html_output`
- `lineage_html_has_stage_detail`
- `lineage_html_renders_stage_node`

---

### v52.5.0 — SLA 監視 Rune

`runes/sla/sla.fav` に `check_freshness` / `check_latency` / `alert` 関数を追加。
SLA 違反時は `!Observe` エフェクト経由でアラートを発火し `Err` を返す。

```favnir
import sla

stage CheckFreshness: DataBatch -> Result<DataBatch> = |batch| {
  bind _ <- sla.check_freshness(batch.timestamp, max_age_seconds: 3600)
  bind _ <- sla.check_latency(stage: "Parse", threshold_ms: 200)
  Ok(batch)
}
```

**完了条件**: Rust テスト 2 件（実績推定 3145 tests passed, 0 failed）
- `sla_rune_latency_check`
- `sla_rune_freshness_check`

**実績（2026-07-21 COMPLETE）**: 3147 tests passed, 0 failed（推定 3145 から +2 補正: v52.4.0 実績が 3144 + code-reviewer 指摘で `sla_rune_alert_check` 追加）— `runes/sla/sla.fav` 新規作成（`check_freshness` / `check_latency` / `alert` の 3 関数、`Sla.*_raw` スタブパターン）、`!Observe` エフェクトはコメントのみ（スタブ実装、Effect enum 追加は将来バージョン）、`v52500_tests` 3 件追加
- `sla_rune_latency_check`
- `sla_rune_freshness_check`
- `sla_rune_alert_check`

---

### v52.6.0 — `fav run --audit-log` データアクセスログ

`fav run --audit-log <output.jsonl>` オプションを追加（既存の `fav audit`（`main.rs` `Some("audit")`・
`fav_audit::cmd_audit`）は Enterprise Governance 用であり独立して継続）。
`!Kafka` / `!Snowflake` / `!S3` のアクセスイベントを JSONL 形式で記録。VM の effect ディスパッチフックに挿入。

```bash
$ fav run pipeline.fav --audit-log audit.jsonl
```

```json
{"ts":"2026-07-18T10:00:00Z","op":"read","effect":"Kafka","topic":"orders"}
{"ts":"2026-07-18T10:00:01Z","op":"write","effect":"Snowflake","table":"orders_v2"}
```

**完了条件**: Rust テスト 2 件（実績推定 3147 tests passed, 0 failed）
- `audit_log_read_event`
- `audit_log_write_event`

**実績（2026-07-21 COMPLETE）**: 3149 tests passed, 0 failed（推定 3147 から +2 補正: v52.5.0 実績が 3147 だったため）— `AUDIT_LOG_PATH` thread-local（`RefCell<Option<String>>`）追加、`set_audit_log_path` / `append_audit_event` 追加（wasm32 除外）、`Kafka.produce_raw` / `Kafka.consume_one_raw` / `Snowflake.execute_raw` アームにフック挿入、`cmd_run` シグネチャに `audit_log: Option<&str>` 追加、`--audit-log` CLI フラグ追加。`!S3` は vm.rs に `"S3.*_raw"` アームが存在しないため本バージョンのスコープ外。
- `audit_log_read_event`
- `audit_log_write_event`

---

### v52.7.0 — OTel 強化（span 属性にスキーマ・リネージ情報付加）

OTel の stage span に `schema.name` / `schema.fields` / `lineage.upstream` / `lineage.downstream` 属性を付与。
`assert_schema` 呼び出し時にスキーマ名を span コンテキストに記録。

```
span: stage.Validate
  schema.name        = "OrderRow"
  schema.fields      = "id,amount,status"
  lineage.upstream   = "Parse"
  lineage.downstream = "snowflake.insert"
```

**完了条件**: Rust テスト 2 件（実績推定 3149 tests passed, 0 failed）
- `otel_span_has_schema_attr`
- `otel_span_has_lineage_attr`

**実績（2026-07-21 COMPLETE）**: 3151 tests passed, 0 failed（推定 3149 から +2 補正: v52.6.0 実績が 3149 だったため）— `OtelSpan.attrs: Vec<(String, String)>` 追加、`otel_add_attr` / `otel_patch_attr_on_last` 追加、`build_otlp_json` / `otel_export_stdout` 更新、`OTEL_PREV_STAGE` thread-local + `reset_stage_lineage` 追加、`SeqStageEnter` に lineage フック（`lineage.upstream` / `lineage.downstream`）、`AssertSchema` 成功時に `schema.name` / `schema.fields` 付与。
- `otel_span_has_schema_attr`
- `otel_span_has_lineage_attr`

---

### v52.8.0 — ドキュメントサイト Data Quality 記事

`site/content/docs/data-quality/assert-schema.mdx` — `assert_schema` の使い方・nullable・strict モード。
`site/content/docs/tools/lineage-enhanced.mdx` — `--with-schema` / `--format html` の使い方。
`site/content/docs/tools/audit-log.mdx` — `fav run --audit-log` の使い方・JSONL フォーマット。

**完了条件**: Rust テスト 2 件（実績推定 3153 tests passed, 0 failed）
- `docs_assert_schema_page_exists`
- `docs_audit_log_page_exists`

**実績（2026-07-22 COMPLETE）**: 3154 tests passed, 0 failed（推定 3153 から +1 補正: code-reviewer 指摘で `docs_lineage_enhanced_page_exists` 追加 + `E0419` アサート追加）— `site/content/docs/data-quality/assert-schema.mdx` 新規作成（nullable/strict-schema/E0419 説明）、`site/content/docs/tools/lineage-enhanced.mdx` 新規作成（--with-schema/--format html/-o オプション）、`site/content/docs/tools/audit-log.mdx` 新規作成（JSONL フォーマット・fav audit との違い明示）、`v52800_tests` 3 件追加
- `docs_assert_schema_page_exists`
- `docs_audit_log_page_exists`
- `docs_lineage_enhanced_page_exists`

---

### v52.9.0 — 安定化・コードフリーズ（Data Quality 2.0 前調整）

全 lint / clippy クリーン確認。`site/content/docs/data-quality-overview.mdx` 骨子作成。

**完了条件**: Rust テスト 2 件（実績推定 3156 tests passed, 0 failed）
- `cargo_toml_version_is_52_9_0`
- `dq_overview_doc_exists`

**実績（2026-07-22 COMPLETE）**: 3156 tests passed, 0 failed — `site/content/docs/data-quality-overview.mdx` 新規作成（v52.1〜v52.8 機能一覧・assert_schema/audit-log 言及）、`cargo clippy -- -D warnings` クリーン確認、`v52900_tests` 2 件追加
- `cargo_toml_version_is_52_9_0`
- `dq_overview_doc_exists`

---

### v53.0.0 — Data Quality & Observability 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「スキーマはランタイムで検証され、データの来歴はグラフで見え、
>  SLA 違反は即座に検知され、アクセスはすべて記録される。
>  Favnir のパイプラインは信頼できるデータを届ける。
>
>  これが Favnir v53.0 — Data Quality & Observability 2.0 の姿である。」

**完了条件**:
- v52.1〜v52.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3157**）
- `v53000_tests` 4 件 pass:
  - `cargo_toml_version_is_53_0_0`
  - `changelog_has_v53_0_0`
  - `milestone_has_data_quality`
  - `readme_mentions_data_quality`
- `MILESTONE.md` に `"Data Quality & Observability 2.0"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

**実績（2026-07-22 COMPLETE）**: 3160 tests passed, 0 failed（≥ 3157 ✓）— `MILESTONE.md` に v53.0.0「Data Quality & Observability 2.0」エントリ追加（宣言文付き）、`README.md` に v53.0 言及追加、`v53000_tests` 4 件追加、`v52900_tests::cargo_toml_version_is_52_9_0` を空化（バージョンバンプ対応）、`cargo clean` 完了（33.5GiB 削除、hello.fav は target 外のため影響なし）
- `cargo_toml_version_is_53_0_0`
- `changelog_has_v53_0_0`
- `milestone_has_data_quality`
- `readme_mentions_data_quality`

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v50.1-v55.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v51.1-v52.0.md`
- 達成宣言: `MILESTONE.md`
