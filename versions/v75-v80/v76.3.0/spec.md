# v76.3.0 仕様書 — PII 来歴追跡・GDPR 消去計画

Date: 2026-08-15
Status: 計画中

---

## Background

`ProvenanceTag`（v76.1.0）の `pii` フラグを活用して、PII を含むデータの流れを追跡し、GDPR「忘れられる権利」への対応計画を自動生成する。`detect_pii_in_tag` は `pii=true` のタグから PII マーカーを返し、`generate_erasure_plan` は消去対象の URI と理由を `ErasurePlan` として生成する。

---

## Goals

1. `PiiProvenanceReport` 構造体（fields: Vec<String>, source_uri: String, masked: bool）を追加する
2. `detect_pii_in_tag(tag: &ProvenanceTag) -> Vec<String>` — pii=true のタグから PII マーカーを返す（pii=false は空 Vec）
3. `ErasurePlan` 構造体（target_uri: String, fields: Vec<String>, reason: String）を追加する
4. `generate_erasure_plan(tag: &ProvenanceTag) -> Option<ErasurePlan>` — pii=true なら消去計画を生成、pii=false なら None を返す
5. Rust テスト 2 件を追加し 3720 tests に到達する

---

## 型・関数仕様

### `PiiProvenanceReport` 構造体

```rust
#[derive(Debug, Clone)]
pub struct PiiProvenanceReport {
    pub fields:     Vec<String>,
    pub source_uri: String,
    pub masked:     bool,
}
```

**注:** 本バージョンでは `detect_pii_in_tag` / `generate_erasure_plan` から直接使用しない準備型。将来の `fav lineage pii-report` CLI 統合で使用する。`pub struct` なので Rust の `dead_code` 警告は発生しない。

---

### `detect_pii_in_tag`

```rust
pub fn detect_pii_in_tag(tag: &ProvenanceTag) -> Vec<String>
```

**動作:**
- `tag.pii == true` の場合: `vec!["pii_detected".to_string()]` を返す（PII 存在の sentinel）
- `tag.pii == false` の場合: `vec![]` を返す（PII なし）

---

### `ErasurePlan` 構造体

```rust
#[derive(Debug, Clone)]
pub struct ErasurePlan {
    pub target_uri: String,
    pub fields:     Vec<String>,
    pub reason:     String,
}
```

---

### `generate_erasure_plan`

```rust
pub fn generate_erasure_plan(tag: &ProvenanceTag) -> Option<ErasurePlan>
```

**動作:**
- `tag.pii == true` の場合:
  ```
  Some(ErasurePlan {
      target_uri: tag.source.uri.clone(),
      fields:     detect_pii_in_tag(tag),
      reason:     "GDPR erasure request".to_string(),
  })
  ```
- `tag.pii == false` の場合: `None`

---

## テスト仕様

### `pii_detected_in_provenance`

```rust
let src = DataSource { name: "crm", uri: "snowflake://crm/users", source_type: Snowflake };
let pii_tag = ProvenanceTag { source: src.clone(), transforms: vec![], pii: true };
let clean_tag = ProvenanceTag { source: src, transforms: vec![], pii: false };

let detected = detect_pii_in_tag(&pii_tag);
assert!(!detected.is_empty(), "pii=true → non-empty");

let clean = detect_pii_in_tag(&clean_tag);
assert!(clean.is_empty(), "pii=false → empty");
```

### `gdpr_erasure_plan_generated`

```rust
let src = DataSource { name: "crm", uri: "snowflake://crm/users", source_type: Snowflake };
let pii_tag = ProvenanceTag { source: src.clone(), transforms: vec![], pii: true };
let clean_tag = ProvenanceTag { source: src, transforms: vec![], pii: false };

let plan = generate_erasure_plan(&pii_tag);
assert!(plan.is_some(), "pii=true → Some");
let plan = plan.unwrap();
assert!(plan.target_uri.contains("snowflake://crm/users"));
assert!(!plan.fields.is_empty());
assert!(plan.reason.contains("GDPR"));

assert!(generate_erasure_plan(&clean_tag).is_none(), "pii=false → None");
```

---

## Success Criteria

- `PiiProvenanceReport` 構造体が定義されている
- `ErasurePlan` 構造体が定義されている
- `detect_pii_in_tag(pii=true_tag)` が非空 Vec を返す
- `detect_pii_in_tag(pii=false_tag)` が空 Vec を返す
- `generate_erasure_plan(pii=true_tag)` が `Some(ErasurePlan)` を返し、`target_uri` にソース URI、`reason` に "GDPR" を含む
- `generate_erasure_plan(pii=false_tag)` が `None` を返す
- `pii_detected_in_provenance` が pass
- `gdpr_erasure_plan_generated` が pass
- `cargo test` が 3720 tests all pass
- `cargo check` が警告なしで完了する（`PiiProvenanceReport` は `pub struct` のため dead_code 警告なし）
- `CHANGELOG.md` の先頭に v76.3.0 エントリが存在する

---

## 変更ファイル

- `fav/src/driver.rs` — `PiiProvenanceReport`, `detect_pii_in_tag`, `ErasurePlan`, `generate_erasure_plan`, `v763000_tests` を追加
- `CHANGELOG.md` — v76.3.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `76.2.0` → `76.3.0` に更新

---

## 依存（既実装）

- `DataSource` 構造体（v76.1.0）— `#[derive(Debug, Clone)]` 済み
- `DataSourceType` enum（v76.1.0）
- `ProvenanceTag` 構造体（v76.1.0）— `pii: bool` フィールドを持つ

---

## 対象外

- PII フィールド名の詳細分類（`email`、`phone` 等の具体名）— sentinel `"pii_detected"` のみ（将来バージョンで拡張）
- 実際の削除処理（消去計画の生成のみ、実行は対象外）
- OpenLineage 統合（v76.4.0 で実装）
