# v37.0.0 spec — Data Quality First マイルストーン宣言

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v37.0.0 |
| テーマ | Data Quality First マイルストーン宣言・★クリーンアップ |
| 前提 | v36.9.0 COMPLETE — v37.0 前調整・安定化完了 |
| 完了条件 | `v37000_tests` 全テスト pass・`cargo test` 0 failures・`MILESTONE.md` 更新 |

## 背景と目的

v36.1〜v36.9 のスプリントで以下を達成した。本バージョンはこれらを統合して Data Quality First マイルストーンを正式宣言し、v37 世代に移行する。

### 達成内容

| バージョン | 内容 |
|---|---|
| v36.1.0 | `schema Name { ... }` インライン定義構文 |
| v36.2.0 | `expect` ブロック — ビジネスルール宣言構文 |
| v36.3.0 | W025 `schema_mismatch` lint ルール |
| v36.4.0 | `fav validate --schema <file> <csv>` — CSV/Parquet スキーマ検証 |
| v36.5.0 | Data Contract 規約（`contracts/` / `fav contract check`）|
| v36.6.0 | E0380〜E0384 スキーマ不整合エラーコード（`error_catalog.rs`）|
| v36.7.0 | Great Expectations 互換エクスポート（`--export ge`）|
| v36.8.0 | `fav schema diff` — フィールドレベル差分と後方互換性チェック |
| v36.9.0 | v37.0 前調整・安定化（W025↔E0380 連携・validate サマリー）|

## ロードマップとの差異

ロードマップの完了条件「テスト数 4000+」は現時点で 2699 件であり未達。後続スプリントで増加させる。
ロードマップ記載の「GitHub Issues P1/P2 ラベル付きオープンバグ 0 件」条件は Favnir が OSS 公開前のため GitHub Issues が存在しない。本バージョンでは対象外とする。

## 実装スコープ

| ファイル | 変更内容 |
|---|---|
| `MILESTONE.md` | v37.0 Data Quality First 宣言セクション追加 |
| `README.md` | v36.0 / v37.0 マイルストーン宣言を履歴セクションに追加 |
| `CHANGELOG.md` | `## [v37.0.0]` エントリ追加 |
| `fav/src/driver.rs` | `v36900_tests::cargo_toml_version_is_36_9_0` スタブ化 |
| `fav/src/driver.rs` | `v37000_tests` モジュール（4 件）追加 |
| `fav/Cargo.toml` | バージョン `36.9.0` → `37.0.0` |
| ビルドキャッシュ | `cargo clean`（★クリーンアップ） |

## v37000_tests の設計

| テスト名 | 検証内容 | `include_str!` パス |
|---|---|---|
| `cargo_toml_version_is_37_0_0` | Cargo.toml に `"37.0.0"` が含まれる | `"../Cargo.toml"` |
| `changelog_has_v37_0_0` | `CHANGELOG.md` に `[v37.0.0]` が含まれる | `"../../CHANGELOG.md"` |
| `milestone_has_data_quality_first` | `MILESTONE.md` に `"Data Quality First"` が含まれる | `"../../MILESTONE.md"` |
| `readme_mentions_data_quality` | `README.md` に `"Data Quality First"` が含まれる（v36.0 の `"Deployment Story"` も同テストで確認） | `"../../README.md"` |

注意: `readme_mentions_data_quality` では v36.0（`"Deployment Story"`）と v37.0（`"Data Quality"`）両方の追加を確認する。スタブ化は関数本体をコメントに置き換えるため `v36900_tests` のテスト件数は変わらず 4 件のまま（2699 + 4 = 2703）。

## 宣言文

```
schema でテーブル/列の型と制約を宣言し、
expect でビジネスルールをパイプラインに埋め込み、
fav validate でデータを検証できる。
スキーマ不整合は W025 lint で静的に検出され、
違反は E0380〜E0384 として報告される。
fav schema diff で変更の後方互換性を即座に把握できる。

これが Favnir v37.0 — Data Quality First の姿である。
```

## ★クリーンアップ

v37.0.0 は x.0.0 マイルストーンのため `cargo clean` が必須（v31〜v36 の x.0.0 と同規約）。
T2 で `cargo clean` → `cargo test` の順に実施し、クリーンビルドでも全テスト pass を確認する。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `MILESTONE.md` に `"Data Quality First"` が含まれる | `milestone_has_data_quality_first` テスト |
| 2 | `README.md` に `"Data Quality"` が含まれる | `readme_mentions_data_quality` テスト |
| 3 | `CHANGELOG.md` に `[v37.0.0]` が含まれる | `changelog_has_v37_0_0` テスト |
| 4 | `Cargo.toml` バージョンが `37.0.0` | `cargo_toml_version_is_37_0_0` テスト |
| 5 | `cargo clean` 実施済み | T2 実行記録 |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2703） | `cargo test` 実行結果（2699 + 4 = 2703） |
