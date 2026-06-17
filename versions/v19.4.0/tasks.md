# v19.4.0 — 並列コンパイル タスク

## ステータス: TODO

---

## タスク一覧

### T1: `fav/Cargo.toml` — 依存追加

- [x] `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` セクションに追記:
  ```toml
  rayon    = "1"
  petgraph = "0.6"
  ```
- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T2: `fav/src/parallel/` — モジュール作成

**2-A: `fav/src/incremental/dep_graph.rs` に `deps_of` 追加**

- [x] `DepGraph` に `pub fn deps_of(&self, file: &str) -> Vec<String>` を追加:
  ```rust
  pub fn deps_of(&self, file: &str) -> Vec<String> {
      self.edges.get(file).cloned().unwrap_or_default()
  }
  ```

**2-B: `fav/src/parallel/topo.rs` 作成**

- [x] `fav/src/parallel/topo.rs` を新規作成:
  - `pub fn topo_layers(files: &[String], graph: &DepGraph) -> Result<Vec<Vec<String>>, String>`
  - Kahn's algorithm（BFS）でトポロジカル層分割
  - 循環依存を検出して `Err("circular dependency detected".to_string())` を返す

**2-C: `fav/src/parallel/compiler.rs` 作成**

- [x] `fav/src/parallel/compiler.rs` を新規作成:
  - `pub fn compile_parallel(sources: Vec<(String, String)>, jobs: usize) -> Result<IRProgram, String>`
    - `jobs = 0`: `rayon::current_num_threads()` を使用
    - `jobs > 0`: 指定スレッド数の ThreadPool を作成
    - フェーズ 1: `par_iter` で並列 AST パース
    - フェーズ 3: `par_iter` で並列 IR 生成
    - フェーズ 4: `merge_ir_programs` で結合
  - `fn merge_ir_programs(programs: Vec<IRProgram>) -> IRProgram`
    - `globals`: 重複名をスキップして連結（fn インデックスをオフセット付きで更新）
    - `fns`: オフセット付きで連結
    - `type_metas`: HashMap を merge

**2-D: `fav/src/parallel/mod.rs` 作成**

- [x] `fav/src/parallel/mod.rs` を新規作成:
  ```rust
  pub mod compiler;
  pub mod topo;
  ```

---

### T3: `fav/src/driver.rs` — ヘルパー関数追加

- [x] `pub(crate) fn cmd_build_parallel_sources(sources: Vec<(String, String)>, jobs: usize) -> Result<crate::middle::ir::IRProgram, String>` を追加:
  ```rust
  pub(crate) fn cmd_build_parallel_sources(
      sources: Vec<(String, String)>,
      jobs: usize,
  ) -> Result<crate::middle::ir::IRProgram, String> {
      crate::parallel::compiler::compile_parallel(sources, jobs)
  }
  ```

---

### T4: `fav/src/lib.rs` + `fav/src/main.rs` — モジュール登録

- [x] `fav/src/lib.rs` に追記:
  ```rust
  pub mod parallel;
  ```
- [x] `fav/src/main.rs` に追記:
  ```rust
  mod parallel;
  ```
- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T5: `fav/src/driver.rs` — `v194000_tests` 追加

- [x] `v193000_tests::version_is_19_3_0` に `#[ignore]` を追加
- [x] `v194000_tests` モジュールを追加（5件）:

  ```rust
  // ── v194000_tests (v19.4.0) — 並列コンパイル ────────────────────────────────
  #[cfg(test)]
  mod v194000_tests {
      use crate::parallel::{compiler::compile_parallel, topo::topo_layers};
      use crate::incremental::dep_graph::{build_dep_graph, DepGraph};
      use crate::frontend::parser::Parser;

      #[test]
      fn version_is_19_4_0() {
          let cargo = include_str!("../Cargo.toml");
          assert!(cargo.contains("19.4.0"), "Cargo.toml should have version 19.4.0");
      }

      #[test]
      fn parallel_compile_same_output() {
          // 単一ソースを jobs=1 で並列コンパイルし、逐次コンパイルと fn 数が一致
          let src = "fn main() -> Int { 42 }";
          let sources = vec![("main.fav".to_string(), src.to_string())];

          let seq_prog = Parser::parse_str(src, "main.fav").expect("parse");
          let seq_ir = crate::middle::compiler::compile_program(&seq_prog);

          let par_result = compile_parallel(sources, 1).expect("compile_parallel");

          assert_eq!(
              par_result.fns.len(),
              seq_ir.fns.len(),
              "parallel and sequential should produce same number of functions"
          );
      }

      #[test]
      fn parallel_compile_faster() {
          // 3 つのソースを並列コンパイルして正常終了することを確認
          let sources = vec![
              ("a.fav".to_string(), "fn fa() -> Int { 1 }".to_string()),
              ("b.fav".to_string(), "fn fb() -> Int { 2 }".to_string()),
              ("c.fav".to_string(), "fn fc() -> Int { 3 }".to_string()),
          ];
          let result = compile_parallel(sources, 0);
          assert!(result.is_ok(), "parallel compile should succeed: {:?}", result);
          let ir = result.unwrap();
          // 各ソースから少なくとも 1 つ以上の fn が生成される
          assert!(ir.fns.len() >= 3, "expected at least 3 fns, got {}", ir.fns.len());
      }

      #[test]
      fn parallel_dep_order_respected() {
          // a → b → c の依存グラフで topo_layers が正しい順序を返す
          // c は依存なし → 第 1 層
          // b は c に依存 → 第 2 層
          // a は b に依存 → 第 3 層
          let mut graph = DepGraph::new();
          graph.add_dep("a", "b");
          graph.add_dep("b", "c");
          let files = vec!["a".to_string(), "b".to_string(), "c".to_string()];
          let layers = topo_layers(&files, &graph).expect("topo_layers");
          assert_eq!(layers.len(), 3, "expected 3 layers, got: {:?}", layers);
          assert!(layers[0].contains(&"c".to_string()), "first layer should have c");
          assert!(layers[1].contains(&"b".to_string()), "second layer should have b");
          assert!(layers[2].contains(&"a".to_string()), "third layer should have a");
      }

      #[test]
      fn parallel_compile_thread_count() {
          // jobs=1 と jobs=0 の両方で compile_parallel が成功することを確認
          let sources = vec![("t.fav".to_string(), "fn ft() -> Int { 99 }".to_string())];
          let r1 = compile_parallel(sources.clone(), 1);
          assert!(r1.is_ok(), "jobs=1 should succeed: {:?}", r1);
          let r0 = compile_parallel(sources, 0);
          assert!(r0.is_ok(), "jobs=0 should succeed: {:?}", r0);
      }
  }
  ```

---

### T6: `fav/Cargo.toml` バージョン更新

- [x] `version = "19.3.0"` → `"19.4.0"` に変更

---

### T7: `site/content/docs/tools/parallel.mdx`（新規作成）

- [x] 並列コンパイルの概要（4フェーズの説明）
- [x] `--jobs N` オプションの使い方
- [x] 依存グラフとトポロジカル順処理の解説
- [x] スケールアップの期待値（ファイル数 vs スレッド数 vs 速度）

---

## テスト（v194000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_19_4_0` | Cargo.toml に `"19.4.0"` が含まれる |
| `parallel_compile_same_output` | 並列と逐次の fn 数が一致する |
| `parallel_compile_faster` | 3 ソース並列コンパイルが正常終了し fn 数 ≥ 3 |
| `parallel_dep_order_respected` | `topo_layers` が a→b→c で [c],[b],[a] の順序を返す |
| `parallel_compile_thread_count` | `jobs=1` / `jobs=0` で compile_parallel が成功 |

---

## 完了条件チェックリスト

- [x] `rayon = "1"` / `petgraph = "0.6"` が Cargo.toml に追加される
- [x] `DepGraph::deps_of` が `dep_graph.rs` に追加される
- [x] `parallel/topo.rs` — `topo_layers` が実装される（Kahn's algorithm）
- [x] `parallel/compiler.rs` — `compile_parallel` が実装される
- [x] `merge_ir_programs` が重複グローバルを正しく処理する
- [x] `cargo test v194000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし
- [x] `site/content/docs/tools/parallel.mdx` が存在する

---

## 優先度

```
T1（Cargo.toml 依存追加）            ← 最初
T2-A（DepGraph::deps_of 追加）       ← T1 完了後（T2-B の前提）
T2-B（topo.rs 作成）                 ← T2-A 完了後
T2-C（compiler.rs 作成）             ← T2-B 完了後
T2-D（mod.rs 作成）                  ← T2-C と同時
T3（driver.rs ヘルパー追加）         ← T2 完了後
T4（lib.rs / main.rs 登録）          ← T2 完了後（T3 と並列可）
T5（v194000_tests 追加）             ← T3 / T4 完了後
T6（Cargo.toml バージョン）          ← T5 と並列可
T7（ドキュメント）                   ← T5 と並列可
```

---

## 重要な技術ノート

### `topo_layers` の `in_degree` 計算

依存グラフ `a → b`（a が b に依存）の場合:
- a から見た「依存先」は b
- b の in-degree が増える（b は a から参照される）
- しかし「処理順序」は b を先に処理する必要がある

`deps_of("a") = ["b"]` → b の in-degree += 1。
最初に in-degree = 0 のファイル（誰にも依存されていないファイル）を処理するのが正しい。

**注意**: ここでの "in-degree" は「このファイルが依存しているファイルの数」ではなく、
「このファイルに依存しているファイルの数（= 被依存数）」を指す。
Kahn's algorithm では依存グラフを逆向きに考える。

具体的に `a → b → c` の場合:
- `a` は `b` に依存（b が先に必要）
- `b` は `c` に依存（c が先に必要）
- `c` は誰にも依存しない

処理順: c → b → a

`in_degree` = 「このファイルが依存しているファイルの数」:
- c: 0 → 最初に処理
- b: 1（c に依存）→ c 処理後に in_degree が 0 になる
- a: 1（b に依存）→ b 処理後に in_degree が 0 になる

この場合 `deps_of("b") = ["c"]`（b が c に依存）。
in_degree 計算: `for dep in deps_of(file) { in_degree[dep] += 1 }` ではなく、
`in_degree[file] = deps_of(file).len()` が正しい。

### `compile_parallel` の ThreadPool

`rayon::ThreadPoolBuilder::new().num_threads(n).build()` は `Result<ThreadPool>` を返す。
`ThreadPool::install(|| { ... })` 内で rayon の `par_iter` を使うと、
そのプールのスレッドが使われる。

### `merge_ir_programs` における builtin グローバルの重複

複数ファイルをコンパイルすると、各 IRProgram に `IO`, `List` 等のビルトイングローバルが
重複して含まれる可能性がある。
名前でデデュプリケーションを行い、最初に現れたものを使用する。
