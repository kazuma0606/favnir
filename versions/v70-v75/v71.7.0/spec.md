# v71.7.0 spec — WebAssembly ターゲット テストカバレッジ確立

Date: 2026-08-10

---

## Background

`fav build --target wasm` は v19.6.0〜v64.7.0 にかけて段階的に実装され、現在は完全に動作する。

- `cmd_build --target wasm` ブランチ（`driver.rs` line 1907）: DCE + Os 最適化付きで `.wasm` 生成
- `cmd_build_wasm(src, out)` (`driver.rs` line 2013): 文字列ソースから WASM バイト列生成
- `build_wasm_artifact` / `build_wasm_artifact_with_config`: 内部 API
- `wasm_exec_main` / `exec_wasm_bytes`: WASM 実行（Wasmtime ベース）

ロードマップ v71.7.0 ではこの WASM パイプライン全体に対して
バージョン固有の Rust テストを 2 件追加し、テストカバレッジを明示的に確立する。

---

## Goals

1. `v717000_tests` モジュールを `driver.rs` に追加する（`v716000_tests` の直後）
2. `wasm_target_compiles`: `build_wasm_artifact_with_config` が有効な WASM バイナリ（`\0asm` マジック）を生成することを確認
3. `wasm_target_runs_simple_pipeline`: `build_wasm_artifact` + `wasm_exec_main` でパイプラインが実際に実行されることを確認
4. Cargo.toml バージョンを `71.7.0` に更新

---

## 使用する内部 API

```rust
// テスト内で使用する関数（v717000_tests は driver.rs 内の mod のため super:: を使用）
use super::{build_wasm_artifact, build_wasm_artifact_with_config, WasmBuildConfig};
use crate::frontend::parser::Parser;
// wasm_exec_main は完全パスで呼ぶ: crate::backend::wasm_exec::wasm_exec_main(...)
```

---

## テスト詳細

### `wasm_target_compiles`

```rust
let src = r#"
public fn main() -> Unit {
    IO.println("wasm-ok")
}
"#;
let program = Parser::parse_str(src, "test.fav").expect("parse");
let config = WasmBuildConfig { dce: true, ..WasmBuildConfig::default() };
let bytes = build_wasm_artifact_with_config(&program, &config).expect("build wasm");
assert!(!bytes.is_empty(), "WASM output must not be empty");
assert_eq!(&bytes[..4], b"\0asm", "must have WASM magic number");
```

### `wasm_target_runs_simple_pipeline`

```rust
let src = r#"
public fn double(x: Int) -> Int { x * 2 }
public fn main() -> Unit { IO.println_int(double(21)) }
"#;
let program = Parser::parse_str(src, "pipeline.fav").expect("parse");
let bytes = build_wasm_artifact(&program).expect("build wasm");
crate::backend::wasm_exec::wasm_exec_main(&bytes).expect("exec wasm");
```

---

## Success Criteria

- `cargo test v717000` で 2 件 pass（0 failures）
- `cargo test` 全体で 3604 tests pass（3602 + 2）
- `fav/Cargo.toml` のバージョンが `71.7.0`

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v717000_tests` モジュール追加（2 テスト） + cargo_toml_version 更新 |
| `fav/Cargo.toml` | バージョン `71.6.0` → `71.7.0` |
| `CHANGELOG.md` | `## [v71.7.0]` エントリ追加 |
| `versions/current.md` | 進行中: v71.7.0 / 次: v71.8.0 |

---

## スコープ外

- `@favnir/wasm` npm パッケージの実際のパブリッシュ: CI/CD スコープ外
- WASM stdio ブリッジの新規実装: 既存実装で十分
- `wasm-bindgen` 統合: 将来バージョン
