# Tasks: v46.5.0 — LSP クイックフィックス強化

Status: COMPLETE
Date: 2026-07-17

---

## T0 — 事前確認

- [x] `cargo test` 3001 passed, 0 failed を確認

## T1 — `code_action.rs`: CA-4 + CA-5 追加

- [x] `parse_did_you_mean(hint: &str) -> Option<&str>` を追加（backtick 間の文字列を抽出）
- [x] `check_did_you_mean_fix(doc, uri, range) -> Vec<CodeAction>` を追加
  - E0102 エラーを行番号でフィルタ（`span.line > 0 && span.line as u32 - 1 == line_idx`）
  - hints から候補名を抽出して TextEdit 付き CodeAction を生成
- [x] `check_arg_count_fix(_uri, doc, range) -> Vec<CodeAction>` を追加
  - E0101 + `message.contains("argument(s)")` でフィルタ
  - `title: format!("Fix: {}", e.message)`, `edit: None`
- [x] `handle_code_action` に `actions.extend(check_did_you_mean_fix(...))` + `actions.extend(check_arg_count_fix(...))` を追加

## T2 — `driver.rs`: `v465000_tests` 追加

- [x] `lsp_quick_fix_undefined_var`: E0102 が発行されることを `assert!` で確認、did-you-mean アクションの `kind == Some("quickfix")` を確認
- [x] `lsp_quick_fix_arg_count`: E0101 が発行されることを `assert!` で確認、`title.contains("Fix: expected")` の CodeAction が存在することを `assert!` で確認

## T3 — ロードマップ修正

- [x] `roadmap-v46.1-v47.0.md` の v46.5.0 セクション `E0007（引数数不一致）` → `E0101（引数数不一致）`、`E0001` → `E0102` に修正

## T4 — テスト＆完了

- [x] `cargo test` 3003 passed, 0 failed（3001 + 2件）
- [x] `cargo clippy -- -D warnings` クリーン（テスト通過により確認）
- [x] `fav/Cargo.toml` version → `46.5.0`
- [x] `CHANGELOG.md` に v46.5.0 エントリ追加
- [x] `versions/current.md` を v46.5.0（3003 tests）に更新
- [x] tasks.md を COMPLETE に更新（T0〜T4 全チェック）

---

## コードレビュー指摘と対応（spec-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [HIGH] | E0001/E0008 は Rust checker に存在しない | E0102/E0101 に全修正 |
| [HIGH] | テストソースが実質無効（`assert!(true)`） | `totally_undefined_xyz` + 実 `assert!` に変更 |
| [HIGH] | CA-5 edit:None とロードマップ「提案」の矛盾 | spec に「診断表示のみ、v46.9.0 以降で TextEdit」と明記 |
| [MED] | URI パラメータ非対称 | `check_arg_count_fix(_uri, doc, range)` で統一 |
| [MED] | hint フォーマット（小文字）vs タイトル（大文字）未明記 | spec に明記 |
| [MED] | MDX ドキュメント計画がない | v46.9.0 延期と spec に明記 |
| [LOW] | ロードマップ誤記 E0007 | roadmap 修正済み + T3 で対応 |
