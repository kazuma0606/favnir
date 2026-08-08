# v64.8.0 Spec — ドキュメントサイト Performance 1.0 総括記事

Version: 64.8.0
Status: 未着手
Base tests: 3445
Target tests: 3447

---

## 概要

`site/content/docs/performance/performance1-overview.mdx` を新規作成する。
v61〜v64 の全機能（DX 2.0 / Language Polish / AOT Native / Incremental & Scale）を
統括する概観記事とクイックスタートガイドを記述。
認定チェックリスト形式（`fav bench` / `fav profile` / `fav build --ci` の通過確認）を掲載。

`driver.rs` に `v64800_tests` を追加し、ファイル存在とクイックスタートセクション記載を
`include_str!` で検証する。

ロードマップ `roadmap-v64.1-v65.0.md` の v64.8.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3445 tests passed, 0 failed を確認
- `site/content/docs/performance/performance1-overview.mdx` が存在しないことを確認（新規作成）
- `site/content/docs/performance/` ディレクトリが存在することを確認（既存）
- `driver.rs` に `v64800_tests` が存在しないことを確認（新規追加）
- `driver.rs` に `v64700_tests` が存在することを確認（`v64800_tests` の挿入位置）

---

## 実装スコープ

### 1. `site/content/docs/performance/performance1-overview.mdx` 新規作成

```mdx
---
title: Performance 1.0 Overview
description: Favnir Performance 1.0 — AOT native compilation, incremental builds, flamegraph profiling, and benchmark comparison in one guide.
---

# Performance 1.0 Overview

Favnir v65.0 achieves **Performance 1.0**: type-safe pipelines that compile to native code,
incremental rebuilds that skip unchanged stages, and benchmarks that outperform pandas by 7×.

## What's in Performance 1.0

| Feature | Version | Command |
|---|---|---|
| AOT native compilation | v62.1 | `fav build --link` |
| CI-ready build output | v64.1 | `fav build --ci` |
| Benchmark regression detection | v64.2 | `fav bench --compare` |
| Performance tuning guide | v64.3 | — |
| Flamegraph (AOT) | v64.4 | `fav profile --flamegraph` |
| External benchmark comparison | v64.5 | `benchmarks/compare/run_comparison.sh` |
| Performance lint | v64.6 | `fav lint --perf` |
| WASM build | v64.7 | `cmd_build_wasm` |

## Quick Start

Get started with Performance 1.0 in four steps:

```bash
# 1. AOT ビルド — ネイティブバイナリ生成
fav build pipeline.fav --link -o dist/pipeline

# 2. ベンチマーク確認
fav bench pipeline.fav --runs 10

# 3. メモリ・CPU プロファイル
fav profile --memory pipeline.fav

# 4. パフォーマンス lint
fav lint --perf pipeline.fav
```

## Performance Certification Checklist

Before declaring a pipeline "Performance 1.0 certified", verify:

- [ ] `fav build pipeline.fav --link --ci -o dist/pipeline` exits 0
- [ ] `fav bench pipeline.fav --runs 10` shows AOT speedup ≥ 3×
- [ ] `fav profile --memory pipeline.fav` shows no stage exceeds budget
- [ ] `fav lint --perf pipeline.fav` reports 0 warnings

## Benchmark Results

Favnir AOT vs industry tools (1M row CSV → Postgres transform):

```
Favnir AOT:   1,180 ms  (847k rows/s)  ✓
pandas:       8,340 ms  (120k rows/s)  7.1× slower
Apache Beam:  5,210 ms  (192k rows/s)  4.4× slower
dbt (SQL):    3,210 ms  (312k rows/s)  2.7× slower
```

See [benchmarks](./benchmarks) for full methodology and reproduction scripts.

## Further Reading

- [AOT Compilation](./aot) — compile to native binary
- [Performance Tuning Guide](./performance) — bottleneck identification workflow
- [Benchmark Comparison](./benchmarks) — external tool comparison
- [Profiling](./profiling) — flamegraph and memory profiling
```

**注意（実装時）**: 上記 MDX 草稿はネストしたコードフェンス（` ```bash ` 等）を含む。
実際の `.mdx` ファイルに書き出す際は外側の mdx ブロックとの混同が生じないよう、
ファイルを直接 `Write` ツールで作成すること（plan.md §注意事項も参照）。

### 2. `driver.rs` — `v64800_tests` 追加

`v64700_tests` の直前に挿入:

```rust
// -- v64800_tests (v64.8.0) -- Performance 1.0 総括記事 --
#[cfg(test)]
mod v64800_tests {
    #[test]
    fn docs_performance1_overview_exists() {
        let content = include_str!("../../site/content/docs/performance/performance1-overview.mdx");
        assert!(!content.is_empty(), "performance1-overview.mdx should not be empty");
        assert!(
            content.contains("Performance 1.0"),
            "should mention 'Performance 1.0': {}",
            &content[..content.len().min(200)]
        );
    }

    #[test]
    fn docs_performance1_has_quickstart() {
        let content = include_str!("../../site/content/docs/performance/performance1-overview.mdx");
        assert!(
            content.contains("fav build"),
            "quickstart should mention 'fav build': {}",
            &content[..content.len().min(200)]
        );
        assert!(
            content.contains("fav bench"),
            "quickstart should mention 'fav bench': {}",
            &content[..content.len().min(200)]
        );
        assert!(
            content.contains("fav profile"),
            "quickstart should mention 'fav profile': {}",
            &content[..content.len().min(200)]
        );
        assert!(
            content.contains("fav lint"),
            "quickstart should mention 'fav lint': {}",
            &content[..content.len().min(200)]
        );
    }
}
```

---

## 完了条件

- `cargo build` エラーなし
- `cargo test --bin fav v64800_tests` で 2 件 PASS
  - `docs_performance1_overview_exists` PASS
  - `docs_performance1_has_quickstart` PASS
- `cargo test -j 8 -- --test-threads=8` で 3447 tests passed, 0 failed

---

## 非スコープ

- MDX の Next.js レンダリング確認（サイトビルドの動作確認）
- `site/` の navigation/sidebar への `performance1-overview.mdx` 追記
- `fav build --target wasm32` の CLI dispatch 統合（v64.9 以降）
- `fav lint --perf` CLI フラグ（`main.rs`）の統合（v64.9 以降）

---

## 技術ノート

### `include_str!` パス

`driver.rs`（`fav/src/driver.rs`）から
`../../site/content/docs/performance/performance1-overview.mdx` を解決すると
`favnir/site/content/docs/performance/performance1-overview.mdx` になる
（`fav/src/` から `../../` = `fav/` の親ディレクトリ = `favnir/`（リポジトリルート））。

`site/content/docs/performance/` ディレクトリは既存（`arrow.mdx` 等あり）。

### テストの検証対象

- `docs_performance1_overview_exists`: 空でなく `"Performance 1.0"` を含む
- `docs_performance1_has_quickstart`: `"fav build"` / `"fav bench"` / `"fav profile"` / `"fav lint"` を含む

ロードマップの完了条件テスト名と完全一致させる。

### テスト挿入位置

`v64700_tests` の直前（`// -- v64700_tests` コメント行の直前）に挿入する。
