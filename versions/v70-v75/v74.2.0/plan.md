# v74.2.0 実装計画 — Multi-tenant Runtime

Date: 2026-08-13

---

## 実装ステップ

### Step 1: 構造体 + 関数を `driver.rs` に追加

```rust
// --- v74.2.0: Multi-tenant Runtime ---

#[derive(Debug, Clone, PartialEq)]
pub struct TenantQuota {
    pub max_memory_mb: u64,
    pub max_cpu_pct: u8,
    pub max_rows: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantTeamConfig {
    pub db_url: String,
    pub s3_bucket: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantConfig {
    pub isolation: String,
    pub quota: TenantQuota,
    pub teams: std::collections::HashMap<String, TenantTeamConfig>,
}

/// rows または memory_mb がクォータを超えた場合に true を返す
pub fn check_tenant_quota_exceeded(quota: &TenantQuota, rows: u64, memory_mb: u64) -> bool {
    rows > quota.max_rows || memory_mb > quota.max_memory_mb
}

/// テナント設定のサマリーを返す
/// 例: "isolation=strict quota(mem=512MB cpu=80% rows=1000000)"
pub fn format_tenant_isolation_report(config: &TenantConfig) -> String {
    format!(
        "isolation={} quota(mem={}MB cpu={}% rows={})",
        config.isolation,
        config.quota.max_memory_mb,
        config.quota.max_cpu_pct,
        config.quota.max_rows,
    )
}
```

### Step 2: `v742000_tests` モジュールを追加

```rust
#[cfg(test)]
mod v742000_tests {
    use super::{
        TenantConfig, TenantQuota, TenantTeamConfig,
        check_tenant_quota_exceeded, format_tenant_isolation_report,
    };
    use std::collections::HashMap;

    #[test]
    fn multitenant_config_parsed() {
        let mut teams = HashMap::new();
        teams.insert("team_a".to_string(), TenantTeamConfig {
            db_url: "${TEAM_A_DB_URL}".to_string(),
            s3_bucket: "team-a-data".to_string(),
        });
        teams.insert("team_b".to_string(), TenantTeamConfig {
            db_url: "${TEAM_B_DB_URL}".to_string(),
            s3_bucket: "team-b-data".to_string(),
        });
        let config = TenantConfig {
            isolation: "strict".to_string(),
            quota: TenantQuota {
                max_memory_mb: 512,
                max_cpu_pct: 80,
                max_rows: 1_000_000,
            },
            teams,
        };
        assert_eq!(config.isolation, "strict");
        assert_eq!(config.quota.max_memory_mb, 512);
        assert_eq!(config.quota.max_cpu_pct, 80);
        assert_eq!(config.quota.max_rows, 1_000_000);
        assert!(config.teams.contains_key("team_a"), "team_a missing");
        assert!(config.teams.contains_key("team_b"), "team_b missing");
        assert_eq!(config.teams["team_a"].s3_bucket, "team-a-data");

        let report = format_tenant_isolation_report(&config);
        assert!(report.contains("isolation=strict"), "isolation missing");
        assert!(report.contains("512MB"), "memory quota missing");
        assert!(report.contains("80%"), "cpu quota missing");
        assert!(report.contains("1000000"), "rows quota missing");
    }

    #[test]
    fn multitenant_resource_quota_enforced() {
        let quota = TenantQuota {
            max_memory_mb: 512,
            max_cpu_pct: 80,
            max_rows: 1_000_000,
        };

        // クォータ以内 → false
        assert!(!check_tenant_quota_exceeded(&quota, 500_000, 256));

        // rows 超過 → true
        assert!(check_tenant_quota_exceeded(&quota, 1_000_001, 256));

        // memory_mb 超過 → true
        assert!(check_tenant_quota_exceeded(&quota, 500_000, 513));

        // 両方超過 → true
        assert!(check_tenant_quota_exceeded(&quota, 2_000_000, 1024));

        // 境界値（ちょうど最大）→ false
        assert!(!check_tenant_quota_exceeded(&quota, 1_000_000, 512));
    }
}
```

### Step 3: バージョン更新

- `fav/Cargo.toml`: `version = "74.1.0"` → `version = "74.2.0"`
- `driver.rs` 内の `version = "74.1.0"` 参照を `version = "74.2.0"` に replace_all
- `version should be 74.1.0` を `version should be 74.2.0` に replace_all
- `cargo build` を実行すると `Cargo.lock` が自動的に `version = "74.2.0"` に更新される

### Step 4: テスト確認

- `cargo test v742000` で 2 件 pass を確認
- `cargo test` 全体で 3673 tests pass を確認

### Step 5: `CHANGELOG.md` 更新

v74.2.0 エントリを先頭に追加。

### Step 6: `versions/current.md` 更新

- 最終更新: `2026-08-13 (v74.2.0)`
- 進行中: `v74.2.0`
- 次: `v74.3.0`
