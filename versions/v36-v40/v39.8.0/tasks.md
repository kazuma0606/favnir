# v39.8.0 タスクリスト — Enterprise cookbook + ガバナンスドキュメント

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v39.1-v40.0.md` の v39.8.0（「Enterprise cookbook + ガバナンスドキュメント」）に沿ったバージョン。
> ロードマップ「Rust テスト 1 件」は推定値。本実装では meta 2 件（version + changelog）+ 機能テスト 1 件（site_has_governance_docs）= 計 3 件を採用。

## T0: 事前確認

- [x]`cargo test` の実測通過数を確認（目安: 2805（v39.7.0 完了時点の実績値））し、実測値をここに記録: 2805
- [x]Cargo.toml バージョンが `39.7.0` であることを確認
- [x]`v39700_tests::cargo_toml_version_is_39_7_0` がライブアサーション（`assert!(cargo.contains("39.7.0"), ...)`）であることを確認し、行番号を Grep で確認して記録: 44209
- [x]`cargo_toml_version_is_39_7_0` に `// NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること` が付いていることを確認（T4 のスタブ化範囲に含まれる）。NOTE コメントが欠落している場合は実装を中断し報告すること
- [x]`v39700_tests` の `changelog_has_v39_7_0` はバージョン変更後も pass することを確認
- [x]`driver.rs` に `v39800_tests` モジュールが存在しないことを確認（今回新規作成）
- [x]`v39700_tests` の閉じ `}` の行番号を Grep/Read で確認し、ここに記録: 44220
- [x]`CHANGELOG.md` に `[v39.8.0]` エントリが存在しないことを確認（今回新規作成）
- [x]`site/content/docs/governance/` ディレクトリが存在しないことを確認（今回新規作成）
- [x]`site/content/cookbook/multi-tenant-etl.mdx` が存在しないことを確認（今回新規作成）
- [x]`site/content/cookbook/secret-manager-vault.mdx` が存在しないことを確認（今回新規作成）
- [x]`site/content/cookbook/ci-policy-gate.mdx` が存在しないことを確認（今回新規作成）
- [x]`versions/current.md` の最新安定版が `v39.7.0`・「次に切る版」が `v39.8.0` であることを確認
- [x]`versions/roadmap/roadmap-v39.1-v40.0.md` の v39.8.0 が未完了（✅ なし）であることを確認（T7 で更新）
- [x]`roadmap-v39.1-v40.0.md` の v39.8.0 テスト件数欄が「Rust テスト 1 件」と記載されていることを確認

## T1: CHANGELOG.md に [v39.8.0] エントリを追加

- [x]`## [v39.7.0]` ヘッダ行の直前に `## [v39.8.0]` エントリを挿入
- [x]日付を `YYYY-MM-DD` 形式の実装当日の日付に変更
- [x]セパレータが `—`（全角ダッシュ U+2014）形式であることを確認
- [x]`### Added` セクションを使用していることを確認（`### Changed` ではない — 新規ファイル追加のため）
- [x]6 MDX ファイル名がすべて記載されていることを確認（spec.md §8 の CHANGELOG テンプレートと一致していること）

## T2: `site/content/docs/governance/` — ガバナンスドキュメント 3 ファイル作成

- [x]`site/content/docs/governance/rbac.mdx` を新規作成
  - [x]frontmatter: `title: "RBAC — ロールベースアクセス制御"` + `description` を含む
  - [x]`## コード例` セクションに `favnir` コードブロックを含む（`auth.require_role` / `auth.check_permission` の使用例）
  - [x]`## ポイント` セクションを含む
- [x]`site/content/docs/governance/audit-log.mdx` を新規作成
  - [x]frontmatter: `title: "Audit Log — パイプライン実行ログ"` + `description` を含む
  - [x]`## コード例` セクションに `Audit.log` / `Audit.start_trace` / `Audit.end_trace` の使用例を含む
  - [x]`fav.toml` の `[audit]` セクション設定例を含む
  - [x]`## ポイント` セクションを含む
- [x]`site/content/docs/governance/policy.mdx` を新規作成
  - [x]frontmatter: `title: "fav policy — 組織ポリシーの宣言的管理"` + `description` を含む
  - [x]`## コード例` セクションに policy ブロック記法を含む
  - [x]`fav policy check --ci` の CI ゲートとしての使い方を含む
  - [x]`## ポイント` セクションを含む

## T3: `site/content/cookbook/` — クックブック 3 ファイル作成

- [x]`site/content/cookbook/multi-tenant-etl.mdx` を新規作成
  - [x]frontmatter: `title: "マルチテナント ETL"` + `description` を含む
  - [x]`tenant.db_schema` / `tenant.s3_prefix` / `tenant.validate_tenant` の使用例を含む
  - [x]`## ポイント` セクションを含む
- [x]`site/content/cookbook/secret-manager-vault.mdx` を新規作成
  - [x]frontmatter: `title: "Secret Manager / Vault 連携"` + `description` を含む
  - [x]`Secret.get_aws` / `Secret.get_vault` / `Secret.get_gcp` / `Secret.get_env` の使用例を含む
  - [x]`fav.toml` `[secrets]` 設定例を含む
  - [x]`## ポイント` セクションを含む
- [x]`site/content/cookbook/ci-policy-gate.mdx` を新規作成
  - [x]frontmatter: `title: "CI ポリシーゲート"` + `description` を含む
  - [x]`fav policy check --ci` を GitHub Actions に組み込む例を含む
  - [x]`fav ci init` 生成 YAML との関係を記載
  - [x]`## ポイント` セクションを含む

## T4: `driver.rs` — `v39700_tests::cargo_toml_version_is_39_7_0` をスタブ化

- [x]Grep で `cargo_toml_version_is_39_7_0` の行番号を確認（T0 で記録済み）
- [x]NOTE コメントとライブアサーション全体 → `// Stubbed: version bumped to 39.8.0 — assertion intentionally removed` に変更
- [x]**注意:** `changelog_has_v39_7_0` はスタブ化しない
- [x]スタブ形式が前バージョンのスタブと一致していることを確認

## T5: `driver.rs` — `v39800_tests` モジュールを新規追加（T1 完了後に実施）

- [x]T1（CHANGELOG 追加）が完了していることを確認してから着手
- [x]T4（スタブ化）が完了していることを確認してから着手
- [x]`v39700_tests` の閉じ `}` の行番号（T0 で記録済み）を Read で特定してから Edit を実行
- [x]`v39700_tests` の閉じ `}` の後に `v39800_tests` モジュールを追加
  - [x]imports 不要（`include_str!` のみ）
  - [x]`cargo_toml_version_is_39_8_0`（NOTE コメント付き）
  - [x]`changelog_has_v39_8_0`
  - [x]`site_has_governance_docs`（governance 3 件 + cookbook 3 件を `include_str!` で参照）
- [x]テスト数が 3 件であることを確認（ロードマップ「Rust テスト 1 件」は推定値 — spec.md 参照）

## T6: バージョン更新（T1〜T5 すべて完了後）

- [x]`fav/Cargo.toml` バージョンを `39.8.0` に更新

## T7: テスト実行 + ドキュメント更新

- [x]`cargo test` 全通過 — ≥ 2808 passed; 0 failed — 実測: 2808 passed, 0 failed
- [x]`v39800_tests` の 3 テストがすべて pass
- [x]`cargo_toml_version_is_39_8_0` が pass
- [x]`changelog_has_v39_8_0` が pass
- [x]`site_has_governance_docs` が pass（6 MDX ファイルの `include_str!` 参照が解決される）
- [x]`versions/v36-v40/v39.8.0/tasks.md` を COMPLETE ステータスに更新（T0〜T7 全チェックボックスを `[x]` に）
- [x]`versions/current.md` を v39.8.0（最新安定版）・v39.9.0（次に切る版）に更新
- [x]`versions/roadmap/roadmap-v39.1-v40.0.md` の v39.8.0 を完了済みにマーク（✅）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `CHANGELOG.md` に `[v39.8.0]` が含まれる | `changelog_has_v39_8_0` テスト |
| 2 | `Cargo.toml` バージョンが `39.8.0` | `cargo_toml_version_is_39_8_0` テスト |
| 3 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2808） | `cargo test` 実行結果（2805 + 3 = 2808） |
| 4 | 6 MDX ファイルが存在する（governance 3 + cookbook 3） | `site_has_governance_docs` テスト（`include_str!` で CI 自動検証） |
| 5 | `roadmap-v39.1-v40.0.md` の v39.8.0 が ✅ | T7 後に目視確認 |
