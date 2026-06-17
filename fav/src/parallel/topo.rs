//! v19.4.0: トポロジカル層分割（Kahn's algorithm）

use crate::incremental::dep_graph::DepGraph;
use std::collections::{HashMap, HashSet};

/// ファイルリストと依存グラフを受け取り、並列処理可能な「層」のリストを返す。
///
/// 戻り値の各 `Vec<String>` は同時にコンパイル可能なファイルの集合。
/// 循環依存がある場合は `Err` を返す。
///
/// `a → b → c`（a が b に依存、b が c に依存）の場合:
/// - `in_degree[file]` = そのファイルが依存しているファイルの数
/// - c: 0 → 第 1 層
/// - b: 1 → c 処理後に第 2 層
/// - a: 1 → b 処理後に第 3 層
pub fn topo_layers(
    files: &[String],
    graph: &DepGraph,
) -> Result<Vec<Vec<String>>, String> {
    // in_degree[file] = deps_of(file).len() （このファイルが依存するファイル数）
    let mut in_degree: HashMap<String, usize> = files
        .iter()
        .map(|f| (f.clone(), graph.deps_of(f).len()))
        .collect();

    let mut remaining: HashSet<String> = files.iter().cloned().collect();
    let mut layers: Vec<Vec<String>> = Vec::new();

    while !remaining.is_empty() {
        // in-degree = 0 のファイルを現在の層に収集
        let mut layer: Vec<String> = remaining
            .iter()
            .filter(|f| in_degree.get(*f).copied().unwrap_or(0) == 0)
            .cloned()
            .collect();

        if layer.is_empty() {
            return Err("circular dependency detected".to_string());
        }

        // 決定論的な順序にする
        layer.sort();

        for file in &layer {
            remaining.remove(file);
            // このファイルに依存していた他ファイルの in_degree を減らす
            for dependent in graph.affected_by(file) {
                if let Some(cnt) = in_degree.get_mut(&dependent) {
                    *cnt = cnt.saturating_sub(1);
                }
            }
        }

        layers.push(layer);
    }

    Ok(layers)
}
