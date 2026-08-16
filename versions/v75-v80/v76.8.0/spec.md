# v76.8.0 仕様書 — Provenance contracts

Date: 2026-08-15
Status: 計画中

---

## Background

コントラクトに来歴ポリシーを組み込む。「このパイプラインの入力は必ず Snowflake から来ること」「PII はマスク済みであること」を型で保証する。`PiiPolicy` enum で PII ポリシーの種別を表現し、`ProvenanceContract` でソース許容リストと PII ポリシーをまとめ、`validate_provenance_contract` で `ProvenanceTag` との整合性を検証する。

---

## Goals

1. `PiiPolicy` enum（MustBeMasked / AllowRaw / MustBeAbsent）を追加する
2. `ProvenanceContract` 構造体（allowed_sources: Vec<DataSourceType>, pii_policy: PiiPolicy）を追加する
3. `validate_provenance_contract(contract: &ProvenanceContract, tag: &ProvenanceTag) -> Result<(), String>` を追加する
4. Rust テスト 2 件を追加し 3730 tests に到達する

---

## 型・関数仕様

### `PiiPolicy` enum

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PiiPolicy {
    MustBeMasked,   // PII データが入力に含まれる可能性があるが、パイプライン到達前にマスクされていること（tag.pii=false が必要）
    AllowRaw,       // PII の有無を問わない（制限なし）
    MustBeAbsent,   // PII がそもそも存在しないこと（tag.pii=false が必要）
}
```

---

### `ProvenanceContract` 構造体

```rust
#[derive(Debug, Clone)]
pub struct ProvenanceContract {
    pub allowed_sources: Vec<DataSourceType>,
    pub pii_policy:      PiiPolicy,
}
```

---

### `validate_provenance_contract`

```rust
pub fn validate_provenance_contract(
    contract: &ProvenanceContract,
    tag: &ProvenanceTag,
) -> Result<(), String>
```

**検証ルール（この順番で適用）:**

1. **ソース種別チェック**: `contract.allowed_sources` が非空の場合、`tag.source.source_type` が `allowed_sources` に含まれていること
   - 違反時: `Err("source type not allowed: <source_type:?> is not in allowed_sources".to_string())`
   - `allowed_sources` が空の場合はスキップ

2. **PII ポリシーチェック**:
   - `PiiPolicy::MustBeMasked`: `tag.pii` が `false` であること → 違反時: `Err("pii policy violated: MustBeMasked requires pii=false".to_string())`
   - `PiiPolicy::AllowRaw`: 制限なし（常に Ok）
   - `PiiPolicy::MustBeAbsent`: `tag.pii` が `false` であること → 違反時: `Err("pii policy violated: MustBeAbsent requires pii=false".to_string())`

3. すべて満たせば `Ok(())`

---

## テスト仕様

### `provenance_contract_source_violation`

```rust
let src = DataSource {
    name:        "api-data".to_string(),
    uri:         "https://api.example.com/data".to_string(),
    source_type: DataSourceType::Api,
};
let tag = ProvenanceTag {
    source:     src,
    transforms: vec![],
    pii:        false,
};
let contract = ProvenanceContract {
    allowed_sources: vec![DataSourceType::Snowflake, DataSourceType::S3],
    pii_policy:      PiiPolicy::AllowRaw,
};
let result = validate_provenance_contract(&contract, &tag);
assert!(result.is_err());
let msg = result.unwrap_err();
assert!(msg.contains("source"), "error should mention source: {}", msg);

// allowed_sources が空の場合はソースチェックをスキップ → Ok
let open_contract = ProvenanceContract {
    allowed_sources: vec![],
    pii_policy:      PiiPolicy::AllowRaw,
};
assert!(validate_provenance_contract(&open_contract, &tag).is_ok());
```

### `provenance_contract_pii_violation`

```rust
let src = DataSource {
    name:        "crm".to_string(),
    uri:         "snowflake://crm/users".to_string(),
    source_type: DataSourceType::Snowflake,
};
let pii_tag = ProvenanceTag {
    source:     src,
    transforms: vec![],
    pii:        true,
};
// MustBeMasked + pii=true → Err
let contract = ProvenanceContract {
    allowed_sources: vec![DataSourceType::Snowflake],
    pii_policy:      PiiPolicy::MustBeMasked,
};
let result = validate_provenance_contract(&contract, &pii_tag);
assert!(result.is_err());
assert!(result.unwrap_err().contains("pii"));

// MustBeAbsent + pii=true → Err
let contract2 = ProvenanceContract {
    allowed_sources: vec![],
    pii_policy:      PiiPolicy::MustBeAbsent,
};
assert!(validate_provenance_contract(&contract2, &pii_tag).is_err());

// AllowRaw + pii=true → Ok
let contract3 = ProvenanceContract {
    allowed_sources: vec![],
    pii_policy:      PiiPolicy::AllowRaw,
};
assert!(validate_provenance_contract(&contract3, &pii_tag).is_ok());
```

---

## Success Criteria

- `PiiPolicy` enum が定義されている（MustBeMasked / AllowRaw / MustBeAbsent）
- `ProvenanceContract` 構造体が定義されている
- `validate_provenance_contract` がソース種別・PII ポリシーを正しく検証する
- `allowed_sources` が空の場合はソースチェックをスキップする
- `provenance_contract_source_violation` が pass
- `provenance_contract_pii_violation` が pass
- `cargo test` が 3730 tests all pass
- `driver.rs` 内の `76.7.0` バージョン文字列アサーションがすべて `76.8.0` に更新されている
- `CHANGELOG.md` の先頭に v76.8.0 エントリが存在する

---

## 変更ファイル

- `fav/src/driver.rs` — `PiiPolicy`, `ProvenanceContract`, `validate_provenance_contract`, `v768000_tests` を追加
- `CHANGELOG.md` — v76.8.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `76.7.0` → `76.8.0` に更新

---

## 依存（既実装）

- `DataSource` 構造体（v76.1.0）
- `DataSourceType` enum（v76.1.0）— バリアント: Snowflake / S3 / Api / Manual / Pipeline
- `ProvenanceTag` 構造体（v76.1.0）

---

## 対象外

- `contract` 構文への Favnir 言語統合（将来バージョン）
- コンパイル時のソース種別チェック（Rust テストのみ、言語レベルの型チェックは将来）
- `PiiPolicy::MustBeMasked` と `MustBeAbsent` の意味論的な差異の実行時検証（現実装では同一の `pii=false` チェック。差異は v77.x 以降で実装予定）
