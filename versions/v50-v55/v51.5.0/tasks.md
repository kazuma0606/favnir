# Tasks: v51.5.0 — インクリメンタルコンパイル依存グラフ

Status: COMPLETE
Date: 2026-07-19

---

## T0 — 事前確認

- [x] `cargo test` 3121 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `incremental/dep_graph.rs` の `DepGraph` derive に `Serialize` / `Deserialize` が**存在しない**ことを確認
- [x] `incremental/dep_graph.rs` に `transitive_affected_by` が**存在しない**ことを確認（新規追加対象）
- [x] `incremental/dep_graph.rs` に `save_dep_graph_json` / `load_dep_graph_json` が**存在しない**ことを確認
- [x] `driver.rs` に `incremental_files_to_rebuild` が**存在しない**ことを確認
- [x] `serde` が `Cargo.toml` の `[dependencies]` に登録済みであることを確認（追加不要）
- [x] `serde_json` が `Cargo.toml` の `[dependencies]` に登録済みであることを確認（`save_dep_graph_json` / `load_dep_graph_json` で使用）
- [x] `tempfile` が `[dev-dependencies]` に登録済みであることを確認（テストで使用）
- [x] `DepGraph.edges` が `private` フィールドであることを確認（serde derive はプライバシー非依存のため問題なし）
- [x] `file_needs_recheck` が `driver.rs` 行 15765 付近に存在することを確認（`incremental_files_to_rebuild` から呼び出す）

## T1 — `DepGraph` に `Serialize`/`Deserialize` 追加（`incremental/dep_graph.rs`）

- [x] `use serde::{Deserialize, Serialize};` をファイル先頭に追加
- [x] `#[derive(Debug, Clone, Default, Serialize, Deserialize)]` に更新
- [x] `cargo build` が通ることを確認

## T2 — `DepGraph::transitive_affected_by` 追加（`incremental/dep_graph.rs`）

- [x] `affected_by` の直後に `transitive_affected_by` を追加
  - [x] BFS: `queue` に `changed` を積む
  - [x] `queue.pop_front()` → `self.affected_by(&cur)` の結果を `visited` に追加し `queue` に積む
  - [x] `visited.contains` で重複防止
  - [x] `visited` を返す（起点の `changed` 自身は含まない）
- [x] `cargo build` が通ることを確認

## T3 — `save_dep_graph_json` / `load_dep_graph_json` 追加（`incremental/dep_graph.rs`）

- [x] `transitive_affected_by` の後に 2 関数を追加
  - [x] `pub fn save_dep_graph_json(graph: &DepGraph, path: &std::path::Path) -> Result<(), String>`:
    - [x] `path.parent()` があれば `create_dir_all`（エラーは `map_err` で `Err(String)` に変換）
    - [x] `serde_json::to_string(graph)` で JSON 化
    - [x] `std::fs::write(path, json)` で保存
  - [x] `pub fn load_dep_graph_json(path: &std::path::Path) -> DepGraph`:
    - [x] `std::fs::read_to_string(path).ok()` でファイル読み込み（不在時は `None`）
    - [x] `serde_json::from_str(&s).ok()` でデシリアライズ（失敗時は `None`）
    - [x] `.unwrap_or_default()` で空の `DepGraph` を返す
- [x] `cargo build` が通ることを確認

## T4 — `incremental_files_to_rebuild` 追加（`driver.rs`）

- [x] `update_fingerprint_cache` の後（行 15788 付近）に `// ── v51.5.0: インクリメンタルコンパイル依存グラフ ──` セクションを追加
- [x] `pub fn incremental_files_to_rebuild` を追加:
  - [x] シグネチャ: `pub fn incremental_files_to_rebuild(stems: &[&str], paths: &[&std::path::Path], graph: &crate::incremental::dep_graph::DepGraph, cache_dir: &std::path::Path) -> (Vec<String>, Vec<String>)`
  - [x] `changed`: `stems.iter().zip(paths.iter()).filter(file_needs_recheck).map(stem.to_string()).collect()`
  - [x] `to_rebuild`: changed + 各 changed に対して `graph.transitive_affected_by(ch)` を追加（重複除去）
  - [x] `skip`: stems から to_rebuild を除いたもの
  - [x] `(to_rebuild, skip)` を返す
- [x] `cargo build` が通ることを確認（`file_needs_recheck` は同ファイルにあるため import 不要）

## T5 — `v51500_tests` 追加 + バージョン更新

- [x] `v51500_tests` モジュールを `v51400_tests` の直前に追加（2 件）:
  - [x] テストの `use`:
    - [x] `use super::{incremental_files_to_rebuild, update_fingerprint_cache};`
    - [x] `use crate::incremental::dep_graph::DepGraph;`
    - [x] `use tempfile::tempdir;`
  - [x] `incremental_dep_graph_rebuilt`:
    - [x] `tempdir()` で一時ディレクトリを作成
    - [x] `a.fav`（"fn a() -> Int { 1 }"）と `b.fav`（"fn b() -> Int { 2 }"）を作成
    - [x] `graph.add_dep("a", "b")` で a は b に依存
    - [x] `update_fingerprint_cache(a_path, cache_dir)` / `update_fingerprint_cache(b_path, cache_dir)` で初期フィンガープリントを保存
    - [x] `std::fs::write(b_path, "fn b() -> Int { 99 }")` で b を変更
    - [x] `incremental_files_to_rebuild(&["a", "b"], &[a_path, b_path], &graph, cache_dir)` を呼ぶ
    - [x] `to_rebuild.contains("b")` を assert（直接変更）
    - [x] `to_rebuild.contains("a")` を assert（b に依存するため）
  - [x] `incremental_transitive_invalidation`:
    - [x] `tempdir()` で一時ディレクトリを作成
    - [x] `a.fav`, `b.fav`, `c.fav` を作成
    - [x] `graph.add_dep("a", "b")`, `graph.add_dep("b", "c")` で a→b→c チェーン
    - [x] 3 ファイルのフィンガープリントを保存
    - [x] `c.fav` の内容を変更
    - [x] `incremental_files_to_rebuild` を呼ぶ
    - [x] `to_rebuild.contains("c")` を assert（直接変更）
    - [x] `to_rebuild.contains("b")` を assert（c に依存）
    - [x] `to_rebuild.contains("a")` を assert（b に依存 → 推移的）
- [x] `fav/Cargo.toml` version → `"51.5.0"`
- [x] `cargo test` 3123 passed, 0 failed（code-review 後: 3124 passed — `dep_graph_json_roundtrip` 追加）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v51.5.0 エントリ追加
- [x] `versions/current.md` を v51.5.0（3124 tests）に更新
- [x] `roadmap-v51.1-v52.0.md` の v51.5.0 実績欄を更新
- [x] tasks.md を COMPLETE に更新（T0〜T5 全 `[x]`）
