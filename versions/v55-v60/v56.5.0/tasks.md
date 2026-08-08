# Tasks — v56.5.0 — OR パターン + パターンガード強化

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.5.0 セクションを確認
- [x] ベーステスト数 3235（v56.4.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が `56.4.0` であることを確認（更新前）
- [x] `Pattern::Or(Vec<Pattern>, Span)` が `ast.rs` L298 付近に存在することを確認（新規追加不要）
- [x] `parse_match_arm` が OR パターンを処理することを確認（`parser.rs` L3569-3578）
- [x] `MatchArm.guard: Option<Box<Expr>>` が `ast.rs` に存在することを確認
- [x] `checker.rs` が `Pattern::Or` を処理することを確認（L4206, L10386 付近）
- [x] `check_unreachable_patterns` が `lint.rs` に存在しないことを確認（新規追加対象）
- [x] `v56500_tests` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x] 最新の lint W コードが W036 であることを確認（W037 が次番）
- [x] `error_catalog.rs` に W コードが存在しないことを確認（E コード専用 — W037 は登録しない）
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` が `"56.4.0"` を期待していることを確認（更新対象）

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `56.5.0` に更新（56.4.0 から変更）
- [x] T2: `fav/src/lint.rs` — W037 `check_unreachable_patterns` 追加
  - [x] W036 コメントブロック直後に `pub fn check_unreachable_patterns(program: &Program) -> Vec<LintError>` を追加
  - [x] `fn check_block_for_unreachable(block: &Block, errors: &mut Vec<LintError>)` を追加
  - [x] `fn check_stmt_for_unreachable(stmt: &Stmt, errors: &mut Vec<LintError>)` を追加
    - [x] `Stmt::Bind(b)` → `b.expr`
    - [x] `Stmt::Chain(c)` → `c.expr`
    - [x] `Stmt::Expr(e)` → `e`
    - [x] `Stmt::Yield(y)` → `y.expr`
    - [x] `Stmt::Return(r)` → `r.expr`
    - [x] `Stmt::ForIn(f)` → `f.iter` + `f.body`（両方）
    - [x] `Stmt::Forall(f)` → `f.body`
    - [x] `Stmt::Expect(e)` → `e.target`（ExpectStmt は `target: Box<Expr>` + `rules: Vec<Expr>`）
  - [x] `fn check_expr_for_unreachable(expr: &Expr, errors: &mut Vec<LintError>)` を追加
    - [x] `Expr::Match` アームを走査し、catch-all（ガードなし `_` / bind）後の直後アームに W037 を発行
    - [x] リテラル重複を `HashSet` で検出し W037 を発行
    - [x] ガード付き catch-all（`_ if cond`）を catch-all 扱いしないことを確認
  - [x] `fn pattern_is_catch_all(pat: &Pattern) -> bool` を追加（`Wildcard` | `Bind` のみ true）
  - [x] `fn pattern_lit_key(pat: &Pattern) -> Option<String>` を追加（`Lit` のみ Some）
- [x] T3: `fav/src/lint.rs` — `lint_program` へ統合
  - [x] `errors.extend(check_unreachable_patterns(program));` を W036 直後に追加
- [x] T4: `fav/src/driver.rs` — `v56500_tests` モジュールを `v56400_tests` の直前に追加
  - [x] `match_or_pattern`: `Ok(_) | Err(_) => "handled"` が Checker エラーなし
  - [x] `match_or_with_guard`: `"yes" | "ok" if true => "positive"` が Checker エラーなし
  - [x] `w037_unreachable_after_wildcard`: `check_unreachable_patterns` が W037 を発行することを確認（**必須**）
- [x] T5: `fav/src/driver.rs` — バージョンチェックテスト更新
  - [x] **`v56300_tests` モジュール内**の `cargo_toml_version_is_56_3_0` テストの期待値を `"56.4.0"` → `"56.5.0"` に更新

---

## テスト・検証

- [x] T6: `cargo build` でコンパイルエラーがないことを確認
- [x] T7: `cargo test` 全通過（**3238 tests passed, 0 failed**）
  - [x] `v56500_tests::match_or_pattern` ok
  - [x] `v56500_tests::match_or_with_guard` ok
  - [x] `v56500_tests::w037_unreachable_after_wildcard` ok
  - [x] 既存 3235 件全通過
- [x] T8: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T9: `CHANGELOG.md` に v56.5.0 エントリを追加（version: `56.4.0 → 56.5.0`）
- [x] T10: `versions/current.md` を v56.5.0 / 3238 tests に更新
- [x] T11: `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.5.0 実績を COMPLETE に更新
  - [x] 実績行に `3235 + 3 = 3238 tests passed, 0 failed（2026-07-26）` を追記
  - [x] 「`PatternOr` AST ノード — 新規追加」記述を「`Pattern::Or` は v17.2.0 実装済み — W037 lint 追加と回帰テスト追加」に修正
- [x] T12: `versions/roadmap/roadmap-v55.1-v60.0.md` の v56.5.0 実績欄も COMPLETE に更新

---

## 完了確認

- [x] `match_or_pattern` pass
- [x] `match_or_with_guard` pass
- [x] `w037_unreachable_after_wildcard` pass（W037 が実際に発行される）
- [x] **3238 tests passed, 0 failed**
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `lint.rs` に `check_unreachable_patterns`（pub）が追加されている
- [x] `check_stmt_for_unreachable` が全 Stmt バリアント（8 件）を網羅している
- [x] `check_unreachable_patterns` が `lint_program` から呼ばれている
- [x] catch-all 後のアームに W037 が発行されることを自動テストで確認済み
- [x] ガード付き catch-all（`_ if cond`）が W037 を発行しないことを確認（`arm.guard.is_none()` チェック実装済み）
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値が `"56.5.0"` になっている
- [x] `error_catalog.rs` は変更していない（W コードは lint.rs のみ）
- [x] `CHANGELOG.md` に v56.5.0 エントリが追加されている（version: `56.4.0 → 56.5.0`）
- [x] `versions/current.md` が v56.5.0 / 3238 tests を反映
- [x] T11 / T12 のロードマップ更新（実績 COMPLETE + PatternOr 記述修正）が完了している

## 実装注意事項（後続バージョン向けメモ）

- `Item::StageDef` は ast.rs に存在しない — `Item::TrfDef(td)` が stage 定義に相当する
- `Block` は `tail` フィールドなし — `expr: Box<Expr>` が tail 式
- `ExpectStmt` は `expr` フィールドなし — `target: Box<Expr>` + `rules: Vec<Expr>`
- Favnir match アームの区切り文字は `=>` (FatArrow)、`->` (Arrow) は関数戻り型
- `LintError::new(code, message, span)` を使用（`LintError { code: String, ... }` は型不一致）
