# v42.2.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2880（前バージョン 2877 + 3）
**実績テスト数**: 2880（v42200_tests 3/3 PASS）

---

## T0 — 事前確認

- [x] `cargo test` が 2877 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `42.1.0` であることを確認
- [x] `v42100_tests::cargo_toml_version_is_42_1_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録（line 44650）
- [x] `v42100_tests` の閉じ `}` の行番号を確認し記録（line 44678）
- [x] `ast.rs` に `CepExpr` が存在しないことを確認
- [x] `CepClause` が `event: String` フィールドを持つことを確認（line 913）
- [x] `parser.rs` に `parse_cep_expr` が存在しないことを確認
- [x] `fmt.rs` / `checker.rs` / `lint.rs` で `CepClause.event` フィールドを参照するコードが存在しないことを確認

---

## T1 — `ast.rs` 更新

- [x] `CepExpr` enum を `CepClause` 構造体の直前に追加（`#[derive(Debug, Clone)]` 付き）
  - `Event(String)` / `Seq(Vec<CepExpr>)` / `Any(Vec<CepExpr>)` / `Not(Box<CepExpr>)` 4 バリアント
- [x] `CepClause.event: String` を `CepClause.expr: CepExpr` に変更

---

## T2 — `parser.rs` 更新

- [x] `parse_cep_expr()` を `parse_cep_pattern_def()` の直前に追加
  - `seq` は `TokenKind::Seq`（予約キーワード）のため `peek() == &TokenKind::Seq` でチェック
  - `any` / `not` は `peek_ident_text()` でチェック
  - その他 → `expect_ident()` → `CepExpr::Event(name)`
- [x] `parse_cep_pattern_def()` の節ループ内を `let expr = self.parse_cep_expr()?;` に置き換え
- [x] `CepClause { expr, within_secs, span }` でコンストラクト

---

## T3 — `driver.rs` 更新

- [x] `v42100_tests::cargo_toml_version_is_42_1_0` をスタブ化
- [x] `v42100_tests::cep_pattern_fields_correct` を AST 変更に合わせて更新
  - `cd.body[0].event` → `CepExpr::Event(ref ev)` パターンマッチ + `assert_eq!(ev, "Login")`
- [x] `v42200_tests` モジュール（3 テスト）を `v42100_tests` の直前に追加
  - `cargo_toml_version_is_42_2_0`（NOTE コメント付き）
  - `cep_seq_parseable`（CepExpr::Seq + within_secs=Some(300) 確認）
  - `cep_any_parseable`（CepExpr::Any + len=3 確認）

---

## T4 — Cargo.toml バージョン bump

- [x] `version = "42.1.0"` → `"42.2.0"`

---

## T5 — CHANGELOG.md 更新

- [x] `[v42.2.0]` エントリを `[v42.1.0]` の直前に追加

---

## T6 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 = 2880 を確認（2877 + 3 件）
- [x] `v42200_tests` 3 件 pass を確認
- [x] `v42100_tests::cep_pattern_fields_correct` が引き続き pass することを確認

---

## T7 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v42.2.0（最新安定版）・v42.3.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v42.1-v43.0.md` の v42.2.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）` を追記）
- [x] `versions/v40-v45/v42.2.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）
- [x] **MILESTONE.md 更新**: 本バージョンは機能リリース（非マイルストーン宣言）のため不要
- [x] **site/ MDX 追加**: cookbook は v42.8.0 で対応予定のため本バージョンは不要

---

## 最終ステータス

- [x] 全タスク完了

## コードレビュー指摘・対応記録

- spec-reviewer [HIGH]: roadmap の v42.2.0〜v43.0.0 の推定テスト数が v42.1.0 実績（2877）ベースでなかったため、v42.2.0〜v43.0.0 の全推定値を修正
- spec-reviewer [MED]: spec.md 影響範囲表に `fmt.rs` / `checker.rs` / `lint.rs` の T0 確認行を追加
- spec-reviewer [MED]: plan.md T2-B に `clause_start` 継続使用コメントを追加
- spec-reviewer [LOW]: spec.md 完了条件表に `not` テスト非スコープ行を追加
- spec-reviewer [LOW]: tasks.md に `site/ MDX` スコープアウト明示を追加

## 実装メモ

- `seq` は Favnir の予約キーワード（`TokenKind::Seq`）のため、`peek_ident_text("seq")` ではなく `peek() == &TokenKind::Seq` を使用する必要があった
- `any` / `not` は予約キーワードではなく `TokenKind::Ident` として処理される（CEP パターン内スコープ限定のため現状問題なし）
- `seq()` / `any()` 空引数チェック（`args.is_empty()` でパーサーエラー）を追加
- `not()` 複数引数チェック（`peek() != RParen` でカスタムエラーメッセージ）を追加
- `cep_seq_parseable` / `cep_any_parseable` テストを args の Event バリアントまで確認するよう強化
