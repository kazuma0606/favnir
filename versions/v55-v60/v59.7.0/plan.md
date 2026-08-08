# v59.7.0 Plan — README / MILESTONE Enterprise 1.0 整備

Date: 2026-07-30

---

## Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml`:
```
version = "59.6.0"  →  version = "59.7.0"
```

---

## Step 2: README.md — Enterprise 1.0 言及追加

既存の v59.0 宣言ブロック（`**v59.0（2026-07-29）で...`）の直後に追記する。

```markdown
**v59.x〜v60.0（2026-07）で、Enterprise 1.0 宣言に向けたスプリントを進めています。**
v56〜v59 で実装した全エンタープライズ機能（RBAC / Secrets / TLS / Audit / Compliance /
Blue-Green Deploy / Cost Visibility / SLA Guarantee / Migration Toolkit / Enterprise Certify）を統合し、
v60.0.0 — Enterprise 1.0 として宣言予定です。
```

---

## Step 3: MILESTONE.md — v60.0.0（予定）エントリ追加

ファイル先頭の `## v59.0.0` エントリの直前に挿入する。

```markdown
## v60.0.0（予定）— Enterprise 1.0

Favnir v60.0 は **Enterprise 1.0** として宣言予定。
v56〜v59 の全エンタープライズ機能（RBAC / Secrets / TLS / Audit / Compliance /
Blue-Green Deploy / Cost Visibility / SLA Guarantee / Migration Toolkit / Enterprise Certify）を統合し、
「企業で安心して選ばれるデータパイプライン言語」として完成する。

---

```

---

## Step 4: enterprise1-overview.mdx 作成

`site/content/docs/enterprise/enterprise1-overview.mdx` を新規作成する。

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

---

## Step 5: driver.rs — v59700_tests 追加

既存の `// ─────────` セパレータ行（`v59600_tests` ブロックの直前）の**前**に挿入する。
パターン: セパレータ行 → 空行 → `// -- v59700_tests ...` コメント → モジュール本体。
既存の `v59600_tests` ブロックとセパレータはそのまま残す。

```rust
// -- v59700_tests (v59.7.0) -- README / MILESTONE Enterprise 1.0 整備 --
#[cfg(test)]
mod v59700_tests {
    #[test]
    fn readme_has_enterprise1_mention() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("Enterprise 1.0"),
            "README.md should mention 'Enterprise 1.0'"
        );
    }

    #[test]
    fn docs_enterprise1_overview_exists() {
        let content = include_str!(
            "../../site/content/docs/enterprise/enterprise1-overview.mdx"
        );
        assert!(
            content.contains("Enterprise 1.0"),
            "enterprise1-overview.mdx should mention 'Enterprise 1.0'"
        );
    }
}
```

**注意**: `include_str!` は compile-time にファイルを読み込むため、
Step 2〜4（README / MILESTONE / MDX 作成）を必ず先に完了させること。

---

## Step 6: driver.rs — ローリングチェック更新

`"59.6.0"` → `"59.7.0"` に一括更新（assertion 7 件 + failure メッセージ 7 件）。

対象:
- `v59000_tests`（rolling check あり）
- `v58900_tests` / `v58000_tests` / `v57900_tests` / `v56300_tests`（通常）
- `v57000_tests`（"59.7.0 (rolling check from v57.0.0)"）
- `v56900_tests`（"59.7.0 (rolling check from v56.9.0)"）

**注意**: `v59100_tests`〜`v59600_tests` は rolling check なし → 変更不要。

---

## Step 7: テスト実行

```bash
cargo test -j 8 -- --test-threads=8
```

確認事項:
- `v59700_tests::readme_has_enterprise1_mention` pass
- `v59700_tests::docs_enterprise1_overview_exists` pass
- 総テスト数 **3322** tests passed, 0 failed

---

## Step 8: 事後処理

- `CHANGELOG.md` に v59.7.0 エントリを追加
- `versions/current.md` を v59.7.0 / 3322 tests に更新
- `versions/roadmap/roadmap-v59.1-v60.0.md` の v59.7.0 実績欄更新・v59.8.0 ベース数を `3322` に確定
- `versions/v55-v60/v59.7.0/tasks.md` を COMPLETE ステータスに更新
