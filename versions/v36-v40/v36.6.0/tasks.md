# v36.6.0 タスクリスト — E0380〜E0384 スキーマ不整合エラーコード

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v36.1-v37.0.md` の v36.6.0（「E0380〜E0384 スキーマ不整合エラーコード」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2681（v36.5.0 完了時点の実績値））し、実測値をここに記録: 2681
- [x] Cargo.toml バージョンが `36.5.0` であることを確認
- [x] `v36500_tests::cargo_toml_version_is_36_5_0` がライブアサーション（`assert!(cargo.contains("36.5.0"), ...)`）であることを確認
- [x] `driver.rs` に `v36600_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `CHANGELOG.md` に `[v36.6.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `error_catalog.rs` に `E0380` が存在しないことを確認（今回追加）
- [x] `error_catalog.rs` の `ErrorEntry` struct フィールド順（code/title/category/description/example/fix）を確認
- [x] `error_catalog.rs` に `pub fn lookup` が存在することを確認
- [x] `ERROR_CATALOG` の末尾エントリが `E0903` であることを確認（実測値: E0903 at line 599）
- [x] `lib.rs` に `pub mod error_catalog;` が存在することを確認（テストから `crate::error_catalog::` でアクセス可能な前提）
- [x] 既存の `E0501`（`schema field missing`、category: `modules`）・`E0502`（`schema type mismatch`、category: `modules`）が存在することを確認し、新規 E038x（category: `schema`）との用途の違いを認識した
- [x] `versions/current.md` の最新安定版が `v36.5.0`・次バージョンが `v36.6.0` であることを確認

## T1: CHANGELOG.md に [v36.6.0] エントリを追加

- [x] `## [v36.5.0]` の `---` セパレータ直後に `## [v36.6.0]` エントリを挿入
- [x] 日付を `YYYY-MM-DD` 形式の実装当日の日付に変更（2026-07-08）

## T2: error_catalog.rs — E0380〜E0384 追加

- [x] `ERROR_CATALOG` 末尾（`];` の直前）に `// ── E038x: スキーマ不整合 (v36.6.0) ────` コメントを追加
- [x] `ErrorEntry { code: "E0380", title: "schema_field_missing", ... }` を追加
- [x] `ErrorEntry { code: "E0381", title: "schema_type_mismatch", ... }` を追加
- [x] `ErrorEntry { code: "E0382", title: "schema_constraint_violated", ... }` を追加
- [x] `ErrorEntry { code: "E0383", title: "schema_duplicate_key", ... }` を追加
- [x] `ErrorEntry { code: "E0384", title: "schema_extra_field", ... }` を追加
- [x] 全エントリで 6 フィールド（code/title/category/description/example/fix）がすべて埋まっていることを確認

## T3: driver.rs — `v36500_tests::cargo_toml_version_is_36_5_0` をスタブ化

- [x] ライブアサーション → `// stubbed: version bumped to 36.6.0` に変更

## T4: driver.rs — `v36600_tests` モジュールを新規追加

- [x] `v36500_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行する（line 42746）
- [x] `v36500_tests` の閉じ `}` の後に `v36600_tests` モジュールを追加
  - [x] `use crate::error_catalog::{lookup, ERROR_CATALOG};` インポート
  - [x] `cargo_toml_version_is_36_6_0`
  - [x] `changelog_has_v36_6_0`
  - [x] `error_catalog_has_schema_codes`（E0380〜E0384 全コード）
  - [x] `e0380_lookup_returns_correct_title`
  - [x] `e0384_lookup_returns_correct_title`

## T5: バージョン更新（T2〜T4 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `36.6.0` に更新（T2〜T4 すべて完了・コンパイルエラー解消の後）

## T6: テスト実行

- [x] `cargo test` 全通過 — ≥（T0 実測値 + 5）passed; 0 failed（v36600_tests 5 件）— 実測: 2686 passed
- [x] `v36600_tests` の 5 テストがすべて pass
- [x] `error_catalog_has_schema_codes` が pass（E0380〜E0384 全コード存在確認）
- [x] `e0380_lookup_returns_correct_title` が pass（title: `schema_field_missing`）
- [x] `e0384_lookup_returns_correct_title` が pass（title: `schema_extra_field`）

## T7: ドキュメント更新

- [x] `versions/v36-v40/v36.6.0/tasks.md` を COMPLETE ステータスに更新
- [x] `versions/current.md` を v36.6.0（最新安定版）・v36.7.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v36.1-v37.0.md` の v36.6.0 を完了済みにマーク（✅）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `error_catalog.rs` に E0380〜E0384 が定義されている | `error_catalog_has_schema_codes` テスト |
| 2 | `CHANGELOG.md` に `[v36.6.0]` が含まれる | `changelog_has_v36_6_0` テスト |
| 3 | `Cargo.toml` バージョンが `36.6.0` | `cargo_toml_version_is_36_6_0` テスト |
| 4 | `lookup("E0380")` が正しい title を返す | `e0380_lookup_returns_correct_title` テスト |
| 5 | `lookup("E0384")` が正しい title を返す | `e0384_lookup_returns_correct_title` テスト |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2686） | T6 実行結果: 2686 passed, 0 failed ✅ |
