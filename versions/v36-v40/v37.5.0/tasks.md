# v37.5.0 タスクリスト — CDC Rune

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v37.1-v38.0.md` の v37.5.0（「CDC Rune」）に沿ったバージョン。
> スコープ: `runes/cdc/cdc.fav` — Debezium JSON 形式 CDC イベント処理（MySQL / Postgres 対応）。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2719（v37.4.0 完了時点の実績値））し、実測値をここに記録: 2719
- [x] Cargo.toml バージョンが `37.4.0` であることを確認
- [x] `v37400_tests::cargo_toml_version_is_37_4_0` がライブアサーション（`assert!(cargo.contains("37.4.0"), ...)`）であることを確認し、行番号を記録: 43330
- [x] `driver.rs` に `v37500_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `CHANGELOG.md` に `[v37.5.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `v37400_tests` の閉じ `}` の行番号を確認し、ここに記録: 43364
- [x] `versions/current.md` の最新安定版が `v37.4.0`・次バージョンが `v37.5.0` であることを確認
- [x] `runes/cdc/` ディレクトリが存在しないことを確認（今回新規作成）
- [x] `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.5.0 が未完了（✅ なし）であることを確認（T8 で更新）

## T1: CHANGELOG.md に [v37.5.0] エントリを追加

- [x] `## [v37.4.0]` の `---` セパレータ直後に `## [v37.5.0]` エントリを挿入
- [x] 日付を `YYYY-MM-DD` 形式の実装当日の日付に変更

## T2: `runes/cdc/rune.toml` 作成

- [x] `runes/cdc/rune.toml` を spec.md §2 に従い作成（mlflow/rune.toml と同フォーマット）

## T3: `runes/cdc/cdc.fav` 作成

- [x] `runes/cdc/cdc.fav` を spec.md §1 に従い作成
  - [x] `CDC.op_name(op: String) -> String` — "c"→"insert" / "u"→"update" / "d"→"delete" / else→"read"
  - [x] `CDC.is_insert(op: String) -> Bool`
  - [x] `CDC.is_update(op: String) -> Bool`
  - [x] `CDC.is_delete(op: String) -> Bool`
  - [x] `CDC.extract_op(json: String) -> String` — `String.contains` で op フィールド抽出
  - [x] `CDC.filter_inserts(events: List<String>) -> List<String>`
  - [x] `CDC.filter_deletes(events: List<String>) -> List<String>`
  - [x] `{ body }` ブロック構文を使用（`else if` が必要なため）

## T4: driver.rs — `v37400_tests::cargo_toml_version_is_37_4_0` をスタブ化

- [x] ライブアサーション → `// Stubbed: version bumped to 37.5.0 — assertion intentionally removed` に変更
- [x] **注意:** `changelog_has_v37_4_0` はスタブ化しない（CHANGELOG に `[v37.4.0]` エントリが残るため）

## T5: driver.rs — `v37500_tests` モジュールを新規追加

- [x] `v37400_tests` の閉じ `}` の行番号（T0 で記録）を Read で特定してから Edit を実行
- [x] `v37400_tests` の閉じ `}` の後に `v37500_tests` モジュールを追加（spec.md §3）
  - [x] `use super::*` / imports 不要（`include_str!` のみ使用）
  - [x] `cargo_toml_version_is_37_5_0`
  - [x] `changelog_has_v37_5_0`
  - [x] `cdc_rune_file_exists`（`include_str!("../../runes/cdc/cdc.fav")` に `CDC.extract_op` が含まれる）
  - [x] `cdc_rune_toml_exists`（`include_str!("../../runes/cdc/rune.toml")` に `cdc` が含まれる）

## T6: バージョン更新（T1〜T5 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `37.5.0` に更新

## T7: テスト実行

- [x] `cargo test` 全通過 — ≥ 2723 passed; 0 failed — 実測: 2723 passed
- [x] `v37500_tests` の 4 テストがすべて pass
- [x] `cargo_toml_version_is_37_5_0` が pass
- [x] `changelog_has_v37_5_0` が pass
- [x] `cdc_rune_file_exists` が pass
- [x] `cdc_rune_toml_exists` が pass

## T8: ドキュメント更新

- [x] `versions/v36-v40/v37.5.0/tasks.md` を COMPLETE ステータスに更新（T0〜T8 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v37.5.0（最新安定版）・v37.6.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.5.0 を完了済みにマーク（✅）し、テスト件数を 4 件に更新

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `37.5.0` | `cargo_toml_version_is_37_5_0` テスト |
| 2 | `CHANGELOG.md` に `[v37.5.0]` が含まれる | `changelog_has_v37_5_0` テスト |
| 3 | `runes/cdc/cdc.fav` に `CDC.extract_op` が含まれる | `cdc_rune_file_exists` テスト |
| 4 | `runes/cdc/rune.toml` に `cdc` が含まれる | `cdc_rune_toml_exists` テスト |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2723） | 実測: 2723 passed, 0 failed ✅ |
