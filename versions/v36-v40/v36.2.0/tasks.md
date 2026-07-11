# v36.2.0 タスクリスト — `expect` ブロック

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v36.1-v37.0.md` の v36.2.0（「`expect` ブロック」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2662 以上）し、実測値をここに記録: 2666
- [x] Cargo.toml バージョンが `36.1.0` であることを確認
- [x] `v36100_tests::cargo_toml_version_is_36_1_0` がライブアサーション（`assert!(cargo.contains("36.1.0"), ...)`）であることを確認
- [x] driver.rs に `v36200_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `CHANGELOG.md` に `[v36.2.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `ast.rs` に `ExpectStmt` 構造体が存在しないことを確認（今回追加）
- [x] `ast.rs` に `Stmt::Expect` が存在しないことを確認（今回追加）
- [x] `versions/current.md` の最新安定版が `v36.1.0`・次バージョンが `v36.2.0` であることを確認
- [x] `fav/src/frontend/parser.rs` の `parse_block` に `"expect"` 文字列が既存で存在しないことを確認（今回新規追加）

## T1: CHANGELOG.md に [v36.2.0] エントリを追加

- [x] `## [v36.1.0]` の `---` セパレータ直後に `## [v36.2.0]` エントリを挿入

## T2: ast.rs — ExpectStmt 構造体と Stmt::Expect 追加

- [x] `ExpectStmt` 構造体（target: Box<Expr> / rules: Vec<Expr> / span: Span）を追加
- [x] `Stmt::Expect(ExpectStmt)` variant を追加
- [x] `impl Stmt { fn span() }` に `Stmt::Expect(e) => &e.span` アームを追加

## T3: parser.rs — expect ブロック解析追加

- [x] `parse_expect_stmt` 関数を追加
- [x] `parse_block` の forall 分岐の後に `expect` キーワード分岐を追加

## T4: match 文への no-op アーム追加（コンパイルエラー解消）

- [x] `cargo build 2>&1 | grep "error\[E0004\]" -A5` で全エラー箇所を確認
- [x] `middle/checker.rs` の各 Stmt match に `Stmt::Expect(_) => {}` 追加
- [x] `middle/compiler.rs` の各 Stmt match に `Stmt::Expect(_) => {}` 追加（返り値型注意）
- [x] `middle/ast_lower_checker.rs` の Stmt match に追加
- [x] `lint.rs` の Stmt match に追加（該当があれば）
- [x] `lineage.rs` の Stmt match に追加（該当があれば）
- [x] `fmt.rs` の Stmt match に追加（該当があれば）
- [x] その他 `Stmt` を exhaustive match する全ファイルで `Stmt::Expect` を網羅

## T5: driver.rs — v36100_tests::cargo_toml_version_is_36_1_0 をスタブ化

- [x] ライブアサーション → `// stubbed: version bumped to 36.2.0` に変更

## T6: driver.rs — v36200_tests モジュールを新規追加

- [x] driver.rs ファイル末尾（`v36100_tests` モジュールの閉じ `}` の後）に `v36200_tests` モジュールを追加
  - [x] `cargo_toml_version_is_36_2_0`
  - [x] `changelog_has_v36_2_0`
  - [x] `expect_stmt_in_ast`
- [x] `parser.rs` の `#[cfg(test)] mod tests` ブロック内に `parse_expect_stmt_basic` テストを追加

## T7: バージョン更新（T4・T5・T6 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `36.2.0` に更新（T4 コンパイルエラー解消・T5 スタブ化・T6 v36200_tests 追加の すべてが完了してから）

## T8: テスト実行

- [x] `cargo test` 全通過 — ≥ 2666 passed; 0 failed（2662 + v36200_tests 3件 + parse_expect_stmt_basic 1件）
- [x] `v36200_tests` の 3 テストが pass
- [x] `parse_expect_stmt_basic` が pass

## T9: ドキュメント更新

- [x] `versions/v36-v40/v36.2.0/tasks.md` を COMPLETE ステータスに更新
- [x] `versions/current.md` を v36.2.0（最新安定版）・v36.3.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v36.1-v37.0.md` の v36.2.0 を完了済みにマーク
- [x] site/ への MDX 追加は不要（`expect` 構文ドキュメントは v36.4 `fav validate` 実装後に追加）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `ast.rs` に `ExpectStmt` 構造体と `Stmt::Expect` が含まれる | `expect_stmt_in_ast` テスト |
| 2 | `CHANGELOG.md` に `[v36.2.0]` が含まれる | `changelog_has_v36_2_0` テスト |
| 3 | `Cargo.toml` バージョンが `36.2.0` | `cargo_toml_version_is_36_2_0` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2666） | T8 実行結果 |
