# v31.5.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `31.4.0` であること
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2437 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v315000_tests` が存在しないこと
- [x] v31.4.0 が COMPLETE であること
- [x] `fav/src/lsp/inlay_hints.rs` が存在しないこと（新規作成対象）
- [x] `lsp/mod.rs` の `initialize` 応答に `inlayHintProvider` が含まれないこと（追加対象）
- [x] `lsp/mod.rs` に `"textDocument/inlayHint"` ハンドラが存在しないこと（追加対象）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `31.4.0` → `31.5.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_31_4_0` をスタブ化
- [x] **T3** `fav/src/lsp/inlay_hints.rs` — 新規作成（`InlayHint` 構造体 + `handle_inlay_hints()` + `collect_bind_hints()`）
- [x] **T4** `fav/src/lsp/mod.rs` — `pub mod inlay_hints;` 宣言追加
- [x] **T5** `fav/src/lsp/mod.rs` — `initialize` 応答に `"inlayHintProvider": true` 追加
- [x] **T6** `fav/src/lsp/mod.rs` — `"textDocument/inlayHint"` ハンドラを追加（`references` アームの直後）
- [x] **T7** `fav/editors/favnir-vscode/package.json` — `inlayHints` capability を追記
- [x] **T8** `fav/src/driver.rs` — `v315000_tests`（3 件）を追加（`use super::*` あり）
- [x] **T9** `CHANGELOG.md` — `[v31.5.0]` セクションを先頭に追記
- [x] **T10** `benchmarks/v31.5.0.json` — 新規作成
- [x] **T11** `versions/current.md` — 「最新安定版」欄を v31.5.0 に更新

---

## テスト確認

- [x] **T12** `cargo test --bin fav v315000 2>&1 | tail -8` — 3/3 PASS
- [x] **T13** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2440 passed、0 failures）

---

## 完了処理

- [x] **T14** `benchmarks/v31.5.0.json` の `tests_passed` を実測値で更新（2440 — 暫定値と一致）
- [x] **T15** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"31.5.0"`
- [x] `lsp/inlay_hints.rs` に `handle_inlay_hints()` が実装されている
- [x] LSP `initialize` 応答に `"inlayHintProvider": true` が含まれる
- [x] `"textDocument/inlayHint"` ハンドラが LSP サーバに追加されている
- [x] `collect_bind_hints()` が `bind n <- ...` 行をパースしてヒントを生成する
- [x] `collect_bind_hints()` が `bind _ <- ...`（ワイルドカード）をスキップする
- [x] `cargo test v315000` — 3/3 PASS
- [x] `cargo test` — 全件 PASS（0 failures）
- [x] `CHANGELOG.md` に `[v31.5.0]` セクション
- [x] `benchmarks/v31.5.0.json` 存在
- [x] `benchmarks/v31.5.0.json` の `tests_passed` が実測値で更新されていること（2440）
- [x] `versions/current.md` を v31.5.0 に更新
- [x] tasks.md が COMPLETE

---

## コードレビューチェックリスト

- [x] `v315000_tests` に `use super::*` があること（3 件）
- [x] `cargo_toml_version_is_31_4_0` が空スタブになっていること（コメント付き）
- [x] `lsp/inlay_hints.rs` が `pub mod inlay_hints;` で `lsp/mod.rs` に登録されていること
- [x] `InlayHint` 構造体が `#[derive(Serialize)]` を持つこと
- [x] `collect_bind_hints()` が `pub(crate)` であること（テストから直接アクセス可能）
- [x] `find_bind_prefix()` が `bind _` を除外していること（name == "_" チェック）
- [x] `find_type_at()` が Span のオーバーラップを正しく判定していること
- [x] `handle_inlay_hints()` が `type_at` が空でもパニックしないこと
- [x] `collect_bind_hints()` のテストが手動 `type_at` を使って実際にヒントを生成・検証していること（パニックしないだけでなく `": "` 始まりのラベルを確認）
- [x] LSP `initialize` 応答の既存 capability が変更されていないこと
- [x] `"textDocument/inlayHint"` ハンドラが `write_response` + `Ok(false)` で終わること
- [x] CRLF ファイル・マルチバイト文字への対応がないこと（OUT OF SCOPE — LF / ASCII 限定）
- [x] site/ MDX が変更されていないこと（OUT OF SCOPE）
