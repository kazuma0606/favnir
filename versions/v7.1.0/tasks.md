# Favnir v7.1.0 Tasks

Date: 2026-05-27

## Goal

`fav explain --lineage` を実装する。
エフェクトと SQL 文字列リテラルを静的解析して Sources / Sinks / Transformations / Pipelines を出力。

---

## Phase A — CLI フラグ追加

- [x] A-1: `fav/src/main.rs` — `explain` コマンドパーサーに `--lineage` フラグを追加
- [x] A-2: `main.rs` — `lineage == true` の場合 `cmd_explain_lineage(file, &format)` を呼び出す

## Phase B — `lineage_analysis` 実装（driver.rs）

- [x] B-1: `LineageReport` / `LineageEntry` / `PipelineLineage` 構造体を定義
- [x] B-2: `fn extract_tables_from_sql(sql: &str) -> (Vec<String>, Vec<String>)` を実装（read/write テーブルを正規表現で抽出）
- [x] B-3: `fn collect_sql_literals(expr: &ast::Expr) -> Vec<String>` を実装（AST 再帰走査で DB.*_raw の文字列引数を収集）
- [x] B-4: `fn lineage_analysis(program: &ast::Program) -> LineageReport` を実装
  - TrfDef / FnDef からエフェクトと SQL を収集
  - FlowDef（seq）からパイプラインチェーンを構築

## Phase C — レンダリング実装（driver.rs）

- [x] C-1: `fn render_lineage_text(report: &LineageReport, filename: &str) -> String` を実装
- [x] C-2: `fn render_lineage_json(report: &LineageReport) -> String` を実装
- [x] C-3: `pub fn cmd_explain_lineage(file: Option<&str>, format: &str)` を実装（ファイル読み込み → lineage_analysis → render）

## Phase D — テスト追加（driver.rs）

- [x] D-1: `extract_tables_from_sql` のユニットテスト（FROM / JOIN / INSERT INTO / UPDATE / DELETE FROM）
- [x] D-2: `lineage_analysis` の統合テスト（stage + seq を含む .fav ソース → LineageReport 確認）
- [x] D-3: `render_lineage_text` のスモークテスト（出力に "Sources:" / "Sinks:" が含まれること）
- [x] D-4: `cargo test` 全件通過確認（1051 件）

## Phase E — ドキュメント更新

- [x] E-1: `site/content/docs/language/pipeline.mdx` に `## fav explain --lineage` セクションを追記

## Phase F — 最終確認

- [x] F-1: このファイルを完了状態に更新

---

## 完了条件まとめ

- `fav explain --lineage pipeline.fav` が Sources / Sinks / Transformations / Pipelines を出力する ✓
- `--format json` で lineage データが JSON 出力される ✓
- SQL 静的文字列からテーブル名が抽出される ✓
- seq 定義からパイプラインチェーンが表示される ✓
- 既存テスト 1051 件が全件通る ✓
