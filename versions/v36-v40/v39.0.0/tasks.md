# v39.0.0 タスクリスト — Intelligence & Assistance マイルストーン宣言

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v38.1-v39.0.md` の v39.0.0（「Intelligence & Assistance マイルストーン宣言 ★クリーンアップ」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2781（v38.9.0 完了時点の実績値））し、実測値をここに記録: 2781
- [x] Cargo.toml バージョンが `38.9.0` であることを確認
- [x] `v38900_tests::cargo_toml_version_is_38_9_0` がライブアサーション（`assert!(cargo.contains("38.9.0"), ...)`）であることを確認し、行番号を記録: 43975
- [x] `v38900_tests` の他テスト（`changelog_has_v38_9_0` / `ai_overview_doc_exists` / `suggest_rs_has_llm_suggest`）はバージョン変更後も pass することを確認（バージョン番号を含まないため影響なし）
- [x] `driver.rs` に `v39000_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `v38900_tests` の閉じ `}` の行番号を Grep/Read で確認し、ここに記録: 44000
- [x] `CHANGELOG.md` に `[v39.0.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `MILESTONE.md` に `"Intelligence & Assistance"` が存在しないことを確認（今回追加）
- [x] `MILESTONE.md` の先頭セクションが `## v38.0.0` であることを確認（`## v39.0.0` を先頭に挿入）
- [x] `README.md` に `"Intelligence & Assistance"` が含まれないことを確認（今回追加）
- [x] `fav/tmp/hello.fav` の存在と内容を確認（cargo clean 後の復元基準として記録）
- [x] `versions/current.md` の最新安定版が `v38.9.0`・次バージョンが `v39.0.0` であることを確認
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v39.0.0 が未完了（✅ なし）であることを確認（T9 で更新）
- [x] `roadmap-v38.1-v39.0.md` の v39.0.0 テスト件数欄が未記入または空であることを確認（T9 で 4 件に更新）

## T1: CHANGELOG.md に [v39.0.0] エントリを追加

- [x] `## [v38.9.0]` の直前に `## [v39.0.0]` エントリを挿入
- [x] 日付を `YYYY-MM-DD` 形式の実装当日の日付に変更
- [x] セパレータが `—`（全角ダッシュ）形式であることを確認

## T2: ★クリーンアップ — cargo clean

- [x] `fav/tmp/hello.fav` が存在することを Read で確認してから実行
- [x] `cargo clean` を実行
- [x] `fav/tmp/hello.fav` が消失していないことを確認（存在・内容正常）
- [x] `cargo build` でコンパイルエラーがないことを確認（T8 の `cargo test` にて確認）

## T3: MILESTONE.md — v39.0.0 セクション追加

- [x] Read で `MILESTONE.md` の先頭（`# Favnir Milestones` と `## v38.0.0` の境界）を確認
- [x] `# Favnir Milestones` ヘッダの直後（`## v38.0.0` の直前）に v39.0.0 セクションを挿入
  - [x] 宣言文（`fav suggest` / `fav generate --from sql` / `fav explain --verbose` / Llm Rune / rag-pipeline テンプレート）を含む
  - [x] 達成コンポーネント表（v38.1〜v38.9 の 9 行）を含む
  - [x] `"Intelligence & Assistance"` キーワードを含む
  - [x] 宣言日（実装当日）を含む
  - [x] セクション末尾に `---` セパレータを追加
- [x] 挿入後の先頭順序が `v39.0.0` → `v38.0.0` → `v37.0.0` → ... になっていることを確認

## T4: README.md — v39.0 マイルストーン宣言行追加

- [x] `**v38.0（2026-07-10）...` 行の直後に追加
  - [x] `**v39.0（YYYY-MM-DD）で、[Intelligence & Assistance](./MILESTONE.md) マイルストーンを宣言しました。**` を追加
  - [x] `"Intelligence & Assistance"` キーワードを含む

## T5: driver.rs — `v38900_tests::cargo_toml_version_is_38_9_0` をスタブ化

- [x] ライブアサーション → `// Stubbed: version bumped to 39.0.0 — assertion intentionally removed` に変更
- [x] **注意:** `changelog_has_v38_9_0` / `ai_overview_doc_exists` / `suggest_rs_has_llm_suggest` はスタブ化しない
- [x] スタブ形式が前バージョン（v38.0.0 等）のスタブと一致していることを確認

## T6: driver.rs — `v39000_tests` モジュールを新規追加（T3・T4 完了後に実施）

- [x] T3（MILESTONE.md 追加）と T4（README.md 追加）が完了していることを確認してから着手
- [x] `v38900_tests` の閉じ `}` の行番号（T0 で記録）を Read で特定してから Edit を実行
- [x] `v38900_tests` の閉じ `}` の後に `v39000_tests` モジュールを追加
  - [x] imports 不要（`include_str!` のみ）
  - [x] `cargo_toml_version_is_39_0_0`
  - [x] `changelog_has_v39_0_0`
  - [x] `milestone_has_intelligence_and_assistance`
  - [x] `readme_mentions_intelligence_assistance`

## T7: バージョン更新（T1〜T6 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `39.0.0` に更新

## T8: テスト実行

- [x] `cargo test` 全通過 — ≥ 2785 passed; 0 failed — 実測: 2785 passed, 0 failed ✅
- [x] `v39000_tests` の 4 テストがすべて pass
- [x] `cargo_toml_version_is_39_0_0` が pass
- [x] `changelog_has_v39_0_0` が pass
- [x] `milestone_has_intelligence_and_assistance` が pass
- [x] `readme_mentions_intelligence_assistance` が pass

## T9: ドキュメント更新（T8 完了後）

- [x] `versions/v36-v40/v39.0.0/tasks.md` を COMPLETE ステータスに更新（T0〜T9 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v39.0.0（最新安定版）・v39.1.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v39.0.0 を完了済みにマーク（✅）し、テスト件数を 4 件に更新

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `MILESTONE.md` に `"Intelligence & Assistance"` が含まれる | `milestone_has_intelligence_and_assistance` テスト ✅ |
| 2 | `README.md` に `"Intelligence & Assistance"` が含まれる | `readme_mentions_intelligence_assistance` テスト ✅ |
| 3 | `CHANGELOG.md` に `[v39.0.0]` が含まれる | `changelog_has_v39_0_0` テスト ✅ |
| 4 | `Cargo.toml` バージョンが `39.0.0` | `cargo_toml_version_is_39_0_0` テスト ✅ |
| 5 | `cargo clean` 実施済み | T2 実行記録 ✅（26.2 GiB 削除） |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2785） | 実測: 2785 passed, 0 failed ✅ |
