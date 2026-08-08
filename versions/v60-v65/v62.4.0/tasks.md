# v62.4.0 タスクリスト

Status: COMPLETE
Version: 62.4.0
Base tests: 3388
Target tests: 3390

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3388 tests passed, 0 failed を確認
- [x] `fav/src/middle/ir.rs` の `IRFnDef`（L41-48）に `effects` フィールドが **存在しない** ことを確認
- [x] `fav/src/middle/ir.rs` の `IRExpr` enum の全 variant を確認（is_aot_pure の match 設計用）
- [x] `fav/src/backend/cranelift_aot.rs` の `lower_lit` が `Lit::Str` に対して `Err` を返すことを確認
- [x] `driver.rs` L16 に `use crate::middle::compiler::compile_program;` が存在することを確認
- [x] `cranelift_aot.rs` に `AotStats` が **存在しない** ことを確認
- [x] `driver.rs` に `cmd_build_aot_stats` が **存在しない** ことを grep で確認
- [x] `main.rs` の `Some("build")` アームに `--aot-stats` が **存在しない** ことを確認
- [x] `driver.rs` に `v62300_tests` が存在することを確認（挿入位置確認）
- [x] `cranelift_aot.rs` L15 の import に `IRFnDef` / `IRStmt` が不足している場合のみ追加が必要か確認

---

## T1: `cranelift_aot.rs` — `AotStats` + `is_aot_pure` + `analyze_for_inlining` 追加

- [x] `AotStats` 構造体を `CraneliftBackend` 定義の直前に追加
  ```rust
  pub struct AotStats {
      pub inlined: Vec<String>,
      pub dispatched: Vec<String>,
  }
  ```
- [x] `is_aot_pure(expr: &IRExpr) -> bool` をモジュールレベル private 関数として追加
  - `Lit::Str` → `false`、それ以外の `Lit` → `true`
  - `Local` → `true`
  - `BinOp(_, lhs, rhs, _)` → `is_aot_pure(lhs) && is_aot_pure(rhs)`
  - `If(cond, then_e, else_e, _)` → 3 つすべて pure
  - `Block(stmts, final_expr, _)` → stmts の IRStmt と final_expr が pure
    - `IRStmt::Bind(_, e)` / `LegacyBind(_, e)` / `Expr(e)` → `is_aot_pure(e)`
    - その他の IRStmt → `false`
  - `_ =>` **`false`**（Global / TrfRef / Call 等すべて非 pure）
- [x] `impl CraneliftBackend` の末尾（`compile_to_binary_pub` の直前）に `analyze_for_inlining` を追加
  ```rust
  pub(crate) fn analyze_for_inlining(ir: &IRProgram) -> AotStats {
      let mut inlined = Vec::new();
      let mut dispatched = Vec::new();
      for fn_def in &ir.fns {
          if is_aot_pure(&fn_def.body) {
              inlined.push(fn_def.name.clone());
          } else {
              dispatched.push(fn_def.name.clone());
          }
      }
      AotStats { inlined, dispatched }
  }
  ```
- [x] `cranelift_aot.rs` L15 の import に不足があれば追加（`IRFnDef` / `IRStmt` / `Lit`）
- [x] `cargo build` でエラーなし

---

## T2: `driver.rs` — `cmd_build_aot_stats` 追加

- [x] `cmd_build_link_target` の直後に追加:
  ```rust
  pub fn cmd_build_aot_stats(src: &str) -> String {
      let program = match crate::frontend::parser::Parser::parse_str(src, "<build>") {
          Ok(p) => p,
          Err(e) => return format!("parse error: {e}"),
      };
      let ir = compile_program(&program);
      let stats = crate::backend::cranelift_aot::CraneliftBackend::analyze_for_inlining(&ir);
      format!(
          "AOT stats: {} inlined, {} dispatched",
          stats.inlined.len(),
          stats.dispatched.len()
      )
  }
  ```
- [x] `cargo build` でエラーなし

---

## T3: `main.rs` — `--aot-stats` フラグ追加

- [x] `let mut aot_stats = false;` を変数宣言部（`let mut link = false;` の近く）に追加
- [x] `Some("build")` アームのループ内に `"--aot-stats"` アームを追加
- [x] `if aot_stats { ... } else if link { ... }` の分岐を追加
  - `aot_stats` ブランチ: `file` 変数を取り出し、ファイルを読んで `cmd_build_aot_stats(&src)` を呼ぶ
    （`--link` ブランチと同じファイル読み込みパターン `std::fs::read_to_string` を使用）
- [x] `cargo build` でエラーなし

---

## T4: `driver.rs` — `v62400_tests` 追加

- [x] `v62300_tests` の直前（ファイル先頭方向）に `v62400_tests` モジュールを挿入
- [x] `use super::*;` を先頭に追加
- [x] `aot_pure_stage_inlined` テスト追加:
  - ソース: `"fn add(a: Int, b: Int) -> Int { a + b }\nfn main() -> Bool { 1 + 2 == 3 }"`
  - `analyze_for_inlining` の `stats.inlined` に `"add"` が含まれることを確認
  - `!stats.inlined.is_empty()` を確認
- [x] `aot_effectful_stage_not_inlined` テスト追加:
  - ソース: `"fn greeting() -> String { \"hello\" }\nfn main() -> Bool { 1 + 2 == 3 }"`
  - `stats.dispatched` に `"greeting"` が含まれることを確認（`Lit::Str` → not pure）
  - `stats.inlined` に `"greeting"` が含まれないことを確認
- [x] `cargo test v62400` で 2 件 PASS

---

## T5: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0
- [x] `cargo test v62400` で 2 件 PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3390 tests passed, 0 failed を確認

---

## T6: ドキュメント更新

- [x] `versions/roadmap/roadmap-v62.1-v63.0.md` v62.4.0 セクションに実績を追記
  - `effects` フィールド廃止により `is_aot_pure`（IR 構造解析）で代替した経緯を記載
  - 削減バイト数表示は v62.9.0 に後送りした旨を記載
- [x] `versions/current.md` の「進行中」を v62.4.0（3390 tests）に更新、「次」を v62.5.0 に
- [x] `CHANGELOG.md` に v62.4.0 エントリを追加
- [x] `site/content/docs/runtime/aot.mdx` — v62.9.0 で対応予定のため本バージョンでは作成しない（スコープ外）
- [x] tasks.md を COMPLETE に更新（本ファイル）

---

## コードレビュー指摘対応

- **[MED] `IRStmt::TrackLine` を `_ => false` で誤判定** — `TrackLine(_) => true`（行番号マーカーは副作用なし）を追加。`RefinementAssert { expr, .. } => is_aot_pure(expr)` も追加して再帰的に判定するよう修正。修正済み。
- **[LOW] `aot_pure_stage_inlined` テストの冗長な assert** — `!stats.inlined.is_empty()` を削除し、`main` 関数もインライン化されているかを確認する assert に変更（`stats.inlined.contains(&"main".to_string())`）。修正済み。
- **[LOW] `--aot-stats` の exit code 非対称** — `parse error:` 始まりの結果を stderr + exit 1 で返すよう修正。他の `cmd_build_*` ブランチと対称化。修正済み。

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3390 passed, 0 failed（ベース 3388 + 2）
- 追記: `IRFnDef.effects` 廃止により `is_aot_pure` IR 構造解析で代替実装
- 削減バイト数表示は v62.9.0 に後送り
- 完了日: 2026-08-01
