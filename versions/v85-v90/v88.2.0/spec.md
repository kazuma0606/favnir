# Spec: v88.2.0 — `material_by_id()` + `MaterialType` enum 完全化

## Background

v88.1.0 で `Material` 型・`MaterialFilter`・`materials()` を実装した。
本バージョンでは単一品目取得関数 `material_by_id()` を `material.fav` に追加し、
`MaterialType` 全バリアント（5件）の存在を Rust テストで担保する。

## Goals

1. `material.fav` に `public fn material_by_id(cfg, material_id)` スタブを追加する
2. `sap_odata.fav` に `material_by_id` の re-export を追加する（手作業確認）
3. `MaterialType` の全バリアント（`FinishedProduct` の存在）を Rust テストで担保する

## API / Syntax Examples

```favnir
-- material.fav に追加

-- 単一品目取得（v88.2.0）
public fn material_by_id(cfg: SapConfig, material_id: String) -> Result<Material, String> {
    Result.err("not implemented")
}
```

## Success Criteria（Rust テストで担保）

- `runes/sap-odata/material.fav` に以下を含む:
  - `"public fn material_by_id("` — 単一品目取得関数
  - `"FinishedProduct"` — MaterialType の主バリアント
- `cargo test` で 4,001 tests, 0 failures
- Rust テスト 2 件:
  - `material_by_id_function_exists`（`material.fav` に `"public fn material_by_id("` を確認）
  - `material_type_enum_has_all_variants`（`material.fav` に 5 バリアント全て `FinishedProduct` / `RawMaterial` / `SemiFinished` / `Trading` / `Service` を確認）

## 手作業確認項目（Rust テスト対象外）

- `sap_odata.fav` に `public fn material_by_id(...)` ラッパーが追加されているか
  - Note: `sap_odata.fav` の re-export は `material.fav` の関数を呼び出す薄いラッパーであり、`material.fav` 側の Rust テストで機能が間接的に保証されるため Rust テスト対象外とする（v88.1.0 からの方針）

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `runes/sap-odata/material.fav` | 追記（`material_by_id` 関数追加） |
| `runes/sap-odata/sap_odata.fav` | 追記（`material_by_id` re-export）※手作業確認のみ、Rust テストなし |
| `fav/src/driver.rs` | `mod v88200_tests` 追加 |
