# Spec: v51.5.0 — インクリメンタルコンパイル依存グラフ

Date: 2026-07-19
Status: 計画中

---

## 概要

ファイル間の import 依存グラフを `.fav-cache/dep-graph.json` に永続化し、
変更ファイルとその推移的依存元のみを再コンパイル対象とする。

```bash
$ fav build
[skip]    b.fav — unchanged
[skip]    c.fav — unchanged
[rebuild] a.fav — changed
[rebuild] d.fav — depends on a.fav
```

v49.3 のインクリメンタル型チェック（`file_needs_recheck`）をコンパイルフェーズにも拡張する。

---

## 前提確認

### 既存実装（変更しない）

| ファイル | 内容 |
|---|---|
| `incremental/dep_graph.rs` | `DepGraph` — `add_dep`, `deps_of`, `affected_by`, `transitive_deps`, `build_dep_graph` |
| `incremental/cache.rs` | `IncrementalCache` — artifact 単位キャッシュ |
| `incremental/fingerprint.rs` | `file_hash`, `content_hash` |
| `driver.rs` (v49.3) | `compute_file_fingerprint`, `file_needs_recheck`, `update_fingerprint_cache` |

### 不足している機能

1. `DepGraph` の JSON シリアライズ / デシリアライズ（`#[derive(Serialize, Deserialize)]`）
2. `save_dep_graph_json` / `load_dep_graph_json`（dep_graph.rs に追加）
3. `DepGraph::transitive_affected_by` — 変更ファイルの推移的な依存元を返す（逆依存 BFS）
4. `incremental_files_to_rebuild` — 変更 + 推移的依存元を rebuild 候補として返す（driver.rs）

---

## 実装詳細

### 1. `DepGraph` に `Serialize`/`Deserialize` 追加（`incremental/dep_graph.rs`）

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DepGraph {
    edges: HashMap<String, Vec<String>>,  // from → Vec<to>（変更なし）
}
```

### 2. `DepGraph::transitive_affected_by` 追加

```rust
/// `changed` が変更されたとき推移的に影響を受けるすべてのファイルを BFS で返す。
/// 例: a→b→c（a は b に依存、b は c に依存）の場合、
/// c の変更に対して [b, a] が返る。
///
/// NOTE: 戻り値に `changed` 自身は含まれない。
/// 呼び出し側（`incremental_files_to_rebuild`）が `changed` を別途 `to_rebuild` に追加して補完する。
pub fn transitive_affected_by(&self, changed: &str) -> Vec<String> {
    let mut visited = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(changed.to_string());
    while let Some(cur) = queue.pop_front() {
        for from in self.affected_by(&cur) {
            if !visited.contains(&from) {
                visited.push(from.clone());
                queue.push_back(from);
            }
        }
    }
    visited
}
```

### 3. `save_dep_graph_json` / `load_dep_graph_json` 追加（`incremental/dep_graph.rs`）

```rust
/// DepGraph を JSON ファイルに保存する（.fav-cache/dep-graph.json）。
pub fn save_dep_graph_json(graph: &DepGraph, path: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create_dir_all error: {e}"))?;
    }
    let json = serde_json::to_string(graph)
        .map_err(|e| format!("dep_graph serialize error: {e}"))?;
    std::fs::write(path, json).map_err(|e| format!("dep_graph write error: {e}"))
}

/// JSON ファイルから DepGraph を読み込む。
/// ファイルが存在しない場合・JSON パースに失敗した場合も空の DepGraph を返す（サイレント）。
/// NOTE: パース失敗（スキーマ変更・ファイル破損）も黙殺されるため、毎回フルリビルドが発生する。
/// デバッグログの追加は将来バージョンのスコープ。
/// NOTE: `std::fs` を使用するため WASM ターゲットでは動作しない（`save_dep_graph_json` も同様）。
pub fn load_dep_graph_json(path: &std::path::Path) -> DepGraph {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}
```

### 4. `incremental_files_to_rebuild` 追加（`driver.rs`）

```rust
/// プロジェクト内の複数ファイルのうち、再ビルドが必要なファイルのステムリストを返す（v51.5.0）。
///
/// アルゴリズム:
/// 1. `file_needs_recheck` で変更ファイルを検出
/// 2. 変更ファイルを起点に `transitive_affected_by` で推移的依存元を列挙
/// 3. (rebuild_list, skip_list) を返す
///
/// # 引数
/// - `stems`: プロジェクト内の全ファイルステム（拡張子なし）
/// - `paths`: ステムに対応するファイルパス（インデックス一致）
/// - `graph`: 依存グラフ
/// - `cache_dir`: フィンガープリントキャッシュディレクトリ
pub fn incremental_files_to_rebuild(
    stems: &[&str],
    paths: &[&std::path::Path],
    graph: &crate::incremental::dep_graph::DepGraph,
    cache_dir: &std::path::Path,
) -> (Vec<String>, Vec<String>) {
    // stems と paths のインデックスが一致していることを前提とする。
    // デバッグビルドで長さ不一致を早期検出する。
    debug_assert_eq!(stems.len(), paths.len(), "stems and paths must have equal length");

    // 1. 直接変更されたファイルを検出
    let changed: Vec<String> = stems
        .iter()
        .zip(paths.iter())
        .filter(|(_, path)| file_needs_recheck(path, cache_dir))
        .map(|(stem, _)| stem.to_string())
        .collect();

    // 2. 推移的依存元を追加
    let mut to_rebuild: Vec<String> = changed.clone();
    for ch in &changed {
        for affected in graph.transitive_affected_by(ch) {
            if !to_rebuild.contains(&affected) {
                to_rebuild.push(affected);
            }
        }
    }

    // 3. skip = 全ファイル - to_rebuild
    let skip: Vec<String> = stems
        .iter()
        .map(|s| s.to_string())
        .filter(|s| !to_rebuild.contains(s))
        .collect();

    (to_rebuild, skip)
}
```

---

## テスト設計（`v51500_tests`, `driver.rs`）

### `incremental_dep_graph_rebuilt`

```
1. tempdir を作成
2. a.fav と b.fav を作成
3. 依存グラフ: a は b に依存
4. cache_dir にフィンガープリントを保存（update_fingerprint_cache）
5. b.fav の内容を変更（file_needs_recheck が true になる）
6. incremental_files_to_rebuild を呼ぶ
7. b が rebuild リストに含まれることを assert
8. a が rebuild リストに含まれることを assert（b に依存するため）
9. （別ファイルがあれば skip リストに含まれることを assert）
```

### `incremental_transitive_invalidation`

```
1. tempdir を作成
2. a.fav, b.fav, c.fav を作成（a→b→c: a は b に依存、b は c に依存）
3. cache_dir にフィンガープリントを保存
4. c.fav の内容を変更
5. incremental_files_to_rebuild を呼ぶ
6. c が rebuild リストに含まれることを assert（直接変更）
7. b が rebuild リストに含まれることを assert（c に依存）
8. a が rebuild リストに含まれることを assert（b に依存 → 推移的）
```

---

## 完了条件

- `cargo test` 3123 passed, 0 failed
- `cargo clippy -- -D warnings` クリーン
- `v51500_tests` 2 件 pass: `incremental_dep_graph_rebuilt`, `incremental_transitive_invalidation`
- `DepGraph` が JSON シリアライズ可能
- `save_dep_graph_json` / `load_dep_graph_json` が実装済み
- `DepGraph::transitive_affected_by` が実装済み
- `incremental_files_to_rebuild` が実装済み
