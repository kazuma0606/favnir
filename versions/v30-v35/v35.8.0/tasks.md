# v35.8.0 タスクリスト — !Effect 廃止完結（LSP / エラーカタログ / MCP / help）

## ステータス: COMPLETE

> コードネーム v35.0C。sprint 作業（lsp/completion.rs・error_catalog.rs・mcp/mod.rs・main.rs 修正、CHANGELOG 追記、v35800_tests pre-existing）は完了済み。
> 本セッションでは half-stub 修正 + Cargo.toml バンプを実施する。
>
> ロードマップ差異: spec.md §「ロードマップとの差異」参照。デプロイ cookbook は後続バージョンで対応。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2646 以上）し、実測値をここに記録: 2646
- [x] Cargo.toml バージョンが `35.7.0` であることを確認
- [x] `v35700_tests::cargo_toml_version_is_35_7_0` がライブアサーション（`assert!(cargo.contains("35.7.0"), ...)`)）であることを確認
- [x] `v35800_tests` モジュールが driver.rs に 5 件存在することを確認
- [x] `v35800_tests::cargo_toml_version_is_35_8_0` が空ボディスタブ（`// stubbed: version bumped to 35.7.0`）であることを確認 → T2 で修正
- [x] `v35900_tests` モジュールが driver.rs にまだ存在しないことを確認（未追加で正常）
- [x] `CHANGELOG.md` に `[v35.8.0]` が含まれることを確認（sprint 完了済み）
- [x] `lsp/completion.rs` に `!Io"` 等の文字列リテラルが存在しないことを確認（sprint 完了済み）
- [x] `error_catalog.rs` の `fix:` フィールドに `!Db` 等が存在しないことを確認（sprint 完了済み）
- [x] ロードマップ由来の `site/content/docs/deploy/` MDX は本バージョンのスコープ外であることを確認（後続バージョンで対応）
- [x] `v35800_tests` モジュールが `v35700_tests` より前に定義されている（逆順）ことを確認済み — 今回の実装では順序は変更しない（スプリント一括構造のため）

## T1: driver.rs — v35700_tests::cargo_toml_version_is_35_7_0 をスタブ化

- [x] ライブアサーション → `// stubbed: version bumped to 35.8.0` に変更

## T2: driver.rs — cargo_toml_version_is_35_8_0 を生きたアサーションに修正

- [x] 空ボディスタブ → `assert!(cargo.contains("35.8.0"), "Cargo.toml must contain version 35.8.0")` に修正
- [x] スタブコメント行（`// stubbed: version bumped to 35.7.0`）を削除

## T3: バージョン更新（T1 完了後）

- [x] `fav/Cargo.toml` バージョンを `35.8.0` に更新

## T4: テスト実行

- [x] `cargo test` 全通過 — N passed; 0 failed（テスト数 ≥ 2646、今回追加テストなし・前バージョンと同数維持）
- [x] `v35800_tests` の 5 テストが pass

## T5: ドキュメント更新

- [x] `versions/v30-v35/v35.8.0/tasks.md` を COMPLETE ステータスに更新
- [x] `versions/current.md` を v35.8.0（最新安定版）・v35.9.0（次バージョン）に更新

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `lsp/completion.rs` に `!Effect` 文字列リテラルが存在しない | `lsp_completion_signatures_no_effect` テスト |
| 2 | `error_catalog.rs` の `fix:` に `!Effect` 構文が存在しない | `error_catalog_fix_no_effect_syntax` テスト |
| 3 | `mcp/mod.rs` のドキュメント文字列に `!Io` 等が存在しない | `mcp_docs_no_effect_annotation` テスト |
| 4 | `CHANGELOG.md` に `[v35.8.0]` が含まれる | `changelog_has_v35_8_0` テスト |
| 5 | `Cargo.toml` バージョンが `35.8.0` | `cargo_toml_version_is_35_8_0` テスト |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2646） | T4 実行結果 |

## spec-reviewer 対応（計画フェーズで適用済み）

| 指摘 | 優先度 | 対応 |
|---|---|---|
| `mcp_docs_no_effect_annotation` テストのパターンが `!Io\\n`（バックスラッシュ+n）で狭すぎる | [HIGH] | `!Io` / `!Http` / `!Db` に修正（driver.rs を編集） |
| driver.rs モジュール定義順が昇順でない（v35800 が v35700 より前） | [HIGH] | スキップ（スプリント一括構造のため）。plan.md・tasks.md に注記追加 |
| T0 の v35900_tests 確認表現が誤り（存在しないモジュールを確認） | [MED] | 「未追加で正常」に書き換え |
| plan.md Step 3 の依存関係表現が弱い（Step 1 のみ言及） | [MED] | 「Step 1 および Step 2 の完了後」に修正 |
| T0 テスト数のハードコード（2646 固定） | [MED] | 実測値記録欄に変更 |
| ロードマップ由来のデプロイ MDX ファイル確認項目が T0 に欠落 | [LOW] | T0 に「スコープ外確認」項目を追加 |
| `lsp_completion_signatures_no_effect` が `!Csv`/`!Sys` を検出しない | [LOW] | plan.md 注意事項に記載（テスト変更はスコープ外） |

## コードレビュー対応（実施後に記録）

| 指摘 | 優先度 | 対応 |
|---|---|---|
| `error_catalog_fix_no_effect_syntax` に `!Io`/`!Http`/`!Llm`/`!Gen`/`!Cache`/`!Queue` が未チェック | [MED] | 6種を追加（driver.rs 編集） |
| `mcp_docs_no_effect_annotation` がインラインコメント内の `!Io` を区別できない | [MED] | 記録のみ（現時点で誤検出なし・mcp/mod.rs に `!Io` 残存なし） |
| `lsp_completion_signatures_no_effect` に `!Stream"`/`!Snowflake"`/`!Postgres"` が未チェック | [LOW] | 3種を追加（driver.rs 編集） |
| `error_catalog.rs` に E0316〜E0318 の欠番 | [LOW] | 記録のみ（将来実装時の注意事項） |
