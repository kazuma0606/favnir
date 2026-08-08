# v62.8.0 タスクリスト

Status: COMPLETE
Version: 62.8.0
Base tests: 3397
Target tests: 3399

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3397 tests passed, 0 failed を確認
  （ロードマップ記載 3396 より +1 — v62.7.0 code-reviewer 対応で `build_resolve_defaults_when_no_toml` が追加されたため）
- [x] `cranelift_aot.rs` に `validate_aot_compat` が **存在しない** ことを grep で確認
- [x] `error_catalog.rs` に `E0427` が **存在しない** ことを grep で確認
- [x] `driver.rs` に `cmd_build_aot_validate` が **存在しない** ことを grep で確認
- [x] `driver.rs` に `v62700_tests` が存在することを確認（挿入位置確認）
- [x] `cranelift_aot.rs` に `analyze_for_inlining` が存在することを確認（挿入位置確認）

---

## T1: `cranelift_aot.rs` — `contains_aot_unsupported` + `validate_aot_compat` 追加

- [x] `CraneliftBackend` の `impl` ブロック閉じ括弧の **直後**（impl 外側）に以下を追加:
  ```rust
  /// v62.8.0: IR 式が AOT 未サポート機能を含むか再帰的に検出する。
  fn contains_aot_unsupported(expr: &IRExpr) -> bool {
      match expr {
          IRExpr::Emit(_, _) => true,
          IRExpr::BinOp(_, lhs, rhs, _) => {
              contains_aot_unsupported(lhs) || contains_aot_unsupported(rhs)
          }
          IRExpr::If(cond, then_e, else_e, _) => {
              contains_aot_unsupported(cond)
                  || contains_aot_unsupported(then_e)
                  || contains_aot_unsupported(else_e)
          }
          IRExpr::Block(stmts, final_expr, _) => {
              stmts.iter().any(|s| match s {
                  IRStmt::Bind(_, e)
                  | IRStmt::LegacyBind(_, e)
                  | IRStmt::Chain(_, e)
                  | IRStmt::Yield(e)
                  | IRStmt::Return(e)
                  | IRStmt::Expr(e) => contains_aot_unsupported(e),
                  IRStmt::SeqChain { expr, .. } => contains_aot_unsupported(expr),
                  IRStmt::TrackLine(_) => false,
                  IRStmt::RefinementAssert { expr, .. } => contains_aot_unsupported(expr),
              }) || contains_aot_unsupported(final_expr)
          }
          _ => false,
      }
  }

  /// v62.8.0: AOT 互換性バリデーション — E0427 エラーメッセージリストを返す。
  pub fn validate_aot_compat(ir: &IRProgram) -> Vec<String> {
      let mut errors = Vec::new();
      for fn_def in &ir.fns {
          if contains_aot_unsupported(&fn_def.body) {
              errors.push(format!(
                  "E0427: unsupported feature in AOT mode in function `{}`",
                  fn_def.name
              ));
          }
      }
      errors
  }
  ```
- [x] `cargo build` でエラーなし

---

## T2: `error_catalog.rs` — E0427 エントリ追加

- [x] `E0426` エントリの直後（`// ── E05xx: モジュール` コメントの直前）に以下を追加:
  ```rust
  // ── E0427: AOT 未サポート機能 (v62.8.0) ──────────────────────────────────────
  ErrorEntry {
      code: "E0427",
      title: "unsupported feature in AOT mode",
      category: "build",
      description: "The pipeline contains a feature that is not supported in AOT compilation mode. \
                    Detected features: effect emission (emit expression).",
      example: "fn f() -> Unit { emit \"order_placed\" }  // E0427: emit is not supported in AOT mode",
      fix: "Use `fav run` instead of `fav build`, or remove the unsupported feature from the pipeline.",
      long_description: Some(
          "AOT (Ahead-of-Time) compilation generates a native binary from the pipeline IR.\n\
           Some Favnir features rely on VM-level dynamic dispatch and cannot be lowered to \
           native code in the current AOT backend.\n\
           \n\
           Unsupported features in AOT mode (v62.8.0):\n\
           - `emit expr` — effect emission requires VM runtime dispatch.\n\
           \n\
           To use effect-emitting stages, run the pipeline with `fav run` which uses \
           the full VM runtime. If AOT output is required, refactor the pipeline to \
           remove emit expressions and use pure data transformations only."
      ),
      suggestion: Some("Run `fav explain E0427` for details on which features are AOT-incompatible."),
  },
  ```
- [x] `cargo build` でエラーなし

---

## T3: `driver.rs` — `cmd_build_aot_validate` 追加

- [x] `cmd_build_aot_stats` の直後に以下を追加（`#[cfg(not(target_arch = "wasm32"))]` は不要 — `std::process::Command` を使用しないため）:
  ```rust
  /// v62.8.0: AOT 互換性チェック — E0427 を返す関数を報告する。
  pub fn cmd_build_aot_validate(src: &str) -> String {
      let program = match crate::frontend::parser::Parser::parse_str(src, "<validate>") {
          Ok(p) => p,
          Err(e) => return format!("parse error: {e}"),
      };
      let ir = compile_program(&program);
      let errors = crate::backend::cranelift_aot::validate_aot_compat(&ir);
      if errors.is_empty() {
          "AOT compatibility check passed.".to_string()
      } else {
          errors.join("\n")
      }
  }
  ```
- [x] `cmd_build_aot_validate` に `#[cfg(not(target_arch = "wasm32"))]` が **付与されていない** ことを確認
- [x] `cargo build` でエラーなし

---

## T4: `driver.rs` — `v62800_tests` 追加

- [x] `v62700_tests` の直前（ファイル先頭方向）に以下を挿入:
  ```rust
  // -- v62800_tests (v62.8.0) -- AOT エラーコード E0427（AOT 未サポート機能検出）--
  #[cfg(test)]
  mod v62800_tests {
      use super::*;

      #[test]
      fn aot_e0427_emit_detected() {
          // emit 式は IRExpr::Emit にコンパイルされ、AOT 未サポートとして検出される
          let src = "fn f() -> Unit { emit \"hello\" }\nfn main() -> Bool { true }";
          let result = cmd_build_aot_validate(src);
          assert!(result.contains("E0427"), "should report E0427 for emit: got {result}");
          assert!(result.contains("f"), "should name the function containing emit: got {result}");
      }

      #[test]
      fn error_catalog_has_e0427() {
          let entry = crate::error_catalog::ERROR_CATALOG
              .iter()
              .find(|e| e.code == "E0427")
              .expect("E0427 should be in error catalog");
          assert_eq!(entry.category, "build", "E0427 category should be 'build'");
          assert!(
              entry.long_description.is_some(),
              "E0427 should have long_description"
          );
      }
  }
  ```
- [x] `cargo test v62800` で 2 件 PASS

---

## T5: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0
- [x] `cargo test v62800` で 2 件 PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3399 tests passed, 0 failed を確認

---

## T6: ドキュメント更新

- [x] `versions/roadmap/roadmap-v62.1-v63.0.md` v62.8.0 セクションに実績を追記
- [x] `versions/current.md` の「進行中」を v62.8.0（3399 tests）に更新、「次」を v62.9.0 に
- [x] `CHANGELOG.md` に v62.8.0 エントリを追加
- [x] tasks.md を COMPLETE に更新（本ファイル）

---

## コードレビュー指摘対応

（spec-reviewer 指摘は実装前に修正済み）
spec-reviewer 指摘（実装前修正）:
- [HIGH] `contains_aot_unsupported` で IRStmt 全バリアント（Chain/Yield/Return/SeqChain）を明示的に処理 → 修正済み
- [HIGH] テスト名 `aot_e0427_eval_detected` → `aot_e0427_emit_detected` に変更（ロードマップにも注記追加）
- [MED] `validate_aot_compat` 挿入位置を「impl 外側・`is_aot_pure` の直後」に明確化

code-reviewer 指摘（実装後修正）:
- [HIGH] `contains_aot_unsupported` が Call/Match/Closure/Collect/FieldAccess/RecordConstruct/RecordSpread/Par/AssertSchema/CallTrfLocal の再帰なし → 全9バリアントに再帰追加（`_ => false` を撤廃）
- [HIGH] `cmd_build_aot_validate` に型チェック欠落 → `Checker::check_program` 呼び出しを compile_program の前に追加
- [MED] 正常系テスト不足 → `aot_no_emit_passes` テスト追加（3 tests → 3400 total）

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3400 passed, 0 failed（base 3397 + 3）
- 主要実装: `validate_aot_compat`（`cranelift_aot.rs`）/ `E0427`（`error_catalog.rs`）/ `cmd_build_aot_validate`（`driver.rs`）
- 完了日: 2026-08-01
