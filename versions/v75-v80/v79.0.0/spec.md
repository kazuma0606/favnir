# v79.0.0 仕様書 — Execution Effects 1.0 宣言 ★クリーンアップ

Date: 2026-08-16
Status: PLANNING

---

## Background

v78.1〜v78.9 で Execution Effects 基盤（キャッシュ統計 / LRU / 結合戦略 / コスト推定 / 実行計画可視化 / 並列実行 / 実行モード / 計画キャッシュ）が完成した。
v79.0.0 はこれを「Execution Effects 1.0」として宣言するクリーンアップバージョン。

宣言文:
> 「`!Cached` がメモを持ち、`!Adaptive` が状況を読み、`!Parallel` が仕事を分ける。
>  実行戦略が型となった Favnir は、最適解を自ら選ぶ。」

---

## Goals

- `cargo clean` による target/ クリーンアップ
- `Cargo.toml` バージョンを `79.0.0` に更新
- `CHANGELOG.md` に v79.0.0 エントリを追加
- `MILESTONE.md` に「Execution Effects 1.0」マイルストーン節を追加
- `README.md` に v79.0 達成を追記
- `versions/current.md` を更新
- `v79000_tests` モジュール（4 件）を `driver.rs` に追加
- テスト総数: 3783 → 3787（+4）

---

## 新規型・関数（なし）

宣言バージョンにつき新機能追加なし。ドキュメントとテストのみ。

---

## テストモジュール仕様

```rust
// --- v79.0.0: Execution Effects 1.0 宣言 ★クリーンアップ ---
#[cfg(test)]
mod v79000_tests {
    // include_str! のみ使用。use super::* は不要（外部シンボル未使用）

    #[test]
    fn cargo_toml_version_is_79_0_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(toml.contains("version = \"79.0.0\""));
    }

    #[test]
    fn changelog_has_v79_0_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v79.0.0]"));
    }

    #[test]
    fn milestone_has_execution_effects() {
        let ms = include_str!("../../MILESTONE.md");
        assert!(ms.contains("Execution Effects 1.0"));
    }

    #[test]
    fn readme_mentions_execution_effects() {
        let rm = include_str!("../../README.md");
        assert!(rm.contains("Execution Effects"));
    }
}
```

---

## CHANGELOG エントリ形式

```
## [v79.0.0] — 2026-08-16 — Execution Effects 1.0 宣言 ★クリーンアップ

### Added
- Execution Effects 1.0 宣言（v78.1〜v78.9 の全 Execution Effects 基盤の完成を宣言）

### Tests
- `cargo_toml_version_is_79_0_0`: Cargo.toml のバージョンが 79.0.0 であることを検証
- `changelog_has_v79_0_0`: CHANGELOG.md に v79.0.0 エントリが存在することを検証
- `milestone_has_execution_effects`: MILESTONE.md に「Execution Effects 1.0」が存在することを検証
- `readme_mentions_execution_effects`: README.md に「Execution Effects」が存在することを検証
```

---

## MILESTONE.md 追加内容

`## v78.0.0` 節の直前に以下を追加:

```markdown
## v79.0.0（2026-08-16）— Execution Effects 1.0 宣言

> 「`!Cached` がメモを持ち、`!Adaptive` が状況を読み、`!Parallel` が仕事を分ける。
>  実行戦略が型となった Favnir は、最適解を自ら選ぶ。」

**Execution Effects 1.0** の宣言バージョン。v78.1〜v78.9 で実装した
Execution Effects 基盤の完成を宣言した。

**v78.1〜v78.9 達成内容:**
- `CacheEntry` / `CacheStats` / `simulate_lru_cache` / `format_cache_stats_report`（LRU キャッシュ統計）— v78.1.0
- `hit_rate` / `merge_cache_stats`（ヒット率・統計マージ）— v78.2.0
- `ExecutionStrategy` / `select_join_strategy`（結合戦略選択）— v78.3.0
- `CostEstimate` / `combine_costs` / `estimate_cost`（コスト推定）— v78.4.0
- `PlanStage` / `ExecutionPlan` / `format_execution_plan`（実行計画可視化）— v78.5.0
- `ParallelConfig` / `PartitionPlan` / `plan_parallel_execution` / `format_parallel_plan`（並列実行）— v78.6.0
- `ExecutionMode` / `ExecutionModeSelector` / `select_execution_mode`（実行モード選択）— v78.7.0
- `PlanCacheEntry` / `PlanCache` / `lookup_plan` / `insert_plan`（実行計画キャッシュ）— v78.8.0
- 安定化・E2E テスト（`execution_effects_full_sprint_all_stable` / `execution_effects_e2e_pipeline_runs`）— v78.9.0

---
```

---

## README.md 追加内容

既存の `## v78.0 — Verifiable Pipelines 宣言` 節の直前に以下を追加:

```markdown
## v79.0 — Execution Effects 1.0 宣言（2026-08-16）

Favnir v79.0 で **Execution Effects 1.0** を宣言しました。

実行戦略（キャッシュ・並列・バッチ/ストリーミング判断）が Favnir の型システムに組み込まれ、
パイプラインが最適な実行戦略を自ら選択できるようになりました。
```

---

## Success Criteria

- `cargo test v79000` で 4 件が pass
- `cargo test` で 3787 tests pass（0 failures）
- `Cargo.toml` の version が `"79.0.0"`
- `CHANGELOG.md` の先頭エントリが `[v79.0.0]`
- `MILESTONE.md` に `Execution Effects 1.0` が存在する
- `README.md` に `Execution Effects` が存在する

---

## Files to modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v79000_tests` モジュール追加（末尾）+ `78.9.0` バージョン文字列アサーションを `79.0.0` に一括更新（`replace_all: true`）|
| `fav/Cargo.toml` | `version = "79.0.0"` に更新 |
| `CHANGELOG.md` | v79.0.0 エントリを先頭に追加 |
| `MILESTONE.md` | v79.0.0 節を先頭に追加 |
| `README.md` | v79.0 達成を追記 |
| `versions/current.md` | 進行中バージョンと次バージョンを更新 |
