# v59.8.0 Spec — ドキュメントサイト Enterprise 1.0 総括記事

Date: 2026-07-30
Status: 設計中

---

## 概要

Enterprise 1.0 宣言（v60.0.0）に向けたドキュメントサイトの総括記事を作成する。

1. `site/content/docs/enterprise/index.mdx` — Enterprise 1.0 の全機能一覧・認定要件・移行ガイド
2. `site/content/cookbook/enterprise-checklist.mdx` — Enterprise 運用に必要な設定チェックリスト

---

## 実装内容

| 項目 | 内容 |
|---|---|
| `site/content/docs/enterprise/index.mdx` | Enterprise 1.0 総括ドキュメントを新規作成 |
| `site/content/cookbook/enterprise-checklist.mdx` | Enterprise 設定チェックリストを新規作成 |
| `fav/src/driver.rs` | `v59800_tests` モジュールを追加（2 件）+ ローリングチェック更新 |
| `fav/Cargo.toml` | バージョン `59.8.0` |

---

## docs/enterprise/index.mdx 内容

```mdx
# Enterprise 1.0 ドキュメント

Favnir v60.0 — Enterprise 1.0 の全機能ドキュメント一覧です。

## 機能一覧

| 機能 | ドキュメント | 実装バージョン |
|---|---|---|
| RBAC | [rbac.mdx](./rbac) | v57.1 |
| Secret 管理 | [secrets.mdx](./secrets) | v57.2 |
| mTLS | [deployment.mdx](./deployment) | v57.3 |
| 監査ログ | [compliance.mdx](./compliance) | v57.5 |
| コンプライアンス | [compliance.mdx](./compliance) | v57.6 |
| Blue-Green Deploy | [deployment.mdx](./deployment) | v58.1 |
| Enterprise Certify | [enterprise1-overview.mdx](./enterprise1-overview) | v59.6 |

## 認定要件

Enterprise 1.0 の認定を受けるには以下を満たす必要があります:

- `[security.rbac]` の設定（RBAC）
- `[secrets]` の設定（Secrets 管理）
- `[security.tls]` の設定（mTLS）
- CI での `--audit-sign` 有効化（監査ログ）
- コンプライアンスレポートの生成（GDPR 等）

`fav certify --level enterprise` で全要件を自動チェックできます。

## 移行ガイド

v1 から Enterprise 1.0 へ移行するには `fav migrate --from v1 --to enterprise --dry-run` を実行してください。
```

`docs_enterprise_index_exists` テストは `include_str!("../../site/content/docs/enterprise/index.mdx").contains("Enterprise 1.0")` で検証する。

---

## cookbook/enterprise-checklist.mdx 内容

cookbook ファイルはフロントマター（`title` / `description`）を持つ慣例に従う。

```mdx
---
title: "Enterprise 1.0 設定チェックリスト"
description: "Favnir Enterprise 1.0 認定に必要な fav.toml / CI 設定の確認リスト"
---

# Enterprise 1.0 設定チェックリスト

## fav.toml チェックリスト

- [ ] `[security.rbac]` セクションを追加（RBAC 設定）
- [ ] `[secrets]` セクションを追加（Secrets 管理）
- [ ] `[security.tls]` セクションを追加（mTLS 設定）
- [ ] `[sla]` セクションを追加して `fav run --sla-enforce` を有効化（SLA 保証）
- [ ] `[env.production]` セクションでマルチ環境設定を追加

## CI チェックリスト

- [ ] `fav run --audit-sign` を CI パイプラインに追加
- [ ] `fav compliance report --framework gdpr` を定期実行に追加
- [ ] `fav certify --level enterprise` を CD に組み込む

## 移行チェックリスト

- [ ] `fav migrate --from v1 --to enterprise --dry-run` で変更内容を確認
- [ ] `fav migrate --from v1 --to enterprise --in-place <file>` で自動修正を適用
```

`cookbook_enterprise_checklist_exists` テストは `include_str!("../../site/content/cookbook/enterprise-checklist.mdx").contains("Enterprise")` で検証する。

---

## テスト

`v59800_tests` モジュールを `v59700_tests` の直前に挿入（2 件）。
`use super::*;` は不要（`include_str!` のみ使用）。

| テスト名 | 検証内容 |
|---|---|
| `docs_enterprise_index_exists` | `include_str!("../../site/content/docs/enterprise/index.mdx").contains("Enterprise 1.0")` |
| `cookbook_enterprise_checklist_exists` | `include_str!("../../site/content/cookbook/enterprise-checklist.mdx").contains("Enterprise")` |

**ベース**: 3322（v59.7.0 実績値）
**完了条件**: 3322 + 2 = **3324 tests passed, 0 failed**

---

## ローリングチェック更新

既存 7 件のローリングアサーションを `"59.7.0"` → `"59.8.0"` に更新:
- `v59000_tests::cargo_toml_version_is_59_0_0`
- `v58900_tests::cargo_toml_version_is_58_9_0`
- `v58000_tests::cargo_toml_version_is_58_0_0`
- `v57900_tests::cargo_toml_version_is_57_9_0`
- `v57000_tests::cargo_toml_version_is_57_0_0`
- `v56900_tests::cargo_toml_version_is_56_9_0`
- `v56300_tests::cargo_toml_version_is_56_3_0`

failure メッセージ 7 件も同様に `"59.8.0"` に更新（全 7 件とも同一パターン `"Cargo.toml version should be 59.7.0"` → `"59.8.0"` — 特殊書式は存在しない）。
**注意**: `// -- v59700_tests (v59.7.0) --` コメント行の `59.7.0` は置換しないこと。
**注意**: `v59000_tests` は rolling check あり（対象）。`v59100_tests`〜`v59700_tests` は rolling check なし（対象外）。

---

## 影響ファイル

| ファイル | 変更内容 |
|---|---|
| `site/content/docs/enterprise/index.mdx` | 新規作成 |
| `site/content/cookbook/enterprise-checklist.mdx` | 新規作成 |
| `fav/src/driver.rs` | `v59800_tests` + ローリングチェック更新 |
| `fav/Cargo.toml` | バージョン `59.8.0` |
| `CHANGELOG.md` | v59.8.0 エントリ追加 |
| `versions/current.md` | 最新安定版を v59.8.0 に更新 |
| `versions/roadmap/roadmap-v59.1-v60.0.md` | v59.8.0 実績欄更新・v59.9.0 ベース数を確定 |
| `versions/v55-v60/v59.8.0/tasks.md` | COMPLETE ステータスに更新 |
