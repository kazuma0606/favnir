# v58.9.0 Spec — 安定化・コードフリーズ（Governance & Deployment 2.0 前調整）

Date: 2026-07-29
Status: 設計中

---

## 概要

v58.1〜v58.8 で実装した Governance & Deployment 機能の安定化バージョン。
`site/content/docs/governance-overview.mdx` の骨子を作成し、v59.0.0（宣言バージョン）への橋渡しとする。
lint / clippy クリーン確認を行い、全テストの継続通過を確認する。

---

## 作成するファイル

| ファイルパス | 内容 |
|---|---|
| `site/content/docs/governance-overview.mdx` | Governance & Deployment 全機能の概要ガイド（骨子） |

---

## governance-overview.mdx の要件

- タイトル: `Governance & Deployment — Overview`
- `# Governance & Deployment Overview` 見出し
- **`"Governance & Deployment"` という文字列を含む**（テストで検証）
- v58.1〜v58.8 で実装した機能の一覧（Blue/Green・カナリア・HA・Schema Migration・Data Catalog・Policy-as-Code・マルチ環境設定）へのリンク骨子

---

## テスト

`v58900_tests` モジュールを `v58800_tests` の直前に挿入:

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_58_9_0` | `Cargo.toml` が `version = "58.9.0"` を含むことを検証 |
| `governance_overview_exists` | `governance-overview.mdx` を `include_str!` で読み込み `"Governance & Deployment"` を含むことを検証 |

`include_str!` パス:
- `"../Cargo.toml"` （driver.rs から 1 階層上）
- `"../../site/content/docs/governance-overview.mdx"` （driver.rs から 2 階層上）

**実際のベース**: 3302（v58.8.0 実績値）
**完了条件**: 3302 + 2 = **3304 tests passed, 0 failed**

---

## ロールアップチェック更新

v58000_tests の全ローリングアサーション（5 件）を `"58.9.0"` に更新。
failure メッセージ（5 件）も `"58.9.0"` に更新。

---

## v59.0.0 カスケード修正（T2 で実施）

ロードマップ v59.0.0 の完了条件は `3299 + 4 = 3303`（古い値）。
v58.9.0 完了後の実績 **3304** をベースに `3304 + 4 = 3308` に修正する。

---

## 影響ファイル

| ファイル | 変更内容 |
|---|---|
| `site/content/docs/governance-overview.mdx` | 新規作成 |
| `fav/src/driver.rs` | v58900_tests + ローリングチェック更新 |
| `fav/Cargo.toml` | バージョン `58.9.0` |
| `CHANGELOG.md` | v58.9.0 エントリ追加 |
| `versions/current.md` | 最新安定版を v58.9.0 に更新 |
| `versions/roadmap/roadmap-v58.1-v59.0.md` | v58.9.0 実績欄に完了記録、v59.0.0 ベース数修正（3299→3304、目標 3303→3308） |

> main.rs の変更はなし（安定化専用バージョン）。
