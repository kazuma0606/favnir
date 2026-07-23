# Plan: v49.6.0 — WASM / Python transpiler 互換確認

## 作業順序

### Step 1: `v496000_tests` 追加

`v495000_tests` の直前に挿入（2テスト）:

#### `python_emit_return_stmt`
- `crate::emit_python::emit_python_str` に `return` 文を含む Favnir ソースを渡す
- 出力 Python コードに `"return"` が含まれることを確認

#### `wasm_compat_return_stmt`
- `include_str!("backend/wasm_codegen.rs")` で wasm_codegen.rs ソースを読み込む
- `"IRStmt::Return"` の match arm が存在することを確認

**パス確認**:
- `crate::emit_python` — driver.rs と同クレートの `src/emit_python.rs`（`pub fn emit_python_str`）
- `include_str!("backend/wasm_codegen.rs")` — `src/driver.rs` から `src/backend/wasm_codegen.rs`

### Step 2: `Cargo.toml` version 更新

`"49.5.0"` → `"49.6.0"`

### Step 3: 完了処理

- `cargo test` 3081 passed を確認
- `cargo clippy -- -D warnings` クリーン確認
- `CHANGELOG.md` に v49.6.0 エントリ追加
- `versions/current.md` 更新（v49.6.0・3081 tests・進行中 v49.7.0）
- `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.6.0 実績を記入
- `tasks.md` を COMPLETE に更新

---

## 変更ファイル一覧

| ファイル | 変更種別 |
|---|---|
| `fav/src/driver.rs` | `v496000_tests` 追加（2テスト）|
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v49.6.0 エントリ |
| `versions/current.md` | バージョン更新 |
| `versions/roadmap/roadmap-v49.1-v50.0.md` | 実績記入 |
| `versions/v45-v50/v49.6.0/tasks.md` | COMPLETE 更新 |

## 変更しないファイル

| ファイル | 理由 |
|---|---|
| `fav/src/emit_python.rs` | `Stmt::Return` は既に完全実装済み |
| `fav/src/backend/wasm_codegen.rs` | MVP 制限として `UnsupportedExpr` を返す設計を維持 |
