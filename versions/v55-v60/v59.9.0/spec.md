# v59.9.0 Spec — 安定化・コードフリーズ（Enterprise 1.0 前調整）

Date: 2026-07-30
Status: 設計中

---

## 概要

v60.0.0 — Enterprise 1.0 宣言に向けた安定化・コードフリーズを行う。

1. `site/content/docs/enterprise/enterprise1-overview.mdx` を拡充（v59.7.0 作成済みファイルに認定手順・クイックスタートを追記）
2. `v59900_tests` モジュールを追加（2 件）
3. ローリングチェック更新（既存 7 件 `59.8.0` → `59.9.0`）

---

## 実装内容

| 項目 | 内容 |
|---|---|
| `site/content/docs/enterprise/enterprise1-overview.mdx` | `## 認定手順` セクションおよび `## クイックスタート` セクションを追記 |
| `fav/src/driver.rs` | `v59900_tests` モジュールを追加（2 件）+ ローリングチェック更新 |
| `fav/Cargo.toml` | バージョン `59.9.0` |

---

## enterprise1-overview.mdx 拡充内容

v59.7.0 で作成した基本コンテンツ（機能一覧テーブル・認定要件）に以下を追記する。

```mdx
## 認定手順

1. `fav.toml` に必要なセクションを追加する（[設定チェックリスト](../cookbook/enterprise-checklist)参照）
2. `fav certify --level enterprise` を実行して全要件を確認する
3. 生成された `enterprise-cert.json` を CI アーティファクトとして保存する
4. `fav migrate --from v1 --to enterprise --dry-run` で既存コードを確認する
5. `fav migrate --from v1 --to enterprise --in-place <file>` で自動修正を適用する

## クイックスタート

```toml
# fav.toml — Enterprise 1.0 最小構成
[security.rbac]
enabled = true

[secrets]
provider = "aws-secrets-manager"

[security.tls]
enabled = true

[sla]
latency_p99_ms   = 200
availability_pct = 99.9
```
```

`enterprise1_overview_doc_complete` テストは `include_str!(...enterprise1-overview.mdx).contains("認定手順")` で検証する。

---

## テスト

`v59900_tests` モジュールを `v59800_tests` の直前に挿入（2 件）。
`use super::*;` は不要（`include_str!` のみ使用）。

| テスト名 | 検証内容 | 種別 |
|---|---|---|
| `cargo_toml_version_is_59_9_0` | `include_str!("../Cargo.toml").contains("version = \"59.9.0\"")` | rolling check（v60.0.0 以降も更新継続） |
| `enterprise1_overview_doc_complete` | `include_str!("../../site/content/docs/enterprise/enterprise1-overview.mdx").contains("認定手順")` かつ `contains("クイックスタート")` | 固定テスト |

**ベース**: 3324（v59.8.0 実績値）
**完了条件**: 3324 + 2 = **3326 tests passed, 0 failed**

### rolling check に関する注意

`cargo_toml_version_is_59_9_0` は v59000_tests の `cargo_toml_version_is_59_0_0` と同じ rolling check パターンを採用する。
v60.0.0 以降、このテストは既存 7 件の rolling check と同様に assertion と failure メッセージが更新され、rolling check プールは 8 件になる。

---

## ローリングチェック更新

既存 7 件のローリングアサーションを `"59.8.0"` → `"59.9.0"` に更新:
- `v59000_tests::cargo_toml_version_is_59_0_0`
- `v58900_tests::cargo_toml_version_is_58_9_0`
- `v58000_tests::cargo_toml_version_is_58_0_0`
- `v57900_tests::cargo_toml_version_is_57_9_0`
- `v57000_tests::cargo_toml_version_is_57_0_0`
- `v56900_tests::cargo_toml_version_is_56_9_0`
- `v56300_tests::cargo_toml_version_is_56_3_0`

failure メッセージ 7 件も同様に `"59.9.0"` に更新（全 7 件とも同一パターン）。
**注意**: `// -- v59800_tests (v59.8.0) --` / `// -- v59700_tests (v59.7.0) --` 等のコメント行は置換しないこと。
**注意**: `v59100_tests`〜`v59800_tests` は rolling check なし（対象外）。

---

## 影響ファイル

| ファイル | 変更内容 |
|---|---|
| `site/content/docs/enterprise/enterprise1-overview.mdx` | `## 認定手順` / `## クイックスタート` セクション追記 |
| `fav/src/driver.rs` | `v59900_tests` + ローリングチェック更新 |
| `fav/Cargo.toml` | バージョン `59.9.0` |
| `CHANGELOG.md` | v59.9.0 エントリ追加 |
| `versions/current.md` | 最新安定版を v59.9.0 に更新 |
| `versions/roadmap/roadmap-v59.1-v60.0.md` | v59.9.0 実績欄更新・v60.0.0 ベース数を確定 |
| `versions/v55-v60/v59.9.0/tasks.md` | COMPLETE ステータスに更新 |
