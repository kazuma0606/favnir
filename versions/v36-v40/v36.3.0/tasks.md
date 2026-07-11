# v36.3.0 タスクリスト — W025 `schema_mismatch` lint ルール

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v36.1-v37.0.md` の v36.3.0（「W025 schema_mismatch lint ルール」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2666 以上）し、実測値をここに記録: 2671
- [x] Cargo.toml バージョンが `36.2.0` であることを確認
- [x] `v36200_tests::cargo_toml_version_is_36_2_0` がライブアサーション（`assert!(cargo.contains("36.2.0"), ...)`）であることを確認
- [x] driver.rs に `v36300_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `CHANGELOG.md` に `[v36.3.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `lint.rs` に `W025` が存在しないことを確認（今回追加）
- [x] `lint.rs` に `check_w025_schema_mismatch` が存在しないことを確認（今回追加）
- [x] `versions/current.md` の最新安定版が `v36.2.0`・次バージョンが `v36.3.0` であることを確認
- [x] `LintError` の正確なフィールド名・コンストラクタを `lint.rs` で確認（`code`/`message`/`span` or `LintError::new()`）
- [x] `Expr::FieldAccess` の正確な variant を `ast.rs` で確認
- [x] `Expr::Closure` のボディフィールドの型を `ast.rs` で確認（variant 名は `Lambda` ではなく `Closure`）
- [x] `Block.expr` フィールドの型を `ast.rs` で確認（`Expr` / `Box<Expr>` / `Option<Box<Expr>>` のいずれか）

## T1: CHANGELOG.md に [v36.3.0] エントリを追加

- [x] `## [v36.2.0]` の `---` セパレータ直後に `## [v36.3.0]` エントリを挿入

## T2: lint.rs — ヘルパー関数追加

- [x] `collect_schema_fields(program)` 関数を追加（`SchemaDef` 名 → フィールド名リスト の Map を返す）
- [x] `collect_field_accesses_expr` 関数を追加（`Expr::FieldAccess` を再帰的に収集）
- [x] `collect_field_accesses_stmt` 関数を追加（`Stmt` 全 variant を exhaustive match）
- [x] `collect_field_accesses` 関数を追加（`Block` を走査）
- [x] `Stmt::Expect` アームが `collect_field_accesses_stmt` に含まれていることを確認

## T3: lint.rs — check_w025_schema_mismatch 追加

- [x] `check_w025_schema_mismatch(program, errors)` 関数を追加（W021 ノーオプの後に配置）
- [x] `schema_fields` が空なら即 `return` するガードを含む

## T4: lint_program への呼び出し追加

- [x] `lint_program` 内の `check_w021_pure_fn_calls_effectful` 呼び出しの直後に `check_w025_schema_mismatch` を追加

## T5: driver.rs — v36200_tests::cargo_toml_version_is_36_2_0 をスタブ化

- [x] ライブアサーション → `// stubbed: version bumped to 36.3.0` に変更

## T6: driver.rs — v36300_tests モジュールを新規追加

- [x] driver.rs ファイル末尾（`v36200_tests` モジュールの閉じ `}` の後）に `v36300_tests` モジュールを追加
  - [x] `cargo_toml_version_is_36_3_0`
  - [x] `changelog_has_v36_3_0`
  - [x] `w025_in_lint_rs`
  - [x] `w025_schema_mismatch_fires`
  - [x] `w025_schema_mismatch_silent`

## T7: バージョン更新（T2・T3・T4・T5・T6 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `36.3.0` に更新（T2〜T6 すべて完了・コンパイルエラー解消の後）

## T8: テスト実行

- [x] `cargo test` 全通過 — ≥ 2671 passed; 0 failed（2666 + v36300_tests 5 件）
- [x] `v36300_tests` の 5 テストがすべて pass
- [x] `w025_schema_mismatch_fires` が pass（W025 が発行されること）
- [x] `w025_schema_mismatch_silent` が pass（W025 が発行されないこと）

## T9: ドキュメント更新

- [x] `versions/v36-v40/v36.3.0/tasks.md` を COMPLETE ステータスに更新
- [x] `versions/current.md` を v36.3.0（最新安定版）・v36.4.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v36.1-v37.0.md` の v36.3.0 を完了済みにマーク

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `lint.rs` に `check_w025_schema_mismatch` が含まれる | `w025_in_lint_rs` テスト |
| 2 | `CHANGELOG.md` に `[v36.3.0]` が含まれる | `changelog_has_v36_3_0` テスト |
| 3 | `Cargo.toml` バージョンが `36.3.0` | `cargo_toml_version_is_36_3_0` テスト |
| 4 | W025 が未定義フィールドアクセスで発行される | `w025_schema_mismatch_fires` テスト |
| 5 | 正常なフィールドアクセスでは W025 が発行されない | `w025_schema_mismatch_silent` テスト |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2671） | T8 実行結果 |
