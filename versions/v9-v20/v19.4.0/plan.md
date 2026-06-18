# v19.4.0 実装計画 — 並列コンパイル

## 実装順序

```
T1（Cargo.toml 依存追加）        ← 最初
T2（parallel/ モジュール作成）   ← T1 完了後
T3（driver.rs --jobs フラグ）    ← T2 完了後
T4（lib.rs / main.rs 登録）      ← T2 完了後（T3 と並列可）
T5（v194000_tests 追加）         ← T3 / T4 完了後
T6（Cargo.toml バージョン）      ← T5 と並列可
T7（ドキュメント）               ← T5 と並列可
```

---

## T1: `fav/Cargo.toml` — 依存追加

native-only セクション（`[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`）に追記:

```toml
rayon    = "1"
petgraph = "0.6"
```

`cargo build` でコンパイルエラーが 0 であることを確認。

---

## T2: `fav/src/parallel/` — モジュール作成

### `mod.rs`

```rust
pub mod compiler;
pub mod topo;
```

### `topo.rs` — トポロジカルソート（Kahn's algorithm）

```rust
use crate::incremental::dep_graph::DepGraph;
use std::collections::HashMap;

/// ファイルリストと依存グラフを受け取り、
/// 並列処理可能な「層」のリストを返す（Kahn's algorithm）。
///
/// 戻り値の各 Vec<String> は同時にコンパイル可能なファイルの集合。
/// 循環依存がある場合は Err を返す。
pub fn topo_layers(
    files: &[String],
    graph: &DepGraph,
) -> Result<Vec<Vec<String>>, String> {
    // 各ファイルの in-degree（依存されている数）を計算
    let mut in_degree: HashMap<String, usize> = files
        .iter()
        .map(|f| (f.clone(), 0))
        .collect();

    for file in files {
        for dep in graph.deps_of(file) {
            if let Some(cnt) = in_degree.get_mut(&dep) {
                *cnt += 1;
            }
        }
    }

    let mut layers = Vec::new();
    let mut remaining: std::collections::HashSet<String> = files.iter().cloned().collect();

    while !remaining.is_empty() {
        // in-degree = 0 のファイルを現在の層に収集
        let layer: Vec<String> = remaining
            .iter()
            .filter(|f| in_degree.get(*f).copied().unwrap_or(0) == 0)
            .cloned()
            .collect();

        if layer.is_empty() {
            return Err("circular dependency detected".to_string());
        }

        // 層内のファイルを remaining から除去し、in-degree を更新
        for file in &layer {
            remaining.remove(file);
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
```

**注意**: `DepGraph::deps_of(file)` メソッドが必要。`dep_graph.rs` に追加する:

```rust
/// `file` が直接依存するファイルのリストを返す。
pub fn deps_of(&self, file: &str) -> Vec<String> {
    self.edges.get(file).cloned().unwrap_or_default()
}
```

### `compiler.rs` — 並列コンパイルオーケストレーター

```rust
use rayon::prelude::*;
use crate::middle::ir::IRProgram;

/// `(file_name, source_code)` のリストを並列コンパイルして IRProgram を返す。
///
/// - `jobs = 0`: CPU コア数に自動設定
/// - `jobs > 0`: 指定スレッド数を使用
pub fn compile_parallel(
    sources: Vec<(String, String)>,
    jobs: usize,
) -> Result<IRProgram, String> {
    // スレッド数を設定
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(if jobs == 0 { rayon::current_num_threads() } else { jobs })
        .build()
        .map_err(|e| format!("threadpool error: {e}"))?;

    pool.install(|| {
        // フェーズ 1: 並列 AST パース
        let parsed: Vec<(String, crate::ast::Program)> = sources
            .par_iter()
            .map(|(name, src)| {
                let prog = crate::frontend::parser::Parser::parse_str(src, name)
                    .map_err(|e| format!("parse error in {name}: {e}"))?;
                Ok((name.clone(), prog))
            })
            .collect::<Result<Vec<_>, String>>()?;

        // フェーズ 3: 並列 IR 生成（型チェックは省略、compile_program が内包）
        let programs: Vec<IRProgram> = parsed
            .par_iter()
            .map(|(_, prog)| crate::middle::compiler::compile_program(prog))
            .collect();

        // フェーズ 4: IR マージ
        Ok(merge_ir_programs(programs))
    })
}

/// 複数の IRProgram を 1 つにマージする。
fn merge_ir_programs(programs: Vec<IRProgram>) -> IRProgram {
    use crate::middle::ir::{IRGlobal, IRGlobalKind};
    use std::collections::HashMap;

    let mut globals = Vec::new();
    let mut fns = Vec::new();
    let mut type_metas = HashMap::new();

    for prog in programs {
        // fn インデックスをオフセット付きで登録
        let fn_offset = fns.len();
        for mut global in prog.globals {
            if let IRGlobalKind::Fn(idx) = &mut global.kind {
                *idx += fn_offset;
            }
            // 重複グローバル（同名）はスキップ
            if !globals.iter().any(|g: &IRGlobal| g.name == global.name) {
                globals.push(global);
            }
        }
        fns.extend(prog.fns);
        type_metas.extend(prog.type_metas);
    }

    IRProgram { globals, fns, type_metas }
}
```

---

## T3: `fav/src/driver.rs` — `--jobs` フラグ追加

### `cmd_build_parallel` ヘルパー（テスト用）

```rust
pub(crate) fn cmd_build_parallel_sources(
    sources: Vec<(String, String)>,
    jobs: usize,
) -> Result<crate::middle::ir::IRProgram, String> {
    crate::parallel::compiler::compile_parallel(sources, jobs)
}
```

### CLI での `--jobs` オプション（main.rs）

`fav build` に `--jobs N` オプションを追加:
```
fav build --jobs 4 src/pipeline.fav -o pipeline.favc
```

---

## T4: モジュール登録

`fav/src/lib.rs` と `fav/src/main.rs` 両方に追記:

```rust
pub mod parallel;  // lib.rs
mod parallel;      // main.rs
```

---

## T5: `v194000_tests` 追加手順

`v193000_tests::version_is_19_3_0` に `#[ignore]` を追加後:

```rust
mod v194000_tests {
    #[test]
    fn version_is_19_4_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("19.4.0"));
    }

    #[test]
    fn parallel_compile_same_output() {
        // 単一ソースを jobs=1 で並列コンパイル
        // 逐次 compile_program の出力と fn 数が一致することを確認
    }

    #[test]
    fn parallel_compile_faster() {
        // 複数ソースで compile_parallel が正常終了することを確認
        // （速度計測は環境依存のため構造テストのみ）
    }

    #[test]
    fn parallel_dep_order_respected() {
        // topo_layers が正しい層順序を返すことを確認
        // a → b → c の依存で [[c], [b], [a]] の順序
    }

    #[test]
    fn parallel_compile_thread_count() {
        // jobs=1 と jobs=0 の両方で compile_parallel が成功することを確認
    }
}
```

---

## 注意点

### `DepGraph::deps_of` の追加

`dep_graph.rs` に `pub fn deps_of(&self, file: &str) -> Vec<String>` を追加する。
`topo.rs` の `in_degree` 計算に必要。

### グローバル `rayon` ThreadPool との干渉

`rayon::ThreadPoolBuilder::new().num_threads(n).build()` でローカル ThreadPool を作成する。
`rayon::ThreadPoolBuilder::global()` などグローバル設定は変更しない（他のテストに影響するため）。

### `compile_program` のスレッドセーフ性

`compile_program` は `NO_TAP_MODE`（`thread_local!`）を参照する。
`thread_local!` は各スレッドに独立した値を持つため、並列実行しても安全。
ただし `set_no_tap_mode()` をマルチスレッド環境で呼ぶ場合は注意が必要。

### `merge_ir_programs` の重複グローバル処理

複数ファイルが同じ名前の builtin グローバル（`IO`, `List` 等）を登録する可能性がある。
既存名との重複チェックを必ず行い、後から来た重複はスキップする。

### テストの並列実行競合

`cargo test` は複数テストを並列実行する。
`compile_parallel` 内の `ThreadPoolBuilder::build()` はローカルプールを作成するので
グローバル rayon 状態を変更せず安全。
