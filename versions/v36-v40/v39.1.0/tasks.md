# v39.1.0 タスクリスト — RBAC Rune

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v39.1-v40.0.md` の v39.1.0（「RBAC Rune」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2785（v39.0.0 完了時点の実績値））し、実測値をここに記録: 2785
- [x] Cargo.toml バージョンが `39.0.0` であることを確認
- [x] `v39000_tests::cargo_toml_version_is_39_0_0` がライブアサーション（`assert!(cargo.contains("39.0.0"), ...)`）であることを確認し、行番号を Grep で確認して記録: 44008
- [x] `v39000_tests` の他 3 テスト（`changelog_has_v39_0_0` / `milestone_has_intelligence_and_assistance` / `readme_mentions_intelligence_assistance`）はバージョン変更後も pass することを確認（バージョン番号を含まないため影響なし）
- [x] `driver.rs` に `v39100_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `v39000_tests` の閉じ `}` の行番号を Grep/Read で確認し、ここに記録: 44036
- [x] `CHANGELOG.md` に `[v39.1.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `runes/auth/auth.fav` が存在しないことを確認（今回新規作成）
- [x] `runes/auth/rune.toml` が存在しないことを確認（今回新規作成）
- [x] `versions/current.md` の最新安定版が `v39.0.0`・「次に切る版」が `v39.1.0` であることを確認
- [x] `versions/roadmap/roadmap-v39.1-v40.0.md` の v39.1.0 が未完了（✅ なし）であることを確認（T9 で更新）
- [x] `roadmap-v39.1-v40.0.md` の v39.1.0 テスト件数欄が「3 件」であることを確認

## T1: CHANGELOG.md に [v39.1.0] エントリを追加

- [x] `## [v39.0.0]` の直前に `## [v39.1.0]` エントリを挿入
- [x] 日付を `YYYY-MM-DD` 形式の実装当日の日付に変更
- [x] セパレータが `—`（全角ダッシュ U+2014）形式であることを確認

## T2: `runes/auth/auth.fav` 新規作成

- [x] spec.md §1 の内容で `runes/auth/auth.fav` を新規作成
- [x] `fn require_role(ctx: AppCtx, role: String) -> Result<Unit, String> !Http` を含む
- [x] `fn check_permission(ctx: AppCtx, permission: String) -> Result<Unit, String> !Http` を含む
- [x] `fn verify_jwt(ctx: AppCtx, token: String) -> Result<String, String> !Http` を含む
- [x] `auth_rune_exists` テストが検証するキーワード `fn require_role` が含まれることを確認

## T3: `runes/auth/rune.toml` 新規作成

- [x] spec.md §2 の内容で `runes/auth/rune.toml` を新規作成
- [x] `[rune]` セクション・`name = "auth"` が含まれることを確認

## T4: `driver.rs` — `v39000_tests::cargo_toml_version_is_39_0_0` をスタブ化

- [x] Grep で `cargo_toml_version_is_39_0_0` の行番号を確認（T0 で記録済み）
- [x] ライブアサーション → `// Stubbed: version bumped to 39.1.0 — assertion intentionally removed` に変更
- [x] **注意:** `changelog_has_v39_0_0` / `milestone_has_intelligence_and_assistance` / `readme_mentions_intelligence_assistance` はスタブ化しない
- [x] スタブ形式が前バージョンのスタブと一致していることを確認

## T5: `driver.rs` — `v39100_tests` モジュールを新規追加（T1・T2 完了後に実施）

- [x] T1（CHANGELOG 追加）と T2（auth.fav 作成）が完了していることを確認してから着手
- [x] `v39000_tests` の閉じ `}` の行番号（T0 で記録済み）を Read で特定してから Edit を実行
- [x] `v39000_tests` の閉じ `}` の後に `v39100_tests` モジュールを追加
  - [x] imports 不要（`include_str!` のみ）
  - [x] `cargo_toml_version_is_39_1_0`
  - [x] `changelog_has_v39_1_0`
  - [x] `auth_rune_exists`（`include_str!("../../runes/auth/auth.fav")` で `fn require_role` を確認）
- [x] `include_str!` パスが正しい形式であることを確認
  - `"../../runes/auth/auth.fav"` — `fav/src/` から 2 階層上のルート → `runes/auth/auth.fav`

## T6: バージョン更新（T1〜T5 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `39.1.0` に更新

## T7: テスト実行

- [x] T6（Cargo.toml 更新）が完了していることを確認してから着手
- [x] `cargo test` 全通過 — ≥ 2788 passed; 0 failed — 実測: 2788 passed, 0 failed ✅
- [x] `v39100_tests` の 3 テストがすべて pass
- [x] `cargo_toml_version_is_39_1_0` が pass
- [x] `changelog_has_v39_1_0` が pass
- [x] `auth_rune_exists` が pass

## T8: ドキュメント更新（T7 完了後）

- [x] `versions/v36-v40/v39.1.0/tasks.md` を COMPLETE ステータスに更新（T0〜T8 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v39.1.0（最新安定版）・v39.2.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v39.1-v40.0.md` の v39.1.0 を完了済みにマーク（✅）し、テスト件数を 3 件に更新

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `runes/auth/auth.fav` に `require_role` が含まれる | `auth_rune_exists` テスト ✅ |
| 2 | `CHANGELOG.md` に `[v39.1.0]` が含まれる | `changelog_has_v39_1_0` テスト ✅ |
| 3 | `Cargo.toml` バージョンが `39.1.0` | `cargo_toml_version_is_39_1_0` テスト ✅ |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2788） | 実測: 2788 passed, 0 failed ✅ |
| 5 | `roadmap-v39.1-v40.0.md` の v39.1.0 が ✅ かつテスト件数が 3 件 | 確認済み ✅ |
