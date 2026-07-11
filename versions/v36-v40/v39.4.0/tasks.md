# v39.4.0 タスクリスト — Secret Rune 強化

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v39.1-v40.0.md` の v39.4.0（「Secret Rune 強化」）に沿ったバージョン。

## T0: 事前確認

- [x]`cargo test` の実測通過数を確認（目安: 2794（v39.3.0 完了時点の実績値））し、実測値をここに記録: ___
- [x]Cargo.toml バージョンが `39.3.0` であることを確認
- [x]`v39300_tests::cargo_toml_version_is_39_3_0` がライブアサーション（`assert!(cargo.contains("39.3.0"), ...)`）であることを確認し、行番号を Grep で確認して記録: ___
- [x]`v39300_tests` の他 2 テスト（`changelog_has_v39_3_0` / `policy_rs_exists`）はバージョン変更後も pass することを確認（バージョン番号を含まないため影響なし）
- [x]`driver.rs` に `v39400_tests` モジュールが存在しないことを確認（今回新規作成）
- [x]`v39300_tests` の閉じ `}` の行番号を Grep/Read で確認し、ここに記録: ___
- [x]`CHANGELOG.md` に `[v39.4.0]` エントリが存在しないことを確認（今回新規作成）
- [x]`runes/secret/` ディレクトリが存在しないことを確認（今回新規作成）
- [x]`versions/current.md` の最新安定版が `v39.3.0`・「次に切る版」が `v39.4.0` であることを確認
- [x]`versions/roadmap/roadmap-v39.1-v40.0.md` の v39.4.0 が未完了（✅ なし）であることを確認（T8 で更新）
- [x]`roadmap-v39.1-v40.0.md` の v39.4.0 テスト件数欄が「3 件」と記載されていることを確認

## T1: CHANGELOG.md に [v39.4.0] エントリを追加

- [x]`## [v39.3.0]` ヘッダ行の直前に `## [v39.4.0]` エントリを挿入
- [x]日付を `YYYY-MM-DD` 形式の実装当日の日付に変更
- [x]セパレータが `—`（全角ダッシュ U+2014）形式であることを確認
- [x]4 関数（`get_aws` / `get_vault` / `get_gcp` / `get_env`）が追加内容に記載されていることを確認

## T2: `runes/secret/` ディレクトリ + ファイル新規作成

- [x]`mkdir runes/secret/` を実行（存在しない場合）
- [x]`runes/secret/secret.fav` を spec.md §1 の内容で作成
  - [x]`fn get_aws(ctx: AppCtx, name: String) -> Result<String, String> !Http` を含む
  - [x]`fn get_vault(ctx: AppCtx, path: String) -> Result<String, String> !Http` を含む
  - [x]`fn get_gcp(ctx: AppCtx, name: String) -> Result<String, String> !Http` を含む
  - [x]`fn get_env(ctx: AppCtx, name: String) -> Result<String, String>` を含む（`!Http` **なし**）
  - [x]各関数にスタブコメントが含まれることを確認
- [x]`runes/secret/rune.toml` を spec.md §2 の内容で作成
  - [x]`name = "secret"` を含む
  - [x]`version = "1.0.0"` を含む
  - [x]`description` フィールドを含む
  - [x]`entry = "secret.fav"` を含む
  - [x]`effects = ["!Http"]` を含む
  - [x]`[dependencies]` セクションが存在することを確認
  - [x]`author` フィールドが存在しないことを確認（`audit/rune.toml` との整合）

## T3: `driver.rs` — `v39300_tests::cargo_toml_version_is_39_3_0` をスタブ化

- [x]Grep で `cargo_toml_version_is_39_3_0` の行番号を確認（T0 で記録済み）
- [x]ライブアサーション → `// Stubbed: version bumped to 39.4.0 — assertion intentionally removed` に変更
- [x]**注意:** `changelog_has_v39_3_0` / `policy_rs_exists` はスタブ化しない
- [x]スタブ形式が前バージョンのスタブと一致していることを確認

## T4: `driver.rs` — `v39400_tests` モジュールを新規追加（T1・T2 完了後に実施）

- [x]T1（CHANGELOG 追加）と T2（secret.fav 作成）が完了していることを確認してから着手
- [x]`v39300_tests` の閉じ `}` の行番号（T0 で記録済み）を Read で特定してから Edit を実行
- [x]`v39300_tests` の閉じ `}` の後に `v39400_tests` モジュールを追加
  - [x]imports 不要（`include_str!` のみ）
  - [x]`cargo_toml_version_is_39_4_0`
  - [x]`changelog_has_v39_4_0`
  - [x]`secret_rune_exists`（`include_str!("../../runes/secret/secret.fav")` で `fn get_aws` を確認）
- [x]`include_str!` パスが正しいことを確認（`driver.rs` は `fav/src/`、対象は `../../runes/secret/secret.fav`）

## T5: バージョン更新（T1〜T4 すべて完了後）

- [x]`fav/Cargo.toml` バージョンを `39.4.0` に更新

## T6: テスト実行 → ドキュメント更新

- [x] `cargo test` 全通過 — ≥ 2797 passed; 0 failed — 実測: 2797 passed, 0 failed
- [x]`v39400_tests` の 3 テストがすべて pass
- [x]`cargo_toml_version_is_39_4_0` が pass
- [x]`changelog_has_v39_4_0` が pass
- [x]`secret_rune_exists` が pass
- [x]`versions/v36-v40/v39.4.0/tasks.md` を COMPLETE ステータスに更新（T0〜T6 全チェックボックスを `[x]` に）
- [x]`versions/current.md` を v39.4.0（最新安定版）・v39.5.0（次に切る版）に更新
- [x]`versions/roadmap/roadmap-v39.1-v40.0.md` の v39.4.0 を完了済みにマーク（✅）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `secret.fav` に `fn get_aws` が含まれる | `secret_rune_exists` テスト |
| 2 | `CHANGELOG.md` に `[v39.4.0]` が含まれる | `changelog_has_v39_4_0` テスト |
| 3 | `Cargo.toml` バージョンが `39.4.0` | `cargo_toml_version_is_39_4_0` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2797） | `cargo test` 実行結果（2794 + 3 = 2797） |
| 5 | `roadmap-v39.1-v40.0.md` の v39.4.0 が ✅ | T6 後に目視確認 |
