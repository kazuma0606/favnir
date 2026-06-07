# Favnir v12.3.0 Tasks

Date: 2026-06-07
Theme: `bind` を真の monadic bind に修正（`--legacy` モード）

---

## Phase A — 現状把握

- [x] A-1: `fav/src/backend/vm.rs` の `IRStmt::Bind` 処理箇所を特定
  - bytecode VM: `Bind` → `StoreLocal`、`Chain` → `ChainCheck + StoreLocal`
  - `VMValue::Variant` の tag 文字列は小文字: `"ok"` / `"err"` / `"some"` / `"none"`
- [x] A-2: `IRStmt::Chain` の処理を確認（`ChainCheck` opcode が escape offset 付きで実装）
- [x] A-3: `--legacy` フラグ → `build_artifact_legacy` → IR transform → `codegen_program` の流れを確認

---

## Phase B — 実装アプローチ変更（IRStmt::LegacyBind + LegacyBindCheck opcode）

当初の VMError アプローチではなく、以下の方針で実装:
- `IRStmt::LegacyBind(u16, IRExpr)` を IR に追加（型チェック不要・runtime 検出）
- `Opcode::LegacyBindCheck = 0x35` を bytecode に追加（非 Result 値は pass-through）
- `apply_legacy_bind_semantics` で `Bind` → `LegacyBind` に変換（型情報不要）

- [x] B-1: `middle/ir.rs` に `IRStmt::LegacyBind(u16, IRExpr)` 追加
- [x] B-2: `backend/codegen.rs` に `Opcode::LegacyBindCheck = 0x35` 追加
- [x] B-3: `emit_stmt` に `IRStmt::LegacyBind` ケース追加（`LegacyBindCheck + StoreLocal`）

---

## Phase C — VM に LegacyBindCheck ハンドラ追加

- [x] C-1: `backend/vm.rs` に `LegacyBindCheck` ハンドラ追加
  - `ok(v)` / `some(v)` → unwrap して push、StoreLocal へ続行
  - `err(e)` / `none` → 値を push し直して offset ジャンプ（escape）
  - その他（Int, String, Unit, etc.）→ そのまま push、StoreLocal へ続行

---

## Phase D — driver.rs: IR transform + legacy artifact ビルド

- [x] D-1: `legacy_transform_expr` / `legacy_transform_stmt` 追加（全 IRExpr 再帰）
- [x] D-2: `apply_legacy_bind_semantics` 追加（全 IRFnDef の body を変換）
- [x] D-3: `build_artifact_legacy` 追加（`compile_program` → transform → `codegen_program`）
- [x] D-4: `cmd_run` の legacy パスで `build_artifact_legacy` を使用

---

## Phase E — 全 IRStmt パターンマッチ更新

- [x] E-1: `driver.rs::collect_tracklines_in_expr` — `LegacyBind` 追加
- [x] E-2: `driver.rs::remap_ir_stmt` — `LegacyBind` 追加
- [x] E-3: `driver.rs::opcode_info` — `LegacyBindCheck` 追加（幅 3 bytes）
- [x] E-4: `backend/wasm_codegen.rs` — 全 5 箇所に `LegacyBind` 追加（UnsupportedExpr として）
- [x] E-5: `middle/ir.rs::collect_stmt_deps` — `LegacyBind` 追加

---

## Phase F — driver.rs: v12300_tests モジュール追加

- [x] F-1: 正常系テスト（3件）
  - [x] `bind_ok_unwraps_value_legacy` — `bind x <- Result.ok(42)` → x = 42（Int）
  - [x] `bind_non_result_unchanged_legacy` — `bind x <- 42` → x = 42（pass-through）
  - [x] `bind_chain_same_ok_semantics` — bind と chain の Ok 結果が同一
- [x] F-2: 短絡系テスト（3件）
  - [x] `bind_propagates_err_legacy` — `bind _ <- Result.err("fail")` → 結果が `Err`
  - [x] `bind_err_skips_subsequent_binds` — Err 後の bind は実行されない（42 に到達しない）
  - [x] `bind_err_stops_seq_pipeline` — seq pipeline の Fail stage が Err → Echo が Err を通過
- [x] F-3: 後方互換確認（1件）
  - [x] `favnir_pipeline_bind_unchanged` — default mode で `bind x <- Ok(42)` → x = Ok(42)（match 経由で確認）
- [x] F-4: バージョン確認（1件）
  - [x] `version_is_12_3_0` — `CARGO_PKG_VERSION == "12.3.0"`
- [x] F-5: `cargo test v12300 -- --nocapture` — 8 件通過確認

---

## Phase G — 全テスト通過確認

- [x] G-1: `cargo test` — 1370 件通過（+8 件）

---

## Phase H — バージョン更新 + コミット

- [x] H-1: `fav/Cargo.toml` version → `"12.3.0"`
- [x] H-2: `cargo build` で `Cargo.lock` 更新
- [x] H-3: `git commit & push`

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `--legacy` モードで `bind x <- Ok(v)` → `x = v`（unwrap） | ✅ |
| `--legacy` モードで `bind x <- Err(e)` → stage が即座に Err で停止 | ✅ |
| Err 後の後続 `bind` は実行されない | ✅ |
| seq pipeline で前段 Err → Err が伝播 | ✅ |
| Favnir pipeline モード（デフォルト）の `bind` は単純代入のまま | ✅ |
| `cargo test v12300` 8 件通過 | ✅ |
| `cargo test` 全通過（1370 件） | ✅ |

## 技術的知見（デバッグ記録）

1. **`IRExpr::ty()` は `Type::Unknown` を返す**: Rust compiler は型推論なし。`Result.ok(42)` の IR 型は `Type::Unknown` のため、コンパイル時に型チェックで Result を検出できない。

2. **解決策: `IRStmt::LegacyBind` + `LegacyBindCheck` opcode**: 型チェック不要。`LegacyBindCheck` は runtime に値を見て分岐（ok/err → 処理、それ以外 → pass-through）。これにより `bind x <- 42` のような非 Result bind も正しく動作する。

3. **`ChainCheck` と `LegacyBindCheck` の違い**: `ChainCheck` は非 Result 値でエラー。`LegacyBindCheck` は非 Result 値を pass-through するため、`bind x <- 42` のような単純代入も機能する。
