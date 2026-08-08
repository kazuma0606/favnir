# Tasks — v57.8.0 — ドキュメントサイト Enterprise Security 記事

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x]`versions/roadmap/roadmap-v57.1-v58.0.md` の v57.8.0 セクションを確認
- [x]`versions/roadmap/roadmap-v55.1-v60.0.md` の v57.8.0 欄が存在することを確認（T12 の更新対象）
- [x]ベーステスト数 3267（v57.7.0 完了時点の実績値）を確認
- [x]`fav/Cargo.toml` が `57.7.0` であることを確認（更新前）
- [x]`v57800_tests` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x]`v57700_tests` が `driver.rs` に存在することを確認（`v57800_tests` の挿入位置として使用）
- [x]`v56300_tests::cargo_toml_version_is_56_3_0` が `"57.7.0"` を期待していることを確認（更新対象）
- [x]`v56900_tests::cargo_toml_version_is_56_9_0` が `"57.7.0"` を期待していることを確認（更新対象）
- [x]`v57000_tests::cargo_toml_version_is_57_0_0` が `"57.7.0"` を期待していることを確認（更新対象・rolling）
- [x]`v57100_tests` 〜 `v57700_tests` に `cargo_toml_version_is_*` が存在しないことを確認（rolling 更新対象外）
- [x]`site/content/docs/enterprise/` ディレクトリが存在しないことを確認（新規作成対象）

---

## 実装タスク

- [x]T1: `fav/Cargo.toml` version を `57.8.0` に更新
- [x]T2: `site/content/docs/enterprise/rbac.mdx` 新規作成
  - [x]`RBAC` キーワードを含む（テスト検証対象）
  - [x]`roles` キーワードを含む（テスト検証対象）
  - [x]`bindings` キーワードを含む（テスト検証対象）
  - [x]`E0424` キーワードを含む（テスト検証対象）
- [x]T3: `site/content/docs/enterprise/secrets.mdx` 新規作成
  - [x]`aws-secrets-manager` / `vault` プロバイダ例を含む
- [x]T4: `site/content/docs/enterprise/compliance.mdx` 新規作成
  - [x]`GDPR` キーワードを含む（テスト検証対象）
  - [x]`SOC2` キーワードを含む（テスト検証対象）
- [x]T5: `fav/src/driver.rs` — `v57800_tests` モジュールを `v57700_tests` の直前に追加
  - [x]`docs_rbac_page_exists` テスト: `include_str!` で rbac.mdx を読み込み 4 キーワードを検証
  - [x]`docs_compliance_page_exists` テスト: `include_str!` で compliance.mdx を読み込み 2 キーワードを検証
- [x]T6: `fav/src/driver.rs` — バージョンチェックテスト更新
  - [x]`v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"57.7.0"` → `"57.8.0"` に更新
  - [x]failure メッセージも `"should be 57.8.0"` に更新
  - [x]`v56900_tests::cargo_toml_version_is_56_9_0` の期待値（rolling）も `"57.7.0"` → `"57.8.0"` に更新
  - [x]`v57000_tests::cargo_toml_version_is_57_0_0` の期待値（rolling）も `"57.7.0"` → `"57.8.0"` に更新
  - [x]モジュール名・関数名は変更しない（慣例）

---

## テスト・検証

- [x]T7: `cargo build` でコンパイルエラーがないことを確認
- [x]T8: `cargo test` 全通過（**3269 tests passed, 0 failed**）
  - [x]`v57800_tests::docs_rbac_page_exists` ok
  - [x]`v57800_tests::docs_compliance_page_exists` ok
  - [x]既存 3267 件全通過
- [x]T9: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x]T10: `CHANGELOG.md` に v57.8.0 エントリを追加
- [x]T11: `versions/current.md` を v57.8.0 / 3269 tests に更新
- [x]T12: `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.8.0 実績を COMPLETE に更新
  - [x]`3267 + 2 = 3269 tests passed, 0 failed（2026-07-28）` を追記
- [x]T13: `versions/roadmap/roadmap-v55.1-v60.0.md` の v57.8.0 実績欄も COMPLETE に更新
  - [x]テスト数推移テーブルの v57.7.0 行（3267）の直後に v57.8.0 行（3269）を追加

---

## 完了確認

- [x]`docs_rbac_page_exists` pass
- [x]`docs_compliance_page_exists` pass
- [x]**3269 tests passed, 0 failed**（ベース 3267 + 2）
- [x]`cargo clippy -- -D warnings` クリーン
- [x]`CHANGELOG.md` に `[v57.8.0]` エントリが追加されている
- [x]`v56300_tests::cargo_toml_version_is_56_3_0` の期待値が `"57.8.0"` になっている
- [x]`v56900_tests::cargo_toml_version_is_56_9_0` の期待値が `"57.8.0"` になっている（rolling）
- [x]`v57000_tests::cargo_toml_version_is_57_0_0` の期待値が `"57.8.0"` になっている（rolling）
- [x]`versions/current.md` が v57.8.0 / 3269 tests を反映
- [x]T12 / T13 のロードマップ更新（実績 COMPLETE）が完了している

---

## 実装メモ

- `include_str!` パス: `fav/src/driver.rs` から `../../site/content/docs/enterprise/<file>.mdx`
- `secrets.mdx` は作成するが直接テストしない（ロードマップ完了条件は 2 件のみ）
- `v57100_tests` 〜 `v57700_tests` には `cargo_toml_version_is_*` が存在しないため rolling 更新対象は v56300 / v56900 / v57000 の 3 件のみ
