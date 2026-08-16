# v74.4.0 タスクリスト — OSS Hardening

Date: 2026-08-14
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `74.3.0` であることを確認
- [x] `cargo test` が 3676 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v743000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v744000_tests` が未存在であることを確認

---

## T1: OSS ファイル 5 件を作成

- [x] `CONTRIBUTING.md` を作成した
  - `"Contributing"` タイトルを含む
  - `"Favnir"` の言及を含む
  - 開発環境セットアップ・PR フロー・コーディング規約を含む
- [x] `SECURITY.md` を作成した
  - `"Security"` タイトルを含む
  - `"Vulnerability"` の言及を含む
  - 脆弱性報告先（メールアドレス）を含む
- [x] `CODE_OF_CONDUCT.md` を作成した
  - Contributor Covenant v2.1 ベースの内容を含む
- [x] `.github/ISSUE_TEMPLATE/bug_report.md` を作成した
  - `name: Bug Report` front matter を含む
- [x] `.github/ISSUE_TEMPLATE/feature_request.md` を作成した
  - `name: Feature Request` front matter を含む

---

## T2: `v744000_tests` モジュールを `driver.rs` に追加

- [x] `// --- v74.4.0: OSS Hardening ---` セクションコメントを追加した
- [x] `v743000_tests` の直後に `v744000_tests` モジュールを追加した
- [x] `oss_contributing_md_exists` テストを実装した
  - `include_str!("../../CONTRIBUTING.md")` を読み込む
  - `"Contributing"` / `"Favnir"` を含むことを assert
- [x] `oss_security_md_exists` テストを実装した
  - `include_str!("../../SECURITY.md")` を読み込む
  - `"Security"` / `"Vulnerability"` を含むことを assert
- [x] `cargo build` でエラーがないことを確認

---

## T3: バージョン更新

- [x] `fav/Cargo.toml` の `version = "74.3.0"` → `version = "74.4.0"` に変更した
- [x] `driver.rs` 内の `version = "74.3.0"` 参照を `version = "74.4.0"` に replace_all した（コメント・セクションヘッダーは置換不要）
- [x] `version should be 74.3.0` を `version should be 74.4.0` に replace_all した（アサートメッセージのみ対象）
- [x] 残存 `74.3.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `cargo build` でエラーがないことを確認
- [x] `fav/Cargo.lock` が `version = "74.4.0"` を含むことを確認

---

## T3.5: バージョン更新後の部分テスト再確認

- [x] `cargo test v744000` で 2 件 pass することを確認

---

## T4: 全体テスト確認

- [x] `cargo test` 全体で 3678 tests pass（0 failures）であることを確認

---

## T5: `CHANGELOG.md` 更新

- [x] `## [v74.4.0]` エントリを先頭に追加した
  - Added: `CONTRIBUTING.md` / `SECURITY.md` / `CODE_OF_CONDUCT.md` / `.github/ISSUE_TEMPLATE/` 2 件
  - Tests: 2 件、合計テスト数 3678（+2）

---

## T6: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-14 (v74.4.0)` に更新した
- [x] 「進行中バージョン」を `v74.4.0` に更新した
- [x] 「次に切る版」を `v74.5.0` に更新した

---

## T7: 最終確認（T5・T6 完了後）

- [x] `cargo test v744000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3678 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `74.4.0` であることを確認
- [x] `CHANGELOG.md` に `[v74.4.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v74.4.0` であることを確認

---

## スコープ外（明示的除外）

- `cargo-deny` 設定（`deny.toml`）と CI 統合（後続バージョンで対応）
- SBOM 生成（後続バージョンで対応）
- MILESTONE.md 更新（宣言バージョンではないため不要）
