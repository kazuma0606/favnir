# v36.1.0 タスクリスト — `schema` インライン定義構文

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v36.1-v37.0.md` の v36.1.0（「`schema` リテラル定義構文」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2656 以上）し、実測値をここに記録: 2659
- [x] Cargo.toml バージョンが `36.0.0` であることを確認
- [x] `v36000_tests::cargo_toml_version_is_36_0_0` がライブアサーション（`assert!(cargo.contains("36.0.0"), ...)`）であることを確認
- [x] driver.rs に `v36100_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `CHANGELOG.md` に `[v36.1.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `ast.rs` に `SchemaDef` 構造体が存在しないことを確認（今回追加）
- [x] `ast.rs` に `Item::SchemaDef` が存在しないことを確認（今回追加）
- [x] `versions/current.md` の最新安定版が `v36.0.0`・次バージョンが `v36.1.0` であることを確認

## T1: CHANGELOG.md に [v36.1.0] エントリを追加

- [x] `## [35.3.0]` の直前に `## [v36.1.0]` エントリを挿入（v36.0.0 の直後）

## T2: ast.rs — SchemaDef 構造体と Item::SchemaDef 追加

- [x] `SchemaDef` 構造体（name: String / fields: Vec<(String, TypeExpr)> / span: Span）を追加
- [x] `Item::SchemaDef(SchemaDef)` variant を追加

## T3: parser.rs — schema インライン定義の解析追加

- [x] `parse_schema_def` 関数を追加
- [x] `parse_item` に `schema Ident {` パターンの分岐を追加（`schema "str"` との衝突なし）

## T4: match 文への no-op アーム追加（コンパイルエラー解消）

- [x] `cargo build 2>&1 | grep "error\[E"` でエラー箇所を確認
- [x] `checker.rs` に `Item::SchemaDef(_) => {}` 追加（該当 match があれば）
- [x] `lint.rs` に `Item::SchemaDef(_) => {}` 追加（該当 match があれば）
- [x] `lineage.rs` に `Item::SchemaDef(_) => {}` 追加（該当 match があれば）
- [x] `fmt.rs` の `fn item` に `Item::SchemaDef(_) => String::new()` 追加（返り値型に合わせること）
- [x] `driver.rs` に `Item::SchemaDef(_) => {}` 追加（該当 match があれば）
- [x] その他 `Item` を match する全ファイルで `Item::SchemaDef` を網羅

## T5: driver.rs — v36000_tests::cargo_toml_version_is_36_0_0 をスタブ化

- [x] ライブアサーション → `// stubbed: version bumped to 36.1.0` に変更

## T6: driver.rs — v36100_tests モジュールを新規追加

- [x] driver.rs ファイル末尾（`v36000_tests` モジュールの閉じ `}` の後）に `v36100_tests` モジュールを追加
  - [x] `cargo_toml_version_is_36_1_0`
  - [x] `changelog_has_v36_1_0`
  - [x] `schema_def_item_in_ast`

## T7: バージョン更新（T4 かつ T5 完了後）

- [x] `fav/Cargo.toml` バージョンを `36.1.0` に更新（T4 コンパイルエラー解消・T5 スタブ化の両方が完了してから）

## T8: テスト実行

- [x] `cargo test` 全通過 — 2662 passed; 0 failed（2656 + v36100_tests 3件 + parser roundtrip 3件 = 2662 ✓）
- [x] `v36100_tests` の 3 テストが pass

## T9: ドキュメント更新

- [x] `versions/v36-v40/v36.1.0/tasks.md` を COMPLETE ステータスに更新
- [x] `versions/current.md` を v36.1.0（最新安定版）・v36.2.0（次バージョン）に更新
- [x] `site/` への MDX 追加は今バージョンでは不要（schema インライン構文の詳細ドキュメントは v36.3 `fav validate` 実装後に追加）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `ast.rs` に `SchemaDef` 構造体と `Item::SchemaDef` が含まれる | `schema_def_item_in_ast` テスト |
| 2 | `CHANGELOG.md` に `[v36.1.0]` が含まれる | `changelog_has_v36_1_0` テスト |
| 3 | `Cargo.toml` バージョンが `36.1.0` | `cargo_toml_version_is_36_1_0` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2659） | T8 実行結果 |
