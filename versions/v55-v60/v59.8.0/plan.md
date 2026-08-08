# v59.8.0 Plan — ドキュメントサイト Enterprise 1.0 総括記事

Date: 2026-07-30

---

## Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml`:
```
version = "59.7.0"  →  version = "59.8.0"
```

---

## Step 2: docs/enterprise/index.mdx 作成

`site/content/docs/enterprise/index.mdx` を新規作成する。
既存の enterprise docs（`rbac.mdx` 等）と同様にフロントマターなし・H1 見出しで開始する。

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

---

## Step 3: cookbook/enterprise-checklist.mdx 作成

`site/content/cookbook/enterprise-checklist.mdx` を新規作成する。
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

---

## Step 4: driver.rs — v59800_tests 追加

**注意**: Step 2〜3（MDX 作成）を先に完了させること（`include_str!` はコンパイル時に読み込む）。

既存の `// ─────────` セパレータ行（`v59700_tests` ブロックの直前）の**前**に挿入する。
パターン: セパレータ行 → 空行 → `// -- v59800_tests ...` コメント → モジュール本体。

```rust
// -- v59800_tests (v59.8.0) -- Enterprise 1.0 ドキュメント総括 --
#[cfg(test)]
mod v59800_tests {
    #[test]
    fn docs_enterprise_index_exists() {
        let content = include_str!("../../site/content/docs/enterprise/index.mdx");
        assert!(
            content.contains("Enterprise 1.0"),
            "enterprise/index.mdx should mention 'Enterprise 1.0'"
        );
    }

    #[test]
    fn cookbook_enterprise_checklist_exists() {
        let content = include_str!("../../site/content/cookbook/enterprise-checklist.mdx");
        assert!(
            content.contains("Enterprise"),
            "enterprise-checklist.mdx should mention 'Enterprise'"
        );
    }
}
```

---

## Step 5: driver.rs — ローリングチェック更新

`"59.7.0"` → `"59.8.0"` に一括更新（assertion 7 件 + failure メッセージ 7 件）。

全 7 件とも同一パターン:
- assertion: `contains("version = \"59.7.0\"")` → `contains("version = \"59.8.0\"")`
- failure msg: `"Cargo.toml version should be 59.7.0"` → `"Cargo.toml version should be 59.8.0"`

**注意**: driver.rs に `// -- v59700_tests (v59.7.0) --` コメント行があり `59.7.0` を含む。
一括置換ツールを使う場合は **このコメント行を置換対象から除外**すること（ヒストリコメントは保持）。
`contains("version = \"59.7.0\"")` という引用符付き文字列のみを置換すれば安全。

**注意**: `v59100_tests`〜`v59700_tests` は rolling check なし → 変更不要。

---

## Step 6: テスト実行

```bash
cargo test -j 8 -- --test-threads=8
```

確認事項:
- `v59800_tests::docs_enterprise_index_exists` pass
- `v59800_tests::cookbook_enterprise_checklist_exists` pass
- 総テスト数 **3324** tests passed, 0 failed

---

## Step 7: 事後処理

- `CHANGELOG.md` に v59.8.0 エントリを追加
- `versions/current.md` を v59.8.0 / 3324 tests に更新
- `versions/roadmap/roadmap-v59.1-v60.0.md` の v59.8.0 実績欄更新・v59.9.0 ベース数を `3324` に確定（Step 6 完了後に実施）
- `versions/v55-v60/v59.8.0/tasks.md` を COMPLETE ステータスに更新
