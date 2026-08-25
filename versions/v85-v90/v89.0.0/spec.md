# Spec: v89.0.0 — SAP Procurement 1.0 宣言

## Background

v88.1.0〜v88.9.0 で SAP Procurement 関連の全型定義・関数スタブ・E2E デモ Lambda 基盤が揃い、
安定化コードフリーズが完了した。
本バージョンは SAP Procurement 1.0 の宣言バージョン。クリーンアップを実施し、
Cargo.toml バージョンを 89.0.0 に更新する。

## 宣言文

> 「在庫と発注が型になった。
>  Material と PurchaseOrder を並べれば、不足が見える。」

## Goals

1. `cargo clean` でビルドキャッシュを削除する
2. `fav/Cargo.toml` バージョンを `88.0.0` → `89.0.0` に更新する
3. `fav/src/driver.rs` 内の `"88.0.0"` 文字列（約40箇所）を `"89.0.0"` に一括更新する（うち `cargo_toml_version` テスト関数は 33 件、残りはアサーションメッセージ等）
4. `CHANGELOG.md` に v89.0.0 エントリを追加する
5. `MILESTONE.md` に SAP Procurement 1.0 マイルストーンを追加する
6. `README.md` を v89.0.0 に更新する
7. `versions/current.md` を v89.0.0 に更新する
8. `driver.rs` に `mod v89000_tests` を追加する（4 件）

## Success Criteria（Rust テストで担保）

- `cargo_toml_version_is_89_0_0`: `Cargo.toml` に `version = "89.0.0"` を含む
- `changelog_has_v89_0_0`: `CHANGELOG.md` に `[v89.0.0]` を含む
- `milestone_has_sap_procurement`: `MILESTONE.md` に `SAP Procurement` を含む
- `sap_odata_rune_has_material_type`: `runes/sap-odata/sap_odata.fav` に `Material` を含む（`std::fs::read_to_string("../runes/sap-odata/sap_odata.fav")` — cargo test CWD は `fav/` のため `../` で親ディレクトリへ移動）
- `cargo test` で 4,019 tests, 0 failures（4,015 + 4）
  ベース 4,015 は v88.9.0 完了時点の値（`roadmap-v88.1-v89.0.md` テスト数推移表参照）

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `fav/Cargo.toml` | version を `88.0.0` → `89.0.0` に更新 |
| `fav/src/driver.rs` | `"88.0.0"` → `"89.0.0"` 一括更新 + `mod v89000_tests` 追加 |
| `CHANGELOG.md` | v89.0.0 エントリ追加（先頭に挿入） |
| `MILESTONE.md` | SAP Procurement 1.0 マイルストーン追加（先頭に挿入） |
| `README.md` | v89.0.0 に更新 |
| `versions/current.md` | v89.0.0 に更新 |

**Note**: `cargo clean` 後は `fav/tmp/hello.fav` が残ることを確認する（`cargo clean` は `target/` のみ削除）。
`hello.fav` が消えた場合は `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }` で復元する。
