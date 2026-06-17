# v19.6.0 Spec — WASM バイナリ最適化

Date: 2026-06-17

## テーマ

WASM バイナリサイズの削減と `fav build --target wasm` パイプラインの強化。
Playground の初期ロードを高速化し、`wasm32-wasi` ターゲットを追加する。

---

## 背景と目的

### 現状の問題

```
現在の @favnir/wasm バイナリ:
  - 全 stdlib / 全 VM opcode handler がバンドルされている
  - 未使用関数が WASM に含まれる（Dead Code）
  - wasm-opt による最適化パスが存在しない
  - wasm32-wasi ターゲットに対応していない
```

### 目標

```
v19.6.0 以降:
  - DCE（Dead Code Elimination）により未使用関数を除去
  - wasm-opt（Binaryen）を自動統合（インストール済みの場合）
  - wasm32-wasi ターゲット対応（WASI ABI 準拠の WASM 生成）
  - サイズ削減率を計測・レポート
  - --wasm-opt レベル（O0/O1/O2/O3）の CLI 制御
```

---

## 機能設計

### 1. Dead Code Elimination（DCE）

`main` 関数を起点とした到達可能性解析で、呼び出されない関数を IRProgram から除去する。
DCE は WASM コード生成前に適用する。

```rust
// src/backend/wasm_dce.rs
pub fn collect_reachable_fns(ir: &IRProgram, entry: &str) -> HashSet<usize>;
pub fn apply_dce(ir: &mut IRProgram, reachable: &HashSet<usize>);
pub struct DceReport { pub removed: usize, pub remaining: usize }
```

### 2. wasm-opt 統合

外部バイナリ `wasm-opt`（Binaryen）を `std::process::Command` 経由で実行する。
未インストールの場合は警告を出し、入力をそのまま返す（エラーにしない）。

```rust
// src/backend/wasm_opt_pass.rs
pub enum WasmOptLevel { O0, O1, O2, O3 }
pub struct WasmSizeReport { pub before: usize, pub after: usize }

impl WasmSizeReport {
    pub fn reduction_pct(&self) -> f64 { ... }
}

pub fn run_wasm_opt(bytes: &[u8], level: WasmOptLevel, strip_debug: bool)
    -> Result<(Vec<u8>, WasmSizeReport), WasmOptError>;
```

### 3. wasm32-wasi ターゲット

WASI ABI 準拠の WASM を生成する。
`fav build --target wasm32-wasi` で有効化。
通常の `--target wasm` との差分:
- メモリインポートを `wasi_snapshot_preview1` からに変更
- `_start` エクスポートを追加（`main` のエイリアス）
- `proc_exit` / `fd_write` インポートを宣言

### 4. CLI フラグ

```bash
# 基本（従来通り）
fav build --target wasm src/main.fav -o main.wasm

# WASI ターゲット
fav build --target wasm32-wasi src/main.fav -o main.wasm
wasmtime main.wasm  # wasmtime で直接実行可能

# wasm-opt レベル指定（デフォルト O1）
fav build --target wasm --wasm-opt=O3 src/main.fav -o main.wasm

# 最適化なし（DCE のみ）
fav build --target wasm --wasm-opt=O0 src/main.fav -o main.wasm

# デバッグ情報保持
fav build --target wasm --wasm-opt=O2 --no-strip-debug src/main.fav

# サイズレポート
fav build --target wasm --wasm-opt=O2 --size-report src/main.fav
# → Before: 48,320 bytes / After: 28,103 bytes / -41.8%
```

### 5. WasmBuildConfig

```rust
#[derive(Debug, Clone)]
pub struct WasmBuildConfig {
    pub target: WasmTarget,
    pub opt_level: WasmOptLevel,
    pub strip_debug: bool,
    pub size_report: bool,
    pub dce: bool,   // デフォルト true
}

#[derive(Debug, Clone, PartialEq)]
pub enum WasmTarget {
    Wasm32,      // --target wasm（従来）
    Wasm32Wasi,  // --target wasm32-wasi
}
```

---

## テスト（v196000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_19_6_0` | Cargo.toml に `"19.6.0"` が含まれる |
| `wasm_dce_reduces_fn_count` | 未使用関数を含む IR に DCE を適用し、関数数が減少する |
| `wasm_size_report_computes` | `WasmSizeReport::reduction_pct` が正しい値を返す |
| `wasm_output_correct` | WASM ビルド → wasmtime 実行でスタックに正しい結果が残る |
| `wasm_wasi_target_builds` | `--target wasm32-wasi` が有効な WASM バイト列を返す |

---

## 完了条件（PASS=5）

1. DCE により未使用関数が IRProgram から除去される
2. `WasmSizeReport` でサイズ削減率が計測できる
3. `wasm-opt` 未インストール環境でも DCE + フォールバックで動作する
4. `--target wasm32-wasi` が動作する（`_start` エクスポート付き WASM 生成）
5. `cargo test v196000` — 5/5 PASS

---

## 対象ファイル

| ファイル | 変更種別 |
|---|---|
| `fav/src/backend/wasm_dce.rs` | 新規 |
| `fav/src/backend/wasm_opt_pass.rs` | 新規 |
| `fav/src/backend/mod.rs` | `wasm_dce` / `wasm_opt_pass` を `pub mod` 追加 |
| `fav/src/driver.rs` | `WasmBuildConfig` / `cmd_build_wasm` 追加・`v196000_tests` 追加 |
| `fav/Cargo.toml` | バージョンを `19.6.0` に更新 |
| `site/content/docs/tools/wasm-opt.mdx` | 新規（ドキュメント） |

---

## 設計上の制約

- `wasm-opt` は外部バイナリ依存——CI で `wasm-opt` が未インストールでも全テスト PASS
- DCE は IR レベル（関数単位）。命令レベルの削除は行わない
- `wasm32-wasi` は既存 WASM 生成コードの拡張として実装（別パスは作らない）
- 新規 Cargo 依存は追加しない（`wasm-encoder` は既存）
