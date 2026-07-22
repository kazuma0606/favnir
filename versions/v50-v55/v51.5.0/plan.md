# Plan: v51.5.0 — インクリメンタルコンパイル依存グラフ

Date: 2026-07-19

---

## 実装順序

### Step 1 — `DepGraph` に `Serialize`/`Deserialize` 追加（`incremental/dep_graph.rs`）

`use serde::{Deserialize, Serialize};` を追加し、`DepGraph` の derive に `Serialize, Deserialize` を追加する。

`serde` は `Cargo.toml` の `[dependencies]` にすでに登録済みのため、追加依存は不要。
`HashMap<String, Vec<String>>` は serde がデフォルトで対応している。

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DepGraph {
    edges: HashMap<String, Vec<String>>,
}
```

---

### Step 2 — `DepGraph::transitive_affected_by` 追加（`incremental/dep_graph.rs`）

既存の `affected_by`（直接依存元を返す）の BFS 版として `transitive_affected_by` を追加する。
`transitive_deps`（直接・推移的依存先 BFS）のミラーイメージ実装。

`affected_by` の呼び出し先を `cur` に置き換えながら BFS を展開する。

---

### Step 3 — `save_dep_graph_json` / `load_dep_graph_json` 追加（`incremental/dep_graph.rs`）

`serde_json::to_string(&graph)` で JSON 化し `std::fs::write` で保存。
`load_dep_graph_json` はファイル不在時に `DepGraph::default()` を返す（サイレント）。

`create_dir_all` で親ディレクトリを自動作成する（`.fav-cache/` が存在しない場合）。

---

### Step 4 — `incremental_files_to_rebuild` 追加（`driver.rs`）

v49.3 の `file_needs_recheck` を再利用。`driver.rs` のコメント `// ── v51.5.0 ──` セクションとして
`file_needs_recheck` + `update_fingerprint_cache` の直後（行 15788 付近）に追加する。

**引数の設計注意点**:
- `stems: &[&str]` — ファイルのステム（拡張子なし、dep_graph のキーと一致）
- `paths: &[&std::path::Path]` — ステムに対応する実際のファイルパス（`file_needs_recheck` に渡す）
- 2 スライスのインデックスが一致している前提（zip で処理）

**アルゴリズム**:
1. changed = filter(stems × paths, file_needs_recheck)
2. to_rebuild = changed + transitive_affected_by(changed の各要素)（重複除去）
3. skip = stems - to_rebuild

---

### Step 5 — `v51500_tests` 追加 + バージョン更新（`driver.rs`）

`v51500_tests` モジュールを `v51400_tests` の直前に追加後、`cargo test` で 3123 passed を確認する。

`tempfile::tempdir()` を使い、実際のファイルを作成してフィンガープリントを保存・変更するテストを作成。

テスト内での依存グラフ構築:
```rust
let mut graph = DepGraph::new();
graph.add_dep("a", "b");  // a は b に依存
graph.add_dep("b", "c");  // b は c に依存（`incremental_transitive_invalidation` 用）
```

`update_fingerprint_cache` でフィンガープリントを保存後、
ファイル内容を `std::fs::write` で書き換えて `file_needs_recheck` が `true` になる状態を作る。

`Cargo.toml` version → `"51.5.0"`、`CHANGELOG.md` 更新。

---

## 注意点

- `DepGraph.edges` は `private` フィールドだが `Serialize`/`Deserialize` は `private` フィールドでも動作する
  （serde の `#[derive]` はフィールドのプライバシーに関係なくシリアライズする）
- `load_dep_graph_json` は `serde_json::from_str` 失敗時も `unwrap_or_default()` でパニックしない
- `tempfile` クレートは `[dev-dependencies]` にすでに登録済みのため追加不要
- `incremental_files_to_rebuild` の `stems` と `paths` のインデックス一致を前提とするため、
  テスト内では `zip` で構築した配列を直接渡す（実装内でも `zip` を使う）
