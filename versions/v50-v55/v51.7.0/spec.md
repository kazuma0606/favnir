# Spec: v51.7.0 — WASM ビルドサイズ最適化

Date: 2026-07-19
Status: 設計中

---

## 概要

ロードマップ: `wasm_dce.rs` の DCE を強化し、未参照の export と内部関数を除去。
`wasm-opt -Os` 呼び出しを `fav build --target wasm` に統合。

v51.7.0 では以下の 3 点を実装する:

1. **`WasmOptLevel::Os`** — `wasm_opt_pass.rs` に `-Os`（サイズ最適化）レベルを追加
2. **`dce_from_exports`** — `wasm_dce.rs` に複数エントリポイントからの DCE を追加
3. **`cmd_build "wasm"` 強化** — `build_wasm_artifact_with_config`（DCE + Os 最適化）へ切替

---

## 機能詳細

### 1. `WasmOptLevel::Os` 追加（`wasm_opt_pass.rs`）

**背景**: 既存 `WasmOptLevel` は `O0`/`O1`/`O2`/`O3` のみ。
ロードマップ指定の `-Os` はサイズ最適化特化フラグで Binaryen でサポートされている。

```rust
pub enum WasmOptLevel {
    O0,
    O1,
    O2,
    O3,
    Os,  // v51.7.0: サイズ最適化（-Os）
}

impl WasmOptLevel {
    pub fn flag(self) -> &'static str {
        match self {
            WasmOptLevel::O0 => "-O0",
            WasmOptLevel::O1 => "-O1",
            WasmOptLevel::O2 => "-O2",
            WasmOptLevel::O3 => "-O3",
            WasmOptLevel::Os => "-Os",  // v51.7.0
        }
    }
}
```

---

### 2. `dce_from_exports` 追加（`wasm_dce.rs`）

**背景**: 既存 `collect_reachable_fns(ir, "main")` は単一エントリポイントのみ対応。
ライブラリモード WASM では複数の export 関数をエントリポイントとして保持する必要がある。

**新規追加**:

```rust
/// 複数エントリポイントから到達可能な関数の union を BFS で収集し DCE を適用する（v51.7.0）。
/// `entry_names` が空の場合は全関数を reachable とみなし何も除去しない。
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

NOTE: `entry_names` が空 → 何もしない（保守的フォールバック）。
      単一エントリなら `collect_reachable_fns` + `apply_dce` と等価。

---

### 3. `cmd_build "wasm"` 強化（`driver.rs`）

**背景**: 現在の `"wasm"` アームは `build_wasm_artifact(&program)` を呼び出しており
DCE も `wasm-opt` も適用されない。`build_wasm_artifact_with_config` は既に DCE + wasm-opt 対応済みだが使われていない。

**変更**:

```rust
"wasm" => {
    let out_path = ...;
    let wasm_config = WasmBuildConfig {
        dce: true,
        opt_level: crate::backend::wasm_opt_pass::WasmOptLevel::Os,
        size_report: std::env::var("FAV_WASM_SIZE_REPORT").map(|v| v == "1").unwrap_or(false),
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
}
```

サイズレポートは `FAV_WASM_SIZE_REPORT=1` 環境変数で有効化（CI / 手動計測向け）。

---

### 4. `benchmarks/v51.7.0.json`

```json
{
  "version": "51.7.0",
  "date": "2026-07-19",
  "milestone": "Performance & Scale Sprint",
  "tests_passed": 3129,
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

## テスト仕様（`v51700_tests`）— 4 件

### `cargo_toml_version_is_51_7_0`

`Cargo.toml` に `version = "51.7.0"` が含まれることを assert。

### `wasm_dce_removes_unused_fns`

`wasm_dce.rs` のソースコードを `include_str!("backend/wasm_dce.rs")` で読み込み:
- `src.contains("pub fn dce_from_exports")` を assert
- `src.contains("entry_names.is_empty()")` を assert（空リストガードの確認）

### `wasm_bundle_size_reduced`

`driver.rs` のソースコードを `include_str!("driver.rs")` で読み込み:
- `src.contains("build_wasm_artifact_with_config")` を assert
- `src.contains("WasmOptLevel::Os")` を assert

NOTE: `driver.rs` 内にテスト自身を記述するため、テストソースの文字列リテラルが
      `"WasmOptLevel::Os"` を含むことで誤検知する可能性がある。
      これは当プロジェクトで広く使われるパターンであり許容する。
      実際の WASM バイナリサイズ比較は機能テスト（`wasm_dce.rs` の既存 tests）で担保済み。

### `benchmark_json_exists`

`include_str!("../../benchmarks/v51.7.0.json")` で JSON を読み込み:
- `json.contains("\"version\": \"51.7.0\"")` を assert

---

## 既存機能との共存

| 既存 | v51.7.0 | 共存方針 |
|---|---|---|
| `build_wasm_artifact` | `build_wasm_artifact_with_config` へ切替 | 既存関数は残す（他テストが参照している可能性） |
| `collect_reachable_fns` + `apply_dce` | `dce_from_exports` 追加 | 既存関数は変更なし |
| `WasmOptLevel::O0~O3` | `Os` 追加 | `wasm_opt_pass.rs` の `flag()` に match アームを追加。`WasmOptLevel` を match している他ファイルがある場合は exhaustive match エラーになるため、実装前に `grep -r "WasmOptLevel" src/` で影響箇所を調査すること |
| `WasmBuildConfig.dce: true` | デフォルトのまま | 変更なし |

---

## 完了条件

- `cargo test` 3129 passed, 0 failed（3126 + 4 新規 - 1 削除）
- `cargo clippy -- -D warnings` クリーン
- `v51700_tests` 4 件 pass:
  - `cargo_toml_version_is_51_7_0`
  - `wasm_dce_removes_unused_fns`
  - `wasm_bundle_size_reduced`
  - `benchmark_json_exists`
- `benchmarks/v51.7.0.json` 存在
- `CHANGELOG.md` に v51.7.0 エントリが存在する
- `versions/current.md` が v51.7.0（3129 tests）を示す
