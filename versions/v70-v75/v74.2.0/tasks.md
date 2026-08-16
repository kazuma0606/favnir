# v74.2.0 タスクリスト — Multi-tenant Runtime

Date: 2026-08-13
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `74.1.0` であることを確認
- [x] `cargo test` が 3671 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v741000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v742000_tests` が未存在であることを確認

---

## T1: 構造体 + 関数を `driver.rs` に追加

- [x] `// --- v74.2.0: Multi-tenant Runtime ---` セクションコメントを追加した
- [x] `#[derive(Debug, Clone, PartialEq)] pub struct TenantQuota` を追加した（max_memory_mb / max_cpu_pct / max_rows）
- [x] `#[derive(Debug, Clone, PartialEq)] pub struct TenantTeamConfig` を追加した（db_url / s3_bucket）
- [x] `#[derive(Debug, Clone, PartialEq)] pub struct TenantConfig` を追加した（isolation / quota / teams）
- [x] `pub fn check_tenant_quota_exceeded(quota: &TenantQuota, rows: u64, memory_mb: u64) -> bool` を実装した
  - rows > quota.max_rows || memory_mb > quota.max_memory_mb で判定
  - 境界値（ちょうど最大）は false を返す
- [x] `pub fn format_tenant_isolation_report(config: &TenantConfig) -> String` を実装した
  - `"isolation=strict quota(mem=512MB cpu=80% rows=1000000)"` 形式
- [x] `cargo build` でエラーがないことを確認

---

## T2: `v742000_tests` モジュールを追加

- [x] `v741000_tests` の直後に `v742000_tests` モジュールを追加した
- [x] `use super::{TenantConfig, TenantQuota, TenantTeamConfig, check_tenant_quota_exceeded, format_tenant_isolation_report}` を追加した
- [x] `multitenant_config_parsed` テストを実装した
  - `TenantConfig` を構築し isolation / quota フィールドを assert
  - `teams` に team_a / team_b が存在することを assert
  - `format_tenant_isolation_report` の出力に isolation / mem / cpu / rows が含まれることを assert
- [x] `multitenant_resource_quota_enforced` テストを実装した
  - クォータ以内 → `false`
  - rows 超過 → `true`
  - memory_mb 超過 → `true`
  - 境界値（ちょうど最大）→ `false`

---

## T3: バージョン更新

- [x] `fav/Cargo.toml` の `version = "74.1.0"` → `version = "74.2.0"` に変更した
- [x] `driver.rs` 内の `version = "74.1.0"` 参照を `version = "74.2.0"` に replace_all した
- [x] `version should be 74.1.0` を `version should be 74.2.0` に replace_all した（アサートメッセージのみ対象）
- [x] 残存 `74.1.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `cargo build` でエラーがないことを確認
- [x] `fav/Cargo.lock` が `version = "74.2.0"` を含むことを確認

---

## T3.5: バージョン更新後の部分テスト再確認

- [x] `cargo test v742000` で 2 件 pass することを確認

---

## T4: 全体テスト確認

- [x] `cargo test` 全体で 3673 tests pass（0 failures）であることを確認

---

## T5: `CHANGELOG.md` 更新

- [x] `## [v74.2.0]` エントリを先頭に追加した
  - Added: `TenantQuota` / `TenantTeamConfig` / `TenantConfig` / `check_tenant_quota_exceeded` / `format_tenant_isolation_report`
  - Tests: 2 件、合計テスト数 3673（+2）

---

## T6: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-13 (v74.2.0)` に更新した
- [x] 「進行中バージョン」を `v74.2.0` に更新した
- [x] 「次に切る版」を `v74.3.0` に更新した

---

## T7: 最終確認（T5・T6 完了後）

- [x] `cargo test v742000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3673 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `74.2.0` であることを確認
- [x] `CHANGELOG.md` に `[v74.2.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v74.2.0` であることを確認

---

## スコープ外（明示的除外）

- VM への実際のクォータ強制適用（後続バージョンで対応）
- `AppCtx` への `ctx.tenant.*` 注入の完全実装（後続バージョンで対応）
- `fav.toml` TOML パース（文字列からの `TenantConfig` 読み込み、後続バージョンで対応）
- `MILESTONE.md` 更新・`site/` MDX 追加（v74.3.0 Documentation Site 2.0 以降で対応）
