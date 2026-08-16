# v75.7.0 実装計画 — Temporal contracts

Date: 2026-08-15
Status: 計画中

---

## 実装ステップ（依存順）

### Step 1: driver.rs — `TemporalContract` 構造体追加

`fav/src/driver.rs` の末尾（v75.6.0 ブロックの後）に追加する。

```rust
// --- v75.7.0: Temporal contracts ---

/// 鮮度・保持ポリシーを組み合わせたコントラクト。
/// `freshness` / `retention` が `None` のフィールドはそのチェックをスキップする。
#[derive(Debug, Clone)]
pub struct TemporalContract {
    pub name:      String,
    pub freshness: Option<FreshnessPolicy>,
    pub retention: Option<RetentionPolicy>,
}
```

### Step 2: driver.rs — `validate_temporal_contract` 関数追加

```rust
/// コントラクトの鮮度・保持ポリシーを検証する。
///
/// # 検証順序
/// 1. 鮮度チェック（freshness が Some のとき）: age > max_age_secs → Err
/// 2. 保持チェック（retention が Some のとき）: age > max_age_days * 86400 → Err
/// 3. 両方通過 → Ok(())
///
/// # 境界値
/// age == max のとき Ok（開区間）。v75.4.0〜v75.6.0 と同方針。
/// 未来タイムスタンプ（data_ts > now）は age=0 として常に Ok。
pub fn validate_temporal_contract(
    contract: &TemporalContract,
    data_ts:  i64,
    now:      i64,
) -> Result<(), String> {
    let age = now.saturating_sub(data_ts).max(0) as u64;
    if let Some(fp) = &contract.freshness {
        if age > fp.max_age_secs {
            return Err(format!(
                "freshness violation: age={age}s exceeds max_age_secs={}",
                fp.max_age_secs
            ));
        }
    }
    if let Some(rp) = &contract.retention {
        let max_secs = rp.max_age_days.saturating_mul(86_400);
        if age > max_secs {
            return Err(format!(
                "retention exceeded: age={age}s exceeds max_age_days={}",
                rp.max_age_days
            ));
        }
    }
    Ok(())
}
```

### Step 3: driver.rs — `format_temporal_contract_report` 関数追加

```rust
/// コントラクト検証結果を人間が読める文字列で返す。
///
/// フォーマット:
/// - Ok 時:  `"[OK] contract={name}"`
/// - Err 時: `"[VIOLATION] contract={name} reason={msg}"`
pub fn format_temporal_contract_report(
    contract: &TemporalContract,
    result:   &Result<(), String>,
) -> String {
    match result {
        Ok(()) => format!("[OK] contract={}", contract.name),
        Err(msg) => format!("[VIOLATION] contract={} reason={}", contract.name, msg),
    }
}
```

### Step 3.5: cargo check 確認

`cargo check` でコンパイルエラーがないことを確認する。

### Step 4: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v75.7.0 エントリを追加する。

### Step 5: driver.rs — テストモジュール追加

`TemporalContract` / `FreshnessPolicy` / `RetentionPolicy` 等の型を参照するため `use super::*` が必要。

```rust
#[cfg(test)]
mod v757000_tests {
    use super::*;

    #[test]
    fn temporal_contract_freshness_violation() {
        let contract = TemporalContract {
            name: "PricingPipeline".to_string(),
            freshness: Some(FreshnessPolicy { max_age_secs: 300, strategy: FreshnessStrategy::Fail }),
            retention: None,
        };
        // age=400 > 300 → Err
        let result = validate_temporal_contract(&contract, 0, 400);
        assert!(result.is_err(), "stale data must fail freshness check");
        let report = format_temporal_contract_report(&contract, &result);
        assert!(report.contains("[VIOLATION]"), "report must contain [VIOLATION]");
        assert!(report.contains("PricingPipeline"), "report must contain contract name");
        // age=300 = boundary → Ok（開区間）
        let result2 = validate_temporal_contract(&contract, 0, 300);
        assert!(result2.is_ok(), "data at boundary must pass");
        let report2 = format_temporal_contract_report(&contract, &result2);
        assert!(report2.contains("[OK]"), "ok report must contain [OK]");
    }

    #[test]
    fn temporal_contract_retention_exceeded() {
        let contract = TemporalContract {
            name: "UserDataPipeline".to_string(),
            freshness: None,
            retention: Some(RetentionPolicy { max_age_days: 7, action: RetentionAction::Delete }),
        };
        // 8日後 → Err
        let result = validate_temporal_contract(&contract, 0, 8 * 86_400);
        assert!(result.is_err(), "data older than 7 days must fail retention check");
        let report = format_temporal_contract_report(&contract, &result);
        assert!(report.contains("[VIOLATION]"), "report must contain [VIOLATION]");
        assert!(report.contains("UserDataPipeline"), "report must contain contract name");
        // 7日 = boundary → Ok（開区間）
        let result2 = validate_temporal_contract(&contract, 0, 7 * 86_400);
        assert!(result2.is_ok(), "data at 7-day boundary must pass");
        let report2 = format_temporal_contract_report(&contract, &result2);
        assert!(report2.contains("[OK]"), "ok report must contain [OK]");
    }
}
```

### Step 6: Cargo.toml・driver.rs バージョン更新

- `Cargo.toml`: `"75.6.0"` → `"75.7.0"`
- `driver.rs` 内の `version = \"75.6.0\"` を `replace_all` で `version = \"75.7.0\"` に更新

### Step 7: versions/current.md 更新

- 「進行中バージョン」を v75.7.0 に更新
- 「次に切る版」を v75.8.0 に更新

### Step 8: 最終確認

- `cargo test` 全件 pass（3706 tests）
- `cargo test v757000` 2 件 pass

---

## 依存関係

```
Step 1 (TemporalContract — FreshnessPolicy・RetentionPolicy を参照)
  └→ Step 2 (validate_temporal_contract)
  └→ Step 3 (format_temporal_contract_report)
Step 2, 3 (関数)
  └→ Step 5 (テスト)
Step 4 (CHANGELOG) — Step 5 より先に実施
Step 6 (バージョン更新) — Step 5 完了後
Step 7 (current.md) — Step 6 完了後
Step 8 (最終確認) — Step 6, 7 完了後
```

## 注意事項

- `validate_temporal_contract` は鮮度チェックを保持チェックより先に行う（先に Err を返す）
- `FreshnessStrategy::Warn` のとき警告ログは出さない（validate は Err のみ返す）
- `rp.max_age_days * 86_400` のオーバーフロー: u64 同士の乗算は u64 でサチュレートしないため、
  実用的な日数（数千日以内）では問題なし。ただし `u64::MAX / 86400` 超は未定義動作 → doc コメントに明記
