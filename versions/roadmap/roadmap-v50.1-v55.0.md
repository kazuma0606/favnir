# Roadmap v50.1.0 〜 v55.0.0 — Real-World Production

Date: 2026-07-18
Status: 計画中（v50.0 完了後に開始）

---

## 前提

- 直前完了: v50.0.0「Language Maturity / Production 2.0」（tests = 3091）
- 本文書は v50.1〜v55.0 の**マスターロードマップ**
- 各マイルストーン開始時に対応するサブスプリントロードマップを作成する

| サブスプリント文書 | カバー範囲 | 状態 |
|---|---|---|
| `roadmap-v50.1-v51.0.md` | v50.1〜v50.9 + v51.0 | 作成済み |
| `roadmap-v51.1-v52.0.md` | v51.1〜v51.9 + v52.0 | 作成済み |
| `roadmap-v52.1-v53.0.md` | v52.1〜v52.9 + v53.0 | 作成済み |
| `roadmap-v53.1-v54.0.md` | v53.1〜v53.9 + v54.0 | 作成済み |
| `roadmap-v54.1-v55.0.md` | v54.1〜v54.9 + v55.0 | 作成済み |

---

## 目標

v50.0「Language Maturity」で「迷わず使える実用言語」を宣言した。
このフェーズは **「現場で選ばれる言語」** を実現する。

3 つの柱を段階的に積み上げ、v55.0「Production 3.0」として宣言する：

1. **Developer Experience 3.0** — 診断の統一・エディタとの統合深化
2. **Performance & Scale** — 大規模パイプラインを安心して動かせる
3. **Data Quality & Observability 2.0** — 信頼できるデータを保証できる

### 既存機能との位置づけ

以下は v50.0 時点で実装済みであり、本ロードマップでは「追加」ではなく「統一・拡張・本番品質化」として扱う：

| 機能 | 既存状態 | 本ロードマップでの方針 |
|---|---|---|
| エラー suggestion / span | `error_catalog.rs` に `suggestion` フィールド、34 件が `None` | 全コードへの統一適用 + 出力経路の一貫化 |
| `fav explain-error <code>` | `main.rs` `Some("explain-error")` として実装済み | `explain` サブコマンドとの統合・全コード網羅 |
| `fav explain --lineage --format mermaid/dot/svg` | `main.rs` ヘルプ文字列・`cmd_explain_lineage` として実装済み | 統合表示・スキーマ情報の付加 |
| `par [A, B]` AST | `ast.rs` `FlwStep::Par` 実装済み | 実行基盤を Tokio 並列化・streaming 対応へ置換 |
| `fav run --debug` / `fav dap` | `main.rs` `Some("run")` / `Some("dap")`、`driver::cmd_run_debug` / `cmd_dap` として実装済み | 既存導線に統合（`fav run --trace/--watch`） |
| `fav audit` | `main.rs` `Some("audit")`、`fav_audit::cmd_audit` として実装済み | 名称衝突回避 → データアクセスログは `fav run --audit-log` |

---

## バージョン計画

---

## v51.0 — Developer Experience 3.0（v50.1〜v50.9）

### v50.1.0 — エラー診断統一 Phase 1（全コード suggestion 補完）

```
// 改善前（suggestion: None のコード例）
E0018: duplicate bind target `order`

// 改善後
E0018: duplicate bind target `order`
  help: `bind` introduces an immutable binding. Use a different name,
        or remove the first binding if it is no longer needed.
```

`error_catalog.rs` の `suggestion: None` が残る 34 件すべてに有意義な修正提案テキストを追加。
カバレッジテスト（全エントリに `suggestion: Some` が存在することを assert）で完備を保証。

**完了条件**: Rust テスト 2 件（`error_suggestion_all_covered` / `error_suggestion_e0018_text`）

---

### v50.2.0 — エラー診断統一 Phase 2（JSON / LSP / CLI 出力の一貫化）

```bash
# CLI
E0001: undefined variable `ordr`
  help: did you mean `order`?

# fav check --json
{ "code": "E0001", "message": "...", "suggestion": "did you mean `order`?", "span": {...} }

# LSP diagnostics
{ "code": "E0001", "message": "...", "data": { "suggestion": "did you mean `order`?" } }
```

`fav check --json` の出力・LSP `textDocument/publishDiagnostics` レスポンス・CLI stderr の 3 経路すべてで
`suggestion` と `span` が一貫して出力されることを保証。各経路の出力構造をテストで固定。

**完了条件**: Rust テスト 2 件（`check_json_includes_suggestion` / `lsp_diagnostic_includes_suggestion`）

---

### v50.3.0 — `explain-error` と `explain` の統合 + 全コード explain テキスト網羅

```bash
# 既存: explain-error <code>  → 継続サポート（後方互換）
$ fav explain-error E0001

# 統合: explain --error <code>  → 新たな正式導線
$ fav explain --error E0001
$ fav explain --error --list          # 全コード一覧
$ fav explain --error --list --format json
```

`fav explain --error <code>` を正式導線として追加し、既存 `fav explain-error` を alias として残す。
`error_catalog.rs` に登録された全エラーコードに詳細 explain テキストを追加（現在は一部のみ記述済み）。

**完了条件**: Rust テスト 2 件（`explain_error_flag_works` / `explain_error_all_codes_have_text`）

---

### v50.4.0 — LSP インレイヒント Phase 1（変数・関数戻り型）

```favnir
// エディタ表示イメージ
let count /*: Int*/ = List.length(items)
fn process(x: Int) -> /*Int*/ { x * 2 }
```

`lsp/` に `textDocument/inlayHint` ハンドラを追加（現行 LSP には未実装）。
`let` 束縛と型注釈なし `fn` 戻り型に推論型を `: Type` 形式でインライン表示。

**完了条件**: Rust テスト 2 件（`lsp_inlay_hint_let_binding` / `lsp_inlay_hint_fn_return`）

---

### v50.5.0 — LSP インレイヒント Phase 2（パイプライン stage 型）

```favnir
// エディタ表示イメージ
pipeline OrderPipeline {
  stage Parse     /*: RawOrder -> Order*/          = |raw| { ... }
  stage Validate  /*: Order -> Result<Order>*/     = |order| { ... }
}
```

パイプラインの各 stage 入出力型を `/* : In -> Out */` 形式でインライン表示。
`lsp/references.rs` の型情報収集を拡張して stage 型を提供。

**完了条件**: Rust テスト 2 件（`lsp_inlay_hint_stage_type` / `lsp_inlay_hint_pipeline_type`）

---

### v50.6.0 — LSP ホバー情報強化（Rune メソッドシグネチャ）

```
// kafka.consume にホバーした場合の表示
kafka.consume
  fn consume(topic: String) -> Stream<RawMessage>  !Kafka

  Consumes messages from the given Kafka topic.
```

`textDocument/hover` の応答に Rune メソッドのシグネチャ・エフェクト・ドキュメントコメントを含める。
`rune.toml` の `[[exports]]` セクションからメタデータを読み込む。

**完了条件**: Rust テスト 2 件（`lsp_hover_rune_method` / `lsp_hover_builtin_fn`）

---

### v50.7.0 — `fav run --trace` / `fav run --watch` 強化

```bash
# 既存: fav run --trace（基礎実装済み）→ stage 別の構造化ログへ強化
$ fav run pipeline.fav --trace
[trace] stage=Parse       in=RawOrder{id:1}  out=Order{id:1,amount:99.0}
[trace] stage=Validate    in=Order{id:1}     out=Ok(Order{id:1})

# 新規: --watch <var> で特定変数の変化を追跡
$ fav run pipeline.fav --watch order.amount --watch order.status
[watch] order.amount: 0.0 → 99.0   (stage: Parse)
[watch] order.status: None → "ok"  (stage: Validate)
```

`fav run --trace` の出力を stage 単位の構造化ログに統一（既存実装を拡張）。
`--watch <var.field>` フラグを新規追加し、VM の変数束縛フックに照合ロジックを挿入。
既存の `fav run --debug`（DAP）・`fav dap` とは独立した軽量導線として位置づける。

**完了条件**: Rust テスト 2 件（`run_trace_structured_output` / `run_watch_tracks_variable`）

---

### v50.8.0 — ドキュメントサイト DX 3.0 記事

`site/content/docs/tools/diagnostics.mdx` — 統一された診断出力・`fav explain --error` の使い方。
`site/content/docs/tools/trace-watch.mdx` — `fav run --trace/--watch` のデバッグパターン。

**完了条件**: Rust テスト 2 件（`docs_diagnostics_page_exists` / `docs_trace_watch_page_exists`）

---

### v50.9.0 — 安定化・コードフリーズ（DX 3.0 前調整）

全 lint / clippy クリーン確認。`site/content/docs/dx3-overview.mdx` 骨子作成（統一診断・インレイヒント・trace の概要）。

**完了条件**: Rust テスト 2 件（`cargo_toml_version_is_50_9_0` / `dx3_overview_doc_exists`）

---

### v51.0.0 — Developer Experience 3.0 宣言 ★クリーンアップ

**宣言文**:

> 「全エラーコードに修正提案が付き、JSON / LSP / CLI で一貫して届く。
>  エディタは型を表示し、trace はパイプラインの流れを可視化する。
>  Favnir の診断は開発者の思考を止めない。
>
>  これが Favnir v51.0 — Developer Experience 3.0 の姿である。」

**完了条件**:
- v50.1〜v50.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3113**）
- `v51000_tests` 4 件 pass（`cargo_toml_version_is_51_0_0` / `changelog_has_v51_0_0` / `milestone_has_dx3` / `readme_mentions_dx3`）
- `MILESTONE.md` に `"Developer Experience 3.0"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

**実績**: — （未実施）

---
---

## v52.0 — Performance & Scale（v51.1〜v51.9）

### v51.1.0 — `par` stage Tokio 並列実行基盤への置換 Phase 1

```favnir
// par [A, B] の AST（FlwStep::Par）は既存。
// v51.1 では逐次実行の実装を Tokio 並列化へ置換する。
pipeline IngestPipeline {
  stage Process: Order -> EnrichedOrder = |order| {
    par [Enrich(order), Validate(order)] |> Merge
  }
}
```

`ast.rs` の `FlwStep::Par` はすでに存在するが、VM の実行は逐次またはスタブ。
`ir.rs` に `Par` opcode を追加し、`compiler.rs` で `par [...]` → `Par` opcode を emit。
VM で `tokio::spawn` を使い `Par` の各要素を並列実行・join。エラーは fail-fast。

**完了条件**: Rust テスト 2 件（`par_stage_runs_parallel` / `par_stage_error_propagation`）

---

### v51.2.0 — `par` Phase 2（Merge.ordered / Merge.any + streaming 対応）

```favnir
// 全完了後に順序通りに結合
par [StageA, StageB, StageC] |> Merge.ordered

// 完了した順に結合（streaming フレンドリー）
par [StageA, StageB, StageC] |> Merge.any
```

`Merge.ordered`（`tokio::join_all` 相当）と `Merge.any`（`FuturesUnordered` 相当）を実装。
`ast.rs` に `MergeMode` enum を追加。Stream<T> を返す stage にも `par` が適用できることを確認。

**完了条件**: Rust テスト 2 件（`par_stage_merge_ordered` / `par_stage_merge_unordered`）

---

### v51.3.0 — ストリーミングバックプレッシャー制御

```toml
# fav.toml
[stream]
buffer_size = 1000   # producer が 1000 件溜まったらブロック
```

```favnir
stage Consume: Stream<RawOrder> -> Stream<Order> = |raw| {
  bind order <- kafka.consume("orders")
  Ok(order)  // バッファ満杯時は自動的にブロック
}
```

`fav.toml` の `[stream]` セクションを解析し `buffer_size` を VM のストリームバッファに適用。
Tokio の `mpsc::channel` で bounded channel を実装し、producer をブロック制御する。

**完了条件**: Rust テスト 2 件（`stream_backpressure_blocks` / `stream_buffer_size_config`）

---

### v51.4.0 — `fav bench` 差分回帰検出

```bash
$ fav bench --all --compare benchmarks/v51.3.0.json
checker_run_time:  1.2ms → 1.8ms  (+50%)  [WARN: exceeds 10% threshold]
compiler_run_time: 0.8ms → 0.9ms  (+12%)  [WARN]
vm_run_time:       2.1ms → 2.0ms  (-5%)   [OK]
```

`fav bench --compare <baseline.json>` フラグを追加。`benchmarks/` ディレクトリの前回結果と比較し、
閾値（デフォルト 10%）超過を警告。`--fail-on-regression` フラグで CI 向け非ゼロ終了コード。

**完了条件**: Rust テスト 2 件（`bench_regression_detected` / `bench_no_regression_passes`）

---

### v51.5.0 — インクリメンタルコンパイル依存グラフ

```bash
# a.fav を変更した場合
$ fav build
[skip]    b.fav — unchanged
[skip]    c.fav — unchanged
[rebuild] a.fav — changed
[rebuild] d.fav — depends on a.fav
```

ファイル間の import 依存グラフを構築し、変更ファイルの推移的依存ファイルのみ再コンパイル。
フィンガープリント（SHA-256）と依存グラフを `.fav-cache/dep-graph.json` に保存（v49.3 の
インクリメンタル型チェックを**コンパイル**フェーズにも拡張）。

**完了条件**: Rust テスト 2 件（`incremental_dep_graph_rebuilt` / `incremental_transitive_invalidation`）

---

### v51.6.0 — checker / compiler ホットパス最適化

`fav profile --build` で checker と compiler の処理時間を段階別に計測。
型代入（`Subst`）のクローン頻度を削減する `SubstRef` 参照共有を導入。
`compiler.rs` の `collect_merged_sources` の重複読み込みをキャッシュで排除。
`benchmarks/v51.6.0.json` に計測結果を保存。

**完了条件**: Rust テスト 2 件（`checker_perf_hot_path_improved` / `compiler_perf_baseline_recorded`）

---

### v51.7.0 — WASM ビルドサイズ最適化

```bash
$ fav build --target wasm
before: 412 KB
after:  287 KB  (-30%)
```

`wasm_dce.rs` の DCE（Dead Code Elimination）を強化し、未参照の export と内部関数を除去。
`wasm-opt -Os` 呼び出しを `build --target wasm` に統合。

**完了条件**: Rust テスト 2 件（`wasm_bundle_size_reduced` / `wasm_dce_removes_unused_fns`）

---

### v51.8.0 — ドキュメントサイト Performance 記事

`site/content/docs/runtime/parallel.mdx` — `par` stage の並列実行・マージモード・バックプレッシャー。
`site/content/docs/tools/bench-regression.mdx` — `fav bench --compare` による回帰検出の使い方。

**完了条件**: Rust テスト 2 件（`docs_parallel_page_exists` / `docs_bench_regression_page_exists`）

---

### v51.9.0 — 安定化・コードフリーズ（Performance & Scale 前調整）

全 lint / clippy クリーン確認。`site/content/docs/performance-overview.mdx` 骨子作成。

**完了条件**: Rust テスト 2 件（`cargo_toml_version_is_51_9_0` / `perf_overview_doc_exists`）

---

### v52.0.0 — Performance & Scale 宣言 ★クリーンアップ

**宣言文**:

> 「並列パイプラインはコアを使い切り、バックプレッシャーは
>  データの氾濫を防ぎ、ベンチマークは退行を即座に検出する。
>  Favnir は大規模データに立ち向かえる言語になった。
>
>  これが Favnir v52.0 — Performance & Scale の姿である。」

**完了条件**:
- v51.1〜v51.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3135**）
- `v52000_tests` 4 件 pass（`cargo_toml_version_is_52_0_0` / `changelog_has_v52_0_0` / `milestone_has_performance_scale` / `readme_mentions_performance_scale`）
- `MILESTONE.md` に `"Performance & Scale"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

**実績**: — （未実施）

---
---

## v53.0 — Data Quality & Observability 2.0（v52.1〜v52.9）

### v52.1.0 — `assert_schema` Phase 1（型チェック）

```favnir
type OrderRow = { id: Int, amount: Float, status: String }

stage ValidateSchema: Map<String, Any> -> Result<OrderRow> = |row| {
  bind validated <- assert_schema<OrderRow>(row)
  Ok(validated)
}
```

`assert_schema<T>(value)` を VM primitive として追加。`T` のフィールド名・型を実行時に検証し、不一致時 `Err` を返す。
`ast.rs` / `compiler.rs` に `AssertSchema` ノードを追加。E0419 エラーコードを追加。

**完了条件**: Rust テスト 2 件（`assert_schema_type_ok` / `assert_schema_type_fail`）

---

### v52.2.0 — `assert_schema` Phase 2（nullable・追加フィールド対応）

```favnir
type FlexRow = { id: Int, name: String, note?: String }

// note フィールドが存在しなくても OK（nullable）
// 想定外フィールドがあれば W036 警告（--strict-schema でエラー化）
bind row <- assert_schema<FlexRow>(raw_map)
```

nullable フィールド（`field?: Type`）を `assert_schema` で許容。想定外フィールドを W036 警告として追加。
`--strict-schema` フラグで W036 をエラー化する。

**完了条件**: Rust テスト 2 件（`assert_schema_nullable_field` / `assert_schema_extra_field_warn`）

---

### v52.3.0 — `fav explain --lineage` 表示強化（スキーマ情報付加）

```bash
# 既存: fav explain --lineage --format mermaid（実装済み）
# v52.3: ノードにスキーマ型情報を付加して拡張

$ fav explain --lineage pipeline.fav --format mermaid --with-schema
```

```
flowchart LR
  kafka.consume["kafka.consume\nStream<RawOrder>"] --> Parse["Parse\nRawOrder → Order"]
  Parse --> Validate["Validate\nOrder → Result<Order>"]
  Validate --> snowflake.insert["snowflake.insert\n!Snowflake(write)"]
```

既存の `fav explain --lineage --format mermaid/dot/svg` に `--with-schema` オプションを追加。
`assert_schema` で検証されたスキーマ名をノードラベルに表示。コマンド名は `fav explain --lineage` に統一
（`fav lineage --graph` という新コマンドは作成しない）。

**完了条件**: Rust テスト 2 件（`lineage_mermaid_with_schema` / `lineage_dot_with_schema`）

---

### v52.4.0 — `fav explain --lineage` インタラクティブ HTML レポート

```bash
$ fav explain --lineage pipeline.fav --format html -o lineage.html
# → ブラウザで開けるインタラクティブな SVG + テーブル
```

`--format html` オプションを追加。依存グラフを SVG でレンダリングし、クリックで stage 詳細
（型・エフェクト・スキーマ）を表示できる自己完結型 HTML を生成。

**完了条件**: Rust テスト 2 件（`lineage_html_output` / `lineage_html_has_stage_detail`）

---

### v52.5.0 — SLA 監視 Rune

```favnir
import sla

stage CheckFreshness: DataBatch -> Result<DataBatch> = |batch| {
  bind _ <- sla.check_freshness(batch.timestamp, max_age_seconds: 3600)
  bind _ <- sla.check_latency(stage: "Parse", threshold_ms: 200)
  Ok(batch)
}
```

`runes/sla/sla.fav` に `check_freshness` / `check_latency` / `alert` 関数を追加。
SLA 違反時は `!Observe` エフェクト経由でアラートを発火し `Err` を返す。

**完了条件**: Rust テスト 2 件（`sla_rune_latency_check` / `sla_rune_freshness_check`）

---

### v52.6.0 — `fav run --audit-log` データアクセスログ

```bash
# fav audit は Enterprise Governance 用（既存）。
# データアクセスログは別フラグとして追加する。
$ fav run pipeline.fav --audit-log audit.jsonl
```

```json
{"ts":"2026-07-18T10:00:00Z","op":"read","effect":"Kafka","topic":"orders"}
{"ts":"2026-07-18T10:00:01Z","op":"write","effect":"Snowflake","table":"orders_v2"}
```

`fav run --audit-log <output.jsonl>` オプションを追加（既存の `fav audit` コマンドとは独立）。
`!Kafka` / `!Snowflake` / `!S3` のアクセスイベントを JSONL 形式で記録。VM の effect ディスパッチフックに挿入。

**完了条件**: Rust テスト 2 件（`audit_log_read_event` / `audit_log_write_event`）

---

### v52.7.0 — OTel 強化（span 属性にスキーマ・リネージ情報付加）

```
span: stage.Validate
  schema.name       = "OrderRow"
  schema.fields     = "id,amount,status"
  lineage.upstream  = "Parse"
  lineage.downstream = "snowflake.insert"
```

OTel の stage span に `schema.name` / `schema.fields` / `lineage.upstream` / `lineage.downstream` 属性を付与。
`assert_schema` 呼び出し時にスキーマ名を span コンテキストに記録。

**完了条件**: Rust テスト 2 件（`otel_span_has_schema_attr` / `otel_span_has_lineage_attr`）

---

### v52.8.0 — ドキュメントサイト Data Quality 記事

`site/content/docs/data-quality/assert-schema.mdx` — `assert_schema` の使い方・nullable・strict モード。
`site/content/docs/tools/lineage-enhanced.mdx` — `--with-schema` / `--format html` の使い方。
`site/content/docs/tools/audit-log.mdx` — `fav run --audit-log` の使い方・JSONL フォーマット。

**完了条件**: Rust テスト 2 件（`docs_assert_schema_page_exists` / `docs_audit_log_page_exists`）

---

### v52.9.0 — 安定化・コードフリーズ（Data Quality 2.0 前調整）

全 lint / clippy クリーン確認。`site/content/docs/data-quality-overview.mdx` 骨子作成。

**完了条件**: Rust テスト 2 件（`cargo_toml_version_is_52_9_0` / `dq_overview_doc_exists`）

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
- `v53000_tests` 4 件 pass（`cargo_toml_version_is_53_0_0` / `changelog_has_v53_0_0` / `milestone_has_data_quality` / `readme_mentions_data_quality`）
- `MILESTONE.md` に `"Data Quality & Observability 2.0"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

**実績**: — （未実施）

---
---

## v54.0 — Integration Sprint（v53.1〜v53.9）

### v53.1.0 — lineage × LSP 統合（リネージをエディタで表示）

```
// Validate stage にホバーした場合
stage Validate
  type:       Order -> Result<Order>
  upstream:   Parse
  downstream: snowflake.insert
  effects:    !Snowflake(write)
  schema:     OrderRow
```

`lsp/references.rs` のホバー応答に lineage 情報（upstream / downstream stage・スキーマ名）を追加。
`lineage.rs` の `collect_lineage` 結果を LSP サーバーにキャッシュし、ホバー時に参照。

**完了条件**: Rust テスト 2 件（`lsp_hover_shows_lineage` / `lsp_hover_lineage_upstream`）

---

### v53.2.0 — bench × par 統合（par stage 個別計測）

```bash
$ fav bench pipeline.fav
par.Enrich:  12.3ms
par.Validate: 18.7ms  ← ボトルネック
par.total:   18.9ms  (limited by Validate)
```

`fav bench` が `par` ブロック内の各 stage を個別に計測し最遅 stage（ボトルネック）を明示。
`benchmarks/<version>.json` のスキーマに `par_stages` フィールドを追加。

**完了条件**: Rust テスト 2 件（`bench_par_stage_individual` / `bench_par_stage_total`）

---

### v53.3.0 — DX × DQ 統合（`assert_schema` 失敗時の詳細 suggestion）

```
E0419: schema validation failed for OrderRow
  --> src/pipeline.fav:8:3
  expected: { id: Int, amount: Float, status: String }
  got:      { id: "abc", amount: 99.0, status: "ok" }
  field `id`: expected Int, got String
  help: use Int.parse(row["id"]) to convert
```

`assert_schema` 失敗時に v50.1〜v50.2 で整備した統一診断フォーマット（span + suggestion）を適用。
`fav explain --error E0419` でフィールド差分の詳細説明を参照できるよう `EXPLAIN_CATALOG` を更新。

**完了条件**: Rust テスト 2 件（`assert_schema_error_has_suggestion` / `assert_schema_diff_shown`）

---

### v53.4.0 — E2E 統合デモ Phase 1（Kafka → par transform → Snowflake）

```favnir
// examples/v55-demo/pipeline.fav
import kafka
import snowflake
import "./stages/enrich" as enrich
import "./stages/validate" as validate

pipeline OrderIngestion {
  stage Consume: Stream<RawOrder> -> Stream<Order> = |raw| {
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

`examples/v55-demo/` ディレクトリに大規模パイプラインデモを作成。`par` 並列 stage・新 import 構文を活用。

**完了条件**: Rust テスト 2 件（`e2e_integration_demo_structure` / `e2e_integration_demo_uses_par`）

---

### v53.5.0 — E2E 統合デモ Phase 2（assert_schema + audit-log + OTel）

```favnir
stage Validate: Order -> Result<ValidOrder> = |order| {
  bind checked <- assert_schema<ValidOrder>(order)
  Ok(checked)
  // fav run --audit-log 実行時: アクセスログに自動記録
  // OTel span に schema.name = "ValidOrder" が付与
}
```

E2E デモに `assert_schema`・`--audit-log` 対応・OTel span を統合。デモ用 `run.sh` を更新。

**完了条件**: Rust テスト 2 件（`e2e_integration_demo_has_schema` / `e2e_integration_demo_has_audit_log`）

---

### v53.6.0 — cookbook 更新

`site/content/cookbook/parallel-pipeline.mdx` — `par` stage・マージモード・バックプレッシャーのレシピ。
`site/content/cookbook/schema-validation.mdx` — `assert_schema` + nullable + OTel のレシピ。

**完了条件**: Rust テスト 2 件（`cookbook_parallel_pipeline_exists` / `cookbook_schema_validation_exists`）

---

### v53.7.0 — ドキュメントサイト全体最終チェック

全 MDX ページのリンク切れ修正・用語統一（`rune` / `stage` / `pipeline` の表記統一）。
`site/content/docs/glossary.mdx` を v51〜v53 新語彙で更新。

**完了条件**: Rust テスト 2 件（`docs_no_broken_links` / `docs_glossary_updated`）

---

### v53.8.0 — CHANGELOG / MILESTONE 整理（v51〜v53 まとめ）

CHANGELOG の v51.0〜v53.0 エントリを整理・補完。MILESTONE.md に v51〜v53 の達成サマリーを追記。

**完了条件**: Rust テスト 2 件（`changelog_has_v51_to_v53_summary` / `milestone_integration_sprint_noted`）

---

### v53.9.0 — 安定化・コードフリーズ（Integration Sprint 前調整）

全 lint / clippy クリーン確認。`site/content/docs/integration-overview.mdx` 骨子作成。

**完了条件**: Rust テスト 2 件（`cargo_toml_version_is_53_9_0` / `integration_overview_doc_exists`）

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
- `v54000_tests` 4 件 pass（`cargo_toml_version_is_54_0_0` / `changelog_has_v54_0_0` / `milestone_has_integration_sprint` / `readme_mentions_integration_sprint`）
- `MILESTONE.md` に `"Integration Sprint"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

**実績**: — （未実施）

---
---

## v55.0 — Production 3.0 宣言（v54.1〜v54.9）

### v54.1.0 — 全エラーコード `fav explain --error` 対応完備

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

`error_catalog.rs` に登録された全エラーコードに `fav explain --error` テキストを追加（`EXPLAIN_CATALOG` を完備）。
`explain_error_all_codes_have_text` テストでカバレッジを強制（E0419 含む将来追加コードも自動カバー）。

**完了条件**: Rust テスト 2 件（`explain_error_all_codes_have_text` / `explain_error_e0419_exists`）

---

### v54.2.0 — `fav run --watch` の高度化（差分表示・サマリー）

```bash
$ fav run pipeline.fav --watch order.amount --watch order.status --watch-diff
[watch] order.amount:  0.0  → 99.0   Δ+99.0  (stage: Parse)
[watch] order.status:  None → "ok"            (stage: Validate)
```

`--watch-diff` フラグを追加し、変化量（数値は差分、文字列は before/after）を表示。
複数 stage にまたがった追跡履歴をまとめてサマリー出力する `--watch-summary` オプションを追加。

**完了条件**: Rust テスト 2 件（`run_watch_diff_numeric` / `run_watch_summary_output`）

---

### v54.3.0 — パフォーマンスリグレッションスイート CI 統合

```yaml
# .github/workflows/bench.yml（既存）に以下ステップを追加
- run: cargo test bench_ -- --nocapture
- run: fav bench --all --compare benchmarks/baseline.json --fail-on-regression
```

既存の `.github/workflows/bench.yml` に `fav bench --fail-on-regression` ステップを追加（新ワークフロー新設ではなく既存を拡張）。
`benchmarks/baseline.json` をリポジトリ管理し、PR ごとに自動比較。

**完了条件**: Rust テスト 2 件（`ci_perf_regression_suite` / `ci_perf_baseline_recorded`）

---

### v54.4.0 — `fav dq-report` データ品質レポートコマンド

```bash
$ fav dq-report --audit-log audit.jsonl --schemas schemas/
Schema validation:  12,450 rows checked, 3 errors (0.02%)
  OrderRow:  12,100 / 12,100 OK
  PaymentRow:   350 / 350 OK
SLA violations:  latency >200ms at 2026-07-18T09:15:00Z (stage: Parse)
```

`fav dq-report` コマンドを追加。`--audit-log` の JSONL ログとスキーマ統計を集計し Markdown レポートを生成。

**完了条件**: Rust テスト 2 件（`cmd_dq_report_generates` / `cmd_dq_report_has_schema_stats`）

---

### v54.5.0 — `fav doctor` 環境診断コマンド

```bash
$ fav doctor
[OK]   Rust toolchain: 1.79.0
[OK]   fav version: 54.5.0
[OK]   fav.toml: valid
[WARN] rune kafka: version 2.1.0 declared but not installed
       run: fav install kafka
[OK]   .fav-cache: intact (fingerprints: 42 files)
```

`fav doctor` コマンドを追加。Rust バージョン・`fav.toml` 有効性・rune インストール状態・
`.fav-cache` 整合性を一括チェック。

**完了条件**: Rust テスト 2 件（`cmd_doctor_passes_clean_env` / `cmd_doctor_detects_missing_rune`）

---

### v54.6.0 — README / CONTRIBUTING 最終整備

`README.md` に Production 3.0 への言及・v51〜v55 機能サマリーを追加。
`CONTRIBUTING.md` のコントリビュート手順を最新化（`fav doctor` / `fav bench` の実行手順追記）。

**完了条件**: Rust テスト 2 件（`readme_has_production3_mention` / `contributing_has_doctor_step`）

---

### v54.7.0 — ドキュメントサイト Production 3.0 overview ページ

```
site/content/docs/production3-overview.mdx
  - v51: Developer Experience 3.0（診断統一・インレイヒント・trace/watch）
  - v52: Performance & Scale（par Tokio・バックプレッシャー・bench 回帰）
  - v53: Data Quality & Observability 2.0（assert_schema・lineage 強化・audit-log）
  - v54: Integration Sprint（lineage × LSP・bench × par・E2E デモ）
  → v55: Production 3.0 宣言
```

`site/content/docs/production3-overview.mdx` を新規作成。

**完了条件**: Rust テスト 2 件（`docs_production3_overview_exists` / `docs_production3_has_v55`）

---

### v54.8.0 — MILESTONE.md Production 3.0 エントリ追加

`MILESTONE.md` に `## v55.0.0（予定）— Production 3.0` エントリを追加。v51〜v54 の達成内容を記録。

**完了条件**: Rust テスト 2 件（`milestone_has_production3` / `milestone_has_v55`）

---

### v54.9.0 — v55.0 前調整・安定化

コードフリーズ。全 lint / clippy クリーン確認。`site/content/docs/production3-overview.mdx` を完成させる。
`cargo test` 全通過を確認して v55.0 へ。

**完了条件**: Rust テスト 2 件（`cargo_toml_version_is_54_9_0` / `production3_overview_doc_complete`）

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
- `v55000_tests` 4 件 pass（`cargo_toml_version_is_55_0_0` / `changelog_has_v55_0_0` / `milestone_has_production3` / `readme_mentions_production3`）
- `MILESTONE.md` に `"Production 3.0"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

**実績**: — （未実施）

---

## テスト数推移

| バージョン | 想定テスト数 | 累積増加 |
|---|---|---|
| v50.0.0（ベース） | 3091 | — |
| v51.0.0 | ~3113 | +22 |
| v52.0.0 | ~3135 | +44 |
| v53.0.0 | ~3157 | +66 |
| v54.0.0 | ~3179 | +88 |
| v55.0.0 | ~3201 | +110 |

各サブスプリント 2 件追加、各マイルストーン 4 件追加（x.0.0 テストモジュール）。
実際の件数はサブスプリントロードマップ作成時に確定する。

---

## 参考リンク

- 前マスターロードマップ（完了）: `versions/roadmap/roadmap-v45.1-v50.0.md`
- 前サブスプリント詳細（完了）: `versions/roadmap/roadmap-v49.1-v50.0.md`
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
