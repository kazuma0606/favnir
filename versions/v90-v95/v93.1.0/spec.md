# Spec: v93.1.0 — EDMX XML パーサー基盤（`parse_edmx`）

Status: TODO

---

## Background

SAP OData の `$metadata` エンドポイントは EDMX XML 形式でエンティティ型定義を返す。
v93.1.0 では、この EDMX XML を解析して Favnir の型定義へ変換するパイプラインの基盤となる
`parse_edmx` 関数と関連構造体を `fav/src/sap_metadata.rs` に実装する。

v10.8.0 で実装した `fav infer --from snowflake --table <name>` と同じ `--from <source>` パターンを
SAP に適用する第一歩。

---

## EDMX の例（解析対象）

```xml
<edmx:Edmx Version="4.0">
  <edmx:DataServices>
    <Schema Namespace="API_BUSINESS_PARTNER">
      <EntityType Name="A_BusinessPartnerType">
        <Key><PropertyRef Name="BusinessPartner"/></Key>
        <Property Name="BusinessPartner" Type="Edm.String" MaxLength="10"/>
        <Property Name="BusinessPartnerName" Type="Edm.String" MaxLength="81"/>
        <Property Name="Country" Type="Edm.String" MaxLength="3"/>
      </EntityType>
    </Schema>
  </edmx:DataServices>
</edmx:Edmx>
```

---

## Goals

1. `fav/src/sap_metadata.rs`（新規作成）に以下を実装する:
   - `EdmxProperty` 構造体（`name: String`, `edm_type: String`）
   - `EdmxEntityType` 構造体（`name: String`, `properties: Vec<EdmxProperty>`）
   - `EdmxSchema` 構造体（`namespace: String`, `entity_types: Vec<EdmxEntityType>`）
   - `parse_edmx(xml: &str) -> Vec<EdmxSchema>` 関数（文字列ベースのシンプルなパーサー）
2. `fav/src/main.rs` に `mod sap_metadata;` を追加する（既存の `mod` 宣言のアルファベット順に合わせて挿入）
3. `fav/src/driver.rs` に `mod v93100_tests`（2 件）を追加する

---

## 実装仕様

### 構造体

```rust
pub struct EdmxProperty {
    pub name:     String,
    pub edm_type: String,   // "Edm.String" / "Edm.Int32" / "Edm.Boolean" 等
}

pub struct EdmxEntityType {
    pub name:       String,
    pub properties: Vec<EdmxProperty>,
}

pub struct EdmxSchema {
    pub namespace:    String,
    pub entity_types: Vec<EdmxEntityType>,
}
```

### `parse_edmx`

```rust
pub fn parse_edmx(xml: &str) -> Vec<EdmxSchema>
```

- v93.1.0 はスタブ実装（空ベクタ返し）でよい
- v93.2.0 以降で段階的に実装する
- 外部クレート（`quick-xml` 等）は使わない（stdlib + 文字列処理のみ）

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/sap_metadata.rs` | 新規作成（構造体 3 件 + `parse_edmx` 関数） |
| `fav/src/main.rs` | `mod sap_metadata;` を追加 |
| `fav/src/driver.rs` | `mod v93100_tests` を追加（2 件） |

---

## Success Criteria

- `cargo test` 全 pass: **4,122 tests, 0 failures**（4,120 + 2）
- `mod v93100_tests` 内の 2 テストが pass する:
  - `sap_metadata_file_exists`: `fav/src/sap_metadata.rs` が存在する
  - `parse_edmx_function_defined`: `sap_metadata.rs` に `parse_edmx` が含まれる

---

## Note

> **テスト数**: ロードマップ計画値は 4,109（4,107+2）だが、v93.0.0 の実測が 4,120 のため、本バージョンは 4,120 + 2 = **4,122** が目標。

> **外部クレート不使用**: `quick-xml` 等の XML パーサーは Cargo.lock を変更し WASM ビルドに影響する可能性があるため、v93.1.0 では文字列ベースのスタブ実装に留める。本格的な XML 解析は v93.5.0 以降で追加する。

> **`parse_edmx` スタブ**: v93.1.0 では `Vec::new()` を返すスタブで十分。テストはファイル存在と関数名の含有のみ確認する。
