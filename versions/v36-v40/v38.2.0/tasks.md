# v38.2.0 タスクリスト — `fav generate --from sql`

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v38.1-v39.0.md` の v38.2.0（「`fav generate --from sql`」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2744（v38.1.0 完了時点の実績値））し、実測値をここに記録: 2744
- [x] Cargo.toml バージョンが `38.1.0` であることを確認
- [x] `v38100_tests::cargo_toml_version_is_38_1_0` がライブアサーション（`assert!(cargo.contains("38.1.0"), ...)`）であることを確認し、行番号を記録: 43631
- [x] `v38100_tests` の他 2 テスト（`changelog_has_v38_1_0` / `suggest_fn_exists`）はバージョン変更後も pass することを確認（`[v38.1.0]` エントリは CHANGELOG から削除しないため影響なし）
- [x] `driver.rs` に `v38200_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `v38100_tests` の閉じ `}` の行番号を確認し、ここに記録: 43649
- [x] `CHANGELOG.md` に `[v38.2.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `fav/src/generate_sql.rs` が存在しないことを確認（今回新規作成）
- [x] `main.rs` に `mod generate_sql;` が存在しないことを確認（今回追加）
- [x] `main.rs` の `Some("generate")` ブロック内 `other =>` catch-all の行番号を確認し、ここに記録: 2428
- [x] `versions/current.md` の最新安定版が `v38.1.0`・次バージョンが `v38.2.0` であることを確認
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.2.0 が未完了（✅ なし）であることを確認（T9 で更新）
- [x] `roadmap-v38.1-v39.0.md` の v38.2.0 テスト件数欄が「3 件」であることを確認（T9 で 6 件に更新）

## T1: CHANGELOG.md に [v38.2.0] エントリを追加

- [x] `## [v38.1.0]` の `---` セパレータ直後に `## [v38.2.0]` エントリを挿入
- [x] 日付を `2026-07-10` に設定
- [x] セパレータが `—`（全角ダッシュ U+2014）形式であることを確認

## T2: `fav/src/generate_sql.rs` 新規作成

- [x] spec.md §1 の内容で `fav/src/generate_sql.rs` を新規作成
- [x] `pub fn sql_to_favnir(sql: &str) -> String` を含む（JOIN/WHERE/SELECT 分岐）
- [x] `fn generate_load` — 出力に `stage` と `LoadData` を含む
- [x] `fn generate_filter` — 出力に `List.filter`（`filter` を含む）
- [x] `fn generate_join` — 出力に `List.join_on`（`join_on` を含む）

## T3: `fav/src/main.rs` — `mod generate_sql;` 追加

- [x] T2（generate_sql.rs 作成）が完了していることを確認してから着手
- [x] Read で `mod suggest;` の行番号を確認（61 行）
- [x] `mod suggest;` の直後に `mod generate_sql;` を追加

## T4: `fav/src/main.rs` — `Some("--from")` アーム追加

- [x] T2・T3 が完了していることを確認してから着手
- [x] Read で `other =>` catch-all の行番号を確認（T0 で記録済み: 2428）
- [x] `Some("api")` アームの閉じ `}` と `other =>` の間（行 2427〜2428 付近）に `Some("--from")` アームを追加
  - [x] args インデックス: args[2]="--from"（match済み）/ args.get(3)=format / args.get(4)=SQL クエリ
  - [x] `fmt = args.get(3)` で `"sql"` を判定
  - [x] `sql = args.get(4)` でクエリ文字列を取得
  - [x] `generate_sql::sql_to_favnir(sql)` を呼び出して `println!`
  - [x] 不明な format は `eprintln!` + `process::exit(1)`

## T5: `driver.rs` — `v38100_tests::cargo_toml_version_is_38_1_0` をスタブ化

- [x] Read で `cargo_toml_version_is_38_1_0` の行番号を確認（43631）
- [x] ライブアサーション → `// Stubbed: version bumped to 38.2.0 — assertion intentionally removed` に変更
- [x] **注意:** `changelog_has_v38_1_0` / `suggest_fn_exists` はスタブ化しない
- [x] スタブ形式が前バージョンのスタブと一致していることを確認

## T6: `driver.rs` — `v38200_tests` モジュールを新規追加（T1・T2 完了後に実施）

- [x] T1（CHANGELOG 追加）と T2（generate_sql.rs 作成）が完了していることを確認してから着手
- [x] `v38100_tests` の閉じ `}` の行番号（43649）を Read で特定してから Edit を実行
- [x] `v38100_tests` の閉じ `}` の後に `v38200_tests` モジュールを追加（6 テスト）
  - [x] `cargo_toml_version_is_38_2_0`
  - [x] `changelog_has_v38_2_0`
  - [x] `generate_sql_fn_exists`（`include_str!("generate_sql.rs")` で `pub fn sql_to_favnir` を確認）
  - [x] `sql_select_to_stage`（`crate::generate_sql::sql_to_favnir("SELECT id, name FROM users")` が `stage` or `Load` を含む）
  - [x] `sql_join_to_stage`（`crate::generate_sql::sql_to_favnir("SELECT ... JOIN ...")` が `join` or `join_on` を含む）
  - [x] `sql_where_to_stage`（`crate::generate_sql::sql_to_favnir("SELECT id FROM users WHERE active = true")` が `filter` or `Filter` を含む）

## T7: バージョン更新（T1〜T6 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `38.2.0` に更新

## T8: テスト実行

- [x] T7（Cargo.toml 更新）が完了していることを確認してから着手
- [x] `cargo test` 全通過 — ≥ 2750 passed; 0 failed — 実測: 2750 passed, 0 failed ✅
- [x] `v38200_tests` の 6 テストがすべて pass
- [x] `cargo_toml_version_is_38_2_0` が pass
- [x] `changelog_has_v38_2_0` が pass
- [x] `generate_sql_fn_exists` が pass
- [x] `sql_select_to_stage` が pass
- [x] `sql_join_to_stage` が pass
- [x] `sql_where_to_stage` が pass

## T9: ドキュメント更新（T8 完了後）

- [x] `versions/v36-v40/v38.2.0/tasks.md` を COMPLETE ステータスに更新（T0〜T9 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v38.2.0（最新安定版）・v38.3.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.2.0 を完了済みにマーク（✅）し、テスト件数を 6 件に更新
- [x] roadmap の v38.2.0 行を Read で確認し ✅ と「6 件」が含まれることをここに記録: ✅ 確認済み（2750 tests passed）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `generate_sql.rs` に `pub fn sql_to_favnir` が含まれる | `generate_sql_fn_exists` テスト ✅ |
| 2 | SELECT SQL が stage を含む出力に変換される | `sql_select_to_stage` テスト ✅ |
| 3 | JOIN SQL が join_on を含む出力に変換される | `sql_join_to_stage` テスト ✅ |
| 4 | WHERE SQL が filter を含む出力に変換される | `sql_where_to_stage` テスト ✅ |
| 5 | `CHANGELOG.md` に `[v38.2.0]` が含まれる | `changelog_has_v38_2_0` テスト ✅ |
| 6 | `Cargo.toml` バージョンが `38.2.0` | `cargo_toml_version_is_38_2_0` テスト ✅ |
| 7 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2750） | 実測: 2750 passed, 0 failed ✅ |
| 8 | `roadmap-v38.1-v39.0.md` の v38.2.0 が ✅ かつテスト件数が 6 件 | T9 後に目視確認 ✅ |
