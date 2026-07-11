# v35.6.0 タスクリスト — ctx 構文統一 + Production Ready 宣言

## ステータス: COMPLETE

> ※ ロードマップ計画（`fav deploy status` + `fav rollback`）から変更 — 詳細は spec.md 参照

## T0: 事前確認

- [x] テスト数: 2646 passed; 0 failed（実測）
- [x] Cargo.toml バージョンが `35.5.0` であることを確認
- [x] `v35500_tests::cargo_toml_version_is_35_5_0` がスタブ済み（`// stubbed: version bumped to 35.6.0`）を確認
- [x] `v35600_tests` モジュールが driver.rs に 5 件存在することを確認
- [x] `cargo_toml_version_is_35_6_0` が半スタブ（`assert!(cargo.contains("35."), ...)`）＋コメント `// Stubbed: version bumped to 35.7.0 in v35.0B` を確認 → T1 で修正
- [x] `ctx-syntax-guide.mdx` に E0374 と ctx: AppCtx が含まれることを確認
- [x] `ctx-syntax-guide.mdx` が 6 セクション構成であることを確認（CHANGELOG の記述より）
- [x] `README.md` に AppCtx が含まれることを確認
- [x] `MILESTONE.md` に Production Ready が含まれることを確認
- [x] `CHANGELOG.md` に [v35.6.0] が含まれることを確認
- [x] `site/content/` MDX の `!Effect` は散文テキスト内のみ（```favnir ブロック外）— スプリント中の変換は完了済みを確認

## T1: driver.rs — `cargo_toml_version_is_35_6_0` を生きたアサーションに修正

- [x] 半スタブ → `assert!(cargo.contains("35.6.0"), "Cargo.toml must contain version 35.6.0")` に修正
- [x] コメント行（"Stubbed: version bumped to 35.7.0 in v35.0B"）を削除

## T2: バージョン更新（T1 完了後）

- [x] `fav/Cargo.toml` バージョンを `35.6.0` に更新

## T3: テスト実行

- [x] `cargo test` 全通過 — 2646 passed; 0 failed（テスト数 ≥ 前バージョン 2646 ✓）
- [x] `v35600_tests` の 5 テストが pass

## T4: CHANGELOG 確認

- [x] `## [v35.6.0]` エントリが存在することを確認（既存）
- [x] エントリ内容が実装内容（ctx-syntax-guide 整備・Production Ready 宣言）と一致を確認

## T5: ドキュメント更新

- [x] `versions/v30-v35/v35.6.0/tasks.md` を COMPLETE ステータスに更新
- [x] `versions/current.md` を v35.6.0（最新安定版）・v35.7.0（次バージョン）に更新

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `MILESTONE.md` に `"Production Ready"` が含まれる | `milestone_has_production_ready` テスト |
| 2 | `ctx-syntax-guide.mdx` に E0374 と `ctx: AppCtx` が含まれる | `ctx_syntax_guide_has_e0374_section` テスト |
| 3 | `README.md` に `"AppCtx"` が含まれる | `readme_ctx_syntax_documented` テスト |
| 4 | `CHANGELOG.md` に `"[v35.6.0]"` が含まれる | `changelog_has_v35_6_0` テスト |
| 5 | `site/content/` MDX に `!Effect` が残存しない | T0 grep 確認（スプリント中実施済みの事後確認） |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 前バージョン） | T3 実行結果 |

## コードレビュー対応（実施後に記録）

| 指摘 | 優先度 | 対応 |
|---|---|---|
| （実施後に記録） | — | — |
