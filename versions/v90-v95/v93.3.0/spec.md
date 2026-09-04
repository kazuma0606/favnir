# Spec: v93.3.0 — `NavigationProperty` → `ExpandClause` フィールド生成

Status: TODO

---

## Background

v93.2.0 で `EdmxEntityType` → Favnir `type` 定義文字列の変換関数（`entity_type_to_favnir` / `edm_type_to_favnir`）を実装した。
v93.3.0 では EDMX の `NavigationProperty`（関連エンティティへのナビゲーション）を解析し、
コメント形式でフィールド候補を型定義に埋め込む `nav_property_to_favnir_comment` 関数と、
`ExpandClause` ヘルパー関数文字列を生成する `nav_to_expand_helper_fn` 関数を追加する。

---

## 変換例

```
-- 入力: EDMX NavigationProperty
<NavigationProperty Name="to_BusinessPartnerAddress"
    Type="Collection(API_BUSINESS_PARTNER.A_BusinessPartnerAddressType)"/>

-- 出力 1: nav_property_to_favnir_comment の戻り値
-- Navigation properties (use with ExpandClause):
-- "to_BusinessPartnerAddress"

-- 出力 2: nav_to_expand_helper_fn の戻り値
--   nav_to_expand_helper_fn("BusinessPartner", "to_BusinessPartnerAddress")
fn business_partner_expand_address() -> ExpandClause<BusinessPartner> {
    expand_nav<BusinessPartner>(["to_BusinessPartnerAddress"])
}
```

---

## Goals

1. `fav/src/sap_metadata.rs` に `nav_property_to_favnir_comment` 関数を追加する
2. `fav/src/sap_metadata.rs` に `nav_to_expand_helper_fn` 関数を追加する
3. `fav/src/driver.rs` に `mod v93300_tests`（2 件）を追加する

---

## 実装仕様

### `nav_property_to_favnir_comment`

```rust
/// ナビゲーションプロパティ名のリストを Favnir コメント文字列に変換する（v93.3.0）
/// nav_names が空のときは空文字列を返す
/// 出力例（改行区切り）:
///   "-- Navigation properties (use with ExpandClause):\n-- \"to_BusinessPartnerAddress\""
pub fn nav_property_to_favnir_comment(nav_names: &[&str]) -> String {
    if nav_names.is_empty() {
        return String::new();
    }
    let mut out = String::from("-- Navigation properties (use with ExpandClause):");
    for name in nav_names {
        out.push_str(&format!("\n-- \"{}\"", name));
    }
    out
}
```

### `nav_to_expand_helper_fn`

エンティティ名とナビゲーションプロパティ名から、`ExpandClause` ヘルパー関数文字列を生成する。

```rust
/// EntityType 名と NavigationProperty 名から ExpandClause ヘルパー関数文字列を生成する（v93.3.0）
/// 例:
///   nav_to_expand_helper_fn("BusinessPartner", "to_BusinessPartnerAddress")
///   →
///   "fn business_partner_expand_address() -> ExpandClause<BusinessPartner> {\n    expand_nav<BusinessPartner>([\"to_BusinessPartnerAddress\"])\n}"
pub fn nav_to_expand_helper_fn(entity_name: &str, nav_name: &str) -> String
```

- 関数名の命名ルール:
  - `entity_name` をスネークケースに変換（`BusinessPartner` → `business_partner`）
  - `nav_name` の先頭 `to_` プレフィックスを除去し残りをスネークケースに変換（`to_BusinessPartnerAddress` → `address`）
  - 関数名 = `{snake_entity}_expand_{snake_nav}`（例: `business_partner_expand_address`）
- 戻り型: `ExpandClause<{entity_name}>`
- 本体: `expand_nav<{entity_name}>(["nav_name"])`

### スネークケース変換ヘルパー（内部関数）

```rust
fn to_snake_case(s: &str) -> String
```

- 大文字の直前に `_` を挿入し全体を小文字化する（`BusinessPartner` → `business_partner`）
- 先頭の `_` は除去する

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/sap_metadata.rs` | `nav_property_to_favnir_comment` / `nav_to_expand_helper_fn` / `to_snake_case` を追加 |
| `fav/src/driver.rs` | `mod v93300_tests` を追加（2 件） |

---

## Success Criteria

- `cargo test` 全 pass: **4,126 tests, 0 failures**（4,124 + 2）
- `mod v93300_tests` 内の 2 テストが pass する:
  - `nav_property_parser_defined`: `sap_metadata.rs` に `nav_property_to_favnir_comment` が含まれる
  - `nav_property_generates_expand_helper`: `sap_metadata.rs` に `nav_to_expand_helper_fn` が含まれる

---

## Note

> **テスト数**: ロードマップ計画値は 4,113（4,111+2）だが、実測ベースは 4,124 + 2 = **4,126** が目標。
