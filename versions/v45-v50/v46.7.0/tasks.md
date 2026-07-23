# Tasks: v46.7.0 — `fav explain --lineage` 2.0

Status: COMPLETE
Date: 2026-07-17

---

## T0 — 事前確認

- [x] `cargo test` 3005 passed, 0 failed を確認

## T1 — `lineage.rs`: `is_dead` + `has_early_return` + `render_lineage_mermaid_with_opts`

- [x] `LineageEntry` に `pub is_dead: bool` フィールドを追加
- [x] `has_early_return(stmts: &[ast::Stmt]) -> bool` ヘルパーを追加（トップレベルのみ、Phase 1 スコープ）
- [x] `TrfDef` ブランチの `LineageEntry` 構築に `is_dead: has_early_return(&trf.body.stmts)` を追加
- [x] `FnDef` ブランチの `LineageEntry` 構築に `is_dead: has_early_return(&fndef.body.stmts)` を追加
- [x] `render_lineage_mermaid` を `render_lineage_mermaid_with_opts(report, false)` に委譲するよう変更
- [x] `render_lineage_mermaid_with_opts(report, show_dead: bool) -> String` を追加

## T2 — `driver.rs`: 既存 `LineageEntry` リテラル修正 + `pub use` + `cmd_explain_lineage`

- [x] 既存 `LineageEntry` 構造体リテラル 5 箇所に `is_dead: false` を追加
- [x] `pub use` ブロックに `render_lineage_mermaid_with_opts` を追加
- [x] `cmd_explain_lineage` シグネチャに `show_dead: bool` を追加
- [x] format = `"mermaid"` で `render_lineage_mermaid_with_opts(&report, show_dead)` を呼ぶよう更新

## T3 — `main.rs`: `--show-dead` フラグ追加

- [x] `--show-dead` アームを `--lineage` パースループに追加
- [x] `let mut show_dead = false;` を追加
- [x] `cmd_explain_lineage(file, &format, show_dead)` に更新

## T4 — `driver.rs`: `v467000_tests` 追加

- [x] `lineage_return_path_is_dead`: `is_dead == true` を確認
- [x] `lineage_happy_path_active`: `is_dead == false` + `show_dead=true` でも `"class Transform deadEntry"` が含まれないことを確認

## T5 — テスト＆完了

- [x] `cargo test` 3007 passed, 0 failed（3005 + 2件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `fav/Cargo.toml` version → `46.7.0`
- [x] `CHANGELOG.md` に v46.7.0 エントリ追加
- [x] `versions/current.md` を v46.7.0（3007 tests）に更新
- [x] tasks.md を COMPLETE に更新（T0〜T5 全チェック）

---

## コードレビュー指摘と対応（spec-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [HIGH] | `--show-dead` CLI フラグが main.rs に未追加 | T3 として実装 |
| [HIGH] | 既存 `LineageEntry` リテラル 5 箇所がコンパイル不能 | T2 で全箇所に `is_dead: false` 追加 |
| [HIGH] | `sanitize_mermaid_id` の所在確認 | lineage.rs 内に確認済み → plan.md に明記 |
| [MED] | テストアサーション文字列が脆弱 | `"class Transform deadEntry"` に変更 |
| [MED] | JSON 後方互換性への言及なし | spec.md に注意書き追加 |
| [LOW] | サイト MDX 更新の記載なし | v46.9.0 で一括対応 |
| [LOW] | 呼び出し元「等」の曖昧さ | plan.md に「1 箇所のみ」と明記 |
