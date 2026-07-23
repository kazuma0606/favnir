# Tasks: v46.6.0 — `fav explain` 2.0 Phase 1（パイプライン図改善）

Status: COMPLETE
Date: 2026-07-17

---

## T0 — 事前確認

- [x] `cargo test` 3003 passed, 0 failed を確認

## T1 — `driver.rs`: `scan_returns` + `is_err_call` + `render_pipeline_mermaid_v2` 追加

- [x] `is_err_call(expr: &Expr) -> bool` 追加（`Apply(Ident("Err"), ...)` 判定）
- [x] `scan_returns(stmts: &[Stmt]) -> (bool, bool)` 追加（トップレベルのみ、Phase 1 スコープ）
- [x] `render_pipeline_mermaid_v2(program: &Program) -> String` 追加（`pub(crate)`）
  - `flowchart LR` + 2つの `classDef` 出力
  - 各 `FnDef` をスキャンしノード + dotted edge + class 付与
  - `$` で始まる名前はスキップ

## T2 — `driver.rs`: `v466000_tests` 追加

- [x] `explain_mermaid_includes_dead_path`: `return x` を含む fn → `-.->` と `deadPath` の両方を確認
- [x] `explain_pipeline_v2`: `return Err(x)` を含む fn → `errPath` と `-.->` を確認

## T3 — テスト＆完了

- [x] `cargo test` 3005 passed, 0 failed（3003 + 2件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `fav/Cargo.toml` version → `46.6.0`
- [x] `CHANGELOG.md` に v46.6.0 エントリ追加
- [x] `versions/current.md` を v46.6.0（3005 tests）に更新
- [x] tasks.md を COMPLETE に更新（T0〜T3 全チェック）

---

## コードレビュー指摘と対応（spec-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [MED] | `pub(crate)` の根拠が spec に未記載 | spec.md に「テストから呼び出すため」と明記 |
| [MED] | `scan_returns` がネスト検出しないことが不明瞭 | spec.md に「トップレベル stmts のみ、Phase 1 スコープ外」と明記 |
| [MED] | `writeln!` の末尾スペース | plan.md から除去 |
| [LOW] | コマンド統合方針が未記述 | spec.md に「コマンド差し替えは v46.7.0 以降」と明記 |
| [LOW] | テストモジュール配置コメント記述 | 軽微、実装に影響なし |
