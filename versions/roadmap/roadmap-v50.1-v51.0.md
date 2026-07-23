# Roadmap v50.1.0 〜 v51.0.0 — Developer Experience 3.0

Date: 2026-07-18
Status: 計画中（v50.0 完了後に開始）

---

## 前提

- 直前完了: v50.0.0「Language Maturity / Production 2.0」（tests = 3091）
- マスターロードマップ: `roadmap-v50.1-v55.0.md`
- 本文書はマスターの v51.0 スプリント部分の詳細版
- **既存機能の扱い**: マスターロードマップ冒頭「既存機能との位置づけ」テーブルを参照。
  `fav explain-error`・`fav explain --lineage`・`par [A, B]` AST・`fav run --debug`・`fav audit` は
  既存実装済み。本スプリントでは「追加」ではなく「統一・拡張・本番品質化」として扱う。

---

## 目標

v50.0 で言語の成熟を宣言した。このスプリントでは
**「診断を統一し、エディタとの統合を深め、開発者の思考を止めない体験」**
を確立して Favnir v51.0 を宣言する。

---

## バージョン計画

### v50.1.0 — エラー診断統一 Phase 1（全コード suggestion 補完）

`error_catalog.rs` の `suggestion: None` が残る 34 件すべてに有意義な修正提案テキストを追加。
カバレッジテスト（全エントリに `suggestion: Some` が存在することを assert）で完備を保証。

```
// 改善前（suggestion: None のコード例）
E0018: duplicate bind target `order`

// 改善後
E0018: duplicate bind target `order`
  help: `bind` introduces an immutable binding. Use a different name,
        or remove the first binding if it is no longer needed.
```

**完了条件**: Rust テスト 2 件（実績推定 3093 tests passed, 0 failed）
- `error_suggestion_all_covered`
- `error_suggestion_e0018_text`

**実績**: 3093 tests passed, 0 failed（2026-07-18 完了）

---

### v50.2.0 — エラー診断統一 Phase 2（JSON / LSP / CLI 出力の一貫化）

`fav check --json` の出力・LSP `textDocument/publishDiagnostics`・CLI stderr の 3 経路すべてで
`suggestion` と `span` が一貫して出力されることを保証。各経路の出力構造をテストで固定。

```bash
# fav check --json
{ "code": "E0001", "message": "...", "suggestion": "did you mean `order`?", "span": {...} }

# LSP diagnostics
{ "code": "E0001", "message": "...", "data": { "suggestion": "did you mean `order`?" } }
```

**完了条件**: Rust テスト 2 件（実績推定 3095 tests passed, 0 failed）
- `check_json_includes_suggestion`
- `lsp_diagnostic_includes_suggestion`

**実績**: 3095 tests passed, 0 failed（2026-07-18 完了）

---

### v50.3.0 — `explain-error` と `explain` の統合 + 全コード explain テキスト網羅

`fav explain --error <code>` を正式導線として追加し、既存 `fav explain-error`（`main.rs` `Some("explain-error")`）を alias として残す。
`error_catalog.rs` に登録された全エラーコードに詳細 explain テキストを追加（現在は一部のみ記述済み）。

```bash
# 既存（後方互換として継続）
$ fav explain-error E0001

# 新たな正式導線
$ fav explain --error E0001
$ fav explain --error --list
$ fav explain --error --list --format json
```

**完了条件**: Rust テスト 2 件（実績推定 3097 tests passed, 0 failed）
- `explain_error_flag_works`
- `explain_error_all_codes_have_text`

**実績**: 3097 tests passed, 0 failed（2026-07-19 完了）

---

### v50.4.0 — LSP インレイヒント Phase 1（変数・関数戻り型）

`textDocument/inlayHint` ハンドラ・`"inlayHintProvider": true`・`bind` 束縛ヒントは v46.4.0 で実装済み。
本バージョンでは `fn` 戻り型ヒント（`collect_fn_return_hints`）を追加し、
型注釈なし `fn` 定義の推論戻り型を ` -> Type` 形式でインライン表示する。

```favnir
// エディタ表示イメージ
bind count /*: Int*/ <- List.length(items)
fn process(x: Int) /*-> Int*/ { x * 2 }
```

**完了条件**: Rust テスト 2 件（実績推定 3099 tests passed, 0 failed）
- `lsp_inlay_hint_let_binding`
- `lsp_inlay_hint_fn_return`

**実績**: 3099 tests passed, 0 failed（2026-07-19 完了）

---

### v50.5.0 — LSP インレイヒント Phase 2（パイプライン stage 型）

パイプラインの各 stage 入出力型を `/* : In -> Out */` 形式でインライン表示。
`lsp/references.rs` の拡張は不要と判断。`inlay_hints.rs` 内の新関数 `collect_pipeline_type_hints` で代替。

```favnir
// エディタ表示イメージ
pipeline OrderPipeline {
  stage Parse     /*: RawOrder -> Order*/          = |raw| { ... }
  stage Validate  /*: Order -> Result<Order>*/     = |order| { ... }
}
```

**完了条件**: Rust テスト 2 件（実績推定 3101 tests passed, 0 failed）
- `lsp_inlay_hint_stage_type`
- `lsp_inlay_hint_pipeline_type`

**実績**: 3101 tests passed, 0 failed（2026-07-19 完了）
- `collect_pipeline_type_hints` を `inlay_hints.rs` に追加（`Type::Arrow` / `Type::Trf` 限定）
- `references.rs` 変更は不要と判断、`inlay_hints.rs` 内の新関数で代替

---

### v50.6.0 — LSP ホバー情報強化（Rune メソッドシグネチャ）

`textDocument/hover` の応答に Rune メソッドのシグネチャ・エフェクト・ドキュメントコメントを含める。
`rune.toml` の `[[exports]]` セクションからメタデータを読み込む。

```
// kafka.consume にホバーした場合の表示
kafka.consume
  fn consume(topic: String) -> Stream<RawMessage>  !Kafka

  Consumes messages from the given Kafka topic.
```

**完了条件**: Rust テスト 2 件（実績推定 3103 tests passed, 0 failed）
- `lsp_hover_rune_method`
- `lsp_hover_builtin_fn`

**実績**: 3103 tests passed, 0 failed（2026-07-19 完了）
- `builtin_hover_at` / `rune_hover_at` / `word_and_ns_at` を `hover.rs` に追加
- `rune.toml` 動的読み込みは未実装、静的 `RUNE_FNS` テーブルで代替

---

### v50.7.0 — `fav run --trace` / `fav run --watch` 強化

`fav run --trace` の出力を stage 単位の構造化ログに統一（既存実装を拡張）。
`--watch <var.field>` フラグを新規追加し、VM の変数束縛フックに照合ロジックを挿入。
既存の `fav run --debug`（DAP）・`fav dap` とは独立した軽量導線として位置づける。

```bash
# fav run --trace（既存 → 構造化ログへ強化）
$ fav run pipeline.fav --trace
[trace] stage=Parse    in=RawOrder{id:1}  out=Order{id:1,amount:99.0}
[trace] stage=Validate in=Order{id:1}    out=Ok(Order{id:1})

# fav run --watch（新規）
$ fav run pipeline.fav --watch order.amount --watch order.status
[watch] order.amount: 0.0 → 99.0   (stage: Parse)
[watch] order.status: None → "ok"  (stage: Validate)
```

**完了条件**: Rust テスト 2 件（実績推定 3105 tests passed, 0 failed）
- `run_trace_structured_output`
- `run_watch_tracks_variable`

**実績**: 3105 tests passed, 0 failed（2026-07-19 完了）
- `SeqStageCheck` Ok 分岐の trace フォーマットを `[trace] stage=NAME  out=VALUE` に変更（構造化ログ）
- `WATCH_FIELDS` スレッドローカル + `set_watch_fields` / `watch_fields` + `SeqStageCheck` watch フック追加
- `in=` フィールドは未実装（SeqStageEnter 時点でスタック上に入力値がない制約）
- CLI `--watch` 解析は未実装（テスト API `set_watch_fields` 経由のみ）
- single-stage seq では SeqStageCheck が発火しないため、テストは 2-stage seq を使用

---

### v50.8.0 — ドキュメントサイト DX 3.0 記事

`site/content/docs/tools/diagnostics.mdx` — 統一された診断出力・`fav explain --error` の使い方。
`site/content/docs/tools/trace-watch.mdx` — `fav run --trace/--watch` のデバッグパターン。

**完了条件**: Rust テスト 2 件（実績推定 3107 tests passed, 0 failed）
- `docs_diagnostics_page_exists`
- `docs_trace_watch_page_exists`

**実績**: 3107 tests passed, 0 failed（2026-07-19 完了）
- `site/content/docs/tools/diagnostics.mdx` 新規作成（統一診断・`fav explain --error`・JSON/LSP 出力例）
- `site/content/docs/tools/trace-watch.mdx` 新規作成（`[trace] stage=NAME  out=VALUE` 構造化ログ・DAP との違い・`--watch` 将来対応予定の明記）

---

### v50.9.0 — 安定化・コードフリーズ（DX 3.0 前調整）

全 lint / clippy クリーン確認。`site/content/docs/dx3-overview.mdx` 骨子作成
（統一診断・インレイヒント・trace/watch の概要）。

**完了条件**: Rust テスト 2 件（実績推定 3109 tests passed, 0 failed）
- `cargo_toml_version_is_50_9_0`
- `dx3_overview_doc_exists`

**実績**: 3109 tests passed, 0 failed（2026-07-19 完了）
- `site/content/docs/dx3-overview.mdx` 新規作成（DX 3.0 全機能概要・v50.1〜v50.8 テーブル・各ドキュメントへのリンク）
- `cargo clippy -- -D warnings` クリーン確認

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
- `v51000_tests` 4 件 pass:
  - `cargo_toml_version_is_51_0_0`
  - `changelog_has_v51_0_0`
  - `milestone_has_dx3`
  - `readme_mentions_dx3`
- `MILESTONE.md` に `"Developer Experience 3.0"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

**実績**: 3113 tests passed, 0 failed（2026-07-19 完了）
- `MILESTONE.md` に v51.0.0 エントリ追加（"Developer Experience 3.0" 宣言文）
- `README.md` に DX 3.0 マイルストーン言及追加
- `v51000_tests` 6 件 pass（ロードマップ必須 4 件 + `dx3_milestone_declared` + `code_freeze_v51_0_0`）
- `cargo clean` 実施（★クリーンアップ完了）

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v50.1-v55.0.md`
- 前サブスプリント（完了）: `versions/roadmap/roadmap-v49.1-v50.0.md`
- 達成宣言: `MILESTONE.md`
