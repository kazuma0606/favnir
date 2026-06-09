# v13.3.0 Tasks — HttpClient / Io / Env interface + compiler.fav E0001 修正

Date: 2026-06-10
Branch: feat/v13-capability-context

---

## Phase A — compiler.fav E0001 修正

- [x] A-1: `fav/src/middle/ast_lower_checker.rs` に `Collect` ケース追加
  - `lower_expr` の fallback 直前に `ast::Expr::Collect(block, _) => v1("ECollect", lower_block(block))`
- [x] A-2: `fav/self/checker.fav` に `ECollect` ケース追加
  - `infer_hm` の `ECall` ケース直後に `ECollect(inner) => ...` ("Unknown" を返す)
- [x] A-3: `cargo run --bin fav -- check self/compiler.fav` → exit 0 確認

---

## Phase B — 組み込み interface 登録

- [x] B-1: `fav/src/middle/checker.rs` に `HttpClient` 登録（4 メソッド）
- [x] B-2: `fav/src/middle/checker.rs` に `Io` 登録（3 メソッド）
- [x] B-3: `fav/src/middle/checker.rs` に `Env` 登録（2 メソッド）
- [x] B-4: `map_ss` ヘルパー（`Type::Map(Box::new(s()), Box::new(s()))`）の追加確認

---

## Phase C — Rune ファイル作成

- [x] C-1: `runes/http/http_client_impl.fav` — HttpClientImpl implements HttpClient
- [x] C-2: `runes/io/io_impl.fav` — IoImpl implements Io
- [x] C-3: `runes/io/io_capture.fav` — IoCapture (test stub) implements Io
- [x] C-4: `runes/env/env_impl.fav` — EnvImpl implements Env
- [x] C-5: `runes/env/mock_env.fav` — MockEnv implements Env

---

## Phase D — W009 追加

- [x] D-1: vm.rs で `IO.env_require_raw` / `IO.env_get_raw` の存在確認
- [x] D-2: `fav/src/lint.rs` の `DEPRECATED_RUNE_CALLS` に IO.* 7 件追加
- [x] D-3: `fav/src/lint.rs` の `DEPRECATED_RUNE_CALLS` に Http.* 2 件追加

---

## Phase E — テスト

- [x] E-1: `v132000_tests` の `version_is_13_2_0` をコメントアウト
- [x] E-2: `v133000_tests` モジュール作成（`fav/src/driver.rs`）
  - [x] `version_is_13_3_0`
  - [x] `collect_expr_lowers_to_ecollect`（collect 式を含む .fav で check）
  - [x] `http_client_interface_registered`
  - [x] `io_interface_registered`
  - [x] `env_interface_registered`
  - [x] `io_interface_println_typecheck`（`fn f(ctx: Io) { ctx.println("x") }`）
  - [x] `env_interface_require_typecheck`（`fn f(ctx: Env) { ctx.require("K") }`）
  - [x] `w009_io_println_deprecated`
  - [x] `w009_http_get_deprecated`
  - [x] `compiler_fav_check_passes`（`Checker::check_program` で self/compiler.fav を確認）
- [x] E-3: `cargo test` 全件パス確認

---

## Phase F — バージョンバンプ + コミット

- [x] F-1: `fav/Cargo.toml` → `version = "13.3.0"`
- [x] F-2: 全テストパス (`cargo test 2>&1 | tail -5`)
- [x] F-3: `git add` + `git commit -m "feat: v13.3.0 — HttpClient/Io/Env interface + compiler.fav collect fix"`

---

## 完了条件

- `fav check self/compiler.fav` → 0 errors
- `cargo test` 全件パス（v13.2.0 より増加）
- HttpClient / Io / Env が `InterfaceRegistry` に登録済み
- W009 が IO.println / Http.get_raw で発火
- Rune ファイル 5 件作成済み
