# v79.2.0 仕様書 — Temporal showcase パイプライン

Date: 2026-08-16
Status: PLANNING

---

## Background

v79.1.0 で `infra/e2e-demo/favnir3-showcase/` の骨格を作成した。
v79.2.0 では `pipeline.fav` に Temporal 機能（v75.x スプリント実装済み）を追加する。

使用する既存 Rust 型・関数（`fav/src/driver.rs` v75.x ブロック）:
- `FreshnessPolicy` struct / `check_freshness` 関数
- `AsOfQuery` struct / `format_as_of_query` 関数
- `ScdRow` struct / `apply_scd2_update` 関数

> **Note**: ロードマップ記載のテスト数（3784）は stale 値。
> 実際のベースは 3789（v79.1.0 完了後の実測値）。完了後は 3791 が正しい。

---

## Goals

`infra/e2e-demo/favnir3-showcase/pipeline.fav` に Temporal ステージを追加し、
ショーケーステストで内容を検証する。

---

## `pipeline.fav` 更新内容

既存の `showcase_pipeline` スケルトンに以下を追加する:

```favnir
// Favnir 3.0 統合ショーケース — pipeline.fav
// v75.1〜v79.8 全機能統合パイプライン

// --- Stage 1: Temporal（v75.x）---
fn load_with_freshness(ctx: AppCtx) -> Result<List<Row>, String> {
    bind snapshot <- AsOfQuery { table: "orders", as_of_ts: ctx.run_ts }
    bind _        <- FreshnessPolicy.check(snapshot, max_age: Duration.hours(1))
    bind history  <- apply_scd2_update(existing_customers, new_data, ctx.run_ts)  // existing_customers / new_data は上位スコープから注入（後続スプリントで具体化）
    Result.ok(history)
}

fn showcase_pipeline(ctx: AppCtx) -> Result<String, String> {
    // Stage 1: Temporal（v75.x）— load_with_freshness で実装
    // Stage 2: Provenance（v76.x）— 後続スプリントで実装
    // Stage 3: Verifiable（v77.x）— 後続スプリントで実装
    // Stage 4: Execution Effects（v78.x）— 後続スプリントで実装
    Result.ok("favnir3-showcase: pipeline skeleton initialized")
}
```

---

## テストモジュール仕様

```rust
// --- v79.2.0: Temporal showcase パイプライン ---
#[cfg(test)]
mod v792000_tests {
    #[test]
    fn showcase_temporal_freshness_check() {
        // pipeline.fav に FreshnessPolicy / AsOfQuery の Temporal 関数が含まれることを確認
        let pipeline = include_str!("../../infra/e2e-demo/favnir3-showcase/pipeline.fav");
        assert!(pipeline.contains("load_with_freshness"), "pipeline.fav must define load_with_freshness");
        assert!(pipeline.contains("AsOfQuery"), "pipeline.fav must reference AsOfQuery");
        assert!(pipeline.contains("FreshnessPolicy"), "pipeline.fav must reference FreshnessPolicy");
    }

    #[test]
    fn showcase_temporal_scd2_applied() {
        // pipeline.fav に apply_scd2_update が含まれることを確認
        let pipeline = include_str!("../../infra/e2e-demo/favnir3-showcase/pipeline.fav");
        assert!(pipeline.contains("apply_scd2_update"), "pipeline.fav must reference apply_scd2_update");
        assert!(pipeline.contains("ctx.run_ts"), "pipeline.fav must reference ctx.run_ts for temporal context");
    }
}
```

注意: `use super::*` は不要（`include_str!` + `assert!` のみ使用）。

---

## CHANGELOG エントリ形式

```
## [v79.2.0] — 2026-08-16 — Temporal showcase パイプライン

### Added
- `infra/e2e-demo/favnir3-showcase/pipeline.fav`: Temporal ステージ追加（load_with_freshness / AsOfQuery / FreshnessPolicy / apply_scd2_update）

### Tests
- `showcase_temporal_freshness_check`: pipeline.fav に FreshnessPolicy / AsOfQuery が含まれることを検証
- `showcase_temporal_scd2_applied`: pipeline.fav に apply_scd2_update / ctx.run_ts が含まれることを検証
```

---

## Success Criteria

- `cargo test v792000` で 2 件が pass
- `cargo test` で 3791 tests pass（0 failures）
- `pipeline.fav` に `load_with_freshness` / `AsOfQuery` / `FreshnessPolicy` / `apply_scd2_update` が存在する

---

## Files to modify / create

| ファイル | 変更内容 |
|---|---|
| `infra/e2e-demo/favnir3-showcase/pipeline.fav` | `load_with_freshness` 関数追加（Temporal ステージ）|
| `fav/src/driver.rs` | `v792000_tests` モジュール追加（末尾）|
| `fav/Cargo.toml` | `version = "79.2.0"` に更新 |
| `CHANGELOG.md` | v79.2.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョンを更新 |
