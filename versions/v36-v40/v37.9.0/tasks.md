# v37.9.0 タスクリスト — v38.0 前調整・安定化

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v37.1-v38.0.md` の v37.9.0（「v38.0 前調整・安定化」）に沿ったバージョン。
> スコープ: `render_lineage_text` サマリー行追加 + `site/content/docs/multi-source-etl.mdx` 作成。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2733（v37.8.0 完了時点の実績値））し、実測値をここに記録: 2733
- [x] Cargo.toml バージョンが `37.8.0` であることを確認
- [x] `v37800_tests::cargo_toml_version_is_37_8_0` がライブアサーション（`assert!(cargo.contains("37.8.0"), ...)`）であることを確認し、行番号を記録: 43529
- [x] `driver.rs` に `v37900_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `CHANGELOG.md` に `[v37.9.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `v37800_tests` の閉じ `}` の行番号を確認し、ここに記録: 43554
- [x] `versions/current.md` の最新安定版が `v37.8.0`・次バージョンが `v37.9.0` であることを確認
- [x] `site/content/docs/multi-source-etl.mdx` が存在しないことを確認（今回新規作成）
- [x] `lineage.rs` の `render_lineage_text` が現時点でサマリー行を含まないことを確認（今回追加）
- [x] `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.9.0 が未完了（✅ なし）であることを確認（T8 で更新）

## T1: CHANGELOG.md に [v37.9.0] エントリを追加

- [x] `## [v37.8.0]` の `---` セパレータ直後に `## [v37.9.0]` エントリを挿入
- [x] 日付を `YYYY-MM-DD` 形式の実装当日の日付に変更
- [x] セパレータが `—`（全角ダッシュ）形式であることを確認

## T2: `lineage.rs` — `render_lineage_text` にサマリー行追加

- [x] Read で `lineage.rs` の `render_lineage_text` 末尾（`out` を返す行 = 行 1066）を確認し、行番号を記録してから Edit を実行
- [x] `render_lineage_text` の `Pipelines:` ブロックの後（`out` を返す直前）にサマリー行を追加
  - [x] `out.push('\n');` を追加
  - [x] `out.push_str(&format!("Total: {} stage(s), {} pipeline(s)\n", report.transformations.len(), report.pipelines.len(),));` を追加
  - [x] 挿入位置が `CrossCloud Flow:` ブロックの後であることを確認
- [x] 変更後も既存の `lineage_dot_contains_digraph` / `lineage_svg_contains_svg_tag` テストに影響しないことを確認（これらは別関数をテストするため影響なし）

## T3: `site/content/docs/multi-source-etl.mdx` 新規作成

- [x] `site/content/docs/multi-source-etl.mdx` を spec.md §2 の内容に従って作成
  - [x] frontmatter（title / description）
  - [x] `List.join_on` コード例を含む
  - [x] `List.fan_out` / `List.fan_in` の説明を含む
  - [x] CDC Rune の説明を含む
  - [x] `fav explain --lineage` の使い方を含む
  - [x] `fav new --template multi-source` の使い方を含む
  - [x] 関連 cookbook へのリンクを含む

## T4: `driver.rs` — `v37800_tests::cargo_toml_version_is_37_8_0` をスタブ化

- [x] ライブアサーション → `// Stubbed: version bumped to 37.9.0 — assertion intentionally removed` に変更
- [x] **注意:** `changelog_has_v37_8_0` / `multi_source_cookbook_files_exist` はスタブ化しない
- [x] スタブ形式が前バージョン（v37.8.0 等）のスタブと一致していることを確認

## T5: `driver.rs` — `v37900_tests` モジュールを新規追加（T3 完了後に実施）

- [x] T3（MDX 作成）が完了し `site/content/docs/multi-source-etl.mdx` が存在することを確認してから着手
- [x] `v37800_tests` の閉じ `}` の行番号（T0 で記録: 43554）を Read で特定してから Edit を実行
- [x] `v37800_tests` の閉じ `}` の後に `v37900_tests` モジュールを追加（spec.md §3）
  - [x] imports 不要（`include_str!` のみ）
  - [x] `cargo_toml_version_is_37_9_0`
  - [x] `changelog_has_v37_9_0`
  - [x] `lineage_text_has_summary_line`
  - [x] `multi_source_etl_doc_exists`

## T6: バージョン更新（T1〜T5 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `37.9.0` に更新

## T7: テスト実行

- [x] `cargo test` 全通過 — ≥ 2737 passed; 0 failed — 実測: 2737 passed, 0 failed
- [x] `v37900_tests` の 4 テストがすべて pass
- [x] `cargo_toml_version_is_37_9_0` が pass
- [x] `changelog_has_v37_9_0` が pass
- [x] `lineage_text_has_summary_line` が pass
- [x] `multi_source_etl_doc_exists` が pass

## T8: ドキュメント更新（T7 完了後）

- [x] `versions/v36-v40/v37.9.0/tasks.md` を COMPLETE ステータスに更新（T0〜T8 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v37.9.0（最新安定版）・v38.0.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.9.0 を完了済みにマーク（✅）し、テスト件数を 4 件に更新

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `37.9.0` | `cargo_toml_version_is_37_9_0` テスト |
| 2 | `CHANGELOG.md` に `[v37.9.0]` が含まれる | `changelog_has_v37_9_0` テスト |
| 3 | `lineage.rs` の `render_lineage_text` がサマリー行を含む | `lineage_text_has_summary_line` テスト |
| 4 | `site/content/docs/multi-source-etl.mdx` が存在し `List.join_on` を含む | `multi_source_etl_doc_exists` テスト |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2737） | 実測: 2737 passed, 0 failed ✅ |
