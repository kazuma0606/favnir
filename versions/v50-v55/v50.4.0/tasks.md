# Tasks: v50.4.0 — LSP インレイヒント Phase 1（変数・関数戻り型）

Status: COMPLETE
Date: 2026-07-19

---

## T0 — 事前確認

- [x] `cargo test` 3097 passed, 0 failed を確認（ベース確認）
- [x] `collect_fn_return_hints` が未実装であることを確認（`grep "fn_return" fav/src/lsp/inlay_hints.rs`）
- [x] `textDocument/inlayHint` ハンドラが `mod.rs` に実装済みであることを確認
- [x] `"inlayHintProvider": true` が capabilities に含まれることを確認
- [x] `cargo clippy -- -D warnings` クリーンであることを確認（ベース）

## T1 — `lsp/inlay_hints.rs` — `collect_fn_return_hints` 追加

- [x] `collect_stage_hints` の直後に `pub(crate) fn collect_fn_return_hints` を追加
  - [x] `fn ` プレフィックスがなければスキップ
  - [x] `->` を含む行（明示的戻り型あり）はスキップ
  - [x] fn 名の byte range を計算（`indent_len + 3` = `"fn ".len()`）
  - [x] `line.rfind(')')` で param list 末尾 `)` の位置を取得
  - [x] `find_type_at` で型を検索し、`InlayHint { label: format!(" -> {}", ty.display()) }` を生成
  - [x] `name_end == 0` ガード（関数名が空の場合のスキップ）
- [x] `handle_inlay_hints` に `hints.extend(collect_fn_return_hints(...))` を追加

## T2 — `v504000_tests` モジュール追加

- [x] `v504000_tests` モジュールを `driver.rs` の `v503000_tests` 直前に追加（3 件）
  - [x] `cargo_toml_version_is_50_4_0`: version = "50.4.0" を assert
  - [x] `lsp_inlay_hint_let_binding`: `collect_bind_hints` が `bind count <- 42` に対して `"Int"` を含むラベルのヒントを返すことを assert
  - [x] `lsp_inlay_hint_fn_return`: `collect_fn_return_hints` が `fn double(x: Int) { ... }` に対して ` -> Int` ヒントを返すことを assert

## T3 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"50.4.0"`
- [x] `v503000_tests::cargo_toml_version_is_50_3_0` を削除
- [x] `cargo test` 3099 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v50.4.0 エントリ追加
- [x] `versions/current.md` を v50.4.0（3099 tests）に更新
- [x] `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.4.0 実績を記入
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）
