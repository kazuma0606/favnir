# v76.4.0 仕様書 — OpenLineage 統合強化

Date: 2026-08-15
Status: 計画中

---

## Background

`ProvenanceTag`（v76.1.0）を OpenLineage 標準ファセット形式に変換し、標準的なリネージ追跡ツール（Marquez、Apache Atlas 等）と統合する基盤を提供する。`provenance_to_openlineage` でファセット構造体を生成し、`format_openlineage_json` でシリアライズされた JSON 文字列を出力する（外部依存なしの手書き JSON フォーマット）。

---

## Goals

1. `OpenLineageFacet` 構造体（producer: String, data_source_uri: String, transforms: Vec<String>）を追加する
2. `provenance_to_openlineage(tag: &ProvenanceTag) -> OpenLineageFacet` を追加する
3. `format_openlineage_json(facet: &OpenLineageFacet) -> String` — JSON 文字列生成を追加する
4. Rust テスト 2 件を追加し 3722 tests に到達する

---

## 型・関数仕様

### `OpenLineageFacet` 構造体

```rust
#[derive(Debug, Clone)]
pub struct OpenLineageFacet {
    pub producer:        String,
    pub data_source_uri: String,
    pub transforms:      Vec<String>,
}
```

---

### `provenance_to_openlineage`

```rust
pub fn provenance_to_openlineage(tag: &ProvenanceTag) -> OpenLineageFacet
```

**動作:**
- `producer`: `"favnir/v76"` 固定
- `data_source_uri`: `tag.source.uri.clone()`
- `transforms`: `tag.transforms.clone()`

---

### `format_openlineage_json`

```rust
pub fn format_openlineage_json(facet: &OpenLineageFacet) -> String
```

**出力フォーマット（外部依存なしの手書き JSON）:**

```json
{"_producer":"<producer>","dataSource":{"uri":"<data_source_uri>"},"transforms":[<"t1","t2",...>]}
```

- `transforms` の各要素はダブルクォートで囲む
- 空の場合は `[]`
- プリティプリントは不要（ミニファイ形式）

**例（transforms あり）:**
```json
{"_producer":"favnir/v76","dataSource":{"uri":"snowflake://crm/users"},"transforms":["mask_pii","normalize_email"]}
```

**例（transforms なし）:**
```json
{"_producer":"favnir/v76","dataSource":{"uri":"s3://bucket"},"transforms":[]}
```

---

## テスト仕様

### `openlineage_facet_from_provenance`

```rust
let source = DataSource {
    name: "crm".to_string(),
    uri: "snowflake://crm/users".to_string(),
    source_type: DataSourceType::Snowflake,
};
let tag = ProvenanceTag {
    source,
    transforms: vec!["mask_pii".to_string(), "normalize_email".to_string()],
    pii: false,
};
let facet = provenance_to_openlineage(&tag);
assert_eq!(facet.producer, "favnir/v76");
assert_eq!(facet.data_source_uri, "snowflake://crm/users");
assert_eq!(facet.transforms.len(), 2);
assert!(facet.transforms.contains(&"mask_pii".to_string()));
```

### `openlineage_json_format`

```rust
let facet = OpenLineageFacet {
    producer:        "favnir/v76".to_string(),
    data_source_uri: "snowflake://crm/users".to_string(),
    transforms:      vec!["mask_pii".to_string()],
};
let json = format_openlineage_json(&facet);
assert!(json.contains("\"_producer\""));
assert!(json.contains("\"favnir/v76\""));
assert!(json.contains("\"dataSource\""));
assert!(json.contains("snowflake://crm/users"));
assert!(json.contains("\"mask_pii\""));

// 空 transforms
let facet2 = OpenLineageFacet {
    producer:        "favnir/v76".to_string(),
    data_source_uri: "s3://bucket".to_string(),
    transforms:      vec![],
};
assert!(format_openlineage_json(&facet2).contains("\"transforms\":[]"));
```

---

## Success Criteria

- `OpenLineageFacet` 構造体が定義されている
- `provenance_to_openlineage` が `producer="favnir/v76"`・URI・transforms を正しくマッピングする
- `format_openlineage_json` が `_producer`・`dataSource`・`transforms` を含む JSON を返す
- 空 transforms は `"transforms":[]` を出力する
- `openlineage_facet_from_provenance` が pass
- `openlineage_json_format` が pass
- `cargo test` が 3722 tests all pass
- `CHANGELOG.md` の先頭に v76.4.0 エントリが存在する

---

## 変更ファイル

- `fav/src/driver.rs` — `OpenLineageFacet`, `provenance_to_openlineage`, `format_openlineage_json`, `v764000_tests` を追加
- `CHANGELOG.md` — v76.4.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `76.3.0` → `76.4.0` に更新

---

## 依存（既実装）

- `DataSource` 構造体（v76.1.0）
- `DataSourceType` enum（v76.1.0）
- `ProvenanceTag` 構造体（v76.1.0）

---

## 対象外

- 外部 JSON ライブラリ（serde_json）への依存（手書き JSON フォーマットのみ）
- OpenLineage API との実際の HTTP 送信（将来バージョン）
- `columnLineage` ファセット（将来バージョン）
- 入力値の JSON 特殊文字（`"` / `\`）のエスケープ処理（入力は ASCII 英数字・記号のみを想定）
- site/ MDX 追加（v77.0 宣言バージョンでまとめて追加予定）
