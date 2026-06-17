# v19.4.0 Spec — 並列コンパイル

## 概要

ファイル単位でコンパイルを並列化する。
現在はシングルスレッドで全ファイルを順次コンパイルしているため、
大きなプロジェクトでスケールしない。
`rayon` による並列 AST 生成 / IR 生成と、依存グラフに基づいたトポロジカル並列型チェックを実装する。

**テーマ**: Production Performance シリーズ第4弾

---

## 動機

```
現在（シングルスレッド）:
  src/a.fav → parse → check → compile
  src/b.fav → parse → check → compile  （直列）
  src/c.fav → parse → check → compile  （直列）
  合計: 3 ファイル × T 秒

v19.4.0（並列）:
  src/a.fav ──┐
  src/b.fav ──┤ rayon par_iter   （並列）
  src/c.fav ──┘
  合計: T 秒（ほぼ変わらず）
```

---

## 並列化フェーズ

```
フェーズ 1: AST 生成（全ファイル完全並列）
  src/a.fav ──┐
  src/b.fav ──┤ rayon par_iter → [AST_a, AST_b, AST_c, ...]
  src/c.fav ──┘

フェーズ 2: 型チェック（依存グラフのトポロジカルソート後に並列）
  types.fav    → （依存なし → 最初に処理）
  utils.fav    → types.fav に依存 → types.fav 完了後に処理
  pipeline.fav → utils.fav に依存 → utils.fav 完了後に処理
  同一層のファイルは並列処理可能

フェーズ 3: IR 生成（型チェック済みファイルを並列）
  [IR_types, IR_utils, IR_pipeline] = rayon par_iter で並列生成

フェーズ 4: リンク（シングルスレッド、全 IR を結合）
  final_artifact = link([IR_types, IR_utils, IR_pipeline])
```

---

## 期待するスケールアップ

| ファイル数 | シングル | 4コア並列 | 8コア並列 |
|---|---|---|---|
| 1 | 1.0 秒 | 1.0 秒 | 1.0 秒 |
| 5 | 5.0 秒 | 1.5 秒 | 1.2 秒 |
| 10 | 10.0 秒 | 3.0 秒 | 2.0 秒 |
| 20 | 20.0 秒 | 6.0 秒 | 4.0 秒 |
| 50 | 50.0 秒 | 14.0 秒 | 8.0 秒 |

---

## 実装内容

### T1: `fav/Cargo.toml` — 依存追加

```toml
rayon    = "1"
petgraph = "0.6"
```

いずれも native-only セクション（`cfg(not(target_arch = "wasm32"))`）に追加。

### T2: `fav/src/parallel/` — 並列コンパイルモジュール（新規）

#### `topo.rs` — トポロジカルソート

```rust
/// DepGraph（v19.3.0 の incremental::dep_graph::DepGraph）を
/// petgraph の DiGraph に変換し、Kahn's algorithm でトポロジカルソートする。
/// 戻り値: 各「層」（同時に処理できるファイルセット）のリスト
pub fn topo_layers(files: &[String], graph: &DepGraph) -> Vec<Vec<String>>
```

#### `compiler.rs` — 並列コンパイルオーケストレーター

```rust
/// 複数の .fav ソース文字列を並列コンパイルし、IRProgram を返す。
///
/// フェーズ 1: rayon par_iter でソース → AST を並列パース
/// フェーズ 2: トポロジカル順に型チェック（同一層は並列）
/// フェーズ 3: rayon par_iter で AST → IR を並列生成
/// フェーズ 4: IR を結合して単一 IRProgram にマージ
pub fn compile_parallel(sources: Vec<(String, String)>, jobs: usize) -> IRProgram
//    sources: Vec<(file_name, source_code)>
//    jobs:    スレッド数（0 = CPU コア数に自動設定）
```

#### `mod.rs`

```rust
pub mod compiler;
pub mod topo;
```

### T3: `fav/src/driver.rs` — `--jobs` フラグ追加

- `fav build --jobs N` オプション:
  - `N = 0`（または省略）: `rayon::current_num_threads()` に自動設定
  - `N > 0`: `rayon::ThreadPoolBuilder::new().num_threads(N)` で固定

- `cmd_build_parallel(files: &[&str], jobs: usize, out: Option<&str>)` 公開ヘルパー（テスト用）

### T4: `fav/src/lib.rs` + `fav/src/main.rs` — モジュール登録

```rust
pub mod parallel;
```

### T5: `fav/src/driver.rs` — `v194000_tests` 追加

- `v193000_tests::version_is_19_3_0` に `#[ignore]` を追加
- `v194000_tests` モジュール（5件）

### T6: `fav/Cargo.toml` バージョン更新

- `version = "19.3.0"` → `"19.4.0"`

### T7: `site/content/docs/tools/parallel.mdx`（新規）

- 並列コンパイルの使い方
- `--jobs N` オプションの説明
- 依存グラフとトポロジカル順処理の解説

---

## テスト（v194000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_19_4_0` | Cargo.toml に `"19.4.0"` が含まれる |
| `parallel_compile_same_output` | 並列コンパイルの出力が逐次コンパイルと一致する |
| `parallel_compile_faster` | 複数ソース並列コンパイルが実行できる（速度計測は環境依存のため構造テスト） |
| `parallel_dep_order_respected` | 依存順序が正しく守られる（topo_layers が正しい順序を返す） |
| `parallel_compile_thread_count` | `jobs=1` / `jobs=0` でコンパイルが完了する |

---

## 完了条件

- [ ] `rayon` / `petgraph` が Cargo.toml に追加される
- [ ] `src/parallel/topo.rs` — `topo_layers` が実装される
- [ ] `src/parallel/compiler.rs` — `compile_parallel` が実装される
- [ ] 並列コンパイルの出力が逐次コンパイルと一致する
- [ ] `--jobs N` でスレッド数を制御できる
- [ ] `cargo test v194000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
- [ ] `site/content/docs/tools/parallel.mdx` が存在する

---

## 技術ノート

### `rayon` の ThreadPool スコープ

`compile_parallel` 内でグローバルな rayon ThreadPool を変更するとテストが干渉する。
`rayon::ThreadPoolBuilder::new().num_threads(n).build_scoped(...)` でスコープ付き ThreadPool を使用する。

### `petgraph` のトポロジカルソート

```rust
use petgraph::graph::DiGraph;
use petgraph::algo::toposort;
```

`toposort` は循環依存がある場合 `Err(Cycle)` を返す。
循環依存はエラーとして `compile_parallel` から返す。

### `compile_program` との関係

v19.4.0 の `compile_parallel` は複数ソースを受け取る新しいエントリポイント。
既存の `compile_program`（単一 `Program`）は変更しない。
`compile_parallel` 内部で各ソースを `parse_str → compile_program` に通してから IR を結合する。

### IRProgram のマージ

複数の `IRProgram` をマージする際:
- `globals`: 重複を排除しながら連結
- `fns`: インデックスを付け直して連結
- `type_metas`: HashMap を merge

### `topo_layers` の実装

Kahn's algorithm（BFS ベース）:
1. 各ノードの in-degree を計算
2. in-degree = 0 のノードを第 1 層に入れる
3. 第 1 層のノードを処理後、依存先の in-degree を減算
4. in-degree = 0 になったノードを次の層に追加
5. 繰り返し
