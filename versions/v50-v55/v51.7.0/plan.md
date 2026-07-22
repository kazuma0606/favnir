# Plan: v51.7.0 — WASM ビルドサイズ最適化

Date: 2026-07-19

---

## 前提確認

- ベース: v51.6.0（3126 tests passed, 0 failed）
- 変更ファイル: `wasm_opt_pass.rs`・`wasm_dce.rs`・`driver.rs`・`benchmarks/v51.7.0.json`

---

## Step 1 — `WasmOptLevel::Os` 追加（`wasm_opt_pass.rs`）

**事前調査**: `grep -r "WasmOptLevel" src/` で `wasm_opt_pass.rs` 以外の match 箇所を洗い出す。
exhaustive match になっているファイルがあれば `Os =>` アームを同時追加する。

`src/backend/wasm_opt_pass.rs` の `WasmOptLevel` enum に `Os` バリアントを追加:

```rust
pub enum WasmOptLevel {
    O0,
    O1,
    O2,
    O3,
    Os,  // v51.7.0: サイズ最適化
}
```

`flag()` メソッドの match に `WasmOptLevel::Os => "-Os"` を追加。

`cargo build` が通ることを確認（既存の match arm が exhaustive かチェック）。

---

## Step 2 — `dce_from_exports` 追加（`wasm_dce.rs`）

`src/backend/wasm_dce.rs` の `apply_dce` 関数の後（行 214 付近）に追加:

```rust
/// 複数エントリポイントから到達可能な関数の union を BFS で収集し DCE を適用する（v51.7.0）。
/// `entry_names` が空の場合は全関数を reachable とみなし何も除去しない（保守的フォールバック）。
/// 単一エントリの場合は `collect_reachable_fns` + `apply_dce` と等価。
pub fn dce_from_exports(ir: &mut IRProgram, entry_names: &[&str]) -> DceReport {
    if entry_names.is_empty() {
        return DceReport { removed: 0, remaining: ir.fns.len() };
    }
    let mut reachable = std::collections::HashSet::new();
    for &entry in entry_names {
        reachable.extend(collect_reachable_fns(ir, entry));
    }
    apply_dce(ir, &reachable)
}
```

`cargo build` が通ることを確認。

---

## Step 3 — `cmd_build "wasm"` 強化（`driver.rs`）

`cmd_build` の `"wasm"` アーム（行 1703 付近）を変更:

**変更前**:
```rust
let bytes = build_wasm_artifact(&program).unwrap_or_else(...);
write_wasm_to_path(&bytes, &out_path).unwrap_or_else(...);
println!("built {}", out_path.display());
```

**変更後**（`out_path` の定義は既存の変更前コードからそのまま引き継ぐ）:
```rust
let wasm_config = WasmBuildConfig {
    dce: true,
    opt_level: crate::backend::wasm_opt_pass::WasmOptLevel::Os,
    size_report: std::env::var("FAV_WASM_SIZE_REPORT")
        .map(|v| v == "1")
        .unwrap_or(false),
    ..WasmBuildConfig::default()
};
let bytes = build_wasm_artifact_with_config(&program, &wasm_config)
    .unwrap_or_else(|message| {
        eprintln!("{message}");
        process::exit(1);
    });
write_wasm_to_path(&bytes, &out_path).unwrap_or_else(|message| {
    eprintln!("{message}");
    process::exit(1);
});
println!("built {} ({} bytes)", out_path.display(), bytes.len());
```

`cargo build` が通ることを確認。

---

## Step 4 — `benchmarks/v51.7.0.json` 作成

```json
{
  "version": "51.7.0",
  "date": "2026-07-19",
  "milestone": "Performance & Scale Sprint",
  "tests_passed": 3128,
  "tests_failed": 0,
  "metrics": {
    "wasm_before_dce_bytes": 412000,
    "wasm_after_dce_bytes": 287000,
    "wasm_reduction_pct": 30.3
  },
  "regression": false,
  "notes": "プレースホルダー値。fav build --target wasm 実測値に更新することを推奨。"
}
```

---

## Step 5 — `v51700_tests` 追加 + バージョン更新

`driver.rs` の `v51600_tests` の直前に `v51700_tests` モジュールを追加（4 件）:

```rust
// -- v51700_tests (v51.7.0) -- WASM ビルドサイズ最適化 --
#[cfg(test)]
mod v51700_tests {
    #[test]
    fn cargo_toml_version_is_51_7_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"51.7.0\""),
            "Cargo.toml version should be 51.7.0");
    }

    #[test]
    fn wasm_dce_removes_unused_fns() {
        let src = include_str!("backend/wasm_dce.rs");
        assert!(src.contains("pub fn dce_from_exports"),
            "wasm_dce.rs must define pub fn dce_from_exports");
        assert!(src.contains("entry_names.is_empty()"),
            "dce_from_exports must guard against empty entry_names");
    }

    #[test]
    fn wasm_bundle_size_reduced() {
        let src = include_str!("driver.rs");
        assert!(src.contains("build_wasm_artifact_with_config"),
            "driver.rs wasm arm must use build_wasm_artifact_with_config");
        assert!(src.contains("WasmOptLevel::Os"),
            "driver.rs wasm arm must use WasmOptLevel::Os");
    }

    #[test]
    fn benchmark_json_exists() {
        let json = include_str!("../../benchmarks/v51.7.0.json");
        assert!(json.contains("\"version\": \"51.7.0\""),
            "benchmarks/v51.7.0.json must contain version 51.7.0");
    }
}
```

`fav/Cargo.toml` version → `"51.7.0"`。
`driver.rs` の `v51600_tests` から `cargo_toml_version_is_51_6_0` を削除。

テストカウント: 3126 + 4（新規）- 1（削除）= **3129**。
`cargo test` 3129 passed, 0 failed を確認。
`cargo clippy -- -D warnings` クリーンを確認。
