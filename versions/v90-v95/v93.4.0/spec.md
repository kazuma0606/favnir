# Spec: v93.4.0 — `EnumType` → Favnir `type E = | A | B` 変換

Status: TODO

---

## Background

v93.3.0 で `NavigationProperty` のコメント生成・ExpandClause ヘルパー関数生成を実装した。
v93.4.0 では EDMX の `EnumType`（列挙型）を Favnir の ADT 型（`type E = | A | B`）に変換する
構造体と関数を追加する。

---

## 変換例

```
-- 入力: EDMX EnumType
<EnumType Name="YY1_BPKIND_CODE">
  <Member Name="1" Value="1"/>
  <Member Name="2" Value="2"/>
  <Member Name="3" Value="3"/>
</EnumType>

-- 出力: Favnir ADT
type Yy1BpkindCode =
    | Val1
    | Val2
    | Val3
```

---

## Goals

1. `fav/src/sap_metadata.rs` に `EdmxEnumMember` 構造体を追加する
2. `fav/src/sap_metadata.rs` に `EdmxEnumType` 構造体を追加する
3. `fav/src/sap_metadata.rs` に `enum_type_to_favnir` 関数を追加する
4. `fav/src/driver.rs` に `mod v93400_tests`（2 件）を追加する

---

## 実装仕様

### 構造体

```rust
/// EDMX EnumType の各メンバー
#[derive(Debug)]
pub struct EdmxEnumMember {
    pub name: String,
}

/// EDMX EnumType（列挙型の型定義）
#[derive(Debug)]
pub struct EdmxEnumType {
    pub name: String,
    pub members: Vec<EdmxEnumMember>,
}
```

### 名前変換ルール

#### EnumType 名 → Favnir 型名

`SCREAMING_SNAKE_CASE` → `PascalCase`（単語区切りを `_` で分割し、各単語の先頭のみ大文字）

例:
- `YY1_BPKIND_CODE` → `Yy1BpkindCode`
- `SALES_ORDER_STATUS` → `SalesOrderStatus`

#### EnumMember 名 → Favnir バリアント名

- 先頭が ASCII 数字の場合: `Val` プレフィックスを付与
- それ以外: そのまま使用

例:
- `"1"` → `Val1`
- `"Active"` → `Active`

### `enum_type_to_favnir`

```rust
pub fn enum_type_to_favnir(et: &EdmxEnumType) -> String
```

- `et.name` を `screaming_snake_to_pascal` で型名に変換
- 各 `EdmxEnumMember` を `    | {variant}` 形式で出力
- 戻り値: `"type {Name} =\n    | Val1\n    | Val2"` 形式の文字列

### 内部ヘルパー `screaming_snake_to_pascal`

```rust
fn screaming_snake_to_pascal(s: &str) -> String
```

- `_` で分割し、各トークンの先頭を大文字・残りを小文字に変換して連結
- `YY1_BPKIND_CODE` → `["YY1", "BPKIND", "CODE"]` → `"Yy1"+"Bpkind"+"Code"` → `"Yy1BpkindCode"`

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/sap_metadata.rs` | `EdmxEnumMember` / `EdmxEnumType` 構造体 + `enum_type_to_favnir` / `screaming_snake_to_pascal` を追加 |
| `fav/src/driver.rs` | `mod v93400_tests` を追加（2 件） |

---

## Success Criteria

- `cargo test` 全 pass: **4,128 tests, 0 failures**（4,126 + 2）
- `mod v93400_tests` 内の 2 テストが pass する:
  - `enum_type_to_favnir_defined`: `sap_metadata.rs` に `enum_type_to_favnir` が含まれる
  - `edmx_enum_type_struct_defined`: `sap_metadata.rs` に `EdmxEnumType` が含まれる

---

## Note

> **テスト数**: ロードマップ計画値 4,128（4,126+2）は実測ベースと一致。

> **テストの網羅性**: 成功条件の 2 テストはソースファイル文字列の存在確認のみ。これはプロジェクト全体の既存パターン（v93.2.0〜v93.3.0 と同様）に合わせた最小限テストである。変換ロジックの正確性（`YY1_BPKIND_CODE` → `Yy1BpkindCode`、`"1"` → `Val1`）は将来のリグレッションテストで追加する。

> **WASM 影響**: `sap_metadata.rs` は pure Rust（外部クレート依存なし）のため WASM ビルドへの影響はない。
