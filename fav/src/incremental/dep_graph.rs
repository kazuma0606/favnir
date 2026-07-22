//! v19.3.0: ファイル依存グラフの構築・追跡

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// ファイル間の依存グラフ。
/// `edges["pipeline"] = ["utils"]` は pipeline が utils に依存することを表す。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

    /// `changed` が変更されたとき推移的に影響を受けるすべてのファイルを BFS で返す（v51.5.0）。
    /// 例: a→b→c（a は b に依存、b は c に依存）の場合、
    /// c の変更に対して [b, a] が返る。
    ///
    /// NOTE: 戻り値に `changed` 自身は含まれない。
    /// 呼び出し側（`incremental_files_to_rebuild`）が `changed` を別途 `to_rebuild` に追加して補完する。
    pub fn transitive_affected_by(&self, changed: &str) -> Vec<String> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut result: Vec<String> = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(changed.to_string());
        while let Some(cur) = queue.pop_front() {
            for from in self.affected_by(&cur) {
                if seen.insert(from.clone()) {
                    result.push(from.clone());
                    queue.push_back(from);
                }
            }
        }
        result
    }
}

/// DepGraph を JSON ファイルに保存する（`.fav-cache/dep-graph.json` 等）（v51.5.0）。
/// NOTE: `std::fs` を使用するため WASM ターゲットでは動作しない。
#[cfg(not(target_arch = "wasm32"))]
pub fn save_dep_graph_json(graph: &DepGraph, path: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create_dir_all error: {e}"))?;
    }
    let json = serde_json::to_string(graph)
        .map_err(|e| format!("dep_graph serialize error: {e}"))?;
    std::fs::write(path, json).map_err(|e| format!("dep_graph write error: {e}"))
}

/// JSON ファイルから DepGraph を読み込む（v51.5.0）。
/// ファイルが存在しない場合・JSON パースに失敗した場合も空の DepGraph を返す（サイレント）。
/// NOTE: パース失敗（スキーマ変更・ファイル破損）も黙殺されるため、毎回フルリビルドが発生する。
/// NOTE: `std::fs` を使用するため WASM ターゲットでは動作しない。
#[cfg(not(target_arch = "wasm32"))]
pub fn load_dep_graph_json(path: &std::path::Path) -> DepGraph {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
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
