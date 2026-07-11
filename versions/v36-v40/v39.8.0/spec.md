# v39.8.0 spec — Enterprise cookbook + ガバナンスドキュメント

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v39.8.0 |
| テーマ | Enterprise cookbook + ガバナンスドキュメント — `site/content/docs/governance/` 3 ファイル + `site/content/cookbook/` 3 ファイル |
| 前提 | v39.7.0 COMPLETE — CI/CD ポリシーゲート完了 |
| 完了条件 | `v39800_tests` 全テスト pass・`cargo test` 0 failures（≥ 2808 件） |

## 背景と目的

v39.1.0〜v39.7.0 で RBAC / Audit Log / Policy / Secret / マルチテナント / fav audit / CI/CD ゲートを実装した。
v39.8.0 ではこれらの機能を活用するためのガバナンスドキュメントとクックブックを整備し、
Enterprise Governance フェーズの成果を実際に使えるドキュメントとして公開する。

> ロードマップ「Rust テスト 1 件」は推定値。本バージョンでは meta 2 件（version + changelog）+ 機能テスト 1 件（site_has_governance_docs）= 3 件を採用する。実際の新規テスト数: 3 件（2805 + 3 = 2808）。

## 実装スコープ

### 1. `site/content/docs/governance/rbac.mdx` — RBAC ドキュメント

```mdx
---
title: "RBAC — ロールベースアクセス制御"
description: "auth Rune を使って Favnir パイプラインにロールベースアクセス制御を組み込む"
---
```

内容:
- `auth.require_role` / `auth.check_permission` / `auth.verify_jwt` の説明
- コード例: 管理者ロールが必要なパイプライン
- ポイント: 権限不足時の `Result.err` 挙動・JWT 連携

### 2. `site/content/docs/governance/audit-log.mdx` — Audit Log ドキュメント

```mdx
---
title: "Audit Log — パイプライン実行ログ"
description: "Audit Rune でパイプライン操作を追跡・記録する"
---
```

内容:
- `Audit.log` / `Audit.start_trace` / `Audit.end_trace` の説明
- コード例: ETL パイプラインにトレースを組み込む
- `fav.toml` の `[audit]` セクション設定例

### 3. `site/content/docs/governance/policy.mdx` — Policy ドキュメント

```mdx
---
title: "fav policy — 組織ポリシーの宣言的管理"
description: "fav policy check / fav policy check --ci で組織ルールをコードで管理する"
---
```

内容:
- `fav policy check` / `fav policy check --ci` の説明
- policy ブロック記法（deny_runes / require_schema / require_tests）
- CI ゲートとしての使い方（exit 1 挙動）

### 4. `site/content/cookbook/multi-tenant-etl.mdx` — マルチテナント ETL クックブック

```mdx
---
title: "マルチテナント ETL"
description: "tenant Rune を使ってテナントごとに DB スキーマ・S3 プレフィックスを分離する"
---
```

内容:
- `tenant.db_schema` / `tenant.s3_prefix` / `tenant.validate_tenant` の使用例
- テナント ID を AppCtx 経由で渡すパターン

### 5. `site/content/cookbook/secret-manager-vault.mdx` — Secret Manager クックブック

```mdx
---
title: "Secret Manager / Vault 連携"
description: "Secret Rune で AWS / Vault / GCP / Env から安全にシークレットを取得する"
---
```

内容:
- `Secret.get_aws` / `Secret.get_vault` / `Secret.get_gcp` / `Secret.get_env` の使用例
- `fav.toml` の `[secrets] backend` 設定
- ローカル開発フォールバック（`get_env`）

### 6. `site/content/cookbook/ci-policy-gate.mdx` — CI ポリシーゲートクックブック

```mdx
---
title: "CI ポリシーゲート"
description: "fav policy check --ci を GitHub Actions に組み込んで PR をブロックする"
---
```

内容:
- `fav ci init` 生成 YAML の確認（Policy check ステップが自動含まれる旨）
- 手動追加例（既存 CI への組み込み）
- 違反時の挙動（stderr + exit 1）

### 7. `fav/src/driver.rs` — テストモジュール更新

#### `v39700_tests::cargo_toml_version_is_39_7_0` のスタブ化

```rust
// Stubbed: version bumped to 39.8.0 — assertion intentionally removed
```

#### `v39800_tests` モジュール新規追加（`v39700_tests` の閉じ `}` の後に追加）

```rust
// ── v39800_tests (v39.8.0) — Enterprise cookbook + ガバナンスドキュメント ─────
#[cfg(test)]
mod v39800_tests {
    // include_str! のみ使用のため imports 不要

    #[test]
    fn cargo_toml_version_is_39_8_0() {
        // NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("39.8.0"), "Cargo.toml must contain version 39.8.0");
    }

    #[test]
    fn changelog_has_v39_8_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v39.8.0]"), "CHANGELOG.md must contain [v39.8.0]");
    }

    #[test]
    fn site_has_governance_docs() {
        // governance ドキュメント 3 件
        let _ = include_str!("../../site/content/docs/governance/rbac.mdx");
        let _ = include_str!("../../site/content/docs/governance/audit-log.mdx");
        let _ = include_str!("../../site/content/docs/governance/policy.mdx");
        // cookbook 3 件
        let _ = include_str!("../../site/content/cookbook/multi-tenant-etl.mdx");
        let _ = include_str!("../../site/content/cookbook/secret-manager-vault.mdx");
        let _ = include_str!("../../site/content/cookbook/ci-policy-gate.mdx");
    }
}
```

> ロードマップ「Rust テスト 1 件」= 推定値。本実装では meta 2 件 + 機能テスト 1 件（site_has_governance_docs）= 3 件を採用。

### 8. `CHANGELOG.md` — `[v39.8.0]` エントリ追加

`## [v39.7.0]` ヘッダ行の直前に挿入（`### Added` セクション使用）:

```
## [v39.8.0] — YYYY-MM-DD

### Added
- `site/content/docs/governance/rbac.mdx` — RBAC ガバナンスドキュメント追加
- `site/content/docs/governance/audit-log.mdx` — Audit Log ガバナンスドキュメント追加
- `site/content/docs/governance/policy.mdx` — Policy ガバナンスドキュメント追加
- `site/content/cookbook/multi-tenant-etl.mdx` — マルチテナント ETL クックブック追加
- `site/content/cookbook/secret-manager-vault.mdx` — Secret Manager クックブック追加
- `site/content/cookbook/ci-policy-gate.mdx` — CI ポリシーゲートクックブック追加
- `v39800_tests` 2 テスト追加（meta 2 件）

---
```

**セパレータは `—`（全角ダッシュ U+2014）**

> **Added セクション使用**: v39.8.0 は新規 MDX ファイル 6 件の追加のため `### Added` を使用する（v39.7.0 の `### Changed` とは異なる）。

### 9. その他ドキュメント更新

- `fav/Cargo.toml`: `39.7.0` → `39.8.0`
- `versions/current.md`: 最新安定版 → v39.8.0、次に切る版 → v39.9.0
- `versions/roadmap/roadmap-v39.1-v40.0.md`: v39.8.0 を ✅ 完了済みにマーク

## 注意事項

### 新規 Rust ソースファイル・main.rs 変更なし

v39.8.0 は MDX ファイル 6 件の追加と meta テスト 2 件のみ。新規 Rust ソースファイルの作成・`main.rs` への `mod` 追加・ディスパッチアーム追加は不要。`compiler.fav` / `checker.fav` 等のセルフホスト側ファイルへの変更も不要。

### MDX ファイルの構造規約

各 MDX ファイルは既存クックブック（例: `jwt-auth.mdx`）と同じ構造に従うこと:
- frontmatter: `title` + `description`（ダブルクォート）
- `# タイトル` 見出し
- 概要段落
- `## コード例` セクション（`favnir` コードブロック）
- `## ポイント` セクション（箇条書き）

### `docs/governance/` ディレクトリは新規作成

`site/content/docs/governance/` ディレクトリは存在しない。Write ツールで新規作成する（親ディレクトリも自動作成される）。

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v39.7.0 | 2805 |
| v39.8.0 追加分 | +3（meta 2 件 + site_has_governance_docs 1 件） |
| v39.8.0 期待値 | 2808 |

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `CHANGELOG.md` に `[v39.8.0]` が含まれる | `changelog_has_v39_8_0` テスト |
| 2 | `Cargo.toml` バージョンが `39.8.0` | `cargo_toml_version_is_39_8_0` テスト |
| 3 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2808） | `cargo test` 実行結果（2805 + 3 = 2808） |
| 4 | 6 MDX ファイルが存在する（governance 3 + cookbook 3） | `site_has_governance_docs` テスト（`include_str!` で 6 ファイル参照） |
| 5 | `roadmap-v39.1-v40.0.md` の v39.8.0 が ✅ | T7 後に目視確認 |
