# v38.3.0 タスクリスト — `fav generate --from csv` 強化

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v38.1-v39.0.md` の v38.3.0（「`fav generate --from csv` 強化」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2750（v38.2.0 完了時点の実績値））し、実測値をここに記録: 2750
- [x] Cargo.toml バージョンが `38.2.0` であることを確認
- [x] `v38200_tests::cargo_toml_version_is_38_2_0` がライブアサーション（`assert!(cargo.contains("38.2.0"), ...)`）であることを確認し、行番号を記録: 43656
- [x] `v38200_tests` の他 5 テスト（`changelog_has_v38_2_0` / `generate_sql_fn_exists` / `sql_select_to_stage` / `sql_join_to_stage` / `sql_where_to_stage`）はバージョン変更後も pass することを確認（`[v38.2.0]` エントリは CHANGELOG から削除しないため影響なし）
- [x] `driver.rs` に `v38300_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `v38200_tests` の閉じ `}` の行番号を確認し、ここに記録: 43704
- [x] `CHANGELOG.md` に `[v38.3.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `fav/src/generate_csv.rs` が存在しないことを確認（今回新規作成）
- [x] `main.rs` に `pub(crate) mod generate_csv;` が存在しないことを確認（今回追加）
- [x] `main.rs` の `match fmt` ブロック内 `_ =>` catch-all（line 2441 付近）の行番号を確認し、ここに記録: 2442（`"csv"` アームをその直前に挿入）
- [x] `versions/current.md` の最新安定版が `v38.2.0`・次バージョンが `v38.3.0` であることを確認
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.3.0 が未完了（✅ なし）であることを確認（T9 で更新）
- [x] `roadmap-v38.1-v39.0.md` の v38.3.0 テスト件数欄が「2 件」（T9 で 4 件に更新予定）であることを確認（「4 件」と既に記載されていれば T9 のロードマップ数値更新は不要）

## T1: CHANGELOG.md に [v38.3.0] エントリを追加

- [x] `## [v38.2.0]` の `---` セパレータ直後に `## [v38.3.0]` エントリを挿入
- [x] 日付を `2026-07-10` に設定
- [x] セパレータが `—`（全角ダッシュ U+2014）形式であることを確認

## T2: `fav/src/generate_csv.rs` 新規作成

- [x] spec.md §1 の内容で `fav/src/generate_csv.rs` を新規作成
- [x] `pub fn csv_to_favnir(csv_path: &str) -> Result<String, String>` を含む（パス traversal ガード: `..` を含む場合 `Err`）
- [x] `pub(crate) fn csv_to_favnir_from_str(csv_str: &str) -> Result<String, String>` を含む（テスト用）
- [x] `fn parse_headers` — 空 CSV の場合 `Err` を返す
- [x] `fn generate_from_headers` — 出力に `type Row`・`schema`・`expect` を含む

## T3: `fav/src/main.rs` — `pub(crate) mod generate_csv;` 追加

- [x] T2（generate_csv.rs 作成）が完了していることを確認してから着手
- [x] Read で `pub(crate) mod generate_sql;` の行番号を確認
- [x] `pub(crate) mod generate_sql;` の直後に `pub(crate) mod generate_csv;` を追加

## T4: `fav/src/main.rs` — `"csv"` 分岐追加 + `_ =>` メッセージ更新

- [x] T2・T3 が完了していることを確認してから着手
- [x] Read で `_ =>` catch-all の行番号を確認（T0 で記録済み、参考値: 2441）
- [x] `_ =>` の直前に `"csv"` アームを追加（spec.md §2 のコードブロックに従う）
  - [x] `csv_path = args.get(4)` で CSV ファイルパスを取得（未指定時 `eprintln!` + `process::exit(1)`）
  - [x] `generate_csv::csv_to_favnir(csv_path)` を呼び出し
  - [x] `Ok(output)` → `println!`、`Err(e)` → `eprintln!` + `process::exit(1)`
- [x] `_ =>` catch-all のメッセージを `"Supported: sql"` → `"Supported: sql, csv"` に更新

## T5: `driver.rs` — `v38200_tests::cargo_toml_version_is_38_2_0` をスタブ化

- [x] Read で `cargo_toml_version_is_38_2_0` の行番号を確認（T0 で記録済み）
- [x] ライブアサーション → `// Stubbed: version bumped to 38.3.0 — assertion intentionally removed` に変更
- [x] **注意:** `changelog_has_v38_2_0` / `generate_sql_fn_exists` / `sql_*` テストはスタブ化しない
- [x] スタブ形式が前バージョンのスタブと一致していることを確認

## T6: `driver.rs` — `v38300_tests` モジュールを新規追加（T1・T2 完了後に実施）

- [x] T1（CHANGELOG 追加）と T2（generate_csv.rs 作成）が完了していることを確認してから着手
- [x] `v38200_tests` の閉じ `}` の行番号（T0 で記録済み、参考値: 43704）を Read で特定してから Edit を実行
- [x] `v38200_tests` の閉じ `}` の後に `v38300_tests` モジュールを追加（4 テスト）
  - [x] `cargo_toml_version_is_38_3_0`
  - [x] `changelog_has_v38_3_0`
  - [x] `generate_csv_fn_exists`（`include_str!("generate_csv.rs")` で `pub fn csv_to_favnir` + `pub(crate) fn csv_to_favnir_from_str` を確認）
  - [x] `csv_to_favnir_basic`（`crate::generate_csv::csv_to_favnir_from_str("id,name\n1,Alice")` が `type Row` + `schema` + `expect` を含む）

## T7: バージョン更新（T1〜T6 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `38.3.0` に更新

## T8: テスト実行

- [x] T7（Cargo.toml 更新）が完了していることを確認してから着手
- [x] `cargo test` 全通過 — ≥ 2754 passed; 0 failed — 実測: 2754 passed, 0 failed
- [x] `v38300_tests` の 4 テストがすべて pass
- [x] `cargo_toml_version_is_38_3_0` が pass
- [x] `changelog_has_v38_3_0` が pass
- [x] `generate_csv_fn_exists` が pass
- [x] `csv_to_favnir_basic` が pass

## T9: ドキュメント更新（T8 完了後）

- [x] `versions/v36-v40/v38.3.0/tasks.md` を COMPLETE ステータスに更新（T0〜T9 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v38.3.0（最新安定版）・v38.4.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.3.0 を完了済みにマーク（✅）し、テスト件数を 4 件に更新
- [x] roadmap の v38.3.0 行を Read で確認し ✅ が含まれることをここに記録: ✅ 確認: ### v38.3.0 — `fav generate --from csv` 強化 ✅
- [x] roadmap の v38.3.0 行を Read で確認し「4 件」が含まれることをここに記録: テスト件数 4 件確認: **完了条件**: Rust テスト 4 件（2754 tests passed, 0 failed）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `generate_csv.rs` に `pub fn csv_to_favnir` が含まれる | `generate_csv_fn_exists` テスト ✅ |
| 2 | CSV から `type Row` + `schema` + `expect` ブロックが生成される | `csv_to_favnir_basic` テスト ✅ |
| 3 | `CHANGELOG.md` に `[v38.3.0]` が含まれる | `changelog_has_v38_3_0` テスト ✅ |
| 4 | `Cargo.toml` バージョンが `38.3.0` | `cargo_toml_version_is_38_3_0` テスト ✅ |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2754） | 実測: 2754 passed, 0 failed ✅ |
| 6 | `roadmap-v38.1-v39.0.md` の v38.3.0 が ✅ かつテスト件数が 4 件 | 更新済み ✅ |
