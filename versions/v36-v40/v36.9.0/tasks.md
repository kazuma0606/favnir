# v36.9.0 タスクリスト — v37.0 前調整・安定化

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v36.1-v37.0.md` の v36.9.0（「v37.0 前調整・安定化」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2695（v36.8.0 完了時点の実績値））し、実測値をここに記録: 2695
- [x] Cargo.toml バージョンが `36.8.0` であることを確認
- [x] `v36800_tests::cargo_toml_version_is_36_8_0` がライブアサーション（`assert!(cargo.contains("36.8.0"), ...)`）であることを確認し、行番号を記録: 43030
- [x] `driver.rs` に `v36900_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `CHANGELOG.md` に `[v36.9.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `lint.rs` の W025 `format!` 文字列が `E0380` を含まないことを確認（今回追加）
- [x] W025 に関係する既存テスト（`v36300_tests` 等）を確認し、メッセージ文字列を直接アサートしていないことを確認（`v36300_tests` は `code` のみアサート、影響なし）
- [x] `cmd_validate` に既存のサマリー行がないことを確認（今回追加）
- [x] `v36800_tests` の閉じ `}` の行番号を確認し、ここに記録: 43086
- [x] `versions/current.md` の最新安定版が `v36.8.0`・次バージョンが `v36.9.0` であることを確認
- [x] `site/content/docs/` 直下に `data-quality.mdx` が存在しないことを確認（今回新規作成）
- [x] `site/content/docs/` ディレクトリ自体が存在することを確認（Glob で確認）

## T1: CHANGELOG.md に [v36.9.0] エントリを追加

- [x] `## [v36.8.0]` の `---` セパレータ直後に `## [v36.9.0]` エントリを挿入
- [x] 日付を `YYYY-MM-DD` 形式の実装当日の日付に変更（2026-07-09）

## T2: lint.rs — W025 メッセージに E0380 参照追加

- [x] `check_w025_schema_mismatch` 内の `format!` を更新
  - [x] `(available: {})` の後に ` [see also: E0380 schema_field_missing]` を追加
  - [x] 変更は 1 行のみであることを確認

## T3: driver.rs — `cmd_validate` サマリー行追加

- [x] `if has_errors { process::exit(1); }` の直後にサマリー出力ブロックを挿入
  - [x] `schema_defs.len()` と `schema_defs.iter().map(|sd| sd.fields.len()).sum::<usize>()` を使用
  - [x] `println!("Validated: {} schema(s), {} field(s) checked", ...)` の形式

## T4: site/content/docs/data-quality.mdx 新規作成

- [x] `site/content/docs/data-quality.mdx` を spec.md の内容に従って作成
- [x] フロントマターに `title` と `description` を含める

## T5: driver.rs — `v36800_tests::cargo_toml_version_is_36_8_0` をスタブ化

- [x] ライブアサーション → `// Stubbed: version bumped to 36.9.0` に変更

## T6: driver.rs — `v36900_tests` モジュールを新規追加

- [x] `v36800_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行する（行番号: 43086）
- [x] `v36800_tests` の閉じ `}` の後に `v36900_tests` モジュールを追加
  - [x] `cargo_toml_version_is_36_9_0`
  - [x] `changelog_has_v36_9_0`
  - [x] `w025_message_references_e0380`（`include_str!("lint.rs")` で確認）
  - [x] `validate_summary_line_added`（`include_str!("driver.rs").contains("Validated: {} schema(s)")` で確認）

## T7: バージョン更新（T2〜T6 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `36.9.0` に更新（T2〜T6 すべて完了・コンパイルエラー解消の後）

## T8: テスト実行

- [x] `cargo test` 全通過 — ≥ 2699 passed; 0 failed — 実測: 2699 passed
- [x] `v36900_tests` の 4 テストがすべて pass
- [x] `cargo_toml_version_is_36_9_0` が pass
- [x] `changelog_has_v36_9_0` が pass
- [x] `w025_message_references_e0380` が pass
- [x] `validate_summary_line_added` が pass

## T9: ドキュメント更新

- [x] `versions/v36-v40/v36.9.0/tasks.md` を COMPLETE ステータスに更新
- [x] `versions/current.md` を v36.9.0（最新安定版）・v37.0.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v36.1-v37.0.md` の v36.9.0 を完了済みにマーク（✅）

## コードレビュー指摘対応（実装後）

| 優先度 | 指摘内容 | 対応 |
|---|---|---|
| [HIGH] | GE エクスポート失敗時にサマリーが先行出力されユーザーが成功と誤認する | サマリー `println!` を GE エクスポートブロックの後に移動、コメント番号も修正 |
| [HIGH] | `validate_summary_line_added` テストが `include_str!` 自己参照により常に PASS | `concat!` マクロで検索文字列を結合し、テストソース内のリテラルと一致しない形式に変更 |
| [MED] | コメント番号 `// 6.` が `// 5.` より前に出現していた | 移動により解消 |
| [MED] | `data-quality.mdx` フロントマターに `order` / `category` が欠落、値が未クォート | `order: 60`, `category: "ガイド"` を追加、値をクォート形式に統一 |

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | W025 メッセージに `E0380 schema_field_missing` が含まれる | `w025_message_references_e0380` テスト ✅ |
| 2 | `cmd_validate` がサマリー行を出力する | `validate_summary_line_added` テスト ✅ |
| 3 | `CHANGELOG.md` に `[v36.9.0]` が含まれる | `changelog_has_v36_9_0` テスト ✅ |
| 4 | `Cargo.toml` バージョンが `36.9.0` | `cargo_toml_version_is_36_9_0` テスト ✅ |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2699） | 実測: 2699 passed, 0 failed ✅ |
