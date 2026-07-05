# v33.9.0 — Spec

## 概要

**テーマ**: 並列コンパイル 確認（ファイル単位での並列型チェック）

**方針**: 確認・記録パターン。v19.4.0 で実装済みの並列コンパイル機能を `v339000_tests` 4 件で確認する。新規コードは追加しない。

---

## 背景

v19.4.0 で実装済み:
- `parallel::compiler::compile_parallel(sources, jobs)` — `Vec<(String, String)>` を rayon で並列コンパイルして `IRProgram` を返す
- `parallel::topo::topo_layers(files, graph)` — Kahn's algorithm によるトポロジカル層分割（循環依存は `Err` を返す）
- `incremental::dep_graph::DepGraph` — 依存グラフ（`add_dep` / `deps_of` / `affected_by`）

v194000_tests（既存）でカバー済み:
- `parallel_compile_same_output` — jobs=1 で逐次と fn 数が一致
- `parallel_compile_faster` — 3 ソース並列コンパイル成功
- `parallel_dep_order_respected` — a→b→c の依存グラフで topo_layers が正しい層順を返す
- `parallel_compile_thread_count` — jobs=0 / jobs=1 両方で成功

v339000_tests では **循環依存エラー** と **空ソースの境界ケース** をカバーする。

---

## 実装スコープ

### 変更ファイル
1. `fav/Cargo.toml` — version `33.8.0` → `33.9.0`
2. `fav/src/driver.rs` — `cargo_toml_version_is_33_8_0` をスタブ化、`v339000_tests` 4 件追加
3. `benchmarks/v33.9.0.json` — 新規作成
4. `CHANGELOG.md` — `[v33.9.0]` セクション先頭追記
5. `versions/current.md` — 最新安定版を v33.9.0 に更新

### 新規ファイル
- `versions/v30-v35/v33.9.0/` — spec.md / plan.md / tasks.md

---

## テスト仕様（v339000_tests）

```rust
#[cfg(not(target_arch = "wasm32"))]
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
        // v194000_tests は正常系（DAG）のみカバー
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
        // v194000_tests は 1〜3 ソースのケースのみカバー
        let result = compile_parallel(vec![], 1);
        assert!(result.is_ok(), "empty sources should compile ok: {:?}", result);
        let ir = result.unwrap();
        assert_eq!(ir.fns.len(), 0, "empty sources should produce 0 fns");
    }
}
```

### 設計注記
- `parallel` モジュールは `lib.rs` で `#[cfg(not(target_arch = "wasm32"))] pub mod parallel` — WASM ゲート必須。`#[cfg(not(target_arch = "wasm32"))]` → `#[cfg(test)]` の順で付与する
- `topo_layers` の Err メッセージは `"circular dependency detected"` — `contains("circular")` でチェック
- `compile_parallel(vec![], 1)` は空ソースで `merge_ir_programs([])` を呼び `IRProgram { globals: [], fns: [], type_metas: {} }` を返す
- `use super::*` なし、import は明示

---

## 完了条件

- [ ] `Cargo.toml` version = `"33.9.0"`
- [ ] `cargo_toml_version_is_33_8_0` が空スタブ
- [ ] `cargo test --bin fav v339000` — 4/4 PASS
- [ ] `cargo test` — 全件 PASS（2532 件、0 failures）
- [ ] `CHANGELOG.md` に `[v33.9.0]` セクション
- [ ] `benchmarks/v33.9.0.json` 存在かつ `tests_passed` が実測値
- [ ] `benchmarks/v33.9.0.json` の `milestone` フィールドが `"Performance & Tooling"`
- [ ] `versions/current.md` を v33.9.0 に更新
