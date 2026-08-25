# Plan: v88.1.0 — `Material` 型定義 + `MaterialFilter` + `materials()`

## 実装ステップ

### Step 1: `runes/sap-odata/material.fav` を新規作成

ファイル先頭に `use sap_odata.types` を追加し、以下を定義する:

1. `public type MaterialType = FinishedProduct | RawMaterial | SemiFinished | Trading | Service`
2. `public type Material = { material_id, description, material_type, base_unit, weight, weight_unit, plant }`
3. `public type MaterialFilter = { material_type, plant, top }`
4. `public fn materials(cfg: SapConfig, filter: MaterialFilter) -> Result<List<Material>, String>` スタブ

### Step 2: `runes/sap-odata/sap_odata.fav` を更新（re-export）

- `use sap_odata.sales_report` の直後に `use sap_odata.material` を追加する
- `format_sales_report` ラッパーの直後に以下を追加:
  ```favnir
  public type MaterialType   = material.MaterialType
  public type Material       = material.Material
  public type MaterialFilter = material.MaterialFilter
  public fn materials(cfg: SapConfig, filter: MaterialFilter) -> Result<List<Material>, String> {
      material.materials(cfg, filter)
  }
  ```
- Note: T2 は手作業確認（Rust テストの対象外）

### Step 3: `fav/src/driver.rs` に `mod v88100_tests` を追加

`mod v88000_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v88100_tests {
    #[test]
    fn material_type_defined_in_rune() {
        let content = std::fs::read_to_string("../runes/sap-odata/material.fav")
            .expect("runes/sap-odata/material.fav should exist");
        assert!(content.contains("MaterialType"), "MaterialType should be defined in material.fav");
    }
    #[test]
    fn materials_function_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/material.fav")
            .expect("runes/sap-odata/material.fav should exist");
        assert!(content.contains("public fn materials("), "materials function should be defined in material.fav");
    }
}
```

### Step 4: `cargo test` で全 pass 確認

3,997 + 2 = 3,999 tests, 0 failures を確認する。

---

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
