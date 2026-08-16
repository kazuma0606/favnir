# v76.1.0 仕様書 — `DataSource` / `ProvenanceTag` 型基盤

Date: 2026-08-15
Status: 計画中

---

## Background

Data Provenance 1.0 スプリント（v76.1〜v77.0）の第一歩。「このデータはどのシステムから来たか」を型として表現する基盤を提供する。`DataSourceType` enum でデータソースの種別を型安全に分類し、`DataSource` 構造体でソースのメタデータを保持する。`ProvenanceTag` は変換履歴と PII フラグを合わせて管理し、`format_provenance_tag` で人間可読なレポート文字列を生成する。

---

## Goals

1. `DataSourceType` enum（Snowflake, S3, Api, Manual, Pipeline）を追加する
2. `DataSource` 構造体（name: String, uri: String, source_type: DataSourceType）を追加する
3. `ProvenanceTag` 構造体（source: DataSource, transforms: Vec<String>, pii: bool）を追加する
4. `format_provenance_tag(tag: &ProvenanceTag) -> String` を追加する
5. Rust テスト 2 件を追加し 3716 tests に到達する

---

## 型・関数仕様

### `DataSourceType` enum

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum DataSourceType {
    Snowflake,
    S3,
    Api,
    Manual,
    Pipeline,
}
```

---

### `DataSource` 構造体

```rust
#[derive(Debug, Clone)]
pub struct DataSource {
    pub name:        String,
    pub uri:         String,
    pub source_type: DataSourceType,
}
```

---

### `ProvenanceTag` 構造体

```rust
#[derive(Debug, Clone)]
pub struct ProvenanceTag {
    pub source:     DataSource,
    pub transforms: Vec<String>,
    pub pii:        bool,
}
```

---

### `format_provenance_tag`

```rust
pub fn format_provenance_tag(tag: &ProvenanceTag) -> String
```

**出力フォーマット:**

```
source=<name> type=<DataSourceType Debug> transforms=[<t1>,<t2>,...] pii=<true|false>
```

- `source=` にはソース名（`tag.source.name`）
- `type=` には `DataSourceType` の `#[derive(Debug)]` 出力（バリアント名そのまま: `Snowflake`, `S3`, `Api`, `Manual`, `Pipeline`）— Rust の `{:?}` フォーマット指定子を使用
- `transforms=` は `[t1,t2,...]` 形式（`Vec::join(",")` で生成、空の場合は `[]`）
- `pii=` は `true` または `false`

**例:**
```
source=crm type=Snowflake transforms=[mask_pii,normalize_email] pii=false
source=upload type=Manual transforms=[] pii=true
```

---

## テスト仕様

### `provenance_tag_created`

```rust
let source = DataSource {
    name: "crm".to_string(),
    uri: "snowflake://warehouse/crm/users".to_string(),
    source_type: DataSourceType::Snowflake,
};
let tag = ProvenanceTag {
    source,
    transforms: vec!["mask_pii".to_string(), "normalize_email".to_string()],
    pii: false,
};
let s = format_provenance_tag(&tag);
assert!(s.contains("crm"));
assert!(s.contains("Snowflake"));
assert!(s.contains("mask_pii"));
assert!(s.contains("pii=false"));
```

### `provenance_pii_flagged`

```rust
// PII あり・変換なし
let source2 = DataSource {
    name: "upload".to_string(),
    uri: "s3://bucket/upload".to_string(),
    source_type: DataSourceType::S3,
};
let tag_pii = ProvenanceTag { source: source2, transforms: vec![], pii: true };
let s_pii = format_provenance_tag(&tag_pii);
assert!(s_pii.contains("pii=true"));
assert!(s_pii.contains("S3"));
assert!(s_pii.contains("transforms=[]"));

// PII なし・変換なし
let source3 = DataSource {
    name: "api".to_string(),
    uri: "https://api.example.com".to_string(),
    source_type: DataSourceType::Api,
};
let tag_clean = ProvenanceTag { source: source3, transforms: vec![], pii: false };
assert!(format_provenance_tag(&tag_clean).contains("pii=false"));
```

---

## Success Criteria

- `DataSourceType` enum が定義されている（Snowflake / S3 / Api / Manual / Pipeline）
- `DataSource` 構造体が定義されている
- `ProvenanceTag` 構造体が定義されている
- `format_provenance_tag` が `source=<name> type=<variant> transforms=[...] pii=<bool>` 形式の文字列を返す（空 transforms → `transforms=[]`、`type=` はバリアント名そのまま）
- `provenance_tag_created` が pass
- `provenance_pii_flagged` が pass
- `cargo test` が 3716 tests all pass
- `CHANGELOG.md` の先頭に v76.1.0 エントリが存在する

---

## 変更ファイル

- `fav/src/driver.rs` — `DataSourceType`, `DataSource`, `ProvenanceTag`, `format_provenance_tag`, `v761000_tests` を追加
- `CHANGELOG.md` — v76.1.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `76.0.0` → `76.1.0` に更新

---

## 依存（既実装）

なし（新規型基盤）

---

## 対象外

- `TracedData` 型（v76.2.0 で実装）
- PII 来歴追跡・GDPR 消去計画（v76.3.0 で実装）
- OpenLineage 統合（v76.4.0 で実装）
- URI / ソース名の SQL インジェクション・形式検証（呼び出し側の責任）
