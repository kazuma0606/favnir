# v62.2.0 タスクリスト

Status: COMPLETE
Version: 62.2.0
Base tests: 3384
Target tests: 3386

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3384 tests passed, 0 failed を確認
- [x] `fav/src/backend/cranelift_aot.rs` に `compile_to_binary(ir: &IRProgram, out_path: &str) -> Result<(), String>` が存在することを確認
- [x] `fav/src/backend/cranelift_aot.rs` に `link_binary` が存在することを確認
- [x] `fav/src/backend/cranelift_aot.rs` に `lower_to_object_pub` が存在することを確認（v62.1.0 で追加済み）
- [x] `fav/src/backend/fav_rt.rs` が **存在しない** ことを確認
- [x] `fav/src/backend/mod.rs` に `pub mod cranelift_aot;` が存在することを確認（`fav_rt` はまだない）
- [x] `main.rs` の `Some("build")` アームに `--link` が **存在しない** ことを確認
- [x] `driver.rs` に `cmd_build_link` が **存在しない** ことを grep で確認
- [x] `driver.rs` に `v62100_tests` が存在することを確認（挿入位置の確認）

---

## T1: `fav_rt.rs` 新規作成

- [x] `fav/src/backend/fav_rt.rs` を新規作成
  - `pub const FAV_RT_VERSION: &str = "0.1.0";`
  - `pub const FAV_RT_PRIMITIVES: &str = "fav_io_print,fav_io_panic";`
  - `pub fn fav_rt_stub_src() -> &'static str` — C スタブ文字列を返す（`fav_io_print` / `fav_io_panic` を含む）
- [x] `fav/src/backend/mod.rs` に `pub mod fav_rt;` を追加
- [x] `cargo build` でエラーなし

---

## T2: `cranelift_aot.rs` — `compile_to_binary_pub` 追加

- [x] `impl CraneliftBackend` の `lower_to_object_pub` の直前に `compile_to_binary_pub` を追加
  ```rust
  pub(crate) fn compile_to_binary_pub(ir: &IRProgram, out_path: &str) -> Result<(), String> {
      Self::compile_to_binary(ir, out_path)
  }
  ```
- [x] `cargo build` でエラーなし

---

## T3: `main.rs` — `--link` フラグ追加

- [x] `Some("build")` アーム内で `let link = false;` + `"--link"` アームを追加
- [x] `link` が true の場合は `cmd_build_link(&src_str, &out_path)` を呼ぶ分岐を追加（既存の `cmd_build_basic` 呼び出しの前に配置）
- [x] `cargo build` でエラーなし

---

## T4: `driver.rs` — `cmd_build_link` 追加

- [x] `cmd_build_basic` の直後に `pub fn cmd_build_link(src: &str, out: &str) -> String` を追加
  - `Parser::parse_str` → `compile_program` → `compile_to_binary_pub(ir, out)`
  - 成功時: `format!("Output: {} (linked binary)", out)`
  - エラー時: `format!("build error: {e}")`
- [x] `cargo build` でエラーなし

---

## T5: `driver.rs` — `v62200_tests` 追加

- [x] `v62100_tests` の直前（ファイル先頭方向）に `v62200_tests` モジュールを挿入
- [x] `use super::*;` を先頭に追加
- [x] `aot_binary_executable` テスト追加
  - `cmd_build_link("fn main() -> Bool { 1 + 2 == 3 }", "pipeline_bin")` の結果が `"parse error:"` を含まないことを確認
- [x] `aot_runtime_stub_linked` テスト追加
  - `crate::backend::fav_rt::fav_rt_stub_src()` が `"fav_io_print"` を含むことを確認
  - `crate::backend::fav_rt::fav_rt_stub_src()` が `"fav_io_panic"` を含むことを確認
- [x] `cargo test v62200` で 2 件 PASS

---

## T6: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0
- [x] `cargo test v62200` で 2 件 PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3386 tests passed, 0 failed を確認

---

## T7: ドキュメント更新

- [x] `versions/roadmap/roadmap-v62.1-v63.0.md` v62.2.0 セクションに実績を追記
  - ロードマップ記載のテスト数（3378）ではなく実績値（3386）で記録
- [x] `versions/current.md` の「進行中」を v62.2.0（3386 tests）に更新、「次」を v62.3.0 に
- [x] `CHANGELOG.md` に v62.2.0 エントリを追加
- [x] `site/content/docs/runtime/aot.mdx` — v62.9.0 で対応予定のため本バージョンでは作成しない（スコープ外）
- [x] tasks.md を COMPLETE に更新（本ファイル）

---

## コードレビュー指摘対応

- **[BUG][HIGH][false positive] `pub mod cranelift_aot` の wasm32 ガード欠如** — `cranelift-*` は `[dependencies]`（wasm32 除外なし）に登録されているため `#[cfg]` ガードは不要。`pg_pool` が `cfg(not(wasm32))` を使うのは PostgreSQL の OS 依存のためで cranelift とは無関係。対応不要。
- **[BUG][MED] `aot_binary_executable` のアサーション条件が弱い** — `#[cfg(not(target_os = "windows"))]` ブロックを追加し、非 Windows では `result.contains("Output:")` も確認するよう強化。修正済み。
- **[SECURITY][MED] `out_path` のパストラバーサル** — 既存コード（v19.2.0 `link_binary`）の問題。v62.2.0 スコープ外。
- **[BUG][LOW] `to_str().unwrap()` のパニックリスク** — 既存コード（v19.2.0 `link_binary`）。v62.2.0 スコープ外。
- **[STYLE][LOW] `///` コメントの `\` 表示崩れ** — レビュアーの grep 出力の表示崩れ（false positive）。実際のファイルは正しく `///` で記述されている。
- **[PERF][LOW] `c_wrapper_src()` が `String` を返す** — 既存コード。スコープ外。
- **[STYLE][LOW] `compile_to_binary_pub` 可視性** — ロードマップ確立パターン（`_pub` サフィックス）に従う設計。受け入れ。

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3386 passed, 0 failed（ベース 3384 + 2）
- 完了日: 2026-08-01
