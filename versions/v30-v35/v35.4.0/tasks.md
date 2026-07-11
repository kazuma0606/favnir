# v35.4.0 タスクリスト — `!Effect` アノテーション廃止 Phase 1

## ステータス: COMPLETE

## T0: 事前確認

- [x] `cargo test 2>&1 | tail -3` でテスト数と failures=0 を実測確認 — 2644 passed; 0 failed
- [x] Cargo.toml バージョンが `35.3.0` であることを確認
- [x] `v35400_tests` モジュールが driver.rs に既存（4件）— spec と照合済み、`changelog_has_v35_4_0` が不足
- [x] `v35300_ci_tests::cargo_toml_version_is_35_3_0` が生きたアサーション（41472行）を確認 — T2 でスタブ化実施
- [x] `parser.rs` に `E0374` エラーが実装済みを確認（v34.8A 実装済み）
- [x] `lint.rs` に `check_w022_deprecated_effect_annotation` が存在しないことを確認
- [x] `error_catalog.rs` に `E0374` エントリが存在することを確認

## T1: driver.rs — v35400_tests モジュール追加

- [x] `v35400_tests` モジュールは既存（4件）
- [x] `cargo_toml_version_is_35_4_0` — スタブ済み（既存）
- [x] `effect_annotation_is_parse_error_e0374` — 実装済み（既存）
- [x] `ctx_appctx_bypasses_effect_check` — 実装済み（既存）
- [x] `w022_lint_removed` — 実装済み（既存）
- [x] `e0374_in_error_catalog` — 実装済み（既存）
- [x] `changelog_has_v35_4_0` テスト追加（新規追加）

## T2: バージョン更新（T1 完了後、テスト前に実施）

- [x] `v35300_ci_tests::cargo_toml_version_is_35_3_0` をスタブ化（"stubbed: version bumped to 35.4.0"）
- [x] `fav/Cargo.toml` バージョンを `35.4.0` に更新

## T3: テスト実行

- [x] `cargo test` 全通過 — 2645 passed; 0 failed
- [x] `v35400_tests` の 6 テストが pass

## T4: CHANGELOG 更新（T3 完了後に実施）

- [x] `CHANGELOG.md` に `## [v35.4.0]` エントリが存在することを確認（既存）

## T5: ドキュメント更新

- [x] `versions/v30-v35/v35.4.0/tasks.md` を COMPLETE ステータスに更新

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `fn f() -> Int !Io { 1 }` のパースが E0374 エラーを返す | `effect_annotation_is_parse_error_e0374` テスト |
| 2 | `ctx: AppCtx` 関数が E0107 を発生させない | `ctx_appctx_bypasses_effect_check` テスト |
| 3 | `check_w022_deprecated_effect_annotation` が lint.rs に存在しない | `w022_lint_removed` テスト |
| 4 | `error_catalog.rs` に `E0374` エントリが存在する | `e0374_in_error_catalog` テスト |
| 5 | `cargo test` 全通過（0 failures） | T3 実行結果 ✅ 2645 passed; 0 failed |
| 6 | `CHANGELOG.md` に `[35.4.0]` エントリが存在する | `changelog_has_v35_4_0` テスト ✅ |

## コードレビュー対応（実施後に記録）

| 指摘 | 優先度 | 対応 |
|---|---|---|
| `cargo_toml_version_is_35_4_0` stub が時系列不正（35.4.0 現在なのに "bumped to 35.5.0"） | MED | 生きたアサーション `assert!(cargo.contains("35.4.0"))` に修正 |
| `ctx_appctx_bypasses_effect_check` で E0107 コードの意味が catalog と乖離 | LOW | `errors.is_empty()` チェックに変更し意図コメントを追加 |
| `w022_lint_removed` テストでコメント残存許容の説明なし | LOW | 「コメント残存は許容、関数実装削除のみを確認」コメントを追加 |
