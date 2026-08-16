# v74.6.0 タスクリスト — `fav audit` 拡張（依存関係セキュリティ機能追加）

Date: 2026-08-14
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `74.5.0` であることを確認
- [x] `cargo test` が 3680 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v745000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v746000_tests` が未存在であることを確認

---

## T1: 構造体 + 関数を `driver.rs` に追加

- [x] `// --- v74.6.0: fav audit 拡張（依存関係セキュリティ機能追加） ---` セクションコメントを追加した
- [x] `#[derive(Debug, Clone, PartialEq)] pub struct DepVulnerability` を追加した（name / version / cve / severity / fix_version）
- [x] `pub fn format_audit_deps_report(vulns: &[DepVulnerability]) -> String` を実装した
  - 空スライスは `"OK  0 vulnerabilities found"` を返す
  - 各エントリを `"severity  name version  cve  Update to fix_version"` 形式で改行区切りに連結
- [x] `pub fn apply_audit_fix(cargo_toml: &str, name: &str, fix_version: &str) -> String` を実装した
  - `name = "old_version"` → `name = "fix_version"` に置換
  - マッチしない場合は元の文字列をそのまま返す
- [x] `cargo build` でエラーがないことを確認

---

## T2: `v746000_tests` モジュールを追加

- [x] `v745000_tests` の直後に `v746000_tests` モジュールを追加した
- [x] `use super::{DepVulnerability, format_audit_deps_report, apply_audit_fix}` を追加した
- [x] `audit_detects_vulnerable_dep` テストを実装した
  - `DepVulnerability` を構築し各フィールドを assert
  - `format_audit_deps_report` の出力に severity / name / CVE / fix_version が含まれることを assert
  - 空スライスで `"OK"` を含む文字列を返すことを assert
- [x] `audit_fix_updates_cargo_toml` テストを実装した
  - `apply_audit_fix` で tokio のバージョンが更新されることを assert
  - serde が変更されないことを assert
  - 存在しないクレートで元の文字列が返ることを assert

---

## T3: バージョン更新

- [x] `fav/Cargo.toml` の `version = "74.5.0"` → `version = "74.6.0"` に変更した
- [x] `driver.rs` 内の `version = "74.5.0"` 参照を `version = "74.6.0"` に replace_all した（コメント・セクションヘッダーは置換不要）
- [x] `version should be 74.5.0` を `version should be 74.6.0` に replace_all した（アサートメッセージのみ対象）
- [x] 残存 `74.5.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `cargo build` でエラーがないことを確認
- [x] `fav/Cargo.lock` が `version = "74.6.0"` を含むことを確認

---

## T3.5: バージョン更新後の部分テスト再確認

- [x] `cargo test v746000` で 2 件 pass することを確認

---

## T4: 全体テスト確認

- [x] `cargo test` 全体で 3682 tests pass（0 failures）であることを確認

---

## T5: `CHANGELOG.md` 更新

- [x] `## [v74.6.0]` エントリを先頭に追加した
  - Added: `DepVulnerability` / `format_audit_deps_report` / `apply_audit_fix`
  - Tests: 2 件、合計テスト数 3682（+2）

---

## T6: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-14 (v74.6.0)` に更新した
- [x] 「進行中バージョン」を `v74.6.0` に更新した
- [x] 「次に切る版」を `v74.7.0` に更新した

---

## T7: 最終確認（T5・T6 完了後）

- [x] `cargo test v746000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3682 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `74.6.0` であることを確認
- [x] `CHANGELOG.md` に `[v74.6.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v74.6.0` であることを確認

---

## スコープ外（明示的除外）

- `cargo audit` CLI の実際の呼び出し（`process::Command` 統合は後続バージョン）
- RustSec データベースへのネットワークアクセス
- `Cargo.lock` の解析・パース
- `fav audit --deps` の main.rs CLI エントリポイント（後続バージョン）
- `--fix` フラグの実ファイル書き込み（後続バージョン）
- `site/` MDX 追加（v75.0.0 または後続フェーズで対応）
- MILESTONE.md 更新（宣言バージョンではないため不要）
