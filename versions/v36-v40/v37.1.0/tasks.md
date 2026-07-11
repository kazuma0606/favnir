# v37.1.0 タスクリスト — 境界付きジェネリクス実用強化

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v37.1-v38.0.md` の v37.1.0（「境界付きジェネリクス実用強化」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2703（v37.0.0 完了時点の実績値））し、実測値をここに記録: 2703
- [x] Cargo.toml バージョンが `37.0.0` であることを確認
- [x] `v37000_tests::cargo_toml_version_is_37_0_0` がライブアサーション（`assert!(cargo.contains("37.0.0"), ...)`）であることを確認し、行番号を記録: 43122
- [x] `v37000_tests` の他 3 テスト（`changelog_has_v37_0_0` / `milestone_has_data_quality_first` / `readme_mentions_data_quality`）はバージョン変更後も pass することを確認（バージョン番号を含まないため影響なし）
- [x] `driver.rs` に `v37100_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `CHANGELOG.md` に `[v37.1.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `runes/generic/` ディレクトリが存在しないことを確認（今回新規作成）
- [x] `fav/src/middle/checker.rs` の `type_implements_bound` 関数を Read で確認し、`"Serialize"` が既存・`"Deserialize"` が欠落していることを確認、行番号を記録: 7599
- [x] `v37000_tests` の閉じ `}` の行番号を確認し、ここに記録: 43151
- [x] `versions/current.md` の最新安定版が `v37.0.0`・次バージョンが `v37.1.0` であることを確認

## T1: CHANGELOG.md に [v37.1.0] エントリを追加

- [x] `## [v37.0.0]` の `---` セパレータ直後に `## [v37.1.0]` エントリを挿入
- [x] 日付を `YYYY-MM-DD` 形式の実装当日の日付に変更（2026-07-09）

## T2: `middle/checker.rs` — `Deserialize` を明示的な有効制約に追加

- [x] T0 で確認した行番号の `"Eq" | "Serialize" | "Clone" => true,` を Read で確認
- [x] `"Eq" | "Serialize" | "Deserialize" | "Clone" => true,` に変更（1 行のみ）
- [x] `cargo build` でコンパイルエラーがないことを確認

## T3: runes/generic/ 新規作成

- [x] `runes/generic/generic.fav` を新規作成（spec.md §2 の内容）
- [x] `runes/generic/rune.toml` を新規作成
  - 注意: rune.toml の存在テストは今バージョンでは追加しない（generic.fav 内容テストのみ）

## T4: driver.rs — `v37000_tests::cargo_toml_version_is_37_0_0` をスタブ化

- [x] ライブアサーション → `// Stubbed: version bumped to 37.1.0` に変更

## T5: driver.rs — `v37100_tests` モジュールを新規追加（T3 完了後）

- [x] `v37000_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行する（行番号: 43151）
- [x] `v37000_tests` の閉じ `}` の後に `v37100_tests` モジュールを追加
  - [x] モジュール先頭に `use super::{build_artifact, exec_artifact_main};` + ローカル `run()` 関数を定義
  - [x] `cargo_toml_version_is_37_1_0`（`include_str!("../Cargo.toml")`）
  - [x] `changelog_has_v37_1_0`（`include_str!("../../CHANGELOG.md")`）
  - [x] `deserialize_constraint_type_checks`（`run()` で `T with Deserialize` の型チェックと実行を確認）
  - [x] `generic_rune_file_exists`（`include_str!("../../runes/generic/generic.fav")` で `"Deserialize"` を確認）

## T6: バージョン更新（T1〜T5 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `37.1.0` に更新（T1〜T5 すべて完了・コンパイルエラー解消の後）

## T7: テスト実行

- [x] `cargo test` 全通過 — ≥ 2707 passed; 0 failed — 実測: 2707 passed
- [x] `v37100_tests` の 4 テストがすべて pass
- [x] `cargo_toml_version_is_37_1_0` が pass
- [x] `changelog_has_v37_1_0` が pass
- [x] `deserialize_constraint_type_checks` が pass
- [x] `generic_rune_file_exists` が pass

## T8: ドキュメント更新

- [x] `versions/v36-v40/v37.1.0/tasks.md` を COMPLETE ステータスに更新
- [x] `versions/current.md` を v37.1.0（最新安定版）・v37.2.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.1.0 を完了済みにマーク（✅）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `37.1.0` | `cargo_toml_version_is_37_1_0` テスト ✅ |
| 2 | `CHANGELOG.md` に `[v37.1.0]` が含まれる | `changelog_has_v37_1_0` テスト ✅ |
| 3 | `T with Deserialize` が型チェックと実行を通る | `deserialize_constraint_type_checks` テスト ✅ |
| 4 | `runes/generic/generic.fav` が `Deserialize` を含む | `generic_rune_file_exists` テスト ✅ |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2707） | 実測: 2707 passed, 0 failed ✅ |
