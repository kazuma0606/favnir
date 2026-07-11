# v36.8.0 タスクリスト — `fav schema diff`

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v36.1-v37.0.md` の v36.8.0（「fav schema diff」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2689（v36.7.0 完了時点の実績値））し、実測値をここに記録: 2689
- [x] Cargo.toml バージョンが `36.7.0` であることを確認
- [x] `v36700_tests::cargo_toml_version_is_36_7_0` がライブアサーション（`assert!(cargo.contains("36.7.0"), ...)`）であることを確認（line 42839）
- [x] `driver.rs` に `v36800_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `driver.rs` に `schema_diff` が存在しないことを確認（今回追加）
- [x] `CHANGELOG.md` に `[v36.8.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `TypeExpr` の全バリアントを確認し、`type_expr_kind` ヘルパーが全バリアントをカバーすることを確認
- [x] `Span` が `file/line/col` を含む `PartialEq` を持つことを確認（`type_expr_kind` 使用の根拠）
- [x] `SchemaDef` が `#[derive(Clone)]` で所有権移動できることを確認
- [x] main.rs にトップレベルの `Some("schema")` アームが存在しないことを確認（競合なし）
- [x] `driver.rs` に `load_file` 関数が存在することを確認
- [x] `v36700_tests` の閉じ `}` の行番号を確認し、ここに記録: 43021
- [x] `versions/current.md` の最新安定版が `v36.7.0`・次バージョンが `v36.8.0` であることを確認

## T1: CHANGELOG.md に [v36.8.0] エントリを追加

- [x] `## [v36.7.0]` の `---` セパレータ直後に `## [v36.8.0]` エントリを挿入
- [x] 日付を `YYYY-MM-DD` 形式の実装当日の日付に変更（2026-07-08）

## T2: driver.rs — `type_expr_kind` ヘルパーと `schema_diff` 追加

- [x] `// ── fav contract check (v36.5.0)` セクションの後に `// ── fav schema diff (v36.8.0)` セクションを追加
- [x] `fn type_expr_kind(ty: &crate::ast::TypeExpr) -> String` をプライベートヘルパーとして追加（TypeExpr 全バリアントをカバー）
- [x] `pub fn schema_diff(old_src: &str, new_src: &str, old_file: &str, new_file: &str) -> Vec<String>` を追加
  - [x] 純粋関数（`load_file` 不使用）
  - [x] 関数内で `use crate::ast::{Item, SchemaDef};` を両方ローカル宣言（`Item` のみでは型推論が失敗）
  - [x] 削除スキーマ / 追加スキーマ / フィールド差分（追加・削除・型変更）をすべて検出
  - [x] 型比較は `type_expr_kind(old_ty) != type_expr_kind(new_ty)` で行う（Span 除外）

## T3: driver.rs — `cmd_schema_diff` 追加

- [x] `schema_diff` の `}` の直後に `pub fn cmd_schema_diff(old_file: Option<&str>, new_file: Option<&str>)` を追加
- [x] 引数欠落時は `eprintln!` + `process::exit(1)`
- [x] `load_file` でソース読み込み → `schema_diff` 呼び出し → 結果を `println!` で出力

## T4: main.rs — `Some("schema")` ルーティング追加

- [x] `use driver::{ ... }` に `cmd_schema_diff` を追加
- [x] `Some("contract") =>` アームの直後に `Some("schema") =>` アームを追加（トップレベルの match アーム）
  - [x] `Some("diff")` サブコマンドで `cmd_schema_diff(old, new)` を呼び出す
  - [x] 不明サブコマンドの場合は `eprintln!` + `process::exit(1)`
- [x] HELP 定数に `schema diff <old.fav> <new.fav>` の説明を追加

## T5: driver.rs — `v36700_tests::cargo_toml_version_is_36_7_0` をスタブ化

- [x] ライブアサーション → `// Stubbed: version bumped to 36.8.0` に変更

## T6: driver.rs — `v36800_tests` モジュールを新規追加

- [x] `v36700_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行する（line 43021）
- [x] `v36700_tests` の閉じ `}` の後に `v36800_tests` モジュールを追加
  - [x] `use super::schema_diff;` インポート
  - [x] `cargo_toml_version_is_36_8_0`
  - [x] `changelog_has_v36_8_0`
  - [x] `schema_diff_detects_added_field`（`+ amount` と `backward-compatible` を検証）
  - [x] `schema_diff_detects_removed_field`（`- status` と `BREAKING` を検証）
  - [x] `schema_diff_no_changes`（`no changes` を検証）
  - [x] `schema_diff_detects_type_changed_field`（`~ id`、`BREAKING`、`String`、`Int` を検証）

## T7: バージョン更新（T2〜T6 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `36.8.0` に更新（T2〜T6 すべて完了・コンパイルエラー解消の後）

## T8: テスト実行

- [x] `cargo test` 全通過 — ≥ 2692 passed; 0 failed — 実測: 2695 passed
- [x] `v36800_tests` の 6 テストがすべて pass
- [x] `schema_diff_detects_added_field` が pass
- [x] `schema_diff_detects_removed_field` が pass
- [x] `schema_diff_no_changes` が pass
- [x] `schema_diff_detects_type_changed_field` が pass

## T9: ドキュメント更新

- [x] `versions/v36-v40/v36.8.0/tasks.md` を COMPLETE ステータスに更新
- [x] `versions/current.md` を v36.8.0（最新安定版）・v36.9.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v36.1-v37.0.md` の v36.8.0 を完了済みにマーク（✅）

## コードレビュー指摘対応（実装後）

| 優先度 | 指摘内容 | 対応 |
|---|---|---|
| [MED] | 削除フィールドに型が表示されない（追加フィールドと非対称） | `"  - {}: {}  (BREAKING: removed)"` に変更し `type_expr_kind(ty)` を使用 |
| [MED] | `HELP` 定数のバージョンが `v4.12.0` のまま | `v36.8.0` に更新 |
| [LOW] | 型変更ループが削除済みフィールドを除外する理由が不明確 | `// 削除済みフィールドは find が None を返すため自動的に除外される` コメント追加 |
| [LOW] | `schema_diff_detects_removed_field` テストが型文字列に依存しない | `assert!(joined.contains("String"))` を追加し型表示も検証 |

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `schema_diff` が追加フィールドを検出する | `schema_diff_detects_added_field` テスト ✅ |
| 2 | `schema_diff` が削除フィールドを BREAKING として検出する | `schema_diff_detects_removed_field` テスト ✅ |
| 3 | 変更なし時に `no changes` を返す | `schema_diff_no_changes` テスト ✅ |
| 4 | 型変更フィールドを BREAKING として検出する | `schema_diff_detects_type_changed_field` テスト ✅ |
| 5 | `CHANGELOG.md` に `[v36.8.0]` が含まれる | `changelog_has_v36_8_0` テスト ✅ |
| 6 | `Cargo.toml` バージョンが `36.8.0` | `cargo_toml_version_is_36_8_0` テスト ✅ |
| 7 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2692） | 実測: 2695 passed, 0 failed ✅ |
