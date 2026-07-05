# v33.9.0 — 実装プラン

## 方針

確認・記録パターン。v19.4.0 実装済みの並列コンパイル（`compile_parallel` / `topo_layers`）を 4 テストで確認する。

---

## 実装ステップ

### Step 1: Cargo.toml バージョン更新
`fav/Cargo.toml` の version を `33.8.0` → `33.9.0` に変更。

### Step 2: benchmarks/v33.9.0.json 作成
```json
{
  "version": "33.9.0",
  "milestone": "Performance & Tooling",
  "date": "2026-07-04",
  "tests_passed": 2532,
  "tests_failed": 0,
  "notes": "並列コンパイル確認（topo_layers 循環依存エラー / compile_parallel 空ソース境界ケース）。v339000_tests 4件追加。"
}
```
（`tests_passed` は `cargo test` 実測後に確定）

### Step 3: driver.rs 更新
1. `cargo_toml_version_is_33_8_0` を空スタブ化（v338000_tests は `#[cfg(not(target_arch = "wasm32"))]` ゲートを持つが、スタブ化してもゲートは外さないこと）
2. `v338000_tests` 直後・`// ── v31.7.0 tests` の前に `v339000_tests` を挿入

> **WASM ゲートについて**: v338000_tests は profiler を import するため `#[cfg(not(target_arch = "wasm32"))]` ゲートが必要だが、v339000_tests が使う `parallel` モジュールは `lib.rs` で `pub mod parallel;`（無条件公開）のためゲート不要。v194000_tests と同じパターン（`#[cfg(test)]` のみ）を採用する。

```rust
// ── v33.9.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v339000_tests {
    use crate::parallel::{compiler::compile_parallel, topo::topo_layers};
    use crate::incremental::dep_graph::DepGraph;

    #[test]
    fn cargo_toml_version_is_33_9_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("33.9.0"), "Cargo.toml must contain '33.9.0'");
    }

    #[test]
    fn benchmark_v33_9_0_exists() {
        let src = include_str!("../../benchmarks/v33.9.0.json");
        assert!(src.contains("33.9.0"), "benchmarks/v33.9.0.json must contain '33.9.0'");
    }

    #[test]
    fn parallel_topo_cyclic_dep_returns_err() {
        // a → b かつ b → a の循環依存で topo_layers が Err を返すことを確認
        let mut graph = DepGraph::new();
        graph.add_dep("a", "b");
        graph.add_dep("b", "a");
        let files = vec!["a".to_string(), "b".to_string()];
        let result = topo_layers(&files, &graph);
        assert!(result.is_err(), "cyclic dependency must return Err");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("circular"),
            "error message must mention 'circular', got: {msg}"
        );
    }

    #[test]
    fn parallel_compile_empty_sources() {
        // 空ソースリストで compile_parallel が Ok(IRProgram { fns: [] }) を返すことを確認
        let result = compile_parallel(vec![], 1);
        assert!(result.is_ok(), "empty sources should compile ok: {:?}", result);
        let ir = result.unwrap();
        assert_eq!(ir.fns.len(), 0, "empty sources should produce 0 fns");
    }
}
```

### Step 4: CHANGELOG.md 更新
先頭に `[v33.9.0]` セクションを追加。

### Step 5: versions/current.md 更新
最新安定版を v33.9.0 に変更。

---

## テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test --bin fav v339000 2>&1 | tail -8
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

---

## 完了処理

- `benchmarks/v33.9.0.json` の `tests_passed` を実測値で確定
- `tasks.md` を COMPLETE に更新（全チェックボックス `[x]`）
