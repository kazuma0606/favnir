# v76.5.0 実装計画 — `fav lineage graph` 可視化

Date: 2026-08-15

---

## Step 1: driver.rs — LineageNodeType enum 追加

`fav/src/driver.rs` の末尾に `// --- v76.5.0: fav lineage graph 可視化 ---` コメントと `LineageNodeType` enum を追加する。

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum LineageNodeType {
    Source,
    Transform,
    Sink,
}
```

---

## Step 2: driver.rs — LineageNode / LineageEdge / LineageGraph 追加

```rust
#[derive(Debug, Clone)]
pub struct LineageNode {
    pub id:        String,
    pub node_type: LineageNodeType,
    pub label:     String,
}

#[derive(Debug, Clone)]
pub struct LineageEdge {
    pub from: String,
    pub to:   String,
}

#[derive(Debug, Clone)]
pub struct LineageGraph {
    pub nodes: Vec<LineageNode>,
    pub edges: Vec<LineageEdge>,
}
```

---

## Step 3: driver.rs — format_lineage_dot 追加

```rust
pub fn format_lineage_dot(graph: &LineageGraph) -> String {
    let mut out = String::from("digraph lineage {\n");
    for edge in &graph.edges {
        out.push_str(&format!("    \"{}\" -> \"{}\"\n", edge.from, edge.to));
    }
    out.push('}');
    out
}
```

---

## Step 4: cargo test（既存テスト通過確認）

`cargo test` を実行し、既存の 3722 テストが引き続き pass することを確認する（新規テストモジュールはまだ追加しない）。

---

## Step 5: CHANGELOG.md 更新

`CHANGELOG.md` の先頭に v76.5.0 エントリを追加する（テストモジュール追加より先）。

---

## Step 6: driver.rs — v765000_tests モジュール追加

```rust
#[cfg(test)]
mod v765000_tests {
    use super::*;  // LineageNodeType, LineageNode, LineageEdge, LineageGraph, format_lineage_dot を参照するため必須

    #[test]
    fn lineage_graph_built() { ... }

    #[test]
    fn lineage_dot_format() { ... }
}
```

---

## Step 7: Cargo.toml バージョン更新

`76.4.0` → `76.5.0`

---

## Step 8: versions/current.md 更新

進行中バージョンを v76.5.0 に、次に切る版を v76.6.0 に更新する。

---

## Step 9: 最終確認

`cargo test` が 3724 tests all pass であることを確認する。
