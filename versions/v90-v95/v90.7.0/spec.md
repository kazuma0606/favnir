# Spec: v90.7.0 — `Ctx.mock` に `sap: MockSapClient` を追加

## Background

v90.3.0 で `MockSapClient` を実装し、v90.4.0 で `Ctx.build()` に SAP 注入を追加した。
`ctx.fav` のコメントには「`Ctx.mock` でテスト用インスタンスを生成する」と記述されているが、
`Ctx.mock` 関数自体はまだ実装されていない。

本バージョンで `Ctx.mock` を `runes/ctx/ctx.fav` に追加し、
`sap: MockSapClient` フィールドを含む AppCtx をテストで簡単に構築できるようにする。

## Goals

1. `Ctx.mock` 関数を `runes/ctx/ctx.fav` に追加する
2. `MockSapClient` にデフォルト値コンストラクタ（`MockSapClient.default()`）を追加する
3. Rust テスト 2 件を `driver.rs` に追加する

## Syntax / API Examples

```favnir
// テスト用 AppCtx の構築
public fn Ctx.mock(sap: MockSapClient) -> AppCtx {
    AppCtx {
        sap: sap
    }
}
```

```favnir
// MockSapClient のデフォルト値（全フィールドを Result.err で初期化）
public fn MockSapClient.default() -> MockSapClient {
    MockSapClient {
        business_partners_result: Result.err("not implemented"),
        sales_orders_result:      Result.err("not implemented"),
        materials_result:         Result.err("not implemented"),
        journal_entries_result:   Result.err("not implemented")
    }
}
```

使用例:
```favnir
bind ctx <- Ctx.mock(MockSapClient {
    business_partners_result: Result.ok([sample_bp]),
    sales_orders_result:      Result.err("not implemented"),
    materials_result:         Result.err("not implemented"),
    journal_entries_result:   Result.err("not implemented")
})
bind partners <- ctx.sap.business_partners(BusinessPartnerFilter {
    country: Option.none(), category: Option.none(),
    changed_after: Option.none(), top: Option.none()
})
```

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `runes/ctx/ctx.fav` | `Ctx.mock(sap: MockSapClient) -> AppCtx` を追加 |
| `runes/sap-odata/mock.fav` | `MockSapClient.default() -> MockSapClient` を追加 |
| `fav/src/driver.rs` | `mod v90700_tests` を追加（テスト 2 件） |

## Success Criteria

- `runes/ctx/ctx.fav` に `Ctx.mock` が含まれる
- `runes/ctx/ctx.fav` の `Ctx.mock` 定義に `sap` フィールドが含まれる
- `runes/sap-odata/mock.fav` に `MockSapClient.default` が含まれる
- `cargo test` で **4,056 tests, 0 failures**（+2）

## Rust テスト仕様

```rust
// mod v90700_tests
fn ctx_mock_has_sap_field() {
    let content = fs::read_to_string("../runes/ctx/ctx.fav").unwrap();
    assert!(content.contains("Ctx.mock"));
    assert!(content.contains("sap:"));
}

fn mock_sap_client_default_exists() {
    let content = fs::read_to_string("../runes/sap-odata/mock.fav").unwrap();
    assert!(content.contains("MockSapClient.default"));
}
```
