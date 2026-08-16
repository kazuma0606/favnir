# v79.5.0 仕様書 — Execution Effects showcase パイプライン

Date: 2026-08-16
Status: PLANNING

---

## Background

v79.4.0 で `contract.fav` に Verifiable 不変条件を追加した。
v79.5.0 では `pipeline.fav` に Execution Effects ステージ（v78.x スプリント実装済み）を追加する。

`fav.toml` の `[effects.cached]` / `[effects.adaptive]` セクションは v79.1.0 で追加済み。

使用する既存 Rust 型（`fav/src/driver.rs` v78.x ブロック）:
- `!Adaptive` エフェクト（v78.x）
- `!Cached` エフェクト（v78.x）

> **Note**: テスト数はベース 3795（v79.4.0 完了後の実測値）。完了後は 3797。

---

## Goals

`infra/e2e-demo/favnir3-showcase/pipeline.fav` に Execution Effects ステージ（`join_stage` 関数）を追加し、ショーケーステストで内容を検証する。

---

## `pipeline.fav` 更新内容

既存の `load_with_provenance` / `showcase_pipeline` の間に以下を追加:

```favnir
// --- Stage 3: Execution Effects（v78.x）---
fn join_stage(ctx: AppCtx, customers: List<Row>, orders: List<Row>) -> Result<List<Row>, String> !Adaptive !Cached {
    bind joined <- customers |> join(orders, on: "id")
    // !Adaptive → row 数に応じて broadcast/hash を自動選択
    // !Cached   → TTL 内は同じ入力に対してキャッシュを返す
    Result.ok(joined)
}
```

また `showcase_pipeline` のコメント行を更新:

```favnir
fn showcase_pipeline(ctx: AppCtx) -> Result<String, String> {
    // Stage 1: Temporal（v75.x）— load_with_freshness で実装済み
    // Stage 2: Provenance（v76.x）— load_with_provenance で実装済み
    // Stage 3: Verifiable（v77.x）— contract.fav で実装済み
    // Stage 4: Execution Effects（v78.x）— join_stage で実装済み
    Result.ok("favnir3-showcase: pipeline skeleton initialized")
}
```

---

## テストモジュール仕様

```rust
// --- v79.5.0: Execution Effects showcase パイプライン ---
#[cfg(test)]
mod v795000_tests {
    const PIPELINE: &str = include_str!("../../infra/e2e-demo/favnir3-showcase/pipeline.fav");

    #[test]
    fn showcase_execution_cached_effect() {
        assert!(PIPELINE.contains("join_stage"), "pipeline.fav must define join_stage");
        assert!(PIPELINE.contains("!Cached"), "pipeline.fav must declare !Cached effect");
    }

    #[test]
    fn showcase_execution_adaptive_effect() {
        assert!(PIPELINE.contains("!Adaptive"), "pipeline.fav must declare !Adaptive effect");
        assert!(PIPELINE.contains("join(orders, on:"), "pipeline.fav must reference join with on: key");
    }
}
```

注意: `use super::*` 不要（`include_str!` + `assert!` のみ）。`const PIPELINE` パターンを採用（v79.2.0 以降の慣例）。

---

## CHANGELOG エントリ形式

```
## [v79.5.0] — 2026-08-16 — Execution Effects showcase パイプライン

### Added
- `infra/e2e-demo/favnir3-showcase/pipeline.fav`: Execution Effects ステージ追加（join_stage / !Adaptive / !Cached）

### Tests
- `showcase_execution_cached_effect`: pipeline.fav に join_stage / !Cached が含まれることを検証
- `showcase_execution_adaptive_effect`: pipeline.fav に !Adaptive / join with on: が含まれることを検証
```

---

## Success Criteria

- `cargo test v795000` で 2 件が pass
- `cargo test` で 3797 tests pass（0 failures）
- `pipeline.fav` に `join_stage` / `!Adaptive` / `!Cached` / `join(orders, on:` が存在する

---

## Files to modify

| ファイル | 変更内容 |
|---|---|
| `infra/e2e-demo/favnir3-showcase/pipeline.fav` | `join_stage` 関数追加（Execution Effects ステージ）|
| `fav/src/driver.rs` | `v795000_tests` モジュール追加（末尾）|
| `fav/Cargo.toml` | `version = "79.5.0"` に更新 |
| `fav/Cargo.lock` | version バンプに追随して自動更新 |
| `CHANGELOG.md` | v79.5.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョンを更新 |
