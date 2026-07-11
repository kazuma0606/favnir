# v38.0.0 タスクリスト — Multi-Source ETL Power マイルストーン宣言

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v37.1-v38.0.md` の v38.0.0（「Multi-Source ETL Power マイルストーン宣言 ★クリーンアップ」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2737（v37.9.0 完了時点の実績値））し、実測値をここに記録: 2737
- [x] Cargo.toml バージョンが `37.9.0` であることを確認
- [x] `v37900_tests::cargo_toml_version_is_37_9_0` がライブアサーション（`assert!(cargo.contains("37.9.0"), ...)`）であることを確認し、行番号を記録: 43561
- [x] `v37900_tests` の他 3 テスト（`changelog_has_v37_9_0` / `lineage_text_has_summary_line` / `multi_source_etl_doc_exists`）はバージョン変更後も pass することを確認（バージョン番号を含まないため影響なし）
- [x] `driver.rs` に `v38000_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `CHANGELOG.md` に `[v38.0.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `MILESTONE.md` に `"Multi-Source ETL Power"` が存在しないことを確認（今回追加）
- [x] `MILESTONE.md` の先頭セクションが `## v37.0.0` であることを確認（`## v38.0.0` を先頭に挿入）
- [x] `README.md` に `"Multi-Source ETL"` が含まれないことを確認（今回追加）
- [x] `fav/tmp/hello.fav` の存在と内容を確認（cargo clean 後の復元基準として記録）
- [x] `v37900_tests` の閉じ `}` の行番号を確認し、ここに記録: 43589
- [x] `versions/current.md` の最新安定版が `v37.9.0`・次バージョンが `v38.0.0` であることを確認
- [x] `versions/roadmap/roadmap-v37.1-v38.0.md` の v38.0.0 が未完了（✅ なし）であることを確認（T9 で更新）
- [x] `roadmap-v37.1-v38.0.md` の v38.0.0 テスト件数欄が未記入または空であることを確認（T9 で 4 件に更新）

## T1: CHANGELOG.md に [v38.0.0] エントリを追加

- [x] `## [v37.9.0]` の `---` セパレータ直後に `## [v38.0.0]` エントリを挿入
- [x] 日付を `YYYY-MM-DD` 形式の実装当日の日付に変更（2026-07-10）
- [x] セパレータが `—`（全角ダッシュ）形式であることを確認

## T2: ★クリーンアップ — cargo clean

- [x] `fav/tmp/hello.fav` が存在することを Read で確認してから実行
- [x] `cargo clean` を実行（26.4 GiB 削除）
- [x] `fav/tmp/hello.fav` が消失していないことを確認（存在・内容正常）
- [x] `cargo build` でコンパイルエラーがないことを確認（T8 の `cargo test` にて確認）

## T3: MILESTONE.md — v38.0.0 セクション追加

- [x] Read で `MILESTONE.md` の先頭（`# Favnir Milestones` と `## v37.0.0` の境界）を確認
- [x] `# Favnir Milestones` ヘッダの直後（`## v37.0.0` の直前）に v38.0.0 セクションを挿入
  - [x] 宣言文（`List.join_on` / `List.fan_out` / CDC Rune / lineage DOT/SVG / multi-source テンプレート）を含む
  - [x] 達成コンポーネント表（v37.1〜v37.9 の 9 行）を含む
  - [x] `"Multi-Source ETL Power"` キーワードを含む
  - [x] 宣言日（2026-07-10）を含む
  - [x] セクション末尾に `---` セパレータを追加
- [x] 挿入後の先頭順序が `v38.0.0` → `v37.0.0` → `v36.0.0` → ... になっていることを確認

## T4: README.md — v38.0 マイルストーン宣言行追加

- [x] `**v37.0（2026-07-09）...` 行の直後に追加
  - [x] `**v38.0（2026-07-10）で、[Multi-Source ETL Power](./MILESTONE.md) マイルストーンを宣言しました。**` を追加
  - [x] `"Multi-Source ETL"` キーワードを含む

## T5: driver.rs — `v37900_tests::cargo_toml_version_is_37_9_0` をスタブ化

- [x] ライブアサーション → `// Stubbed: version bumped to 38.0.0 — assertion intentionally removed` に変更
- [x] **注意:** `changelog_has_v37_9_0` / `lineage_text_has_summary_line` / `multi_source_etl_doc_exists` はスタブ化しない
- [x] スタブ形式が前バージョン（v37.9.0 等）のスタブと一致していることを確認

## T6: driver.rs — `v38000_tests` モジュールを新規追加（T3・T4 完了後に実施）

- [x] T3（MILESTONE.md 追加）と T4（README.md 追加）が完了していることを確認してから着手
- [x] `v37900_tests` の閉じ `}` の行番号（T0 で記録: 43589）を Read で特定してから Edit を実行
- [x] `v37900_tests` の閉じ `}` の後に `v38000_tests` モジュールを追加
  - [x] imports 不要（`include_str!` のみ）
  - [x] `cargo_toml_version_is_38_0_0`
  - [x] `changelog_has_v38_0_0`
  - [x] `milestone_has_multi_source_etl_power`
  - [x] `readme_mentions_multi_source_etl`

## T7: バージョン更新（T1〜T6 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `38.0.0` に更新

## T8: テスト実行

- [x] `cargo test` 全通過 — ≥ 2741 passed; 0 failed — 実測: 2741 passed, 0 failed ✅
- [x] `v38000_tests` の 4 テストがすべて pass
- [x] `cargo_toml_version_is_38_0_0` が pass
- [x] `changelog_has_v38_0_0` が pass
- [x] `milestone_has_multi_source_etl_power` が pass
- [x] `readme_mentions_multi_source_etl` が pass

## T9: ドキュメント更新（T8 完了後）

- [x] `versions/v36-v40/v38.0.0/tasks.md` を COMPLETE ステータスに更新（T0〜T9 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v38.0.0（最新安定版）・v38.1.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v37.1-v38.0.md` の v38.0.0 を完了済みにマーク（✅）し、テスト件数を 4 件に更新

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `MILESTONE.md` に `"Multi-Source ETL Power"` が含まれる | `milestone_has_multi_source_etl_power` テスト ✅ |
| 2 | `README.md` に `"Multi-Source ETL"` が含まれる | `readme_mentions_multi_source_etl` テスト ✅ |
| 3 | `CHANGELOG.md` に `[v38.0.0]` が含まれる | `changelog_has_v38_0_0` テスト ✅ |
| 4 | `Cargo.toml` バージョンが `38.0.0` | `cargo_toml_version_is_38_0_0` テスト ✅ |
| 5 | `cargo clean` 実施済み | T2 実行記録 ✅（26.4 GiB 削除） |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2741） | 実測: 2741 passed, 0 failed ✅ |
