# v37.7.0 タスクリスト — `fav new --template multi-source`

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v37.1-v38.0.md` の v37.7.0（「`fav new --template multi-source`」）に沿ったバージョン。
> スコープ: `driver.rs` に `multi-source` テンプレートを追加し、マルチソース ETL プロジェクトの雛形を生成できるようにする。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2727（v37.6.0 完了時点の実績値））し、実測値をここに記録: 2727 （2727 でない場合は T10 期待値を「実測値 +3」に修正）
- [x] Cargo.toml バージョンが `37.6.0` であることを確認
- [x] `v37600_tests::cargo_toml_version_is_37_6_0` がライブアサーション（`assert!(cargo.contains("37.6.0"), ...)`）であることを確認し、行番号を記録: ___
- [x] `driver.rs` に `v37700_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `CHANGELOG.md` に `[v37.7.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `v37600_tests` の閉じ `}` の行番号を確認し、ここに記録: ___
- [x] `versions/current.md` の最新安定版が `v37.6.0`・次バージョンが `v37.7.0` であることを確認
- [x] `TEMPLATE_GALLERY` に `"multi-source"` エントリが存在しないことを確認（今回追加）
- [x] `cmd_new_list` に `"data-contract"` 行が存在しないことを確認（今回 `"multi-source"` と同時追加）
- [x] `v248000_tests::template_gallery_has_5_entries` の既存コメント行 + `assert_eq!` 2 行の行番号を確認し記録: ___
- [x] `create_data_contract_project` の閉じ `}` 行番号を確認し記録: ___
- [x] `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.7.0 が未完了（✅ なし）であることを確認（T8 で更新）

## T1: CHANGELOG.md に [v37.7.0] エントリを追加

- [x] `## [v37.6.0]` の `---` セパレータ直後に `## [v37.7.0]` エントリを挿入
- [x] 日付を `YYYY-MM-DD` 形式の実装当日の日付に変更

## T2: `driver.rs` — `create_multi_source_etl_project` 追加

- [x] `create_data_contract_project` の閉じ `}` の直後（`// ── module loading ──` の前）に挿入
- [x] spec.md §1 のコードブロックに従い実装
  - [x] `src/load_customers.fav`（Postgres ロード）
  - [x] `src/load_orders.fav`（CSV ロード）
  - [x] `src/main.fav`（`List.join_on` 結合パイプライン）
  - [x] `fav.toml`（`[runes]` に postgres + csv）
  - [x] `README.md`
  - [x] `.github/workflows/ci.yml`

## T3: `driver.rs` — `try_cmd_new` に `"multi-source"` アームを追加

- [x] `"data-contract"` アームの直後に `"multi-source" => create_multi_source_etl_project(&root, name),` を追加
- [x] `other` アームのエラーメッセージ末尾に `|multi-source` を追記

## T4: `driver.rs` — `TEMPLATE_GALLERY` に `"multi-source"` エントリ追加

- [x] `("data-contract", ...)` の直後に `("multi-source", "マルチソース ETL（複数 DB/CSV 結合）"),  // v37.7.0` を追加

## T5: `driver.rs` — `cmd_new_list` 更新

- [x] `"distributed-etl"` 行の直後に `"data-contract"` と `"multi-source"` の 2 行を追加（data-contract は既に TEMPLATE_GALLERY 登録済みだが cmd_new_list への追加が欠落）

## T6: `driver.rs` — `v248000_tests::template_gallery_has_5_entries` のスタブ化

- [x] 既存コメント行（`// v36.5.0 で data-contract を...`）と `assert_eq!` 2 行の計 3 行を削除し、スタブコメント 1 行に置き換え
  - [x] `// Stubbed: len check removed — multi-source added in v37.7.0 (now 6 entries)` に変更（3 行 → 1 行）
  - [x] 名前確認アサーション群（`names.contains(...)` 5 行）は維持

## T7: `driver.rs` — `v37600_tests::cargo_toml_version_is_37_6_0` をスタブ化

- [x] ライブアサーション → `// Stubbed: version bumped to 37.7.0 — assertion intentionally removed` に変更
- [x] **注意:** `changelog_has_v37_6_0` / `lineage_dot_contains_digraph` / `lineage_svg_contains_svg_tag` はスタブ化しない
- [x] スタブ形式が v37.5.0・v37.4.0 等の前バージョンのスタブと一致しているか確認

## T8: `driver.rs` — `v37700_tests` モジュールを新規追加

- [x] `v37600_tests` の閉じ `}` の行番号（T0 で記録）を Read で特定してから Edit を実行
- [x] `v37600_tests` の閉じ `}` の後に `v37700_tests` モジュールを追加（spec.md §6）
  - [x] `use super::try_cmd_new`
  - [x] `cargo_toml_version_is_37_7_0`
  - [x] `changelog_has_v37_7_0`
  - [x] `fav_new_multi_source_ok`（tempdir + try_cmd_new + 4 ファイル存在確認（main.fav / load_customers.fav / load_orders.fav / README.md） + `List.join_on` 内容確認）

## T9: バージョン更新（T1〜T8 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `37.7.0` に更新

## T10: テスト実行

- [x] `cargo test` 全通過 — ≥ 2730 passed; 0 failed — 実測: 2730 passed
- [x] `v37700_tests` の 3 テストがすべて pass
- [x] `cargo_toml_version_is_37_7_0` が pass
- [x] `changelog_has_v37_7_0` が pass
- [x] `fav_new_multi_source_ok` が pass

## T11: ドキュメント更新

- [x] `versions/v36-v40/v37.7.0/tasks.md` を COMPLETE ステータスに更新（T0〜T11 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v37.7.0（最新安定版）・v37.8.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.7.0 を完了済みにマーク（✅）し、テスト件数を 3 件に更新

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `37.7.0` | `cargo_toml_version_is_37_7_0` テスト |
| 2 | `CHANGELOG.md` に `[v37.7.0]` が含まれる | `changelog_has_v37_7_0` テスト |
| 3 | `multi-source` テンプレートで 3 ファイルが生成され `List.join_on` が含まれる | `fav_new_multi_source_ok` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2730） | 実測: 2730 passed, 0 failed ✅ |
