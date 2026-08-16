# v74.4.0 実装計画 — OSS Hardening

Date: 2026-08-14

---

## 実装ステップ

### Step 1: OSS ファイル 5 件を作成

**`CONTRIBUTING.md`**:

```markdown
# Contributing to Favnir

Favnir へのコントリビューションを歓迎します。

## 開発環境セットアップ

```bash
git clone https://github.com/favnir/favnir
cd favnir/fav
cargo build
cargo test
```

## PR フロー

1. Issue を立てて方針を確認する
2. `feat/xxx` ブランチを切る
3. 実装 → テスト追加 → `cargo test` 全通過を確認
4. PR を開く（タイトルに `feat:` / `fix:` / `docs:` プレフィックスを付ける）

## コーディング規約

- `cargo fmt` を実行してからコミットする
- 新機能には必ずテストを追加する
- エラーは `Result<T, E>` で返す（`unwrap()` は避ける）
```

**`SECURITY.md`**:

```markdown
# Security Policy

## Supported Versions

| Version | Supported |
|---|---|
| 74.x.x | Yes |
| < 74.0.0 | No |

## Reporting a Vulnerability

セキュリティの脆弱性を発見した場合は、GitHub Issues では**なく**、
メールで `security@favnir.dev` に報告してください。

報告には以下を含めてください:
- 脆弱性の説明
- 再現手順
- 影響範囲の評価

48 時間以内に応答します。
```

**`CODE_OF_CONDUCT.md`**:

```markdown
# Contributor Covenant Code of Conduct

## Our Pledge

（Contributor Covenant v2.1 ベース）

すべての参加者に対してハラスメントのない環境を提供することを誓います。

## Our Standards

望ましい行動:
- 他の参加者への敬意と思いやり
- 建設的なフィードバック

受け入れられない行動:
- ハラスメント・差別・侮辱的な言動

## Enforcement

違反の報告は `conduct@favnir.dev` へ。
```

**`.github/ISSUE_TEMPLATE/bug_report.md`**:

```markdown
---
name: Bug Report
about: Report a bug in Favnir
---

## 概要

バグの概要を 1〜2 文で説明してください。

## 再現手順

1. ...
2. ...

## 期待する動作

...

## 実際の動作

...

## 環境

- Favnir バージョン: `fav --version`
- OS:
```

**`.github/ISSUE_TEMPLATE/feature_request.md`**:

```markdown
---
name: Feature Request
about: Suggest a new feature for Favnir
---

## 概要

提案する機能を 1〜2 文で説明してください。

## 動機

なぜこの機能が必要ですか？

## 提案する実装

（オプション）実装アイデアがあれば記述してください。
```

### Step 2: `v744000_tests` モジュールを `driver.rs` に追加

`v743000_tests` の直後に追加する。

```rust
// --- v74.4.0: OSS Hardening ---

#[cfg(test)]
mod v744000_tests {
    #[test]
    fn oss_contributing_md_exists() {
        let src = include_str!("../../CONTRIBUTING.md");
        assert!(src.contains("Contributing"), "CONTRIBUTING.md title missing");
        assert!(src.contains("Favnir"), "Favnir mention missing");
    }

    #[test]
    fn oss_security_md_exists() {
        let src = include_str!("../../SECURITY.md");
        assert!(src.contains("Security"), "SECURITY.md title missing");
        assert!(src.contains("Vulnerability"), "vulnerability section missing");
    }
}
```

### Step 3: バージョン更新

- `fav/Cargo.toml`: `version = "74.3.0"` → `version = "74.4.0"`
- `driver.rs` 内の `version = "74.3.0"` 参照を `version = "74.4.0"` に replace_all（コメント・セクションヘッダーは置換不要）
- `version should be 74.3.0` を `version should be 74.4.0` に replace_all（アサートメッセージのみ）
- `cargo build` で `Cargo.lock` が自動更新される

### Step 4: テスト確認

- `cargo test v744000` で 2 件 pass を確認
- `cargo test` 全体で 3678 tests pass を確認

### Step 5: `CHANGELOG.md` 更新

v74.4.0 エントリを先頭に追加。

### Step 6: `versions/current.md` 更新

- 最終更新: `2026-08-14 (v74.4.0)`
- 進行中: `v74.4.0`
- 次: `v74.5.0`
