# v82.3.0 実装計画

## 方針

**前提**: v82.2.0 完了済み（3,869 tests pass）。

`test_framework.rs` に依存グラフ型・関数を追加し、`driver.rs` に `v82300_tests` を追加する。

---

## 実装ステップ

### Step 1: `ContractDependency` 構造体を追加

`fav/src/test_framework.rs` の v82.2.0 セクション末尾に続けて追加する。

```rust
// ── v82.3.0: ContractDependency / DependencyGraph ────────────────────────────
/// パイプライン間の単一契約依存エッジ。
#[derive(Debug, Clone, PartialEq)]
pub struct ContractDependency {
    pub upstream: String,
    pub downstream: String,
    pub output_contract: String,
}
```

### Step 2: `DependencyGraph` 構造体を追加

```rust
/// 契約間の有向依存グラフ。
#[derive(Debug, Clone, PartialEq)]
pub struct DependencyGraph {
    pub dependencies: Vec<ContractDependency>,
}
```

### Step 3: `build_dependency_graph` を実装

contracts の全ペア (i, j) を走査し、
`contracts[i].output` の field name 群と `contracts[j].input` の field name 群の
共通要素があれば `ContractDependency` エッジを作成する。

```rust
pub fn build_dependency_graph(contracts: &[IoContract]) -> DependencyGraph {
    let mut deps = Vec::new();
    for (i, upstream) in contracts.iter().enumerate() {
        let out_names: std::collections::HashSet<&str> =
            upstream.output.iter().map(|f| f.name.as_str()).collect();
        for (j, downstream) in contracts.iter().enumerate() {
            if i == j { continue; }
            let has_match = downstream.input.iter().any(|f| out_names.contains(f.name.as_str()));
            if has_match {
                deps.push(ContractDependency {
                    upstream: upstream.name.clone(),
                    downstream: downstream.name.clone(),
                    output_contract: upstream.name.clone(),
                });
            }
        }
    }
    DependencyGraph { dependencies: deps }
}
```

### Step 4: `detect_circular_dependencies` を実装

隣接リストを構築して DFS でバックエッジを検出する。
簡易実装として、隣接リスト上で「A の到達可能ノードから A への辺」を探す。

```rust
pub fn detect_circular_dependencies(graph: &DependencyGraph) -> Vec<Vec<String>> {
    use std::collections::{HashMap, HashSet};
    // 隣接リスト構築
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for dep in &graph.dependencies {
        adj.entry(dep.upstream.as_str()).or_default().push(dep.downstream.as_str());
    }
    let mut cycles: Vec<Vec<String>> = Vec::new();
    let nodes: Vec<&str> = adj.keys().copied().collect();
    for start in &nodes {
        // start から到達できる全ノードを BFS で列挙
        let mut visited: HashSet<&str> = HashSet::new();
        let mut stack: Vec<&str> = vec![start];
        while let Some(node) = stack.pop() {
            if visited.contains(node) { continue; }
            visited.insert(node);
            if let Some(neighbors) = adj.get(node) {
                for &nb in neighbors {
                    stack.push(nb);
                }
            }
        }
        // start 自身に戻る辺がある = サイクル
        if visited.contains(start) && adj.get(start).map_or(false, |nb| !nb.is_empty()) {
            // start の隣接ノードの中に start から到達できて start にも辺を張れるものを探す
            if let Some(neighbors) = adj.get(start) {
                for &nb in neighbors {
                    if let Some(nb_neighbors) = adj.get(nb) {
                        if nb_neighbors.contains(start) {
                            let cycle = vec![start.to_string(), nb.to_string()];
                            if !cycles.contains(&cycle) {
                                cycles.push(cycle);
                            }
                        }
                    }
                }
            }
        }
    }
    cycles
}
```

> 注: 本実装は 2 ノードサイクル（A→B→A）の検出に特化。長いサイクルは将来拡張。

### Step 5: `format_dependency_graph` を実装

```rust
pub fn format_dependency_graph(graph: &DependencyGraph) -> String {
    graph.dependencies
        .iter()
        .map(|d| format!("{} -> {} ({})", d.upstream, d.downstream, d.output_contract))
        .collect::<Vec<_>>()
        .join("\n")
}
```

### Step 6: CHANGELOG 更新

`CHANGELOG.md` の先頭に v82.3.0 エントリを追加する。

### Step 7: `v82300_tests` テストモジュール追加（driver.rs）

`fav/src/driver.rs` 末尾に `#[cfg(test)] mod v82300_tests` を追加する。

- `dependency_graph_built_from_contracts`: output/input フィールド名の一致でエッジが生成されることを確認
- `circular_dependency_detected`: A→B, B→A のグラフでサイクルが検出されることを確認

### Step 8: `cargo test` 全通過確認

3,871 tests pass（+2）、0 failures であることを確認する。
