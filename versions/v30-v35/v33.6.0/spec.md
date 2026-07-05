# v33.6.0 — Spec: WASM 最適化 確認・テスト補強

## 概要

v33.6.0 は **WASM 最適化**（DCE / wasm-opt / WasmBuildConfig）の確認・テスト補強バージョン。

ロードマップ v33.6 のテーマ「Playground の初期ロードを高速化する（WASM サイズ 50% 削減）」は
v19.6.0 で既に実装済みである。

| コンポーネント | 実装済み | バージョン |
|---|---|---|
| `WasmBuildConfig`（target / opt_level / strip_debug / size_report / dce）| ✓ | v19.6.0 |
| `WasmOptLevel`（O0 / O1 / O2 / O3）| ✓ | v19.6.0 |
| `WasmTarget`（Wasm32 / Wasm32Wasi）| ✓ | v19.6.0 |
| `build_wasm_artifact_with_config` | ✓ | v19.6.0 |
| `wasm_dce::collect_reachable_fns` / `apply_dce` | ✓ | v19.6.0 |
| `WasmSizeReport::reduction_pct()` | ✓ | v19.6.0 |
| `v196000_tests` — 4 件（`wasm_dce_reduces_fn_count` 等）| ✓ | v19.6.0 |

v33.6.0 では新規実装は行わず、`v336000_tests` で動作を確認・記録するにとどまる
（v33.1〜v33.5 と同じ「確認・記録」パターン）。

---

## WASM 最適化 仕様確認

### WasmBuildConfig デフォルト値

```rust
WasmBuildConfig::default() == WasmBuildConfig {
    target:      WasmTarget::Wasm32,
    opt_level:   WasmOptLevel::O0,  // DCE のみ、外部ツール不使用
    strip_debug: false,
    size_report: false,
    dce:         true,              // DCE はデフォルト有効
}
```

### DCE（デッドコード除去）の設計

- `collect_reachable_fns(&ir, "main")` — `main` から到達可能な関数名の集合を返す
- `apply_dce(&mut ir, &reachable)` — 到達不可能な関数を `ir.fns` から除去
- 到達可能な関数は**絶対に除去されない**（安全性保証）

### WasmOptLevel::O0 の動作

`O0` は外部の `wasm-opt` を呼ばない。DCE のみを適用する。
`O1`〜`O3` は `wasm-opt` が PATH にインストールされている場合に呼び出す。

---

## 追加するテスト（v336000_tests — 4 件）

v196000_tests（`wasm_dce_reduces_fn_count` / `wasm_size_report_computes` /
`wasm_output_correct` / `wasm_wasi_target_builds`）と被らないよう設計する。

モジュール冒頭 import パターン:

```rust
mod v336000_tests {
    use crate::backend::wasm_dce::{apply_dce, collect_reachable_fns};
    use crate::driver::{WasmBuildConfig, WasmTarget};
    use crate::backend::wasm_opt_pass::WasmOptLevel;
    use crate::frontend::parser::Parser;
    use crate::middle::compiler::compile_program;
}
```

`use super::*` **なし**。

### テスト 1: バージョン確認

```rust
fn cargo_toml_version_is_33_6_0() {
    let src = include_str!("../Cargo.toml");
    assert!(src.contains("33.6.0"), "Cargo.toml must contain '33.6.0'");
}
```

### テスト 2: ベンチマーク存在確認

```rust
fn benchmark_v33_6_0_exists() {
    let src = include_str!("../../benchmarks/v33.6.0.json");
    assert!(src.contains("33.6.0"), "benchmarks/v33.6.0.json must contain '33.6.0'");
}
```

### テスト 3: DCE は到達可能な関数を保持する（逆ケース）

v196000_tests::wasm_dce_reduces_fn_count は「到達不可能な関数が除去される」を確認する。
v33.6.0 では「到達可能な関数は除去されない」を確認し、DCE の安全性保証を記録する。

> **設計注釈**: Favnir IR は関数名をマングルしない。`IrFn.name` フィールドはソース上の
> 関数名（例: `"helper"`）をそのまま保持するため、`f.name.contains("helper")` の
> 部分一致チェックは安全に使用できる。

```rust
fn wasm_dce_keeps_reachable_fn() {
    // helper は main から呼ばれるため DCE で除去されてはならない
    let src = r#"
fn helper() -> Int { 10 }
public fn main() -> Unit !Io {
    IO.println(Int.to_string(helper()))
}
"#;
    let prog = Parser::parse_str(src, "test.fav").expect("parse");
    let mut ir = compile_program(&prog);
    let reachable = collect_reachable_fns(&ir, "main");
    apply_dce(&mut ir, &reachable);
    let has_helper = ir.fns.iter().any(|f| f.name.contains("helper"));
    assert!(has_helper, "DCE must not remove reachable function 'helper'");
}
```

### テスト 4: WasmBuildConfig デフォルト値の確認

v196000_tests は `WasmBuildConfig { dce: true, ..WasmBuildConfig::default() }` と使うが、
デフォルト値自体を検証するテストは存在しない。
v33.6.0 では `WasmBuildConfig::default()` の各フィールドをアサートして設計を記録する。

```rust
fn wasm_default_config_is_o0_with_dce() {
    // WasmBuildConfig のデフォルトは O0（外部ツール不使用）+ DCE 有効
    let config = WasmBuildConfig::default();
    assert!(matches!(config.opt_level, WasmOptLevel::O0), "default opt_level should be O0");
    assert!(config.dce, "default dce should be true");
    assert!(!config.strip_debug, "default strip_debug should be false");
    assert!(matches!(config.target, WasmTarget::Wasm32), "default target should be Wasm32");
}
```

---

## テストモジュールの配置

`v336000_tests` は `v335000_tests` の閉じ括弧（`}`）の直後、
かつ `// ── v31.7.0 tests` コメントの前に挿入する。

---

## 完了条件

- `Cargo.toml` version = `"33.6.0"`
- `cargo_toml_version_is_33_5_0` が空スタブになっていること
- `cargo test --bin fav v336000` — 4/4 PASS
- `cargo test` — 全件 PASS（2520 件、0 failures）
- `CHANGELOG.md` に `[v33.6.0]` セクション
- `benchmarks/v33.6.0.json` 存在かつ `tests_passed` が実測値
- `benchmarks/v33.6.0.json` の `milestone` フィールドが `"Performance & Tooling"` であること
- `versions/current.md` を v33.6.0 に更新
- `tasks.md` がすべて `[x]` で COMPLETE に更新されていること
