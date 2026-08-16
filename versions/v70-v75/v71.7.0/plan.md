# v71.7.0 実装計画 — WebAssembly ターゲット テストカバレッジ確立

Date: 2026-08-10

---

## 依存関係

```
T0（事前確認）
  └→ T1（v717000_tests 追加）
       └→ T2（cargo_toml_version テスト更新）
            └→ T3（Cargo.toml バージョン更新）
                 └→ T4（CHANGELOG.md 更新）
                      └→ T5（versions/current.md 更新）
                           └→ T6（最終確認）
```

---

## 実装ステップ

### Step 0: 事前確認

- `fav/Cargo.toml` のバージョンが `71.6.0` であることを確認
- `cargo test` が 3602 tests pass であることを確認
- `driver.rs` に `v716000_tests` モジュールが存在し、`v717000_tests` が未存在であることを確認
- `build_wasm_artifact` および `build_wasm_artifact_with_config` の関数シグネチャを確認

### Step 1: `v717000_tests` モジュール追加（`driver.rs`）

`v716000_tests` モジュールの直後に以下を追加:

```rust
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod v717000_tests {
    // driver.rs 内の mod のため super:: を使用（crate::driver:: は非 pub に非アクセス）
    use super::{build_wasm_artifact, build_wasm_artifact_with_config, WasmBuildConfig};
    use crate::frontend::parser::Parser;
    // wasm_exec_main は完全パスで呼ぶ

    #[test]
    fn wasm_target_compiles() {
        let src = r#"
public fn main() -> Unit {
    IO.println("wasm-ok")
}
"#;
        let program = Parser::parse_str(src, "test.fav").expect("parse");
        let config = WasmBuildConfig { dce: true, ..WasmBuildConfig::default() };
        let bytes = build_wasm_artifact_with_config(&program, &config)
            .expect("build_wasm_artifact_with_config should succeed");
        assert!(!bytes.is_empty(), "WASM output must not be empty");
        assert_eq!(&bytes[..4], b"\0asm", "must have WASM magic number");
    }

    #[test]
    fn wasm_target_runs_simple_pipeline() {
        let src = r#"
public fn double(x: Int) -> Int { x * 2 }
public fn main() -> Unit { IO.println_int(double(21)) }
"#;
        let program = Parser::parse_str(src, "pipeline.fav").expect("parse");
        let bytes = build_wasm_artifact(&program)
            .expect("build_wasm_artifact should succeed");
        crate::backend::wasm_exec::wasm_exec_main(&bytes)
            .expect("wasm_exec_main should succeed");
    }
}
```

### Step 2: `cargo_toml_version` テスト文字列を更新

`driver.rs` 内の `"71.6.0"` バージョンアサーション文字列を `"71.7.0"` に一括更新。

### Step 3: `fav/Cargo.toml` バージョン更新

`version = "71.6.0"` → `version = "71.7.0"`

### Step 4: `cargo test v717000` で 2 件 pass 確認

`cargo test` 全体で 3604 tests pass を確認。

### Step 5: `CHANGELOG.md` 更新

先頭に `## [v71.7.0]` エントリを追加。

### Step 6: `versions/current.md` 更新

- 進行中: `v71.7.0`（WebAssembly ターゲット テストカバレッジ確立）
- 次: `v71.8.0`

---

## 注意事項

- `build_wasm_artifact_with_config` は `WasmOptLevel::Os` ではなく `WasmOptLevel::O0` がデフォルト
  — テストは `dce: true` のみ指定、opt_level はデフォルト（O0）で十分
- `wasm_exec_main` は Wasmtime ベース。`#[cfg(not(target_arch = "wasm32"))]` ガードが必要
- `driver.rs` の `cargo_toml_version` アサーションは多数存在するため sed/replace_all で一括更新
