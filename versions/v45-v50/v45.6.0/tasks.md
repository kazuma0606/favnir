# Tasks: v45.6.0 — エラーメッセージ改善 Phase 1

Status: COMPLETE
Date: 2026-07-16

---

## T0 — 事前確認

- [x] `cargo test` 2980 passed, 0 failed を確認

## T1 — `error_catalog.rs`: `ErrorEntry` に `suggestion` フィールド追加

- [x] `ErrorEntry` struct に `pub suggestion: Option<&'static str>` を追加
- [x] `ERROR_CATALOG` の全エントリに `suggestion: None,` を追加（コンパイルエラー防止）
  - 注意: `ErrorEntry` は `Default` 未実装のため `..Default::default()` は不可、全エントリへの明示追加が唯一の方法
- [x] 主要エントリ（E0101 / E0102 / E0103）に有意な suggestion テキストを設定

## T2 — `checker.rs`: `Expr::Apply` 引数数不一致に hint 追加

- [x] `Expr::Apply` → `Type::Fn` の引数数チェック箇所（line ~4724）を特定
- [x] `type_error` を `type_error_h` に変更し、関数名と期待引数数を含む hint を追加
- [x] `Type::Error` の返却を維持していることを確認

## T3 — `driver.rs`: テストモジュール + バージョン更新

- [x] `fav/Cargo.toml` version → `45.6.0`
- [x] `v456000_tests` モジュール追加（2件）
  - [x] `check_with_hints` ヘルパー定義（errors の code + hints を返す）
  - [x] `e0102_suggestion_similar_name` — `ordr()` で E0102 + hints に "order" が含まれる
  - [x] `e0101_suggestion_arg_count` — `add(1)` で E0101 + hints に "2" が含まれる

## T4 — テスト＆完了

- [x] `cargo test` 2982 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v45.6.0 エントリ追加
- [x] `versions/current.md` を v45.6.0（2982 tests）に更新
- [x] tasks.md を COMPLETE に更新（T0〜T4 全チェック）

## コードレビュー指摘と対応

- [HIGH] code-reviewer: `ErrorEntry.suggestion` が `cmd_explain_error` で表示されていない（デッドデータ）→ `cmd_explain_error` に `Suggestion` セクション追加
- [MED] code-reviewer: `type_error_to_diag` が `TypeError.hints` を JSON 出力に含めない → `CheckDiagnostic` に `hints: Vec<String>` フィールド追加（`skip_serializing_if = "Vec::is_empty"`）
- [MED] code-reviewer: E0215 カタログ記述と実 emit コード E0101 のミスマッチ → 既存バグのためスコープ外、tasks.md に記録のみ
- [LOW] code-reviewer: `default_suggestion` とカタログの二重管理、hint 文体不統一、E0101 説明ミスリーディング → 設計上の負債として将来バージョンで対応
- [HIGH] spec-reviewer: `suggestion` 型 `Option<String>` vs `Option<&'static str>` 不一致 → `&'static str` の理由を spec.md に明記、ロードマップ修正
- [HIGH] spec-reviewer: テスト数 2979 → 実績 2982 → ロードマップ修正
- [HIGH] spec-reviewer: テスト名 `e0001_suggestion_similar_name` が実際は E0102 → `e0102_suggestion_similar_name` に修正（spec/plan/tasks/ロードマップ）
- [HIGH] spec-reviewer: テスト名 `e0007_suggestion_arg_count` が実際は E0101 → `e0101_suggestion_arg_count` に修正（spec/plan/tasks/ロードマップ）
- [MED] spec-reviewer: `Default` 未実装のため `..Default::default()` 不可 → tasks.md T1 に注意書き追加
- [LOW] spec-reviewer: `site/` 変更有無が未記載 → spec.md の変更しないファイルに追記
