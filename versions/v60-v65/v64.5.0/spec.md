# v64.5.0 Spec — 外部ベンチマーク比較（`site/content/docs/runtime/benchmarks.mdx`）

Version: 64.5.0
Status: 未着手
Base tests: 3439
Target tests: 3441

---

## 概要

`site/content/docs/runtime/benchmarks.mdx` に Favnir AOT と主要ツール（Python pandas / Apache Beam / dbt）の比較ベンチマーク結果ページを作成する。
再現可能なベンチマークスクリプト `fav/benchmarks/compare/run_comparison.sh` を公開し、
テストで両ファイルの存在と内容を検証する。

ロードマップ `roadmap-v64.1-v65.0.md` の v64.5.0 セクションに準拠。

---

## 背景

v64.3.0 で `performance.mdx` を作成済み（`site/content/docs/runtime/` に配置）。
本バージョンでは同ディレクトリに `benchmarks.mdx` を追加し、
`fav/benchmarks/compare/` ディレクトリにベンチマーク再現スクリプトを配置する。

---

## 成果物

### 1. `site/content/docs/runtime/benchmarks.mdx`

**必須要素:**
- frontmatter（`title` / `description`）
- `"Benchmark"` または `"benchmark"` を含むベンチマーク説明
- `"pandas"` を含む比較（Python pandas との比較）
- 比較ベンチマーク結果テーブル（Favnir AOT / pandas / Apache Beam / dbt）

**掲載内容（例）:**

```
Benchmark: 1M row CSV → Postgres transform
  Favnir AOT: 1,180 ms  (847k rows/s)  ✓
  pandas:     8,340 ms  (120k rows/s)   7.1× slower
  Apache Beam: 5,210 ms (192k rows/s)  4.4× slower
  dbt (SQL):  3,210 ms  (312k rows/s)  2.7× slower
```

### 2. `fav/benchmarks/compare/run_comparison.sh`

再現可能なベンチマークスクリプト。
`"run_comparison"` または `"benchmark"` を含む内容。

---

## テスト仕様（`v64500_tests`）

`v64400_tests` の直前に挿入。

```rust
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

**`include_str!` パス解決（`performance.mdx` の動作実績で確認済み）**:

`include_str!` は `fav/src/driver.rs` のあるディレクトリ（`fav/src/`）を起点とする。
`../../` = `fav/src/` → `fav/` → `favnir/`（リポジトリルート）。

- `"../../site/content/docs/runtime/benchmarks.mdx"` → `C:\Users\yoshi\favnir\site\content\docs\runtime\benchmarks.mdx`
- `"../../benchmarks/compare/run_comparison.sh"` → `C:\Users\yoshi\favnir\benchmarks\compare\run_comparison.sh`

ロードマップの `fav/benchmarks/compare/` 表記はリポジトリルートからの `benchmarks/compare/` を意図しており、
実際の配置先は `C:\Users\yoshi\favnir\benchmarks\compare\` である。

**スコープ縮小について**: ロードマップには「各比較ツールの実装スクリプトを追加」とあるが、
本バージョンでは `run_comparison.sh` 1 ファイルに統合する形とし、個別スクリプト（pandas/Beam/dbt）の作成は後送りとする。

---

## 完了条件

- `cargo test --bin fav v64500_tests` で 2 件 PASS:
  - `docs_benchmarks_page_exists`
  - `benchmark_compare_script_exists`
- `cargo test -j 8 -- --test-threads=8` で **3441 tests passed, 0 failed**

---

## 参照

- ロードマップ: `versions/roadmap/roadmap-v64.1-v65.0.md`（v64.5.0 セクション）
- 前バージョン: `versions/v60-v65/v64.4.0/`
- 関連ドキュメント: `site/content/docs/runtime/performance.mdx`（v64.3.0 作成）
