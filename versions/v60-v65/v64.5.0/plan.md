# v64.5.0 Plan — 外部ベンチマーク比較

Version: 64.5.0
Status: 未着手

---

## 作業順序

### Step 1: `include_str!` パスの確認

`driver.rs` の `include_str!` は `fav/src/driver.rs` からの相対パスで解決される。

- `../../site/content/docs/runtime/benchmarks.mdx`
  → `C:\Users\yoshi\favnir\site\content\docs\runtime\benchmarks.mdx` ✓
- `../../benchmarks/compare/run_comparison.sh`
  → `C:\Users\yoshi\favnir\benchmarks\compare\run_comparison.sh` ✓

ロードマップの `fav/benchmarks/compare/` 記載はリポジトリルートからの表記と解釈し、
実際は `C:\Users\yoshi\favnir\benchmarks\compare\` に配置する。

### Step 2: `site/content/docs/runtime/benchmarks.mdx` 作成

frontmatter:
```
---
title: Benchmark Comparison
description: Favnir AOT vs pandas, Apache Beam, and dbt on 1M-row CSV transforms.
---
```

必須コンテンツ:
- `"Benchmark"` / `"benchmark"` のいずれかを含む
- `"pandas"` を含む比較結果

### Step 3: `benchmarks/compare/run_comparison.sh` 作成

`C:\Users\yoshi\favnir\benchmarks\compare\run_comparison.sh` を作成。

```bash
#!/usr/bin/env bash
# run_comparison.sh — Reproducible benchmark: Favnir AOT vs pandas / Apache Beam / dbt
# Usage: bash benchmarks/compare/run_comparison.sh
```

`"benchmark"` または `"run_comparison"` を含む内容。

### Step 4: `driver.rs` — `v64500_tests` 追加

`v64400_tests` の直前（`// -- v64400_tests (v64.4.0) -- flamegraph AOT --` コメント行の前）に挿入。

```rust
// -- v64500_tests (v64.5.0) -- 外部ベンチマーク比較 --
#[cfg(test)]
mod v64500_tests {
    #[test]
    fn docs_benchmarks_page_exists() {
        let content = include_str!("../../site/content/docs/runtime/benchmarks.mdx");
        assert!(!content.is_empty(), "benchmarks.mdx should not be empty");
        assert!(content.contains("Benchmark") || content.contains("benchmark"),
            "benchmarks.mdx should mention 'Benchmark'");
        assert!(content.contains("pandas"),
            "benchmarks.mdx should compare with pandas");
    }

    #[test]
    fn benchmark_compare_script_exists() {
        let content = include_str!("../../benchmarks/compare/run_comparison.sh");
        assert!(!content.is_empty(), "run_comparison.sh should not be empty");
        assert!(
            content.contains("run_comparison") || content.contains("benchmark"),
            "run_comparison.sh should mention benchmark or run_comparison"
        );
    }
}
```

### Step 5: ビルド・テスト

```bash
cd /c/Users/yoshi/favnir/fav && cargo build 2>&1 | tail -5
cargo test --bin fav v64500_tests 2>&1 | tail -10
cargo test -j 8 -- --test-threads=8 2>&1 | grep "^test result"
```

---

## 注意事項

- `benchmarks/compare/` ディレクトリは新規作成（`mkdir -p` 相当が必要）
- `run_comparison.sh` は実行権限付与不要（`include_str!` でテストするのみ）
- `benchmarks.mdx` の `include_str!` パスは `performance.mdx` と同じディレクトリ
