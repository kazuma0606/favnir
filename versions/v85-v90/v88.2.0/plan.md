# Plan: v88.2.0 — `material_by_id()` + `MaterialType` enum 完全化

## 実装ステップ

### Step 1: `runes/sap-odata/material.fav` に `material_by_id` を追加

`materials()` 関数の直後に追加:

```favnir
-- 単一品目取得（v88.2.0）
public fn material_by_id(cfg: SapConfig, material_id: String) -> Result<Material, String> {
    Result.err("not implemented")
}
```

### Step 2: `runes/sap-odata/sap_odata.fav` を更新（re-export）

`materials()` ラッパーの直後に追加:

```favnir
public fn material_by_id(cfg: SapConfig, material_id: String) -> Result<Material, String> {
    material.material_by_id(cfg, material_id)
}
```

Note: T2 は手作業確認（Rust テストの対象外）

### Step 3: `fav/src/driver.rs` に `mod v88200_tests` を追加

`mod v88100_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v88200_tests {
    #[test]
    fn material_by_id_function_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/material.fav")
            .expect("runes/sap-odata/material.fav should exist");
        assert!(content.contains("public fn material_by_id("), "material_by_id should be defined in material.fav");
    }
    #[test]
    fn material_type_enum_has_all_variants() {
        let content = std::fs::read_to_string("../runes/sap-odata/material.fav")
            .expect("runes/sap-odata/material.fav should exist");
        assert!(content.contains("FinishedProduct"), "MaterialType should have FinishedProduct variant");
        assert!(content.contains("RawMaterial"), "MaterialType should have RawMaterial variant");
        assert!(content.contains("SemiFinished"), "MaterialType should have SemiFinished variant");
        assert!(content.contains("Trading"), "MaterialType should have Trading variant");
        assert!(content.contains("Service"), "MaterialType should have Service variant");
    }
}
```

### Step 4: `cargo test` で全 pass 確認

3,999 + 2 = 4,001 tests, 0 failures を確認する。

---

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
