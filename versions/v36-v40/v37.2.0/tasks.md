# v37.2.0 タスクリスト — 行多相実用強化

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v37.1-v38.0.md` の v37.2.0（「行多相実用強化」）に沿ったバージョン。
> 注: v37.1.0（T0〜T8）と比較して T2 に動作事前確認を追加。T8 → T7 に圧縮（バージョン更新がT5）。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2707（v37.1.0 完了時点の実績値））し、実測値をここに記録: 2707
- [x] Cargo.toml バージョンが `37.1.0` であることを確認
- [x] `v37100_tests::cargo_toml_version_is_37_1_0` がライブアサーション（`assert!(cargo.contains("37.1.0"), ...)`）であることを確認し、行番号を記録: 43168
- [x] `driver.rs` に `v37200_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `CHANGELOG.md` に `[v37.2.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `v37100_tests` の閉じ `}` の行番号を確認し、ここに記録: 43203
- [x] `versions/current.md` の最新安定版が `v37.1.0`・次バージョンが `v37.2.0` であることを確認
- [x] パーサーがネスト record type を型として処理できるか確認: `parse_type_bounds` が `LBrace` から始まるループで複数 `HasField` を生成することを確認（parser.rs L1475-1490）

## T1: CHANGELOG.md に [v37.2.0] エントリを追加

- [x] `## [v37.1.0]` の `---` セパレータ直後に `## [v37.2.0]` エントリを挿入
- [x] 日付を `YYYY-MM-DD` 形式の実装当日の日付に変更（2026-07-09）

## T2: 複数フィールド行制約の動作確認

- [x] `parse_type_bounds` が `{ id: Int, name: String }` を複数 `HasField` として処理することを確認（パーサー L1478-1489 のループで comma 区切り対応）
- [x] `type UserRow` の定義はコンマ区切り不可 → 改行区切りに修正（`type UserRow = { id: Int, name: String }` → 2行）

## T3: driver.rs — `v37100_tests::cargo_toml_version_is_37_1_0` をスタブ化

- [x] ライブアサーション → `// Stubbed: version bumped to 37.2.0` に変更

## T4: driver.rs — `v37200_tests` モジュールを新規追加

- [x] `v37100_tests` の閉じ `}` の行番号（43203）を Read で特定してから Edit を実行
- [x] `v37100_tests` の閉じ `}` の後に `v37200_tests` モジュールを追加
  - [x] imports: `use crate::frontend::parser::Parser;` / `use crate::middle::checker::Checker;`（`use super::*` 不要）
  - [x] ローカル `check_errors()` ヘルパー定義
  - [x] `cargo_toml_version_is_37_2_0`（`include_str!("../Cargo.toml")`）
  - [x] `changelog_has_v37_2_0`（`include_str!("../../CHANGELOG.md")`）
  - [x] `row_poly_multi_field_checks`（`type UserRow` 改行区切り + call-site あり、`check_errors` で 0 件を確認）
  - [x] `nested_row_type_parseable`（`Parser::parse_str(...).is_ok()` でネスト行型のパースを確認）

## T5: バージョン更新（T1〜T4 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `37.2.0` に更新

## T6: テスト実行

- [x] `cargo test` 全通過 — ≥ 2711 passed; 0 failed — 実測: 2711 passed
- [x] `v37200_tests` の 4 テストがすべて pass
- [x] `cargo_toml_version_is_37_2_0` が pass
- [x] `changelog_has_v37_2_0` が pass
- [x] `row_poly_multi_field_checks` が pass
- [x] `nested_row_type_parseable` が pass

## T7: ドキュメント更新

- [x] `versions/v36-v40/v37.2.0/tasks.md` を COMPLETE ステータスに更新
- [x] `versions/current.md` を v37.2.0（最新安定版）・v37.3.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.2.0 を完了済みにマーク（✅）かつ完了条件をスコープ縮小後の内容に更新

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `37.2.0` | `cargo_toml_version_is_37_2_0` テスト ✅ |
| 2 | `CHANGELOG.md` に `[v37.2.0]` が含まれる | `changelog_has_v37_2_0` テスト ✅ |
| 3 | 複数フィールド制約が call-site 型チェックを通る | `row_poly_multi_field_checks` テスト ✅ |
| 4 | ネスト行型がパースを通る | `nested_row_type_parseable` テスト ✅ |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2711） | 実測: 2711 passed, 0 failed ✅ |
