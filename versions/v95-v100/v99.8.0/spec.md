# Spec: v99.8.0 — 総合ドキュメント

## Background

v99.1〜v99.7 で SAP Platform の全機能（SapClient・QueryBuilder・CircuitBreaker・TenantContext・Masked<T>・SlaDefinition・ベンチマーク）が完成した。
v99.8.0 では、これらを統合したエンタープライズ向けガイドドキュメント 3 本を作成し、SAP Platform 1.0 の外部公開準備を完了する。

前提: `versions/v95-v100/v99.8.0/` ディレクトリが存在すること（存在しなければ作成する）。

## Goals

1. `site/content/docs/guides/sap-platform.mdx` — SAP Platform 1.0 全体像ガイドを作成する
2. `site/content/docs/guides/sap-migration.mdx` — v95.0 → v100.0 移行ガイドを作成する
3. `site/content/docs/guides/sap-enterprise-checklist.mdx` — 本番投入チェックリストを作成する
4. `fav/src/driver.rs` に `mod v99800_tests`（2 テスト）を追加する

## 成果物仕様

### sap-platform.mdx

SAP Platform 1.0 全体像を説明するガイドドキュメント。以下を含む:

- SAP Platform 1.0 の概要（v86.0〜v99.0 で構築した機能一覧）
- `sap-odata` Rune の主要型（SapTomlConfig・SapEnvironment・SapClient・QueryBuilder）
- `ctx` Rune の統合（AppCtx.sap・AppCtx.unmask・Ctx.for_tenant_mock）
- SAP Workflow（!Approval 型・IFlowClient）
- ガードレール（CircuitBreaker・TenantContext・Masked<T>・SlaDefinition）

コードサンプル例:
```
bind client <- ctx.sap.connect("BP")
bind result <- client.query_builder<BusinessPartner>()
    |> QueryBuilder.select(["BpId", "FullName"])
    |> QueryBuilder.filter("Country eq 'JP'")
    |> QueryBuilder.page(100, 0)
    |> client.execute
```

キーワード: `SAP Platform`, `sap-platform`

### sap-migration.mdx

v95.0 から v100.0 への移行手順を説明するガイド。以下を含む:

- v95.0（$batch・Lambda SnapStart）→ v99.x の変更概要
- `SapClient` interface 切り替え手順
- `CircuitBreaker` 導入ステップ
- `TenantContext` / `Masked<T>` 移行パターン
- `fav sla-check` の設定例

キーワード: `migration` または `移行`（テストで検証するキーワードはこの 2 択）

### sap-enterprise-checklist.mdx

SAP 連携を本番環境に投入する前の確認チェックリスト。以下を含む:

- 認証・認可チェック（SSM Parameter Store / Terraform IAM）
- SLA 定義確認（`fav sla-check --config sla.toml`）
- CircuitBreaker 設定確認
- マルチテナント分離確認
- GDPR マスキング確認（Masked<T>）
- E2E テスト通過確認
- モニタリング設定確認

キーワード: `checklist`, `enterprise`, `production`

### mod v99800_tests（driver.rs）

```rust
#[cfg(test)]
mod v99800_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn sap_platform_mdx_exists() {
        std::fs::read_to_string("../site/content/docs/guides/sap-platform.mdx")
            .expect("sap-platform.mdx should exist (v99.8.0)");
    }
    #[test]
    fn sap_platform_all_docs_have_keywords() {
        let platform = std::fs::read_to_string("../site/content/docs/guides/sap-platform.mdx")
            .expect("sap-platform.mdx should exist");
        let migration = std::fs::read_to_string("../site/content/docs/guides/sap-migration.mdx")
            .expect("sap-migration.mdx should exist");
        let checklist = std::fs::read_to_string("../site/content/docs/guides/sap-enterprise-checklist.mdx")
            .expect("sap-enterprise-checklist.mdx should exist");
        assert!(platform.contains("SAP Platform"), "sap-platform.mdx must contain 'SAP Platform'");
        assert!(migration.contains("migration") || migration.contains("移行"), "sap-migration.mdx must contain migration keyword");
        assert!(checklist.contains("checklist") || checklist.contains("チェック"), "sap-enterprise-checklist.mdx must contain checklist keyword");
    }
}
```

## Success Criteria

- 3 MDX ファイルが `site/content/docs/guides/` に存在する
- `sap-platform.mdx` が `SAP Platform` を含む
- `sap-migration.mdx` が移行キーワードを含む
- `sap-enterprise-checklist.mdx` がチェックリストキーワードを含む
- `mod v99800_tests` の 2 テストが pass する
- 合計テスト数: 4,273（4,271 + 2）

## Files to Modify

| ファイル | 操作 |
|---|---|
| `site/content/docs/guides/sap-platform.mdx` | 新規作成 |
| `site/content/docs/guides/sap-migration.mdx` | 新規作成 |
| `site/content/docs/guides/sap-enterprise-checklist.mdx` | 新規作成 |
| `fav/src/driver.rs` | `mod v99800_tests` 追加 |
| `CHANGELOG.md` | v99.8.0 エントリ追加 |
| `versions/current.md` | v99.8.0 に更新 |
