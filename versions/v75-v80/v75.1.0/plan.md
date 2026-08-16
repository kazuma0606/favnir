# v75.1.0 実装計画 — `FreshnessPolicy` 型基盤

Date: 2026-08-14

---

## 事前確認（T0）

- [ ] `fav/Cargo.toml` のバージョンが `75.0.0` であることを確認
- [ ] `cargo test` が全 pass（3692 tests）であることを確認
- [ ] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## ステップ 1: driver.rs に型定義・関数を追加

**ファイル**: `fav/src/driver.rs`

以下を追加する（テストモジュールの外に配置）:

```rust
// --- v75.1.0: FreshnessPolicy 型基盤 ---

#[derive(Debug, Clone, PartialEq)]
pub enum FreshnessStrategy {
    Warn,
    Fail,
}

#[derive(Debug, Clone)]
pub struct FreshnessPolicy {
    pub max_age_secs: u64,
    pub strategy: FreshnessStrategy,
}

pub fn check_freshness(data_ts: i64, now: i64, policy: &FreshnessPolicy) -> bool {
    let age = (now - data_ts).max(0) as u64;
    age <= policy.max_age_secs
}

pub fn format_freshness_warning(policy: &FreshnessPolicy, age_secs: u64) -> String {
    let strategy = match policy.strategy {
        FreshnessStrategy::Warn => "Warn",
        FreshnessStrategy::Fail => "Fail",
    };
    format!(
        "[FreshnessPolicy] STALE: data age={}s exceeds max_age={}s (strategy={})",
        age_secs, policy.max_age_secs, strategy
    )
}
```

---

## ステップ 2: CHANGELOG.md にエントリ追加（テスト追加より先）

**ファイル**: `CHANGELOG.md`

最上部に v75.1.0 エントリを追加する:

```markdown
## [v75.1.0] — 2026-08-14 — `FreshnessPolicy` 型基盤

### Added
- `FreshnessStrategy` enum（Warn / Fail）
- `FreshnessPolicy` 構造体（max_age_secs: u64, strategy: FreshnessStrategy）
- `check_freshness(data_ts: i64, now: i64, policy: &FreshnessPolicy) -> bool`
- `format_freshness_warning(policy: &FreshnessPolicy, age_secs: u64) -> String`

### Tests
- `v751000_tests` 2 件追加（合計テスト数: 3694, +2）
  - `freshness_policy_enforced` — TTL 内のデータが鮮度 OK と判定される
  - `freshness_stale_detected` — TTL 超過のデータが鮮度違反と判定される
```

---

## ステップ 3: driver.rs にテストモジュールを追加

**ファイル**: `fav/src/driver.rs`

`use super::*;` は `FreshnessStrategy` / `FreshnessPolicy` / `check_freshness` / `format_freshness_warning` を参照するために必須。

```rust
#[cfg(test)]
mod v751000_tests {
    use super::*;

    #[test]
    fn freshness_policy_enforced() {
        let policy = FreshnessPolicy {
            max_age_secs: 300,
            strategy: FreshnessStrategy::Fail,
        };
        let now = 1_700_000_000_i64;
        let data_ts = now - 299; // 299秒前 → TTL内
        assert!(
            check_freshness(data_ts, now, &policy),
            "data within TTL should be fresh"
        );
    }

    #[test]
    fn freshness_stale_detected() {
        let policy = FreshnessPolicy {
            max_age_secs: 300,
            strategy: FreshnessStrategy::Warn,
        };
        let now = 1_700_000_000_i64;
        let data_ts = now - 301; // 301秒前 → TTL超過
        assert!(
            !check_freshness(data_ts, now, &policy),
            "data exceeding TTL should be stale"
        );
        let warning = format_freshness_warning(&policy, 301);
        assert!(warning.contains("STALE"), "warning should contain STALE");
        assert!(warning.contains("max_age=300s"), "warning should contain max_age");
    }
}
```

---

## ステップ 4: `fav/Cargo.toml` バージョン更新

`"75.0.0"` → `"75.1.0"`

---

## ステップ 5: `versions/current.md` 更新

- 「進行中バージョン」を v75.1.0 に更新
- 「次に切る版」を v75.2.0 に更新

---

## ステップ 6: 動作確認

```bash
cd fav && cargo test -- --test-threads=8 2>&1 | tail -5
# 期待: test result: ok. 3694 passed

cargo test v751000 -- --nocapture
# freshness_policy_enforced / freshness_stale_detected の 2 件が pass
```

---

## 実装順序まとめ

```
T0: 事前確認（バージョン・テスト数・ファイル存在確認）
1: driver.rs — 型定義・関数追加（FreshnessStrategy / FreshnessPolicy / check_freshness / format_freshness_warning）
2: CHANGELOG.md — v75.1.0 エントリ追加（テスト追加より先）
3: driver.rs — v751000_tests モジュール追加
4: Cargo.toml — バージョン更新
5: versions/current.md — 更新
6: cargo test 全 pass 確認（3694 tests）
```
