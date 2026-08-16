# v78.9.0 仕様書 — 安定化・コードフリーズ

Date: 2026-08-16

---

## Background

v78.1.0〜v78.8.0 で `!Cached` / `!Adaptive` / `!Parallel` エフェクト基盤を構築した。
v78.9.0 はコードフリーズ安定化スプリントであり、**新機能の追加は一切行わない**。
v78.1〜v78.8 の全型・関数が連携して正しく動作することを統合テストで証明し、v79.0.0 宣言に向けて品質を確定する。

---

## Goals

- v78.1〜v78.8 の全型・関数を横断する統合テストを 2 件追加する
- バグ修正のみ受け入れる（新機能追加なし）
- 対象: `fav/src/driver.rs` のみ（他ファイル変更なし）

---

## Success Criteria

1. `execution_effects_full_sprint_all_stable`: v78.1〜v78.8 の全スプリント型が単一テスト内で正しく連携することを検証
2. `execution_effects_e2e_pipeline_runs`: パイプライン全体（モード選択 → コスト推定 → 計画生成 → キャッシュ → 取得 → 可視化）の E2E 動作を検証
3. Rust テスト 2 件（`v789000_tests` モジュール）が pass
4. `cargo test` 全 pass（3781 + 2 = 3783 tests）

---

## 統合テスト設計

### `execution_effects_full_sprint_all_stable`

v78.1〜v78.8 の主要型・関数を順に呼び出し、すべて期待値を返すことを assert する:

| スプリント | 使用型/関数 | assert 内容 |
|---|---|---|
| v78.1 | `CacheEntry` / `check_cache_valid` | TTL 内でキャッシュが有効 |
| v78.2 | `simulate_lru_cache` / `hit_rate` | ヒット率が 0.0 以上 |
| v78.3 | `select_join_strategy` | 小テーブル → BroadcastJoin |
| v78.4 | `estimate_broadcast_cost` / `select_min_cost_strategy` | コスト比較が正常 |
| v78.5 | `format_execution_plan` | 出力に "Execution Plan: StableCheck" を含む |
| v78.6 | `plan_parallel_execution` | partition 数が一致 |
| v78.7 | `select_execution_mode` | 小データ + 高レイテンシ許容 → Adaptive |
| v78.8 | `insert_plan` / `lookup_plan` | キャッシュヒット |

### `execution_effects_e2e_pipeline_runs`

単一の "シミュレートされたパイプライン実行" を通じて E2E を検証:

1. `ExecutionModeSelector` で `Batch` モードを選択
2. `estimate_broadcast_cost` / `estimate_hash_cost` でコスト推定
3. `select_min_cost_strategy` で最小コスト戦略を選択
4. `ExecutionPlan` を構築し `format_execution_plan` で可視化
5. `PlanCache` に `insert_plan` で挿入
6. `lookup_plan` で取得 → `Some` を assert
7. 取得した plan の pipeline 名を assert

---

## Error Codes

なし

---

## 注記

`changelog_has_v78_9_0` テストの追加は対象外。x.0.0 宣言バージョン（例: v73.0.0、v74.0.0）のみに追加する慣例のため、パッチ・マイナーバージョンでは追加しない。

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v789000_tests` モジュール追加（統合テスト 2 件） |
| `fav/Cargo.toml` | version を `78.8.0` → `78.9.0` に変更 |
| `fav/Cargo.lock` | 自動更新 |
| `CHANGELOG.md` | v78.9.0 エントリ追加 |
| `versions/current.md` | 進行中バージョンを v78.9.0 に更新 |
