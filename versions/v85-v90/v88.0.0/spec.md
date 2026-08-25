# Spec: v88.0.0 — SAP Sales 1.0 宣言 ★クリーンアップ

## Background

v87.1〜v87.9 で SalesOrder の型定義・CRUD・ページネーション・売上レポート集計・
テスト・安定化を完了した。本バージョンは「SAP Sales 1.0」の宣言バージョンであり、
クリーンアップ（`cargo clean`・バージョン更新・ドキュメント更新）を行う。

## 宣言文

> 「受注が型になった。
>  `sales_orders()` で絞り込み、`sales_order_by_id()` で明細まで取得できる。
>  日次売上レポートが、Favnir の 10 行で書ける。」

## Goals

1. `cargo clean` でビルド成果物を削除し、クリーンな状態を確認する
2. `Cargo.toml` バージョンを `87.0.0` → `88.0.0` に更新する
3. `CHANGELOG.md` に v88.0.0 エントリを追加する
4. `MILESTONE.md` に SAP Sales 1.0 マイルストーンを追加する
5. `README.md` を最新状態に更新する
6. `versions/current.md` を v88.0.0 に更新する
7. `driver.rs` 内の既存 `cargo_toml_version_is_` テストを `88.0.0` に一括更新する
8. `mod v88000_tests` を追加する（4 件）

## Success Criteria（Rust テストで担保）

- `fav/Cargo.toml` に `version = "88.0.0"` が含まれる
- `CHANGELOG.md` に `[v88.0.0]` エントリが含まれる
- `MILESTONE.md` に `SAP Sales` が含まれる
- `runes/sap-odata/sap_odata.fav` に `SalesOrder` が含まれる
- `cargo test` で 3,997 tests, 0 failures
- Rust テスト 4 件:
  - `cargo_toml_version_is_88_0_0`
  - `changelog_has_v88_0_0`
  - `milestone_has_sap_sales`
  - `sap_odata_rune_has_sales_order_type`

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `fav/Cargo.toml` | version を `87.0.0` → `88.0.0` に更新 |
| `fav/src/driver.rs` | 既存 `cargo_toml_version_is_` テストを `88.0.0` に一括更新 + `mod v88000_tests` 追加 |
| `CHANGELOG.md` | v88.0.0 エントリを先頭に追加 |
| `MILESTONE.md` | SAP Sales 1.0 マイルストーンを先頭に追加 |
| `README.md` | 最新バージョン情報を更新 |
| `versions/current.md` | v88.0.0 に更新 |
