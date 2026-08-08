# v59.7.0 Spec — README / MILESTONE Enterprise 1.0 整備

Date: 2026-07-30
Status: 設計中

---

## 概要

Enterprise 1.0 宣言（v60.0.0）に向けた文書整備を行う。

1. `README.md` に Enterprise 1.0 への言及と v56〜v60 機能サマリーを追加
2. `MILESTONE.md` に `## v60.0.0（予定）— Enterprise 1.0` エントリを追加
3. `site/content/docs/enterprise/enterprise1-overview.mdx` を新規作成

---

## 実装内容

| 項目 | 内容 |
|---|---|
| `README.md` | Enterprise 1.0 の言及・v56〜v60 機能サマリーを追記 |
| `MILESTONE.md` | `## v60.0.0（予定）— Enterprise 1.0` エントリを先頭付近に追加 |
| `site/content/docs/enterprise/enterprise1-overview.mdx` | Enterprise 1.0 概要ドキュメントを新規作成 |
| `fav/src/driver.rs` | `v59700_tests` モジュールを追加（2 件）+ ローリングチェック更新 |
| `fav/Cargo.toml` | バージョン `59.7.0` |

---

## README.md 追記内容

既存の v59.0.0 言及ブロック（`## 最新安定版` 付近）の後に以下を追加:

```markdown
**v59.x〜v60.0（2026-07）で、Enterprise 1.0 宣言に向けたスプリントを進めています。**
v56〜v59 で実装した全エンタープライズ機能（RBAC / Secrets / TLS / Audit / Compliance /
Blue-Green Deploy / Cost Visibility / SLA Guarantee / Migration Toolkit / Enterprise Certify）を統合し、
v60.0.0 — Enterprise 1.0 として宣言予定です。
```

`readme_has_enterprise1_mention` テストは `README.md.contains("Enterprise 1.0")` で検証する。

---

## MILESTONE.md 追記内容

`## v59.0.0` エントリの直前（ファイル先頭付近）に追加:

```markdown
## v60.0.0（予定）— Enterprise 1.0

Favnir v60.0 は **Enterprise 1.0** として宣言予定。
v56〜v59 の全エンタープライズ機能（RBAC / Secrets / TLS / Audit / Compliance /
Blue-Green Deploy / Cost Visibility / SLA Guarantee / Migration Toolkit / Enterprise Certify）を統合し、
「企業で安心して選ばれるデータパイプライン言語」として完成する。
```

`milestone_has_enterprise1` テスト（v60.0.0 に存在）は既に確認済み。
本バージョンのテスト `readme_has_enterprise1_mention` は README の `"Enterprise 1.0"` 含有を確認する。

---

## enterprise1-overview.mdx 内容

`site/content/docs/enterprise/enterprise1-overview.mdx` に以下の内容を作成:

```mdx
# Enterprise 1.0 Overview

Favnir v60.0 — Enterprise 1.0 は、v56〜v59 で実装した全エンタープライズ機能を統合し、
企業で安心して選ばれるデータパイプライン言語として完成する。

## Enterprise 1.0 機能一覧

| 機能 | 実装バージョン | 概要 |
|---|---|---|
| RBAC | v57.1 | ロールベースアクセス制御 |
| Secret 管理 | v57.2 | AWS SM / Vault / GCP SM 統合 |
| mTLS | v57.3 | 相互 TLS 接続 |
| 監査ログ | v57.5 | 署名付き監査ログ |
| コンプライアンス | v57.6 | GDPR / SOC2 / HIPAA レポート |
| Blue-Green Deploy | v58.1 | 無停止デプロイ |
| Cost Visibility | v59.3 | パイプラインコスト見積もり |
| SLA Guarantee | v59.2 | SLA 監視・アラート統合 |
| Migration Toolkit | v59.5 | v1 → Enterprise 自動移行 |
| Enterprise Certify | v59.6 | Enterprise 1.0 認定チェック |

## 認定要件

`fav certify --level enterprise` で全要件を確認できます。
```

`docs_enterprise1_overview_exists` テストは `include_str!(...enterprise1-overview.mdx).contains("Enterprise 1.0")` で検証する。

---

## テスト

`v59700_tests` モジュールを `v59600_tests` の直前に挿入（2 件）。

| テスト名 | 検証内容 |
|---|---|
| `readme_has_enterprise1_mention` | `include_str!("../../README.md").contains("Enterprise 1.0")` |
| `docs_enterprise1_overview_exists` | `include_str!("../../site/content/docs/enterprise/enterprise1-overview.mdx").contains("Enterprise 1.0")` |

**ベース**: 3320（v59.6.0 実績値）
**完了条件**: 3320 + 2 = **3322 tests passed, 0 failed**

---

## ローリングチェック更新

既存 7 件のローリングアサーションを `"59.6.0"` → `"59.7.0"` に更新:
- `v59000_tests::cargo_toml_version_is_59_0_0`
- `v58900_tests::cargo_toml_version_is_58_9_0`
- `v58000_tests::cargo_toml_version_is_58_0_0`
- `v57900_tests::cargo_toml_version_is_57_9_0`
- `v57000_tests::cargo_toml_version_is_57_0_0`（rolling check from v57.0.0）
- `v56900_tests::cargo_toml_version_is_56_9_0`（rolling check from v56.9.0）
- `v56300_tests::cargo_toml_version_is_56_3_0`

failure メッセージ 7 件も同様に `"59.7.0"` に更新（全 7 件とも同一パターン `"Cargo.toml version should be 59.6.0"` → `"59.7.0"` — 特殊書式は存在しない）。
**注意**: `v59000_tests` は rolling check あり（対象）。`v59100_tests`〜`v59600_tests` は rolling check なし（対象外）。

---

## 影響ファイル

| ファイル | 変更内容 |
|---|---|
| `README.md` | Enterprise 1.0 言及・v56〜v60 機能サマリー追記 |
| `MILESTONE.md` | `v60.0.0（予定）— Enterprise 1.0` エントリ追加 |
| `site/content/docs/enterprise/enterprise1-overview.mdx` | 新規作成 |
| `fav/src/driver.rs` | `v59700_tests` + ローリングチェック更新 |
| `fav/Cargo.toml` | バージョン `59.7.0` |
| `CHANGELOG.md` | v59.7.0 エントリ追加 |
| `versions/current.md` | 最新安定版を v59.7.0 に更新 |
| `versions/roadmap/roadmap-v59.1-v60.0.md` | v59.7.0 実績欄更新・v59.8.0 ベース数を確定 |
| `versions/v55-v60/v59.7.0/tasks.md` | COMPLETE ステータスに更新 |
