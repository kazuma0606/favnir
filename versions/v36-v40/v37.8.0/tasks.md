# v37.8.0 タスクリスト — Multi-Source cookbook 5 本

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v37.1-v38.0.md` の v37.8.0（「Multi-Source cookbook 5 本」）に沿ったバージョン。
> スコープ: v37.x スプリントで追加した機能（`List.join_on` / CDC Rune / `List.fan_out` / 境界付きジェネリクス / リネージ DOT/SVG）を実用レシピとして 5 本の cookbook MDX に整備する。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2730（v37.7.0 完了時点の実績値））し、実測値をここに記録: 2730 （2730 でない場合は T9 期待値を「実測値 +3」に修正）
- [x] Cargo.toml バージョンが `37.7.0` であることを確認
- [x] `v37700_tests::cargo_toml_version_is_37_7_0` がライブアサーション（`assert!(cargo.contains("37.7.0"), ...)`）であることを確認し、行番号を記録: 43497
- [x] `driver.rs` に `v37800_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `CHANGELOG.md` に `[v37.8.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `v37700_tests` の閉じ `}` の行番号を確認し、ここに記録: 43522
- [x] `versions/current.md` の最新安定版が `v37.7.0`・次バージョンが `v37.8.0` であることを確認
- [x] 以下の 5 ファイルがすべて存在しないことを確認（今回新規作成）:
  - [x] `site/content/cookbook/join-two-tables.mdx`
  - [x] `site/content/cookbook/cdc-postgres-to-warehouse.mdx`
  - [x] `site/content/cookbook/fan-out-by-region.mdx`
  - [x] `site/content/cookbook/generic-etl-function.mdx`
  - [x] `site/content/cookbook/lineage-visualization.mdx`
- [x] `driver.rs` の既存 cookbook `include_str!` パス（行 37323 等）が `../../site/content/cookbook/` 形式であることを確認
- [x] `v37700_tests` の閉じ `}` の直後から driver.rs の末尾まで `v37800_tests` 以降のモジュールが存在しないことを確認
- [x] `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.8.0 が未完了（✅ なし）であることを確認（T7 で更新）

## T1: CHANGELOG.md に [v37.8.0] エントリを追加

- [x] `## [v37.7.0]` の `---` セパレータ直後に `## [v37.8.0]` エントリを挿入
- [x] 日付を `YYYY-MM-DD` 形式の実装当日の日付に変更

## T2: 5 つの cookbook MDX ファイルを作成

- [x] `site/content/cookbook/join-two-tables.mdx`（spec.md §1）
  - [x] frontmatter（title / description）
  - [x] `List.join_on` コード例を含む
- [x] `site/content/cookbook/cdc-postgres-to-warehouse.mdx`（spec.md §2）
  - [x] frontmatter（title / description）
  - [x] `CDC.filter_inserts` コード例を含む
- [x] `site/content/cookbook/fan-out-by-region.mdx`（spec.md §3）
  - [x] frontmatter（title / description）
  - [x] `List.fan_out` コード例を含む
- [x] `site/content/cookbook/generic-etl-function.mdx`（spec.md §4）
  - [x] frontmatter（title / description）
  - [x] `Serialize` キーワードを含む
- [x] `site/content/cookbook/lineage-visualization.mdx`（spec.md §5）
  - [x] frontmatter（title / description）
  - [x] `--format dot` コマンド例を含む

## T3: `driver.rs` — `v37700_tests::cargo_toml_version_is_37_7_0` をスタブ化

- [x] ライブアサーション → `// Stubbed: version bumped to 37.8.0 — assertion intentionally removed` に変更
- [x] **注意:** `changelog_has_v37_7_0` / `fav_new_multi_source_ok` はスタブ化しない
- [x] スタブ形式が前バージョン（v37.6.0 等）のスタブと一致していることを確認

## T4: `driver.rs` — `v37800_tests` モジュールを新規追加（T2 完了後に実施）

- [x] `v37700_tests` の閉じ `}` の行番号（T0 で記録）を Read で特定してから Edit を実行
- [x] `v37700_tests` の閉じ `}` の後に `v37800_tests` モジュールを追加（spec.md §6）
  - [x] imports 不要（`include_str!` のみ）— `generic` に変数名変更（`gen` が Rust 2024 予約語のため）
  - [x] `cargo_toml_version_is_37_8_0`
  - [x] `changelog_has_v37_8_0`
  - [x] `multi_source_cookbook_files_exist`（5 ファイルの `include_str!` + キーワード確認）

## T5: バージョン更新（T1〜T4 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `37.8.0` に更新

## T6: テスト実行

- [x] `cargo test` 全通過 — ≥ 2733 passed; 0 failed — 実測: 2733 passed, 0 failed
- [x] `v37800_tests` の 3 テストがすべて pass
- [x] `cargo_toml_version_is_37_8_0` が pass
- [x] `changelog_has_v37_8_0` が pass
- [x] `multi_source_cookbook_files_exist` が pass

## T7: ドキュメント更新（T6 完了後）

- [x] `versions/v36-v40/v37.8.0/tasks.md` を COMPLETE ステータスに更新（T0〜T7 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v37.8.0（最新安定版）・v37.9.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.8.0 を完了済みにマーク（✅）し、テスト件数を 3 件に更新

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `37.8.0` | `cargo_toml_version_is_37_8_0` テスト |
| 2 | `CHANGELOG.md` に `[v37.8.0]` が含まれる | `changelog_has_v37_8_0` テスト |
| 3 | 5 つの cookbook ファイルがすべて存在し各キーワードを含む | `multi_source_cookbook_files_exist` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2733） | 実測: 2733 passed, 0 failed ✅ |
