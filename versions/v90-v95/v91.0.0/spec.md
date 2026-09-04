# Spec: v91.0.0 — SAP Ctx 統合 1.0 宣言 ★クリーンアップ

## Background

v90.1〜v90.9 で `ctx.sap.*` 統合を完成させた。本バージョンは宣言バージョンであり、
以下のクリーンアップを行って **SAP Ctx 統合 1.0** を正式宣言する:

1. v90.5.0 で deprecated コメントを付けた旧 `cfg: SapConfig` スタイル関数を削除する
2. `Cargo.toml` バージョンを `91.0.0` に更新する
3. CHANGELOG / MILESTONE / README / `versions/current.md` を更新する

## 宣言文

> 「`ctx.sap.business_partners(filter)` と書けば、SAP にアクセスできる。
>  設定は `AppCtx` に隠れ、テストは `MockSapClient` で差し替わる。
>  それが、Favnir SAP Ctx 統合 1.0 である。」

## Goals

1. `cargo clean` を実施する（ビルドキャッシュ削除）
2. `Cargo.toml` バージョンを `91.0.0` に更新する
3. CHANGELOG に v91.0.0 エントリを追加する
4. MILESTONE に SAP Ctx 統合 1.0 完了を記録する
5. README に `ctx.sap` パターンの言及を追加する
6. `versions/current.md` を v91.0.0 に更新する
7. driver.rs の `cargo_toml_version_is_90_0_0` テストを `91_0_0` に更新する
8. deprecated 旧 `cfg: SapConfig` 関数を削除する
   - `runes/sap-odata/sap_odata.fav` — `business_partners_cfg` / `sales_orders_cfg` / `materials_cfg` / `journal_entries_cfg` の 4 関数
   - `runes/sap-odata/business_partner.fav` / `sales_order.fav` / `material.fav` / `journal_entry.fav` — `cfg: SapConfig` を受け取る関数 variants（実態を確認して削除）
9. `mod v91000_tests` を追加する（4 件）

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version = "91.0.0" |
| `CHANGELOG.md` | v91.0.0 エントリ追加 |
| `MILESTONE.md` | SAP Ctx 統合 1.0 完了記録 |
| `README.md` | `ctx.sap` パターン言及追加 |
| `versions/current.md` | v91.0.0 に更新 |
| `fav/src/driver.rs` | `cargo_toml_version_is_90_0_0` → `91_0_0` に更新 + `mod v91000_tests` 追加 |
| `runes/sap-odata/sap_odata.fav` | deprecated `*_cfg` 4 関数を削除 |
| `runes/sap-odata/business_partner.fav` | `cfg: SapConfig` 受け取り variants を削除（実態確認後） |
| `runes/sap-odata/sales_order.fav` | 同上 |
| `runes/sap-odata/material.fav` | 同上 |
| `runes/sap-odata/journal_entry.fav` | 同上 |

## Rust テスト仕様（`v91000_tests` — 4 件）

```rust
fn cargo_toml_version_is_91_0_0() {
    let content = fs::read_to_string("../fav/Cargo.toml").unwrap();
    assert!(content.contains("version = \"91.0.0\""));
}

fn changelog_has_v91_0_0() {
    let content = fs::read_to_string("../CHANGELOG.md").unwrap();
    assert!(content.contains("v91.0.0"));
}

fn milestone_has_sap_ctx_integration() {
    let content = fs::read_to_string("../MILESTONE.md").unwrap();
    assert!(content.contains("SAP Ctx 統合 1.0"));
}

fn readme_mentions_ctx_sap() {
    let content = fs::read_to_string("../README.md").unwrap();
    assert!(content.contains("ctx.sap"));
}
```

## Success Criteria

- `cargo test` で **4,065 tests, 0 failures**（+4）
- `runes/sap-odata/sap_odata.fav` に `deprecated` が含まれない（削除完了）
- CHANGELOG / MILESTONE / README / current.md がすべて更新済み
