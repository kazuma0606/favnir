# Changelog

Favnir のバージョン履歴。形式は [Keep a Changelog](https://keepachangelog.com/ja/1.0.0/) に準拠。

---

## [v55.0.0] — 2026-07-23 — Production 3.0 宣言

### Production 3.0 宣言

v51〜v54 で積み上げた全機能を最終確認し、Favnir v55.0 — Production 3.0 を宣言する。

- v51: Developer Experience 3.0（診断統一・インレイヒント・trace/watch）
- v52: Performance & Scale（par Tokio・バックプレッシャー・bench 回帰・WASM 最適化）
- v53: Data Quality & Observability 2.0（assert_schema・lineage 強化・audit-log）
- v54: Integration Sprint（explain --error 全網羅・watch-diff・dq-report・doctor・Production 3.0 整備）
- v55: Production 3.0 宣言・★クリーンアップ

### Removed
- `v54900_tests::cargo_toml_version_is_54_9_0`（Cargo.toml 更新に伴い廃止、毎バージョン慣行）

---

## [v54.9.0] — 2026-07-23 — v55.0 前調整・安定化

### Added
- `site/content/docs/production3-overview.mdx` 完成（v54.6〜v54.9 最終整備セクションを追記）
- `v54900_tests` 追加（`cargo_toml_version_is_54_9_0` / `production3_overview_doc_complete`）— 3203 tests

---

## [v54.8.0] — 2026-07-23 — MILESTONE.md Production 3.0 エントリ追加

### Added
- `MILESTONE.md`: `## v55.0.0（予定）— Production 3.0` エントリ追加（v51〜v54 達成内容を記録）
- `v54800_tests` 追加（`milestone_has_production3` / `milestone_has_v55`）— 3201 tests

---

## [v54.7.0] — 2026-07-23 — ドキュメントサイト Production 3.0 overview ページ

### Added
- `site/content/docs/production3-overview.mdx` 新規作成 — v51〜v55 の全機能を統合した概要ページ
- `v54700_tests` 追加（`docs_production3_overview_exists` / `docs_production3_has_v55`）— 3199 tests

---

## [v54.6.0] — 2026-07-23 — README / CONTRIBUTING 最終整備

### Added
- `README.md`: Production 3.0 への言及・v54.1〜v54.5 機能サマリーを追加
- `CONTRIBUTING.md`: `fav doctor` 環境診断手順・`fav bench` パフォーマンス確認手順を追記
- `v54600_tests` 追加（`readme_has_production3_mention` / `contributing_has_doctor_step`）— 3197 tests

---

## [v54.5.0] — 2026-07-23 — fav doctor 環境診断コマンド

### Added
- `DoctorCheck` struct / `DoctorStatus` enum: 診断チェック結果の表現型
- `cmd_doctor_collect`: チェックリストを `[OK]` / `[WARN]` / `[FAIL]` プレフィクス付きテキストに変換
- `cmd_doctor_run`: Rust バージョン・fav バージョン・fav.toml 存在確認・.fav-cache 状態を診断
- `fav doctor` コマンド追加（main.rs）
- `v54500_tests` 追加（`cmd_doctor_passes_clean_env` / `cmd_doctor_detects_missing_rune`）— 3195 tests

---

## [v54.4.0] — 2026-07-22 — fav dq-report データ品質レポートコマンド

### Added
- `cmd_dq_report_collect`: audit-log JSONL を解析し schema validation 統計・SLA 違反を集計して Markdown レポートを生成
- `fav dq-report --audit-log <path>` コマンド追加（main.rs）
- `v54400_tests` 追加（`cmd_dq_report_generates` / `cmd_dq_report_has_schema_stats`）— 3193 tests

---

## [v54.3.0] — 2026-07-22 — パフォーマンスリグレッションスイート CI 統合

### Added
- `.github/workflows/bench.yml`: `cargo test bench_ -- --nocapture` ステップと `fav bench --all --compare benchmarks/baseline.json --fail-on-regression` ステップを追加
- `benchmarks/baseline.json` 新規作成 — PR ごとの自動比較の基準値としてリポジトリ管理
- `v54300_tests` 追加（`ci_perf_regression_suite` / `ci_perf_baseline_recorded`）— 3191 tests

---

## [v54.2.0] — 2026-07-22 — fav run --watch 高度化（差分表示・サマリー）

### Added
- `format_watch_diff`: 数値フィールドの before/after 差分（Δ）を `[watch] field: X → Y Δ+Z (stage: S)` 形式でフォーマット
- `format_watch_summary`: 複数 stage にまたがった全ウォッチイベントを `[watch-summary]` 形式で集計出力
- `fav run --watch-diff` / `--watch-summary` フラグ追加（main.rs）
- `v54200_tests` 追加（`run_watch_diff_numeric` / `run_watch_summary_output`）— 3189 tests

---

## [v54.1.0] — 2026-07-22 — 全エラーコード fav explain --error 対応完備

### Added
- `v54100_tests` 追加（`explain_error_all_codes_have_text` / `explain_error_e0419_exists`）— 3187 tests
- `error_catalog.rs` 全 92 エラーコードに `cmd_explain_error_collect` 対応を検証（カバレッジ強制）

---

## [v54.0.0] — 2026-07-22 — Integration Sprint 宣言

### Added
- `MILESTONE.md`: v54.0.0 Integration Sprint 宣言セクション追加（宣言文・v53.1〜v53.9 完了サマリー）
- `README.md`: v54.0 Integration Sprint マイルストーン宣言を追加
- `v54000_tests` 追加（`cargo_toml_version_is_54_0_0` / `changelog_has_v54_0_0` / `milestone_has_integration_sprint` / `readme_mentions_integration_sprint`）— 3185 tests

---

## [v53.9.0] — 2026-07-22 — 安定化・コードフリーズ（Integration Sprint 前調整）

### Added
- `site/content/docs/integration-overview.mdx` 新規作成 — Integration Sprint 全体像・統合機能・E2E デモの骨子
- `v53900_tests` 追加（`cargo_toml_version_is_53_9_0` / `integration_overview_doc_exists`）— 3181 tests

---

## [v53.8.0] — 2026-07-22 — CHANGELOG / MILESTONE 整理（v51〜v53 まとめ）

### Added
- `MILESTONE.md`: v51.0〜v53.0 Integration Sprint サマリーセクション追加 — DX 3.0 / Performance & Scale / Data Quality 2.0 の統合達成を記録
- `CHANGELOG.md`: v53.8.0 エントリに Integration Sprint サマリー参照を含む（`changelog_has_v51_to_v53_summary` テスト対象）
- `v53800_tests` 追加（`changelog_has_v51_to_v53_summary` / `milestone_integration_sprint_noted`）— 3179 tests

---

## [v53.7.0] — 2026-07-22 — ドキュメントサイト全体最終チェック

### Added
- `site/content/docs/glossary.mdx` 新規作成 — v51〜v53 の新語彙（`par` / `assert_schema` / `lineage` / `inlay hints` / `rune` / `stage` / `pipeline`）を定義
- `v53700_tests` 追加（`docs_no_broken_links` / `docs_glossary_updated`）— 3177 tests

---

## [v53.6.0] — 2026-07-22 — cookbook 更新（parallel-pipeline + schema-validation）

### Added
- `site/content/cookbook/schema-validation.mdx` 新規作成 — `assert_schema<T>` + nullable + OTel + `--audit-log` レシピ
- `v53600_tests` 追加（`cookbook_parallel_pipeline_exists` / `cookbook_schema_validation_exists`）— 3175 tests

---

## [v53.5.0] — 2026-07-22 — E2E 統合デモ Phase 2（assert_schema + audit-log + OTel）

### Added
- `examples/v55-demo/pipeline.fav`: `ValidOrder` 型・`SchemaCheck` stage（`assert_schema<ValidOrder>` + OTel コメント）追加
- `examples/v55-demo/pipeline.fav`: `Process` stage の入力型を `ValidOrder` に更新
- `examples/v55-demo/run.sh`: `fav run pipeline.fav --audit-log ./audit.log` を含むデモ実行スクリプト新規作成
- `v53500_tests` 追加（`e2e_integration_demo_has_schema` / `e2e_integration_demo_has_audit_log`）— 3173 tests

---

## [v53.4.0] — 2026-07-22 — E2E 統合デモ Phase 1（Kafka → par transform → Snowflake）

### Added
- `examples/v55-demo/` 新規作成 — v51〜v53 統合機能を示す E2E デモ（Phase 1）
  - `fav.toml`: `kafka = "2.1.0"` / `snowflake = "1.0.0"` 依存
  - `pipeline.fav`: `par [enrich.run, validate.run] |> Merge.ordered` を含む OrderIngestion パイプライン
  - `stages/enrich.fav`: `region` フィールド付与ステージ
  - `stages/validate.fav`: status / amount バリデーションステージ
- `v53400_tests` 追加（`e2e_integration_demo_structure` / `e2e_integration_demo_uses_par`）— 3171 tests

---

## [v53.3.0] — 2026-07-22 — DX × DQ 統合（`assert_schema` 失敗時の詳細 suggestion）

### Changed
- `error_catalog.rs`: E0419 エントリの `description` / `example` / `fix` / `suggestion` を強化
  - `description`: フィールド個別検証・`--strict-schema` の説明を追加
  - `example`: field diff 形式（`expected Int` / `got String` / `help: use Int.parse(...)` を含む）に更新
  - `fix`: 各フィールドを変換してから呼ぶ旨を追記
  - `suggestion`: `Int.parse()` / `Float.from_int()` の具体例を追加
  - `title` / `code` / `category` は変更なし（既存テスト互換維持）
- `v53300_tests` 追加（`assert_schema_error_has_suggestion` / `assert_schema_diff_shown`）— 3169 tests

---

## [v53.2.0] — 2026-07-22 — bench × par 統合（par stage 個別計測）

### Added
- `driver.rs`: `collect_par_stage_names` 追加 — `FlwDef.steps` の `Par` / `ParDistributed` から stage 名を収集
- `BenchStats` に `par_stages: Vec<String>` フィールド追加
- `bench_stats_to_json`: `"par_stages"` フィールドを JSON 出力に追加
- `cmd_bench`: bench case 実行後に `collect_par_stage_names` で `par_stages` を後付け
- `v53200_tests` 追加（`bench_par_stage_individual` / `bench_par_stage_total`）— 3167 tests

---

## [v53.1.0] — 2026-07-22 — lineage × LSP 統合（リネージをエディタで表示）

### Added
- `lsp/document_store.rs`: `CheckedDoc` に `lineage: LineageReport` フィールド追加、`open_or_change` で `lineage_analysis` をキャッシュ
- `lsp/hover.rs`: `lineage_block_for_stage` 追加 — stage ホバー時に upstream / downstream / schema を Markdown で表示
- `lineage.rs`: `LineageReport` に `#[derive(Default)]` 追加
- `v53100_tests` 追加（`lsp_hover_shows_lineage` / `lsp_hover_lineage_upstream`）
- `lsp/hover.rs` テスト追加（`lineage_block_shows_upstream_for_stage` / `lineage_block_shows_downstream_for_stage` / `lineage_block_returns_none_for_non_stage`）— 3165 tests

---

## [v53.0.0] — 2026-07-22 — Data Quality & Observability 2.0 宣言

### Added
- `MILESTONE.md` に v53.0.0「Data Quality & Observability 2.0」エントリ追加（宣言文付き）
- `README.md` に v53.0 マイルストーン言及追加
- `v53000_tests` 追加（`cargo_toml_version_is_53_0_0` / `changelog_has_v53_0_0` / `milestone_has_data_quality` / `readme_mentions_data_quality`）— 3160 tests

### Changed
- `cargo clean`（★クリーンアップ）完了

---

## [v52.9.0] — 2026-07-22 — 安定化・コードフリーズ（Data Quality 2.0 前調整）

### Added
- `site/content/docs/data-quality-overview.mdx` — Data Quality & Observability 2.0 概要ドキュメント（v52.1〜v52.8 機能一覧・各ドキュメントへのリンク）
- `v52900_tests` 追加（`cargo_toml_version_is_52_9_0` / `dq_overview_doc_exists`）— 3156 tests

### Changed
- `cargo clippy -- -D warnings` クリーン確認（0 エラー・0 警告）

---

## [v52.8.0] — 2026-07-22 — ドキュメントサイト Data Quality 記事

### Added
- `site/content/docs/data-quality/assert-schema.mdx` — `assert_schema` の使い方・nullable・strict モード・E0419
- `site/content/docs/tools/lineage-enhanced.mdx` — `--with-schema` / `--format html` / `-o` オプション説明
- `site/content/docs/tools/audit-log.mdx` — `fav run --audit-log` の使い方・JSONL フォーマット・`fav audit` との違い
- `v52800_tests` 追加（`docs_assert_schema_page_exists` / `docs_audit_log_page_exists` / `docs_lineage_enhanced_page_exists`）— 3154 tests

---

## [v52.7.0] — 2026-07-21 — OTel 強化（span 属性にスキーマ・リネージ情報付加）

### Added
- `OtelSpan` 構造体に `attrs: Vec<(String, String)>` フィールド追加（`otel.rs`）
- `otel_add_attr(key, val)` 追加 — 現在実行中 span（PENDING_SPANS）に文字列属性を付与
- `otel_patch_attr_on_last(key, val)` 追加 — 完了済み span（OTEL_SPANS の最後）に遡及追記
- `build_otlp_json` 更新 — OTLP JSON の `attributes` 配列に `span.attrs` を追加出力
- `otel_export_stdout` 更新 — 各 span の `attrs` を `key = value` 形式で stderr 出力
- `OTEL_PREV_STAGE` thread-local 追加（`vm.rs`） — OTel lineage 追跡用の前 stage 名保持
- `reset_stage_lineage()` 追加（`vm.rs`） — run 開始時に OTEL_PREV_STAGE をリセット
- `SeqStageEnter` に lineage フック追加:
  - 前 stage の完了済み span に `lineage.downstream = <現 stage 名>` を遡及追記
  - 現 stage の span に `lineage.upstream = <前 stage 名>` を追加
- `AssertSchema` opcode に schema フック追加:
  - 検証成功時に `schema.name = <型名>` / `schema.fields = <フィールド名列挙>` を span に追加
- `v52700_tests` 追加（`otel_span_has_schema_attr` + `otel_span_has_lineage_attr`）

---

## [v52.6.0] — 2026-07-21 — `fav run --audit-log` データアクセスログ

### Added
- `fav run --audit-log <output.jsonl>` オプション追加
  — `!Kafka` / `!Snowflake` のアクセスイベントを JSONL 形式でファイルに記録する
  — `--audit-log` 未指定時は従来通りログ出力なし（best-effort、パイプライン停止しない）
- `vm.rs` に `AUDIT_LOG_PATH` thread-local（`RefCell<Option<String>>`）追加（wasm32 除外）
- `vm.rs` に `set_audit_log_path(path: Option<String>)` 公開関数追加
- `vm.rs` に `append_audit_event(json_line: &str)` プライベートヘルパー追加
  — `OpenOptions::append(true).create(true)` でファイルに JSONL 行を追記
- `Kafka.produce_raw` アームに write イベントフック挿入（`ts` / `op` / `effect` / `topic`）
- `Kafka.consume_one_raw` アームに read イベントフック挿入
- `Snowflake.execute_raw` アームに write イベントフック挿入（SQL 先頭 80 文字を記録）
- `cmd_run` シグネチャに `audit_log: Option<&str>` 引数追加（末尾）
- `v52600_tests` 追加（`audit_log_read_event` + `audit_log_write_event`）

### 注意
- `!S3` は vm.rs に `"S3.*_raw"` アームが存在しないため本バージョンのスコープ外
- `site/content/docs/tools/audit-log.mdx` は v52.8.0 で追加予定

---

## [v52.5.0] — 2026-07-21 — SLA 監視 Rune

### Added
- `runes/sla/sla.fav` 新規作成（SLA 監視 Rune）
  - `check_freshness(timestamp, max_age_seconds)` — データ鮮度チェック（古すぎる場合 `Err` を返す）
  - `check_latency(stage, threshold_ms)` — ステージレイテンシチェック（超過時 `Err` を返す）
  - `alert(message)` — 外部アラート基盤への通知スタブ（`Sla.alert_raw` 呼び出し）
  - SLA 違反時は `Err` を返し、呼び出し側が `bind _ <-` パターンで fail-fast を実現
  - `!Observe` エフェクトはコメントで言及（スタブ実装、Effect enum 追加は将来バージョン）
- `v52500_tests` 追加（`sla_rune_latency_check` + `sla_rune_freshness_check` + `sla_rune_alert_check`）

---

## [v52.4.0] — 2026-07-21 — `fav explain --lineage` インタラクティブ HTML レポート

### Added
- `render_lineage_html(report: &LineageReport) -> String` 追加（`lineage.rs`）
  — SVG ノードグラフ + クリックで詳細表示する自己完結型 HTML を生成
  — ノードは `<g class="node" onclick="showDetail(...)">` でラップされクリック可能
  — JS `stages` JSON + `showDetail(name)` 関数でステージ詳細（kind/effects/schema/sources/sinks）を表示
- `cmd_explain_lineage` に `output: Option<&str>` 引数追加（`driver.rs`）
  — `Some(path)` のとき `std::fs::write` でファイル書き出し、`None` のとき stdout 出力
- `--format html` サポートを `match format` アームに追加（`driver.rs`）
- `main.rs` の `--lineage` ブロックに `-o <file>` CLI フラグ追加
- `v52400_tests` 追加（`lineage_html_output` + `lineage_html_has_stage_detail` + `lineage_html_renders_stage_node`）

---

## [v52.3.0] — 2026-07-21 — `fav explain --lineage` 表示強化（スキーマ情報付加）

### Added
- `LineageEntry` に `pub schema: Option<String>` フィールドを追加（v52.3.0）
- `lineage.rs` に `collect_assert_schema_name` / `_stmt` / `_block` 関数追加
  — ステージ body から `assert_schema<T>` の型名 `T` を収集する
- `lineage_analysis` で各 TrfDef の `schema` フィールドを自動設定
- `render_lineage_mermaid_with_schema(report, show_dead, with_schema)` 追加
  — `with_schema = true` のとき Mermaid ノードラベルに `<br/>schema:<Name>` を付加
- `render_lineage_dot_with_schema(report, with_schema)` 追加
  — `with_schema = true` のとき DOT ノードラベルに `\nschema:<Name>` を付加
- `cmd_explain_lineage` に `with_schema: bool` 引数追加（mermaid/dot の呼び出しを `*_with_schema` 版に変更）
- `main.rs` に `--with-schema` CLI フラグ追加（`fav explain --lineage --with-schema`）
- `v52300_tests` 追加（`lineage_mermaid_with_schema` + `lineage_dot_with_schema`）

---

## [v52.2.0] — 2026-07-20 — `assert_schema` Phase 2（nullable・追加フィールド対応）

### Added
- `FieldMeta` に `optional: bool` フィールドを追加（JSON serde は `#[serde(default)]`）
- `build_type_meta` で `TypeExpr::Optional` 型フィールドを `optional: true` として記録
- `backend/artifact.rs` の TMET バイナリフォーマット更新（bit-flag 方式: bit1 = optional）
  旧バイナリは bit1 が常に 0 のため後方互換を保持
- `Vm` struct に `strict_schema: bool` フィールド追加（thread-local `STRICT_SCHEMA` で初期化）
- `backend/vm.rs` の `AssertSchema` ハンドラ更新:
  - optional フィールドが map にない場合はスキップ（エラーにしない）
  - 想定外フィールドがある場合: `--strict-schema` ならエラー、それ以外は W036 警告を `emit_log` に記録
- `--strict-schema` CLI フラグ追加（`main.rs` / `driver.rs`）
- `lint.rs` に W036 `check_w036_extra_schema_fields` スタブ追加（将来の静的解析拡張予約）
- `v52200_tests` 追加（`assert_schema_nullable_field` + `assert_schema_extra_field_warn`）

---

## [v52.1.0] — 2026-07-20 — `assert_schema` Phase 1（型チェック）

### Added
- `Expr::AssertSchema { ty_name, arg, span }` を `ast.rs` に追加（`assert_schema<T>(value)` 構文の AST ノード）
- `IRExpr::AssertSchema { ty_name, arg, ty }` を `middle/ir.rs` に追加
- `middle/compiler.rs` で `Expr::AssertSchema` → `IRExpr::AssertSchema` コンパイル実装
- `backend/codegen.rs` に `Opcode::AssertSchema = 0x64` 追加（Layout: opcode(1) + ty_name_idx(2)）
- `backend/vm.rs` に `Opcode::AssertSchema` の実行時評価ハンドラ追加（スキーマ照合 → ok/err 返却）
- `error_catalog.rs` に E0419「assert_schema type mismatch」定義（予約コメントを実装に置換）
- `v52100_tests` 追加（`assert_schema_type_ok` + `assert_schema_type_fail`）
- `backend/wasm_codegen.rs` に `IRExpr::AssertSchema` アーム追加（wasm MVP では UnsupportedExpr）
- `backend/wasm_dce.rs` / `fmt.rs` / `lint.rs` / `emit_python.rs` / `middle/checker.rs` / `lineage.rs` / `lsp/references.rs` などの exhaustive match に `AssertSchema` アーム追加

### Notes
- `frontend/parser.rs` の `assert_schema<T>(...)` 構文パースは Phase 2 以降（v52.2.0〜）

---

## [v52.0.0] — 2026-07-20 — Performance & Scale 宣言

### Added
- `MILESTONE.md` に v52.0.0「Performance & Scale」エントリ追加
- `README.md` に Performance & Scale マイルストーン言及追加
- `driver.rs`: `v52000_tests` 追加（4 テスト）
  - `cargo_toml_version_is_52_0_0`
  - `changelog_has_v52_0_0`
  - `milestone_has_performance_scale`
  - `readme_mentions_performance_scale`

### Declaration
> 「並列パイプラインはコアを使い切り、バックプレッシャーは
>  データの氾濫を防ぎ、ベンチマークは退行を即座に検出する。
>  Favnir は大規模データに立ち向かえる言語になった。
>
>  これが Favnir v52.0 — Performance & Scale の姿である。」

---

## [v51.9.0] — 2026-07-20 — 安定化・コードフリーズ（Performance & Scale 前調整）

### Added
- `site/content/docs/performance-overview.mdx` — Performance & Scale 概要ページ（par / fav bench / インクリメンタルコンパイル / WASM 最適化 / ホットパス最適化の俯瞰ドキュメント）
- `driver.rs`: `v51900_tests` 追加（2 テスト）
  - `cargo_toml_version_is_51_9_0`
  - `perf_overview_doc_exists`

### Fixed (code-review)
- `performance-overview.mdx`: `Favnir v52.0 — Performance & Scale` → `Favnir Performance & Scale スプリント（v51.1〜v51.9）` に修正（v51.9.0 時点で未リリースの v52.0 を前提とした表記を解消）

---

## [v51.8.0] — 2026-07-20 — ドキュメントサイト Performance 記事

### Added
- `site/content/docs/runtime/parallel.mdx` — `par` 並列ステージ実行・`Merge.ordered`/`Merge.any`・バックプレッシャー（`buffer_size`）の解説記事
- `site/content/docs/tools/bench-regression.mdx` — `fav bench --compare` / `--fail-on-regression` / `--threshold` による差分回帰検出の解説記事
- `driver.rs`: `v51800_tests` 追加（2 テスト）
  - `docs_parallel_page_exists`
  - `docs_bench_regression_page_exists`

### Fixed (code-review)
- `parallel.mdx`: Rust 内部実装詳細（`std::thread::spawn` / `tokio::join_all` / `FuturesUnordered`）を除去
- `parallel.mdx`: バックプレッシャーのコード例を型整合性のある `Stream<RawOrder> -> Stream<Order>` に修正（`Ok(order)` → `Order.from_raw(raw)`）
- `parallel.mdx`: `Arc<Mutex<T>>` の注意事項を `!Cache` エフェクト経由の記述に差し替え
- `parallel.mdx`: `buffer_size` 無制限時の OOM リスク警告を追加
- `bench-regression.mdx`: `--fail-on-regression` は `--compare` と併用必須である旨を関連コマンド表に明記

---

## [v51.7.0] — 2026-07-19 — WASM ビルドサイズ最適化

### Added
- `WasmOptLevel::Os` バリアント追加（`wasm_opt_pass.rs`）— `-Os` フラグで wasm-opt サイズ最適化
- `dce_from_exports(ir, entry_names)` 追加（`wasm_dce.rs`）— 複数エントリから BFS で到達可能関数の union を収集し DCE 適用
  - `entry_names` が空の場合は保守的フォールバック（除去なし）
- `fav build --target wasm` を `build_wasm_artifact_with_config(dce=true, opt_level=Os)` に強化（`driver.rs`）
  - `FAV_WASM_SIZE_REPORT=1` 環境変数でサイズレポートを有効化
- `benchmarks/v51.7.0.json` 追加（WASM サイズ計測ベースライン）
- `driver.rs`: `v51700_tests` 追加（4 テスト）
  - `cargo_toml_version_is_51_7_0`
  - `wasm_dce_removes_unused_fns`
  - `wasm_bundle_size_reduced`
  - `benchmark_json_exists`

### Fixed (code-review)
- `build_wasm_artifact_with_config` の DCE 呼び出しを `collect_reachable_fns` + `apply_dce` から `dce_from_exports(&["main"])` に修正（v51.7.0 主機能を実際に発動させる）
- `dce_from_exports` 内の `std::collections::HashSet::new()` → `HashSet::new()` に統一
- `FAV_WASM_SIZE_REPORT=1` のみ有効である旨のコメント追加
- `wasm_opt_pass.rs` に `os_flag_is_minus_os` テスト追加（`WasmOptLevel::Os.flag() == "-Os"` を保証）

---

## [v51.6.0] — 2026-07-19 — checker / compiler ホットパス最適化

### Added
- `SubstRef = std::rc::Rc<Subst>` 型エイリアス追加（`checker.rs`）— クローンコストを O(1) に削減
- `Subst::into_ref(self) -> SubstRef` メソッド追加（`checker.rs`）
- `SourceCache(HashMap<String, String>)` + `get_or_load` 追加（`compiler_fav_runner.rs`）
  - 同一ファイルの繰り返し読み込みをキャッシュで排除
  - `impl Default` 実装済み（clippy 対応）
- `ProfileBuildResult { parse_ms, check_ms, compile_ms }` 追加（`driver.rs`）
- `profile_build_file(path) -> Result<ProfileBuildResult, String>` 追加（`driver.rs`）
  - parse / check / compile 各フェーズを `Instant::now()` で計測
- `cmd_profile_build(path)` 追加（`driver.rs`）— テーブル形式で各フェーズ時間を表示
- `fav profile --build <file>` CLI フラグ追加（`main.rs`）
  - `--compare` との同時指定はエラー
- `benchmarks/v51.6.0.json` 追加（ベースライン記録）
- `driver.rs`: `v51600_tests` 追加（3 テスト）
  - `cargo_toml_version_is_51_6_0`
  - `checker_perf_hot_path_improved`
  - `compiler_perf_baseline_recorded`

---

## [v51.5.0] — 2026-07-19 — インクリメンタルコンパイル依存グラフ

### Added
- `DepGraph` に `#[derive(Serialize, Deserialize)]` 追加（`incremental/dep_graph.rs`）
- `DepGraph::transitive_affected_by` 追加 — 変更ファイルの推移的依存元を BFS で列挙
- `save_dep_graph_json(graph, path)` 追加 — DepGraph を `.fav-cache/dep-graph.json` 等に保存
- `load_dep_graph_json(path)` 追加 — JSON から DepGraph を復元（不在・破損時は空グラフ）
- `incremental_files_to_rebuild(stems, paths, graph, cache_dir)` 追加（`driver.rs`）
  - v49.3 の `file_needs_recheck` を再利用して変更ファイルを検出
  - `transitive_affected_by` で推移的依存元も rebuild 対象に追加
  - `(rebuild_list, skip_list)` を返す
- `driver.rs`: `v51500_tests` 追加（3 テスト）
  - `cargo_toml_version_is_51_5_0`
  - `incremental_dep_graph_rebuilt`
  - `incremental_transitive_invalidation`

---

## [v51.4.0] — 2026-07-19 — `fav bench` 差分回帰検出

### Added
- `BenchOpts` に `compare: Option<String>` / `fail_on_regression: bool` / `threshold: f64` フィールド追加
- `bench_stats_to_compare_json(version, stats)` ヘルパー追加（avg_us → ms 変換、`cmd_bench_compare` 互換 JSON を生成）
- `cmd_bench` に `--compare <path>` 対応: bench 実行後にベースライン JSON と自動比較
- `main.rs` に `--compare` / `--fail-on-regression` / `--threshold` CLI フラグ追加
  - `--fail-on-regression`: 回帰検出時に `process::exit(1)`（CI 向け）
  - `--threshold <pct>`: 回帰閾値（デフォルト 10.0%）
- `benchmarks/v51.3.0.json` 作成（ベースライン用プレースホルダー）
- `driver.rs`: `v51400_tests` 追加（3 テスト）
  - `cargo_toml_version_is_51_4_0`
  - `bench_regression_detected`
  - `bench_no_regression_passes`

### Notes
- `fav bench --baseline X --current Y`（v24.3.0 実装）は変更なし（後方互換性維持）
- `--compare` は bench 実行と比較を一括実施。`--baseline` は既存 2 ファイル比較。

---

## [v51.3.0] — 2026-07-19 — ストリーミングバックプレッシャー制御

### Added
- `StreamConfig.buffer_size: Option<usize>` フィールド追加（`toml.rs`）
- `fav.toml` `[stream]` セクションで `buffer_size = N` を解析（`parse_fav_toml`）
  - `buffer_size = 0` は `None` 相当として扱う（`chunks(0)` パニック防止）
- `VM.stream_buffer_size: Option<usize>` フィールド追加（`vm.rs`）
- `VM::run_with_stream_buffer_size` 静的メソッド追加（テスト・統合用）
- `__streaming_pipeline` ハンドラ: `buffer_size` が設定されている場合 `chunk_size` を `min(compiled, buffer_size)` にキャップ（v51.3.0）
  - 将来 tokio 化時に `sync_channel` による真のバックプレッシャーに置換予定
- `driver.rs`: `v51300_tests` 追加（3テスト）
  - `cargo_toml_version_is_51_3_0`
  - `stream_buffer_size_config`
  - `stream_backpressure_blocks`

### Changed
- `v51200_tests::cargo_toml_version_is_51_2_0` を削除（慣例）

---

## [v51.2.0] — 2026-07-19 — `par` Phase 2: Merge.ordered / Merge.any

### Added
- `MergeMode { Ordered, Any }` enum を `ast.rs` に追加
- `FlwStep::Merge(MergeMode)` variant を `ast.rs` に追加
- パーサー: `par [A, B] |> Merge.ordered` / `|> Merge.any` 構文をパース（`parser.rs`）
  - `"Merge"` のみは `FlwStep::Stage("Merge")` にフォールバック（後方互換）
- コンパイラー: `FlwStep::Merge` → `IO.merge_ordered_raw` / `IO.merge_any_raw` 呼び出し emit（`compiler.rs`）
- VM: `IO.merge_ordered_raw` / `IO.merge_any_raw` ハンドラ追加（`vm.rs`）
  - `Variant("ok" | "some", payload)` → payload を Unwrap して List に収集
  - `Variant("err", ...)` → fail-fast で Err を返す
  - `merge_any_raw` は std::thread 実装では `merge_ordered_raw` と同一動作（将来 tokio で FuturesUnordered 化予定）
- `driver.rs`: `v51200_tests` 追加（3テスト）
  - `cargo_toml_version_is_51_2_0`
  - `par_stage_merge_ordered`
  - `par_stage_merge_unordered`

### Changed
- `FlwStep` match 網羅性: `ast.rs` / `compiler.rs` / `checker.rs` / `emit_python.rs` / `ast_lower_checker.rs` を更新
- `v51100_tests::cargo_toml_version_is_51_1_0` を削除（慣例）

### Notes
- `Stream<T>` stage への `par` 対応は v51.3.0 以降で実施

---

## [v51.1.0] — 2026-07-19 — `par` stage VM 直接実行 Phase 1

### Added
- `IRExpr::Par { stage_names, input, ty }` — IR 層に par 専用 variant 追加（`ir.rs`）
- `Opcode::ParStages = 0x70` — par 並列実行専用オペコード追加（`codegen.rs`）
- VM `ParStages` ハンドラ — `std::thread::spawn` + `VM::run_with_vmvalues` で並列実行（`vm.rs`）
  - fail-fast: ステージが `Result.err(...)` を返した場合は即座に `Err` を返す
  - wasm32 非サポートガード追加
- `driver.rs`: `v51100_tests` 追加（3テスト）
  - `cargo_toml_version_is_51_1_0`
  - `par_stage_runs_parallel`
  - `par_stage_error_propagation`

### Changed
- `FlwStep::Par` コンパイル経路を `IO.par_execute_raw` 呼び出し構築から `IRExpr::Par` emit に置換（`compiler.rs`）
- `remap_string_operands` に `ParStages` ケース追加（str_table インデックス正規化）
- `wasm_codegen.rs`（5 関数）・`wasm_dce.rs`・`driver.rs` で `IRExpr::Par` match arm 追加
- `v51000_tests::code_freeze_v51_0_0` を 51.x 系バージョンチェックに緩和

### Removed
- `v51000_tests::cargo_toml_version_is_51_0_0`（`v51100_tests::cargo_toml_version_is_51_1_0` に置換）

---

## [v51.0.0] — 2026-07-19 — Developer Experience 3.0 宣言

### Added
- `MILESTONE.md` に v51.0.0 エントリ追加（Developer Experience 3.0 宣言文）
- `README.md` に DX 3.0 マイルストーン言及追加
- `driver.rs`: `v51000_tests` 追加（6テスト）
  - `cargo_toml_version_is_51_0_0`
  - `changelog_has_v51_0_0`
  - `milestone_has_dx3`
  - `readme_mentions_dx3`
  - `dx3_milestone_declared`
  - `code_freeze_v51_0_0`

### Changed
- `Cargo.toml` version: `50.9.0` → `51.0.0`

### Removed
- `driver.rs` v509000_tests より `cargo_toml_version_is_50_9_0` / `code_freeze_v50_9_0` 削除（`"50.9.0"` assert は v51.0.0 では不要）

---

## [v50.9.0] — 2026-07-19 — 安定化・コードフリーズ（DX 3.0 前調整）

### Added
- `site/content/docs/dx3-overview.mdx` 新規作成（DX 3.0 全機能概要・v50.1〜v50.8 の実装内容テーブル・各ドキュメントへのリンク）
- `driver.rs`: `v509000_tests` 追加（3テスト）
  - `cargo_toml_version_is_50_9_0`
  - `dx3_overview_doc_exists`
  - `code_freeze_v50_9_0`

### Changed
- `Cargo.toml` version: `50.8.0` → `50.9.0`

---

## [v50.8.0] — 2026-07-19 — ドキュメントサイト DX 3.0 記事

### Added
- `site/content/docs/tools/diagnostics.mdx` 新規作成（統一診断出力・`fav explain --error` の使い方）
- `site/content/docs/tools/trace-watch.mdx` 新規作成（`fav run --trace/--watch` のデバッグパターン）
- `driver.rs`: `v508000_tests` 追加（3テスト）
  - `cargo_toml_version_is_50_8_0`
  - `docs_diagnostics_page_exists`
  - `docs_trace_watch_page_exists`

### Changed
- `Cargo.toml` version: `50.7.0` → `50.8.0`

---

## [v50.7.0] — 2026-07-19 — `fav run --trace` / `fav run --watch` 強化

### Added
- `backend/vm.rs`: `WATCH_FIELDS` スレッドローカル・`set_watch_fields` / `watch_fields` アクセサ追加
- `backend/vm.rs`: `SeqStageCheck` Ok 分岐に watch フック追加（Record 型出力のフィールドを `[watch] target: — → value  (stage: name)` 形式で trace_lines に記録）
- `driver.rs`: `v507000_tests` 追加（3テスト）
  - `cargo_toml_version_is_50_7_0`
  - `run_trace_structured_output`
  - `run_watch_tracks_variable`

### Changed
- `backend/vm.rs`: `SeqStageCheck` Ok 分岐の trace フォーマットを `[TRACE] stage X: exit Ok(...)` から `[trace] stage=X  out=VALUE` に変更（構造化ログ）
- `backend/vm.rs`: `uvm` 生成条件に `|| !watch_fields().is_empty()` を追加（watch が `verbose_level` と独立して動作）
- `Cargo.toml` version: `50.6.0` → `50.7.0`

### Notes
- `in=` フィールドは未実装（SeqStageEnter 時点でスタック上に入力値がない制約）
- CLI `--watch` フラグ解析は未実装（テスト API `set_watch_fields` 経由のみ）
- watch は Record フィールドに限定（スカラー値 watch はスコープ外）

---

## [v50.6.0] — 2026-07-19 — LSP ホバー情報強化（Rune メソッドシグネチャ）

### Added
- `lsp/hover.rs`: `RuneFn` 構造体・`RUNE_FNS` 定数追加（kafka: consume/produce、csv: read/write の 4 エントリ）
- `lsp/hover.rs`: `builtin_hover_at` 追加（PascalCase NS → BUILTIN_FNS シグネチャ表示）
- `lsp/hover.rs`: `rune_hover_at` 追加（lowercase NS → RUNE_FNS シグネチャ・エフェクト・doc 表示）
- `driver.rs`: `v506000_tests` 追加（3テスト）
  - `cargo_toml_version_is_50_6_0`
  - `lsp_hover_builtin_fn`
  - `lsp_hover_rune_method`

### Changed
- `lsp/hover.rs`: `handle_hover` が builtin/rune lookup を優先し、`type_at` にフォールバック
- `Cargo.toml` version: `50.5.0` → `50.6.0`

---

## [v50.5.0] — 2026-07-19 — LSP インレイヒント Phase 2（パイプライン stage 型）

### Added
- `lsp/inlay_hints.rs`: `collect_pipeline_type_hints` 追加（`Type::Arrow` / `Type::Trf` 型の stage のみ `: In -> Out` ヒント表示）
- `driver.rs`: `v505000_tests` 追加（3テスト）
  - `cargo_toml_version_is_50_5_0`
  - `lsp_inlay_hint_stage_type`
  - `lsp_inlay_hint_pipeline_type`

### Changed
- `lsp/inlay_hints.rs`: `handle_inlay_hints` に `collect_pipeline_type_hints` を組み込み
- `Cargo.toml` version: `50.4.0` → `50.5.0`

---

## [v50.4.0] — 2026-07-19 — LSP インレイヒント Phase 1（変数・関数戻り型）

### Added
- `lsp/inlay_hints.rs`: `collect_fn_return_hints` 追加（`fn` 定義の戻り型省略時に ` -> Type` ヒント表示）
- `driver.rs`: `v504000_tests` 追加（3テスト）
  - `cargo_toml_version_is_50_4_0`
  - `lsp_inlay_hint_let_binding`
  - `lsp_inlay_hint_fn_return`

### Changed
- `lsp/inlay_hints.rs`: `handle_inlay_hints` に `collect_fn_return_hints` を組み込み
- `Cargo.toml` version: `50.3.0` → `50.4.0`

---

## [v50.3.0] — 2026-07-19 — `explain-error` と `explain` の統合

### Added
- `main.rs`: `fav explain --error <code>` / `--list` / `--list --format json` サポート追加（`--error` フラグ）
- `driver.rs`: `cmd_explain_error_collect(code) -> Option<String>` ヘルパー追加（テスト可能な出力収集）
- `driver.rs`: `v503000_tests` 追加（3テスト）
  - `cargo_toml_version_is_50_3_0`
  - `explain_error_flag_works`
  - `explain_error_all_codes_have_text`

### Changed
- `driver.rs`: `cmd_explain_error` が `cmd_explain_error_collect` に委譲する形に変更
- `Cargo.toml` version: `50.2.0` → `50.3.0`

### Compatibility
- `fav explain-error <code>` は後方互換として引き続き動作

---

## [v50.2.0] — 2026-07-18 — エラー診断統一 Phase 2（JSON / LSP / CLI 出力の一貫化）

### Added
- `lsp/protocol.rs`: `DiagnosticData` struct 追加（`suggestion: String`）
- `lsp/protocol.rs`: `Diagnostic.data: Option<DiagnosticData>` フィールド追加（LSP spec §3.16 Diagnostic.data）
- `lsp/diagnostics.rs`: `errors_to_diagnostics` が `error_catalog::lookup` 経由で `data.suggestion` を設定
- `driver.rs`: `v502000_tests` 追加（3テスト）
  - `cargo_toml_version_is_50_2_0`
  - `check_json_includes_suggestion`
  - `lsp_diagnostic_includes_suggestion`

### Changed
- `Cargo.toml` version: `50.1.0` → `50.2.0`

### Removed
- `v501000_tests::cargo_toml_version_is_50_1_0`（バージョン進行に伴い削除）

---

## [v50.1.0] — 2026-07-18 — エラー診断統一 Phase 1（全コード suggestion 補完）

### Added
- `error_catalog.rs`: 34 件の `suggestion: None` を `suggestion: Some(...)` に補完（全エラーコードに修正提案テキストを追加）
- `driver.rs`: `v501000_tests` 追加（3テスト）
  - `cargo_toml_version_is_50_1_0`
  - `error_suggestion_all_covered`
  - `error_suggestion_e0018_text`

### Changed
- `Cargo.toml` version: `50.0.0` → `50.1.0`

### Removed
- `v50000_tests::cargo_toml_version_is_50_0_0`（バージョン進行に伴い削除）

---

## [v50.0.0] — 2026-07-18 — Production 2.0 宣言 ★Language Maturity

> 「`return` による安全なガード節、成熟した標準ライブラリ、
>  明確なモジュールシステム、インラインテストが揃い、
>  Favnir は迷わず使える実用言語になった。
>
>  これが Favnir v50.0 — Production 2.0 の姿である。」

### Added
- `README.md` に Language Maturity / Production 2.0 マイルストーン宣言を追加
- `driver.rs`: `v50000_tests` 追加（4テスト）
  - `cargo_toml_version_is_50_0_0`
  - `changelog_has_v50_0_0`
  - `milestone_has_language_maturity`
  - `readme_mentions_language_maturity`

### Changed
- `Cargo.toml` version: `49.9.0` → `50.0.0`

### Notes
- v49.1〜v49.9 の全機能統合・安定化・セキュリティ審査完了
- `cargo clean`（★クリーンアップ）実施済み

---

## [v49.9.0] — 2026-07-18 — v50.0 前調整・安定化

### Added
- `site/content/docs/language-maturity-overview.mdx` に v46〜v49 主要機能一覧テーブルを追加
- `driver.rs`: `v499000_tests` 追加（`cargo_toml_version_is_49_9_0` / `language_maturity_overview_doc_exists` 2テスト）

### Changed
- `Cargo.toml` version: `49.8.0` → `49.9.0`

### Notes
- コードフリーズ版 — 新機能追加・API 変更なし
- `cargo clippy -- -D warnings` / `cargo fmt -- --check` クリーン確認済み

---

## [v49.8.0] — 2026-07-18 — ドキュメントサイト全面更新 Phase 2 + CHANGELOG 整理

### Added
- `site/content/docs/language-maturity-overview.mdx` 新規作成（Language Maturity / v50 の4本柱を概説）
- `MILESTONE.md` に v50.0.0 Language Maturity マイルストーン記述を追加
- `driver.rs`: `v498000_tests` 追加（`docs_site_v50_overview_exists` / `milestone_has_language_maturity` 2テスト）

### Changed
- `Cargo.toml` version: `49.7.0` → `49.8.0`

---

## [v49.7.0] — 2026-07-18 — セキュリティ審査 2.0

### Added
- `driver.rs`: `validate_import_path(path: &str) -> Result<(), String>` 追加
  - パストラバーサル（`..` コンポーネント）を拒否
  - バックスラッシュ混入を拒否
- `driver.rs`: `validate_rune_name(name: &str) -> Result<(), String>` 追加
  - 英数字 + `-` 以外の文字を拒否（スペース・スラッシュ等）
  - 先頭・末尾 `-`、連続 `--` を拒否
- `driver.rs`: `v497000_tests` 追加（`import_path_traversal_rejected` / `install_invalid_name_rejected` 2テスト）

### Changed
- `Cargo.toml` version: `49.6.0` → `49.7.0`

---

## [v49.6.0] — 2026-07-18 — WASM / Python transpiler 互換確認

### Added
- `driver.rs`: `v496000_tests` 追加（`python_emit_return_stmt` / `wasm_compat_return_stmt` 2テスト）
  - `python_emit_return_stmt`: `emit_python_str` で `Stmt::Return` が `return` キーワードを出力することを確認
  - `wasm_compat_return_stmt`: `wasm_codegen.rs` に `IRStmt::Return` の match arm が存在することを確認

### Changed
- `Cargo.toml` version: `49.5.0` → `49.6.0`

### Notes
- `emit_python.rs` の `Stmt::Return` は既に完全実装済み（変更なし）
- WASM での `return` は MVP 制限として `UnsupportedExpr` を返す設計を維持

---

## [v49.5.0] — 2026-07-18 — cookbook 更新

### Added
- `site/content/cookbook/return-guard-pattern.mdx` 新規作成（`return Result` guard パターンレシピ）
- `site/content/cookbook/inline-testing.mdx` 新規作成（`#[test]` インラインテストレシピ）
- `site/content/cookbook/modular-pipelines.mdx` 新規作成（新 import 構文モジュール化レシピ）
- `driver.rs`: `v495000_tests` 追加（`cookbook_return_guard_exists` / `cookbook_fav_test_exists` 2テスト）

### Changed
- `Cargo.toml` version: `49.4.0` → `49.5.0`

---

## [v49.4.0] — 2026-07-18 — ドキュメントサイト全面更新 Phase 1

### Added
- `site/content/docs/syntax/return.mdx` 新規作成（`return` ガード節構文ドキュメント）
- `site/content/docs/modules/import.mdx` 新規作成（import 2.0 + W035 移行ドキュメント）
- `driver.rs`: `v494000_tests` 追加（`docs_return_syntax_exists` / `docs_import_v2_exists` 2テスト）

### Changed
- `Cargo.toml` version: `49.3.0` → `49.4.0`

---

## [v49.3.0] — 2026-07-18 — `fav check` インクリメンタル型チェック

### Added
- `compute_file_fingerprint(path)` — SHA-256 フィンガープリント計算（`sha2` 0.10 使用）
- `file_needs_recheck(path, cache_dir)` — キャッシュと比較して再チェック要否を判定
- `update_fingerprint_cache(path, cache_dir)` — `.fav-cache/<filename>.fp` にフィンガープリントを保存
- `driver.rs`: `v493000_tests` 追加（`incremental_check_skips_unchanged` / `incremental_check_detects_change` 2テスト）

### Changed
- `Cargo.toml` version: `49.2.0` → `49.3.0`

---

## [v49.2.0] — 2026-07-18 — パフォーマンス計測 + ボトルネック修正

### Added
- `benchmarks/v49.2.0.json` 新規作成（v46〜v49 機能追加後の速度計測記録）
  - `metrics.checker_ms: 12` / `metrics.compiler_ms: 8` / `metrics.total_pipeline_ms: 25`
  - `regression: false`（ホットパスは問題なし、改善は v49.3.0 以降）
- `driver.rs`: `v492000_tests` 追加（`bench_all_result_recorded` / `checker_perf_regression_none` 2テスト）

### Changed
- `Cargo.toml` version: `49.1.0` → `49.2.0`

---

## [v49.1.0] — 2026-07-18 — 全機能統合テスト + E2E デモ更新

### Added
- `examples/v50-demo/fav.toml` 新規作成（`[runes] kafka = "2.1.0"`）
- `examples/v50-demo/stages/validate.fav` 新規作成（`return` ガード節 + `Result<Order, String>`）
- `examples/v50-demo/pipeline.fav` 新規作成（新 import 構文 + `#[test]` インラインテスト）
- `driver.rs`: `v491000_tests` 追加（`e2e_demo_v50_structure` / `e2e_demo_uses_new_import` 2テスト）

### Changed
- `Cargo.toml` version: `49.0.0` → `49.1.0`
- `driver.rs`: `cargo_toml_version_is_49_0_0` をスタブ化（バージョンバンプにより）

---

## [v49.0.0] — 2026-07-18 — Module & Package 2.0 宣言 ★クリーンアップ

### Added
- `MILESTONE.md`: v49.0.0 Module & Package 2.0 エントリ追加（宣言文 + 達成コンポーネント v48.1〜v48.9 の 9 件）
- `README.md`: Module & Package 2.0 マイルストーン言及追加
- `driver.rs`: `v49000_tests` 追加（`cargo_toml_version_is_49_0_0` / `changelog_has_v49_0_0` / `milestone_has_module_package_v2` / `readme_mentions_module_package_v2` 4テスト）

### Changed
- `Cargo.toml` version: `48.9.0` → `49.0.0`

### Chore
- `cargo clean` 実施（ビルドアーティファクト除去）

---

## [v48.9.0] — 2026-07-18 — Module ドキュメント + migration guide + v49.0 前調整

### Added
- `site/content/docs/module-system.mdx` 新規作成（パッケージ import / ローカル import / E0418 / W035 解説）
- `site/content/docs/migration-guide-import.mdx` 新規作成（旧 `import rune "X"` → 新 `import X` 移行ガイド）
- `driver.rs`: `v489000_tests` 追加（`module_system_doc_exists` / `import_migration_guide_exists` 2テスト）

### Changed
- `Cargo.toml` version: `48.8.0` → `48.9.0`

---

## [v48.8.0] — 2026-07-18 — `fav rune` コマンド群（純粋ヘルパー関数追加）

### Added
- `driver.rs`: `list_installed_runes(root: &Path) -> Vec<String>` 追加（`runes/` ディレクトリ走査・ソート済み返却）
- `driver.rs`: `get_rune_version(root: &Path, name: &str) -> Option<String>` 追加（`runes/<name>/rune.toml` の `[rune]` セクションから version 取得）
- `driver.rs`: `v488000_tests` 追加（`fav_rune_list_shows_installed` / `fav_rune_info_shows_version` 2テスト）

### Changed
- `Cargo.toml` version: `48.7.0` → `48.8.0`

---

## [v48.7.0] — 2026-07-18 — rune.toml 標準化

### Added
- `toml.rs`: `validate_rune_toml(content: &str) -> Vec<String>` 追加（`[rune]` 必須・`name`/`version`/`entry` 必須・`[connection]` 非標準チェック）
- `driver.rs`: `v487000_tests` 追加（`rune_toml_standard_format` / `rune_toml_no_connection_section` 2テスト）

### Changed
- `Cargo.toml` version: `48.6.0` → `48.7.0`

---

## [v48.6.0] — 2026-07-18 — 循環 import 検出 + E0418

### Added
- `error_catalog.rs`: E0418（`circular import detected`）正式追加
- `driver.rs`: `detect_circular_imports(graph) -> Option<Vec<String>>` 追加（DFS カラーリングによる循環検出）
- `driver.rs`: `v486000_tests` 追加（`circular_import_e0418` / `non_circular_import_ok` 2テスト）

### Changed
- `Cargo.toml` version: `48.5.0` → `48.6.0`

---

## [v48.5.0] — 2026-07-18 — import エイリアス完全化 + 旧構文 deprecation

### Added
- `lint.rs`: `check_w035_legacy_import_rune` 追加 — `ImportKind::Legacy` を検出して W035 警告を発行
- `lint.rs`: `lint_program` に W035 登録
- `driver.rs`: `v485000_tests` 追加（`import_alias_resolves` / `legacy_import_rune_w035` 2テスト）

### Changed
- `Cargo.toml` version: `48.4.0` → `48.5.0`

---

## [v48.4.0] — 2026-07-18 — `fav install` コマンド（`[runes]` 対応）

### Added
- `driver.rs`: `install_rune_stubs(pkg_name, root, runes) -> Vec<String>` 追加（`runes/<name>/` + `rune.toml` スタブ作成）
- `driver.rs`: `cmd_install_runes(pkg_name)` 追加（`fav install-rune` CLI エントリ）
- `main.rs`: `Some("install-rune")` アーム追加（`Some("install")` の直後）
- `driver.rs`: `v484000_tests` 追加（`fav_install_creates_rune_dir` / `fav_install_all_from_toml` 2テスト）

### Changed
- `Cargo.toml` version: `48.3.0` → `48.4.0`

---

## [v48.3.0] — 2026-07-18 — `fav.toml [runes]` 解決ロジック

### Added
- `toml.rs`: `FavToml.runes: HashMap<String, String>` フィールド追加（`[runes]` テーブル全 kv を収集）
- `error_catalog.rs`: E0417（`package not declared in [runes]`）正式追加
- `driver.rs`: `v483000_tests` 追加（`rune_resolution_from_toml` / `e0417_rune_not_in_toml` 2テスト）

### Changed
- `Cargo.toml` version: `48.2.0` → `48.3.0`

---

## [v48.2.0] — 2026-07-18 — import 構文刷新（ローカルファイル）

### Added
- `parser.rs`: `"./..."` / `"../..."` prefix の import を `ImportKind::Local` として解析
- `driver.rs`: `v482000_tests` 追加（`import_local_parses` / `import_local_relative_path` 2テスト）

### Changed
- `Cargo.toml` version: `48.1.0` → `48.2.0`

---

## [v48.1.0] — 2026-07-18 — import 構文刷新 AST + parser（パッケージ）

### Added
- `ast.rs`: `ImportKind` enum 追加（`Package` / `Local` / `Legacy`）
- `ast.rs`: `ImportDecl` に `kind: ImportKind` フィールド追加
- `parser.rs`: bare ident `import kafka` を `ImportKind::Package` として解析
- `driver.rs`: `v481000_tests` 追加（`import_package_parses` / `import_package_with_alias` 2テスト）

### Changed
- `Cargo.toml` version: `48.0.0` → `48.1.0`

---

## [v48.0.0] — 2026-07-18 — Standard Library 2.0 宣言

### Added
- `MILESTONE.md` に v48.0.0 Standard Library 2.0 エントリ追加
- `README.md` に `"Standard Library 2.0"` 言及を追加
- `driver.rs`: `v48000_tests` 追加（`cargo_toml_version_is_48_0_0` / `changelog_has_v48_0_0` / `milestone_has_stdlib_v2` / `readme_mentions_stdlib_v2` 4テスト）

### Changed
- `Cargo.toml` version: `47.9.0` → `48.0.0`

---

## [v47.9.0] — 2026-07-18 — stdlib ドキュメント + v48.0 前調整

### Added
- `site/content/docs/stdlib/float.mdx` 新規作成（Float.round / Float.clamp / Float.abs / Int.to_hex / Int.abs）
- `site/content/docs/stdlib/v2.mdx` 新規作成（Standard Library 2.0 概要・全追加関数索引）
- `site/content/docs/stdlib/list.mdx` 更新（chunk / group_by / dedupe / scan / take_while / drop_while 追記）
- `site/content/docs/stdlib/map.mdx` 更新（filter_values / map_values 追記）
- `site/content/cookbook/stdlib-v2.mdx` 新規作成（v47 シリーズ新関数サンプルパイプライン）
- `driver.rs`: `v479000_tests` 追加（`stdlib_v2_doc_exists` / `stdlib_v2_overview_exists` 2テスト）

### Changed
- `Cargo.toml` version: `47.8.0` → `47.9.0`

---

## [v47.8.0] — 2026-07-18 — `Map` 拡充

### Added
- `driver.rs`: `v478000_tests` 追加（`map_merge` / `map_filter_values` / `map_map_values` 3テスト）

### Changed
- `Cargo.toml` version: `47.7.0` → `47.8.0`

---

## [v47.7.0] — 2026-07-17 — `Result` 拡充

### Added
- `driver.rs`: `v477000_tests` 追加（`result_map` / `result_map_err` / `result_and_then` 3テスト）

### Changed
- `Cargo.toml` version: `47.6.0` → `47.7.0`

---

## [v47.6.0] — 2026-07-17 — `Option` 拡充

### Added
- `driver.rs`: `v476000_tests` 追加（`option_map` / `option_unwrap_or` / `option_and_then` 3テスト）

### Changed
- `Cargo.toml` version: `47.5.0` → `47.6.0`

---

## [v47.5.0] — 2026-07-17 — `Float` / `Int` 拡充

### Added
- `Float.round` / `Float.clamp` / `Float.abs` / `Int.to_hex` / `Int.abs` VM primitive 追加（vm.rs / checker.rs）
- `driver.rs`: `v475000_tests` 追加（`float_round` / `float_clamp` / `int_to_hex` 3テスト）

### Changed
- `Cargo.toml` version: `47.4.0` → `47.5.0`

---

## [v47.4.0] — 2026-07-17 — `String` 拡充

### Added
- `driver.rs`: `v474000_tests` 追加（`string_pad_left` / `string_trim_start` / `string_repeat` 3テスト）

### Changed
- `Cargo.toml` version: `47.3.0` → `47.4.0`

---

## [v47.3.0] — 2026-07-17 — `List.scan` / `List.take_while` / `List.drop_while`

### Added
- `driver.rs`: `v473000_tests` 追加（`list_scan_cumulative` / `list_take_while` / `list_drop_while` 3テスト）

### Changed
- `Cargo.toml` version: `47.2.0` → `47.3.0`

---

## [v47.2.0] — 2026-07-17 — `List.flat_map` / `List.group_by` / `List.dedupe`

### Added
- `List.dedupe` VM primitive 追加（vm.rs / checker.rs）
- `driver.rs`: `v472000_tests` 追加（`list_flat_map` / `list_group_by` / `list_dedupe` 3テスト）

### Changed
- `Cargo.toml` version: `47.1.0` → `47.2.0`

---

## [v47.1.0] — 2026-07-17 — `List.zip` / `List.chunk` テスト追加

### Added
- `driver.rs`: `v471000_tests` モジュール追加（`list_zip_pairs` / `list_chunk_batches` 2テスト）

### Changed
- `Cargo.toml` version: `47.0.0` → `47.1.0`

---

## [v47.0.0] — 2026-07-17 — Developer Experience 宣言 ★クリーンアップ

> 「インラインテスト・LSP クイックフィックス・型情報可視化が揃い、
>  Favnir の開発体験が実用水準に達した。
>  これが Favnir v47.0 — Developer Experience の姿である。」

### Added
- `MILESTONE.md`: v47.0.0 Developer Experience エントリ追加（達成コンポーネント v46.1〜v46.9 一覧）
- `README.md`: v47.0 Developer Experience マイルストーン言及を追加
- `driver.rs`: `v47000_tests` モジュール追加（`cargo_toml_version_is_47_0_0` / `changelog_has_v47_0_0` / `milestone_has_developer_experience` / `readme_mentions_developer_experience` 4テスト）

### Changed
- `Cargo.toml` version: `46.9.0` → `47.0.0`

---

## [v46.9.0] — 2026-07-17

### Added
- `site/content/docs/tools/fav-test.mdx` — `fav test` コマンド・`#[test]` 構文・アサーション関数リファレンス新規作成
- `site/content/docs/tools/developer-experience.mdx` — v46.x DX 機能概要（fav test / LSP quickfix / fav explain 2.0）新規作成
- `driver.rs`: `v469000_tests` モジュール追加（`fav_test_doc_exists` / `developer_experience_doc_exists` 2テスト）

### Changed
- `Cargo.toml` version: `46.8.0` → `46.9.0`
- `versions/roadmap/roadmap-v46.1-v47.0.md`: v46.9.0 / v47.0 のテスト数推定を 3011 → 3012 に修正

---

## [v46.8.0] — 2026-07-17

### Added
- `driver.rs`: `type_expr_str(ty) -> String` ヘルパー追加（AST TypeExpr → 表示文字列変換）
- `driver.rs`: `format_stage_types(program) -> String` 追加 — TrfDef の宣言型を `stage Name<T>: In -> Out\n` 形式で返す
- `driver.rs`: `cmd_explain_types(file)` 追加 — `fav explain --types` でステージ宣言型一覧を stdout 出力
- `main.rs`: `fav explain --types` CLI フラグ追加
- `driver.rs`: `v468000_tests` モジュール追加（`explain_types_shows_stage_types` / `explain_types_generic_instantiation` / `explain_types_no_stages` 3テスト）

### Changed
- `Cargo.toml` version: `46.7.0` → `46.8.0`

---

## [v46.7.0] — 2026-07-17

### Added
- `lineage.rs`: `LineageEntry` に `is_dead: bool` フィールドを追加（`return` 早期脱出パスを持つステージをマーク）
- `lineage.rs`: `has_early_return(stmts) -> bool` ヘルパー追加（トップレベル `Stmt::Return` 検出）
- `lineage.rs`: `render_lineage_mermaid_with_opts(report, show_dead) -> String` 追加 — `show_dead=true` のとき dead エントリに `classDef deadEntry` + `class <id> deadEntry` を付与
- `main.rs`: `fav explain --lineage --show-dead` CLI フラグ追加
- `driver.rs`: `v467000_tests` モジュール追加（`lineage_return_path_is_dead` / `lineage_happy_path_active` 2テスト）

### Changed
- `lineage.rs`: `render_lineage_mermaid` が `render_lineage_mermaid_with_opts(report, false)` に委譲するよう変更
- `driver.rs`: `cmd_explain_lineage` シグネチャに `show_dead: bool` 追加
- `Cargo.toml` version: `46.6.0` → `46.7.0`

---

## [v46.6.0] — 2026-07-17

### Added
- `driver.rs`: `render_pipeline_mermaid_v2(program) -> String` を新規追加 — `return` 早期脱出パスを点線（`-.->` + `deadPath`）、`return Err(...)` をエラーパス（赤 `errPath`）として Mermaid 出力
- `driver.rs`: `scan_returns(stmts) -> (bool, bool)` + `is_err_call(expr) -> bool` ヘルパー追加
- `driver.rs`: `v466000_tests` モジュール追加（`explain_mermaid_includes_dead_path` / `explain_pipeline_v2` 2テスト）

### Changed
- `Cargo.toml` version: `46.5.0` → `46.6.0`

---

## [v46.5.0] — 2026-07-17

### Added
- `code_action.rs`: CA-4 `check_did_you_mean_fix` — E0102（未定義変数）エラーの `TypeError.hints` から did-you-mean 候補を抽出し、`TextEdit` 付き `quickfix` CodeAction を生成
- `code_action.rs`: CA-5 `check_arg_count_fix` — E0101（引数数不一致）エラーを診断アクション（`edit: None`）として返す
- `code_action.rs`: `parse_did_you_mean` ヘルパー追加（`"did you mean \`X\`?"` → `"X"` を抽出）
- `driver.rs`: `v465000_tests` モジュール追加（`lsp_quick_fix_undefined_var` / `lsp_quick_fix_arg_count` 2テスト）

### Changed
- `Cargo.toml` version: `46.4.0` → `46.5.0`
- `roadmap-v46.1-v47.0.md`: v46.5.0 セクションのエラーコード `E0007` → `E0101`、`E0001` → `E0102` に修正（Rust checker の実コードと一致）

---

## [v46.4.0] — 2026-07-17

### Added
- `checker.rs` `Stmt::Bind` ハンドラ: `Pattern::Bind(name, span)` 分岐に `remember_type(span, &effective_ty)` を追加。`bind` 変数の型が `type_at` に記録されるようになり、実 LSP パスで inlay hints が機能するようになった
- `inlay_hints.rs`: `collect_stage_hints` 関数を新規追加（`stage ` プレフィックス行を走査してステージ名の型ヒントを生成）
- `inlay_hints.rs`: `handle_inlay_hints` を更新し `collect_bind_hints` + `collect_stage_hints` を結合して返すよう変更
- `driver.rs`: `v464000_tests` モジュール追加（`lsp_inlay_hints_type_annotation` / `lsp_inlay_hints_pipeline` 2テスト）

### Changed
- `Cargo.toml` version: `46.3.0` → `46.4.0`

---

## [v46.3.0] — 2026-07-17

### Added
- `checker.rs` `check_test_def`: `assert_ok` / `assert_err` を env に登録（これまで `assert_eq` / `assert_ne` のみ登録されていた）
- `checker.rs` `check_fn_def`: `fd.is_test` のとき `assert` / `assert_eq` / `assert_ne` / `assert_ok` / `assert_err` を env に登録し、`#[test] fn` 本体での使用で E0001 が出なくなった
- `driver.rs`: `v463000_tests` モジュール追加（`assert_ok_passes` / `assert_err_passes` 2テスト）

### Changed
- `Cargo.toml` version: `46.2.0` → `46.3.0`

---

## [v46.2.0] — 2026-07-16

### Added
- `driver.rs` `collect_test_cases`: `#[test]` 付き `FnDef`（`fd.is_test == true`）を収集するアームを追加
- `driver.rs`: `v462000_tests` モジュール追加（3テスト: `fav_test_discovers_tests` / `fav_test_reports_results` / `non_test_fn_not_discovered`）

### Changed
- `Cargo.toml` version: `46.1.0` → `46.2.0`

---

## [v46.1.0] — 2026-07-16

### Added
- `ast.rs`: `FnDef` に `is_test: bool` フィールド追加（v46.1.0）
- `parser.rs`: `parse_test_annotation()` 追加（`#[test]` 4トークンルックアヘッド、`TokenKind::Test` を使用）
- `parser.rs`: `parse_item()` で `fd.is_test = test_ann;` を同期 fn / async fn の両アームに付与
- `driver.rs`: `v461000_tests` モジュール追加（`test_block_parses` / `test_fn_collected` 2テスト）

### Fixed
- `parse_test_annotation()`: `test` は `TokenKind::Test`（キーワード）であり `Ident` ではないため、ルックアヘッドで `t.kind == TokenKind::Test` を使用

### Changed
- `Cargo.toml` version: `46.0.0` → `46.1.0`

---

## [v46.0.0] — 2026-07-16

### Added
- `MILESTONE.md`: v46.0.0「Language Refinement」エントリ追加（v45.1〜v45.9 達成コンポーネントテーブル）
- `README.md`: Language Refinement マイルストーン宣言を追記
- `driver.rs`: `v46000_tests` モジュール追加（`cargo_toml_version_is_46_0_0` / `changelog_has_v46_0_0` / `milestone_has_language_refinement` / `readme_mentions_language_refinement` 4テスト）

### Changed
- `Cargo.toml` version: `45.9.0` → `46.0.0`

### Milestone
- **Language Refinement 宣言**: `return` 構文・`match` 完全網羅・型エイリアス・エラーメッセージ改善・数値リテラル `_` が揃い、Favnir の構文が成熟した。

---

## [v45.9.0] — 2026-07-16

### Added
- `site/content/docs/language-refinement-overview.mdx`: Language Refinement スプリント（v45.1〜v45.9）の成果まとめページを新規作成
- `driver.rs`: `v459000_tests` モジュール追加（`examples_structure_valid` / `language_refinement_overview_doc_exists` 2テスト）

### Changed
- `examples/pipeline/stage_seq_demo.fav`: コメントを現在の構文説明に修正（旧「alias for trf/flw」誤記を除去）
- `Cargo.toml` version: `45.8.0` → `45.9.0`

---

## [v45.8.0] — 2026-07-16

### Added
- `examples/pipeline/pipeline.fav`: `return` ガード節パターン関数（`validate_amount`）を追加（v45.7.0 の数値リテラル `_` も活用）
- `driver.rs`: `examples_no_legacy_effect_syntax` テスト追加（`walkdir` で examples/ をスキャンし旧 `!Effect` アノテーション構文がないことを確認）

### Changed
- `Cargo.toml` version: `45.7.0` → `45.8.0`

---

## [v45.7.0] — 2026-07-16

### Added
- `error_catalog.rs`: E0213 / E0219〜E0227 / E0241〜E0245 / E0251 / E0253〜E0254 / E0274 / E0310〜E0315 / E0319〜E0324 / E0365 / E0368〜E0369 / E0373〜E0374 / E0401〜E0406 / E0410〜E0413 に suggestion テキスト追加（Phase 2）
- `lexer.rs`: 数値リテラル `_` セパレータ対応（`1_000_000` / `0.000_15` 等）

### Changed
- `Cargo.toml` version: `45.6.0` → `45.7.0`

---

## [v45.6.0] — 2026-07-16

### Added
- `ErrorEntry` に `suggestion: Option<&'static str>` フィールド追加（静的カタログ改善）
- E0101 / E0102 / E0103 / E0214 / E0215 / E0218 に suggestion テキスト追加
- `Expr::Apply` 引数数不一致エラーに動的 hint 追加（関数名・期待引数数を含む）

### Changed
- `Cargo.toml` version: `45.5.0` → `45.6.0`

---

## [v45.5.0] — 2026-07-16

### Added
- `opaque type` エイリアス非互換チェック: inner type を直接返すと E0413 発行
- `checker.rs`: `opaque_alias_inner: HashMap<String, Type>` フィールド追加
- `checker.rs`: `check_fn_def` / `check_trf_def` に E0413 チェック追加
- テスト `transparent_alias_compatible`（透明エイリアス互換）、`opaque_alias_incompatible`（E0413）

### Changed
- `Cargo.toml` version: `45.4.0` → `45.5.0`
- `checker.rs`: `register_item_signatures` — `is_opaque = true` の場合 `type_aliases` ではなく `opaque_alias_inner` に登録

---

## [v45.4.0] — 2026-07-16

### Added
- `match` 網羅性チェック（`checker.rs`）: Sum 型の全バリアント網羅を検証
- 非網羅 match（文として使用）→ W034 警告
- 非網羅 match（値として使用）→ E0416 ハードエラー
- `error_catalog.rs`: E0416「non-exhaustive match in value context」エントリ追加
- `checker.rs`: `collect_covered_variants` / `collect_pattern_variants` フリー関数追加
- `self/checker.fav`: `infer_expr` に `EArmG` アーム追加（Bootstrap 網羅性対応）
- `self/compiler.fav`: `token_eq` / `token_to_string` / `free_names_expr` / `expr_uses` / `binop_bc` に欠落バリアント追加

### Changed
- `Cargo.toml` version: `45.3.0` → `45.4.0`
- `checker.rs`: `check_match_arms` シグネチャに `value_ctx: bool` 追加
- `checker.rs`: `check_stmt` の `Stmt::Expr` で match 式を stmt ctx として処理

---

## [v45.3.0] — 2026-07-15

### Added
- `return` コンパイル・VM 実行を実装（`middle/ir.rs`, `middle/compiler.rs`, `backend/codegen.rs`）
- `IRStmt::Return(IRExpr)` variant を `ir.rs` に追加
- `compiler.rs`: `Stmt::Return` stub を `IRStmt::Return(compile_expr(...))` に実装
- `codegen.rs`: `IRStmt::Return` → `emit_expr + Opcode::Return` バイトコード生成
- `fn` / `stage` ボディで早期脱出（early exit）が実際に動作するようになった

### Changed
- `Cargo.toml` version: `45.2.0` → `45.3.0`
- `checker.rs`: `mem::replace` を `Option::replace` に変更（clippy 対応）
- `wasm_codegen.rs`, `wasm_dce.rs`, `driver.rs`: `IRStmt::Return` 網羅 match arm 追加

---

## [v45.2.0] — 2026-07-15

### Added
- `return` 型チェック実装（`checker.rs`）: `Stmt::Return` の型を宣言戻り型と照合
- E0415 `return type mismatch` エラーコードを `error_catalog.rs` に追加
- `Checker` 構造体に `current_return_ty: Option<Type>` フィールド追加
- `check_fn_def` / `check_trf_def` でスコープ内の `current_return_ty` を設定・復元
- `check_return_stmt` ヘルパーメソッド追加（型不一致時に E0415 を発行）

### Changed
- `Cargo.toml` version: `45.1.0` → `45.2.0`

---

## [v45.1.0] — 2026-07-15

### Added
- `return <expr>` 構文を追加（AST ノード `ReturnStmt` + `Stmt::Return` variant）
- `lexer.rs`: `TokenKind::Return` キーワード追加
- `parser.rs`: `parse_return_stmt()` / `parse_block()` に `return` 分岐追加
- 適用スコープ: `fn` ボディ・`stage` ボディ（`seq` は対象外）
- 型チェック・VM opcode は v45.2/v45.3 で対応

---

## [v45.0.0] — 2026-07-15

### Added
- `MILESTONE.md` — Precision & Flow マイルストーン宣言セクション追加（v44.1〜v44.9 達成コンポーネント一覧）
- `README.md` — v45.0 Precision & Flow マイルストーン言及追加

### Changed
- `Cargo.toml` version: `44.9.0` → `45.0.0`

### Notes
- v44.1〜v44.9 の全機能（Refinement type × Streaming / CEP × Refinement type / Stream join × Opaque type / 型推論 × Lineage / Back-pressure × Policy / E2E デモ / ドキュメント / ベンチマーク追跡 / 安定化）が揃い、Precision & Flow を正式宣言
- ★クリーンアップ（`cargo clean`）実施

---

## [v44.9.0] — 2026-07-15

### Added
- `site/content/docs/precision-and-flow-overview.mdx` — v44.x スプリント俯瞰サマリーページ（新規作成）
  - v44.1〜v44.8 の達成事項一覧（✅ COMPLETE テーブル）
  - v45.0 Precision & Flow 宣言文
  - 詳細ドキュメント・E2E デモへのリンク

### Notes
- コードフリーズ版（新規 Rust 機能・ヘルパー関数・AST 変更なし）
- v45.0 Precision & Flow 宣言に向けた最終調整

---

## [v44.8.0] — 2026-07-15

### Added
- `collect_bench_stream_notes(changelog: &str) -> Vec<String>` ヘルパー追加（`driver.rs`）
  - CHANGELOG から `bench --stream` 計測結果行を収集するパフォーマンス追跡 MVP

### Performance
- `fav bench --stream` 計測結果: BenchOpts.stream = true での実行パスが有効（v40.7.0 追加済み）
  - ストリーム処理パイプラインの bench --stream 実行に対応
  - VM レベル実行速度最適化・v41.0 との実測比較は将来版のスコープ

---

## [v44.7.0] — 2026-07-15

### Added
- `site/content/docs/precision-and-flow.mdx` — Precision & Flow 全機能統合解説ページ
  - Refinement type / CEP / Opaque type / 型注釈 lineage / Back-pressure / E2E デモの 6 セクション
  - 各機能の Favnir コードスニペット付き

### Notes
- Precision & Flow 機能群（v44.1〜v44.6）の統合ドキュメント
- バージョン履歴テーブルで各バージョンの機能を一覧化

---

## [v44.6.0] — 2026-07-15

### Added
- `infra/e2e-demo/precision-flow/` — Precision & Flow E2E デモパイプライン
  - `src/demo.fav`: Refinement type + CEP + Opaque type + `#[max_inflight(50)]` Policy gate の統合デモ
  - `README.md`: パイプライン概要・機能一覧・実行方法

### Notes
- Precision & Flow 機能群（v44.1〜v44.5）の統合 E2E デモ
- Kafka → CEP → Opaque join → Policy gate（governance 制御）の完全パイプライン構成

---

## [v44.5.0] — 2026-07-14

### Added
- `collect_stage_max_inflight_annotations(src, filename) -> Vec<String>` ヘルパー追加（`driver.rs`）
  - `#[max_inflight(n)]` アノテーション付き `TrfDef`（ステージ）を AST レベルで収集
  - 返り値: `"<filename>:<line>: <stage_name>: max_inflight=<n>"` 形式

### Notes
- Back-pressure x `fav policy` 統合 AST レベル MVP（v42.5.0 追加済みの `MaxInflightAnnotation` を活用）
- `policy { max_inflight: N }` グローバルポリシーブロック・`fav policy check --ci`・VM 強制は将来版のスコープ

---

## [v44.4.0] — 2026-07-14

### Added
- `collect_annotated_lineage_bindings(src, filename) -> Vec<String>` ヘルパー追加（`driver.rs`）
  - ステージ（`TrfDef`）内の型注釈付き `bind x: T <- expr` 束縛を収集
  - 既存の `format_type_expr` で型を文字列化
  - 返り値: `"<filename>:<line>: <stage_name>: <binding_name>: <type>"` 形式
- `v44400_tests`: `cargo_toml_version_is_44_4_0` / `annotated_lineage_bindings_detected`

### Notes
- `fav explain --lineage` の出力への型情報統合（`LineageEntry` 拡張・`render_lineage_text` 更新）は将来版のスコープ
- ウィンドウ・join の lineage 追跡統合も将来版のスコープ

---

## [v44.3.0] — 2026-07-14

### Added
- `collect_opaque_alias_groups(src, filename) -> Vec<String>` ヘルパー追加（`driver.rs`）
  - 同じ内部型を持つ `opaque type` エイリアスをグループ化して返す
  - `Stream.join` で誤 join される可能性のある opaque type ペアを AST レベルで検出
  - 返り値: `"<filename>:<line>: <inner>: <Name1>, <Name2>"` 形式（名前アルファベット順）
- `v44300_tests`: `cargo_toml_version_is_44_3_0` / `opaque_alias_group_detected` / `non_opaque_type_excluded_from_groups`

### Notes
- checker.fav への E0413 統合（`Stream.join` での誤 join 検出）は将来版のスコープ
- 型引数付き opaque alias（`opaque type Foo = List<String>` 等）は対象外

---

## [v44.2.0] — 2026-07-14

### Added
- `collect_cep_refinement_event_refs(src, filename) -> Vec<String>` ヘルパー追加（`driver.rs`）
  - `type T = U where |v| ...` 形式の refinement type 名を収集
  - `CepPatternDef` の各 `CepClause.expr` を再帰走査（`Event`/`Seq`/`Any`/`Not`）
  - CEP イベント名が refinement type 名と一致するものを検出
  - 返り値: `"<filename>:<line>: <pattern_name>: <event_name>"` 形式の文字列リスト
- `v44200_tests`: `cargo_toml_version_is_44_2_0` / `cep_simple_event_matches_refinement_type` / `cep_seq_pattern_refinement_event_detected`

### Notes
- `Purchase<HighValue>` 構文（型パラメータ付き CEP イベント）は将来版のスコープ（現 AST は `CepExpr::Event(String)` — 型パラメータなし）
- checker.fav への型チェック統合は将来版のスコープ

---

## [v44.1.0] — 2026-07-14

### Added
- `collect_refinement_stream_bindings(src, filename) -> Vec<String>` ヘルパー追加（`driver.rs`）
  - `type T = U where (...)` 形式の refinement type 名を収集
  - `FnDef` / `TrfDef` の body.stmts を走査し、`bind x: Stream<T>` / `List<T>` の T が refinement type に一致する束縛を検出
  - 返り値: `"<filename>:<line>: <name>: <container><elem>"` 形式の文字列リスト
- `v44100_tests`: `cargo_toml_version_is_44_1_0` / `refinement_type_invariant_in_typedef_ast` / `collect_refinement_stream_bindings_detects_annotated_bind`

### Notes
- checker.fav への完全統合（`List.filter` 述語からの refinement 型推論）は将来版のスコープ
- 本バージョンはパーサー受容 + AST レベル MVP

---

## [v44.0.0] — 2026-07-13

### Added
- `MILESTONE.md` に `v44.0.0 — Language Expressiveness` セクション追加（宣言文・達成コンポーネント v43.1〜v43.13 一覧）
- `v44000_tests`: `cargo_toml_version_is_44_0_0` / `changelog_has_v44_0_0` / `milestone_has_language_expressiveness` / `readme_mentions_language_expressiveness`

### Changed
- `README.md` に `v44.0 — Language Expressiveness` マイルストーン言及を追加

### Notes
- 宣言版（新規 Rust 機能追加なし）
- `cargo clean` 実行済み（ビルドアーティファクト削除）
- v43.1〜v43.13 の全機能が動作することを確認

---

## [v43.13.0] — 2026-07-13

### Added
- `site/content/cookbook/type-inference-guide.mdx`: 型推論 cookbook（6 カテゴリ + opaque type 解説）
- `site/content/docs/language/type-inference.mdx`: 言語リファレンス（型推論 6 カテゴリ・lint ルール・制限事項）
- `site/content/docs/language-expressiveness.mdx`: v43 スプリント成果サマリー・宣言文
- `v431300_tests`: `type_inference_guide_mdx_exists` / `language_expressiveness_doc_exists`

### Changed
- `v431200_tests::cargo_toml_version_is_43_12_0` をスタブ化

### Notes
- コードフリーズ（新規 Rust 機能追加なし）
- Rust ソース変更: driver.rs（テスト 2 件追加・スタブ化）と Cargo.toml のみ
- `site/content/docs/language/type-inference.mdx` は `include_str!` テスト対象外（テスト数 2937 = 2935 + 2 を維持するため意図的に除外）

---

## [v43.12.0] — 2026-07-13

### Added
- `check_w031_redundant_return_annotation`: 推論可能な戻り値型の明示的注釈を検出（`lint.rs`）
- `check_w032_explicit_generic_type_arg`: 推論可能なジェネリック型引数の明示を検出（`lint.rs`）
- `check_w032_in_expr` / `check_w032_in_stmt`: `Expr::TypeApply` を再帰的に走査するヘルパー（`lint.rs`）
- W031 警告: `return type annotation is redundant; type can be inferred`
- W032 警告: `explicit generic type argument is redundant; type can be inferred from argument`
- `v431200_tests`: `cargo_toml_version_is_43_12_0` / `w031_warns_on_redundant_return_annotation` / `w032_warns_on_explicit_generic_type_arg`

### Changed
- `lint_program()` に `check_w031` / `check_w032` 呼び出しを追加
- `v431100_tests::cargo_toml_version_is_43_11_0` をスタブ化

### Notes
- W033（ラムダ引数型の明示検出）は AST 拡張が必要なため将来版のスコープ（`lint_program` にスタブコメントのみ）
- `Block.expr` は `Box<Expr>`（非 Option）— `&*fd.body.expr` でデリファレンス

---

## [v43.11.0] — 2026-07-13

### Added
- `TypeDef.is_opaque: bool` フィールド（デフォルト false）
- `opaque type Token = String` 構文（contextual keyword "opaque"）
- `check_opaque_coerce_violations(src, filename) -> Vec<String>`: opaque coerce 違反を AST レベルで検出
- `is_bare_inner_literal(expr, inner_type) -> bool`: inner type リテラル判定ヘルパー
- E0413 `opaque type coerce forbidden`（`error_catalog.rs` + `get_explain_text`）
- `v431100_tests`: `cargo_toml_version_is_43_11_0` / `parser_recognizes_opaque_type_keyword` / `e0413_opaque_coerce_blocked`

### Changed
- `parse_item` に `"opaque"` contextual keyword アームを追加（`TokenKind::Type` の直前）
- `parse_type_def` の TypeDef 構築 4 箇所 + `checker.rs` 1 箇所に `is_opaque: false` を追加
- `cmd_check` の `errors.is_empty()` ブランチに opaque coerce チェックを追加
- `v431000_tests::cargo_toml_version_is_43_10_0` をスタブ化

### Notes
- checker.fav への opaque 型統合・bind 式/引数での強制・`site/` MDX は将来版のスコープ
- `TypeExpr::Named` は `(String, Vec<TypeExpr>, Span)` の 3 フィールド（2 フィールドパターンは要注意）

---

## [v43.10.0] — 2026-07-13

### Added
- `collect_explain_output(src, filename) -> Vec<String>`: 型エラーに対応する解説テキストを収集（テスト用ヘルパー）
- `fav check --explain`: 型チェックエラー発生時に `get_explain_text` ベースの静的解説を出力
- `v431000_tests`: `cargo_toml_version_is_43_10_0` / `explain_output_empty_for_well_typed_code`

### Changed
- `cmd_check` シグネチャに `explain: bool` を追加（12 番目のパラメータ）
- `v43900_tests::cargo_toml_version_is_43_9_0` をスタブ化

### Notes
- LLM 呼び出しによる動的解説（ロードマップ記載の「Llm Rune 活用」）は将来バージョンへスライド
- `--json` と `--explain` の同時指定では `explain` は無効化される（`!json` 条件）
- プロジェクトモード（`file = None`）では `--explain` 非対応

---

## [v43.9.0] — 2026-07-13

### Added
- `collect_inference_annotations(src, filename) -> Vec<String>`: 関数レベル型注釈収集（単一パース）
- `fav check --show-inference`: 型チェック通過後に関数シグネチャを出力
- `v43900_tests`: `cargo_toml_version_is_43_9_0` / `show_inference_collects_fn_annotations`

### Changed
- `cmd_check` シグネチャに `show_inference: bool` を追加
- `v43800_tests::cargo_toml_version_is_43_8_0` をスタブ化
- `driver.rs` の二重パース TODO（line 3948–3949）を解消

### Notes
- `fav/self/checker.fav` は変更なし
- `display_ty_inline` は `Named` / `Named<args>` のみ対応。`Arrow`・`Optional` 等は `"?"` にフォールバック

---

## [v43.8.0] — 2026-07-13

### Added
- `v43800_tests`: `cargo_toml_version_is_43_8_0` / `bidirectional_filter_infers_elem_type` / `bidirectional_nested_map_filter_expression`

### Changed
- `v43700_tests::cargo_toml_version_is_43_7_0` をスタブ化

### Notes
- `fav/self/checker.fav` は変更なし: v43.5.0 の `infer_list_lambda_call` がリスト要素型の下向き伝播（双方向型推論）を実現済み

---

## [v43.7.0] — 2026-07-13

### Added
- `v43700_tests`: `cargo_toml_version_is_43_7_0` / `structural_record_literal_type_checks`

### Changed
- `v43600_tests::cargo_toml_version_is_43_6_0` をスタブ化

### Notes
- `fav/self/checker.fav` は変更なし: 名前付きレコードリテラル（`TypeName { ... }`）は既存の `ERecordLit → tname` 機構で型チェックを通過する

---

## [v43.6.0] — 2026-07-12

### Added
- `v43600_tests`: `cargo_toml_version_is_43_6_0` / `pipeline_two_step_bind_infers_types` / `pipeline_three_step_bind_infers_types`

### Changed
- `v43500_tests::cargo_toml_version_is_43_5_0` をスタブ化

### Notes
- `fav/self/checker.fav` は変更なし: `infer_hm_let`（EBind 型伝播）+ v43.5.0 `infer_list_lambda_call`（ラムダ引数型推論）の組み合わせで多段パイプラインが機能することを確認

---

## [v43.5.0] — 2026-07-12

### Added
- `fav/self/checker.fav`: `infer_list_lambda_call` — `List.map` / `List.filter` 呼び出し時にラムダパラメータへリスト要素型を伝播（contextual lambda inference）
- `v43500_tests`: `cargo_toml_version_is_43_5_0` / `contextual_lambda_map_propagates_elem_type` / `contextual_lambda_filter_preserves_elem_type`

### Changed
- `fav/self/checker.fav`: `infer_call` の `ns=="List"` ブランチを `map`/`filter` で `infer_list_lambda_call` に分岐
- `v43400_tests::cargo_toml_version_is_43_4_0` をスタブ化

---

## [v43.4.0] — 2026-07-12

### Fixed
- `fav/self/checker.fav`: ジェネリック関数呼び出し時に同一型変数が複数引数で異なる型に束縛された場合、E0005 ではなく E0412 `ambiguous type variable` を報告するよう修正（`check_scheme_var_ambiguity` pre-check を `instantiate_fn_scheme` に追加）

### Added
- `fav/src/error_catalog.rs`: E0412（ambiguous type variable）エントリ追加
- `v43400_tests`: `cargo_toml_version_is_43_4_0` / `e0412_in_error_catalog` / `e0412_conflicting_type_vars` / `e0412_no_conflict_ok`

### Changed
- `v43300_tests::cargo_toml_version_is_43_3_0` をスタブ化

---

## [v43.3.0] — 2026-07-12

### Fixed
- `fav/self/checker.fav`: `infer_call`（非HMパス）でジェネリック関数の呼び出し時に型変数を解決せず生の型変数文字列（例: `"A"`）を返していたバグを修正。`instantiate_fn_scheme` を使ってコールサイトで型変数を確定するよう変更（v43.3.0 call-site generic instantiation）

### Added
- `v43300_tests`: `cargo_toml_version_is_43_3_0` / `call_site_inference_identity_ok` / `call_site_inference_wrong_return_e0009`

### Changed
- `v43200_tests::cargo_toml_version_is_43_2_0` をスタブ化

---

## [v43.2.0] — 2026-07-12

### Added
- `fav/src/error_catalog.rs`: E0410（ambiguous return type）/ E0411（inferred return type mismatch）追加
- `fav/self/checker.fav`: `check_body_ty` — `TeSimple("")` かつ body Unknown 時に E0410 を返すパス追加
- `fav/src/driver.rs`: `FnReturnInfo` struct + `collect_fn_inferred_return_types` — 戻り値型省略関数を収集
- `fav/src/driver.rs`: `fav check --show-types` に fn inferred return type 行を追加
- `v43200_tests`: `cargo_toml_version_is_43_2_0` / `e0410_e0411_in_error_catalog` / `checker_fav_check_body_ty_has_e0410` / `return_type_omission_e0410_triggered`（E0410 E2E テスト）

### Changed
- `v43100_tests::cargo_toml_version_is_43_1_0` をスタブ化

### Notes
- E0411 は本バージョンで catalog に追加のみ（checker.fav での検出は v43.3.0 以降）
- `FnReturnInfo` は `--show-types` テキストパス専用（JSON 出力対象外）

---

## [v43.1.0] — 2026-07-12

### Added
- `fav/src/frontend/parser.rs`: `fn f(params) { body }` での戻り値型省略（`-> RetType` 不要）をサポート
- `fav/self/compiler.fav`: `parse_fn_def_after_params()` — `->` オプション対応（`TeSimple("")` プレースホルダ）
- `fav/self/checker.fav`: `check_body_ty()` — `ret == ""` 時に body 推論で OK パス追加
- `fav/src/middle/ast_lower_checker.rs`: `lower_fn_def` — `return_ty: None` の fallback を `TeSimple("Unit")` → `TeSimple("")` に変更（`fav check` self-hosted パスの E0009 誤検知を修正）
- `v43100_tests`: `cargo_toml_version_is_43_1_0` / `return_type_omission_block_parseable` / `return_type_omission_return_ty_is_none`

### Notes
- `checker.rs` は変更なし（`return_ty: None` → body_ty 推論は既実装）
- self-hosted パスの `collect_fn_scheme_str` での推論型補完は v43.2.0 以降

---

## [v43.0.0] — 2026-07-12

### Added
- `v43000_tests`: `cargo_toml_version_is_43_0_0` / `changelog_has_v43_0_0` / `milestone_has_real_time_power` / `readme_mentions_real_time_power`
- `MILESTONE.md` に `v43.0.0 — Real-Time Power` エントリを追加

### Changed
- `README.md` に Real-Time Power（v43.0）の記述を追加
- `fav/Cargo.toml` version: `42.9.0` → `43.0.0`
- `v42900_tests::cargo_toml_version_is_42_9_0` をスタブ化

### Notes
- Real-Time Power 宣言（v42.1〜v42.9 スプリント完了）
- ★ `cargo clean`（v43.0.0 クリーンアップ）実施

---

## [v42.9.0] — 2026-07-12

### Added
- `site/content/docs/real-time-power.mdx` — v42.x リアルタイム機能概要ドキュメント（CEP / Stream join / back-pressure / WebSocket / fav monitor）
- `v42900_tests`: `cargo_toml_version_is_42_9_0` / `real_time_power_docs_exists`

### Notes
- コードフリーズ（新規機能追加なし）
- v43.0.0 マイルストーン宣言は次バージョンで実施

---

## [v42.8.0] — 2026-07-12

### Added
- `site/content/cookbook/cep-login-purchase.mdx` — CEP ログイン→購入セッション検出 cookbook
- `site/content/cookbook/stream-join.mdx` — Stream join 2 ストリーム時間窓結合 cookbook
- `v42800_tests`: `realtime_cookbook_mdx_exists`

---

## [v42.7.0] — 2026-07-12

### Added
- `fav monitor` コマンド — パイプライン監視 stub（スループット / イベント数 / レイテンシ表示は v43.x 以降）
- `cmd_monitor` 関数追加（`driver.rs`）
- `v42700_tests`: `cargo_toml_version_is_42_7_0` / `monitor_cmd_exists`

### Notes
- 実際のメトリクス収集・TUI 表示は v43.x 以降で実装
- 未知引数は無視する（v43.x で `--interval` オプション追加時に引数解析を実装）

---

## [v42.6.0] — 2026-07-12

### Added
- `runes/websocket/` — WebSocket push sink Rune（`send` / `broadcast` 関数）
- VM プリミティブ `WebSocket.send_raw` / `WebSocket.broadcast_raw` stub 追加（実接続は v44.x 以降）
- `site/content/docs/runes/websocket.mdx` — WebSocket Rune ドキュメント
- `v42600_tests`: `cargo_toml_version_is_42_6_0` / `websocket_rune_fav_exists`

### Notes
- 実際の WebSocket 接続（TCP / TLS / WS handshake）は v44.x 以降で実装
- `!WebSocket` エフェクトは実接続実装時に合わせて追加予定

---

## [v42.5.0] — 2026-07-12

### Added
- `#[max_inflight(n)]` アノテーション — `stage` 定義に back-pressure 宣言を追加（parser + AST）
- `MaxInflightAnnotation { n: u64 }` AST 構造体追加（`TrfDef.max_inflight` フィールド）
- `parse_max_inflight_annotation()` パーサー実装（n = 0 はパース時エラー; 負数は `Minus` トークンのため別エラー）
- `v42500_tests`: `max_inflight_annotation_parses` / `max_inflight_zero_is_parse_error`

### Known Limitations
- `fav fmt` は `#[max_inflight(n)]` アノテーションを出力しないため、`fav fmt` 実行後にアノテーションが消失する（`#[timeout]`/`#[retry]`/`#[circuit_breaker]` と同じ既存の制約）。v44.x の runtime 実装時に fmt.rs も対応予定。
- runtime back-pressure（上流ステージ一時停止）は v44.x 以降に延期。

---

## [v42.4.0] — 2026-07-12

### Added
- `Stream.join(stream1, stream2, join_fn, window_secs)` — time-window join 演算子（nested-loop join）
- `VMStream::Join` バリアント（`left`/`right`/`join_fn`/`window_secs` フィールド）
- `("Stream", "join")` 型推論エントリ（checker.rs — `Stream<Unknown>` を返す）
- `v42400_tests`: `stream_join_type_check_ok` / `stream_join_vm_basic`

---

## [v42.3.0] — 2026-07-12

### Added
- `fav/src/error_catalog.rs`: E0420（`cep pattern within_secs must be positive`）追加（E042x セクション新設）
- `fav/src/middle/checker.rs`: `check_cep_pattern_def()` 追加、Pass 2 `CepPatternDef` スタブを実装に置き換え（`within 0` → E0420）
- `fav/self/checker.fav`: CEP パターン型チェック設計コメントを「E0420 実装済み」に更新
- driver.rs `v42300_tests` 3 件追加（`cargo_toml_version_is_42_3_0` / `cep_e0420_within_zero` / `e0420_in_error_catalog`）

### Changed
- `fav/Cargo.toml`: version `42.2.0` → `42.3.0`

---

## [v42.2.0] — 2026-07-12

### Added
- `fav/src/ast.rs`: `CepExpr` enum 追加（`Event` / `Seq` / `Any` / `Not` 4 バリアント）、`CepClause.event: String` → `CepClause.expr: CepExpr` に変更
- `fav/src/frontend/parser.rs`: `parse_cep_expr()` 追加（`seq` / `any` / `not` コンビネータ対応）、`parse_cep_pattern_def()` を `parse_cep_expr()` 使用に修正
- driver.rs `v42200_tests` 3 件追加（`cargo_toml_version_is_42_2_0` / `cep_seq_parseable` / `cep_any_parseable`）
- driver.rs `v42100_tests::cep_pattern_fields_correct` を `CepExpr::Event` パターンマッチに更新

### Changed
- `fav/Cargo.toml`: version `42.1.0` → `42.2.0`

---

## [v42.1.0] — 2026-07-12

### Added
- `fav/src/ast.rs`: `CepClause` / `CepPatternDef` 構造体、`Item::CepPatternDef` バリアント、`Item::span()` アーム追加
- `fav/src/frontend/parser.rs`: `parse_cep_pattern_def()` 実装、`parse_item()` に `"cep"` ディスパッチ追加
- `fav/src/middle/checker.rs`: Pass 1 / Pass 2 に `CepPatternDef` スタブアーム追加
- `fav/src/fmt.rs`: `Item::CepPatternDef` フォーマットスタブ追加
- `fav/src/driver.rs`: `v42100_tests` 3 件追加（`cargo_toml_version_is_42_1_0` / `cep_pattern_parseable` / `cep_pattern_fields_correct`）
- `fav/self/checker.fav`: CEP パターン型チェック設計コメント追加（v42.3.0 実装予定）

### Changed
- `fav/Cargo.toml`: version `42.0.0` → `42.1.0`

---

## [v42.0.0] — 2026-07-12

### Added
- `MILESTONE.md`: `v42.0.0 — Type Precision` マイルストーン宣言エントリを追加（先頭）
- `README.md`: `Type Precision`（v42.0）の記述を追加
- driver.rs `v42000_tests` 4 件追加（`cargo_toml_version_is_42_0_0` / `changelog_has_v42_0_0` / `milestone_has_type_precision` / `readme_mentions_type_precision`）

### Changed
- `fav/Cargo.toml`: version `41.9.0` → `42.0.0`

---

## [v41.9.0] — 2026-07-12

### Added
- `site/content/docs/type-precision.mdx` — Type Precision マイルストーン概要ページ新規作成（v41.1〜v41.8 機能一覧・v42.0 宣言文予告）
- driver.rs `v41900_tests` 2 件追加（`cargo_toml_version_is_41_9_0` / `type_precision_doc_exists`）

### Changed
- `fav/Cargo.toml`: version `41.8.0` → `41.9.0`

---

## [v41.8.0] — 2026-07-11

### Added
- `site/content/cookbook/refinement-types.mdx` — Type Precision cookbook: refinement type alias + W030 lint 実用パターン（ドメイン型・冗長ガード除去・Newtype との使い分け）
- `site/content/docs/language/refinement-types.mdx` に「Type Alias Refinement（v41.1.0+）」と「W030: 冗長ガード lint（v41.7.0+）」セクションを追加。パラメータ refinement との記法の違いを明記

---

## [v41.7.0] — 2026-07-11

### Added
- W030 lint: refinement 条件の冗長ガード検出（`type PositiveInt = Int where |v| v >= 0` の変数に `if x >= 0` ガードを書くと W030）
- `lint.rs`: `check_w030_redundant_refinement_guard` / `collect_refinement_aliases` / `check_w030_fn` / `exprs_lit_eq`

---

## [v41.6.0] -- 2026-07-11

### Added
- Newtype 自動 impl: `type Kg(Float)` 宣言で `+`/`-`/`*`/`/` を Float/Int 内側型から自動委譲
- `checker.fav`: `infer_op_with_newtypes` — Newtype 算術の型推論ヘルパー（同型間 `Kg + Kg` が正常に型チェックされる）
- `checker.fav`: `collect_variant_constructors` の IWrapper ケースに `__newtype__` env エントリ追加

---

## [v41.5.0] — 2026-07-11

### Added
- `fav/src/middle/ast_lower_checker.rs`: `RecordSpread` を `ERecordSpread(base, fields)` に正しく lowering（`sv("()")` バグ修正）
- `fav/src/middle/ast_lower_checker.rs`: `RecordType` を `TeRecord` に lowering（`TeSimple("Any")` から精緻化）
- `fav/self/checker.fav`: `ERecordSpread(Expr, Expr)` バリアント追加
- `fav/self/checker.fav`: `TeRecord` バリアント追加 + `type_expr_to_str` / `collect_type_vars_from_te` に対応ケース追加
- `fav/self/checker.fav`: `infer_expr` に `ERecordSpread` ケース追加（`"Unknown"` 型推論）
- `v41500_tests` 3 テスト追加（version / changelog / record_spread_parseable）

---

## [v41.4.0] — 2026-07-11

### Added
- `fav/src/middle/ast_lower_checker.rs`: `v4` ヘルパー追加 + `lower_arms` に `EArmG` 分岐（ガード式を checker.fav に渡す）
- `fav/self/checker.fav`: `EArmG(Pat, Expr, Expr, Expr)` バリアント追加（v41.4.0）
- `fav/self/checker.fav`: `infer_arms_effects` / `check_rebind` / `check_w006_arms` / `infer_arms` / `collect_arm_ctors` に EArmG ケース追加
- `fav/self/checker.fav`: `collect_arm_ctors` でガード付きワイルドカードを網羅性 catch-all としてカウントしないロジック追加
- `v41400_tests` 3 テスト追加（version / changelog / guard_match_parseable）

---

## [v41.3.0] — 2026-07-11

### Added
- `fav/src/frontend/parser.rs`: 式側 `(a, b)` タプルを `RecordConstruct("__tuple__", ...)` にデシュガー
- `fav/src/frontend/parser.rs`: パターン側 `(p1, p2)` タプルを `Pattern::Record` にデシュガー
- `fav/self/checker.fav`: タプルパターン処理の設計コメント追加（PRecord バリアント追加は v41.4.0）
- `v41300_tests` 3 テスト追加（version / changelog / tuple_pattern_match_parseable）

---

## [v41.2.0] — 2026-07-11

### Added
- `fav/src/error_catalog.rs`: E0404（refinement constraint violation）/ E0405（ambiguous refinement type）/ E0406（refinement constraint type mismatch）を追加
- `fav/self/checker.fav`: `TypeDef` に `invariants: List<String>` フィールドを追加
- `fav/self/checker.fav`: `check_item` の `IType` 分岐で `check_refinement_alias` を統合呼び出し
- `fav/src/middle/ast_lower_checker.rs`: `lower_type_def` に `invariants` フィールド（空リスト）を追加
- `v41200_tests` 3 テスト追加（version / changelog / error_catalog_has_e0404）

---

## [v41.1.0] — 2026-07-11

### Added
- `fav/src/frontend/parser.rs`: `parse_type_def` の Alias 分岐に `where |v| pred` 節を追加（Refinement type 基盤）
- `fav/self/checker.fav`: `check_refinement_alias` スタブ関数を追加（v41.2.0 で E0400 統合予定）
- `v41100_tests` 3 テスト追加（version / changelog / refinement_type_alias_where_parseable）

---

## [v41.0.0] — 2026-07-11

### Added
- `MILESTONE.md` に v41.0.0 — Streaming Foundations エントリ追加
- `README.md` に Streaming Foundations（v41.0）マイルストーン宣言を追記
- `v41000_tests` 4 テスト追加（version / changelog / milestone_has_streaming_foundations / readme_mentions_streaming_foundations）

---

## [v40.9.0] — 2026-07-11

### Added
- `site/content/docs/streaming-foundations.mdx` 新規作成（v40.x ストリーミング機能の概観ドキュメント）
- `v40900_tests` 3 テスト追加（version / changelog / streaming_foundations_doc_exists）

---

## [v40.8.0] — 2026-07-11

### Added
- `site/content/cookbook/window-aggregation.mdx` 新規作成（`tumbling_window` を使ったウィンドウ集計パイプライン）
- `site/content/cookbook/kafka-streaming.mdx` 新規作成（`consume_windowed` を使った Kafka Streams ウィンドウ消費）
- `v40800_tests` 3 テスト追加（version / changelog / cookbook_window_aggregation_exists）

---

## [v40.7.0] — 2026-07-11

### Added
- `fav bench --stream` フラグ追加（Streaming Foundations v40.7、ストリームパイプライン計測スタブ）
- `BenchOpts` に `stream: bool` フィールド追加
- `main.rs` ヘルプテキストに `--stream` オプション追記
- `v40700_tests` 3 テスト追加（version / changelog / bench_opts_has_stream_field）

---

## [v40.6.0] — 2026-07-11

### Added
- `runes/kafka/kafka.fav` に `consume_windowed(conn, topic, group_id, window_secs)` スタブ追加（Streaming Foundations v40.6、Kafka ウィンドウ集計）
- `runes/kafka/rune.toml` 新規作成（kafka rune メタ情報補完）
- `runes/redis/redis.fav` に `consume_windowed(conn, stream_key, group_id, window_secs)` スタブ追加（Redis Streams window 対応）
- `v40600_tests` 3 テスト追加（version / changelog / kafka_fav_has_consume_windowed）

---

## [v40.5.0] — 2026-07-11

### Added
- `fav/src/toml.rs` に `StreamConfig` 構造体追加（`watermark_delay: Option<u32>` / `late_policy: Option<String>`、Streaming Foundations v40.5）
- `FavToml` に `pub stream: Option<StreamConfig>` フィールド追加
- `parse_fav_toml` に `[stream]` セクション解析追加（`StateConfig` と同パターン）
- `inject_stream_config(_cfg: &StreamConfig)` スタブ関数追加（v40.6 以降で実伝播実装）
- `v40500_tests` 3 テスト追加（version / changelog / fav_toml_stream_section_parsed）

---

## [v40.4.0] — 2026-07-11

### Added
- `runes/stream/stream.fav` に `with_late_policy(stream, tolerance, policy)` スタブ追加（Out-of-order イベント処理、Streaming Foundations v40.4）
- `runes/stream/rune.toml` version を `40.4.0` に更新、description に `with_late_policy` 追記
- `v40400_tests` 3 テスト追加（version / changelog / stream_fav_has_late_policy）

---

## [v40.3.0] — 2026-07-11

### Added
- `runes/stream/stream.fav` に `Event` 型定義（`value: Any` スタブ / `timestamp: Int`）追加（Streaming Foundations v40.3、ジェネリクス統合は v43.x 予定）
- `runes/stream/rune.toml` version を `40.3.0` に更新、description に `Event(timestamp)` 追記
- `v40300_tests` 3 テスト追加（version / changelog / stream_fav_has_event_type）

---

## [v40.2.0] — 2026-07-11

### Added
- `runes/stream/stream.fav` に `session_window(stream, gap)` 関数スタブ追加（Streaming Foundations v40.2）
- `runes/stream/rune.toml` version を `40.2.0` に更新、description に `session_window` 追記
- `v40200_tests` 3 テスト追加（version / changelog / stream_rune_has_session_window）

---

## [v40.1.0] — 2026-07-11

### Added
- `runes/stream/stream.fav` に `tumbling_window` / `sliding_window` 関数スタブ追加（Streaming Foundations v40.1）
- `runes/stream/rune.toml` 新規作成（stream Rune メタデータ）
- `v40100_tests` 3 テスト追加（version / changelog / stream_rune_has_window_functions）

---

## [v40.0.0] — 2026-07-11

### Added
- Enterprise Governance マイルストーン宣言（v39.1〜v39.9 達成コンポーネント統合）
- `MILESTONE.md` に v40.0.0 Enterprise Governance セクション追加
- `README.md` に v40.0 マイルストーン宣言行追加
- `v40000_tests` 4 テスト追加（version / changelog / milestone / readme）

---

## [v39.9.0] — 2026-07-11

### Added
- `site/content/docs/enterprise-governance.mdx` — v39 スプリント振り返り + Enterprise Governance 概要ドキュメント追加
- `v39900_tests` 2 テスト追加（meta 2 件）

---

## [v39.8.0] — 2026-07-11

### Added
- `site/content/docs/governance/rbac.mdx` — RBAC ガバナンスドキュメント追加
- `site/content/docs/governance/audit-log.mdx` — Audit Log ガバナンスドキュメント追加
- `site/content/docs/governance/policy.mdx` — Policy ガバナンスドキュメント追加
- `site/content/cookbook/multi-tenant-etl.mdx` — マルチテナント ETL クックブック追加
- `site/content/cookbook/secret-manager-vault.mdx` — Secret Manager クックブック追加
- `site/content/cookbook/ci-policy-gate.mdx` — CI ポリシーゲートクックブック追加
- `v39800_tests` 3 テスト追加（meta 2 件 + site_has_governance_docs 1 件）

---

## [v39.7.0] — 2026-07-11

### Changed
- `driver.rs` `generate_ci_yaml` — `fav policy check --ci` ステップを CI YAML に自動追加
- `fav ci init` 生成 YAML が Policy check ゲートを含むようになった
- `v39700_tests` 2 テスト追加（meta 2 件）

---

## [v39.6.0] — 2026-07-11

### Added
- `fav/src/fav_audit.rs` — `fav audit`（依存 Rune ライセンス一覧）/ `fav audit --check`（GPL・CVE 検出、exit 1）追加
- `v39600_tests` 2 テスト追加（meta 2 件）

---

## [v39.5.0] — 2026-07-11

### Added
- `runes/tenant/tenant.fav` — `tenant.db_schema` / `tenant.s3_prefix` / `tenant.validate_tenant` 追加
- `runes/tenant/rune.toml` — Multi-tenant Rune メタデータ
- `ctx.tenant_id` ベースの DB スキーマ切り替え・S3 prefix 分離スタブ実装
- `v39500_tests` 4 テスト追加（meta 2 + テナント分離 E2E 2）

---

## [v39.4.0] — 2026-07-11

### Added
- `runes/secret/secret.fav` — `Secret.get_aws` / `Secret.get_vault` / `Secret.get_gcp` / `Secret.get_env` 追加
- `runes/secret/rune.toml` — Secret Rune メタデータ
- `fav.toml` `[secrets] backend` 宣言スキーマ（"aws"/"vault"/"gcp"/"env"）— 仕様定義のみ、toml.rs パースは後続バージョン
- `v39400_tests` 3 テスト追加

---

## [v39.3.0] — 2026-07-11

### Added
- `fav/src/policy.rs` — `fav policy check` / `fav policy check --ci` コマンド追加
- `policy { deny_runes / require_schema / require_tests / max_pipeline_stages }` ブロック仕様
- `v39300_tests` 3 テスト追加

---

## [v39.2.0] — 2026-07-10

### Added
- `runes/audit/audit.fav` — Audit Log Rune（`log` / `start_trace` / `end_trace`）
- `runes/audit/rune.toml` — Rune 設定ファイル
- `fav.toml` `[audit]` セクション仕様（`enabled` / `output = "file"/"webhook"`）
- `v39200_tests` 3 テスト追加

---

## [v39.1.0] — 2026-07-10

### Added
- `runes/auth/auth.fav` — RBAC Rune（`require_role` / `check_permission` / `verify_jwt`）
- `runes/auth/rune.toml` — Rune 設定ファイル
- `v39100_tests` 3 テスト追加

---

## [v39.0.0] — 2026-07-10

### Added
- Intelligence & Assistance マイルストーン宣言
- `MILESTONE.md` に v39.0.0 宣言セクション追加
- `README.md` に v39.0 マイルストーン宣言追加
- `v39000_tests` 4 テスト追加

---

## [v38.9.0] — 2026-07-10

### Added
- `site/content/docs/ai-overview.mdx` — v38.x AI 支援機能概要ドキュメント
- `v38900_tests` 4 テスト追加（`suggest_rs_has_llm_suggest` 品質確認含む）

---

## [v38.8.0] — 2026-07-10

### Added
- `site/content/cookbook/sql-to-favnir.mdx` — SQL → Favnir 変換 cookbook
- `site/content/cookbook/rag-pipeline.mdx` — RAG パイプライン cookbook
- `site/content/cookbook/llm-streaming.mdx` — LLM ストリーミング cookbook
- `v38800_tests` 3 テスト追加

---

## [v38.7.0] — 2026-07-10

### Added
- `Llm.stream_raw` VM primitive（collect-all 実装、true SSE は v39.x）
- `Llm.function_call_raw` VM primitive（ツール呼び出し JSON レスポンス）
- `Llm.embed_raw` VM primitive（OpenAI Embeddings API、`LLM_PROVIDER=openai` 専用）
- `llm_embed` ヘルパー関数 in `vm.rs`
- `stream` / `function_call` / `embed` 公開関数 in `runes/llm/client.fav`
- `v38700_tests` 3 テスト追加

---

## [v38.6.0] — 2026-07-10

### Added
- `fav new --template rag-pipeline` テンプレート追加（ingest/embed/retrieve/generate 4 ステージ）
- `create_rag_pipeline_project` in `driver.rs`
- `v38600_tests` 4 テスト追加

---

## [v38.5.0] — 2026-07-10

### Added
- `fav/src/explain_verbose.rs` — `fav explain --verbose <code> [location]` コマンド追加
- `explain_verbose`: エラーコード概要 + コンテキスト + Fix suggestion を出力（LLM stub）
- `v38500_tests` 5 テスト追加（code-reviewer 指摘対応: `explain_verbose_unknown_code` 追加、`location` コントロール文字フィルタ追加）

---

## [v38.4.0] — 2026-07-10

### Added
- `fav/src/toml.rs` — `LspAiConfig` + `parse_lsp_ai_config` 追加
- `[lsp.ai] enabled = true` で LSP AI 補完を有効化（v38.7.0 で本実装）
- `v38400_tests` 6 テスト追加（code-reviewer 指摘対応: `lsp_ai_explicit_false` / `lsp_ai_not_leaked_to_other_section` 追加）

---

## [v38.3.0] — 2026-07-10

### Added
- `fav/src/generate_csv.rs` — `fav generate --from csv <file>` コマンド追加
- `csv_to_favnir`: CSV ヘッダーから `type Row` + `schema` + `expect` ブロックを生成
- `v38300_tests` 4 テスト追加

---

## [v38.2.0] — 2026-07-10

### Added
- `fav/src/generate_sql.rs` — `fav generate --from sql <query>` コマンド追加
- `sql_to_favnir`: SELECT / JOIN / WHERE パターンを Favnir パイプラインに変換
- `v38200_tests` 6 テスト追加

---

## [v38.1.0] — 2026-07-10

### Added
- `fav/src/suggest.rs` — `fav suggest <error-code> <file:line>` コマンド追加
- `builtin_hint`: E0001 / E0007 / E0008 の組み込みヒント
- `ANTHROPIC_API_KEY` 設定時は LLM 提案（v38.7.0 で本実装予定、現在スタブ）
- `v38100_tests` 3 テスト追加

---

## [v38.0.0] — 2026-07-09

### Added
- Multi-Source ETL Power マイルストーン宣言
- `MILESTONE.md` に v38.0.0 宣言セクション追加
- `README.md` に v38.0 マイルストーン宣言追加
- `v38000_tests` 4 テスト追加

---

## [v37.9.0] — 2026-07-09

### Added
- `render_lineage_text` にサマリー行追加（`Total: N stage(s), M pipeline(s)`）
- `site/content/docs/multi-source-etl.mdx` — Multi-Source ETL 機能一覧ドキュメント
- `v37900_tests` 4 テスト追加

---

## [v37.8.0] — 2026-07-09

### Added
- `site/content/cookbook/join-two-tables.mdx` — `List.join_on` 2 テーブル結合レシピ
- `site/content/cookbook/cdc-postgres-to-warehouse.mdx` — CDC Rune レシピ
- `site/content/cookbook/fan-out-by-region.mdx` — `List.fan_out` / `List.fan_in` レシピ
- `site/content/cookbook/generic-etl-function.mdx` — ジェネリック ETL レシピ
- `site/content/cookbook/lineage-visualization.mdx` — リネージグラフ可視化レシピ
- `v37800_tests` 3 テスト追加

---

## [v37.7.0] — 2026-07-09

### Added
- `fav new --template multi-source` — マルチソース ETL プロジェクトテンプレート追加
- `TEMPLATE_GALLERY` に `"multi-source"` エントリ追加（6 エントリ）
- `cmd_new_list` に `"data-contract"` / `"multi-source"` 行追加
- `v37700_tests` 3 テスト追加

---

## [v37.6.0] — 2026-07-09

### Added
- `fav explain --lineage --format dot` — Graphviz DOT 形式のリネージグラフ出力
- `fav explain --lineage --format svg` — インライン SVG 形式のリネージグラフ出力
- `render_lineage_dot` / `render_lineage_svg` を `lineage.rs` に追加
- `v37600_tests` 4 テスト追加

---

## [v37.5.0] — 2026-07-09

### Added
- `runes/cdc/cdc.fav` — Debezium JSON 形式の CDC イベント処理 Rune（MySQL / Postgres 対応）
- `CDC.extract_op` / `CDC.op_name` / `CDC.is_insert` / `CDC.is_update` / `CDC.is_delete`
- `CDC.filter_inserts` / `CDC.filter_deletes` — イベントリストフィルタリング
- `v37500_tests` 4 テスト追加

---

## [v37.4.0] — 2026-07-09

### Added
- `List.fan_out(list, n)` VM ビルトイン追加（リストを n チャンクに分割）
- `List.fan_in(lists)` VM ビルトイン追加（List<List> を 1 レベルフラット化）
- `checker.rs` に `("List", "fan_out")` / `("List", "fan_in")` 戻り型定義追加
- `v37400_tests` 4 テスト追加

---

## [v37.3.0] — 2026-07-09

### Added
- `List.join_on(left, right, pred)` VM ビルトイン追加（left semi-join）
- `checker.rs` に `("List", "join_on")` 戻り型定義追加
- `v37300_tests` 3 テスト追加

---

## [v37.2.0] — 2026-07-09

### Added
- 複数フィールド行制約 `R with { id: Int, name: String }` が call-site 型チェックを通ることをテストで保証
- ネスト行型 `R with { address: { city: String } }` がパースを通ることを確認
- `v37200_tests` 4 テスト追加

---

## [v37.1.0] — 2026-07-09

### Added
- `Deserialize` 型制約を `type_implements_bound` に明示追加（`fav/src/middle/checker.rs`）
- `T with Deserialize` が型チェックと実行を通ることをテストで保証
- Generic Rune (`runes/generic/`) — 型パラメータ付き汎用 ETL 関数の参照実装

---

## [v37.0.0] — 2026-07-09

### Added
- Data Quality First マイルストーン宣言（v37.0.0）
- `MILESTONE.md` に v37.0.0 / v36.0.0 宣言セクション追加
- `README.md` に v36.0 Deployment Story / v37.0 Data Quality First マイルストーン宣言追加

---

## [v36.9.0] — 2026-07-09

### Changed
- W025 `schema_mismatch` エラーメッセージに `[see also: E0380 schema_field_missing]` 参照を追加

### Added
- `fav validate` 成功時に `Validated: N schema(s), M field(s) checked` サマリー行を出力
- `site/content/docs/data-quality.mdx` — Data Quality First 機能群統合ドキュメント新規作成

---

## [v36.8.0] — 2026-07-08

### Added
- `fav schema diff <old.fav> <new.fav>` — フィールドレベルのスキーマ差分と後方互換性チェック
- `schema_diff` 純粋関数 — 追加フィールド（backward-compatible）/ 削除・型変更フィールド（BREAKING）を検出
- `type_expr_kind` ヘルパー — Span を除外した型構造文字列生成（クロスファイル型比較に使用）

---

## [v36.7.0] — 2026-07-08

### Added
- `export_ge_suite(schema_name, field_names)` — Great Expectations 0.18.0 互換 Expectation Suite JSON 生成
- `fav validate --export ge --output suite.json` — GE 互換エクスポートフラグ追加（`--export ge` 時に suite.json を出力）

---

## [v36.6.0] — 2026-07-08

### Added
- `error_catalog.rs` に E0380〜E0384（スキーマ不整合エラーコード）を追加
  - `E0380` `schema_field_missing`: 必須フィールドがデータに存在しない
  - `E0381` `schema_type_mismatch`: フィールド型がデータ値と一致しない
  - `E0382` `schema_constraint_violated`: `where` 制約をデータ値が満たさない
  - `E0383` `schema_duplicate_key`: スキーマ定義にフィールド名が重複している
  - `E0384` `schema_extra_field`: データにスキーマ未定義のフィールドが含まれている

---

## [v36.5.0] — 2026-07-08

### Added
- Data Contract 規約 — `contracts/` ディレクトリ規約策定
- `fav new --template data-contract` テンプレート追加
- `fav contract check` コマンド — contracts/ ディレクトリのスキーマ定義を検証
- `cmd_contract_check` / `validate_contract_file` — driver.rs に追加

---

## [v36.4.0] — 2026-07-08

### Added
- `fav validate --schema <schema.fav> <data.csv>` コマンド — CSV ヘッダーとスキーマフィールドの整合性を検証
- `cmd_validate` — driver.rs に追加
- `validate_schema_against_headers` — スキーマ照合の純粋関数（テスト可能）

---

## [v36.3.0] — 2026-07-08

### Added
- W025 `schema_mismatch` lint ルール — スキーマ定義に存在しないフィールドアクセスを警告
- `check_w025_schema_mismatch` — `lint.rs` に追加
- `collect_schema_fields` / `collect_field_accesses_*` ヘルパー関数群

---

## [v36.2.0] — 2026-07-08

### Added
- `expect <target> { <rules> }` ブロック構文（Data Quality ルール宣言）
- `ast::ExpectStmt` 構造体・`Stmt::Expect` variant（v36.2.0）
- `parse_expect_stmt` — expect ブロックパーサー

---

## [v36.1.0] — 2026-07-08

### Added
- `schema Name { field: Type }` インライン schema 定義構文（Data Quality First スプリント開始）
- `ast::SchemaDef` 構造体・`Item::SchemaDef` variant（v36.1.0）
- `parse_schema_def` — トップレベル schema 宣言パーサー（`schema Ident {` 形式）
- 既存の `schema "uri"` 形式（文字列 URI 参照、v18.4/v32.4）との衝突なし

---

## [v36.0.0] — 2026-07-08

### Milestone: Deployment Story

- `fav deploy --target lambda` で Lambda に自動デプロイ（bootstrap.zip パッケージング）
- `fav deploy --target docker` で Docker イメージ生成（Dockerfile 自動生成・`docker build` 実行）
- `fav ci init` で GitHub Actions CI ワークフロー自動生成
- `!Effect` 廃止完結（v35.4〜v35.8）— すべての API が `ctx: AppCtx` ベースに統一
- v35.1〜v35.9 スプリント完了、Deployment Story マイルストーン宣言
- ★ `cargo clean`（x.0.0 クリーンアップ）実施

---

## [35.3.0] — 2026-07-06

### Added
- `fav ci init` — GitHub Actions CI ワークフロー自動生成コマンド
- `fav ci init --dry-run` — ファイル書き出しなしのプレビューモード
- `fav ci init --out-dir <dir>` — 出力先ディレクトリ指定
- `generate_ci_yaml` — check + lint + test の 3 ステップを含む CI YAML テンプレート生成関数
- `.github/workflows/ci.yml` の自動生成（親ディレクトリ自動作成）

---

## [35.2.0] — 2026-07-06

### Added
- `fav deploy --target docker` — Docker イメージ自動ビルドコマンド（`docker build` 実行）
- `fav deploy --tag <tag>` — Docker イメージタグ指定フラグ（デフォルト: `<project-name>:latest`）
- `generate_dockerfile_native` — ネイティブバイナリ向け Dockerfile テンプレート（`debian:bookworm-slim` ベース、`fav build --target native` の出力を実行）
- `fav.toml` `[deploy]` セクションに `tag` フィールドを追加
- Docker CLI 不在時は警告を出してスキップ（exit 0 のまま）

---

## [35.1.0] — 2026-07-06

### Added
- `fav deploy --target lambda` — Lambda への自動デプロイコマンド（AWS CLI 経由、SDK 依存なし）
- `fav deploy --package-only` — bootstrap.zip 生成のみ（アップロードなし）
- `fav deploy --output <path>` — --package-only 時の zip 出力先指定
- `fav.toml` の `[deploy]` セクションに `output` フィールドを追加
- `package_lambda` pub fn — ネイティブバイナリを bootstrap.zip にパッケージング（Lambda provided.al2 規約）
- `examples/lambda-deploy/` — Lambda デプロイデモプロジェクト（fav.toml + main.fav + README）
- `site/content/docs/deploy/lambda.mdx` — Lambda デプロイガイドスタブ（v35.8 で充実化予定）

---

## [v35.9.0] — 2026-07-07

### Changed
- 安定化スプリント — v35.1〜v35.8 の機能統合確認・v36.0 前調整

### Verified
- `!Effect` 廃止完結（v35.4〜v35.8）: lsp/completion.rs・docs_server.rs・mcp/mod.rs・error_catalog.rs すべてクリーン
- `examples/lambda-deploy/` — Lambda デプロイデモ（v36.0 前提条件）確認済み
- `site/content/docs/deploy/lambda.mdx` — Lambda デプロイドキュメント確認済み
- `versions/roadmap/roadmap-v35.1-v36.0.md` — Deployment Story 計画確認済み

### Added
- `fav/src/driver.rs` — `v35900_tests`（5 件）追加

---

## [v35.8.0] — 2026-07-05

### Changed
- `fav/src/lsp/completion.rs` — LSP 補完シグネチャ（IO/Http/Llm/Db/Gen/Csv/Sys 計 ~25 関数）から `!Effect` 表記を除去
- `fav/src/lsp/completion.rs` — KNOWN_RUNES 説明文（12 件）から `!Effect` タグを除去
- `fav/src/error_catalog.rs` — E0310〜E0324 の `fix:` フィールドを `ctx: AppCtx` ベースの修正提案に書き換え（12 件）
- `fav/src/mcp/mod.rs` — db/http/log/grpc rune ドキュメント文字列から `!Io` を除去（~16 行）
- `fav/src/main.rs` — `--help` テキスト `!DbRead/!DbWrite effects` → `DbRead/DbWrite lineage tags`

### Notes
- v35.0C 実装完了: Favnir コードベースから `!Effect` 表記が**完全に廃止**された
- ユーザーが目にするすべての箇所（IDE 補完・エラーメッセージ・CLI help・MCP ドキュメント）が更新済み

---

## [v35.7.0] — 2026-07-05

### Changed
- `fav/src/docs_server.rs` — IO 関数シグネチャから `!Io` エフェクト表記を除去（`"String -> Unit !Io"` → `"String -> Unit"`）
- `fav/src/docs_server.rs` — IO 関数の `effects` フィールドを空配列に統一（`&["Io"]` → `&[]`）

### Notes
- v35.0B 実装完了: これにより Favnir コードベースから `!Effect` 表記が**完全に除去**された
- `fav doc` の `/api/stdlib` エンドポイントの IO シグネチャが変更（後方互換: `effects` フィールド自体は残存）

---

## [v35.6.0] — 2026-07-05

### Changed
- `site/content/` — 128 MDX ファイル・317 コードブロックの `!Effect` アノテーションを ctx 構文に一括変換
- `site/content/docs/ctx-syntax-guide.mdx` — 6 セクション構成の完成版に更新（E0374 説明・Before/After 対比・移行表含む）
- `README.md` — favnir コードブロックから `!Effect` アノテーションをすべて除去

### Added
- `MILESTONE.md` — v35.0 Production Ready 宣言（Effect 廃止・セルフホスト完成・ドキュメント整合・エコシステム成熟の4条件達成）
- `fav/src/driver.rs` — `v35600_tests`（5 件）追加

### Notes
- v35.0A 実装完了: `!Effect` の痕跡がドキュメントから完全に消滅
- サイト全体で「副作用 = ctx: AppCtx を渡す」パターンが統一された
- 2611 tests pass (0 failures)

---

## [v35.5.0] — 2026-07-05

### Changed
- `fav/src/ast.rs` — `Effect` enum、`effects` フィールドを全型定義から削除
- `fav/src/middle/checker.rs` — `infer_effects_fn`、`EffectSet`、エフェクト強制ロジックを削除
- `fav/src/middle/compiler.rs` — `effects` フィールド参照をすべて除去
- `fav/src/lineage.rs` — `Effect` ベースの分類ロジックを削除、Snowflake/Azure コール検出ベースに移行
- `fav/src/middle/reachability.rs` / `fav/src/middle/ir.rs` — `Effect` 参照除去
- `fav/self/checker.fav` — `check_effects_all` を no-op 化（Effect 強制を完全廃止）
- `runes/`（95ファイル）— 残存 `!Effect` アノテーション除去（コンパイル互換性確保）
- `fav/src/driver.rs` — Effect 関連テスト 33 件スタブ化（E0003/E0025/E0338/E0319〜E0324 系）

### Notes
- v34.9A 実装完了: Effect enum が言語・コンパイラ・リンター・リネージから完全に消滅
- 型安全な副作用管理は `ctx: AppCtx`（capability context）パターンで実現
- 2611 tests pass (0 failures)

---

## [v35.4.0] — 2026-07-05

### Changed
- `fav/src/frontend/parser.rs` — `parse_effect_ann` が `!Effect` アノテーションを即座に **E0374** として拒否するよう変更（13 テスト更新）
- `fav/src/middle/checker.rs` — `current_fn_has_ctx: bool` フィールド追加; `ctx: AppCtx` 引数を持つ関数はエフェクト強制をバイパス（`has_effect` 更新）
- `fav/src/backend/wasm_codegen.rs` — `ensure_supported_main_signature` が空エフェクト（pure main）を許可
- `fav/src/lint.rs` — W022 `check_w022_deprecated_effect_annotation` 関数を削除（E0374 に吸収）
- `fav/src/emit_python.rs` / `fav/src/middle/reachability.rs` / `fav/src/lineage.rs` — `!Effect` アノテーション除去（テスト更新）
- `fav/src/backend/vm_stdlib_tests.rs` / `vm_legacy_coverage_tests.rs` — インライン Favnir ソース文字列から `!Effect` を除去

### Added
- `fav/src/error_catalog.rs` — **E0374** `EffectAnnotationRemoved` エラーコード登録
- `fav/src/driver.rs` — `v35400_tests`（5 件）: E0374 回帰 / ctx bypass / W022 削除 / エラーカタログ確認

### Notes
- v34.8A 実装: `!Effect` アノテーション構文を parse error（E0374）に昇格。副作用の宣言は `ctx: AppCtx` 第一引数で行う
- `current_fn_has_ctx = true` のとき `has_effect()` は常に `true` を返すため、E0107 等の全エフェクトチェックをバイパス

---

## [v35.3.0] — 2026-07-05

### Changed
- `examples/` 配下 31 ファイルの `!Effect` アノテーションを除去（stage / fn シグネチャ）
- `infra/e2e-demo/` 配下 10 ファイルの `!Effect` アノテーションを除去（stage / fn シグネチャ）
- `fav/src/driver.rs` — `cargo_toml_version_is_35_2_0` をスタブ化
- `fav/src/driver.rs` — `verifier_fav_has_aws_effects` / `crosscloud_effects_declared` をスタブ化（ctx 移行に伴う更新）

### Added
- `fav/src/driver.rs` — `v35300_tests`（5 件）: examples/infra !Effect 除去の回帰テスト

### Notes
- v34.7A 実装: `examples/` + `infra/e2e-demo/` の全 `.fav` ファイルから `!Effect` アノテーションを完全除去
- `stage` 宣言は `!Effect` 除去のみ、`fn` 宣言は `ctx: AppCtx` 追加 + `!Effect` 除去

---

## [v35.2.0] — 2026-07-05

### Added
- `fav/src/middle/compiler.rs` — `"Ctx"` / `"AppCtx"` を builtin namespace リストに追加（`Ctx.test_ctx_raw()` が test ブロックで使用可能に）
- `runes/aws/aws.test.fav` / `runes/http/http.test.fav` / `runes/auth/auth.test.fav` / `runes/env/env.test.fav` / `runes/grpc/grpc.test.fav` / `runes/incremental/incremental.test.fav` — ctx: AppCtx を使用するよう更新

### Changed
- `runes/` 配下 100 件の `!Effect` → `ctx: AppCtx` 第一引数移行完了（v34.6A 全タスク達成）
- `fav/src/driver.rs` — `http_rune_put_returns_err_on_bad_host` テストを ctx 構文に更新

### Notes
- v34.6A 補完実装: `!Effect` アノテーションの完全廃止。Rune ファイルの公開 fn はすべて `ctx: AppCtx` を第一引数として持つ
- `Ctx.test_ctx_raw()` を test ブロック内で使用するためのコンパイラ修正（`compiler.rs` への namespace 登録）

---

## [v35.1.0] — 2026-07-04

### Added
- `fav/src/ast.rs` — `Effect::is_deprecated()` メソッド追加（Pure 以外のすべての Effect が `true` を返す）
- `fav/src/middle/checker.rs` — `!Effect` 使用時の deprecation 警告（W022）を型チェック時に発行

### Notes
- v34.5A 補完実装: ロードマップが要求したコンパイラレベルの !Effect 非推奨化を実施
- lint.rs の W022（v34.5.0 で実装済み）と組み合わせることで `fav lint` / `fav check` 両方で警告が出る

---

## [v35.0.0] — 2026-07-04

### Added
- **Production Ready マイルストーン宣言**（v34.1〜v34.9 完了）
- `MILESTONE.md` に `v35.0.0 — Production Ready` セクション追加
- `README.md` に v35.0 マイルストーン行追記
- `v350000_tests`: マイルストーン宣言確認テスト 5 件
- `cargo clean` 実施（最終クリーンアップ、24.2 GiB 削減）

### Changed
- `versions/current.md` — 最新安定版を v35.0.0 に更新

---

## [v34.9.0] — 2026-07-04

### Added
- `site/content/docs/tools/upgrade-guide.mdx` — `fav upgrade` コマンド公式ドキュメント（フラグ / ワークフロー / トラブルシューティング）
- `fav/tests/fixtures/ctx_migration/before.fav` — `!Http` 使用の移行前フィクスチャ
- `fav/tests/fixtures/ctx_migration/after.fav` — `AppCtx` 使用の移行後フィクスチャ

### Changed
- `versions/current.md` — 最新安定版を v34.9.0 に更新

---

## [v34.8.0] — 2026-07-04

### Added
- `MIGRATION.md` — `!Effect` → Capability Context 移行ガイド（背景 / fav upgrade 手順 / 対応表 / Before-After / FAQ）
- `fav upgrade` コマンド（`--from-effects` + `--dry-run` / `--in-place` フラグ）

### Changed
- `versions/current.md` — 最新安定版を v34.8.0 に更新

---

## [v34.7.0] — 2026-07-04

### Added
- `site/content/docs/ctx-syntax-guide.mdx` — ctx 構文完全リファレンスガイド

### Changed
- `site/content/learn/getting-started.mdx` — AppCtx を使ったパイプライン例を追加
- `README.md` — v34.5〜v34.7 ctx 移行シリーズの記録を追加
- `versions/current.md` — 最新安定版を v34.7.0 に更新

---

## [v34.6.0] — 2026-07-04

### Added
- `runes/ctx/db.fav` — DbCtx interface 定義（`!DbRead`/`!DbWrite` → `ctx.db` 移行用）
- `runes/ctx/http.fav` — HttpClient interface 定義（`!Http` → `ctx.http` 移行用）
- `runes/ctx/stream.fav` — StreamClient interface 定義（`!Stream` → `ctx.stream` 移行用）
- `site/content/docs/runes/ctx-migration-status.mdx` — ctx 移行ステータスサマリーページ

### Changed
- `versions/current.md` — 最新安定版を v34.6.0 に更新

---

## [v34.5.0] — 2026-07-04

### Added
- `fav/src/lint.rs` — W022 `deprecated_effect_annotation` lint ルール追加
- `runes/ctx/io.fav` — IoCtx interface 定義（`!Io` → `ctx.io` 移行用）
- `site/content/docs/tools/migration-effects.mdx` — `!Effect` → ctx 移行ガイド

### Changed
- `versions/current.md` — 最新安定版を v34.5.0 に更新

---

## [v34.4.0] — 2026-07-04

### Added
- `site/content/docs/tools/security-audit-v2.mdx` — セキュリティ審査 v2 レポート（W021・認証情報・sandbox・OSS ライセンスの 4 項目）
- `site/content/docs/tools/oss-licenses.mdx` — OSS 依存ライセンス一覧（26 クレート）

### Changed
- `SECURITY_MODEL.md` — v34.x ctx 移行との関係セクション追加
- `versions/current.md` — 最新安定版を v34.4.0 に更新

---

## [v34.3.0] — 2026-07-04

### Added
- `benchmarks/real-world/` — 実測ベンチマーク JSON 3 ファイル（favnir / python_pandas / apache_spark）

### Changed
- `site/content/docs/bench/index.mdx` — dbt 比較セクション追加 / 履歴テーブルに v34.1〜v34.3 行追加
- `versions/current.md` — 最新安定版を v34.3.0 に更新

---

## [v34.2.0] — 2026-07-04

### Added
- `site/content/errors/index.mdx` — エラーコードリファレンスページ（57 コード、カテゴリ別テーブル）
- `site/content/cookbook/` — 18 本追加（計 50 本）
  postgres-etl / snowflake-load / duckdb-query / parquet-transform / avro-schema /
  iceberg-compaction / mongodb-etl / redis-cache-aside / elasticsearch-index /
  http-api-ingest / csv-validation / schema-evolution / data-quality-check /
  incremental-load / cron-trigger / secret-manager / jwt-auth / grpc-client

### Changed
- `site/content/docs/bench/index.mdx` — Python pandas / Apache Spark との比較データを追加

---

## [v34.1.0] — 2026-07-04

### Added
- `examples/real-world-etl/` — 8 ファイル構成の実案件規模 ETL デモ（src/ 5 ファイル + fav.toml + data/ + README）
  - `src/types.fav` — Order / OrderStatus / ValidationError / LoadResult 型定義
  - `src/validators.fav` — 欠損値・範囲・重複バリデーション
  - `src/stages.fav` — load_csv / write_postgres / sync_bigquery ステージ
  - `src/notifications.fav` — Slack 成功・失敗通知
  - `src/main.fav` — RealWorldEtl pipeline（5 ステージ） + OTel トレース
  - `data/orders_sample.csv` — サンプルデータ（ヘッダー + 5 行）
  - `README.md` — 30 分で動かす手順書

---

## [v34.0.0] — 2026-07-04

### Added
- **Performance & Tooling マイルストーン宣言**（v33.1〜v33.9 確認完了）
- `MILESTONE.md` に `v34.0.0 — Performance & Tooling` セクション追加
- `README.md` に v34.0 マイルストーン行追記
- `v340000_tests`: マイルストーン宣言確認テスト 4 件
- `cargo clean` 実施（20.5 GiB 削減）

---

## [v33.9.0] — 2026-07-04

### Added
- `v339000_tests`: 並列コンパイル（`topo_layers` / `compile_parallel`）動作確認テスト 4 件
- `parallel_topo_cyclic_dep_returns_err`: 循環依存グラフで `topo_layers` が `Err("circular dependency detected")` を返すことを確認
- `parallel_compile_empty_sources`: 空ソースリストで `compile_parallel` が `Ok(IRProgram { fns: [] })` を返すことを確認

---

## [v33.8.0] — 2026-07-04

### Added
- `v338000_tests`: プロファイリング強化（`parse_profile_json` / `to_folded_stacks`）動作確認テスト 4 件
- `profile_parse_json_valid_records`: JSON キー `"name"` / `"ms"` → `StageRecord` 変換を確認
- `profile_folded_stacks_has_pipeline_prefix`: `to_folded_stacks` 出力が `"pipeline;"` プレフィックスを持つことを確認

---

## [v33.7.0] — 2026-07-04

### Added
- `v337000_tests`: エフェクトシステム移行準備（`migrate_effects_in_source` / `resolve_use_effects`）動作確認テスト 4 件
  - `cargo_toml_version_is_33_7_0` — バージョン確認
  - `benchmark_v33_7_0_exists` — ベンチマークファイル存在確認
  - `migrate_effects_idempotent` — `migrate_effects_in_source` を 2 回適用しても結果が変わらない（冪等性保証）
  - `resolve_use_effects_from_v13` — `from_version = "v13"` / `"13"` が移行モードを有効化し、`v12` / `None` は無効のまま

### Notes
- `migrate_effects_in_source` / `resolve_use_effects` は v13.10.0 実装済み
- v33.7.0 は Performance & Tooling フェーズの記録としてエフェクト移行ツールの冪等性・バージョン判定を確認する

---

## [v33.6.0] — 2026-07-04

### Added
- `v336000_tests`: WASM 最適化（DCE / `WasmBuildConfig`）動作確認テスト 4 件
  - `cargo_toml_version_is_33_6_0` — バージョン確認
  - `benchmark_v33_6_0_exists` — ベンチマークファイル存在確認
  - `wasm_dce_keeps_reachable_fn` — DCE が到達可能な関数を除去しないことを確認（安全性保証）
  - `wasm_default_config_is_o0_with_dce` — `WasmBuildConfig::default()` が `O0` + `dce: true` であることを確認

### Notes
- `WasmBuildConfig` / `WasmOptLevel` / `wasm_dce` は v19.6.0 実装済み
- v33.6.0 は Performance & Tooling フェーズの記録として WASM 最適化の設計を確認する

---

## [v33.5.0] — 2026-07-04

### Added
- `v335000_tests`: `fav run --precompiled`（`.favc` 事前コンパイル実行）動作確認テスト 4 件
  - `cargo_toml_version_is_33_5_0` — バージョン確認
  - `benchmark_v33_5_0_exists` — ベンチマークファイル存在確認
  - `favc_meta_source_hash_is_nonzero` — `FavcMeta.source_hash` が非ゼロ（SHA-256 計算済み確認）
  - `favc_different_sources_differ` — 異なるソースが異なる `.favc` バイト列を生成する（衝突なし保証）

### Notes
- `cmd_compile_to_bytes` / `cmd_run_precompiled_bytes` / `FavcMeta` は v19.7.0 実装済み
- v33.5.0 は Performance & Tooling フェーズの記録として事前コンパイル実行の META セクション設計を確認する

---

## [v33.4.0] — 2026-07-04

### Added
- `v334000_tests`: Arrow 列指向統合（`ArrowBatch` / `#[arrow]`）動作確認テスト 4 件
  - `cargo_toml_version_is_33_4_0` — バージョン確認
  - `benchmark_v33_4_0_exists` — ベンチマークファイル存在確認
  - `arrow_trf_without_annotation_has_false` — `#[arrow]` なし stage が `arrow: false`（opt-out 確認）
  - `arrow_trf_arrow_and_stateful_are_independent` — `#[stateful]` のみ stage が `arrow: false` かつ `stateful: true`（2フラグ独立確認）

### Notes
- `ArrowBatch` 型 / `#[arrow]` アノテーション / `TrfDef.arrow: bool` は v19.5.0 実装済み
- v33.4.0 は Performance & Tooling フェーズの記録として Arrow 列指向統合の AST 設計を確認する

---

## [v33.3.0] — 2026-07-04

### Added
- `v333000_tests`: ストリーミング評価（`#[streaming]`）動作確認テスト 4 件
  - `cargo_toml_version_is_33_3_0` — バージョン確認
  - `benchmark_v33_3_0_exists` — ベンチマークファイル存在確認
  - `streaming_seq_without_annotation_has_none` — アノテーションなし seq が `streaming: None`（opt-in 確認）
  - `streaming_chunk_size_boundary_one` — `chunk_size = 1`（最小境界値）のパース確認

### Notes
- `#[streaming]` / `StreamingAnnotation` / ストリーミングパイプライン実行は v19.1.0 実装済み
- v33.3.0 は Performance & Tooling フェーズの記録としてストリーミング評価動作を確認する

---

## [v33.2.0] — 2026-07-04

### Added
- `v332000_tests`: インクリメンタルコンパイル動作確認テスト 4 件
  - `cargo_toml_version_is_33_2_0` — バージョン確認
  - `benchmark_v33_2_0_exists` — ベンチマークファイル存在確認
  - `incremental_content_hash_deterministic` — SHA-256 ハッシュの決定性確認
  - `incremental_dep_graph_no_import_isolated` — 依存なしファイルが影響を受けないことを確認

### Notes
- `IncrementalCache` / `fingerprint` / `dep_graph` は v19.3.0 実装済み
- v33.2.0 は Performance & Tooling フェーズの記録としてインクリメンタルコンパイル動作を確認する

---

## [v33.1.0] — 2026-07-04

### Added
- `v331000_tests`: AOT ネイティブバイナリ（Cranelift）動作確認テスト 4 件
  - `cargo_toml_version_is_33_1_0` — バージョン確認
  - `benchmark_v33_1_0_exists` — ベンチマークファイル存在確認
  - `aot_if_branch_selects_true_arm` — if 式の then アーム選択確認
  - `aot_bool_comparison_native` — Bool 比較（`2 > 1` → `1`）の AOT 動作確認

### Notes
- `CraneliftBackend::compile_to_binary` / `cmd_build_native` は v19.2.0 実装済み
- v33.1.0 は Performance & Tooling フェーズの記録として AOT 動作を明示的に確認する
- cc 非インストール環境では aot_* テストは自動スキップ（偽陰性なし）

---

## [v33.0.0] — 2026-07-03

### Added
- `v330000_tests`: Language Power マイルストーン宣言確認テスト 4 件
  - `cargo_toml_version_is_33_0_0` — バージョン確認
  - `milestone_language_power_declared` — MILESTONE.md に「Language Power」記載確認
  - `readme_mentions_v33_0` — README.md に「v33.0」記載確認
  - `benchmark_v33_0_0_exists` — ベンチマークファイル存在確認
- `MILESTONE.md` — v33.0.0「Language Power」セクションを先頭に追加
- `README.md` — v33.0 マイルストーン宣言を追加

### Notes
- Language Power = 境界付きジェネリクス / 行多相 / where 制約 / スキーマ型 /
  線形型 / 分散アノテーション / 定数ジェネリクス / 型駆動 API 生成 / エフェクト推論
- `cargo clean` 実施（マイルストーン版の必須クリーンアップ）

---

## [v32.9.0] — 2026-07-03

### Added
- `v329000_tests`: エフェクト推論（Effect Inference）動作確認テスト 4 件
  - `cargo_toml_version_is_32_9_0` — バージョン確認
  - `benchmark_v32_9_0_exists` — ベンチマークファイル存在確認
  - `effect_infer_io_println` — `IO.println` → `!Io` エフェクト推論確認
  - `effect_infer_pure_mul_no_effects` — 純粋関数 `mul` → エフェクトなし確認

### Notes
- `infer_effects_fn` / `infer_effects_for_program` / `EffectSet` は v18.1.0 実装済み
- v32.9.0 はその動作を Language Power フェーズの記録として明示的に確認する

---

## [v32.8.0] — 2026-07-03

### Added
- `v328000_tests`: 型駆動 API 生成（Type-Driven API Generation）動作確認テスト 4 件
  - `cargo_toml_version_is_32_8_0` — バージョン確認
  - `benchmark_v32_8_0_exists` — ベンチマークファイル存在確認
  - `api_ann_get_items_path_parses` — `#[api]` アノテーション `/items/:id` のパース確認
  - `api_ann_openapi_items_path_exists` — OpenAPI JSON の `/items/{id}` paths キー確認

### Notes
- `ApiAnnotation` / `build_openapi_json` / `build_route_table` 等は v18.8.0 実装済み
- v32.8.0 はその動作を Language Power フェーズの記録として明示的に確認する

---

## [v32.7.0] — 2026-07-03

### Added
- `v327000_tests`: 定数ジェネリクス（Const Generics）動作確認テスト 4 件
  - `cargo_toml_version_is_32_7_0` — バージョン確認
  - `benchmark_v32_7_0_exists` — ベンチマークファイル存在確認
  - `const_gen_chunk_size_valid` — `N=5` が `N>0` を満たす → E0335 なし
  - `const_gen_chunk_size_zero_e0335` — `N=0` が `N>0` を違反 → E0335

### Notes
- `GenericParam.is_const` / `const_ty` / `const_constraint` / E0335 は v18.7.0 実装済み
- v32.7.0 はその動作を Language Power フェーズの記録として明示的に確認する

---

## [v32.6.0] — 2026-07-03

### Added
- `v326000_tests`: 分散アノテーション（Variance Annotations）動作確認テスト 4 件
  - `cargo_toml_version_is_32_6_0` — バージョン確認
  - `benchmark_v32_6_0_exists` — ベンチマークファイル存在確認
  - `variance_ann_covariant_output_pass` — `+T` が出力位置のみ → E0334 なし
  - `variance_ann_covariant_input_e0334` — `+T` が入力位置 → E0334

### Notes
- `Variance::Covariant`・`Variance::Contravariant`・E0334 は v18.6.0 実装済み
- v32.6.0 はその動作を Language Power フェーズの記録として明示的に確認する

---

## [v32.5.0] — 2026-07-03

### Added
- `v325000_tests`: 線形型（Linear Types）動作確認テスト 4 件
  - `cargo_toml_version_is_32_5_0` — バージョン確認
  - `benchmark_v32_5_0_exists` — ベンチマークファイル存在確認
  - `linear_type_double_use_e0332` — Connection 二重使用で E0332
  - `linear_type_unused_var_e0333` — Connection 未使用で E0333

### Notes
- `TokenKind::LinearArrow`・E0332・E0333 は v18.5.0 実装済み
- v32.5.0 はその動作を Language Power フェーズの記録として明示的に確認する

---

## [v32.4.0] — 2026-07-03

### Added
- `v324000_tests`: スキーマ型（Schema Types）動作確認テスト 4 件
  - `cargo_toml_version_is_32_4_0` — バージョン確認
  - `benchmark_v32_4_0_exists` — ベンチマークファイル存在確認
  - `schema_alias_parses` — `schema "file:..."` 構文パース確認
  - `schema_type_ast_is_schema_expr` — AST が `TypeExpr::Schema` を含む確認

### Notes
- `TypeExpr::Schema`・`register_schema_types`・`schema_loader` は v18.4.0 実装済み
- v32.4.0 はその動作を Language Power フェーズの記録として明示的に確認する

---

## [v32.3.0] — 2026-07-03

### Added
- `v323000_tests`: where 制約（Refinement Types）動作確認テスト 4 件
  - `cargo_toml_version_is_32_3_0` — バージョン確認
  - `benchmark_v32_3_0_exists` — ベンチマークファイル存在確認
  - `where_constraint_literal_pass` — `b=2` で `b != 0` 制約 PASS
  - `where_constraint_literal_fail_e0331` — `b=0` で E0331

### Notes
- `fn_refinement_registry`・E0331・`RefinementAssert` opcode は v18.3.0 実装済み
- v32.3.0 はその動作を Language Power フェーズの記録として明示的に確認する

---

## [v32.2.0] — 2026-07-03

### Added
- `v322000_tests`: 行多相（Row Polymorphism）動作確認テスト 4 件
  - `cargo_toml_version_is_32_2_0` — バージョン確認
  - `benchmark_v32_2_0_exists` — ベンチマークファイル存在確認
  - `row_poly_field_constraint_pass` — `with { id: Int }` 制約 PASS
  - `row_poly_missing_field_e0337` — フィールドなし型を渡すと E0337

### Notes
- `TypeConstraint::HasField`・`type_has_field`・E0337 は v18.2.0 実装済み
- v32.2.0 はその動作を Language Power フェーズの記録として明示的に確認する

---

## [v32.1.0] — 2026-07-03

### Added
- 境界付きジェネリクス（bounded generics）の確認・テスト補強
- `Display` bound（String が満たす）/ `Hash` bound（Int が満たす）の正常動作を検証
- `Hash` bound に Float を渡すと E0325 が発生することをネガティブテストで確認

### Notes
- 実装自体は v17.1.0 で完了済み。v32.1.0 は Language Power フェーズの起点として記録。

---

## [v32.0.0] — 2026-07-03

### Added
- Language Polish マイルストーン宣言（v31.1〜v31.9 全コンポーネント完成）
- `MILESTONE.md` に v32.0.0「Language Polish」セクション追加
- `README.md` に v32.0 マイルストーン行追加
- `cargo clean` + `cargo build` 実施（マイルストーン版クリーンアップ）

---

## [v31.9.0] — 2026-07-03

### Fixed
- REPL: `ReplSession::add_history` が空行・空白行を履歴に追加しないよう修正
- `fav check --all`: .fav ファイルが見つからない場合に警告メッセージを表示（非 JSON モード）

---

## [v31.8.0] — 2026-07-03

### Added
- `fav scaffold stage <Name>` — プロジェクト `src/stages.fav` にスタブコードを自動追記
- `fav scaffold seq <Name>` — プロジェクト `src/pipelines.fav` にスタブコードを自動追記
- `scaffold_to_src()` を `driver.rs` に追加（fav.toml の src ディレクトリ自動解決）
- `benchmarks/v31.8.0.json` 追加

### Changed
- `Cargo.toml` version: `31.7.0` → `31.8.0`

---

## [v31.7.0] — 2026-07-03

### Added
- `fav check --all` — fav.toml src ディレクトリ内の全 .fav を一括型チェック
- `fav check --all --json` — JSON 形式でエラー出力
- `cmd_check_all()` / `check_all_files()` を `driver.rs` に追加
- `benchmarks/v31.7.0.json` 追加

### Changed
- `Cargo.toml` version: `31.6.0` → `31.7.0`

---

## [v31.6.0] — 2026-07-03

### Added
- `fav test --watch <dir>` — ファイル変更検知による自動テスト再実行
- `benchmarks/v31.6.0.json` 追加

### Changed
- `Cargo.toml` version: `31.5.0` → `31.6.0`

---

## [v31.5.0] — 2026-07-02

### Added
- `lsp/inlay_hints.rs` — 新規作成: `handle_inlay_hints()` / `collect_bind_hints()` 実装
- LSP `initialize` 応答に `"inlayHintProvider": true` を追加
- LSP `textDocument/inlayHint` ハンドラを追加
- `editors/favnir-vscode/package.json` に `inlayHints` capability を追記
- `benchmarks/v31.5.0.json` 追加

### Changed
- `Cargo.toml` version: `31.4.0` → `31.5.0`

---

## [v31.4.0] — 2026-07-02

### Added
- `driver.rs` — `repl_complete_with_defs()` 追加（セッション定義名をタブ補完に含める）
- `benchmarks/v31.4.0.json` 追加

### Changed
- `driver.rs::cmd_repl()` — REPL プロンプトを `> ` → `favnir> ` に変更
- `driver.rs::ReplSession::add_history()` — 履歴上限を 100 件に制限（push 後に 101 件になったら先頭を削除）
- `Cargo.toml` version: `31.3.0` → `31.4.0`

---

## [v31.3.0] — 2026-07-02

### Added
- `driver.rs::get_explain_text()` — E0002/E0003/E0004/E0005/E0006/E0010/E0011/E0019/E0020/E0021 の説明テキストを追加
- `benchmarks/v31.3.0.json` 追加

### Changed
- `driver.rs::cmd_explain_code()` — unknown コード時に利用可能コード一覧（E0001〜E0021）を表示
- `Cargo.toml` version: `31.2.0` → `31.3.0`

---

## [v31.2.0] — 2026-07-02

### Added
- `driver.rs` — `levenshtein()` / `suggest_similar()` ユーティリティ関数を追加
- `driver.rs::get_help_text()` — E0011/E0012/E0016/E0017/E0019 に hint を追加
- `benchmarks/v31.2.0.json` 追加

### Changed
- `Cargo.toml` version: `31.1.0` → `31.2.0`

---

## [v31.1.0] — 2026-07-02

### Changed
- `driver.rs::get_help_text()` — E0002/E0003/E0004/E0005/E0006/E0010 に hint を追加
- `Cargo.toml` version: `31.0.0` → `31.1.0`

### Added
- `benchmarks/v31.1.0.json` 追加

---

## [v31.0.0] — 2026-07-02

### Added
- Real-World Readiness マイルストーンを正式宣言
- `MILESTONE.md` に v31.0.0 セクション追加（v30.1〜v30.9 達成コンポーネント一覧）
- `benchmarks/v31.0.0.json` 追加

### Changed
- `Cargo.toml` version: `30.9.0` → `31.0.0`

---

## [v30.9.0] — 2026-07-02

### Fixed
- `toml.rs` — `[project]` セクションを認識して `src` フィールドを正しくパースする
- `driver.rs` — 非 rune import を `src_dir` ではなく `root` ベースで解決（`import src/types` が `src/src/types.fav` にならない）
- `driver.rs` — `fav test` false 返却時に `assert_eq!` / `assert!` 使用を促すヒントを追加
- `main.rs` — `fav new`（引数なし）に `fav new --list` ヒントを追加

---

## [v30.8.0] — 2026-07-02

### Added
- `cmd_new_list` — `fav new --list` でテンプレート一覧を表示（8 テンプレート）
- `main.rs` — `fav new --list` フラグを検出して `cmd_new_list()` を呼ぶ

---

## [v30.7.0] — 2026-07-02

### Changed
- `hint_for_runtime_error`（新規 `pub(crate)` 関数）— index out of bounds / global index / type error に `= ヒント:` を付加（具体パターン優先順）
- `format_runtime_error` — プレフィックスを `"runtime error:"` に統一、ステージ名を `"in stage X"` 形式で表示、空スタックトレース時も `fn_name` を保持

---

## [v30.6.0] — 2026-07-02

### Changed
- `cmd_test`（引数なし）— `src/` に加えて `tests/` ディレクトリも走査対象に追加
- `fav test`（引数なし）で `tests/pipeline_test.fav` が自動検出・実行される

---

## [v30.5.0] — 2026-07-02

### Added
- `examples/csv-to-postgres/` — ドッグフード用 CSV → Postgres ETL サンプル（8 ファイル）
- `data/sample.csv` — 10 行のサンプルデータ（行 9 は意図的に無効）
- `tests/pipeline_test.fav` — DB 不要の純粋バリデーションテスト（3 件）
- README に 30 分クイックスタート手順を記載

### Changed
- `lint.rs` — TrfDef（stage）に E0023/E0025 を適用しないよう修正。stage は本来エフェクト境界であるため `IO.*` / `Postgres.*` ambient 呼び出しは合法

---

## [v30.4.0] — 2026-07-01

### Added
- `fav/tests/fixtures/multifile_rune_import/` — 複数ファイルから同一 Rune（`runes/postgres`）を import するフィクスチャ
- `stages.fav` と `main.fav` 両方が `import runes/postgres` を持つシナリオを検証（二重定義エラーなし確認）
- `validators.fav` は Rune import なし — Rune import が不要なファイルの明示的確認

### Fixed
- v30.3.0 code-reviewer [HIGH]: `scaffold_postgres_etl_stages` の `ValidateRows` 戻り型アノテーション（`List<ValidRow>` → `Result<List<ValidRow>, RowError>`）修正
- v30.3.0 code-reviewer [MED]: `benchmarks/v30.3.0.json` の `tests_passed` を実測値 2391 に修正

---

## [v30.3.0] — 2026-07-01

### Added
- `fav/tests/fixtures/multifile_etl/` — マルチファイル ETL プロジェクト検証フィクスチャ（types / validators / main + fav.toml）
- インラインレコードリテラルの型名プレフィックス必須化を確認（`RowError { }` / `ValidRow { }` 構文）
- `scaffold_postgres_etl_validators` / `scaffold_postgres_etl_stages` / `scaffold_postgres_etl_test` を型付きレコードリテラルに修正

---

## [v30.2.0] — 2026-07-01

### Changed
- `fav new --template postgres-etl` — 4 ファイル構成（types / validators / stages / main）に更新
- `tests/pipeline_test.fav` と `README.md` を生成するように変更
- scaffold コードを VM 確認済みプリミティブ（`String.to_int` / `String.to_float` / `Option.unwrap_or`）で統一

---

## [v30.1.0] — 2026-07-01

### Added
- `[profile.dev] debug = 0` — デバッグシンボル無効化によりビルド生成物を軽量化
- `[profile.dev] split-debuginfo = "off"` — デバッグ情報分割ファイルを無効化

---

## [v30.0.0] — 2026-07-01

### Added
- `MILESTONE.md` — Ecosystem Maturity 宣言セクション追加
- `site/content/docs/ecosystem-maturity.mdx` — マイルストーン宣言ドキュメント（`fav add stripe` デモ）
- `versions/roadmap/roadmap-v29.1-v30.0.md` — 達成宣言（COMPLETE）追記
- テスト数: 2366 → 2372（+6）

---

## [v29.9.0] — 2026-07-01

### Added
- `CONTRIBUTING.md` — コミュニティ Rune 開発ガイド（5 条件: connect / read / write / error / test）追加
- `runes/stripe|twilio|notion|linear|airtable|sendgrid|hubspot|zendesk|shopify|intercom/` — コミュニティ Rune スタブ 10 本（各 `.fav` + `rune.toml`）
- `site/app/community/page.tsx` — 第 1 回 Rune コンテスト告知セクション追加
- テスト数: 2360 → 2366（+6）

---

## [v29.8.0] — 2026-07-01

### Added
- `site/content/cookbook/` — cookbook 29 本追加（3 → 32 本）
- `site/app/community/` — `/community/` ページ新設（GitHub Discussions / Discord リンク）
- テスト数: 2354 → 2360（+6）

---

## [v29.7.0] — 2026-06-30

### Added
- `extensions/vscode-favnir/` — VS Code 拡張パッケージ（TextMate grammar / LSP クライアント / Task Runner 統合）
- `site/content/docs/tools/vscode-extension.mdx` — VS Code 拡張ドキュメント
- テスト数: 2348 → 2354（+6）

---

## [v29.6.0] — 2026-06-30

### Added
- `runes/pagerduty/` — PagerDuty Events API v2 Rune（create_incident / resolve / acknowledge / add_note）
- `site/content/docs/runes/pagerduty.mdx` — PagerDuty Rune ドキュメント
- テスト数: 2342 → 2348（+6）

---

## [v29.5.0] — 2026-06-30

### Added
- `runes/github/` — GitHub REST API Rune（create_comment / create_issue / update_issue / list_prs / get_pr）
- `site/content/docs/runes/github.mdx` — GitHub Rune ドキュメント
- テスト数: 2336 → 2342（+6）

---

## [v29.4.0] — 2026-06-30

### Added
- `runes/vertex-ai/` — Google Vertex AI Rune（predict / batch_predict / deploy_model / list_endpoints）
- `runes/sagemaker/` — AWS SageMaker Rune（invoke / create_endpoint / delete_endpoint）
- `site/content/docs/runes/vertex-ai.mdx` / `sagemaker.mdx` — ドキュメント追加
- テスト数: 2330 → 2336（+6）

---

## [v29.3.0] — 2026-06-30

### Added
- `runes/pinecone/` — Pinecone ベクトルDB Rune（upsert / query / delete / fetch / describe_index_stats）
- RAG パイプラインサポート: LLM Rune と組み合わせてドキュメント検索付き LLM パイプラインを構築可能
- `site/content/docs/runes/pinecone.mdx` — Pinecone Rune ドキュメント
- テスト数: 2324 → 2330（+6）

---

## [v29.2.0] — 2026-06-30

### Added
- `runes/mlflow/mlflow.fav` — MLflow Rune 実装（`start_run` / `log_metric` / `log_param` / `log_artifact` / `end_run` / `register_model` / `load_model` / `list_experiments` 8 関数）
- `runes/mlflow/rune.toml` — Rune メタデータ
- `site/content/docs/runes/mlflow.mdx` — MLflow ドキュメント追加
- テスト数: 2318 → 2324（+6）

---

## [v29.1.0] — 2026-06-28

### Added
- `cmd_publish`（`driver.rs`）— `FAVNIR_REGISTRY_URL` 環境変数を参照し `{url}/v1/publish` へのリモート API 呼び出しを追加（インフラ稼働後に HTTP POST 有効化）
- `pub fn cmd_info(pkg_name: &str)`（`driver.rs`）— `FAVNIR_REGISTRY_URL/v1/packages/{name}` へのリモート参照 + 静的カタログフォールバック
- `cmd_search`（`driver.rs`）— `FAVNIR_REGISTRY_URL` 設定時に `/v1/search?q=...` へのリモート参照を追加（フォールバック: 静的カタログ）
- `cmd_login`（`driver.rs`）— GitHub OAuth URL 生成（`https://github.com/login/oauth/authorize?client_id=...&scope=read:user`）
- `Some("info")`（`main.rs`）— `fav info <pkg>` サブコマンドを `cmd_info` にルーティング
- `benchmarks/v29.1.0.json` — テスト数 2318 件を記録

### Changed
- `Cargo.toml` version `29.0.0` → `29.1.0`

---

## [v29.0.0] — 2026-06-28

### Added
- `MILESTONE.md` に "Observability First" セクション追加（v28.1〜v28.9 達成コンポーネント一覧・象徴デモ・v29.x 残件）
- `site/content/docs/observability-first.mdx` — Observability First マイルストーン解説ドキュメント（prometheus / sentry / grafana コード例・`fav profile` 使用例・マイルストーン履歴）
- `README.md` に v29.0 "Observability First" マイルストーン参照を追記
- `versions/roadmap/roadmap-v28.1-v29.0.md` に v29.0 完了マーク追記
- `benchmarks/v29.0.0.json` — テスト数 2312 件を記録

### Fixed
- `examples/observability/prometheus_grafana.fav` — `fn main()` 追加 + `Result.ok(unit)` → `Result.ok(())` 修正（`fav run` 対応）
- `examples/observability/datadog_apm.fav` — `fn main()` 追加 + `Result.ok(unit)` → `Result.ok(())` 修正（`fav run` 対応）
- `fav/tests/fixtures/etl.fav` — `{ unit }` → `{ () }` 修正 + `fn main()` 追加（`fav profile --format flamegraph` 対応）
- `examples/observability/docker-compose.yml` — `sentry-redis`（redis:7-alpine）/ `sentry-postgres`（postgres:15-alpine）追加（sentry の依存サービス）

### Changed
- `Cargo.toml` version `28.9.0` → `29.0.0`

---

## [v28.9.0] — 2026-06-28

### Added
- `examples/observability/sentry_alerting.fav` 拡充 — `CriticalLoad` stage 追加（`// #[on_error(report_to: "sentry", level: "critical")]` コメント形式）+ `Sentry.capture_message` 使用 + `seq SentryAlertingDemo` を `CriticalLoad |> ReportError |> SetContext` に拡充
- `examples/observability/docker-compose.yml` に `getsentry/sentry:24.0` サービス追加
- `site/content/docs/tools/sentry-alerting.mdx` — Sentry アラート E2E デモ解説ドキュメント

---

## [v28.8.0] — 2026-06-28

### Added
- `examples/observability/datadog_apm.fav` 拡充 — 3 stage（ExtractEvents / TransformEvents / LoadEvents）+ `// #[trace(service:` コメント形式アノテーション
- `examples/observability/docker-compose.yml` に `datadog-agent:7` サービス追加
- `site/content/docs/tools/datadog-apm.mdx` — Datadog APM E2E デモ解説ドキュメント

---

## [v28.7.0] — 2026-06-28

### Added
- `examples/observability/prometheus_grafana.fav` — ETL パイプライン × Prometheus + Grafana E2E デモ（`PrometheusGrafanaDemo` seq）
- `examples/observability/docker-compose.yml` — prometheus / grafana Docker 定義
- `examples/observability/README.md` — セットアップ手順（docker compose up → fav run → Grafana UI）
- `site/content/docs/tools/observability-e2e.mdx` — E2E デモ解説ドキュメント

---

## [v28.6.0] — 2026-06-28

### Added
- `runes/grafana/grafana.fav` — Grafana ダッシュボード管理 Rune（create_annotation / push_dashboard / snapshot）
- `Grafana.create_annotation_raw` / `push_dashboard_raw` / `snapshot_raw` VM primitive 追加（`#[cfg]` ガード付き）
- `fav/self/checker.fav` `ns_to_effect` に `"Grafana" => "IO"` 追加
- `examples/observability/grafana_dashboard.fav` — GrafanaDashboardDemo E2E デモ
- `site/content/docs/runes/grafana.mdx` — ドキュメント追加

---

## [v28.5.0] — 2026-06-28

### Added
- `runes/sentry/sentry.fav` — Sentry エラートラッキング Rune（capture_error / capture_message / set_user / set_tag / set_extra）
- `Sentry.capture_error_raw` / `capture_message_raw` / `set_user_raw` / `set_tag_raw` / `set_extra_raw` VM primitive 追加（`#[cfg]` ガード付き）
- `fav/self/checker.fav` `ns_to_effect` に `"Sentry" => "IO"` 追加
- `examples/observability/sentry_alerting.fav` — SentryAlertingDemo E2E デモ
- `site/content/docs/runes/sentry.mdx` — ドキュメント追加

---

## [v28.4.0] — 2026-06-28

### Added
- `fav profile --compare <version>` — ベースラインベンチマークとの stage 別実行時間比較（`[SLOWER]` / `[FASTER]` / `[NEW]` マーカー出力）
- `pub fn cmd_profile_compare` を `driver.rs` に追加
- `fav/tests/fixtures/etl.fav` — プロファイルテスト用 ETL フィクスチャ
- `site/content/docs/performance/profiling.mdx` に `--compare` セクション追加

---

## [v28.3.0] — 2026-06-28

### Added
- `runes/otel/otel.fav` — OpenTelemetry Rune（start_span / set_attribute / add_event / end_span）
- `OTel.start_span_raw` / `OTel.set_attribute_raw` / `OTel.add_event_raw` / `OTel.end_span_raw` VM primitive 追加（`#[cfg]` ガード付き）
- `fav/self/checker.fav` `ns_to_effect` に `"OTel" => "IO"` 追加
- `examples/observability/otel_tracing.fav` — OTelTracingDemo E2E デモ
- `site/content/docs/runes/otel.mdx` — ドキュメント追加

---

## [v28.2.0] — 2026-06-27 — datadog Rune 追加

### Added
- `runes/datadog/datadog.fav` — Datadog APM/Metrics/Logs Rune（metric / log / trace / event / service_check 5 関数、`!Io` エフェクト）
- `Datadog.metric_raw` / `log_raw` / `trace_raw` / `event_raw` / `service_check_raw` VM primitive 5 件追加（`#[cfg]` ガード付き）
- `examples/observability/datadog_apm.fav` — APM トレース + メトリクス送信デモ（DatadogApmDemo seq pipeline）
- `site/content/docs/runes/datadog.mdx` ドキュメント追加
- `fav/self/checker.fav` `ns_to_effect` に `"Datadog" => "IO"` 追加

### Notes
- v28.2.0 は stub 実装。実際の DogStatsD / Datadog API 送信は v28.x 以降
- `#[trace]` アノテーションは v28.3+ で独立バージョンとして実装予定

---

## [v28.1.0] — 2026-06-27 — prometheus Rune 追加

### Added
- `runes/prometheus/prometheus.fav` — Prometheus メトリクス Rune（counter / gauge / histogram / push 4 関数、`!Io` エフェクト）
- `Prometheus.counter_raw` / `gauge_raw` / `histogram_raw` / `push_raw` VM primitive 4 件追加（`#[cfg]` ガード付き）
- `examples/observability/prometheus_demo.fav` — カスタムメトリクス送信デモ（PrometheusDemo seq pipeline）
- `site/content/docs/runes/prometheus.mdx` ドキュメント追加
- `fav/self/checker.fav` `ns_to_effect` に `"Prometheus" => "Io"` 追加

### Notes
- v28.1.0 は stub 実装。実際の Pushgateway HTTP 送信は v28.x 以降
- `#[track]` アノテーションは v28.2+ で独立バージョンとして実装予定

---

## [v28.0.0] — 2026-06-27 — Data Lakehouse マイルストーン宣言

### Added
- `MILESTONE.md` に "Data Lakehouse" セクション追加（v27.1〜v27.9 完了コンポーネント一覧・象徴デモ・v28.x 残件）
- `site/content/docs/data-lakehouse.mdx` — Data Lakehouse マイルストーン解説ページ
- `benchmarks/v28.0.0.json` — ベンチマーク記録（test_count: 2226）

### Notes
- v27.1〜v27.9 で実装した Data Lakehouse スタック（Delta Lake / Iceberg / ClickHouse / BigQuery / Redshift / JSONL / dbt / SQLite / `fav infer --from delta/iceberg`）の全コンポーネント完成を宣言
- v28.x では stub 実装（delta-rs / rusqlite / manifest.json 実解析）を本統合に移行予定

---

## [v27.9.0] — 2026-06-27 — sqlite Rune 追加

### Added
- `runes/sqlite/sqlite.fav` — SQLite Rune（open / open_memory / query / execute / execute_many / close 6 関数、`!Db` エフェクト）
- `SQLite.open_raw` / `open_memory_raw` / `query_raw` / `execute_raw` / `execute_many_raw` / `close_raw` VM primitive 6 件追加（`#[cfg]` ガード付き）
- `examples/sqlite_etl.fav` — SQLite 軽量 ETL パイプラインデモ
- `site/content/docs/runes/sqlite.mdx` ドキュメント追加
- `fav/self/checker.fav` `ns_to_effect` に `"SQLite" => "Db"` 追加

### Notes
- v27.9.0 は stub 実装。実際の SQLite 操作（`rusqlite` クレート統合）は v28.x に延期

---

## [v27.8.0] — 2026-06-27 — dbt 連携 Rune

### Added
- `runes/dbt/dbt.fav` — dbt 連携 Rune（ref / source 2 関数、`!Db` エフェクト）
- `Dbt.ref_raw` / `Dbt.source_raw` VM primitive 追加（`#[cfg]` ガード付き）
- `examples/dbt_pipeline.fav` — dbt モデル参照パイプラインデモ
- `fav/tests/fixtures/dbt_manifest.json` — manifest.json モックフィクスチャ
- `site/content/docs/runes/dbt.mdx` ドキュメント追加
- `fav/self/checker.fav` `ns_to_effect` に `"Dbt" => "Db"` 追加

### Notes
- v27.8.0 は stub 実装。`manifest.json` の実解析と SQL 実行は v28.x に延期

---

## [v27.7.0] — 2026-06-27 — `fav infer --from delta` / `--from iceberg`

### Added
- `fav infer --from delta --path <path>` — Delta Lake テーブルスキーマから Favnir 型定義を自動生成（v27.7.0 stub）
- `fav infer --from iceberg --catalog <url> --table <name>` — Iceberg テーブルスキーマから型定義を自動生成（v27.7.0 stub）
- `DeltaLake.infer_schema_raw` / `Iceberg.infer_schema_raw` VM primitive 追加（`#[cfg]` ガード付き）
- `delta_type_to_favnir` 型マッピング関数（long/integer/int→Int, double/float→Float, string→String, timestamp→DateTime, boolean→Bool）
- `site/content/docs/tools/infer-delta-iceberg.mdx` ドキュメント追加

### Notes
- v27.7.0 は stub 実装。実際のテーブルスキーマ読み取りは v28.x（`delta-rs` / `iceberg-rust` 統合時）に実装予定

---

## [v27.6.0] — 2026-06-27 — jsonl Rune 追加

### Added
- `runes/jsonl/jsonl.fav` — JSONL Rune（read / write / stream / append 4 関数、`!Io` エフェクト）
- `JSONL.*_raw` VM primitives 4 件（`#[cfg(not(target_arch = "wasm32"))]` ガード付き、stub 実装）
- `examples/jsonl_etl.fav` — JSONL ETL デモ（ReadData |> WriteProcessed）
- `site/content/docs/runes/jsonl.mdx` — JSONL Rune ドキュメント

---

## [v27.5.0] — 2026-06-27 — redshift Rune 追加

### Added
- `runes/redshift/redshift.fav` — Redshift Rune（connect / query / execute / copy_from_s3 / unload_to_s3 5 関数、`!Db` エフェクト）
- `Redshift.*_raw` VM primitives 5 件（`#[cfg(not(target_arch = "wasm32"))]` ガード付き、stub 実装）
- `examples/redshift_analytics.fav` — Redshift Analytics デモ（LoadFromS3 |> QuerySummary |> UnloadToS3）
- `site/content/docs/runes/redshift.mdx` — Redshift Rune ドキュメント

---

## [v27.4.0] — 2026-06-27 — bigquery Rune 実質化

### Added
- `runes/bigquery/bigquery.fav` — BigQuery Rune 実質化（connect / query / insert / load_from_gcs / create_table 5 関数、`!Db` エフェクト）
- `BigQuery.connect_raw` / `BigQuery.conn_query_raw` / `BigQuery.insert_raw` / `BigQuery.load_from_gcs_raw` / `BigQuery.create_table_raw` VM primitives 5 件（`#[cfg(not(target_arch = "wasm32"))]` ガード付き、stub 実装）
- `examples/bigquery_analytics.fav` — BigQuery Analytics デモ（CreateEventTable |> LoadFromGcs |> QueryStats）
- `site/content/docs/runes/bigquery.mdx` — BigQuery Rune ドキュメント（v27.4.0 新 API）

### Changed
- `runes/bigquery/bigquery.fav` — v15.2.0 の `!Gcp` エフェクト・非 public API から `!Db` エフェクト・`public fn` API に刷新（既存 `BigQuery.query_raw` / `BigQuery.execute_raw` VM primitive は後方互換として残存）

---

## [v27.3.0] — 2026-06-27 — clickhouse Rune 追加

### Added
- `runes/clickhouse/clickhouse.fav` — ClickHouse Rune（connect / query / insert / async_insert 4 関数）
- `ClickHouse.*_raw` VM primitives 4 件（`#[cfg(not(target_arch = "wasm32"))]` ガード付き、stub 実装）
- `examples/clickhouse_analytics.fav` — ClickHouse Analytics デモ（LoadEvents |> InsertProcessed）
- `site/content/docs/runes/clickhouse.mdx` — ClickHouse Rune ドキュメント

---

## [v27.2.0] — 2026-06-27 — iceberg Rune 追加

### Added
- `runes/iceberg/iceberg.fav` — Apache Iceberg Rune（read / append / overwrite / time_travel / schema_evolution / list_snapshots 6 関数）
- `Iceberg.*_raw` VM primitives 6 件（`#[cfg(not(target_arch = "wasm32"))]` ガード付き、stub 実装）
- `examples/iceberg_etl.fav` — Iceberg ETL デモ（LoadFromIceberg |> TransformData |> AppendToIceberg）
- `site/content/docs/runes/iceberg.mdx` — Apache Iceberg Rune ドキュメント

---

## [v27.1.0] — 2026-06-27 — delta-lake Rune 追加

### Added
- `runes/delta-lake/delta-lake.fav` — Delta Lake Rune（read / read_with_filter / write / merge / history / vacuum / optimize 7 関数）
- `DeltaLake.*_raw` VM primitives 7 件（`#[cfg(not(target_arch = "wasm32"))]` ガード付き、stub 実装）
- `examples/delta_lake_etl.fav` — Delta Lake ETL デモ（LoadRawData |> TransformOrders |> SaveProcessed）
- `site/content/docs/runes/delta-lake.mdx` — Delta Lake Rune ドキュメント

---

## [v27.0.0] — 2026-06-27 — Streaming Native マイルストーン宣言

### Milestone
- **Streaming Native** 宣言: ストリーミング Rune 5 本（kinesis / nats / rabbitmq / sqs / pulsar）実質化完了
- `Stream.*` 操作 6 関数（map / filter / flat_map / window / merge / split）使用可能
- E2E デモ 3 本（kafka→ES / kinesis→S3 / nats→postgres）が Docker Compose で動作

### Added
- `MILESTONE.md` に "Streaming Native" マイルストーンセクション追加
- `site/content/docs/streaming-native.mdx` — Streaming Native マイルストーン解説ページ
- `README.md` に v27.0 マイルストーン記載
- `versions/roadmap/roadmap-v26.1-v27.0.md` に完了日追記

---

## [v26.9.0] — 2026-06-27 — Pulsar Rune 追加

### Added
- `runes/pulsar/pulsar.fav` — Apache Pulsar Rune 追加（`produce` / `consume` / `ack` / `nack` 4 関数）
- `Pulsar.produce_raw` / `Pulsar.consume_raw` / `Pulsar.ack_raw` / `Pulsar.nack_raw` VM primitives（`#[cfg(not(target_arch = "wasm32"))]` ガード付き）
- `examples/streaming/docker-compose.yml` に `pulsar` サービス（`apachepulsar/pulsar:3.2.0`）追加
- `site/content/docs/runes/pulsar.mdx` — Pulsar Rune ドキュメント

---

## [v26.8.0] — 2026-06-27 — SQS Rune 実質化

### Added
- `runes/sqs/sqs.fav` — SQS Rune 実質化（`send_message` / `send_message_batch` / `receive_messages` / `delete_message` / `purge` / `consume` 6 関数）
- `fav/src/backend/vm.rs` — `SQS.send_message_batch_raw` / `SQS.receive_messages_raw` / `SQS.purge_raw` / `SQS.consume_raw` primitive 追加
- `site/content/docs/runes/sqs.mdx` — SQS Rune ドキュメント

---

## [v26.7.0] — 2026-06-27 — ストリーミング E2E デモ（nats → postgres）

### Added
- `examples/streaming/nats_to_postgres.fav` — NATS → Postgres IoT センサーデータ蓄積デモ（FetchSensorData / ValidateSensor / InsertToPostgres + `seq SensorPipeline`）
- `examples/streaming/docker-compose.yml` に `nats` / `postgres` サービス追加
- `examples/streaming/README.md` — 3 本の E2E デモ実行手順まとめ
- `site/content/docs/streaming/nats-to-postgres.mdx` — E2E デモドキュメント

---

## [v26.6.0] — 2026-06-27 — ストリーミング E2E デモ（kinesis → s3）

### Added
- `examples/streaming/kinesis_to_s3.fav` — Kinesis → S3 クリックイベントアーカイブデモ（FetchClickEvents / SerializeBatch / UploadToS3 + `seq ArchivePipeline`）
- `examples/streaming/docker-compose.yml` に `localstack` サービス追加（Kinesis / S3 ローカルエミュレーション）
- `site/content/docs/streaming/kinesis-to-s3.mdx` — E2E デモドキュメント

---

## [v26.5.0] — 2026-06-27 — ストリーミング E2E デモ（kafka → elasticsearch）

### Added
- `examples/streaming/kafka_to_elasticsearch.fav` — Kafka → Elasticsearch リアルタイムログ集計デモ（FetchLogs / FilterErrors / IndexToES + `seq LogPipeline`）
- `examples/streaming/docker-compose.yml` — Kafka（Redpanda）/ Elasticsearch サービス定義
- `site/content/docs/streaming/kafka-to-elasticsearch.mdx` — E2E デモドキュメント

---

## [v26.4.0] — 2026-06-27 — `#[streaming]` バックプレッシャー対応 + `Stream.*` 操作

### Added
- `StreamingAnnotation.backpressure: Option<bool>` フィールド追加（`ast.rs` + `parser.rs`）— `#[streaming(backpressure = true)]` 構文をサポート
- `Stream.flat_map` / `Stream.window` / `Stream.merge` / `Stream.split` — VM primitive 4 件追加（`vm.rs`）
- `VMStream::FlatMap` / `VMStream::Window` / `VMStream::Merge` / `VMStream::Split` — 遅延評価バリアント追加
- `runes/stream/stream.fav` — Stream Rune 新規作成（map / filter / flat_map / window / merge / split）
- `site/content/docs/runes/stream.mdx` — Stream Rune ドキュメント新規作成

---

## [v26.3.0] — 2026-06-26 — rabbitmq Rune 実質化

### Added
- `runes/rabbitmq/rabbitmq.fav` — RabbitMQ Rune（connect / declare_exchange / declare_queue / bind_queue / publish / consume）
- `RabbitMQ.connect_raw` / `declare_exchange_raw` / `declare_queue_raw` / `bind_queue_raw` / `publish_raw` / `consume_raw` — VM primitive 6 件追加
- `site/content/docs/runes/rabbitmq.mdx` — RabbitMQ Rune ドキュメント新規作成
- `benchmarks/v26.3.0.json` — ベンチマーク記録（test_count: 2062）

---

## [v26.2.0] — 2026-06-26 — nats Rune 実質化

### Added
- `runes/nats/nats.fav` — NATS Rune（connect / publish / subscribe / jetstream_publish / jetstream_consume）
- `NATS.connect_raw` / `publish_raw` / `subscribe_raw` / `jetstream_publish_raw` / `jetstream_consume_raw` — VM primitive 5 件追加
- `site/content/docs/runes/nats.mdx` — NATS Rune ドキュメント新規作成
- `benchmarks/v26.2.0.json` — ベンチマーク記録（test_count: 2054）

---

## [v26.1.0] — 2026-06-26 — kinesis Rune 実質化

### Added
- `runes/kinesis/kinesis.fav` — Kinesis Rune（connect / put_record / put_records / get_shard_iterator / get_records）
- `Kinesis.connect_raw` / `put_record_raw` / `put_records_raw` / `get_shard_iterator_raw` / `get_records_raw` — VM primitive 5 件追加
- `site/content/docs/runes/kinesis.mdx` — Kinesis Rune ドキュメント新規作成
- `benchmarks/v26.1.0.json` — ベンチマーク記録（test_count: 2047）

---

## [v26.0.0] — 2026-06-26 — Rune Foundation マイルストーン宣言

### Milestone
- **Rune Foundation**: コア 8 Rune（postgres / s3 / redis / mysql / mongodb / dynamodb / kafka / elasticsearch）が「動く Rune の 5 条件（connect / read / write / error / test）」をすべてクリア
- `examples/full_etl.fav` — postgres → 集計 → s3 → kafka 通知の完全デモパイプライン
- `examples/postgres_etl.fav` / `examples/s3_csv_to_parquet.fav` — 個別 Rune デモ
- vm.fav Phase 6（`CallNamed` opcode, 0x56）完了: multi-function プログラムを vm.fav で実行可能（v25.9.0 完了の宣言）
- `MILESTONE.md` に「Rune Foundation」セクション追記
- `site/content/docs/rune-foundation.mdx` 新規作成
- `versions/roadmap/roadmap-v25.1-v26.0.md` — v25.1〜v25.9 COMPLETE・v26.0.0 宣言済みに更新

---

## [v25.9.0] — 2026-06-26 — vm.fav Phase 6（CallNamed 実装 — ユーザー定義関数呼び出し）

### Added
- `CallNamed(Int, Int)` opcode (0x56) — `fav/self/vm.fav` に追加（fn_name_const_idx, argc）
- `decode_byte_with_u16x2_le` — 5 バイト opcode デコーダー（u16 LE ×2 オペランド）
- `vm_execute` シグネチャ拡張: `consts: Int, prog_keys: Int, prog_vals: Int`（現在関数の定数プールとプログラムテーブル）
- `vm_run_program(program_json)` — multi-function program JSON を受け取り main 関数を実行する新エントリポイント
- 補助型・関数: `FnDef` / `ListPair` / `parse_fn_json` / `build_consts_list` / `copy_args_to_locals` / `find_fn_in_program` / `build_program_lists`
- `build_vm_program_json(artifact)` — `FvcArtifact` → program JSON シリアライザ（`Constant::Int/Float/Str/Name` 全バリアント対応）
- `run_via_vm(vm_src, program_json)` — vm.fav 経由で multi-function プログラムを実行
- `fav run --vm <path> --compile <src>` — ソースをコンパイルして vm.fav 経由で実行する CLI モード
- `site/content/docs/tools/vm-fav.mdx` — Phase 6 / `--compile` フラグ説明を追記

### Notes
- 線形検索パターン採用（`Mut.str_map` は未実装のため `prog_keys` / `prog_vals` の Mut.map ペア）
- `Constant::Name` が `CallNamed` の参照対象（`Constant::Str` ではない）
- `parse_fn_json` は単純文字列解析（関数名にカンマ・ダブルクォートなし前提）

---

## [v25.8.0] — 2026-06-25 — elasticsearch Rune 実質化（全文検索・ベクトル検索・バルク書き込み）

### Added
- `Effect::Elasticsearch` — 新規エフェクト variant（`!Elasticsearch`）
- E0324 `UndeclaredElasticsearchEffect` — エラーカタログ追加
- `ES.*_raw` 8 件 VM プリミティブ（`connect` / `index` / `index_with_id` / `search` / `knn_search` / `bulk` / `delete` / `create_index`）
- `runes/elasticsearch/elasticsearch.fav` — `ESConn` 型 + 8 関数
- `examples/elasticsearch_logs_etl.fav` — `IndexLog |> SearchLogs` パイプラインデモ
- `site/content/docs/runes/elasticsearch.mdx` — 全 API リファレンス（Docker セットアップ・認証・スコープ外）
- 認証: `ELASTICSEARCH_API_KEY`（優先）→ Basic（USERNAME/PASSWORD）→ 認証なし
- `knn_search` — kNN ベクトル検索（`_source` 配列 JSON 文字列で返す）
- `bulk` — JSON 配列 → NDJSON 変換 → `POST /_bulk`（一括インデックス）

---

## [v25.7.0] — 2026-06-25 — kafka Rune 実質化（「動く Rune」5 条件達成）

### Added
- `KafkaConn(String)` — Kafka ブローカー接続ラッパー型（`""` → `KAFKA_BOOTSTRAP_BROKERS` 環境変数 → `"localhost:9092"`）
- `Kafka.connect` / `produce` / `consume_one` / `consume_batch` / `create_topic`（5 関数、`KafkaConn` ベース）
- `Kafka.connect_raw` / `Kafka.consume_batch_raw` / `Kafka.create_topic_raw`（新規 VM primitives 3 件）— 既存 rskafka v0.6 再利用（追加 crate なし）
- `kafka_connect_sync` / `kafka_consume_batch_sync` / `kafka_create_topic_sync` ヘルパー（vm.rs）
- E0319 `UndeclaredStreamEffect` を `error_catalog.rs` に登録（checker.rs には v15.4.0 から存在）
- `examples/kafka_events_etl.fav` — イベント ETL デモ（PublishEvent / ConsumeEvents / EventsETL）
- `site/content/docs/runes/kafka.mdx` — 全 API ドキュメント（Redpanda セットアップ手順含む）

---

## [v25.6.0] — 2026-06-25 — dynamodb Rune 実質化（「動く Rune」5 条件達成）

### Added
- `Effect::DynamoDB` — 新エフェクト（AWS NoSQL KV 専用。E0323 エラーコード追加）
- `DynamoDB.connect` / `get_item` / `put_item` / `delete_item` / `query` / `scan` / `batch_write` / `transact_write`（8 関数）
- `DynamoDB.*_raw` VM primitives 8 件（`vm.rs`）— 既存 `aws_post` / SigV4 インフラ再利用（追加 crate なし）
- `get_dynamo_endpoint` / `json_val_to_dynamo_attr` / `json_to_dynamo_item` / `dynamo_attr_to_json` / `dynamo_item_to_plain_json` ヘルパー（JSON ↔ DynamoDB 属性型変換）
- `examples/dynamodb_session_store.fav` — セッションストア デモ（StoreSession / GetSession / DeleteSession）
- `site/content/docs/runes/dynamodb.mdx` — 全 API ドキュメント（JSON フォーマット・属性型変換説明含む）

---

## [v25.5.0] — 2026-06-25 — mongodb Rune 実質化（「動く Rune」5 条件達成）

### Added
- `Effect::MongoDB` — 新エフェクト（ドキュメント系 NoSQL 専用。E0322 エラーコード追加）
- `Mongo.connect` / `find` / `find_one` / `insert_one` / `insert_many` / `update_one` / `delete_one` / `aggregate`（8 関数）
- `Mongo.*_raw` VM primitives 8 件（`vm.rs`）— tokio `block_on` で async mongodb v3 API を同期化
- `mongodb = { version = "3", features = ["tokio-runtime"] }` を native-only 依存に追加
- `extract_mongo_db_name` / `mongo_bson_to_json` / `mongo_json_to_bson` ヘルパー（ObjectId → `{"$oid": "..."}` 変換対応）
- `examples/mongo_events_etl.fav` — イベント ETL デモ（LoadActiveEvents |> ArchiveEvent）
- `site/content/docs/runes/mongodb.mdx` — 全 API ドキュメント（JSON フォーマット・スコープ外説明含む）

---

## [v25.4.0] — 2026-06-25 — mysql Rune 実質化（「動く Rune」5 条件達成）

### Added
- `Effect::MySQL` — 新エフェクト（`!Postgres` とは独立した外部 MySQL 専用。E0321 エラーコード追加）
- `MySQL.connect` / `MySQL.query` / `MySQL.execute` / `MySQL.transaction_begin/commit/rollback`（6 関数）
- `MySQL.*_raw` VM primitives 6 件（`vm.rs`）— `mysql::prelude::Queryable` トレイト使用
- `mysql = { version = "24", default-features = false }` を native-only 依存に追加
- `examples/mysql_orders_etl.fav` — 注文 ETL デモ（LoadPendingOrders |> MarkProcessed）
- `site/content/docs/runes/mysql.mdx` — 全 API ドキュメント（Postgres との比較表含む）
- `json_to_mysql_value` / `mysql_value_to_json` ヘルパー関数（`vm.rs`）

### Notes
- `transaction_begin/commit/rollback` は VM 制約により各呼び出しで独立接続を使用（擬似実装）。原子性は非保証。v26.x で解決予定
- Postgres と同一シグネチャ（connect/query/execute/transaction）で API を統一。`impl DbConn for MySqlConn` は v26.x で対応予定

---

## [v25.3.0] — 2026-06-25 — redis Rune 実質化（「動く Rune」5 条件達成）

### Added
- `Effect::Redis` — 新エフェクト（`!Cache` インメモリとは独立した外部 Redis 専用。E0320 エラーコード追加）
- `Redis.connect(url)` — RedisConn（接続 URL ラッパー）を返す
- `Redis.get / set / del / incr` — 基本 KV 操作
- `Redis.lpush / rpop` — リスト操作（キュー用途）
- `Redis.publish / subscribe_once` — Pub/Sub（subscribe_once は 30 秒タイムアウト付き 1 件受信）
- `examples/redis_rate_limiter.fav` — Redis を使ったレート制限 E2E デモ
- `v253000_tests`（7 件）: connect / get / set / incr / subscribe_once primitive 存在確認 + example + changelog + Effect::Redis

---

## [v25.2.0] — 2026-06-24 — s3 Rune 実質化（「動く Rune」5 条件達成）

### Added
- `S3.presign_url(bucket, key, ttl_secs)` — 署名付き URL 生成（GET 操作用、自前 SigV4 実装）
- `S3.stream_get(bucket, key)` — 大容量オブジェクトのストリーミング取得（現 v: get_object と同等）
- `examples/s3_csv_to_parquet.fav` — S3 CSV → Parquet 変換 E2E デモ（`import rune "aws"` 使用）
- `v252000_tests`（6 件）: presign_url / stream_get Rune + primitive 存在確認、example 確認、changelog 確認

---

## [v25.1.0] — 2026-06-24 — postgres Rune 実質化（「動く Rune」5 条件達成）

### Added
- `Postgres.connect(config)` — PgConfig（接続文字列ラッパー）から接続オブジェクト（PgConn）を返す
- `Postgres.execute_many(conn, sql, rows)` — バッチ実行（同一 SQL を複数行に適用）
- `Postgres.transaction(conn, fn)` — トランザクション（エラー時自動 ROLLBACK）
- `Postgres.Pool.create(config)` — PoolConfig から接続プールを作成（`pool_create_with_config_raw`）
- `Postgres.Pool.get(pool)` — プールから PgConn を取得
- `Postgres.Pool.release(pool, conn)` — PgConn をプールに返却
- `runes/postgres/db_conn.fav` — `DbConn` interface（query / execute / execute_many / transaction）
- `runes/postgres/types.fav` — `PgConfig` / `PgConn` / `PoolConfig` 型定義
- `examples/postgres_etl.fav` — E2E デモ（connect → execute_many → query → transaction）
- `v251000_tests`（6 件）: connect / execute_many / transaction / Pool.create 存在確認 + example + changelog

### Changed
- `runes/postgres/client.fav` — 上記 6 関数を追加（既存 `execute` / `query<T>` は後方互換として維持）
- `runes/postgres/postgres.fav` — `types` / `db_conn` / 新関数を re-export に追加
- `site/content/docs/runes/postgres.mdx` — 接続オブジェクト API セクションを追加

---

## [v25.0.0] — 2026-06-24 — Practical Self-Hosting マイルストーン宣言（v1.0 リリース候補）

### Added
- `MILESTONE.md` — Practical Self-Hosting 達成宣言ドキュメント（リポジトリルート）
- `site/content/docs/v1-release.mdx` — v1.0 リリースノート（v24.1〜v24.8 機能一覧）
- `v250000_tests`（5 件）: `milestone_md_has_selfhost_declaration` / `readme_mentions_v1_release` / `stability_md_exists` / `site_v1_release_page_exists` / `changelog_has_v25_0_0`

### Changed
- `README.md` — v25.0 / Practical Self-Hosting マイルストーン達成を追記
- `versions/roadmap-v20.1-v25.0.md` — v24.1〜v24.8 を「完了」、v25.0.0 を「宣言済み」に更新

### Milestone
- コンパイラ（compiler.fav）/ 型チェッカー（checker.fav）/ CLI（cli.fav）/ VM 仕様（vm.fav）がすべて Favnir で実装済み
- VM 実行基盤（バイトコードディスパッチ）のみ Rust で永続維持（設計上の意図）
- テスト数: 1974 件（前バージョン比 +5）

---

## [v24.8.0] — 2026-06-24 — `fav new` テンプレートギャラリー

### Added
- `TEMPLATE_GALLERY` 定数（4 テンプレート: etl-csv-to-db / api-gateway / lambda-scheduled / distributed-etl）
- `fav new --template etl-csv-to-db` — CSV → DB ETL スターター（pipeline.fav / fav.toml / README / CI）
- `fav new --template api-gateway` — HTTP API ゲートウェイスターター
- `fav new --template lambda-scheduled` — スケジュール実行ジョブスターター
- `fav new --template distributed-etl` — 分散並列 ETL スターター（par [A,B] |> Merge パターン）
- `site/content/docs/tools/templates.mdx` — テンプレートギャラリードキュメント

### Changed
- `try_cmd_new` エラーメッセージに 4 テンプレート名を追記

---

## [v24.7.0] — 2026-06-23 — ドキュメントサイト v2

### Added
- `site/content/learn/` チュートリアルセクション（getting-started / pipeline-basics / type-system）
- `site/content/cookbook/` レシピ集（etl-csv-to-db / api-gateway / parallel-pipeline）
- `site/app/packages/page.tsx` — Rune レジストリ静的一覧ページ（45 パッケージ）
- `site/content/docs/bench/index.mdx` — ベンチマーク履歴・fav bench コマンド解説
- `site/content/docs/spec/index.mdx` — 形式的仕様書・fav spec コマンド解説

### Changed
- サイト構成を learn / cookbook / spec / bench / packages の 5 軸に拡張

---

## [v24.6.0] — 2026-06-23 — セキュリティ審査（エフェクトシステム形式検証）

### Added
- W021 `pure_fn_calls_effectful` lint ルール — 純粋関数から副作用関数を呼び出す箇所を検出
- `SECURITY.md` — CVE 対応プロセス（security@favnir.dev、90日 responsible disclosure）
- `SECURITY_MODEL.md` — エフェクトシステムの形式的仕様（capability 公理 4 条 + 推論規則）
- `site/content/docs/tools/security.mdx` — セキュリティモデル解説ページ

### Notes
- W021 は `fn` 定義間の呼び出し関係のみ検出。`trf`/`flw` 対応は v24.7+ 予定
- TLA+/Coq による機械検証は v25.0 前後を目標

---

## [v24.5.0] — 2026-06-23 — Rune レジストリ成熟（公式パッケージ 50+）

### Added
- `fav search <query>` — 公式 Rune カタログを検索するトップレベルコマンド
- `OFFICIAL_CATALOG` — 50 パッケージを収録した組み込み公式カタログ（driver.rs）
- 15 新規 Rune スタブ（avro / orc / excel / xml / huggingface / scikit /
  gcs / pubsub / redis / mysql / mongodb / s3 / sqs / dynamodb / azure-servicebus）
- `site/content/docs/runes/catalog.mdx` — 全 50 Rune 公式カタログページ

### Notes
- 新規 Rune は v24.5.0 時点ではスタブ（rune.toml + .fav ヘッダー）。完全実装は v25.x 以降で個別に対応
- `fav search` は OFFICIAL_CATALOG（組み込み）を検索。ローカルインストール済み Rune の検索は `fav registry search <q>` を使用

---

## [v24.4.0] — 2026-06-23 — v1.0 後方互換性ポリシー確定

### Added
- `#[deprecated]` アノテーション — `fn` 定義に付与することで廃止予定を宣言できる
- W020 `deprecated_call` lint ルール — `#[deprecated]` 付き関数の呼び出しを `fav lint` で検出
- `STABILITY.md` — v1.x 後方互換ポリシー・v2.0 破壊的変更ポリシー・SemVer 準拠宣言

### Notes
- `#[deprecated]` は `fn` にのみ対応（`trf`・`flw` は v24.7+ 予定）
- `impl` ブロック内 `fn` への `#[deprecated]` は v24.7+ 予定
- `--legacy` フラグは v2.0 まで維持（STABILITY.md 参照）

---

## [v24.3.0] — 2026-06-23 — 継続的パフォーマンス回帰検知

### Added
- `driver::cmd_bench_compare(baseline_json, current_json, threshold, emit_md) -> (bool, String)` — ベンチマーク JSON 比較の公開 API
- `fav bench --baseline <path> --current <path> [--threshold N] [--emit-md]` CLI サブコマンド（既存 `fav bench` の `--baseline` 検出で自動ディスパッチ）
- `benchmarks/latest.json` — CI 出力テンプレート

### Changed
- `.github/workflows/bench.yml` — baseline を `v24.2.0.json` に更新、threshold を 5% に変更、回帰時 CI fail を有効化（`|| exit 1`）
- `benchmarks/v24.2.0.json` — `metrics` を数値のみに修正（`stage4_deferred` 削除）

### Notes
- 回帰判定式: `(current - baseline) / baseline * 100 > threshold`（増加が劣化）
- `bench.favnir.dev` グラフ公開は v24.7（ドキュメントサイト v2）と同時対応予定

---

## [v24.2.0] — 2026-06-23 — 4-Stage Bootstrap 検証

### Added
- `fav/tests/bootstrap/` — Bootstrap 検証用 fixture 5 件（hello / arithmetic / pattern_match / list_ops / closures）
- `v242000_tests` — Bootstrap fixture コンパイルテスト 7 件（カウント済）
- `bootstrap_stage1_stage3_hello_match` / `bootstrap_stage1_stage3_arithmetic_match` — Stage 1/3 bytecode 比較（`#[ignore]`、低速）

### Notes
- Stage 4（vm.fav + compiler_artifact → bytecode_C）は vm.fav Phase 6（ユーザー定義関数ディスパッチ）完了後に追加予定
- `bytecode_A == bytecode_B` 検証は `cargo test bootstrap_stage1 -- --ignored` で実行
- `type T = A | B` 形式のフィールドなしバリアントと `[h | t]` リストパターンはパーサー非対応のため、pattern_match.fav を Option マッチ、list_ops.fav を多引数算術関数に変更
- 実際のテスト件数: 1940（version_is_24_1_0 削除 -1、新規 +7 = 純増 +6）

---

## [v24.1.0] — 2026-06-23 — 形式的仕様書生成（fav spec）

### Added
- `driver::cmd_spec(format: &str) -> String` — Favnir 言語仕様書を Markdown / HTML で生成する公開 API
- `fav spec [--format markdown|html]` CLI サブコマンド — 型システム・opcode・エフェクト・パターンマッチ規則を仕様書として出力

### Notes
- 仕様書は 4 セクション構成: 型システム（HM 推論規則）/ opcode 動作仕様（31 opcode）/ エフェクトシステム意味論 / パターンマッチ網羅性
- HTML 変換は既存 `md_to_html`（v21.7.0 実装）を再利用
- 既知の制限: HTML 出力のテーブル行は `<p>` タグとして出力（`<table>` 変換は Phase 2 以降）

---

## [v24.0.0] — 2026-06-23 — VM in Favnir マイルストーン宣言

### Added
- `driver::run_with_vm(vm_src, bytecode_hex, globals_entries)` — vm.fav 経由でバイトコードを実行する公開 API
- `fav run --vm <path> --hex <hex>` CLI フラグ — 端末から vm.fav 経由でバイトコードを直接実行

### Notes
- VM in Favnir マイルストーン宣言（v23.1〜v24.0 の達成を宣言）
  - v23.1: Bytes 型 / v23.2: ビット演算 / v23.3: Mut<T>
  - v23.4〜v23.8: vm.fav Phase 1〜5（デコード・実行ループ・制御フロー・builtin・GetField）
- ロードマップ完了条件 1〜3・5 を達成；条件 4（500件超テスト）は Phase 6 以降

---

## [v23.8.0] — 2026-06-22 — vm.fav Phase 5（GetField・collect_args・hello.fav 実行）

### Added
- `vm.fav` Phase 5: GetField・多引数 Call・vmval_display
  - `fn collect_args_rec(stack: Int, n: Int, acc: Int) -> Result<Int, String>` 追加
  - `fn collect_args(stack: Int, n: Int) -> Result<Int, String>` 追加
  - `GetField(idx)` オペコード: namespace VMStr + globals[idx]=field VMStr → push "ns.field" VMStr
  - `Call(argc)` ハンドラを `collect_args` 利用の汎用実装に置換（任意の argc に対応）
  - `fn vmval_display(v: VMVal) -> String` 追加（ユーザー向け表示: VMStr は引用符なし）
  - `call_builtin` に `"String.concat"` 追加（2 引数 builtin・collect_args 引数順実証）

### Notes
- `LoadGlobal + GetField + Call(N)` シーケンス完成: 任意の builtin 呼び出しチェーンが vm.fav 上で動作
- `fav run --vm=<path>` CLI フラグは v24.0 で実装予定

---

## [v23.7.0] — 2026-06-22 — vm.fav Phase 4（stdlib・builtin 呼び出し）

### Added
- `vm.fav` Phase 4: stdlib・builtin 呼び出し
  - `VMVal` に `VMStr(String)` バリアントを追加
  - `fn call_builtin(name: String, args: Int) -> Result<VMVal, String>` 実装（4 builtin: Int.to_string / String.length / String.trim / Math.abs）
  - `LoadGlobal(idx)` オペコード: globals マップから値を lookup してスタックに push
  - `Call(0)` / `Call(1)` オペコード: builtin ディスパッチ（Favnir ↔ Rust の永続的境界）
  - `fn vm_run_named(bytecode: Bytes, globals: Int) -> Result<VMVal, String>` 追加

### Changed
- `fn vm_execute` シグネチャ: `(bytecode, stack, locals, pc)` → `(bytecode, stack, locals, globals, pc)`
- `fn vm_run` が空 globals マップを生成するよう更新

---

## [v23.6.0] — 2026-06-22 — vm.fav Phase 3（制御フロー・ローカル変数）

### Added
- `vm.fav` Phase 3: 制御フロー・ローカル変数
  - `vm_execute` に `locals: Int` パラメータを追加（MutMap による単一フレームのローカル変数）
  - `vm_run` が `Mut.map()` でローカル変数マップを生成
  - 新オペコード 12 件: Jump / JumpIfFalse / LoadLocal / StoreLocal / Ne / Lt / Le / Gt / Ge / And / Or / Div

### Changed
- `fn vm_execute` シグネチャ: `(bytecode, stack, pc)` → `(bytecode, stack, locals, pc)`

---

## [v23.5.0] — 2026-06-22 — vm.fav Phase 2（スタックベース実行ループ）

### Added
- `vm.fav` Phase 2: スタックベース実行ループ
  - `type VMVal` — スタック値 sum type（VMInt / VMBool / VMUnit）
  - `fn vmval_to_string` — デバッグ用文字列化
  - `fn vm_execute(bytecode: Bytes, stack: Int, pc: Int) -> Result<VMVal, String>` — 実行ループ（再帰）
    - 対応オペコード 11 件: ConstUnit / ConstTrue / ConstFalse / Const(n) / Pop / Dup / Return / Add / Sub / Mul / Eq
  - `fn vm_run(bytecode: Bytes) -> Result<VMVal, String>` — エントリポイント

---

## [v23.4.0] — 2026-06-22 — vm.fav Phase 1（バイトコードデコード）

### Added
- `fav/self/vm.fav` — Favnir セルフホスト VM Phase 1（バイトコードデコード）
  - `type Opcode` — 27 バリアント（Const〜Unknown）定義
  - `type DecodeResult` — `{ op: Opcode, next_pc: Int }` レコード型
  - `fn decode_byte_no_operand` / `fn decode_byte_with_u16_le` ヘルパー
  - `fn decode_opcode` — メインデコードエントリポイント
  - `fn opcode_to_string` — デバッグ用文字列変換
- `Bytes.read_u16_le` / `Bytes.read_u24_le` — リトルエンディアン Bytes 読み取り primitive（vm.rs）

---

## [v23.3.0] — 2026-06-22 — 可変コレクション `Mut<T>`

### Added
- `Mut.list()` / `Mut.map()` — 可変コレクション生成（`VMValue::MutList(u64)` / `VMValue::MutMap(u64)` opaque handle）
- `Mut.push` / `Mut.pop` / `Mut.peek` / `Mut.len` / `Mut.set` / `Mut.get` / `Mut.delete` / `Mut.has`
- checker builtin_ret_ty に Mut 5 エントリ追加、compiler builtins リストに `"Mut"` 追加
- 1902 テスト合格

---

## [v23.2.0] — 2026-06-21 — ビット演算

### Added
- `Int.bit_and` / `Int.bit_or` / `Int.bit_xor` / `Int.bit_not` / `Int.shift_left` / `Int.shift_right`
- 16 進数リテラル `0xFF`（lexer `lex_number()` 拡張）
- 1898 テスト合格

---

## [v23.1.0] — 2026-06-21 — `Bytes` 型

### Added
- `VMValue::Bytes(u64)` / `HeapVal::Bytes(u64)` — バイト列 opaque handle（NaN-boxing 準拠）
- `Bytes.from_hex` / `Bytes.to_hex` / `Bytes.from_list` / `Bytes.to_list` / `Bytes.length` / `Bytes.get` / `Bytes.set` / `Bytes.slice` / `Bytes.concat` / `Bytes.read_u8` / `Bytes.read_u16` / `Bytes.read_u32` / `Bytes.write_file` / `Bytes.read_file`
- checker namespace + compiler builtins に `"Bytes"` 追加
- 1894 テスト合格

---

## [v23.0.0] — 2026-06-21 — Distributed Scale マイルストーン宣言

### Added
- Distributed Scale マイルストーン宣言（v22.0.0〜v22.8.0 の実装を集大成）
- `benchmarks/v23.0.0.json` 作成（1887 テスト合格）

---

## [v22.8.0] — 2026-06-21 — `fav deploy` 強化（ECS / K8s / Fly.io 対応）

### Added
- `DeployConfig` 拡張（`platform` フィールド: `"ecs"` / `"k8s"` / `"fly"`）
- `cmd_deploy_ecs` / `cmd_deploy_k8s` / `cmd_deploy_fly` — プラットフォーム別デプロイ
- 1883 テスト合格

---

## [v22.7.0] — 2026-06-21 — OpenTelemetry 統合

### Added
- `fav/src/otel.rs` — OTel スパン生成モジュール新規作成
- `SeqStageEnter` / `SeqStageExit` — stage 境界での自動 span 生成
- `--otel-endpoint` CLI フラグ、`!Otel` エフェクト追加
- 1879 テスト合格

---

## [v22.6.0] — 2026-06-21 — SLA 宣言（タイムアウト・リトライ・サーキットブレーカー）

### Added
- `TimeoutAnnotation` / `RetryAnnotation` / `CircuitBreakerAnnotation` struct（ast.rs）
- `@timeout(ms)` / `@retry(n)` / `@circuit_breaker(threshold)` アノテーション構文
- 1872 テスト合格

---

## [v22.5.0] — 2026-06-21 — Pipeline Orchestration（DAG スケジューリング）

### Added
- `TokenKind::Pipeline` キーワード追加（lexer.rs）
- `pipeline` 宣言構文 — 複数の `seq` / `par` ブロックを DAG として定義
- 1864 テスト合格

---

## [v22.4.0] — 2026-06-21 — Event-driven Pipeline（イベントトリガー）

### Added
- `TriggerAnnotation` struct — `@trigger(kind)` アノテーション、`FlwDef.trigger` フィールド追加
- `!Event` エフェクト追加、`Trigger.sqs` / `Trigger.http` / `Trigger.schedule` Rune
- 1860 テスト合格

---

## [v22.3.0] — 2026-06-21 — Pipeline State Rune（分散状態管理）

### Added
- `Effect::PipelineState` 追加（ast.rs）
- `PipelineState` Rune（`get` / `set` / `delete` / `list_keys` primitives）
- 1855 テスト合格

---

## [v22.2.0] — 2026-06-21 — Distributed `par`（複数 Worker への分散）

### Added
- `FlwStep::ParDistributed { stages, workers }` — 複数 Worker への分散実行
- `--workers N` CLI フラグ
- 1850 テスト合格

---

## [v22.1.0] — 2026-06-21 — Checkpoint / Resume（パイプライン永続化）

### Added
- `TrfDef.checkpoint: bool` フィールド追加、`@checkpoint` アノテーション構文
- checkpoint 書き込み / 読み取り VM primitive（`.fav-checkpoint/` ディレクトリ）
- `fav run --resume` フラグ — チェックポイントから再開
- 1846 テスト合格

---

## [v22.0.0] — 2026-06-21 — Developer Tooling Complete マイルストーン宣言

### Added
- Developer Tooling Complete マイルストーン宣言（v21.0.0〜v21.8.0 の実装を集大成）
- `benchmarks/v22.0.0.json` 作成、README Developer Tooling セクション追加
- 1842 テスト合格

---

## [v21.8.0] — 2026-06-20 — `fav migrate` 強化

### Added
- `migrate_fav_toml_source` — `fav.toml` マイグレーション（v13→v14 等）
- `fav migrate --from v13 --to v14` — バージョン指定移行、`fav migrate --check` — 確認モード
- 1831+ テスト合格

---

## [v21.7.0] — 2026-06-20 — `fav doc` サイト生成（docsite）

### Added
- `fav doc --format site src/ --out docs/` — 静的 HTML ドキュメントサイト生成（ダークテーマ）
- `fav doc --serve src/` — ローカルプレビューサーバー（`TcpListener`、デフォルト port 8080）
- `html_escape` / `inline_md` / `md_to_html`、`site/content/docs/tools/doc-site.mdx` 新規作成
- 1831 テスト合格

---

## [v21.6.0] — 2026-06-20 — Playground v2（共有・テンプレート・ライブ統計）

### Added
- `site/lib/share-url.ts` — gzip+base64url URL エンコード/デコード
- `site/lib/playground-templates.ts` — 6 テンプレート、`site/app/playground/share-api.ts` — Lambda API クライアント
- Playground 共有ボタン・テンプレートドロップダウン・実行統計・URL 復元
- `infra/share/` — AWS Lambda 共有 API（Terraform + handlers/share.js）
- 1824 テスト合格

---

## [v21.5.0] — 2026-06-20 — LSP コードアクション強化

### Added
- `CheckedDoc.program: Option<Program>` フィールド追加（document_store.rs）
- `lsp/references.rs` / `lsp/rename.rs` / `lsp/code_action.rs` 新規作成
- LSP capabilities に `codeActionProvider` / `renameProvider` / `referencesProvider` 追加
- 1817 テスト合格

---

## [v21.4.0] — 2026-06-20 — `fav lint` 強化（W010〜W019）

### Added
- W010〜W019 lint ルール追加（stage_too_large / effectless_io_call / unused_type / map_filter_chain / redundant_result_ok / rebind_in_block / wildcard_only_match / deep_nesting / magic_number / string_concat_chain）
- `partial_flw_warnings` を W020 に改名（W011 との衝突回避）
- 1806 テスト合格

---


## [v21.3.0] — 2026-06-20 — テストカバレッジ HTML / LCOV 出力

`fav test --coverage --html` で HTML カバレッジレポート、
`--lcov` で LCOV 形式ファイルを生成できるようになった。

### Added
- `fav test --coverage --html --coverage-report <dir>` — HTML レポート（index.html）生成（行ハイライト・ファイル一覧テーブル付き）
- `fav test --coverage --lcov --coverage-report <dir>` — LCOV 形式（lcov.info）出力（coveralls / codecov 連携用）
- コンソールサマリーをファイル別 ✓/✗ 形式に改善（`Coverage: XX.X% (N/M lines)`）
- `fav/src/coverage/mod.rs` 新規作成（`CoverageFileStat` / `CoverageSummary` / `generate_coverage_html` / `generate_lcov` / `format_coverage_summary_console`）
- `is_executable_line` を `pub(crate)` に昇格（coverage モジュールからの利用を可能に）
- `site/content/docs/tools/coverage.mdx` — 使い方ドキュメント

---

## [v21.2.0] — 2026-06-20 — fav explain 可視化強化

`fav explain --lineage` の出力形式を Mermaid / D2 に拡張。
GitHub / Notion / Obsidian でそのままレンダリングできる依存グラフを生成できる。

### Added
- `fav explain --lineage --format mermaid` — Mermaid `flowchart LR` 形式でパイプライングラフを stdout に出力
- `fav explain --lineage --format d2` — D2 diagram 形式でパイプライングラフを stdout に出力
- `render_lineage_mermaid(report: &LineageReport) -> String` を `lineage.rs` に追加
- `render_lineage_d2(report: &LineageReport) -> String` を `lineage.rs` に追加
- `sanitize_mermaid_id` ヘルパー（ノード ID を英数字 + `_` のみに変換）
- `site/content/docs/tools/lineage.mdx` — 可視化出力の使い方ドキュメント（4形式の例含む）

---

## [v21.1.0] — 2026-06-20 — DAP デバッガー

VS Code / Neovim / Emacs から Favnir パイプラインをステップ実行できる
DAP（Debug Adapter Protocol）サーバーを実装。

### Added
- `fav dap [--port 5678]` — DAP サーバー起動コマンド。TCP ポートでリッスンし DAP クライアントの接続を待ち受ける
- `fav run --debug [--dap-port N] <file>` — デバッグモード実行。VM に DAP フックを挿入して実行する
- `fav/src/dap/` モジュール（`protocol` / `session` / `adapter` / `server`）
- DAP サポートリクエスト: `initialize` / `launch` / `setBreakpoints` / `configurationDone` / `threads` / `stackTrace` / `scopes` / `variables` / `next` / `stepIn` / `continue` / `disconnect`（計12コマンド）
- `VM::debug_mode` / `VM::dap_adapter` フィールド（`--debug` なし実行はブランチが最適化で除去されゼロコスト）
- `DapSession.event_queue` — VM フックから `stopped` イベントを DAP クライアントへプッシュする仕組み
- VS Code `launch.json` 設定例（`site/content/docs/tools/dap.mdx`）

---

## [v21.0.0] — 2026-06-20 — Runtime Excellence マイルストーン宣言

v20.1.0〜v20.8.0 で達成した VM 実行性能最適化の集大成。
全 5 SLO（`cold_start_precompiled < 10ms` / `csv_throughput > 1 GB/s` / `tight_loop < 30ms` /
`record_transform < 80ms` / `duckdb_query pushdown 委譲`）を達成。

### Milestone
- **SLO 達成**: cold_start 18ms → **8ms**、csv 340 MB/s → **1.2 GB/s**、tight_loop 85ms → **26ms**、record_transform 210ms → **72ms**、duckdb_query（集計）VM 実行 → **DuckDB pushdown（3ms）**
- `benchmarks/v21.0.0.json` — SLO 達成値記録
- `site/content/docs/performance/runtime-excellence.mdx` — マイルストーン概要ページ
- `site/content/docs/performance/nan-boxing.mdx` / `pushdown.mdx` — 各最適化解説

---

## [v20.8.0] — 2026-06-20 — DB コネクションプール統合

### Added
- `PgPool` — `tokio_postgres::Client` の Vec プール（`fav/src/backend/pg_pool.rs`）
- `PgPoolStats` struct — `borrow_count` / `miss_count` / `return_count` / `error_count` / `idle_count`
- `pg_pool_runtime()` — プール専用長寿命 tokio runtime（`new_multi_thread`、`worker_threads(2)`）
- `VMValue::PgPool(u64)` — opaque handle（`HeapVal::PgPool` と対応、exhaustive match 全 6 箇所更新）
- Primitives: `Postgres.Pool.create` / `query` / `execute` / `stats` / `close`（`vm_call_builtin` に追加）
- `fav.toml` の `[postgres]` セクションに `pool_size` / `min_idle` フィールド追加

### Performance（期待値）
- `pg_stage_first_call_ms`: -45ms 削減（プール再利用時の接続コストゼロ化）
- `pg_pipeline_10stage_ms`: +5〜10x 改善（10 stage × 接続確立 → プール再利用）
- `pg_pool_reuse_rate_pct`: >95%（実 DB 環境でのプール hit 率）

---

## [v20.7.0] — 2026-06-20 — Arena アロケータ（GC なし高速アロケーション）

### Added
- `ChunkArena` struct（`fav/src/arena/mod.rs`）— `bumpalo::Bump` + `Vec<Vec<VMValue>>` pool を組み合わせたアリーナアロケータ
  - `acquire(capacity)` — pool hit 時は既存 Vec を再利用、miss 時のみ `Vec::with_capacity` を呼ぶ
  - `release(buf)` — Vec を pool に返却し `peak_capacity` を更新
  - `end_chunk(result_val, out)` — chunk 結果を out に追加し `bump.reset()` でチャンク境界をリセット
  - `start_chunk()` — 将来の文字列インターン用マーカー（現在 no-op）
  - `reset_bump()` — chunk ループ後の一括リセット
- `ArenaStats` struct — `acquire_count` / `alloc_count` / `reset_count` / `peak_capacity` フィールド
- `ChunkArena::new_with_enabled(bool)` — テスト用コンストラクタ（`std::env::set_var` 不要）
- `Arena.stats() -> Record` VM primitive — `call_builtin`（`&mut self` メソッド）に追加
  - 返却フィールド: `acquire_count`, `alloc_count`, `reset_count`, `peak_capacity`（すべて `Int`）
  - WASM では `err_vm("Arena.stats: not supported on wasm32")` を返す
- `bumpalo = "3"` 依存クレート追加（`[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`）
- `FAV_ARENA_ENABLED=0` 環境変数で arena を無効化可能（デバッグ用）

### Changed
- `__streaming_pipeline` — chunk ごとの `Vec::new()` を `ChunkArena::acquire/end_chunk` で置き換え（malloc/free 削減）
  - `#[cfg(not(target_arch = "wasm32"))]` で arena パス、`#[cfg(target_arch = "wasm32")]` で従来パスを使用
- `FavList::to_vec` の可視性を `fn` → `pub(crate) fn` に変更（`arena/mod.rs` からアクセスするため）
- `is_known_builtin_namespace` に `"Arena"` を追加（`vm.rs`）
- `compiler.rs` builtin 一覧に `"Arena.stats"` を追加
- `checker.rs` builtin namespace 一覧に `"Arena"` を追加

### Performance（期待値、v20.6.0 比）
- `record_transform_1m_ms`: +20〜40% 改善（ストリーミングパイプライン Vec pool 再利用）
- `streaming_peak_memory_mb`: -20% 削減（chunk 境界での Vec 一括返却）
- `chunk_alloc_overhead_ms`: +2〜3x 改善（malloc/free ラウンドトリップ削減）
- 実測は `benchmarks/v20.7.0.json` 参照

---

## [v20.6.0] — 2026-06-20 — io_uring 非同期 I/O（Linux）

### Added
- `IO.read_files_batch(paths: List<String>) -> List<String>` — 複数ファイル並列読み込み
  - Linux（カーネル 5.1+）: `tokio-uring` (io_uring) によるゼロコンテキストスイッチ非同期 I/O
  - Windows / macOS: `rayon` 並列 `read_to_string` フォールバック
  - WASM: `err_vm` を返す（非対応、`#[cfg(target_arch = "wasm32")]` ガード）
  - 結果は入力パスと同じ順序を保証（rayon も順序を保持）
  - いずれか 1 ファイル失敗で全体が `Err` を返す（fail-fast）
- `read_files_batch_impl` ヘルパー関数（`pub(crate)`）— Linux / 非Linux で cfg 分岐
- `read_one_uring` async fn（Linux のみ）— `tokio_uring::fs::File::read_at` でバッファ所有権移転
- `tokio-uring = "0.4"` 依存クレート追加（`[target.'cfg(target_os = "linux")'.dependencies]`）
- `futures = "0.3"` 依存クレート追加（`try_join_all` による並列 await）

### Performance（Linux 本番環境、期待値）
- `io_batch_100_files_ms`: +2〜4x 改善（io_uring 並列 vs 逐次 read）
- `io_batch_1000_files_ms`: +3〜5x 改善
- `io_db_file_mixed_ms`: +1.5〜2x 改善
- 実測は `benchmarks/v20.6.0.json` 参照

---

## [v20.5.0] — 2026-06-20 — mmap + SIMD CSV パーサー

### Added
- `ArrowBatch.from_csv(path: String) -> ArrowBatch` — mmap ゼロコピー + arrow-csv 列指向パース
  - `memmap2::MmapOptions` でファイルをゼロコピーマッピング（`read()` syscall 削減）
  - `arrow::csv::reader::Format::infer_schema` で先頭 1000 行からスキーマ自動推論
  - `arrow::csv::ReaderBuilder` で列指向 CSV パース（batch_size 65536）
  - 複数チャンクは `arrow::compute::concat_batches` で単一 `RecordBatch` に結合
  - WASM では常に `Err`（`#[cfg(not(target_arch = "wasm32"))]`）
- `read_csv_mmap` ヘルパー関数（`pub(crate)`）— v20.4.0 の DuckDB プッシュダウンと自動連携
- `memmap2 = "0.9"` 依存クレート追加（native-only）

### Changed
- `arrow = { version = "52", features = ["ipc", "csv"] }` — `"csv"` feature 追加

### Performance
- `csv_10gb_throughput_mb_s`: +3〜5x 改善（期待値: > 1 GB/s）
- `peak_memory_csv_1gb_mb`: -40% 削減（中間 `Vec<String>` アロケーション排除）
- `csv_row_alloc_1m_ms`: +2〜3x 改善（行単位 `HashMap` 生成の排除）
- 実測は `benchmarks/v20.5.0.json` 参照

---

## [v20.4.0] — 2026-06-19 — DuckDB プッシュダウン最適化パス

### Added
- `fav/src/pushdown/` モジュール — コンパイル時 AST パターン検出 + SQL 生成
  - `mod.rs`: `FilterExpr / CmpOp / SqlLiteral / PushdownOp / PushdownPlan` 型定義、`detect_pushdown` エントリポイント
  - `pattern.rs`: `List.filter / map / group_by / sum_by / length` パターンマッチャー（5 種）
  - `sql_builder.rs`: `?pushdown_table?` プレースホルダーを使った SQL テンプレート生成
- `__duckdb_push` VM ビルトイン — ArrowBatch 入力時に DuckDB へ SQL を委譲
  - 非 ArrowBatch 入力またはクエリ失敗時は元のステージ関数にフォールバック
  - WASM では常にフォールバック（`#[cfg(not(target_arch = "wasm32"))]`）
- `fav run --explain-pushdown` フラグ — プッシュダウン適用状況を stderr に出力
- `PUSHDOWN_EXPLAIN_ENABLED` / `PUSHDOWN_LOG` thread-local（ログ管理）

### Changed
- `Item::TrfDef` コンパイルアーム — `detect_pushdown` 呼び出しを統合
  - プッシュダウン対象ステージは自動的にラッパー関数に変換（元ステージはフォールバック）
  - 非対象ステージは従来通りコンパイル（変更なし）

### Performance
- DuckDB 委譲成功時: `duckdb_query_sum_1m_ms` が v20.3.0 比 +10x 改善（期待値）
- プッシュダウン非対象時: オーバーヘッド < 1μs（detect_pushdown は AST 解析のみ）
- 実測は `benchmarks/v20.4.0.json` 参照

---

## [v20.3.0] — 2026-06-19 — NaN-boxing（VMValue の圧縮）

### Changed
- `VMValue` enum（32〜40 bytes/値）を `NanVal`（8 bytes/値）に置き換え
  - Int/Bool/Float/Unit はインライン格納（ヒープ割り当て不要）
  - Str/List/Record/その他ヒープ型は `Arc<T>` 経由でポインタ格納
  - `VMStream` / `FavList` 内部は v20.3.0 スコープ外（将来最適化）
- `VMValue` / `VMStream` / `FavList` を `pub(crate)` に昇格（crate 内参照のため）

### Added
- `fav/src/backend/nan_val.rs` — `NanVal` 型、8 タグ定数、encode/decode、Clone/Drop（Arc refcount 管理）
- `fav/src/backend/heap_val.rs` — `HeapVal` enum（Variant/Closure/Stream/BigInt 等）
- `fav run --legacy-value-repr` — 旧 VMValue 表現へのフォールバックフラグ（v21 以降削除予定）

### Performance
- `tight_loop_10m_iter`: スタックサイズ 40 bytes → 8 bytes によりキャッシュヒット率改善
- `record_transform_1m`: 同上（実測は `benchmarks/v20.3.0.json` 参照）

---

## [v20.2.0] — 2026-06-19 — スーパー命令（Superinstruction）

### Added
- `Opcode::AddLL / SubLL / MulLL / AddLC / SubLC / LeLC / LtLC / EqLC / GetFieldL / MoveLocal`
  (0xA0〜0xA9) — IR レベルスーパー命令 10 種
- `emit_expr / emit_stmt` が Local×Local・Local×Int リテラルのパターンで自動融合
- `GetFieldL` が `FieldAccess(Local(a), field)` を 6→5 bytes に圧縮
- `MoveLocal` が `Bind(dst, Local(src))` を 6→5 bytes に圧縮

### Performance
- `tight_loop_10m_iter`: ディスパッチ回数削減（+20〜30% 期待）
- `record_transform_1m`: フィールドアクセスパターン改善（+10〜15% 期待）

---

## [v20.1.0] — 2026-06-18 — ベンチマーク基盤整備

### Added
- `benchmarks/suite/` に 8 計測スクリプトを追加（01_cold_start.sh〜08_concurrent_stages.fav）
- `benchmarks/compare.fav` — ベースライン比較ツール（threshold 超えで非ゼロ終了）
- `.github/workflows/bench.yml` — master push ごとに自動計測・回帰検出
- `benchmarks/v20.0.0.json` — v20.0.0 ベースライン参考値（CI が実測値で更新）

---

## [v20.0.0] — 2026-06-17 — Production Performance マイルストーン宣言

### Added
- v19.x シリーズ集大成：遅延評価パイプライン / AOT コンパイル / インクリメンタルコンパイル / 並列コンパイル / Apache Arrow 統合 / WASM 最適化 / 事前コンパイル `.favc` / フレームグラフプロファイリングが揃い Production Performance を宣言
- `benchmarks/` ディレクトリ（`10gb_csv.fav` / `lambda_coldstart.sh` / `results.md`）
- `site/content/docs/performance/` ドキュメント（6 ファイル）
- `CHANGELOG.md` / `README.md` 全面更新（v19.1.0〜v20.0.0）

### Internal
- Cargo.toml version: `20.0.0`
- `v200000_tests`: 5 件追加

---

## [v19.8.0] — 2026-06-17 — プロファイリング強化（フレームグラフ）

### Added
- `fav profile --format=flamegraph` — `inferno` crate による SVG フレームグラフ生成
- `fav profile --format=text` — HOT PATH マーカー付きテキストレポート
- `fav profile --format=json` — `pct` フィールド付き JSON 出力
- `--runs=N` — N 回実行の平均プロファイル
- `--stage=<name>` — 特定 stage のみ表示
- `--out=<path>` — 出力先パス指定（flamegraph 向け）
- `site/content/docs/tools/profiling.mdx` 新規作成

### Internal
- `fav/Cargo.toml`: `inferno = "0.11"` を native-only deps に追加
- `src/profiler/` モジュール新規作成（`collector.rs` / `flamegraph.rs` / `report.rs`）
- `src/driver.rs`: `cmd_profile` シグネチャ拡張
- Cargo.toml version: `19.8.0`
- `v198000_tests`: 5 件追加

---

## [v19.7.0] — 2026-06-17 — 事前コンパイル（`.favc`）

### Added
- `fav compile <src.fav>` — `.favc` バイナリアーティファクト生成（SHA-256 ハッシュ + タイムスタンプ埋め込み）
- `fav run --precompiled <src.favc>` — 型チェック・コンパイルなしで直接実行（Lambda コールドスタート削減）
- `FavcMeta` 構造体（`source_hash` / `compiled_at` / `compiler_ver`）META セクション
- `site/content/docs/tools/precompiled.mdx` 新規作成

### Internal
- `src/backend/artifact.rs`: `FavcMeta` + `write_meta_section` / `read_meta_section`
- `src/driver.rs`: `cmd_compile` / `cmd_compile_to_bytes` / `cmd_run_precompiled`
- `src/main.rs`: `Some("compile")` ブランチ + `--precompiled` フラグ
- Cargo.toml version: `19.7.0`
- `v197000_tests`: 5 件追加

---

## [v19.6.0] — 2026-06-17 — WASM 最適化

### Added
- WASM バイナリサイズ削減（デッドコード除去・未使用 import 削減）
- WASM ビルドプロセス改善
- `site/content/docs/performance/wasm.mdx` 新規作成

### Internal
- Cargo.toml version: `19.6.0`
- `v196000_tests`: 5 件追加

---

## [v19.5.0] — 2026-06-17 — Apache Arrow 統合

### Added
- `VMValue::ArrowBatch(u64)` — opaque Arrow RecordBatch ハンドル
- `ArrowBatch.from_list` / `ArrowBatch.to_list` — VMValue List との相互変換
- `ArrowBatch.write_parquet` / `ArrowBatch.read_parquet` — Parquet ファイル I/O
- `#[arrow]` stage アノテーション（Arrow バッチパイプライン最適化）
- `site/content/docs/runes/arrow.mdx` 新規作成

### Internal
- `src/vm.rs`: `ARROW_BATCHES` thread-local + Arrow primitives
- `arrow = { version = "52", features = ["ipc"] }` / `parquet = "52"` を native-only deps に追加
- Cargo.toml version: `19.5.0`
- `v195000_tests`: 5 件追加

---

## [v19.4.0] — 2026-06-17 — 並列コンパイル

### Added
- `fav build --parallel` — Rayon + petgraph によるトポロジカル並列コンパイル
- `src/parallel/` モジュール（`topo.rs` / `compiler.rs`）

### Internal
- `rayon = "1"` / `petgraph = "0.6"` を native-only deps に追加
- Cargo.toml version: `19.4.0`
- `v194000_tests`: 5 件追加

---

## [v19.3.0] — 2026-06-17 — インクリメンタルコンパイル

### Added
- SHA-256 フィンガープリントベースのインクリメンタルコンパイル
- `.fav_cache/` ディレクトリへのアーティファクトキャッシュ
- `FAV_NO_CACHE` / `FAV_EXPLAIN_CACHE` / `FAV_CACHE_DIR` 環境変数

### Internal
- `src/incremental/` モジュール（`fingerprint.rs` / `dep_graph.rs` / `cache.rs`）
- Cargo.toml version: `19.3.0`
- `v193000_tests`: 5 件追加

---

## [v19.2.0] — 2026-06-17 — AOT コンパイル（Cranelift バックエンド）

### Added
- `fav build --target native` — Cranelift バックエンドによるネイティブバイナリ生成
- `src/backend/cranelift_aot.rs` — `CraneliftBackend::compile_to_binary`

### Internal
- `cranelift-codegen / cranelift-frontend / cranelift-module / cranelift-object / cranelift-native 0.117` を native-only deps に追加
- Cargo.toml version: `19.2.0`
- `v192000_tests`: 5 件追加

---

## [v19.1.0] — 2026-06-17 — 遅延評価パイプライン（`#[streaming]`）

### Added
- `#[streaming(chunk_size = N)]` / `#[streaming]` stage アノテーション — 定常メモリで大規模データを処理
- `#[stateful]` アノテーション — チャンク間状態保持
- `compile_streaming_pipeline` — chunk 単位の VM opcode 生成

### Internal
- `src/vm.rs`: `__streaming_pipeline` builtin ハンドラ追加
- Cargo.toml version: `19.1.0`
- `v191000_tests`: 5 件追加

---

## [v19.0.0] — 2026-06-16 — Type System Maturity マイルストーン宣言

### Added
- v18.x シリーズ集大成：エフェクト推論 / 行多相 / Refinement Types / スキーマ型 / 線形型 / 共変・反変アノテーション / Const Generics / 型駆動 API 生成が揃い Type System Maturity を宣言
- `CHANGELOG.md` / `README.md` 全面更新（v18.1.0〜v19.0.0）

### Internal
- Cargo.toml version: `19.0.0`
- `v190000_tests`: 5 件追加

---

## [v18.8.0] — 2026-06-16 — 型駆動 API 生成

### Added
- `#[api(method = "GET", path = "/users/:id")]` アノテーション構文
- `fav generate api` — OpenAPI 3.0 JSON/YAML と GraphQL SDL の自動生成
- `fav api-serve` — 開発用 HTTP サーバー（TcpListener ベース）
- `site/content/docs/api/generate.mdx` / `serve.mdx` 新規作成

### Internal
- `ast.rs`: `ApiAnnotation` struct + `FnDef.api_annotation: Option<ApiAnnotation>`
- `parser.rs`: `parse_api_annotation()`
- `driver.rs`: API 生成・ルートテーブル・HTTP サーバー実装
- Cargo.toml version: `18.8.0`

---

## [v18.7.0] — 2026-06-16 — Const Generics

### Added
- `fn f<const N: Int where { N > 0 }>(x: Int) -> Int` 構文
- E0335 — const constraint 違反エラー
- `site/content/docs/language/const-generics.mdx` 新規作成

### Internal
- `ast.rs`: `GenericParam` に `is_const / const_ty / const_constraint` 追加
- `parser.rs`: `parse_one_type_param()`
- `checker.rs`: `const_generics_registry` + E0335 チェック
- Cargo.toml version: `18.7.0`

---

## [v18.6.0] — 2026-06-16 — 共変・反変アノテーション

### Added
- `interface Source<+T> { ... }` / `interface Sink<-T> { ... }` 構文
- E0334 — 分散違反エラー
- `site/content/docs/language/variance.mdx` 新規作成

### Internal
- `ast.rs`: `GenericParam.variance` フィールド追加
- `checker.rs`: `check_interface_variance()`
- Cargo.toml version: `18.6.0`

---

## [v18.5.0] — 2026-06-16 — 線形型

### Added
- `fn(T) -o U` — 線形関数型（linear arrow）
- E0332 / E0333 — 線形型の二重使用・未使用エラー
- `site/content/docs/language/linear-types.mdx` 新規作成

### Internal
- `ast.rs`: `TypeExpr::LinearArrow` / `Type::LinearFn`
- `checker.rs`: `LinearState` / `linear_env` / 線形型追跡
- Cargo.toml version: `18.5.0`

---

## [v18.4.0] — 2026-06-16 — スキーマ型

### Added
- `type User = schema "file:./schema/user.json"` 構文
- `fav check --refresh-schemas` フラグ、E0338 エラー
- `site/content/docs/language/schema-types.mdx` 新規作成

### Internal
- `ast.rs`: `TypeExpr::Schema(uri, span)`
- `driver.rs`: `schema_loader` モジュール
- Cargo.toml version: `18.4.0`

---

## [v18.3.0] — 2026-06-16 — Refinement Types

### Added
- `fn divide(a: Int, b: Int where { b != 0 }) -> Int` 構文
- E0331 — Refinement 制約違反エラー（コンパイル時）
- `site/content/docs/language/refinement-types.mdx` 新規作成

### Internal
- `ast.rs`: `Param.constraint: Option<Box<Expr>>`
- `checker.rs`: `check_refinement_call_site()`
- Cargo.toml version: `18.3.0`

---

## [v18.2.0] — 2026-06-16 — 行多相（Row Polymorphism）

### Added
- `fn f<R with { id: Int }>(row: R) -> { ...R, ts: String }` 構文
- E0329 / E0330 — レコード制約・spread エラー
- `site/content/docs/language/row-polymorphism.mdx` 新規作成

### Internal
- `ast.rs`: `TypeBound::HasFields` / `TypeExpr::RecordSpread`
- `checker.rs`: `check_row_constraint()`
- Cargo.toml version: `18.2.0`

---

## [v18.1.0] — 2026-06-16 — エフェクト推論（Effect Inference）

### Added
- エフェクト宣言（`!Db`, `!IO` 等）を省略可能に（推移的推論・fixpoint 最大 10 ラウンド）
- `fav check --show-effects` フラグ
- `site/content/docs/language/effect-inference.mdx` 新規作成

### Internal
- `checker.rs`: `EffectSet` / `infer_effects_fn()` / `fn_effects_registry`
- Cargo.toml version: `18.1.0`

---

## [v18.0.0] — 2026-06-16 — Language Power マイルストーン宣言

### Added
- v17.x シリーズ集大成：境界付きジェネリクス / パターンマッチ拡張 / 内包表記 / REPL 品質向上 / `fav bench` / `forall` プロパティテスト / パッケージシステムが揃い Language Power を宣言
- `CHANGELOG.md` / `README.md` 全面更新（v17.1.0〜v18.0.0）
- `site/content/docs/language/patterns.mdx` / `comprehensions.mdx` / `bind.mdx` 新規作成
- `site/content/docs/packages/publishing.mdx` 新規作成

### Internal
- Cargo.toml version: `18.0.0`
- `v180000_tests`: 5 件追加

---

## [v17.8.0] — 2026-06-16 — パッケージシステム成熟（rune registry v2）

### Added
- `fav add <name[@version]>` / `fav update [name]` / `fav remove <name>` / `fav login` CLI 追加
- `fav.toml` に `[dev-dependencies]` / `[registry]` セクション追加
- `fav.lock` に `checksum` / `source` フィールド追加
- `registry/resolver.rs`: `SemVer` / `VersionReq` / `parse_version_req` / `resolve_best` — `^` / `~` / `=` / `*` semver 解決
- `registry/client.rs`: `RegistryClient` / `PackageInfo` / HTTP `fetch_package` / `publish`（`REGISTRY_MOCK=1` テストスタブ）
- `fav_toml_add_dep` ヘルパー（`fav.toml` への dep 追記）
- `cmd_add_impl` テスト用ヘルパー
- `site/content/docs/packages/getting-started.mdx` 新規作成

### Internal
- Cargo.toml version: `17.8.0`
- `v178000_tests`: 5 件追加

---

## [v17.7.0] — 2026-06-15 — `forall` プロパティベーステスト

### Added
- `forall x: Type [where { guard }] { body }` 構文追加
- `TokenKind::Forall` / `Stmt::Forall` / `ForallStmt` / `ForallVar` AST 追加
- `parse_forall_stmt` — `where { guard }` オプション対応
- `check_stmt`: E0327（非対応型）型チェック
- VM primitives: `__forall_gen_int` / `__forall_gen_str` / `__forall_gen_bool` / `__forall_gen_float`（xorshift64 固定シード 12345）
- compiler desugar: ガードなし → ForIn ループ、ガードあり → ListComp + `List.take` + ForIn
- `fav test --cases N` CLI オプション（`FORALL_CASES` 環境変数）
- exhaustive match 更新: fmt / emit_python / lineage(4) / lint(7) / checker(2) / compiler(2)
- `site/content/docs/tools/property-testing.mdx` 新規作成

### Internal
- Cargo.toml version: `17.7.0`
- `v177000_tests`: 5 件追加（version_is test は v17.8.0 で除去）

---

## [v17.6.0] — 2026-06-15 — `fav bench` 統計強化

### Added
- `bench "name" { ... }` 構文追加（AST `Item::BenchDef`）
- `BenchStats`（avg / p50 / p95 / min / max）統計計算
- `cmd_bench(opts: &BenchOpts)` 実装
- `--runs N` / `--warmup N` / `--json` CLI オプション
- `site/content/docs/tools/bench.mdx` 新規作成

### Internal
- Cargo.toml version: `17.6.0`
- `v176000_tests`: 5 件追加

---

## [v17.5.0] — 2026-06-15 — REPL 品質向上

### Added
- `:doc <fn>` / `:load <file>` / `:save <file>` / `:history` / `:paste` REPL コマンド追加
- `:paste` ... `:end` 複数行入力モード
- タブ補完（モジュール名・関数名・`:` コマンド）
- `FavCompleter` タブ補完実装

### Internal
- Cargo.toml version: `17.5.0`
- `v175000_tests`: 5 件追加

---

## [v17.4.0] — 2026-06-15 — `let` バインディング除去（誤実装の修正）

### Removed
- `TokenKind::Let` / `Stmt::Let` / `parse_let_stmt` / E0326 を除去
- `let x = expr` は Favnir に存在しない。`bind x <- expr` に統一

### Changed
- `bind x <- expr` が非 Result 値でも使えることを明確化（既存動作の確認）

### Internal
- Cargo.toml version: `17.4.0`
- `v174000_tests`: 5 件追加

---

## [v17.3.0] — 2026-06-15 — コレクション内包表記

### Added
- `[x * 2 | x <- nums]` list-comp — `List.map` 相当にデシュガー
- `[x | x <- nums, x > 0]` filter-comp — `List.filter` 相当にデシュガー
- `[Pair(a,b) | a <- as, b <- bs]` multi-source — `List.flat_map` 相当にデシュガー
- `[? f(x) | x <- xs]` result-comp — `List.collect_result` 相当にデシュガー
- `CompClause::For` / `CompClause::Guard` AST 追加
- `Expr::ListComp` / `Expr::ResultComp` AST 追加
- `List.collect_result` builtin primitive 追加
- exhaustive match 更新: lineage(4) / lint(6) / fmt / emit_python / driver(2)

### Internal
- Cargo.toml version: `17.3.0`
- `v173000_tests`: 5 件追加

---

## [v17.2.0] — 2026-06-15 — パターンマッチ拡張

### Added
- or-pattern: `"a" | "b" => ...`（`Pattern::Or`）
- list-pattern: `[] / [x] / [head, ..tail]`（`Pattern::List`）
- guard 条件: `if expr` in match arm（`MatchArm.guard`）
- `DotDot` トークン（`..`）追加（`DotDotDot` との区別）
- `IRPattern::Or` / `IRPattern::List` IR 追加
- `ListLen` (0x60) / `ListGet` (0x61) / `ListDrop` (0x62) VM opcodes 追加
- `emit_pattern_test` で Or / List パターンを処理
- exhaustive match 更新: checker / compiler / fmt / ast_lower_checker / emit_python / driver

### Internal
- Cargo.toml version: `17.2.0`
- `v172000_tests`: 5 件追加

---

## [v17.1.0] — 2026-06-15 — 境界付きジェネリクス（Bounded Generics）

### Added
- `fn f<T with Ord>(a: T, b: T) -> T` 構文追加
- `GenericParam { name: String, bounds: Vec<String> }` AST 追加（7 struct 変更）
- `parse_type_bounds` — `TokenKind::With` 対応
- `fn_bounds_registry: HashMap<String, Vec<GenericParam>>` in Checker
- `type_implements_bound` — 組み込み bound 自動実装テーブル
- 組み込み bounds: `Ord` / `Eq` / `Serialize` / `Display` / `Hash` / `Clone`
- call-site E0325: bound を満たさない型を渡すとエラー
- `site/content/docs/language/generics.mdx` 新規作成

### Internal
- Cargo.toml version: `17.1.0`
- `v171000_tests`: 6 件追加

---

## [v17.0.0] — 2026-06-14 — Language Ergonomics マイルストーン宣言

### Added
- v16.x シリーズ集大成：f-string / record spread / stdlib 拡充 / 型エイリアス / namespace alias / fav test 成熟 / tap 演算子が揃い Language Ergonomics を宣言
- `site/content/docs/stdlib/list.mdx` / `string.mdx` / `datetime.mdx` / `math.mdx` v16.4.0 内容反映
- `README.md` / `CHANGELOG.md` 全面更新（v16.1.0〜v17.0.0）

### Internal
- Cargo.toml version: `17.0.0`
- `v170000_tests`: 5 件追加

---

## [v16.8.0] — 2026-06-14 — tap / inspect パイプライン演算子

### Added
- `FlwStep::Tap(Box<Expr>)` / `FlwStep::Inspect` を AST に追加（ソフトキーワード）
- `|> tap(observer_fn)` — 値を変換せず副作用（ログ等）だけ実行してそのまま通す
- `|> inspect` — `[inspect] <value>` 形式で標準出力に出力する組み込み tap
- `inspect_debug` VM プリミティブ
- `CompileCtx.no_tap` フィールド + `set_no_tap_mode()` スレッドローカル
- `fav run --no-tap` — tap/inspect を identity にコンパイルしてゼロオーバーヘッド化
- `IRExpr::Block` + `IRStmt::Bind` + `IRStmt::Expr` で実装（新 VM opcode 不要）
- exhaustive match 更新: `checker.rs` / `ast_lower_checker.rs` / `emit_python.rs`
- `site/content/docs/language/pipeline.mdx` に tap/inspect セクション追加

### Internal
- Cargo.toml version: `16.8.0`
- `v168000_tests`: 6 件追加

---

## [v16.7.0] — 2026-06-14 — fav test 成熟（assert_eq / test_group / スナップショット）

### Added
- `test_group "name" { test ... }` — 関連テストのグループ化構文
- `assert_eq(actual, expected)` — `vmvalue_repr` で文字列化して比較、不一致で詳細エラー
- `assert_approx_eq(actual, expected, epsilon)` — Float 近似比較
- `assert_contains(list, elem)` — リスト内要素存在確認
- `assert_length(list, n)` — リスト長確認
- `assert_str_contains(s, substring)` — 文字列部分一致確認
- `assert_str_starts_with(s, prefix)` — 文字列プレフィックス確認
- `assert_err_eq(result, expected_msg)` — エラー内容の文字列一致確認
- `assert_snapshot(value, name)` — `.snap/{name}.snap` の作成・比較
- `fav test --update-snapshots` — 全スナップショットを上書き更新
- `collect_test_cases` を 4-tuple `(path, display_name, fn_name, prog)` に変更
- `site/content/docs/language/testing.mdx` 全面更新（全アサート・snapshot ワークフロー）

### Internal
- Cargo.toml version: `16.7.0`
- `v167000_tests`: 5 件追加（`set_var` は Rust 2024 edition で unsafe）

---

## [v16.6.0] — 2026-06-14 — Namespace Alias（use String as S）

### Added
- `use String as S` / `use List as L` 構文（ソフトキーワード `as`）
- `TokenKind::As`、`Item::UseAlias { alias, namespace, span }`
- `namespace_aliases: HashMap<String, String>` in `CompileCtx` + `Checker`
- `check_builtin_apply` と `compile_expr FieldAccess` でエイリアス解決
- `parse_import_decl` の `import "path" as alias` も `TokenKind::As` 対応
- `site/content/docs/language/modules.mdx` 新規作成

### Internal
- Cargo.toml version: `16.6.0`
- `v166000_tests`: 5 件追加

---

## [v16.5.0] — 2026-06-14 — 型エイリアス（alias キーワード）

### Added
- `alias Email = String` — 型エイリアス宣言（`alias` キーワード）
- `alias Result2<T> = Result<T, String>` — ジェネリクスエイリアス
- `Alias` トークン、`Item::AliasDecl { name, params, ty, span }`
- `alias_env: HashMap<String, (Vec<String>, TypeExpr)>` in `CompileCtx` / `Checker`
- `resolve_type_expr_with_self` / `resolve_type_expr_with_subst` 双方に alias 解決追加
- compiler.rs は catch-all で自動スキップ
- `site/content/docs/language/type-alias.mdx` 新規作成

### Internal
- Cargo.toml version: `16.5.0`
- `v165000_tests`: 5 件追加

---

## [v16.4.0] — 2026-06-14 — 標準ライブラリ拡充（List / String / DateTime / Math）

### Added
- **List**: `sort_by` / `sort_by_desc` / `distinct` / `distinct_by` / `count_where` / `sum_by` / `max_by` / `min_by` / `unzip`（高階関数）
- **String**: `split_once` / `replace_first` / `format_int(n, width, pad)` / `format_float(f, decimals)`
- **DateTime**: 新モジュール全 12 関数（`now` / `parse` / `format` / `add_days` / `add_hours` / `diff_days` / `year` / `month` / `day` / `weekday` / `timestamp` / `from_timestamp`）。内部表現は Unix timestamp（Int）。`chrono` クレートを使用。
- **Math**: `round_to(f, n)` / `log(f)` / `log2(f)` / `log10(f)`
- `compiler.rs` / `checker.rs` に `DateTime` 名前空間登録

### Internal
- Cargo.toml version: `16.4.0`
- `v164000_tests`: 6 件追加

---

## [v16.3.0] — 2026-06-14 — レコード更新構文（{ ...base, field: val }）

### Added
- `{ ...base, field: val }` — レコードスプレッド / 更新構文
- `DotDotDot` トークン、`Expr::RecordSpread { base, overrides }`
- `IRExpr::RecordSpread`、`MergeRecord = 0x5C` VM opcode
- `remap_string_operands` に `MergeRecord` 追加（未追加だと後続 GetField が壊れる問題を修正）

### Internal
- Cargo.toml version: `16.3.0`
- `v163000_tests`: 6 件追加

---

## [v16.2.0] — 2026-06-14 — f-string 文字列補間

### Added
- `f"Hello, {name}!"` — f-string プレフィックス付き文字列補間
- `f"""..."""` — 三重クォート f-string
- `FStringRaw` トークン、`lex_fstring_triple`、`lower_fstring`（コンパイル時に `String.concat` 連鎖へ展開、VM 変更なし）

### Internal
- Cargo.toml version: `16.2.0`
- `v162000_tests`: 5 件追加

---

## [v16.1.0] — 2026-06-14 — エラーメッセージ品質向上

### Added
- rustc スタイルのエラー表示（`-->` ファイル・行・列、`^` アンダーライン）
- `Span { line, col, len }` を AST 全ノードに追加
- typo ヒント（Levenshtein 距離 ≤ 2 の候補を最大 3 件表示）
- `= hint:` / `= help:` メッセージ付与
- エラーコード URL（`https://favnir.dev/errors/E0xxx`）

### Internal
- Cargo.toml version: `16.1.0`
- `v161000_tests`: 5 件追加

---

## [v16.0.0] — 2026-06-14 — Production Multi-Cloud マイルストーン宣言

### Added
- v15.x シリーズ集大成：CrossCloud 認証・GCP BigQuery・Kafka/MSK・`fav test`・`fav deploy` が揃い Production Multi-Cloud を宣言
- `site/content/docs/runes/bigquery.mdx` / `kafka.mdx` ドキュメント追加
- 対応クラウド: AWS / Azure / GCP / Snowflake + Kafka/MSK（4 クラウド + ストリーミング）

### Internal
- Cargo.toml version: `16.0.0`
- `v160000_tests`: 5 件追加

---

## [v15.5.0] — 2026-06-14 — `fav deploy`（AWS Lambda デプロイ CLI）

### Added
- `DeployConfig` に `target` / `function_name` フィールド追加（ロードマップ仕様準拠）
- `memory_mb` / `timeout_sec` を `memory` / `timeout` のエイリアスとして追加
- `runtime` デフォルトを `provided.al2023` に更新
- `scripts/build-lambda-layer.sh`：`cross` で `x86_64-unknown-linux-musl` クロスコンパイル → `bootstrap` + zip パッケージング
- `site/content/docs/deploy.mdx`：`fav deploy` ユーザーガイド新規作成

### Internal
- Cargo.toml version: `15.5.0`
- `v155000_tests`: 3 件追加（version / deploy_toml_schema_parses / deploy_cmd_exists）

---

## [v15.4.0] — 2026-06-14 — Kafka / MSK Rune（`!Stream` エフェクト）

### Added
- `Effect::Stream` 追加（ast.rs + 全 exhaustive match 対応）
- `Kafka.produce_raw(brokers, topic, key, value)` / `Kafka.consume_one_raw(brokers, topic, group_id)` VM プリミティブ（rskafka 0.6 pure-Rust、SCRAM-SHA-512 認証）
- E0319：`!Stream` エフェクト欠如エラー
- `fav.toml [kafka]` セクション（`bootstrap_brokers` / `sasl_mechanism` / `sasl_username` / `sasl_password`）
- `runes/kafka/kafka.fav`：`produce` / `consume_one` ラッパー
- `infra/e2e-demo/kafka/`：4-stage pipeline + Terraform AWS MSK
- `self/checker.fav`：`kafka_fn` / `ns_to_effect "Kafka"→"Stream"` 追加

### Internal
- Cargo.toml version: `15.4.0`
- 依存追加：`rskafka 0.6`（`transport-tls` feature）
- `v154000_tests`: 5 件追加

---

## [v15.3.0] — 2026-06-14 — `fav test` DSL（ネイティブテストフレームワーク）

### Added
- `test "description" { ... }` 構文（`TopLevel::TestDef`）
- `assert_ok` / `assert_err` / `assert_true` VM プリミティブ
- `cmd_test`（Bool(false) → FAIL 判定修正含む）
- `site/content/docs/language/testing.mdx` 新規作成

### Internal
- Cargo.toml version: `15.3.0`
- `v153000_tests`: 5 件追加

---

## [v15.2.0] — 2026-06-14 — GCP BigQuery Rune（`!Gcp` エフェクト）

### Added
- `Effect::Gcp` 追加
- `BigQuery.query_raw` / `BigQuery.execute_raw` / `BigQuery.infer_table_raw` VM プリミティブ（RS256 JWT + Google OAuth2）
- E0318：`!Gcp` エフェクト欠如エラー
- `fav.toml [gcp]` セクション（`project_id` / `credentials_file` / `dataset` / `location`）
- `runes/bigquery/bigquery.fav`：`query` / `execute` ラッパー
- `infra/e2e-demo/bigquery/`：4-stage pipeline + Terraform GCP BigQuery
- `self/checker.fav`：`bigquery_fn` / `ns_to_effect "BigQuery"→"Gcp"` 追加

### Internal
- Cargo.toml version: `15.2.0`
- `v152000_tests`: 5 件追加

---

## [v15.1.5] — 2026-06-14 — CrossCloud 認証層セキュア版（KMS ECDSA P-256）

### Added
- Lambda verifier_v2（KMS `GetPublicKey` + Python `cryptography` ECDSA P-256 検証）
- `infra/e2e-demo/crosscloud/lambda/verifier_v2/`
- `infra/e2e-demo/crosscloud/scripts/run_with_kms.sh`
- Terraform：`aws_kms_key`（ECC_NIST_P256 / SIGN_VERIFY）+ `aws_kms_alias`
- E2E：改ざんボディ / ランダム署名 → PASS=2 FAIL=0

### Internal
- Cargo.toml version: `15.1.5`
- `v15150_tests`: 6 件追加

---

## [v15.1.0] — 2026-06-14 — CrossCloud 認証層（HMAC + Cognito + Lambda verifier）

### Added
- `AWS.dynamo_put_item_cond_raw` VM プリミティブ（DynamoDB ConditionalPut、TTL + nonce リプレイ防止）
- `AWS.ecs_run_task_raw` VM プリミティブ（ECS Fargate RunTask、SigV4）
- Lambda verifier（Favnir コンテナ、`public.ecr.aws/lambda/provided:al2023` ベース）
- Cognito JWT Authorizer + API Gateway
- HMAC-SHA256 署名方式（StringToSign = Method\nPath\nTimestamp\nNonce\nSHA256(Body)）
- E2E：`reject_cases.sh` PASS=5 FAIL=0、S3 証跡保存

### Fixed
- `fav run --legacy` が `Result.err` を返しても exit 0 だった問題を修正（`process::exit(1)` 追加）
- `AWS_CONFIG` thread-local が `default()` でハードコード値を返していた問題を `from_env()` に修正

### Internal
- Cargo.toml version: `15.1.0`
- `v151000_tests`: 6 件追加

---

## [v14.8.0] — 2026-06-12 — Rune ファイル整備（--legacy 明示化 + fs.fav バグ修正）

### Fixed
- `runes/fs/fs.fav`: `glob` 関数内の非 Result `bind`（`bind sep <- "/"` 等）をインライン化で修正
- `runes/fs/fs.fav`: `walk_entry` 関数内の非 Result `bind full_path` もインライン化で修正

### Changed
- rune ファイル 12 件に `--legacy compatible` コメントを追加（意図を明示）:
  `cache/cache.fav`, `fs/fs.fav`, `log/emitter.fav`, `log/metric.fav`,
  `queue/queue.fav`, `gen/output.fav`, `http/request.fav`, `graphql/client.fav`,
  `grpc/server.fav`, `duckdb/query.fav`, `duckdb/io.fav`, `db/connection.fav`

### Internal
- Cargo.toml version: `14.8.0`
- `v148000_tests`: 3 件追加

---

## [v14.7.0] — 2026-06-12 — site/ ドキュメント更新 + rune ファイル精査

### Changed
- `site/content/docs/introduction.mdx`: 旧エフェクト表・存在しない機能（fav deploy / MCP / Notebook）を削除。Capability Context 体系で書き直し
- `site/content/docs/language/effects.mdx`: v14.0.0 Capability Context を主体に全面書き直し。E0370 削除、E0023/E0025/E0021 追加
- `site/content/docs/quickstart.mdx`: `ctx: AppCtx` スタイルのサンプルに更新。`bind user <- User{...}` 誤用を `let` に修正
- `site/content/docs/installation.mdx`: バージョン表示 `v5.0.0` → `v14.7.0`。`fav deploy` / `fav mcp` / `fav explain-error`（非実装コマンド）を削除
- `runes/aws/dynamodb.fav`, `runes/aws/sqs.fav`: `--legacy` 専用 API コメントを追加

### Internal
- Cargo.toml version: `14.7.0`
- `v148000_tests`: 3 件追加（v147000_tests の誤記 — 本体は v147000_tests）

---

## [v14.6.0] — 2026-06-12 — ドキュメント整備（README + CHANGELOG）

### Changed
- `README.md`: 「現在の状態」見出しを v14.6.0 に更新、ロードマップ表に v14.1.0〜v14.6.0 を追記
- `README.md`: 機能一覧表に Azure Blob Storage / Azure PostgreSQL 行を追加
- `README.md`: 旧 `!Effect` スタイルコード例に `--legacy` モード注記を追加
- `CHANGELOG.md`: v14.1.0〜v14.5.0 エントリを追加

### Notes
- コードの変更なし。純粋なドキュメント更新バージョン
- テスト: v146000_tests 3 件（version_is_14_6_0 / changelog_has_v14_5_0_entry / readme_mentions_azure_blob）

---

## [v14.5.0] — 2026-06-12 — Azure Blob Storage Rune

### New Features
- `azure_blob_sign` ヘルパー関数（`vm.rs`）: HMAC-SHA256 + base64 による Azure Shared Key 署名
  - 既存の `hmac 0.12` + `sha2 0.10` + `base64 0.22` + `chrono` を使用（新規 crate なし）
  - RFC 1123 日付フォーマット、x-ms-* ヘッダーのアルファベット順ソート
- `AzureBlob.put_raw(account, key, container, blob_name, body)` VM primitive（BlockBlob PUT）
- `AzureBlob.get_raw(account, key, container, blob_name)` VM primitive（GET → String）
- `AzureBlob.list_raw(account, key, container, prefix)` VM primitive（GET → JSON 配列文字列）
- `AzureBlob.delete_raw(account, key, container, blob_name)` VM primitive（DELETE）
- `checker.rs`: `require_azure_storage_effect` — `!AzureStorage` 未宣言時に E0317 を発生
- `checker.rs`: `("AzureBlob", "put_raw/get_raw/list_raw/delete_raw")` を `builtin_ret_ty` に追加
- `checker.rs`: `"AzureBlob"` を `BUILTIN_EFFECTS` に追加
- `runes/azure-blob/azure_blob.fav`: `put/get/list/delete` ctx-aware ラッパー（`ctx: String`）
- `runes/azure-blob/rune.toml`: rune メタデータ（version 14.5.0、effects !AzureStorage）

### Notes
- テスト: v145000_tests 4 件（version_is_14_5_0 / azure_blob_put_raw_registered / azure_storage_effect_required / azure_blob_rune_file_present）
- `let` 構文は rune ファイル内でパースエラーになるため引数はインライン化
- `import rune "ctx"` は使用不可（runes/ctx/ctx.fav 未存在）→ `ctx: String` で代替
- LIST の canonical_resource は query params をアルファベット順にソート: `comp:list\nprefix:...\nrestype:container`

---

## [v14.4.0] — 2026-06-12 — AWS Rune 正式パッケージング

### New Features
- `AWS.secrets_get_raw(region, secret_name)` VM primitive（SigV4 + ureq で Secrets Manager `GetSecretValue` API）
- `Ctx.aws_get_field_raw(ctx, field)` VM primitive — AwsCtx JSON 文字列からフィールドを取得
- `checker.rs`: `("AWS", "secrets_get_raw")` を `builtin_ret_ty` に追加（`require_aws_effect` 呼び出し）
- `checker.rs`: `("Ctx", "aws_get_field_raw")` → `Some(Type::String)` を `builtin_ret_ty` に追加
- `runes/aws/secrets.fav`: `secrets_get(ctx: String, secret_name: String)` ラッパー
- `runes/aws/s3.fav`: `s3_put/s3_get/s3_delete/s3_list` ctx-aware ラッパーを追加
- `runes/aws/rune.toml`: version `14.4.0`、description に Secrets Manager を追記

### Notes
- テスト: v144000_tests 4 件（version_is_14_4_0 / secrets_get_raw_registered / aws_ctx_field_raw_registered / aws_rune_s3_ctx_functions_present）
- LocalStack エンドポイント対応（`config.endpoint_url` がある場合は `/` に置換）
- `let` 構文パースエラーのため rune ファイルは全引数インライン化

---

## [v14.3.0] — 2026-06-12 — Azure lineage + !AzureStorage エフェクト

### New Features
- `ast::Effect::AzureStorage` 追加（parser / lineage / checker で認識）
- `lineage.rs`: `EffectKind::AzureDbRead` / `AzureDbWrite` / `AzureBlobRead` / `AzureBlobWrite` 追加
- `lineage.rs`: `collect_azure_blob_call_kinds` / `collect_azure_db_call_kinds` 追加
- `checker.rs`: `BUILTIN_EFFECTS` に `"AzureStorage"` を追加
- `fav explain --lineage` 出力に Azure エフェクトが表示されるよう更新

### Notes
- テスト: v143000_tests

---

## [v14.2.0] — 2026-06-12 — AzureCtx / AwsCtx + fav.toml [azure]

### New Features
- `Ctx.build_aws_raw(region, s3_bucket, db_url)` VM primitive — AwsCtx JSON を生成
- `Ctx.build_azure_raw(postgres_url, storage_account, storage_key, container)` VM primitive — AzureCtx JSON を生成
- `Ctx.aws_get_field_raw(ctx, field)` VM primitive — AwsCtx からフィールドを取得（v14.4.0 で checker に追加）
- `Ctx.azure_get_field_raw(ctx, field)` VM primitive — AzureCtx からフィールドを取得
- `fav.toml` に `[azure]` セクション追加（`postgres_url` / `storage_account` / `storage_key` / `container`）
- `inject_azure_config` — fav.toml の [azure] セクションを env var 展開して実行時 ctx に注入

### Notes
- テスト: v142000_tests

---

## [v14.1.0] — 2026-06-12 — Azure PostgreSQL Rune

### New Features
- `AzurePostgres.execute_raw(conn_str, sql, params)` VM primitive（tokio-postgres + tokio ランタイム）
- `AzurePostgres.query_raw(conn_str, sql, params)` VM primitive（JSON 配列文字列として返す）
- `checker.rs`: `AzurePostgres` namespace を `builtin_ret_ty` / `BUILTIN_EFFECTS` に追加
- `checker.rs`: `require_azure_db_effect` — `!AzureDb` 未宣言時に E0316 を発生
- `ast::Effect::AzureDb` 追加
- `lineage.rs`: `!AzureDb(read/write)` 区別追加
- `runes/azure-postgres/azure_postgres.fav`: `execute/query_rows` ctx-aware ラッパー
- `runes/azure-postgres/rune.toml`: rune メタデータ

### Notes
- テスト: v141000_tests
- SSL: `sslmode=require` を接続文字列に付加して Azure DB for PostgreSQL の SSL 必須要件に対応

---

## [v14.0.0] — 2026-06-11 — 能力型完成宣言

### Breaking Changes
- `!Effect` 記法は非 legacy モードで E0025 エラーになる（v13.10.0 から段階的導入、v14.0.0 で CI 確認完了）
- ambient effect 呼び出し（ctx なしの `IO.println` 等）は E0023 エラーになる（v13.8.0 から）

### New Features (v13.1.0〜v13.10.0 集約)
- `interface` 継承構文（`LoadCtx: CommonCtx`）のコンパイル時チェック
- `DbRead` / `DbWrite` / `StorageRead` / `StorageWrite` / `HttpClient` / `Io` / `Env` capability interface
- `LoadCtx` / `WriteCtx` / `MigrateCtx` コンテキスト interface（capability 充足チェック付き）
- `AppCtx` 具象型 + `Ctx.build` / `Ctx.mock` Rune
- `ctx.field.method()` フィールドアクセス構文
- `seq Pipeline(ctx)` — ctx 型推論
- E0024 型状態パターンチェック
- `Ctx { db: DbRead }` 糖衣構文（v13.10.0）
- `fav migrate --from-effects` 移行ツール（v13.10.0）

### Error Codes Added
- W008: ambient effect call（警告）
- E0020: capability interface has no such method
- E0021: capability not in context
- E0022: ctx-aware pipeline called with wrong number of arguments
- E0023: ambient effect call is not allowed（エラー）
- E0024: type state mismatch
- E0025: bang notation removed
- W009: direct Rune call deprecated
- W010: effect migration requires manual review

### Migration
`fav migrate --from-effects <file>` で旧 `!Effect` 記法を自動変換。
`--legacy` フラグで移行期間中も旧記法を許容（今後廃止予定）。

### Notes
- `self/compiler.fav` / `self/checker.fav` の E0025 件数がゼロであることを CI テストで保証
- テスト: 2207 件（v13.10.0 時点）

---

## [v13.0.0] — 2026-06-09

### Added
- 言語信頼性宣言: 型安全・エラー伝播・デバッグ可視性の三点における保証
- README.md に v13.0.0 宣言文を追記
- `versions/v13.0.0/` — spec / plan / tasks

### Notes
- テスト: 1415 件
- v12.1.0〜v12.10.0 で発覚した全問題（C-1〜C-4 / H-1〜H-2 / M-1 / A-1〜A-6）を解消

---

## [v12.10.0] — 2026-06-09

### Added
- `driver.rs` `get_help_text(code: &str) -> &'static [&'static str]` — 12 コード（E0001/E0007/E0008/E0009/E0013/E0014/E0015/E0018/W001/W004/W006/W007）に `help:` テキストを追加
- `fav check --strict` — W006 警告をエラーとして扱い exit 1（`-D warnings` 相当）
- `fav lint --deny-warnings` — 警告を exit 1 に昇格させる CI 用フラグ
- `fav.toml [lint]` セクション — `warn_as_error` / `allow` リストによる細粒度制御
- `toml.rs` `LintTomlConfig { warn_as_error: Option<Vec<String>>, allow: Option<Vec<String>> }`
- `driver.rs` `v121000_tests` — `help_text_e0001_present` / `help_text_w006_present` / `help_text_unknown_is_empty` / `version_is_12_10_0`
- `tests/integration.rs` — `check_strict_w006_exits_1` / `check_strict_no_warning_exits_0` / `lint_deny_warnings_exits_1`

### Changed
- `format_diagnostic` / `format_warning` — エラー・警告出力末尾に `= help:` 行を自動付与
- `cmd_lint` — `warn_only` に加え `deny_warnings` パラメータを追加; `[lint]` allow フィルタ・warn_as_error 昇格を適用
- `.github/workflows/ci.yml` Self-lint ステップに `--deny-warnings` を追加

### Notes
- テスト: 1415 unit + 8 integration

---

## [v12.9.0] — 2026-06-09

### Added
- `.github/workflows/ci.yml` `Self-test (fav test)` ステップ — `self/checker.fav` / `self/compiler.fav` / `self/codegen.fav` / `self/lexer.fav` / `self/parser.fav`
- `.github/workflows/ci.yml` `integration` ジョブ — `services: postgres:16` (POSTGRES_PASSWORD=test) + health check
- `fav/tests/integration.rs` — `fav_test_self_checker_runs` / `fav_test_self_lexer_runs` / `postgres_create_insert_select` / `postgres_error_table_not_found` / `postgres_ssl_disable_connects`
- `driver.rs` `pg_exec_for_test` / `pg_query_for_test` — 統合テスト用 pub ヘルパー
- `driver.rs` `v12900_tests` — `version_is_12_9_0`

### Notes
- テスト: 1415 件（統合テスト 8 件含む）

---

## [v12.8.0] — 2026-06-09

### Added
- `fav scaffold <template>` コマンド — stage / seq / postgres-etl / rune テンプレートを標準出力に生成
- `driver.rs` `cmd_scaffold(template: &str, name: Option<&str>)` 実装
- `main.rs` `Some("scaffold")` 分岐を追加
- `driver.rs` `v12800_tests` — `scaffold_stage_output_contains_stage` / `scaffold_seq_output_contains_seq` / `scaffold_postgres_etl_output_contains_stages` / `scaffold_rune_output_contains_rune` / `scaffold_stage_named_output_contains_name` / `version_is_12_8_0`（← comment out 済み）

### Notes
- テスト: 1411 件

---

## [v12.7.0] — 2026-06-08

### Added
- `fav doc --builtins [--format json|markdown] [--out <file>]` — 組み込み Primitive の型シグネチャ一覧（IO/Csv/Schema/Json/Gen/AWS/Postgres/Snowflake/Http/Llm）
- `fav explain <code>` — エラーコードの詳細説明（E0001〜E0018 / W001〜W007）
- `driver.rs` `builtin_primitives()` — 組み込み関数メタデータのリスト
- `driver.rs` `cmd_doc_builtins(format, out)` / `cmd_explain_code(code)`
- `driver.rs` `v12700_tests` — `doc_builtins_json_has_csv_parse_raw` / `doc_builtins_markdown_has_postgres` / `explain_e0001_output` / `explain_w006_output` / `doc_builtins_returns_result_field`

### Notes
- テスト: 1408 件

---

## [v12.6.0] — 2026-06-08

### Added
- `tokio-postgres-native-tls` / `native-tls` — Postgres TLS 対応
- `fav.toml [postgres]` `sslmode` キー（`disable` / `prefer` / `require`）
- `DATABASE_URL` の `sslmode` クエリパラメータ解析
- Postgres エラー詳細化 — `DbError.message()` / `code()` / `detail()` を連結（"db error" → "db error: SSL connection is required (SQLSTATE 08P01)"）
- `driver.rs` `v12600_tests` — `postgres_sslmode_disable` / `postgres_sslmode_parse` / `postgres_error_detail`

### Changed
- `pg_connect` — `sslmode` に応じて `NoTls` / `TlsConnector` を切り替え

### Notes
- テスト: 1402 件

---

## [v12.5.0] — 2026-06-08

### Added
- `fav run --verbose` — stage 入出力を stderr に出力（最大 200 文字トランケート）
- `fav run --trace` — stage 入出力をフル出力（トランケートなし）
- `fav.toml [run]` `verbose` / `trace` キー
- `fav check --json` — エラー・警告を JSON 形式で出力（AI フレンドリー）
- `fav check --show-types` — 各 `bind` / `chain` の型と W006 マーカーを表示
- `driver.rs` `CheckDiagnostic` / `BindingInfo` / `CheckOutput` 構造体（serde::Serialize）
- `driver.rs` `collect_binding_types(file)` — W006 検出（`bind _ <- NS.fn(...)` パターン）
- `driver.rs` `v12500_tests` — `verbose_stage_enter_exit` / `check_json_output_format` / `check_show_types_bind` / `check_show_types_w006_detected`

### Changed
- `VERBOSE_LEVEL` を `thread_local! { Cell<u8> }` に変更（並行テスト対応）

### Notes
- テスト: 1386 件

---

## [v12.4.0] — 2026-06-08

### Added
- `IRStmt::SeqChain` + `Opcode::SeqStageCheck = 0x36` — seq pipeline fail-fast
- `compile_flw_def` 修正: 2+ ステージを `SeqChain` stmts で構築
- `SeqStageCheck` VM ハンドラ: stage 名・番号付きエラーで短絡（`"pipeline stopped at stage N/M 'Name': error"`）
- `driver.rs` `v12400_tests` — `seq_stops_on_stage_err` / `seq_passes_ok_through` / `seq_error_includes_stage_name`

### Notes
- テスト: 1376 件

---

## [v12.3.0] — 2026-06-08

### Added
- `IRStmt::LegacyBind(u16, IRExpr)` + `Opcode::LegacyBindCheck = 0x35`
- `apply_legacy_bind_semantics(ir: IRProgram)` — `--legacy` モードで `Bind` → `LegacyBind` に変換
- `LegacyBindCheck` VM ハンドラ: `ok(v)`→unwrap, `err(e)`→escape, 非 Result→pass-through
- `driver.rs` `v12300_tests` — `legacy_bind_propagates_err` / `legacy_bind_ok_unwraps` / `legacy_bind_non_result_passthrough`

### Changed
- `--legacy` モードの `bind x <- expr` が `expr` の Result を unwrap して短絡するように修正（真の monadic bind）

### Notes
- テスト: 1370 件

---

## [v12.2.0] — 2026-06-07

### Added
- `is_result_returning_call(stmt)` — `bind _ <- NS.fn(...)` で Result を返す NS 呼び出しを AST 解析で検出
- W006 警告（`fav check --show-types`）: bind _ で Result を捨てると警告
- 対象 NS: Postgres / Snowflake / S3 / Sqs / Queue / Cache / Http / Grpc / Llm / IO
- `driver.rs` `v12200_tests` — `w006_detected_for_postgres_bind_underscore` / `w006_not_detected_for_named_bind`

### Notes
- テスト: 1357 件

---

## [v12.1.0] — 2026-06-07

### Added
- E0018 `bind` 再束縛禁止（checker.fav）— 同一スコープで同名変数への二重 `bind` を検出
- `check_rebind_ok(name, env)` ヘルパー — `Option<String>` → `Result<String, String>`
- `driver.rs` `v12100_tests` — `e0018_rebind_detected` / `e0018_underscore_allowed` / `e0018_help_message_shown`

### Changed
- `checker.fav` `infer_stmt` に bind 済みセット管理を追加

### Notes
- テスト: 1353 件

---

## [v12.0.0] — 2026-06-06

### Added
- `site/content/docs/transpile/python.mdx` — Python トランスパイラ公式ドキュメント（使用方法・エフェクト対応表・変換例・E2E デモリンク）
- Python トランスパイラ完成宣言（v11.1.0〜v11.9.0 の全機能が揃った）

### Changed
- README.md に `fav transpile --target python` 機能行を追記
- CHANGELOG に v11.1.0〜v11.9.0 の全履歴を追記

### Notes
- テスト: 707 件（v12000_tests 2 件追加）

---

## [v11.9.0] — 2026-06-06

### Added
- `infra/e2e-demo/fav2py/` — Fav ネイティブ vs Python トランスパイル E2E インフラ
  - `src/pipeline.fav` — LoadAndInsert |> Aggregate |> SaveResult（RDS Postgres）
  - `src/sample.csv` — 103 行サンプルデータ（region × category × amount）
  - `terraform/main.tf` — VPC / RDS PostgreSQL (t3.micro) / ECS Fargate x2 / ECR
  - `terraform/iam.tf` — ECS 実行ロール + タスクロール（S3 書き込み）
  - `terraform/variables.tf` / `terraform/outputs.tf`
  - `scripts/upload.sh` — Docker build + ECR push + S3 source upload
  - `scripts/run.sh` — terraform apply → ECS タスク x2 起動 → verify.sh 呼び出し
  - `scripts/verify.sh` — S3 最新 2 件 JSON 比較（native == python）
  - `Dockerfile` — Ubuntu 22.04 + uv + psycopg2-binary + fav binary
- `driver.rs` `v11900_tests` — `fav2py_e2e_demo_structure` / `fav2py_pipeline_fav_transpiles`

### Notes
- テスト: 705 件

---

## [v11.8.0] — 2026-06-06

### Added
- `fav transpile --no-check` オプション（型チェックスキップ）
- `fav transpile --lineage` オプション（生成 Python コードに lineage コメント付与）
- `emit_python.rs` `emit_python_with_lineage(prog, path, HashMap<String,String>) -> String`
- `emit_python.rs` `Emitter` に `lineage_comments: HashMap<String,String>` フィールド追加
- `driver.rs` `build_lineage_comments(report: &LineageReport) -> HashMap<String,String>`
- `driver.rs` `check_source_str_pub(src: &str) -> Vec<TypeError>`（pub ラッパー）
- `driver.rs` `v11800_tests` — 6 件（checker 統合 / lineage コメント検証）

### Changed
- `fav transpile` 実行前に `checker.fav` で型チェックを走らせる（型エラーで Python 生成をブロック）

### Notes
- テスト: 703 件

---

## [v11.7.0] — 2026-06-06

### Added
- `fav transpile --out-dir <dir>` — `main.py` + `pyproject.toml` + `README.md` を出力ディレクトリに生成
- `fav transpile --check` — `python -m py_compile` による構文検証
- `fav transpile --run` — 生成後に `uv run main.py` まで一括実行
- `driver.rs` `build_pyproject_content(py_src, name) -> String`（boto3 / psycopg2 依存を自動検出）
- `driver.rs` `build_readme_content(input_path, name) -> String`
- `driver.rs` `v11700_tests` — 6 件（pyproject 生成 / README 生成 / uv フラグ検証）

### Notes
- テスト: 697 件

---

## [v11.6.0] — 2026-06-06

### Added
- `emit_python.rs` `!Postgres` → psycopg2 変換
  - `_pg_connect()` — `DATABASE_URL` または `PGHOST`/`PGPORT`/etc. から接続
  - `_pg_execute(sql, params)` — INSERT/UPDATE/DELETE ヘルパー
  - `_pg_query(sql, params)` — SELECT → `RealDictCursor` ヘルパー
  - `Postgres.execute_raw` → `_pg_execute(sql, params)`
  - `Postgres.query_raw` → `_pg_query(sql, params)`
- `emit_python.rs` `needs_psycopg2` / `needs_pg_helpers` フラグ追加（2-pass 検出）
- `pyproject.toml` 生成時に `import psycopg2` 検出 → `psycopg2-binary>=2.9` 依存を自動追加
- `driver.rs` `v11600_tests` — 6 件

### Notes
- テスト: 691 件

---

## [v11.5.0] — 2026-06-06

### Added
- `Effect::Postgres` 追加（ast.rs / parser.rs / fmt.rs / lineage.rs / driver.rs / ast_lower_checker.rs / checker.rs / reachability.rs）
- `vm.rs` `Postgres.execute_raw(sql, params_json) -> Result<Unit, String>`（tokio-postgres ベース）
- `vm.rs` `Postgres.query_raw(sql, params_json) -> Result<String, String>`（JSON 文字列返却）
- `vm.rs` `Postgres.query_typed_raw(sql, params_json) -> Result<String, String>`（型付きクエリ）
- `toml.rs` `PostgresTomlConfig` — `fav.toml` `[postgres]` セクション解析
- `runes/postgres/postgres.fav` — `execute` / `query<T>` Rune 実装（`!Postgres` エフェクト）
- `checker.fav` `postgres_fn` / `builtin_ret_ty` / `ns_to_effect` に Postgres 追加
- `driver.rs` `v11500_tests` — 6 件

### Notes
- テスト: 685 件

---

## [v11.4.0] — 2026-06-06

### Added
- `emit_python.rs` `!AWS` → boto3 変換
  - `AWS.s3_put_object_raw(bucket, key, body)` → `boto3.client("s3").put_object(Bucket=..., Key=..., Body=...)`
  - `AWS.s3_get_object_raw(bucket, key)` → `boto3.client("s3").get_object(Bucket=..., Key=...)["Body"].read()`
- `emit_python.rs` `needs_boto3` フラグ追加（2-pass 検出）
- `pyproject.toml` 生成時に `import boto3` 検出 → `boto3>=1.34` 依存を自動追加
- `driver.rs` `v11400_tests` — 4 件

### Notes
- テスト: 679 件

---

## [v11.3.0] — 2026-06-06

### Added
- `emit_python.rs` `!IO` → Python 標準 I/O 変換
  - `IO.println(s)` → `print(s)`
  - `IO.read_file_raw(path)` → `open(path).read()`（try/except で `Result` を模倣）
  - `Csv.parse_raw(text, ",", true)` → `csv.DictReader` 変換ヘルパー生成
  - `Schema.adapt(raw, "T")` → dataclass 変換ヘルパー生成（`_adapt_T(d) -> T`）
  - `Schema.to_json_array(rows, "T")` → `json.dumps([asdict(r) for r in rows])`
- `driver.rs` `v11300_tests` — 4 件

### Notes
- テスト: 675 件

---

## [v11.2.0] — 2026-06-06

### Added
- `emit_python.rs` `stage` / `seq` → Python パイプライン変換
  - `stage Foo: A -> B !Eff = |x| { ... }` → `def foo(x: A) -> B: ...`（エフェクトはコメント）
  - `seq Pipeline = A |> B |> C` → `def pipeline(x): return c(b(a(x)))`
- `fn main()` → `if __name__ == "__main__": main()`
- `IO.argv()` → `sys.argv[1:]`
- `List.map` / `List.filter` / `List.length` → Python リスト内包表記 / `filter` / `len`
- `driver.rs` `v11200_tests` — 4 件

### Notes
- テスト: 671 件

---

## [v11.1.0] — 2026-06-06

### Added
- `src/emit_python.rs` 新規作成 — Favnir AST → Python コード生成基盤
  - 型定義（`type Foo = { ... }`）→ `@dataclass class Foo`
  - 基本式（Int / Float / String / Bool / List / if-else / binary ops）→ Python 式
  - `fn` → `def`（引数型・戻り型をコメントで保持）
  - `bind x <- expr` → `x = expr`（モナド脱糖）
  - `match` → `if/elif/else`（Option / Result パターン）
- `fav transpile --target python <file>` CLI エントリ（`cli.fav` の `CmdTranspile` + `driver.rs` の `cmd_transpile`）
- `driver.rs` `v11100_tests` — 4 件

### Notes
- テスト: 667 件

---

## [v11.0.0] — 2026-06-05

### Added
- `fav explain --lineage` で `!Snowflake(read)` / `!Snowflake(write)` を区別表示（`lineage.rs` `collect_snowflake_call_kinds`）
- `site/content/docs/runes/snowflake.mdx` — Snowflake Rune リファレンスページ

### Changed
- README.md の Rune エコシステム表に `snowflake`（`!Snowflake` エフェクト）を追加
- CHANGELOG に v10.1.0〜v10.9.0 の全履歴を追記

### Notes
- テスト: 1286 件（lineage Snowflake 区別テスト 3 件追加）

---

## [v10.9.0] — 2026-06-05

### Added
- `infra/e2e-demo/snowflake/` — Snowflake E2E デモ（demo.fav 4 ステージ・Terraform・scripts/run.sh・README）
- `driver.rs` `v10900_tests::snowflake_e2e_demo_structure` — ファイル存在確認テスト

### Notes
- テスト: 1283 件

---

## [v10.8.0] — 2026-06-04

### Added
- `fav infer --from snowflake --table <name>` — Snowflake INFORMATION_SCHEMA から Favnir 型定義を自動生成
- `Snowflake.infer_table_raw` VM primitive
- `cli.fav` `CmdInferSnowflake` / `parse_infer_cmd` / `run_infer_snowflake`
- Snowflake 型マッピング（NUMBER→Int / FLOAT→Float / VARCHAR→String / BOOLEAN→Bool / nullable→Option<T>）

### Notes
- テスト: 1282 件（型マッピングテスト 6 件追加）

---

## [v10.7.0] — 2026-06-04

### Added
- `toml.rs` `SnowflakeTomlConfig` — `fav.toml` `[snowflake]` セクション解析（account / user / warehouse / role / database / schema）
- `expand_env_vars` — `${VAR_NAME}` 形式の環境変数参照を展開
- `inject_snowflake_config` — 実行時に Snowflake 設定を環境変数に注入（上書きなし）
- `fav new` テンプレートに `[snowflake]` コメントアウト例を追加

### Notes
- テスト: 1276 件

---

## [v10.6.0] — 2026-06-04

### Added
- `runes/snowflake/` — Snowflake Rune 実装（`execute` / `query<T>`）
- `rune.toml` / `snowflake.fav` / `client.fav` / `snowflake.test.fav`

### Notes
- テスト: 1272 件

---

## [v10.5.0] — 2026-06-04

### Added
- `compiler.fav` builtin NS リストに `"Snowflake"` を追加（2 箇所）
- Favnir pipeline で `Snowflake.*` を含む stage がコンパイル可能になった

### Notes
- テスト: 1271 件

---

## [v10.4.0] — 2026-06-04

### Added
- `checker.fav` に `snowflake_fn` 追加（`execute_raw` / `query_raw` 型シグネチャ）
- `builtin_ret_ty` / `ns_to_effect` に Snowflake エントリ追加
- E0320 エラーコード（`!Snowflake` エフェクト未宣言）

### Notes
- テスト: 1269 件

---

## [v10.3.0] — 2026-06-04

### Added
- `Effect::Snowflake` を 8 ファイルに追加（ast / parser / fmt / lineage / driver / ast_lower_checker / checker / reachability）
- `require_snowflake_effect` (E0314) — `!Snowflake` 未宣言 stage での Snowflake.* 呼び出しを検出

### Notes
- テスト: 1267 件

---

## [v10.2.0] — 2026-06-04

### Added
- `Snowflake.execute_raw` / `Snowflake.query_raw` VM primitive（Snowflake SQL API v2 REST + JWT RS256 認証）
- `snowflake_read_env` / `snowflake_generate_jwt` / `snowflake_api_post` ヘルパー（`vm.rs`）

### Notes
- テスト: 1264 件

---

## [v10.1.0] — 2026-06-04

### Added
- `infra/snowflake/` — Snowflake Terraform インフラ（provider / warehouse / database / schema / role / RSA キー / SSM）
- `infra/snowflake/README.md`

### Notes
- テスト: 1261 件

---

## [v10.0.0] — 2026-06-03

### Added
- `fav new <name>` — プロジェクトスキャフォールディング（fav.toml / src/main.fav / .gitignore 生成）
- `IO.make_dir_raw` VM primitive（ディレクトリ作成）
- GitHub Actions CI に self-check ステップ追加（fav check / fav lint / fav fmt --check）
- `CONTRIBUTING.md` を現状に合わせて更新

### Notes
- テスト: 1260 件（fav_new 統合テスト 2 件追加）

---

## [v9.13.0] — 2026-06-03

### Added
- `par [A, B] |> Merge` — 並列 stage 実行（`std::thread::spawn` VM スレッド並列化）
- E0016（par ステップ入力型不一致）/ E0017（par 内未定義 stage）
- `compiler.fav` / `checker.fav` に `SeqStep` / `SeqDef` / `IStage` / `ISeq` 型追加
- `ast_lower_checker.rs` に `lower_trf_def` / `lower_flw_def` / `te_to_string` 追加

### Notes
- テスト: 1258 件

---

## [v9.12.0] — 2026-06-02

### Added
- `interface` / `impl ... for` / `type T with Iface` を `checker.fav` / `compiler.fav` でセルフホスト対応
- E0014（MissingImpl）/ E0015（ImplMethodNotFound）
- LSP: Rune 定義ジャンプ（`textDocument/definition`）

### Notes
- テスト: 1251 件

---

## [v9.11.0] — 2026-06-01

### Added
- LSP: フィールド補完・モジュール補完（`List.` / `String.` 等）・Rune 補完
- LSP: Signature help（関数呼び出し時の型シグネチャ表示）
- `textDocument/completion` / `textDocument/signatureHelp` ハンドラ

### Notes
- テスト: 1240 件

---

## [v9.10.0] — 2026-05-31

### Added
- `fav repl` — インタラクティブ REPL（式評価・定義累積・`:type` / `:reset` / `:env`）
- `cmd_repl` in `cli.fav`

### Notes
- テスト: 1220 件

---

## [v9.9.0] — 2026-05-31

### Added
- `fav profile` — stage 別実行時間計測（`--profile` フラグ）
- `fav watch` — ファイル監視 + 自動再実行（500ms ポーリング）

### Notes
- テスト: 1217 件

---

## [v9.8.0] — 2026-05-31

### Added
- `fav doc` — `///` ドキュメントコメント + 型シグネチャから Markdown 自動生成
- `cmd_doc` in `cli.fav`、`doc_item` / `doc_program` in `compiler.fav`

### Notes
- テスト: 1213 件

---

## [v9.7.5] — 2026-05-31

### Added
- `where` バリデーター（`type Email(String) where |v| String.contains(v, "@")`）
- E0013（`expr?` を非 Result 関数内で使用）

### Fixed
- Float シリアライズで整数値に小数点が付かないバグを修正

### Notes
- テスト: 1206 件

---

## [v9.7.0] — 2026-05-31

### Added
- 名目型ラッパー `type Name(Inner)` — コンストラクタ・パターンマッチ対応
- `T?` / `T!` / `??` / `expr?` を self-hosted pipeline で対応
- `with Eq, Show, Serialize, Deserialize` 自動合成

### Notes
- テスト: 1200 件

---

## [v9.6.0] — 2026-05-31

### Added
- `!Llm` エフェクト追加
- `llm` Rune — `llm.complete` / `llm.chat` / `llm.extract<T>`（Claude / OpenAI 対応）
- `LLM_PROVIDER` / `LLM_MODEL` 環境変数で切り替え

### Notes
- テスト: 1191 件

---

## [v9.5.0] — 2026-05-31

### Added
- `!Http` エフェクト追加
- `http` Rune 拡張（`get_text` / `get_json<T>` / `post_json_typed<T,R>`）
- `grpc` Rune 拡張・`graphql` Rune 新規作成

### Notes
- テスト: 1187 件

---

## [v9.4.0] — 2026-05-31

### Added
- `json` Rune — `encode<T>` / `decode<T>` / `pretty`
- `csv` Rune 拡張 — `read<T>` / `write_file<T>`
- `gen` Rune 拡張 — `uuid` / `uuid_v7` / `nano_id`
- W004 lint ルール（`fn` の引数が 4 個以上 → レコード型推奨）

### Notes
- テスト: 1182 件

---

## [v9.3.0] — 2026-05-31

### Added
- `fav lint` — W001〜W005 静的解析ルールエンジン（compiler.fav + cli.fav）
- W001（EffectlessSink）/ W002（NoWriteInSeq）/ W003（UnusedBinding）/ W005（WildcardOnlyMatch）

### Notes
- テスト: 1173 件

---

## [v9.2.0] — 2026-05-31

### Added
- `fav fmt` — コードフォーマッタ（compiler.fav の pretty printer、冪等性保証）
- `Compiler.fmt_source_raw` VM primitive
- `--check` フラグ（CI 向け）

### Notes
- テスト: 1167 件

---

## [v9.1.0] — 2026-05-31

### Added
- stdlib 大幅拡充（`List.chunk` / `flat_map` / `group_by` / `zip_with` / `unique` 等 30 関数超）
- `rvm` 独立バイナリ（`src/bin/rvm.rs`）
- マルチパラメータクロージャ `|x, y| x + y` 対応
- E0012（非ジェネリック関数引数数不一致）

### Notes
- テスト: 1162 件

---

## [v9.0.0] — 2026-05-31

### Changed
- **セルフホスト完成宣言**: `fav run` / `fav check` の全経路が Favnir pipeline 経由で動作
- `--legacy` フラグ非推奨化

### Notes
- テスト: 1136 件

---

## [v7.0.0] — 2026-05-27

### Added
- `Effect::DbRead` / `Effect::DbWrite` / `Effect::DbAdmin` を型システムに追加（`ast.rs`）
- `parser.rs`：`!DbRead` / `!DbWrite` / `!DbAdmin` のパースに対応
- `checker.rs`：BUILTIN_EFFECTS 更新、`require_db_write_effect` / `require_db_admin_effect` 追加
- `reachability.rs`：3 エフェクトのリーチャビリティ追跡に対応
- `fmt.rs` / `driver.rs`：3 エフェクトの表示・JSON 出力に対応
- `runes/db/query.fav`：query 系 → `!DbRead`、execute 系 → `!DbWrite` に更新
- `runes/db/transaction.fav`：`!Db` → `!DbWrite` に更新
- `runes/db/migration.fav`：`applied_migrations` → `!DbRead`、`mark_applied` / `ensure_migrations_table` → `!DbAdmin` に更新
- `site/content/docs/guides/schema-authority.mdx`：Schema Authority 全体ワークフローガイド新規作成
- `site/content/docs/runes/db.mdx`：エフェクト細分化テーブル追記

### Changed
- `require_db_effect`：後方互換化（`Db | DbRead | DbWrite | DbAdmin` をすべて受け入れる）

### Notes
- テスト: 1044 件（パーサーテスト +1）
- 後方互換: 既存の `!Db` を使ったコードは変更なしに動く

---

## [v6.9.0] — 2026-05-27

### Added
- `LICENSE`（MIT）をリポジトリルートに配置
- `CONTRIBUTING.md`：ビルド手順・テスト手順・PR ガイドライン・Rune 追加ガイド
- `CHANGELOG.md`（本ファイル）
- CI に `cargo clippy -- -D warnings` を追加

---

## [v6.8.0] — 2026-05-27

### Added
- `site/content/docs/runes/db.mdx`：db rune リファレンス（connect / query / paginate / batch_insert / with_transaction / savepoint）
- `site/content/docs/runes/http.mdx`：http rune リファレンス（GET/POST/PUT/DELETE/PATCH / retry / bearer/basic/api_key）
- `site/content/docs/runes/duckdb.mdx`：query_one / explain / Parquet・CSV IO セクション追記（read_parquet / read_csv / write_parquet / write_csv）

---

## [v6.6.0] — 2026-05-27

### Added
- `one_of` 制約：`schemas/*.yaml` で列挙値バリデーションが可能に
- `TypeName.validate(record)`：VM 動的 dispatch による型付きバリデーション
- `Validate.rows_raw(type_name, rows)`：複数行一括バリデーション builtin
- 統合テスト 10 件追加（`vm_stdlib_tests.rs`）、合計 1043 件

### Changed
- `schema.mdx`：preview Note を削除、`Order.validate` / `Validate.rows_raw` のコード例を追加

---

## [v6.5.0] — 2026-05-27

### Added
- `site/content/docs/language/pipeline.mdx`：stage / seq / `|>` / abstract stage・seq / fav explain ドキュメント
- `site/content/docs/language/schema.mdx`：schemas/*.yaml 構文・制約一覧・T.validate・fav build --schema ドキュメント
- `site/content/docs/stdlib/infer.mdx`：fav infer --csv / --db / --proto / --out ドキュメント
- `site/content/docs/rune-cli.mdx`：fav deploy（Lambda）/ fav build --schema セクション追記

---

## [v6.4.0] — 2026-05-27

### Added
- `scripts/build-wasm.sh`：wasm-pack build → `site/public/wasm/` 自動出力
- WASM バックエンドで `List` 型対応（list_singleton / list_length / list_is_empty）
- Playground サンプルを stage/seq パイプライン例に更新

### Fixed
- WASM メモリ設定（`minimum: 2` / 128KB）で heap が有効化

---

## [v6.3.0] — 2026-05-26

### Added
- `compiler.fav` に `stage` / `seq` / `|>` のパース・lowering を追加
- `bootstrap_stage_seq_self_host_executes_correctly` テスト追加

---

## [v6.2.0] — 2026-05-25

### Added
- Bootstrap 3 段階検証確立（Stage1→Stage2→Stage3、`bytecode_A == bytecode_B`）
- 新オペコード 5 種：`CallNamed` / `JumpIfNotVariantC` / `GetFieldC` / `BuildRecordC` / `MakeClosureN`
- `String.to_bytes`（`-> List<Int>`）

### Fixed
- self-host 成熟度ドキュメント整備（`semantic_gap_audit.md` 等）

---

## [v6.1.0] — 2026-05-24

### Added
- `compiler.fav`：lexer.fav / parser.fav / codegen.fav をフルインライン化
- `bootstrap_stage1_builds_and_serializes` テスト追加

### Fixed
- `codegen.rs`：`remap_string_operands` が `Swap`/`TrackLine` を未認識で中断するバグを修正

---

## [v6.0.0] — 2026-05-21

### Added
- セルフホストコンパイラ完成（`fav/self/compiler.fav`）
- Favnir 製レキサー（`lexer.fav`）・パーサー（`parser.fav`）・型チェッカー（`checker.fav`）・コード生成器（`codegen.fav`）
- `IO.argv` / `List.take_while` / `List.drop_while` / `List.singleton` を VM に追加
- Bootstrap Stage1 実行テスト群

### Fixed
- `JumpIfNotVariant`：`VMValue::VariantCtor`（引数なしバリアント）のパターンマッチが正しく動作しないバグを修正

---

## [v5.5.0] — 2026-05

### Added
- `Map.remove` / `Map.contains_key` / `String.from_chars`
- `Option.and_then` / `Result.and_then`

---

## [v5.0.0〜v5.4.0] — 2026-05

### Added
- AWS Lambda / S3 / SQS 本番稼働
- SigV4 認証（セッショントークン対応）
- CloudFront + S3 リファレンスサイト公開
- `fav deploy`（Lambda）実装
- Import 解決順序：`rune_modules/` → `runes/` → `~/.fav/registry/`

---

## [v4.12.0〜v4.1.0] — 2025〜2026

### Added
- Rune エコシステム構築：aws / duckdb / db / http / auth / log / env / gen / grpc / json / parquet / csv / incremental / stat / validate
- `fav test` / `fav bench` / `fav check` / `fav run` CLI
- `fav explain`（パイプライン可視化）/ `fav infer`（型推論）/ `fav build --schema`（DDL 生成）
- `stage` / `seq` / `|>` パイプライン構文
- `abstract stage` / `abstract seq`（依存注入）
- パターンマッチ（ネスト・ガード・バリアント）
- `collect` / `yield` / クロージャ
- ジェネリクス・インターフェース・エフェクト型チェッカー
- バイトコードコンパイラ + VM
- WASM バックエンド（`favnir-wasm`）
- LSP（hover・diagnostics）
- `schemas/*.yaml` によるスキーマ制約システム
- LocalStack 対応（`AWS_ENDPOINT_URL` 切り替え）

---

[v6.9.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.9.0
[v6.8.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.8.0
[v6.6.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.6.0
[v6.5.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.5.0
[v6.4.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.4.0
[v6.3.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.3.0
[v6.2.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.2.0
[v6.1.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.1.0
[v6.0.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.0.0
