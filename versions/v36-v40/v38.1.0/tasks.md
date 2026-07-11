# v38.1.0 タスクリスト — `fav suggest`

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v38.1-v39.0.md` の v38.1.0（「`fav suggest`」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2741（v38.0.0 完了時点の実績値））し、実測値をここに記録: 2741
- [x] Cargo.toml バージョンが `38.0.0` であることを確認
- [x] `v38000_tests::cargo_toml_version_is_38_0_0` がライブアサーション（`assert!(cargo.contains("38.0.0"), ...)`）であることを確認し、行番号を記録: 43597
- [x] `v38000_tests` の他 3 テスト（`changelog_has_v38_0_0` / `milestone_has_multi_source_etl_power` / `readme_mentions_multi_source_etl`）はバージョン変更後も pass することを確認（`[v38.0.0]` エントリは CHANGELOG から削除しないため影響なし）
- [x] `driver.rs` に `v38100_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `v38000_tests` の閉じ `}` の行番号を確認し、ここに記録: 43624
- [x] `CHANGELOG.md` に `[v38.1.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `fav/src/suggest.rs` が存在しないことを確認（今回新規作成）
- [x] `main.rs` に `mod suggest;` が存在しないことを確認（今回追加）
- [x] `versions/current.md` の最新安定版が `v38.0.0`・次バージョンが `v38.1.0` であることを確認
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.1.0 が未完了（✅ なし）であることを確認（T9 で更新）
- [x] `roadmap-v38.1-v39.0.md` の v38.1.0 テスト件数欄が「2 件」であることを確認（T9 で 3 件に更新）

## T1: CHANGELOG.md に [v38.1.0] エントリを追加

- [x] `## [v38.0.0]` の `---` セパレータ直後に `## [v38.1.0]` エントリを挿入
- [x] 日付を `2026-07-10` に設定
- [x] セパレータが `—`（全角ダッシュ U+2014）形式であることを確認

## T2: `fav/src/suggest.rs` 新規作成

- [x] spec.md §1 の内容で `fav/src/suggest.rs` を新規作成
- [x] `pub fn cmd_suggest(error_code: &str, location: &str) -> Result<(), String>` を含む
- [x] `fn builtin_hint(error_code: &str) -> String` を含む（E0001 / E0007 / E0008）
- [x] `fn llm_suggest(...) -> String` を含む（現時点はスタブ）
- [x] `fn read_source(location: &str) -> Result<String, String>` を含む

## T3: `fav/src/main.rs` — `mod suggest;` 追加

- [x] T2（suggest.rs 作成）が完了していることを確認してから着手
- [x] Read で `mod rune_cmd;` の行番号を確認
- [x] `mod rune_cmd;` の直後に `mod suggest;` を追加

## T4: `fav/src/main.rs` — `Some("suggest")` ディスパッチアーム追加

- [x] Read で `Some("registry")` の行番号を確認
- [x] `Some("registry")` と同じ match ブロック内に `Some("suggest")` アームを追加
  - [x] `suggest::cmd_suggest(error_code, location)` を呼ぶ
  - [x] エラー時は `eprintln!` + `std::process::exit(1)`

## T5: `driver.rs` — `v38000_tests::cargo_toml_version_is_38_0_0` をスタブ化

- [x] Read で `cargo_toml_version_is_38_0_0` の行番号を確認（T0 で記録済み: 43597）
- [x] ライブアサーション → `// Stubbed: version bumped to 38.1.0 — assertion intentionally removed` に変更
- [x] **注意:** `changelog_has_v38_0_0` / `milestone_has_multi_source_etl_power` / `readme_mentions_multi_source_etl` はスタブ化しない
- [x] スタブ形式が前バージョン（v38.0.0 等）のスタブと一致していることを確認

## T6: `driver.rs` — `v38100_tests` モジュールを新規追加（T1・T2 完了後に実施）

- [x] T1（CHANGELOG 追加）と T2（suggest.rs 作成）が完了していることを確認してから着手
- [x] `v38000_tests` の閉じ `}` の行番号（T0 で記録済み: 43624）を Read で特定してから Edit を実行
- [x] `v38000_tests` の閉じ `}` の後に `v38100_tests` モジュールを追加
  - [x] imports 不要（`include_str!` のみ）
  - [x] `cargo_toml_version_is_38_1_0`
  - [x] `changelog_has_v38_1_0`
  - [x] `suggest_fn_exists`（`include_str!("suggest.rs")` で `pub fn cmd_suggest` を確認）

## T7: バージョン更新（T1〜T6 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `38.1.0` に更新

## T8: テスト実行

- [x] T7（Cargo.toml 更新）が完了していることを確認してから着手
- [x] `cargo test` 全通過 — ≥ 2744 passed; 0 failed — 実測: 2744 passed, 0 failed ✅
- [x] `v38100_tests` の 3 テストがすべて pass
- [x] `cargo_toml_version_is_38_1_0` が pass
- [x] `changelog_has_v38_1_0` が pass
- [x] `suggest_fn_exists` が pass

## T9: ドキュメント更新（T8 完了後）

- [x] `versions/v36-v40/v38.1.0/tasks.md` を COMPLETE ステータスに更新（T0〜T9 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v38.1.0（最新安定版）・v38.2.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.1.0 を完了済みにマーク（✅）し、テスト件数を 3 件に更新

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `suggest.rs` に `pub fn cmd_suggest` が含まれる | `suggest_fn_exists` テスト ✅ |
| 2 | `CHANGELOG.md` に `[v38.1.0]` が含まれる | `changelog_has_v38_1_0` テスト ✅ |
| 3 | `Cargo.toml` バージョンが `38.1.0` | `cargo_toml_version_is_38_1_0` テスト ✅ |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2744） | 実測: 2744 passed, 0 failed ✅ |
| 5 | `roadmap-v38.1-v39.0.md` の v38.1.0 が ✅ かつテスト件数が 3 件 | T9 後に目視確認 ✅ |
