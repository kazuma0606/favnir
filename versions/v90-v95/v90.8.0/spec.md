# Spec: v90.8.0 — サイトドキュメント更新（ctx.sap パターンガイド）

## Background

v90.1〜v90.7 で `ctx.sap.*` スタイルへの移行を完了した。
しかし `site/content/docs/runes/sap-odata.mdx` は旧スタイル
（`sap_config_from_env()` + `sap_odata.METHOD(cfg, filter)`）のままである。

本バージョンでドキュメントを新スタイルに更新し、
`ctx.sap` パターン / `MockSapClient` ユニットテスト / `Ctx.build` 自動注入
の 3 セクションを追加する。

## Goals

1. `sap-odata.mdx` の既存コード例を `ctx.sap.*` スタイルに書き換える
2. `ctx.sap` パターンの使い方セクションを追加する
3. `MockSapClient` を使ったユニットテストの書き方セクションを追加する
4. `Ctx.build` への自動設定注入の説明セクションを追加する
5. Rust テスト 2 件を `driver.rs` に追加する

## 更新内容詳細

### 既存コード例の書き換え

各エンティティ（BusinessPartner / SalesOrder / Material / JournalEntry）の
コード例を旧スタイル → 新スタイルに書き換える:

```favnir
-- 変更前（旧スタイル）
bind cfg      <- sap_odata.sap_config_from_env()
bind partners <- sap_odata.business_partners(cfg, BusinessPartnerFilter { ... })

-- 変更後（新スタイル）
bind partners <- ctx.sap.business_partners(BusinessPartnerFilter { ... })
```

### 追加セクション: `ctx.sap` パターン

```favnir
-- ctx.sap パターン: AppCtx 経由で SAP にアクセスする
fn sync_business_partners(ctx: AppCtx) -> Result<Int, String> {
    bind partners <- ctx.sap.business_partners(BusinessPartnerFilter {
        country:       Option.some("JP"),
        category:      Option.none(),
        changed_after: Option.none(),
        top:           Option.some(100)
    })
    Result.ok(List.length(partners))
}
```

### 追加セクション: `MockSapClient` ユニットテスト

```favnir
-- MockSapClient を使ったユニットテスト
bind ctx <- Ctx.mock(MockSapClient {
    business_partners_result: Result.ok([]),
    sales_orders_result:      Result.err("not implemented"),
    materials_result:         Result.err("not implemented"),
    journal_entries_result:   Result.err("not implemented")
})
bind partners <- ctx.sap.business_partners(BusinessPartnerFilter {
    country: Option.none(), category: Option.none(),
    changed_after: Option.none(), top: Option.none()
})
```

`MockSapClient.default()` でデフォルト値を使うことも可能:
```favnir
bind ctx <- Ctx.mock(MockSapClient.default())
```

### 追加セクション: `Ctx.build` 自動設定注入

`Ctx.build()` は `fav.toml [sap]` / 環境変数から SAP 設定を自動読み込みして
`SapODataClient` を生成し `ctx.sap` に注入する。

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `site/content/docs/runes/sap-odata.mdx` | 既存コード例を `ctx.sap.*` に書き換え + 3 セクション追加 |
| `fav/src/driver.rs` | `mod v90800_tests` を追加（テスト 2 件） |

## Success Criteria

- `sap-odata.mdx` に `ctx.sap` が含まれる
- `sap-odata.mdx` に `MockSapClient` が含まれる
- `sap-odata.mdx` に `sap_config_from_env` が含まれない（旧スタイル完全除去）
- `cargo test` で **4,058 tests, 0 failures**（+2）

## Rust テスト仕様

```rust
// mod v90800_tests
fn docs_sap_odata_mentions_ctx_sap() {
    let content = fs::read_to_string("../site/content/docs/runes/sap-odata.mdx").unwrap();
    assert!(content.contains("ctx.sap"));
}

fn docs_sap_odata_mentions_mock_sap_client() {
    let content = fs::read_to_string("../site/content/docs/runes/sap-odata.mdx").unwrap();
    assert!(content.contains("MockSapClient"));
}
```
