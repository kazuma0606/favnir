# v82.2.0 実装計画

## 方針

**前提**: v82.1.0 完了済み（3,867 tests pass）。

`test_framework.rs` に SLA 関連の型・関数を追加し、`driver.rs` に `v82200_tests` を追加する。

---

## 実装ステップ

### Step 1: `SlaTarget` 構造体を追加

`fav/src/test_framework.rs` の `v82.1.0` セクション末尾に続けて追加する。

```rust
/// SLA 目標値。
#[derive(Debug, Clone, PartialEq)]
pub struct SlaTarget {
    pub max_latency_ms: u64,
    pub min_throughput_rps: f64,
    pub min_availability_pct: f64,
}
```

### Step 2: `SlaContract` 構造体を追加

```rust
/// SLA を型として宣言するパイプライン契約。
/// `adaptive_strategy` は Favnir 3.0 の `!Adaptive` エフェクトと連携する文字列スタブ。
/// `cache_ttl_secs` は `!Cached` エフェクトと連携するキャッシュ TTL スタブ。
#[derive(Debug, Clone)]
pub struct SlaContract {
    pub name: String,
    pub target: SlaTarget,
    pub adaptive_strategy: Option<String>,
    pub cache_ttl_secs: Option<u64>,
}
```

### Step 3: `SlaStatus` enum を追加

```rust
/// SLA 評価結果。
/// - `Met`: すべての目標を満たしている
/// - `AtRisk(String)`: 目標に近いが超過はしていない（将来拡張用、本バージョンでは未使用）
/// - `Breached(String)`: 目標を超過している（メッセージに詳細を含む）
#[derive(Debug, PartialEq)]
pub enum SlaStatus {
    Met,
    AtRisk(String),
    Breached(String),
}
```

### Step 4: `evaluate_sla` 関数を実装

```rust
/// 実測値と SLA 目標を比較し、`SlaStatus` を返す。
///
/// 判定順序:
/// 1. レイテンシ超過 → `Breached`
/// 2. スループット不足 → `Breached`
/// 3. いずれも違反なし → `Met`
pub fn evaluate_sla(
    contract: &SlaContract,
    actual_latency_ms: u64,
    actual_rps: f64,
) -> SlaStatus {
    if actual_latency_ms > contract.target.max_latency_ms {
        return SlaStatus::Breached(format!(
            "latency exceeded: {} ms > {} ms",
            actual_latency_ms, contract.target.max_latency_ms
        ));
    }
    if actual_rps < contract.target.min_throughput_rps {
        return SlaStatus::Breached(format!(
            "throughput below minimum: {} rps < {} rps",
            actual_rps, contract.target.min_throughput_rps
        ));
    }
    SlaStatus::Met
}
```

### Step 5: `format_sla_status` 関数を実装

```rust
/// `SlaStatus` を人間が読める文字列に変換する。
pub fn format_sla_status(status: &SlaStatus) -> String {
    match status {
        SlaStatus::Met => "SLA: Met".into(),
        SlaStatus::AtRisk(msg) => format!("SLA: AtRisk — {msg}"),
        SlaStatus::Breached(msg) => format!("SLA: Breached — {msg}"),
    }
}
```

### Step 6: CHANGELOG 更新

`CHANGELOG.md` の先頭に v82.2.0 エントリを追加する。

### Step 7: `v82200_tests` テストモジュール追加（driver.rs）

`fav/src/driver.rs` 末尾に `#[cfg(test)] mod v82200_tests` を追加する。

- `sla_contract_met_within_target`: レイテンシ・スループット共に目標内 → `Met`、`format_sla_status` が `"SLA: Met"` を返す
- `sla_contract_breached_over_latency`: レイテンシが max_latency_ms を超過 → `Breached`、メッセージに数値を含む

### Step 8: `cargo test` 全通過確認

3,869 tests pass（+2）、0 failures であることを確認する。
