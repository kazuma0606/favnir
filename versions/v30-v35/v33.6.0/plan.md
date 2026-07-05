# v33.6.0 — Plan: WASM 最適化 確認

## アプローチ

「確認・記録」パターン（v33.1〜v33.5 踏襲）。
v19.6.0 で実装済みの WASM 最適化を `v336000_tests`（4 件）で記録する。
新規実装なし。

---

## 実装手順

### Step 1: 前提確認
- `Cargo.toml` version = `33.5.0` であること
- `cargo test` が 2516 passed, 0 failed であること
- `mod v336000_tests` が `driver.rs` に存在しないこと
- `v196000_tests` のテスト名（`wasm_dce_reduces_fn_count` 等）と重複しないこと

### Step 2: バージョン更新
- `fav/Cargo.toml`: `33.5.0` → `33.6.0`
- `fav/src/driver.rs`: `cargo_toml_version_is_33_5_0` を空スタブ化

### Step 3: テスト追加
- `fav/src/driver.rs` に `v336000_tests`（4 件）を追加
- 挿入位置: `v335000_tests` 閉じ括弧の直後、`// ── v31.7.0 tests` の前
- `use super::*` なし
- 必要な import を明示（`wasm_dce` / `WasmBuildConfig` / `WasmOptLevel` / `WasmTarget` / `Parser` / `compile_program`）

### Step 4: ドキュメント更新
- `CHANGELOG.md` 先頭に `[v33.6.0]` セクション追加
- `benchmarks/v33.6.0.json` 新規作成（暫定 2520）
- `versions/current.md` 更新

### Step 5: テスト確認
- `cargo test --bin fav v336000` — 4/4 PASS
- `cargo test` — 全件 PASS（2520 passed 想定）

### Step 6: 完了処理
- `benchmarks/v33.6.0.json` `tests_passed` を実測値で確定
- `tasks.md` を COMPLETE に更新

---

## リスク

| リスク | 対策 |
|---|---|
| v196000_tests テスト名との重複 | 逆ケース（保持 / デフォルト確認）を選ぶ |
| `ir.fns` のフィールド構造が変わっている | v196000_tests の `ir.fns.len()` 参照から構造を確認済み |
| DCE 後に `helper` の関数名がマングルされている | `.name.contains("helper")` で部分一致チェック |
| `WasmBuildConfig::default()` の変更 | driver.rs l.1840〜1849 で確認済み |

---

## 差分見積もり

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version 1 行 |
| `fav/src/driver.rs` | スタブ 1 件 + テスト 4 件（約 55 行）|
| `CHANGELOG.md` | セクション 1 つ（約 15 行）|
| `benchmarks/v33.6.0.json` | 新規（8 行）|
| `versions/current.md` | 2〜3 行更新 |
