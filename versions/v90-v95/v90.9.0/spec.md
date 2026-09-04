# Spec: v90.9.0 — 安定化・コードフリーズ

## Background

v90.1〜v90.8 で実装した `ctx.sap.*` 統合（SapClient interface / MockSapClient / Ctx.build / Ctx.mock / pipeline.fav 書き換え / ドキュメント更新）の
最終確認スプリント。新機能追加はなく、通しテストによる動作確認とバグ修正のみを行う。

## Goals

1. v90.1〜v90.8 の全実装が一貫していることを通しテストで確認する
2. `pipeline.fav` の `ctx.sap.*` 書き換えが完全であることを smoke テストで検証する
3. `runes/sap-odata/` ディレクトリに `mock.fav` が存在することを確認する
4. Rust テスト 2 件を `driver.rs` に追加する

## テスト仕様

### `sap_ctx_integration_smoke_all_scenarios`

`infra/e2e-demo/sap-odata/pipeline.fav` に以下がすべて含まれることを確認する:
- `sync_business_partners`（シナリオ 1 関数名）
- `daily_sales_report`（シナリオ 2 関数名）
- `check_stock_vs_orders`（シナリオ 3 関数名）
- `outstanding_payables`（シナリオ 4 関数名）
- `ctx.sap.`（新スタイルのアクセスパターン）

```rust
fn sap_ctx_integration_smoke_all_scenarios() {
    let content = fs::read_to_string("../infra/e2e-demo/sap-odata/pipeline.fav").unwrap();
    for name in &[
        "sync_business_partners",
        "daily_sales_report",
        "check_stock_vs_orders",
        "outstanding_payables",
        "ctx.sap.",
    ] {
        assert!(content.contains(name), "pipeline.fav should contain {}", name);
    }
}
```

### `sap_ctx_mock_client_in_rune_dir`

`runes/sap-odata/` ディレクトリ内に `mock.fav` が存在することを確認する。
v90.3.0 実装の再確認を新規テスト名（`sap_ctx_mock_client_in_rune_dir`）で担保する。

```rust
fn sap_ctx_mock_client_in_rune_dir() {
    let path = std::path::Path::new("../runes/sap-odata/mock.fav");
    assert!(path.exists(), "runes/sap-odata/mock.fav should exist");
}
```

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `mod v90900_tests` を追加（テスト 2 件） |

## Success Criteria

- `cargo test` で **4,061 tests, 0 failures**（+2）
- `sap_ctx_integration_smoke_all_scenarios` pass: `pipeline.fav` に 5 文字列すべて含まれる
- `sap_ctx_mock_client_in_rune_dir` pass: `runes/sap-odata/mock.fav` が存在する
