# Tasks: v45.4.0 — `match` 網羅性改善 + W034 / E0416

Status: COMPLETE
Date: 2026-07-16

---

## T0 — 事前確認

- [x] `cargo test` 2974 passed, 0 failed を確認

## T1 — `error_catalog.rs`: E0416 エントリ追加

- [x] `// ── E0416〜E0419: 予約` コメントを `ErrorEntry { code: "E0416", ... }` に置換

## T2 — `checker.rs`: フリー関数追加

- [x] `collect_covered_variants(arms: &[MatchArm]) -> (Vec<String>, bool)` 追加
- [x] `collect_pattern_variants(pat, covered, has_catch_all)` 追加
  - [x] `Wildcard` / `Bind` → `has_catch_all = true`
  - [x] `Variant(name, ..)` → `covered.push(name)`
  - [x] `Or(pats)` → 再帰
  - [x] ガード付きアームはスキップ

## T3 — `checker.rs`: `check_match_arms` 変更

- [x] シグネチャに `value_ctx: bool` 追加
- [x] 末尾に Sum 型 exhaustiveness チェックを追加
  - [x] `Type::Named` の場合のみ `type_defs` lookup
  - [x] `TypeBody::Sum` の場合のみチェック実行
  - [x] `has_catch_all = true` なら網羅とみなしスキップ
  - [x] missing variants あり + `value_ctx = true` → `E0416` type_error
  - [x] missing variants あり + `value_ctx = false` → `W034` type_warning
- [x] `check_expr` 内の呼び出し元 (`Expr::Match`) に `true` を追加

## T4 — `checker.rs`: `check_stmt` 更新

- [x] `Stmt::Expr(e)` で `Expr::Match` を検出して `check_match_arms(..., false)` を呼ぶ
  - [x] `check_expr(e)` を先に呼ばないこと（二重エラー防止）

## T5 — `driver.rs`: テストモジュール + バージョン更新

- [x] `fav/Cargo.toml` version → `45.4.0`
- [x] `v454000_tests` モジュール追加（3件）
  - [x] `check_src` ヘルパー定義（`.code.to_string()` で `Vec<String>` に変換）
  - [x] `match_exhaustive_ok`
  - [x] `match_w034_missing_variant`
  - [x] `match_e0416_value_context`

## T6 — テスト＆完了

- [x] `cargo test` 2977 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v45.4.0 エントリ追加
- [x] tasks.md を COMPLETE に更新（T0〜T6 全チェック）

## コードレビュー指摘と対応

- [HIGH] spec-reviewer: `plan.md` テストコードの `.code.clone()` が `&'static str` → `String` 変換不可 → `.to_string()` に修正
- [HIGH] spec-reviewer: ロードマップのテスト推定数 2975 が実態と不一致 → 2977 に修正（`roadmap-v45.1-v46.0.md`）
- [MED] spec-reviewer: `check_stmt` 二重 `check_expr` リスク未明示 → `plan.md` に警告コメント追記
- [MED] spec-reviewer: E0101 と W034/E0416 共起時の挙動が未定義 → `spec.md` に明記
- [MED] spec-reviewer: `collect_pattern_variants` が `spec.md` に未記載 → `spec.md` §3 に補助関数シグネチャ追記
- [LOW] spec-reviewer: `lint.rs` の変更有無が不明確 → スコープ制限に「変更不要」と明記
- [実装中] Favnir sum 型構文は `| Red | Green | Blue`（先頭 `|`）、match アームは `=>` であり `->` は不可 → テストコード修正
- [実装中] `self/checker.fav` の `infer_expr` に `EArmG` アーム欠落 → 追加（exhaustiveness チェックが正しく動作）
- [実装中] `self/compiler.fav` に 6 箇所の非網羅 match（`token_eq`, `token_to_string`, `free_names_expr`, `binop_bc`, `expr_uses`）→ 欠落バリアント追加
