# v75.7.0 仕様書 — Temporal contracts

Date: 2026-08-15
Status: 計画中

---

## Background

v75.1.0〜v75.6.0 で実装した `FreshnessPolicy`（鮮度）と `RetentionPolicy`（保持）を組み合わせ、**コントラクト**として単一の型にまとめる。パイプラインの「時間要件」を一箇所で宣言でき、検証・レポートが統一 API で行えるようになる。

---

## Goals

1. `TemporalContract` 構造体（name: String, freshness: Option<FreshnessPolicy>, retention: Option<RetentionPolicy>）を追加する
2. `validate_temporal_contract(contract: &TemporalContract, data_ts: i64, now: i64) -> Result<(), String>` を追加する
3. `format_temporal_contract_report(contract: &TemporalContract, result: &Result<(), String>) -> String` を追加する
4. Rust テスト 2 件を追加し 3706 tests に到達する

---

## 型・関数仕様

### `TemporalContract` 構造体

```rust
#[derive(Debug, Clone)]
pub struct TemporalContract {
    pub name:      String,
    pub freshness: Option<FreshnessPolicy>,
    pub retention: Option<RetentionPolicy>,
}
```

- `freshness` が `None` の場合は鮮度チェックをスキップ
- `retention` が `None` の場合は保持チェックをスキップ
- 両方 `None` の場合は常に `Ok(())`

---

### `validate_temporal_contract`

```rust
pub fn validate_temporal_contract(
    contract: &TemporalContract,
    data_ts:  i64,
    now:      i64,
) -> Result<(), String>
```

**検証ロジック（優先度順）:**

1. **鮮度チェック**（`freshness: Some(policy)` のとき）:
   - `age = (now - data_ts).max(0) as u64`（未来タイムスタンプは age=0）
   - `age > policy.max_age_secs` のとき `Err("freshness violation: age={age}s exceeds max_age_secs={max}")`
2. **保持チェック**（`retention: Some(policy)` のとき）:
   - `age > policy.max_age_days * 86400` のとき `Err("retention exceeded: age={age}s exceeds max_age_days={days}")`
3. 両方通過 → `Ok(())`

**境界値:** `age == max` のとき Ok（開区間。v75.4.0〜v75.6.0 と同方針）。

---

### `format_temporal_contract_report`

```rust
pub fn format_temporal_contract_report(
    contract: &TemporalContract,
    result:   &Result<(), String>,
) -> String
```

**出力フォーマット:**
- Ok 時:  `"[OK] contract={name}"`
- Err 時: `"[VIOLATION] contract={name} reason={msg}"`

---

## Favnir コード例

```favnir
bind contract <- TemporalContract {
    name: "PricingPipeline",
    freshness: FreshnessPolicy { max_age_secs: 300, strategy: Fail },
    retention: RetentionPolicy { max_age_days: 90, action: Delete }
}
bind result <- validate_temporal_contract(contract, data_ts, now_secs)
ctx.io.println(format_temporal_contract_report(contract, result))
```

---

## Success Criteria

- `TemporalContract` 構造体が定義されている
- `validate_temporal_contract` が鮮度・保持それぞれ正しく Err を返す
- `format_temporal_contract_report` が `[OK]`/`[VIOLATION]` フォーマットを生成する
- `cargo test` が 3706 tests all pass
- `CHANGELOG.md` の先頭に v75.7.0 エントリが存在する

---

## テスト仕様

### `temporal_contract_freshness_violation`

- `contract = TemporalContract { name: "PricingPipeline", freshness: Some(FreshnessPolicy { max_age_secs: 300, strategy: FreshnessStrategy::Fail }), retention: None }`
- `data_ts=0, now=400`（age=400 > 300）→ `Err`
- `data_ts=0, now=300`（age=300 = boundary）→ `Ok(())`（開区間）
- `format_temporal_contract_report` の Err ケースが `"[VIOLATION]"` と `"PricingPipeline"` を含む

### `temporal_contract_retention_exceeded`

- `contract = TemporalContract { name: "UserDataPipeline", freshness: None, retention: Some(RetentionPolicy { max_age_days: 7, action: RetentionAction::Delete }) }`
- `data_ts=0, now=8 * 86400`（8日後）→ `Err`
- `data_ts=0, now=7 * 86400`（7日 = boundary）→ `Ok(())`（開区間）
- `format_temporal_contract_report` の Ok ケースが `"[OK]"` を含む

---

## 変更ファイル

- `fav/src/driver.rs` — `TemporalContract`, `validate_temporal_contract`, `format_temporal_contract_report`, `v757000_tests` を追加
- `CHANGELOG.md` — v75.7.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `75.6.0` → `75.7.0` に更新

---

## 依存型（既実装）

- `FreshnessPolicy { max_age_secs: u64, strategy: FreshnessStrategy }` — v75.1.0 で実装済み
- `FreshnessStrategy` enum（Warn, Fail）— v75.1.0 で実装済み
- `RetentionPolicy { max_age_days: u64, action: RetentionAction }` — v75.5.0 で実装済み
- `RetentionAction` enum（Delete, Archive, Anonymize）— v75.5.0 で実装済み

`max_age_days * 86400` のオーバーフロー対策として `saturating_mul` を使用する（u64::MAX / 86400 超は実用上到達しないが安全性のため）。

---

## 対象外

- Favnir 言語レベルでの `contract` キーワード拡張（将来バージョン予定）
- `FreshnessStrategy::Warn` の警告ログ出力（`validate_temporal_contract` は `Warn` 戦略のときも `Ok(())` を返す。ログ出力は呼び出し側の責任）
- 複数の鮮度・保持ポリシーの組み合わせ（各フィールドは単一ポリシーのみ）
