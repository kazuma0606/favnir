# v76.2.0 仕様書 — `TracedData` 型

Date: 2026-08-15
Status: 計画中

---

## Background

v76.1.0 で実装した `ProvenanceTag` を使って、データに来歴を付けて持ち回る `TracedData` 型を提供する。変換を経ても来歴が追跡でき、パイプラインの join 時には `merge_provenance` で複数来歴をマージする（pii は OR で伝播する）。

---

## Goals

1. `TracedData` 構造体（data: String, provenance: ProvenanceTag）を追加する
2. `map_traced(t: TracedData, transform_label: &str) -> TracedData` — 変換ラベルを provenance.transforms に追記する
3. `merge_provenance(a: &ProvenanceTag, b: &ProvenanceTag) -> ProvenanceTag` — join 時の来歴マージ（pii は OR）を追加する
4. Rust テスト 2 件を追加し 3718 tests に到達する

---

## 型・関数仕様

### `TracedData` 構造体

```rust
#[derive(Debug, Clone)]
pub struct TracedData {
    pub data:       String,       // シリアライズ済みデータ（変換で変化しない）
    pub provenance: ProvenanceTag,
}
```

---

### `map_traced`

```rust
pub fn map_traced(t: TracedData, transform_label: &str) -> TracedData
```

**動作:**
- `t.provenance.transforms` に `transform_label` を追記した新しい `TracedData` を返す
- transforms は呼び出し順に追記される（FIFO — 最初の呼び出しが先頭）
- `data` フィールドはそのまま保持する（変換済みデータの置き換えは呼び出し側の責任）
- `pii` フラグはそのまま保持する（PII フラグの変更は呼び出し側の責任）

**例:**
```
map_traced(t, "mask_pii") → t.provenance.transforms に "mask_pii" が追加される
map_traced(t2, "normalize_email") → transforms = ["mask_pii", "normalize_email"]
```

---

### `merge_provenance`

```rust
pub fn merge_provenance(a: &ProvenanceTag, b: &ProvenanceTag) -> ProvenanceTag
```

**動作:**
- `source`: `a.source` をベースとする（join 左辺のソースを優先）
- `transforms`: `a.transforms` + `b.transforms` を連結
- `pii`: `a.pii || b.pii`（一方でも PII あれば merged は PII あり）

---

## テスト仕様

### `traced_map_appends_transform`

```rust
// 初期 TracedData を作成
let source = DataSource { name: "crm", uri: "...", source_type: DataSourceType::Snowflake };
let tag = ProvenanceTag { source, transforms: vec![], pii: false };
let t = TracedData { data: "rows".to_string(), provenance: tag };

// 1 回変換
let t2 = map_traced(t, "mask_pii");
assert!(t2.provenance.transforms.contains(&"mask_pii".to_string()));
assert_eq!(t2.provenance.transforms.len(), 1);

// 2 回変換
let t3 = map_traced(t2, "normalize_email");
assert_eq!(t3.provenance.transforms.len(), 2);
assert!(t3.provenance.transforms.contains(&"normalize_email".to_string()));

// data は変化しない
assert_eq!(t3.data, "rows");
```

### `traced_merge_propagates_pii`

```rust
let src_a = DataSource { name: "a", uri: "s3://a", source_type: DataSourceType::S3 };
let src_b = DataSource { name: "b", uri: "s3://b", source_type: DataSourceType::Manual };

// pii=false + pii=true → merged pii=true
let tag_a = ProvenanceTag { source: src_a.clone(), transforms: vec!["t1".to_string()], pii: false };
let tag_b = ProvenanceTag { source: src_b.clone(), transforms: vec!["t2".to_string()], pii: true };
let merged = merge_provenance(&tag_a, &tag_b);
assert!(merged.pii, "pii propagates via OR");
assert_eq!(merged.transforms.len(), 2); // t1 + t2

// pii=false + pii=false → merged pii=false
let tag_c = ProvenanceTag { source: src_a, transforms: vec![], pii: false };
let tag_d = ProvenanceTag { source: src_b, transforms: vec![], pii: false };
let merged2 = merge_provenance(&tag_c, &tag_d);
assert!(!merged2.pii, "both false → false");
```

---

## Success Criteria

- `TracedData` 構造体が定義されている
- `map_traced` が変換ラベルを追記した新しい `TracedData` を返す（data・pii は不変、transforms は FIFO 順）
- `merge_provenance` が pii を OR で伝播し、transforms を連結する
- `traced_map_appends_transform` が pass
- `traced_merge_propagates_pii` が pass
- `cargo test` が 3718 tests all pass
- `CHANGELOG.md` の先頭に v76.2.0 エントリが存在する

---

## 変更ファイル

- `fav/src/driver.rs` — `TracedData`, `map_traced`, `merge_provenance`, `v762000_tests` を追加
- `CHANGELOG.md` — v76.2.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `76.1.0` → `76.2.0` に更新

---

## 依存（既実装）

- `DataSource` 構造体（v76.1.0）— `#[derive(Debug, Clone)]` 済み（`src_a.clone()` テスト使用のため必須）
- `DataSourceType` enum（v76.1.0）
- `ProvenanceTag` 構造体（v76.1.0）

---

## 対象外

- `TracedData.data` の実際のデータ変換（変換後データの置き換えは呼び出し側の責任）
- PII フラグの自動変更（`map_traced` は pii を変更しない）
- `ErasurePlan` / `PiiProvenanceReport`（v76.3.0 で実装）
