# Spec: v88.1.0 — `Material` 型定義 + `MaterialFilter` + `materials()`

## Background

v88.0.0 で SAP Sales 1.0 を宣言し、Sprint 4（SAP Procurement 1.0）を開始する。
本バージョンでは品目マスタ（`Material`）の Favnir 型・フィルタ型・一覧取得関数を実装する。
`runes/sap-odata/material.fav` を新規作成し、`sap_odata.fav` に re-export する。

## Goals

1. `runes/sap-odata/material.fav` を新規作成する
2. `MaterialType` enum（5 バリアント）を定義する
3. `Material` 型（品目マスタ）を定義する
4. `MaterialFilter` 型（フィルタ条件）を定義する
5. `public fn materials(cfg, filter)` 関数スタブを追加する
6. `sap_odata.fav` に re-export を追加する（手作業確認）

## API / Syntax Examples

```favnir
-- runes/sap-odata/material.fav

use sap_odata.types

public type MaterialType = FinishedProduct | RawMaterial | SemiFinished | Trading | Service

public type Material = {
    material_id:   String,
    description:   String,
    material_type: MaterialType,
    base_unit:     String,
    weight:        Option<Float>,
    weight_unit:   Option<String>,
    plant:         Option<String>
}

public type MaterialFilter = {
    material_type: Option<MaterialType>,
    plant:         Option<String>,
    top:           Option<Int>
}

public fn materials(cfg: SapConfig, filter: MaterialFilter) -> Result<List<Material>, String> {
    Result.err("not implemented")
}
```

## Success Criteria（Rust テストで担保）

- `runes/sap-odata/material.fav` が存在し、以下を含む:
  - `"MaterialType"`（MaterialType 型定義の確認）
  - `"public fn materials("` — materials 関数
- `cargo test` で 3,999 tests, 0 failures
- Rust テスト 2 件:
  - `material_type_defined_in_rune`（`material.fav` に `"MaterialType"` が含まれることを確認）
  - `materials_function_exists`（`material.fav` に `"public fn materials("` が含まれることを確認）

## 手作業確認項目（Rust テスト対象外）

- `sap_odata.fav` が `use sap_odata.material` を含み、`MaterialType` / `Material` / `MaterialFilter` / `materials` を re-export している

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `runes/sap-odata/material.fav` | **新規作成** |
| `runes/sap-odata/sap_odata.fav` | 追記（`use` + re-export 4 件） |
| `fav/src/driver.rs` | `mod v88100_tests` 追加 |
