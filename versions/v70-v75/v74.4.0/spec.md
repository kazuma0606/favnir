# v74.4.0 仕様書 — OSS Hardening

Date: 2026-08-14

---

## Background

Favnir を GitHub 上での公開 OSS として機能させるために、
コントリビューションガイド・セキュリティポリシー・行動規範・Issue テンプレートを整備する。
外部コントリビューターが安心して参加できる環境と、脆弱性報告の明確な窓口を設ける。

---

## Goals

1. `CONTRIBUTING.md` — 開発環境セットアップ・PR フロー・コーディング規約を作成する
2. `SECURITY.md` — 脆弱性報告手順を作成する
3. `CODE_OF_CONDUCT.md` — Contributor Covenant v2.1 ベースの行動規範を作成する
4. `.github/ISSUE_TEMPLATE/bug_report.md` — バグ報告テンプレートを作成する
5. `.github/ISSUE_TEMPLATE/feature_request.md` — 機能要望テンプレートを作成する
6. `v744000_tests` モジュール（2 件）を追加する
   - `oss_contributing_md_exists`
   - `oss_security_md_exists`

---

## ファイル構成

```
CONTRIBUTING.md              # 開発環境・PR フロー・コーディング規約
SECURITY.md                  # 脆弱性報告手順
CODE_OF_CONDUCT.md           # Contributor Covenant v2.1 ベース
.github/ISSUE_TEMPLATE/
├── bug_report.md            # バグ報告テンプレート
└── feature_request.md       # 機能要望テンプレート
```

### `CONTRIBUTING.md` 最小構成

```markdown
# Contributing to Favnir

...開発環境セットアップ・PR フロー...
```

### `SECURITY.md` 最小構成

```markdown
# Security Policy

## Reporting a Vulnerability

...脆弱性報告手順...
```

---

## Rust テスト（`driver.rs` の `v744000_tests`）

```rust
fn oss_contributing_md_exists() {
    let src = include_str!("../../CONTRIBUTING.md");
    assert!(src.contains("Contributing"), "CONTRIBUTING.md title missing");
    assert!(src.contains("Favnir"), "Favnir mention missing");
}

fn oss_security_md_exists() {
    let src = include_str!("../../SECURITY.md");
    assert!(src.contains("Security"), "SECURITY.md title missing");
    assert!(src.contains("Vulnerability"), "vulnerability section missing");
}
```

`include_str!` パス: `fav/src/driver.rs` から `../../` = リポジトリルート（`favnir/`）

---

## Success Criteria

1. `oss_contributing_md_exists` テストが pass する
   - `CONTRIBUTING.md` が存在し `"Contributing"` / `"Favnir"` を含む
2. `oss_security_md_exists` テストが pass する
   - `SECURITY.md` が存在し `"Security"` / `"Vulnerability"` を含む
3. `cargo test` で 3678 tests pass（0 failures）

---

## スコープ外（明示的除外）

- `cargo-deny` 設定（`deny.toml`）と CI 統合（後続バージョンで対応）
- SBOM 生成（後続バージョンで対応）

---

## Error Codes

新規エラーコードなし

---

## Files to Modify / Create

| ファイル | 変更内容 |
|---|---|
| `CONTRIBUTING.md` | 新規作成 |
| `SECURITY.md` | 新規作成 |
| `CODE_OF_CONDUCT.md` | 新規作成 |
| `.github/ISSUE_TEMPLATE/bug_report.md` | 新規作成 |
| `.github/ISSUE_TEMPLATE/feature_request.md` | 新規作成 |
| `fav/src/driver.rs` | `v744000_tests` 追加（`include_str!` パス: `../../CONTRIBUTING.md` / `../../SECURITY.md`） |
| `fav/Cargo.toml` | `version = "74.4.0"` に更新 |
| `CHANGELOG.md` | v74.4.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョン・次に切る版を更新 |
