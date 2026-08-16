# v74.2.0 仕様書 — Multi-tenant Runtime

Date: 2026-08-13

---

## Background

複数チームが同一 Favnir インスタンスを共有する場合、ステージ間のリソース分離・チームごとの
DB/S3 設定注入が必要になる。本バージョンでは `fav.toml` の `[tenant]` / `[tenant.<name>]`
セクションに対応するデータ構造と、クォータ超過チェック関数を `driver.rs` に追加する。

本バージョンは基盤データ構造と関数のみを実装する。ロードマップ v74.2.0 の実装内容のうち
`[tenant]` TOML パース（toml.rs 拡張）・VM クォータ強制・`AppCtx` 注入は
後続バージョン（v74.X、未定）に延期する。

---

## Goals

1. `TenantQuota` 構造体（max_memory_mb / max_cpu_pct / max_rows）を定義する
2. `TenantTeamConfig` 構造体（db_url / s3_bucket）を定義する
3. `TenantConfig` 構造体（isolation / quota / teams）を定義する
4. `check_tenant_quota_exceeded(quota, rows, memory_mb) -> bool` を実装する
5. `format_tenant_isolation_report(config) -> String` を実装する
6. `v742000_tests` モジュール（2 件）を追加する

---

## API / 設定例

```toml
# fav.toml
[tenant]
isolation = "strict"
quota.max_memory_mb = 512
quota.max_cpu_pct   = 80
quota.max_rows      = 1_000_000

[tenant.team_a]
db_url    = "${TEAM_A_DB_URL}"
s3_bucket = "team-a-data"

[tenant.team_b]
db_url    = "${TEAM_B_DB_URL}"
s3_bucket = "team-b-data"
```

### `TenantQuota` 構造体

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TenantQuota {
    pub max_memory_mb: u64,  // デフォルト 512
    pub max_cpu_pct: u8,     // デフォルト 80（%）
    pub max_rows: u64,       // デフォルト 1_000_000
}
```

### `TenantTeamConfig` 構造体

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TenantTeamConfig {
    pub db_url: String,
    pub s3_bucket: String,
}
```

### `TenantConfig` 構造体

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TenantConfig {
    pub isolation: String,                           // "strict" | "relaxed"
    pub quota: TenantQuota,
    pub teams: std::collections::HashMap<String, TenantTeamConfig>,
}
```

### `check_tenant_quota_exceeded`

```rust
// 実際の rows または memory_mb がクォータを超えたら true
pub fn check_tenant_quota_exceeded(quota: &TenantQuota, rows: u64, memory_mb: u64) -> bool
```

注: `max_cpu_pct` フィールドは将来の VM クォータ強制用に予約されており、
本バージョンでは関数内で参照しない（CPU 使用率の計測は VM 統合後に実装）。

### `format_tenant_isolation_report`

```rust
// "isolation=strict quota(mem=512MB cpu=80% rows=1000000)" 形式で返す
pub fn format_tenant_isolation_report(config: &TenantConfig) -> String
```

---

## Success Criteria

1. `multitenant_config_parsed` テストが pass する
   - `TenantConfig` を構築し、isolation / quota フィールドが正しいことを assert
   - `teams` に team_a / team_b が存在することを assert
2. `multitenant_resource_quota_enforced` テストが pass する
   - rows / memory_mb がクォータ以下 → `false`
   - rows がクォータ超過 → `true`
   - memory_mb がクォータ超過 → `true`
3. `cargo test` で 3673 tests pass（0 failures）

---

## Error Codes

新規エラーコードなし

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `TenantQuota` / `TenantTeamConfig` / `TenantConfig` / `check_tenant_quota_exceeded` / `format_tenant_isolation_report` + `v742000_tests` 追加 |
| `fav/Cargo.toml` | `version = "74.2.0"` に更新 |
| `CHANGELOG.md` | v74.2.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョン・次に切る版を更新 |
