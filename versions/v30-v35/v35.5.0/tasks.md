# v35.5.0 タスクリスト — `!Effect` 廃止 Phase 2

## ステータス: COMPLETE

> ※ ロードマップ計画（`deploy.fav` 宣言的デプロイ設定）から変更 — 詳細は spec.md「ロードマップとの差異」を参照

## T0: 事前確認

- [x] テスト数: 2645 passed; 0 failed（実測）
- [x] Cargo.toml バージョンが `35.4.0` であることを確認
- [x] `v35400_tests::cargo_toml_version_is_35_4_0` が生きたアサーション（41512行）であることを確認 → T2 でスタブ化実施
- [x] `ast.rs` に `pub enum Effect {` が存在しないことを確認（削除済み）
- [x] `ast.rs` に `effects: Vec<Effect>` が存在しないことを確認
- [x] `parser.rs` に `fn parse_effects_acc` が存在しないことを確認
- [x] `v35500_tests` モジュールが 5 件存在（`changelog_has_v35_5_0` 不足を確認）→ T1 で追加
- [x] `checker.rs` 行 8232 のコメント `effect_registry removed` が実態と乖離を確認 → T1 でコメント修正実施

## T1: driver.rs — v35500_tests モジュール追加

- [x] `v35500_tests` モジュールは既存（5件）
- [x] `cargo_toml_version_is_35_5_0` — スタブ済み（既存）
- [x] `effect_enum_removed_from_ast` — 実装済み（既存）
- [x] `effects_field_removed_from_fn_def` — 実装済み（既存）
- [x] `parse_effects_acc_removed_from_parser` — 実装済み（既存）
- [x] `effect_def_no_longer_registers_in_checker` — 実装済み（既存）
- [x] `changelog_has_v35_5_0` テスト追加（新規追加）
- [x] `checker.rs` 行 8232 のコメントを `effect declarations are no-ops — registration stubbed (effect_registry field retained)` に修正

## T2: バージョン更新（T1 完了後、テスト前に実施）

- [x] `v35400_tests::cargo_toml_version_is_35_4_0` をスタブ化（"stubbed: version bumped to 35.5.0"）
- [x] `fav/Cargo.toml` バージョンを `35.5.0` に更新

## T3: テスト実行

- [x] `cargo test` 全通過 — 2646 passed; 0 failed
- [x] `v35500_tests` の 6 テストが pass

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の `## [v35.5.0]` エントリが存在することを確認（既存）

## T5: ドキュメント更新

- [x] `versions/v30-v35/v35.5.0/tasks.md` を COMPLETE ステータスに更新

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `ast.rs` に `pub enum Effect {` が存在しない | `effect_enum_removed_from_ast` テスト |
| 2 | `ast.rs` に `effects: Vec<Effect>` が存在しない | `effects_field_removed_from_fn_def` テスト |
| 3 | `parser.rs` に `fn parse_effects_acc` が存在しない | `parse_effects_acc_removed_from_parser` テスト |
| 4 | `effect Payment` のパースがエラーなしで通る | `effect_def_no_longer_registers_in_checker` テスト |
| 5 | `cargo test` 全通過（0 failures） | T3 実行結果 ✅ 2646 passed; 0 failed |
| 6 | `CHANGELOG.md` に `[35.5.0]` エントリが存在する | `changelog_has_v35_5_0` テスト ✅ |

## コードレビュー対応（実施後に記録）

| 指摘 | 優先度 | 対応 |
|---|---|---|
| `cargo_toml_version_is_35_5_0` スタブコメントに "v35.0A" という存在しないバージョンが含まれ事実誤認 | MED | `// stubbed: version bumped to 35.6.0` に修正 |
| `effect_enum_removed_from_ast` のコメント「コンパイル通過 = 削除証明」は論理的に誤り | LOW | 単純な確認コメントに修正 |
| `effect_def_no_longer_registers_in_checker` のコメントが `effect_registry` 残存と矛盾 | LOW | `effect_registry field is retained but never written to` に修正 |
