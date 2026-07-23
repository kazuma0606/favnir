# Tasks: v46.4.0 — LSP inlay hints 強化

Status: COMPLETE
Date: 2026-07-17

---

## T0 — 事前確認

- [x] `cargo test` 2999 passed, 0 failed を確認

## T1 — `checker.rs`: `Stmt::Bind` に `remember_type` 追加

- [x] `Stmt::Bind` ハンドラの `Pattern::Bind(name, span)` 分岐に `self.remember_type(span, &effective_ty)` を追加
- [x] `check_pattern_bindings` は変更しない

## T2 — `inlay_hints.rs`: `collect_stage_hints` 追加 + `handle_inlay_hints` 更新

- [x] `collect_stage_hints(source, type_at) -> Vec<InlayHint>` を追加（`collect_bind_hints` と対称な実装）
- [x] `find_stage_prefix(line) -> Option<&str>` を追加（`"stage "` プレフィックスを検出）
- [x] `handle_inlay_hints` を更新して `collect_bind_hints` + `collect_stage_hints` の結果を結合

## T3 — `driver.rs`: `v464000_tests` 追加

- [x] `lsp_inlay_hints_type_annotation`: LspServer 経由で `didOpen` → `inlayHint` の JSON-RPC シーケンスを実行し、レスポンスに `": Int"` が含まれることを `assert!(text.contains("\": Int\""))` で確認
- [x] `lsp_inlay_hints_pipeline`: `collect_stage_hints` を直接呼び出し、手動構築した `type_at` に対してヒントが 1 件以上返り `label.starts_with(": ")` であることを確認

## T4 — テスト＆完了

- [x] `cargo test` 3001 passed, 0 failed（2999 + 2件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `fav/Cargo.toml` version → `46.4.0`
- [x] `CHANGELOG.md` に v46.4.0 エントリ追加
- [x] `versions/current.md` を v46.4.0（3001 tests）に更新
- [x] tasks.md を COMPLETE に更新（T0〜T4 全チェック）

## コードレビュー指摘と対応

| 重大度 | 箇所 | 内容 | 対応 |
|---|---|---|---|
| [MED] | `versions/current.md` line 3 | `最終更新: 2026-07-16` が古い | `2026-07-17` に修正 |
| [LOW] | `inlay_hints.rs` `collect_stage_hints` | `collect_bind_hints` にある `name == "_"` スキップが欠落 | 同様のスキップ処理を追加 |
| [LOW] | `driver.rs` `lsp_inlay_hints_pipeline` | `Span::new` の col=7 の意味が非自明 | コメントで `start/end` の意味と col が無視される旨を明記 |
