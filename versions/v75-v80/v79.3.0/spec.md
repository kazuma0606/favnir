# v79.3.0 仕様書 — Provenance showcase パイプライン

Date: 2026-08-16
Status: PLANNING

---

## Background

v79.2.0 で `pipeline.fav` に Temporal ステージを追加した。
v79.3.0 では Provenance 機能（v76.x スプリント実装済み）を追加する。

使用する既存 Rust 型・関数（`fav/src/driver.rs` v76.x ブロック）:
- `DataSource` struct（v76.1.0）
- `TracedData` struct / `map_traced` 関数（v76.2.0）
- `OpenLineageFacet` struct / `provenance_to_openlineage` 関数（v76.4.0）

> **Note**: ロードマップ記載のテスト数（3793）はベース 3791 からの +2 で正しい。
> 実際のベースは 3791（v79.2.0 完了後の実測値）。完了後は 3793。

---

## Goals

`infra/e2e-demo/favnir3-showcase/pipeline.fav` に `load_with_provenance` 関数を追加し、
ショーケーステストで内容を検証する。

---

## `pipeline.fav` 更新内容

`load_with_freshness` 関数の下に以下を追加する:

```favnir
// --- Stage 2: Provenance（v76.x）---
fn load_with_provenance(ctx: AppCtx, rows: List<Row>) -> Result<TracedData, String> {
    bind source <- DataSource {
        name: "snowflake-crm",
        uri: "snowflake://warehouse/crm/users",
        source_type: Snowflake
    }
    bind raw    <- TracedData.wrap(rows, source)
    bind masked <- raw |> TracedData.map(mask_pii, label: "mask_pii")
    bind facet  <- OpenLineage.from_provenance(masked.provenance)
    Result.ok(masked)
}
```

`showcase_pipeline` のコメントを「Stage 2: load_with_provenance で実装済み」に更新する。

注意: `rows` をシグネチャ引数として明示（v79.2.0 code-reviewer 指摘の教訓）。

---

## テストモジュール仕様

```rust
// --- v79.3.0: Provenance showcase パイプライン ---
#[cfg(test)]
mod v793000_tests {
    const PIPELINE: &str = include_str!("../../infra/e2e-demo/favnir3-showcase/pipeline.fav");

    #[test]
    fn showcase_provenance_traced() {
        assert!(PIPELINE.contains("load_with_provenance"), "pipeline.fav must define load_with_provenance");
        assert!(PIPELINE.contains("TracedData"), "pipeline.fav must reference TracedData");
        assert!(PIPELINE.contains("DataSource"), "pipeline.fav must reference DataSource");
        assert!(PIPELINE.contains("mask_pii"), "pipeline.fav must reference mask_pii");
    }

    #[test]
    fn showcase_provenance_openlineage_generated() {
        assert!(PIPELINE.contains("OpenLineage"), "pipeline.fav must reference OpenLineage");
        assert!(PIPELINE.contains("masked.provenance"), "pipeline.fav must reference masked.provenance");
    }
}
```

注意: `use super::*` 不要（`include_str!` + `assert!` のみ使用）。
`const PIPELINE` パターンを v79.2.0 と同様に採用（code-reviewer 推奨）。

---

## CHANGELOG エントリ形式

```
## [v79.3.0] — 2026-08-16 — Provenance showcase パイプライン

### Added
- `infra/e2e-demo/favnir3-showcase/pipeline.fav`: Provenance ステージ追加（load_with_provenance / TracedData / DataSource / OpenLineage）

### Tests
- `showcase_provenance_traced`: pipeline.fav に TracedData / DataSource / mask_pii が含まれることを検証
- `showcase_provenance_openlineage_generated`: pipeline.fav に OpenLineage / masked.provenance が含まれることを検証
```

---

## Success Criteria

- `cargo test v793000` で 2 件が pass
- `cargo test` で 3793 tests pass（0 failures）
- `pipeline.fav` に `load_with_provenance` / `TracedData` / `DataSource` / `OpenLineage` / `masked.provenance` が存在する

---

## Files to modify

| ファイル | 変更内容 |
|---|---|
| `infra/e2e-demo/favnir3-showcase/pipeline.fav` | `load_with_provenance` 関数追加（Provenance ステージ）+ showcase_pipeline コメント更新 |
| `fav/src/driver.rs` | `v793000_tests` モジュール追加（末尾）|
| `fav/Cargo.toml` | `version = "79.3.0"` に更新 |
| `CHANGELOG.md` | v79.3.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョンを更新 |
| `fav/Cargo.lock` | version バンプに追随して自動更新 |
