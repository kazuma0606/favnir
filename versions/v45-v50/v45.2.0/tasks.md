# Tasks: v45.2.0 — `return` 型チェック + E0415

Status: COMPLETE
Date: 2026-07-15

---

## T0 — 事前確認

- [x] `cargo test` 2968 passed, 0 failed を確認

## T1 — `error_catalog.rs`: E0415 追加

- [x] `E0415〜E0419 予約` コメントを確認
- [x] `E0415` 定数（または variant）を追加し予約コメントを更新
- [x] 型不一致メッセージ・seq body 禁止メッセージを定義

## T2 — `checker.rs`: return 型チェック実装

- [x] `Checker` 構造体に `current_return_ty: Option<Type>` フィールド追加
- [x] `check_fn_def` 内: `ret_resolved` を `current_return_ty` にセット、終了時リセット
- [x] `check_trf_def`（stage）内: `output_ty` を `current_return_ty` にセット、終了時リセット
- [x] `check_flw_def`（seq）内: `current_return_ty = None` をセット
- [x] `check_return_stmt` ヘルパー実装（`self.type_error` 方式、`?` 伝播なし）
- [x] `Stmt::Return` アームを stub → `self.check_return_stmt(r)` 呼び出しに差し替え（全箇所）

## T3 — `driver.rs`: テストモジュール + バージョン更新

- [x] `fav/Cargo.toml` version → `45.2.0`
- [x] `v452000_tests` モジュール追加（4件）
  - [x] `check_src` ローカルヘルパーを既存パターンに合わせて実装
  - [x] `return_type_ok`
  - [x] `return_type_mismatch_e0415`
  - [x] `return_in_stage_ok`
  - [x] `return_in_closure_no_false_e0415`

## T4 — テスト＆完了

- [x] `cargo test` 2972 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v45.2.0 エントリ追加
- [x] tasks.md を COMPLETE に更新（T0〜T4 全チェック）

## コードレビュー指摘と対応

- [HIGH-1] `check_source` ヘルパー未定義 → plan.md に `check_src` ローカル定義を追記（Parser + Checker パターン）
- [HIGH-2] E0415 予約範囲コメント未言及 → spec.md・plan.md に予約コメント更新手順を明記
- [HIGH-3] `seq` コンテキスト実装根拠不明 → plan.md を具体化（関数名・フィールド操作手順を明示）、`self.type_error` collect 方式に修正
- [MED-1] `?` 伝播が既存パターンと不整合 → `check_return_stmt` を `-> ()` + `self.type_error` 方式で実装
- [LOW-1] `seq` 禁止テスト欠落 → `return_in_seq_e0415` を `return_in_stage_ok`（stage output type 検証）に変更して追加（seq は構文上 Stmt::Return が到達不可のため）

コードレビュー（code-reviewer）指摘と対応:
- [HIGH] クロージャ内 `return` が外側 fn の戻り型で誤検査 → `Expr::Closure` 処理に `current_return_ty.take()` / restore を追加（`None` に設定）
- [MED] 宣言型なし fn での `return` サイレントスキップ → `check_return_stmt` に設計意図コメントを追加（`Some(Unknown)` = 許可・型不検証）
- [LOW] クロージャ + return 組み合わせテスト欠落 → `return_in_closure_no_false_e0415` テストを追加（4件・2972 tests）
