//! v19.3.0: ファイル依存グラフの構築・追跡

use std::collections::HashMap;

/// ファイル間の依存グラフ。
/// `edges["pipeline"] = ["utils"]` は pipeline が utils に依存することを表す。
#[derive(Debug, Clone, Default)]
pub struct DepGraph {
    /// from → Vec<to>
    edges: HashMap<String, Vec<String>>,
}

impl DepGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// `from` が `to` に依存することを登録する。
    pub fn add_dep(&mut self, from: &str, to: &str) {
        self.edges
            .entry(from.to_string())
            .or_default()
            .push(to.to_string());
    }

    /// `file` が直接・推移的に依存するファイルをすべて返す（BFS）。
    pub fn transitive_deps(&self, file: &str) -> Vec<String> {
        let mut visited = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(file.to_string());
        while let Some(cur) = queue.pop_front() {
            if let Some(deps) = self.edges.get(&cur) {
                for dep in deps {
                    if !visited.contains(dep) {
                        visited.push(dep.clone());
                        queue.push_back(dep.clone());
                    }
                }
            }
        }
        visited
    }

    /// `file` が直接依存するファイルのリストを返す。
    pub fn deps_of(&self, file: &str) -> Vec<String> {
        self.edges.get(file).cloned().unwrap_or_default()
    }

    /// `changed` ファイルが変更されたとき影響を受けるファイルを返す（逆依存）。
    pub fn affected_by(&self, changed: &str) -> Vec<String> {
        self.edges
            .iter()
            .filter(|(_, deps)| deps.iter().any(|d| d == changed))
            .map(|(from, _)| from.clone())
            .collect()
    }
}

/// AST の `use` 宣言（`Program.uses`）からソースファイルの依存グラフを構築する。
/// - `use utils.{ format_date }` → path = ["utils", "format_date"] → utils に依存
/// - `use json` → path = ["json"]（rune import、スキップ）
pub fn build_dep_graph(program: &crate::ast::Program, source_stem: &str) -> DepGraph {
    let mut graph = DepGraph::new();
    for path in &program.uses {
        // 2 セグメント以上 = ファイル import（"utils.format_date" 形式）
        if path.len() >= 2 {
            graph.add_dep(source_stem, &path[0]);
        }
    }
    graph
}
