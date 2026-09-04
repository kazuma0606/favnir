# Spec: v93.2.0 — `EntityType` → Favnir `type` 変換

Status: TODO

---

## Background

v93.1.0 で `parse_edmx` スタブと `EdmxEntityType` / `EdmxProperty` 構造体を定義した。
v93.2.0 では `EdmxEntityType` を Favnir の `type` 定義文字列に変換する2つの関数を実装する。

---

## 変換例

```
-- 入力: EdmxEntityType
EdmxEntityType {
    name: "A_BusinessPartnerType",
    properties: [
        EdmxProperty { name: "BusinessPartner",     edm_type: "Edm.String" },
        EdmxProperty { name: "BusinessPartnerName", edm_type: "Edm.String" },
        EdmxProperty { name: "Country",             edm_type: "Edm.String" },
    ]
}

-- 出力: Favnir 型定義文字列（アライメントなし。v93.7.0 の fav fmt 適用後に整形される）
type BusinessPartner = {
    BusinessPartner: String,
    BusinessPartnerName: String,
    Country: String
}
```

---

## Goals

1. `fav/src/sap_metadata.rs` に `edm_type_to_favnir` 関数を追加する
2. `fav/src/sap_metadata.rs` に `entity_type_to_favnir` 関数を追加する
3. `fav/src/driver.rs` に `mod v93200_tests`（2 件）を追加する

---

## EDM → Favnir 型マッピング

| EDM 型 | Favnir 型 |
|---|---|
| `Edm.String` | `String` |
| `Edm.Int32` / `Edm.Int64` | `Int` |
| `Edm.Decimal` | `Float` |
| `Edm.Boolean` | `Bool` |
| `Edm.DateTimeOffset` | `String` |
| `Edm.Guid` | `String` |
| その他 | `String`（フォールバック） |

## エンティティ名変換

SAP の EntityType 名（例: `A_BusinessPartnerType`）を Favnir 型名に変換するルール:
1. 先頭の「1 文字 + `_`」パターン（`A_` / `I_` / `C_` / `Z_` 等すべて）を除去
2. 末尾の `Type` サフィックスを除去

例: `A_BusinessPartnerType` → `BusinessPartner`

> **実装方針**: プレフィックス除去は特定文字の列挙ではなく、2 文字目が `_` かどうかで汎用判定する（`et.name.as_bytes()[1] == b'_'`）。

---

## 実装仕様

### `edm_type_to_favnir`

```rust
pub fn edm_type_to_favnir(edm_type: &str) -> &'static str {
    match edm_type {
        "Edm.String" | "Edm.DateTimeOffset" | "Edm.Guid" => "String",
        "Edm.Int32" | "Edm.Int64"                        => "Int",
        "Edm.Decimal"                                    => "Float",
        "Edm.Boolean"                                    => "Bool",
        _                                                => "String",
    }
}
```

### `entity_type_to_favnir`

```rust
pub fn entity_type_to_favnir(et: &EdmxEntityType) -> String
```

- エンティティ名からプレフィックス（`A_` / `I_` / `C_`）と末尾 `Type` を除去して Favnir 型名を生成
- 各 `EdmxProperty` を `FieldName: FavnirType,` 形式でインデント付き出力
- 戻り値: `"type FavnirName = {\n    Field: Type,\n    ...\n}"` 形式の文字列

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/sap_metadata.rs` | `edm_type_to_favnir` / `entity_type_to_favnir` を追加 |
| `fav/src/driver.rs` | `mod v93200_tests` を追加（2 件） |

---

## Success Criteria

- `cargo test` 全 pass: **4,124 tests, 0 failures**（4,122 + 2）
- `mod v93200_tests` 内の 2 テストが pass する:
  - `entity_type_to_favnir_defined`: `sap_metadata.rs` に `entity_type_to_favnir` が含まれる
  - `edm_type_to_favnir_defined`: `sap_metadata.rs` に `edm_type_to_favnir` が含まれる

---

## Note

> **テスト数**: ロードマップ計画値は 4,111（4,109+2）だが、実測ベースは 4,122 + 2 = **4,124** が目標。
