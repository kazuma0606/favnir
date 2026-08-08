# v64.3.0 Spec — パフォーマンスガイド（`site/content/docs/runtime/performance.mdx`）

Version: 64.3.0
Status: 未着手

---

## 概要

`site/content/docs/runtime/performance.mdx` を新規作成する。
AOT コンパイル・差分コンパイル・並列最適化・DAG 最適化・バックプレッシャーの
使い方をまとめたパフォーマンスチューニングガイド。
`fav bench` / `fav profile` の出力の読み方・ボトルネック特定手順を掲載。
既存の `aot.mdx`（v62.9 作成）と並んで `site/content/docs/runtime/` に配置。

`driver.rs` に `v64300_tests` を追加し、ファイル存在と AOT セクション記載を `include_str!` で検証する。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3435 tests passed, 0 failed を確認
- `site/content/docs/runtime/performance.mdx` が存在しないことを確認（新規作成）
- `site/content/docs/runtime/aot.mdx` が存在することを確認（同ディレクトリに並べて配置）
- `driver.rs` に `v64300_tests` が存在しないことを確認（新規追加）
- `driver.rs` に `v64200_tests` が存在することを確認（`v64300_tests` の挿入位置）

---

## 実装スコープ

### 1. `site/content/docs/runtime/performance.mdx` 新規作成

```mdx
---
title: Performance Tuning Guide
description: Optimize Favnir pipelines with AOT compilation, incremental builds, parallel execution, DAG optimization, and backpressure control.
---

# Performance Tuning Guide

This guide covers the key techniques for maximizing Favnir pipeline performance.

## AOT Compilation

Compile pipelines to native binaries for maximum throughput.

```bash
# AOT ビルド → ネイティブバイナリ生成
fav build pipeline.fav --link -o dist/pipeline

# CI 向け機械可読出力
fav build pipeline.fav --link --ci -o dist/pipeline
```

Use `fav bench` to compare VM vs AOT performance:

```bash
fav bench pipeline.fav --runs 10
```

Output:
```
Mode     | Mean (ms) | P99 (ms)
---------|-----------|----------
VM       |     8.200 |    9.100
AOT      |     1.400 |    1.600
Speedup  |    5.86x  |
```

## Incremental Compilation

Favnir caches compilation artifacts in `.fav-cache/`. Only changed stages are recompiled.

```bash
fav run pipeline.fav          # 初回: full compile
fav run pipeline.fav          # 2回目: cache hit (skipped recompile)
fav cache status              # キャッシュ状態確認
```

## Parallel Execution

Use `par` to run independent stages concurrently:

```bash
fav run pipeline.fav --opt-stats   # DAG 最適化の効果確認
fav parallel-stats                 # スレッド数・キュー深度確認
```

## DAG Optimization

Favnir automatically eliminates dead stages and fuses consecutive pure stages.

```bash
fav run pipeline.fav --opt-stats
```

Output:
```
opt-stats: 1 dead stage(s) eliminated, 2 pure stage(s) fused into 1 pure_run block
```

## Backpressure Control

Configure `[backpressure]` in `fav.toml` to prevent queue overflow:

```toml
[backpressure]
strategy = "block"
max_queue_depth = 500
warn_threshold = 400
```

## Bottleneck Identification Workflow

```bash
# 1. メモリプロファイル
fav profile --memory pipeline.fav

# 2. AOT で高速化
fav build pipeline.fav --link -o dist/pipeline

# 3. DAG 最適化の効果確認
fav run pipeline.fav --opt-stats

# 4. ベンチマーク比較
fav bench pipeline.fav --runs 10
```

## Regression Detection

Track performance over time with benchmark comparison:

```toml
[bench]
regression_threshold_pct = 10
```

```bash
fav bench pipeline.fav --suite etl-standard
```
```

### 2. `driver.rs` — `v64300_tests` 追加

`v64200_tests` の直前に挿入:

```rust
// -- v64300_tests (v64.3.0) -- パフォーマンスガイド --
#[cfg(test)]
mod v64300_tests {
    #[test]
    fn docs_performance_guide_exists() {
        let content = include_str!("../../site/content/docs/runtime/performance.mdx");
        assert!(!content.is_empty(), "performance.mdx should not be empty");
        assert!(content.contains("Performance"), "should mention Performance: {}", &content[..content.len().min(200)]);
    }

    #[test]
    fn docs_performance_has_aot_section() {
        let content = include_str!("../../site/content/docs/runtime/performance.mdx");
        assert!(content.contains("AOT"), "performance guide should have AOT section: {}", &content[..content.len().min(200)]);
        assert!(content.contains("fav bench"), "performance guide should mention fav bench: {}", &content[..content.len().min(200)]);
        assert!(content.contains("fav profile"), "performance guide should mention fav profile: {}", &content[..content.len().min(200)]);
    }
}
```

---

## 完了条件

- `cargo build` エラーなし
- `cargo test --bin fav v64300_tests` で 2 件 PASS
  - `docs_performance_guide_exists` PASS
  - `docs_performance_has_aot_section` PASS
- `cargo test -j 8 -- --test-threads=8` で 3437 tests passed, 0 failed

---

## 非スコープ

- MDX の Next.js レンダリング確認（サイトビルドの動作確認）
- `site/` の navigation/sidebar への `performance.mdx` 追記
- `fav profile --memory` の実際の CLI 統合（コマンド例はガイド内に記載のみ）
- `.fav-cache/` キャッシュ機構の新規実装（v63.x で実装済み・ガイドはコマンド例記載のみ）
- `fav parallel-stats` コマンドの新規実装（v63.4.0 で `cmd_parallel_stats` として実装済み・ガイドはコマンド例記載のみ）

---

## 技術ノート

### `include_str!` パス

`driver.rs`（`fav/src/driver.rs`）から `../../site/content/docs/runtime/performance.mdx` を解決すると
`favnir/site/content/docs/runtime/performance.mdx` になる（既存 `aot.mdx` 等と同じパターン）。

### テストの検証対象

- `docs_performance_guide_exists`: ファイルが空でなく `"Performance"` を含む
- `docs_performance_has_aot_section`: `"AOT"` `"fav bench"` `"fav profile"` を含む

ロードマップの完了条件テスト名 (`docs_performance_guide_exists` / `docs_performance_has_aot_section`) と一致させる。

### ベーステスト数

実際のベース: 3435（v64.2.0 完了後）
ロードマップ記載: 3422（古い値）
目標: 3435 + 2 = **3437**
