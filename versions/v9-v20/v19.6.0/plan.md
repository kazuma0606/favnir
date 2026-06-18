# v19.6.0 Plan — WASM バイナリ最適化

## 前提確認

### 既存の WASM ビルドフロー

```
Program (AST)
    ↓  compile_program() [compiler.rs]
IRProgram
    ↓  wasm_codegen_program() [wasm_codegen.rs]
Vec<u8>  (WASM バイト列)
    ↓  write to file / execute via wasmtime
```

### 関連ファイル

- `fav/src/backend/wasm_codegen.rs`（2466 行）— WASM コード生成本体
  - `wasm_codegen_program(ir: &IRProgram) -> Result<Vec<u8>, WasmCodegenError>`
  - `collect_used_builtins()` — 使用 builtin の収集（DCE のモデル）
- `fav/src/backend/wasm_exec.rs`（175 行）— `wasmtime` 経由の WASM 実行
- `fav/src/backend/mod.rs` — `pub mod wasm_codegen; pub mod wasm_exec;`
- `fav/src/driver.rs` — `build_wasm_artifact(program) -> Result<Vec<u8>>`

### 現状の問題点

- `wasm_codegen_program` には DCE が一切ない（全 `ir.fns` を WASM に含める）
- `wasm-opt` 統合なし
- `wasm32-wasi` ターゲットなし
- サイズ計測なし

---

## 実装計画

### T1: `src/backend/wasm_dce.rs`（新規）— Dead Code Elimination

```rust
use std::collections::{HashMap, HashSet, VecDeque};
use crate::middle::ir::{IRExpr, IRFnDef, IRGlobal, IRGlobalKind, IRProgram, IRStmt};

/// main を起点に到達可能な fn インデックスを BFS で収集。
pub fn collect_reachable_fns(ir: &IRProgram, entry: &str) -> HashSet<usize> {
    // entry 名から fn インデックスを解決
    let fn_idx_by_name: HashMap<String, usize> = ir.globals.iter()
        .filter_map(|g| {
            if let IRGlobalKind::Fn(idx) = g.kind { Some((g.name.clone(), idx)) }
            else { None }
        })
        .collect();

    let entry_idx = match fn_idx_by_name.get(entry) {
        Some(idx) => *idx,
        None => return HashSet::new(),
    };

    let mut visited: HashSet<usize> = HashSet::new();
    let mut queue: VecDeque<usize> = VecDeque::new();
    queue.push_back(entry_idx);

    while let Some(fn_idx) = queue.pop_front() {
        if !visited.insert(fn_idx) { continue; }
        if let Some(fn_def) = ir.fns.get(fn_idx) {
            collect_fn_calls_stmts(&fn_def.body, &fn_idx_by_name, &ir.globals, &mut queue);
        }
    }
    visited
}

fn collect_fn_calls_stmts(
    stmts: &[IRStmt],
    by_name: &HashMap<String, usize>,
    globals: &[IRGlobal],
    queue: &mut VecDeque<usize>,
) {
    for stmt in stmts {
        collect_fn_calls_stmt(stmt, by_name, globals, queue);
    }
}

fn collect_fn_calls_stmt(stmt: &IRStmt, by_name: &HashMap<String, usize>, globals: &[IRGlobal], queue: &mut VecDeque<usize>) {
    // IRStmt を再帰的に走査して IRExpr::Global(idx) の Fn 種別を収集する
    // （IRStmt / IRExpr の実際の enum 構造に合わせて実装）
    // ...（exhaustive match）
}

fn collect_fn_calls_expr(expr: &IRExpr, by_name: &HashMap<String, usize>, globals: &[IRGlobal], queue: &mut VecDeque<usize>) {
    match expr {
        IRExpr::Global(idx, _) => {
            if let Some(g) = globals.get(*idx as usize) {
                if let IRGlobalKind::Fn(fn_idx) = g.kind {
                    queue.push_back(fn_idx);
                }
            }
        }
        // Call, FieldAccess, Block, Match, If, etc. — 再帰的に処理
        _ => {}
    }
}

/// DCE を適用し、unreachable な fn を ir.fns から除去。
/// IRGlobal / IRExpr::Global の fn インデックスを remap する。
pub fn apply_dce(ir: &mut IRProgram, reachable: &HashSet<usize>) -> DceReport {
    let original_count = ir.fns.len();

    // old_fn_idx → new_fn_idx のマッピングを構築
    let mut remap: HashMap<usize, usize> = HashMap::new();
    let mut new_fns = Vec::new();
    for (old_idx, fn_def) in ir.fns.iter().enumerate() {
        if reachable.contains(&old_idx) {
            remap.insert(old_idx, new_fns.len());
            new_fns.push(fn_def.clone());
        }
    }
    ir.fns = new_fns;

    // IRGlobal の Fn 参照を remap
    for g in ir.globals.iter_mut() {
        if let IRGlobalKind::Fn(ref mut idx) = g.kind {
            if let Some(&new_idx) = remap.get(&*idx) {
                *idx = new_idx;
            }
        }
    }

    // IRGlobal: Fn 種別で到達不可能なものを除去（名前なしの closure 等）
    ir.globals.retain(|g| match g.kind {
        IRGlobalKind::Fn(idx) => idx < ir.fns.len(),
        _ => true,
    });

    DceReport {
        removed: original_count - ir.fns.len(),
        remaining: ir.fns.len(),
    }
}

#[derive(Debug, Clone)]
pub struct DceReport {
    pub removed: usize,
    pub remaining: usize,
}
```

#### 実装上の注意

- `collect_fn_calls_stmt` / `collect_fn_calls_expr` は `IRStmt` / `IRExpr` の exhaustive match が必要。
  `ir.rs` の enum を参照して全 variant に対応する。
- `ir.fns` の再帰インデックス（closure、local fn 等）も remap 対象。
- DCE 適用後に `wasm_codegen_program(ir)` を呼ぶ。

---

### T2: `src/backend/wasm_opt_pass.rs`（新規）— wasm-opt 統合・サイズ計測

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmOptLevel {
    O0, // DCE のみ（wasm-opt 不使用）
    O1,
    O2,
    O3,
}

impl WasmOptLevel {
    pub fn flag(&self) -> &'static str {
        match self {
            WasmOptLevel::O0 => "-O0",
            WasmOptLevel::O1 => "-O1",
            WasmOptLevel::O2 => "-O2",
            WasmOptLevel::O3 => "-O3",
        }
    }
}

#[derive(Debug, Clone)]
pub struct WasmSizeReport {
    pub before: usize,
    pub after: usize,
}

impl WasmSizeReport {
    pub fn reduction_pct(&self) -> f64 {
        if self.before == 0 { return 0.0; }
        (1.0 - self.after as f64 / self.before as f64) * 100.0
    }
}

#[derive(Debug)]
pub enum WasmOptError {
    NotInstalled,        // wasm-opt バイナリが見つからない
    ExitNonZero(i32),   // wasm-opt が非 0 終了
    Io(String),          // ファイル I/O エラー
}

/// wasm-opt バイナリを `std::process::Command` 経由で実行する。
/// バイナリが見つからない場合は `WasmOptError::NotInstalled` を返す（エラーにしない）。
pub fn run_wasm_opt(
    bytes: &[u8],
    level: WasmOptLevel,
    strip_debug: bool,
) -> Result<(Vec<u8>, WasmSizeReport), WasmOptError> {
    use std::io::Write;
    use std::process::Command;

    let before = bytes.len();

    // O0 の場合は wasm-opt を呼ばない
    if level == WasmOptLevel::O0 {
        return Ok((bytes.to_vec(), WasmSizeReport { before, after: before }));
    }

    // tempfile を使って入力 WASM を書き出す
    let mut in_file = tempfile::NamedTempFile::new()
        .map_err(|e| WasmOptError::Io(e.to_string()))?;
    in_file.write_all(bytes).map_err(|e| WasmOptError::Io(e.to_string()))?;
    let out_file = tempfile::NamedTempFile::new()
        .map_err(|e| WasmOptError::Io(e.to_string()))?;

    let mut cmd = Command::new("wasm-opt");
    cmd.arg(level.flag());
    if strip_debug { cmd.arg("--strip-debug"); }
    cmd.arg("--vacuum");
    cmd.arg(in_file.path());
    cmd.arg("-o").arg(out_file.path());

    let status = match cmd.status() {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(WasmOptError::NotInstalled);
        }
        Err(e) => return Err(WasmOptError::Io(e.to_string())),
    };

    if !status.success() {
        return Err(WasmOptError::ExitNonZero(status.code().unwrap_or(-1)));
    }

    let optimized = std::fs::read(out_file.path())
        .map_err(|e| WasmOptError::Io(e.to_string()))?;
    let after = optimized.len();
    Ok((optimized, WasmSizeReport { before, after }))
}

/// wasm-opt を試みる。未インストールの場合は入力をそのまま返す。
pub fn try_wasm_opt(
    bytes: Vec<u8>,
    level: WasmOptLevel,
    strip_debug: bool,
) -> (Vec<u8>, WasmSizeReport) {
    let before = bytes.len();
    match run_wasm_opt(&bytes, level, strip_debug) {
        Ok(result) => result,
        Err(WasmOptError::NotInstalled) => {
            eprintln!("wasm-opt not found; skipping optimization (install binaryen to enable)");
            (bytes, WasmSizeReport { before, after: before })
        }
        Err(e) => {
            eprintln!("wasm-opt failed: {:?}; using unoptimized output", e);
            (bytes, WasmSizeReport { before, after: before })
        }
    }
}
```

---

### T3: `src/backend/mod.rs` — 新規モジュール追加

```rust
pub mod wasm_dce;
pub mod wasm_opt_pass;
```

---

### T4: `src/driver.rs` — `WasmBuildConfig` / `build_wasm_artifact_with_config`

#### 追加する型

```rust
// driver.rs の先頭付近
#[derive(Debug, Clone, PartialEq)]
pub enum WasmTarget {
    Wasm32,      // --target wasm（従来）
    Wasm32Wasi,  // --target wasm32-wasi
}

#[derive(Debug, Clone)]
pub struct WasmBuildConfig {
    pub target: WasmTarget,
    pub opt_level: crate::backend::wasm_opt_pass::WasmOptLevel,
    pub strip_debug: bool,
    pub size_report: bool,
    pub dce: bool,
}

impl Default for WasmBuildConfig {
    fn default() -> Self {
        Self {
            target: WasmTarget::Wasm32,
            opt_level: crate::backend::wasm_opt_pass::WasmOptLevel::O0,
            strip_debug: false,
            size_report: false,
            dce: true,
        }
    }
}
```

#### ヘルパー関数

```rust
pub fn build_wasm_artifact_with_config(
    program: &ast::Program,
    config: &WasmBuildConfig,
) -> Result<Vec<u8>, String> {
    use crate::backend::wasm_dce::{apply_dce, collect_reachable_fns};
    use crate::backend::wasm_opt_pass::{try_wasm_opt};

    let mut ir = crate::middle::compiler::compile_program(program);

    // DCE: main から到達不可能な fn を除去
    if config.dce {
        let reachable = collect_reachable_fns(&ir, "main");
        apply_dce(&mut ir, &reachable);
    }

    // WASM コード生成
    let bytes = if config.target == WasmTarget::Wasm32Wasi {
        crate::backend::wasm_codegen::wasm_codegen_program_wasi(&ir)
            .map_err(|e| e.to_string())?
    } else {
        crate::backend::wasm_codegen::wasm_codegen_program(&ir)
            .map_err(|e| e.to_string())?
    };

    // wasm-opt（オプション）
    let (bytes, report) = try_wasm_opt(bytes, config.opt_level, config.strip_debug);

    if config.size_report {
        eprintln!(
            "WASM size: before={} bytes, after={} bytes, reduced={:.1}%",
            report.before, report.after, report.reduction_pct()
        );
    }

    Ok(bytes)
}
```

---

### T5: `wasm_codegen.rs` — `wasm_codegen_program_wasi` 追加

`wasm_codegen_program` のバリアント。差分:
- `_start` エクスポートを追加（`main` エクスポートに加えて）
- WASI エクスポートセクションに `memory` を含める
- `proc_exit` インポートを `wasi_snapshot_preview1` から宣言

```rust
pub fn wasm_codegen_program_wasi(ir: &IRProgram) -> Result<Vec<u8>, WasmCodegenError> {
    // 既存の wasm_codegen_program をベースに
    // _start エクスポートを追加
    let mut bytes = wasm_codegen_program(ir)?;
    // wasm-encoder で _start セクションを追加する最小実装
    // 実際には wasm_codegen_program のリファクタリングが必要
    // 簡易実装: wasm-encoder の Module を再構築して _start を追加
    let _ = bytes; // placeholder
    wasm_codegen_program_wasi_impl(ir)
}
```

最小実装アプローチ:
- `wasm_codegen_program` と共通の内部関数 `wasm_codegen_program_inner(ir, wasi: bool)` を作る
- `wasi = true` の場合に export セクションへ `_start` を追加する

---

### T6: `v196000_tests` 追加（driver.rs 末尾）

```rust
// ── v196000_tests (v19.6.0) — WASM バイナリ最適化 ──────────────────────────
#[cfg(test)]
mod v196000_tests {
    use crate::backend::wasm_dce::{DceReport, apply_dce, collect_reachable_fns};
    use crate::backend::wasm_opt_pass::{WasmOptLevel, WasmSizeReport};
    use crate::driver::{WasmBuildConfig, WasmTarget, build_wasm_artifact_with_config};
    use crate::frontend::parser::Parser;
    use crate::middle::compiler::compile_program;

    #[test]
    fn version_is_19_6_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("19.6.0"), "Cargo.toml should have version 19.6.0");
    }

    #[test]
    fn wasm_dce_reduces_fn_count() {
        // unreachable な helper 関数を含むプログラムで DCE が fn を除去することを確認
        let src = r#"
fn unused_helper() -> Int { 42 }
public fn main() -> Int { 1 }
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        let mut ir = compile_program(&prog);
        let before_count = ir.fns.len();
        let reachable = collect_reachable_fns(&ir, "main");
        let report = apply_dce(&mut ir, &reachable);
        assert!(
            ir.fns.len() < before_count,
            "DCE should remove unreachable functions: before={}, after={}",
            before_count, ir.fns.len()
        );
        assert!(report.removed > 0, "DceReport.removed should be > 0");
    }

    #[test]
    fn wasm_size_report_computes() {
        let report = WasmSizeReport { before: 1000, after: 600 };
        let pct = report.reduction_pct();
        assert!(
            (pct - 40.0).abs() < 0.01,
            "Expected 40.0% reduction, got {:.2}%", pct
        );
        let zero_report = WasmSizeReport { before: 0, after: 0 };
        assert_eq!(zero_report.reduction_pct(), 0.0);
    }

    #[test]
    fn wasm_output_correct() {
        // WASM ビルド → wasmtime 実行で正しい結果を得る
        let src = r#"
public fn main() -> Int { 21 + 21 }
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        let config = WasmBuildConfig {
            dce: true,
            ..WasmBuildConfig::default()
        };
        let bytes = build_wasm_artifact_with_config(&prog, &config)
            .expect("build wasm");
        let result = crate::backend::wasm_exec::exec_wasm_main_int(&bytes)
            .expect("exec wasm");
        assert_eq!(result, 42);
    }

    #[test]
    fn wasm_wasi_target_builds() {
        // --target wasm32-wasi が有効な WASM バイト列を生成する
        let src = r#"
public fn main() -> Int { 1 }
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        let config = WasmBuildConfig {
            target: WasmTarget::Wasm32Wasi,
            dce: true,
            ..WasmBuildConfig::default()
        };
        let bytes = build_wasm_artifact_with_config(&prog, &config)
            .expect("build wasm32-wasi");
        // WASM マジックナンバー確認
        assert_eq!(&bytes[..4], b"\0asm", "Should start with WASM magic");
        // バイト列が non-empty であることを確認
        assert!(bytes.len() > 8, "WASM output too small");
    }
}
```

---

### T7: `site/content/docs/tools/wasm-opt.mdx`（新規）

ドキュメント内容:
- WASM 最適化の概要（DCE + wasm-opt）
- `fav build --target wasm --wasm-opt=O3` の使い方
- `fav build --target wasm32-wasi` の使い方と wasmtime での実行方法
- wasm-opt のインストール方法（`brew install binaryen` / apt）
- サイズレポートの読み方

---

## 実装順序

```
T1（wasm_dce.rs — DCE 実装）                    ← 最優先
T2（wasm_opt_pass.rs — wasm-opt 統合）           ← T1 と並列可
T3（backend/mod.rs — モジュール追加）             ← T1/T2 完了後
T4（driver.rs — WasmBuildConfig）                ← T3 完了後
T5（wasm_codegen.rs — wasm_codegen_program_wasi）← T4 と並列可
T6（v196000_tests — 5件追加）                    ← T4/T5 完了後
T7（Cargo.toml バージョン更新 + ドキュメント）    ← T6 完了後
```

---

## IRExpr / IRStmt traversal のコード例

```rust
fn collect_fn_calls_expr(
    expr: &IRExpr,
    globals: &[IRGlobal],
    queue: &mut VecDeque<usize>,
) {
    match expr {
        IRExpr::Global(idx, _) | IRExpr::TrfRef(idx, _) => {
            if let Some(g) = globals.get(*idx as usize) {
                if let IRGlobalKind::Fn(fn_idx) = g.kind {
                    queue.push_back(fn_idx);
                }
            }
        }
        IRExpr::Block(stmts, tail, _) => {
            for s in stmts { collect_fn_calls_stmt(s, globals, queue); }
            collect_fn_calls_expr(tail, globals, queue);
        }
        IRExpr::Call { func, args, .. } => {
            collect_fn_calls_expr(func, globals, queue);
            for a in args { collect_fn_calls_expr(a, globals, queue); }
        }
        IRExpr::If { cond, then_, else_, .. } => {
            collect_fn_calls_expr(cond, globals, queue);
            collect_fn_calls_expr(then_, globals, queue);
            if let Some(e) = else_ { collect_fn_calls_expr(e, globals, queue); }
        }
        IRExpr::Match { scrutinee, arms, .. } => {
            collect_fn_calls_expr(scrutinee, globals, queue);
            for arm in arms { collect_fn_calls_expr(&arm.body, globals, queue); }
        }
        IRExpr::Closure { global_idx, .. } => {
            if let Some(g) = globals.get(*global_idx as usize) {
                if let IRGlobalKind::Fn(fn_idx) = g.kind {
                    queue.push_back(fn_idx);
                }
            }
        }
        IRExpr::Local(..) | IRExpr::Lit(..) | IRExpr::Unit(_) => {}
        // 残りの variant は再帰的に処理
        _ => {}
    }
}
```

実際の `IRExpr` enum の variant は `src/middle/ir.rs` を参照して exhaustive match にする。

---

## 重要な技術ノート

### DCE の正確性

- `main` 以外のパブリック関数（`public fn`）も起点に含める必要があるか検討
- closure の `global_idx` は関数インデックスへの参照——remap が必要
- `ir.globals` の `IRGlobalKind::Fn(idx)` の idx はゼロベースの `ir.fns` インデックス

### wasm_codegen_program_wasi の最小実装

`wasm_codegen_program` の出力をベースに、エクスポートセクションに `_start` を追加する。
`wasm-encoder` の `Module` を直接再構築するより、既存出力に追加する方が安全。
ただし wasm-encoder は immutable な `Module` を使うため、生成済みバイト列への追加は不可。
→ `wasm_codegen_program_inner(ir, add_start_export: bool)` を内部関数として切り出すのが正解。

### `exec_wasm_main_int` の確認

`wasm_exec.rs` に存在する既存関数（または `exec_wasm` を参照して適切な関数名を確認する）。
v196000 テストでは既存の WASM 実行関数をそのまま使う。

### `WasmOptLevel::O0` = DCE のみ

`O0` は wasm-opt を呼ばない（DCE だけ行う）ため、CI でも常に動作する。
wasm-opt テストは行わない（外部ツール依存のため）。
`wasm_size_report_computes` は純粋な Rust 計算のみをテストする。
