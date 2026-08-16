# v79.4.0 仕様書 — Verifiable showcase パイプライン

Date: 2026-08-16
Status: PLANNING

---

## Background

v79.3.0 で `pipeline.fav` に Provenance ステージを追加した。
v79.4.0 では `contract.fav` に Verifiable 不変条件（v77.x スプリント実装済み）を追加する。

使用する既存 Rust 型（`fav/src/driver.rs` v77.x ブロック）:
- `PipelineInvariant` struct（v77.1.0）
- `ProbabilisticContract` struct（v77.8.0）
- `VerificationReport` struct（v77.5.0）

> **Note**: テスト数はベース 3793（v79.3.0 完了後の実測値）。完了後は 3795。

---

## Goals

`infra/e2e-demo/favnir3-showcase/contract.fav` に `Favnir3ShowcaseContract` 不変条件ブロックを追加し、ショーケーステストで内容を検証する。

---

## `contract.fav` 更新内容

既存の `ShowcaseContract3` 型・`validate_showcase_contract` 関数の下に以下を追加:

```favnir
// --- Verifiable セクション（v77.x）---
contract Favnir3ShowcaseContract {
    input:     { rows: List<Row> }
    output:    { processed: List<Row> }
    invariant: output.row_count <= input.row_count
    invariant: SUM(output.amount) >= 0.0
    probabilistic_invariant score_dist:
        confidence: 0.95, sample_size: 1000,
        property: AVG(score) BETWEEN 40.0 AND 60.0
}
```

---

## テストモジュール仕様

```rust
// --- v79.4.0: Verifiable showcase パイプライン ---
#[cfg(test)]
mod v794000_tests {
    const CONTRACT: &str = include_str!("../../infra/e2e-demo/favnir3-showcase/contract.fav");

    #[test]
    fn showcase_verifiable_invariants_declared() {
        assert!(CONTRACT.contains("Favnir3ShowcaseContract"), "contract.fav must define Favnir3ShowcaseContract");
        assert!(CONTRACT.contains("invariant"), "contract.fav must declare invariants");
        assert!(CONTRACT.contains("row_count"), "contract.fav must reference row_count invariant");
    }

    #[test]
    fn showcase_verifiable_probabilistic_contract() {
        assert!(CONTRACT.contains("probabilistic_invariant"), "contract.fav must declare probabilistic_invariant");
        assert!(CONTRACT.contains("confidence"), "contract.fav must specify confidence");
        assert!(CONTRACT.contains("sample_size"), "contract.fav must specify sample_size");
    }
}
```

注意: `use super::*` 不要（`include_str!` + `assert!` のみ）。`const CONTRACT` パターンを採用（v79.2.0 以降の慣例）。

---

## CHANGELOG エントリ形式

```
## [v79.4.0] — 2026-08-16 — Verifiable showcase パイプライン

### Added
- `infra/e2e-demo/favnir3-showcase/contract.fav`: Verifiable 不変条件追加（Favnir3ShowcaseContract / invariant / probabilistic_invariant）

### Tests
- `showcase_verifiable_invariants_declared`: contract.fav に Favnir3ShowcaseContract / invariant / row_count が含まれることを検証
- `showcase_verifiable_probabilistic_contract`: contract.fav に probabilistic_invariant / confidence / sample_size が含まれることを検証
```

---

## Success Criteria

- `cargo test v794000` で 2 件が pass
- `cargo test` で 3795 tests pass（0 failures）
- `contract.fav` に `Favnir3ShowcaseContract` / `invariant` / `probabilistic_invariant` / `confidence` / `sample_size` が存在する

---

## Files to modify

| ファイル | 変更内容 |
|---|---|
| `infra/e2e-demo/favnir3-showcase/contract.fav` | `Favnir3ShowcaseContract` 不変条件ブロック追加 |
| `fav/src/driver.rs` | `v794000_tests` モジュール追加（末尾）|
| `fav/Cargo.toml` | `version = "79.4.0"` に更新 |
| `fav/Cargo.lock` | version バンプに追随して自動更新 |
| `CHANGELOG.md` | v79.4.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョンを更新 |
