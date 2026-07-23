# Tasks: v46.1.0 — `#[test]` ブロック AST + parser

Status: COMPLETE
Date: 2026-07-16

---

## T0 — 事前確認

- [x] `cargo test` 2992 passed, 0 failed を確認

## T1 — `ast.rs`: `FnDef.is_test` 追加

- [x] `FnDef` 構造体に `pub is_test: bool,` を追加（`deprecated: bool` の直後）

## T2 — `parser.rs`: `parse_test_annotation()` 追加

- [x] `parse_deprecated_annotation()` の直後に `parse_test_annotation()` を実装
- [x] `# [ test ] ` の 4 トークンをルックアヘッドで検出（`TokenKind::Test` キーワードを使用）

## T3 — `parser.rs`: `parse_item()` への適用

- [x] `let deprecated_ann = ...` の直後に `let test_ann = self.parse_test_annotation()?;` 追加
- [x] `fd.is_test = test_ann;` 追加（2箇所）:
  - `TokenKind::Fn` アーム（同期 fn、parser.rs 699 行付近）
  - `TokenKind::Async` → `Fn` アーム（async fn、parser.rs 728 行付近）

## T4 — `parser.rs`: `FnDef { ... }` 構築に `is_test: false` 追加

- [x] `parse_fn_def()` の `Ok(FnDef { ... })` 構築に `is_test: false,` を追加

## T5 — `driver.rs`: v461000_tests 追加

- [x] `v461000_tests` モジュール追加（`v46000_tests` の直後）
- [x] `test_block_parses` テスト実装
- [x] `test_fn_collected` テスト実装（`FnDef.is_test == true` で `name == "test_add"` を確認）

## T6 — テスト＆完了

- [x] `cargo build` クリーン
- [x] `cargo test` 2994 passed, 0 failed（2992 + 2件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `fav/Cargo.toml` version → `46.1.0`
- [x] `CHANGELOG.md` に v46.1.0 エントリ追加
- [x] `versions/current.md` を v46.1.0（2994 tests）に更新
- [x] tasks.md を COMPLETE に更新（T0〜T6 全チェック）

## コードレビュー指摘と対応

| 重大度 | 箇所 | 内容 | 対応 |
|---|---|---|---|
| [HIGH] | spec/plan コードスニペット | `self.peek() == Some(&TokenKind::Hash)` — 実際は `&TokenKind::Hash` | 修正済み |
| [HIGH] | roadmap の `TestBlock` 記述と実装の乖離 | `FnDef.is_test: bool` で実装する設計判断を spec に追記、roadmap 更新 | 修正済み |
| [HIGH] | `parse_test_annotation()` | `test` は `TokenKind::Test`（キーワード）なので `Ident` パターンが不一致 → 実装時に発覚 | `t.kind == TokenKind::Test` に修正 |
| [MED] | `fmt.rs` `fn_def()` | `is_test` を出力しないため `fav fmt` でアノテーションが消える | `test_prefix` を先頭に追加して修正 |
| [MED] | `driver.rs` `v46000_tests::cargo_toml_version_is_46_0_0` | Cargo.toml が `46.1.0` になったため assert 失敗 | 慣例に従い assert を空化（コメントのみ） |
| [LOW] | `versions/current.md` | `cargo install` バージョンが `46.0.0` のまま | `46.1.0` に修正 |
| [LOW] | `lint.rs` `is_test` 免除 | 将来 `pub #[test] fn` を許可した際 L001 誤発火の可能性 | v46.x 将来対応 |
| [MED] | テスト数の不一致（roadmap 2991 vs spec 2994） | spec に注記追加、roadmap の推定値を 2994 に修正 | 修正済み |
| [MED] | tasks.md T3 の 2 箇所が不明確 | 同期 fn / async fn アームの行番号を明記 | 修正済み |
| [LOW] | `#[test]` と `#[deprecated]` 同時付与が未明記 | spec に「v46.1.0 スコープ外」を追記 | 修正済み |
| [LOW] | 2 ステップ初期化の説明が不明確 | spec に説明追記 | 修正済み |
| [LOW] | site/ MDX が v46.9.0 スコープである旨が未記載 | spec の変更しないファイルに追記 | 修正済み |
