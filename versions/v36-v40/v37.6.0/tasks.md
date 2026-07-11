# v37.6.0 タスクリスト — `fav lineage --graph`

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v37.1-v38.0.md` の v37.6.0（「`fav lineage --graph`」）に沿ったバージョン。
> スコープ: `lineage.rs` に `render_lineage_dot` / `render_lineage_svg` を追加し、DOT・SVG 形式のリネージグラフ出力を実現する。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2723（v37.5.0 完了時点の実績値））し、実測値をここに記録: 2723
- [x] Cargo.toml バージョンが `37.5.0` であることを確認
- [x] `v37500_tests::cargo_toml_version_is_37_5_0` がライブアサーション（`assert!(cargo.contains("37.5.0"), ...)`）であることを確認し、行番号を記録: 43371
- [x] `driver.rs` に `v37600_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `CHANGELOG.md` に `[v37.6.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `v37500_tests` の閉じ `}` の行番号を確認し、ここに記録: 43396
- [x] `versions/current.md` の最新安定版が `v37.5.0`・次バージョンが `v37.6.0` であることを確認
- [x] `lineage.rs` の `render_lineage_d2` 終端行と `sanitize_mermaid_id` 開始行を確認し記録: d2終端=1134 / sanitize開始=1138
- [x] `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.6.0 が未完了（✅ なし）であることを確認（T8 で更新）

## T1: CHANGELOG.md に [v37.6.0] エントリを追加

- [x] `## [v37.5.0]` の `---` セパレータ直後に `## [v37.6.0]` エントリを挿入
- [x] 日付を `YYYY-MM-DD` 形式の実装当日の日付に変更

## T2: `lineage.rs` — `render_lineage_dot` / `render_lineage_svg` 追加

- [x] `render_lineage_d2` の閉じ `}` の直後（`sanitize_mermaid_id` の前）に挿入
- [x] `render_lineage_dot`（spec.md §1）
  - [x] `digraph lineage {` ヘッダー・`rankdir=LR`・`node [shape=box ...]`
  - [x] `sanitize_mermaid_id` を流用してノード ID 生成
  - [x] `[label="<name>\n<kind>"]` ノード定義
  - [x] pipeline steps からエッジ定義（`->` 記法）
- [x] `render_lineage_svg`（spec.md §2）
  - [x] `<svg>` タグ・`<defs>` 内矢印マーカー定義
  - [x] 各 `LineageEntry` を矩形 + テキスト 2 行でレンダリング
  - [x] `name_to_idx` HashMap を使いエッジを `<line>` として描画
  - [x] `</svg>` 閉じタグ

## T3: `driver.rs` — `pub use` 更新

- [x] `render_lineage_d2,` の後に `render_lineage_dot, render_lineage_svg,` を追加

## T4: `driver.rs` — `cmd_explain_lineage` 更新

- [x] `"d2"` アームの直後に `"dot"` / `"svg"` アームを挿入
- [x] `other` アームのエラーメッセージに `dot, svg` を追加

## T5: `driver.rs` — help テキスト更新

- [x] `--format <text|json|mermaid|d2>` → `--format <text|json|mermaid|d2|dot|svg>` に変更

## T6: driver.rs — `v37500_tests::cargo_toml_version_is_37_5_0` をスタブ化

- [x] ライブアサーション → `// Stubbed: version bumped to 37.6.0 — assertion intentionally removed` に変更
- [x] **注意:** `changelog_has_v37_5_0` / `cdc_rune_file_exists` / `cdc_rune_toml_exists` はスタブ化しない

## T7: driver.rs — `v37600_tests` モジュールを新規追加

- [x] `v37500_tests` の閉じ `}` の行番号（T0 で記録）を Read で特定してから Edit を実行
- [x] `v37500_tests` の閉じ `}` の後に `v37600_tests` モジュールを追加（spec.md §6）
  - [x] `use super::{render_lineage_dot, render_lineage_svg}`
  - [x] `use crate::lineage::{LineageReport, LineageEntry, PipelineLineage}`
  - [x] `make_report()` ヘルパー（transformations 2 件 + pipeline 1 件）
  - [x] `cargo_toml_version_is_37_6_0`
  - [x] `changelog_has_v37_6_0`
  - [x] `lineage_dot_contains_digraph`（`digraph lineage` / `LoadUsers` / `SaveResult` / エッジを確認）
  - [x] `lineage_svg_contains_svg_tag`（`<svg` / `LoadUsers` / `SaveResult` / `marker-end` を確認）

## T8: バージョン更新（T1〜T7 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `37.6.0` に更新

## T9: テスト実行

- [x] `cargo test` 全通過 — ≥ 2727 passed; 0 failed — 実測: 2727 passed
- [x] `v37600_tests` の 4 テストがすべて pass
- [x] `cargo_toml_version_is_37_6_0` が pass
- [x] `changelog_has_v37_6_0` が pass
- [x] `lineage_dot_contains_digraph` が pass
- [x] `lineage_svg_contains_svg_tag` が pass

## T10: ドキュメント更新

- [x] `versions/v36-v40/v37.6.0/tasks.md` を COMPLETE ステータスに更新（T0〜T10 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v37.6.0（最新安定版）・v37.7.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.6.0 を完了済みにマーク（✅）し、テスト件数を 4 件に更新

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `37.6.0` | `cargo_toml_version_is_37_6_0` テスト |
| 2 | `CHANGELOG.md` に `[v37.6.0]` が含まれる | `changelog_has_v37_6_0` テスト |
| 3 | `render_lineage_dot` が `digraph lineage` を出力する | `lineage_dot_contains_digraph` テスト |
| 4 | `render_lineage_svg` が `<svg` タグを出力する | `lineage_svg_contains_svg_tag` テスト |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2727） | 実測: 2727 passed, 0 failed ✅ |
