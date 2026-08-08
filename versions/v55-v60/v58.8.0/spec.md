# v58.8.0 Spec — ドキュメントサイト Governance & Deployment 記事

Date: 2026-07-29
Status: 設計中

---

## 概要

v58.1〜v58.7 で実装した Governance & Deployment 機能（Blue/Green・カナリア・HA・
スキーマ管理・Data Catalog・Policy-as-Code・マルチ環境設定）のドキュメントサイト記事を作成する。
MDX ファイル 3 件を新規作成し、`include_str!` ベースの Rust テストで存在と内容を検証する。

---

## 作成する MDX ファイル

| ファイルパス | 内容 |
|---|---|
| `site/content/docs/enterprise/deployment.mdx` | Blue/Green・カナリア・HA の設定と運用ガイド |
| `site/content/docs/enterprise/governance.mdx` | Schema Migration・Data Catalog・Policy-as-Code ガイド |
| `site/content/cookbook/multi-env-pipeline.mdx` | マルチ環境設定（dev / staging / prod）のレシピ |

---

## 各 MDX ファイルの要件

### deployment.mdx

- タイトル: `Enterprise Deployment — Blue/Green, Canary, HA`
- 含むべきキーワード: `"Blue/Green"`, `"canary"`, `"HA"`, `"--strategy blue-green"`, `"--ha"`

### governance.mdx

- タイトル: `Enterprise Governance — Schema, Catalog, Policy`
- 含むべきキーワード: `"Policy-as-Code"`, `"Schema Migration"`, `"Data Catalog"`, `"E0426"`

### multi-env-pipeline.mdx

- タイトル: `Cookbook: Multi-Environment Pipeline`
- 含むべきキーワード: `"--env"`, `"dev"`, `"staging"`, `"prod"`
- **Rust テスト対象外**（人手確認のみ）

---

## テスト

`v58800_tests` モジュールを `v58700_tests` の直前に挿入:

| テスト名 | 内容 |
|---|---|
| `docs_deployment_page_exists` | `deployment.mdx` を `include_str!` で読み込み `"Blue/Green"` を含むことを検証 |
| `docs_governance_page_exists` | `governance.mdx` を `include_str!` で読み込み `"Policy-as-Code"` を含むことを検証 |

`include_str!` パス: `"../../site/content/docs/enterprise/deployment.mdx"` 等（driver.rs から 2 階層上）

**実際のベース**: 3300（v58.7.0 code-review 後の実績値）
**完了条件**: 3300 + 2 = **3302 tests passed, 0 failed**

---

## ロールアップチェック更新

v58000_tests の全ローリングアサーション（5 件）を `"58.8.0"` に更新。
failure メッセージ（5 件）も `"58.8.0"` に更新。

---

## v58.9.0 ベース数修正

ロードマップ v58.9.0 の完了条件は `3297 + 2 = 3299`（大幅に古い値）。
v58.7.0 code-review 完了後の実績 3300 をベースに `3302 + 2 = 3304` に修正する。

**v59.0.0 のカスケード修正は v58.9.0 実装完了後に行う**（本バージョンでは v58.9.0 のみ修正）。

---

## 影響ファイル

| ファイル | 変更内容 |
|---|---|
| `site/content/docs/enterprise/deployment.mdx` | 新規作成 |
| `site/content/docs/enterprise/governance.mdx` | 新規作成 |
| `site/content/cookbook/multi-env-pipeline.mdx` | 新規作成 |
| `fav/src/driver.rs` | v58800_tests + ローリングチェック更新 |
| `fav/Cargo.toml` | バージョン `58.8.0` |
| `CHANGELOG.md` | v58.8.0 エントリ追加 |
| `versions/current.md` | 最新安定版を v58.8.0 に更新 |
| `versions/roadmap/roadmap-v58.1-v59.0.md` | v58.8.0 実績欄に完了記録、v58.9.0 ベース数修正（3297→3302、目標 3299→3304） |

> main.rs の変更はなし（ドキュメント専用バージョン）。
