# Favnir v8.1.0 Tasks

Date: 2026-05-28
Theme: fav check 配線 — checker.fav を fav check パイプラインに接続

---

## Phase A: ast_lower_checker.rs（Rust AST → VMValue 変換）

- [x] A-1: `src/middle/ast_lower_checker.rs` 新規作成
  — `v0/v1/v2/v3/vm_str/vm_bool/vm_int/vm_list/vm_record` ヘルパー関数定義
- [x] A-2: `lower_lit(lit: &ast::Lit) -> VMValue`
  — LInt / LFloat / LStr / LBool / LUnit
- [x] A-3: `lower_binop(op: &ast::BinOp) -> VMValue`
  — OpAdd / OpSub / ... / OpOr の 13 変換
- [x] A-4: `lower_pat(pat: &ast::Pattern) -> VMValue`
  — PWild / PVar / PInt / PFloat / PStr / PBool / PUnit / PVariant / PVariantP
- [x] A-5: `lower_te(te: &ast::TypeExpr) -> VMValue`
  — TeSimple / TeList / TeOption / TeResult / TeMap / TeFn + フォールバック
- [x] A-6: `lower_arg_list(args: &[ast::Expr]) -> VMValue`
  — rev().fold() で EArgNil → EArgList チェーンを構築（先頭引数が先頭）
- [x] A-7: `lower_arms(arms: &[ast::MatchArm]) -> VMValue`
  — EArmNil から rev().fold() で EArm チェーンを構築
- [x] A-8: `lower_field_list(fields: &[(String, ast::Expr)]) -> VMValue`
  — EFieldNil から rev().fold() で EField チェーンを構築
- [x] A-9: `lower_block(block: &ast::Block) -> VMValue` および `lower_stmts_and_tail`
  — Stmt::Bind → EBind / Stmt::Expr → EBlock / Stmt::Chain → EBind / 末尾 → lower_expr
  — スタックオーバーフロー対策で反復処理に変更
- [x] A-10: `lower_expr(expr: &ast::Expr) -> VMValue`
  — ELit / EVar / EBinOp / EAccess / ELambda / ERecordLit / EIf / EMatch / EBlock / ECall + フォールバック
- [x] A-11: `lower_apply(func: &ast::Expr, args: &[ast::Expr]) -> VMValue`
  — Ident/FieldAccess を ECall(ns, fname, arglist) に分解
- [x] A-12: `lower_pipeline(steps: &[ast::Expr], ...) -> VMValue`
  — Pipeline を ECall チェーンに展開
- [x] A-13: `lower_param(p: &ast::Param) -> VMValue`
  — `{name: String, ty: TypeExpr}` VMValue レコード
- [x] A-14: `lower_fn_def(fd: &ast::FnDef) -> VMValue`
  — is_public / name / effects / params / ret / body を VMValue レコードに変換
  — `effect_to_str(e: &ast::Effect) -> String` ヘルパー
- [x] A-15: `lower_type_def(td: &ast::TypeDef) -> VMValue`
  — Record body → is_record=true, fields, variants=[]
  — Sum body → is_record=false, variants（VariantDef のリスト）, fields=[]
- [x] A-16: `lower_test_def(td: &ast::TestDef) -> VMValue`
  — `{name, body}` レコード
- [x] A-17: `lower_item(item: &ast::Item) -> Option<VMValue>`
  — IFn / IType / ITest; TrfDef / FlwDef 等は None を返す
- [x] A-18: `pub fn lower_program(prog: &ast::Program) -> VMValue`
  — `{items: List<Item>}` レコード
- [x] A-19: `src/middle/mod.rs` に `pub mod ast_lower_checker;` を追加
- [x] A-20: `cargo build` で型エラーなく通ることを確認

---

## Phase B: checker_fav_runner.rs（checker.fav ローダー＋ランナー）

- [x] B-1: `src/checker_fav_runner.rs` 新規作成（middle/ 外、main.rs にのみ宣言）
- [x] B-2: `OnceLock<Arc<FvcArtifact>>` でアーティファクトキャッシュを実装
  — `CARGO_MANIFEST_DIR/self/checker.fav` をロード → parse → compile → codegen
- [x] B-3: `pub fn run_checker_fav(prog_vm: Value) -> Result<(), Vec<String>>`
  — アーティファクトの `check` 関数を VM 実行
  — `Ok(Value::Variant("ok", _))` → `Ok(())`
  — `Err(Value::Variant("err", Some(msg)))` → `Err(lines)`
- [x] B-4: `pub fn msgs_to_type_errors(msgs: Vec<String>) -> Vec<TypeError>`
  — `"E0xxx: message"` → `TypeError { message, span: Span::default() }`
- [x] B-5: `main.rs` に `mod checker_fav_runner;` を追加
- [x] B-6: スモークテスト — `checker.fav` がロード・パース・コンパイル・実行できることを確認

---

## Phase C: driver.rs — check_single_file の差し替え

- [x] C-1: 既存 `check_single_file` を `check_single_file_legacy` にリネーム（fallback 用）
- [x] C-2: 新しい `check_single_file(path, legacy: bool)` を実装
  — `legacy=false` の場合: `ast_lower_checker::lower_program` → `checker_fav_runner::run_checker_fav`
  — `legacy=true` の場合: `check_single_file_legacy` に委譲
- [x] C-3: `cmd_check` に `--legacy-check` フラグを追加
- [x] C-4: 既存の内部呼び出しは `check_single_file_legacy` に統一（既存テストを壊さない）

---

## Phase D: vm.rs — Compiler.check_raw の更新

- [x] D-1: `"Compiler.check_raw"` の型チェック部分を `checker_fav_runner::run_checker_fav` に差し替え
  — パース部分（Rust parser）は既存のまま
  — `ast_lower_checker::lower_program` でローした VMValue を渡す

---

## Phase E: driver.rs 統合テスト（3 件）

- [x] E-1: `checker_fav_wire_valid_fn` — 型エラーなし Favnir ソースが `Ok` を返す
- [x] E-2: `checker_fav_wire_generic_fn` — ジェネリクス関数を含むソースが通る
- [x] E-3: `checker_fav_wire_self_check` — `checker.fav` 自身が checker.fav でチェック通過（完全ブートストラップ）

---

## Phase F: 最終確認・ドキュメント

- [x] F-1: `fav check fav/self/checker.fav` — checker.fav 経由で no errors
- [x] F-2: `cargo test` — 1106+ tests passing（+3 新規）
- [x] F-3: `site/content/docs/language/self-host-checker.mdx` に v8.1.0 セクション追記
  — 「型チェッカーのセルフホスト完成」と fav check 配線の概要を記載
- [x] F-4: このファイルを完了状態に更新
- [x] F-5: commit

---

## 完了条件

- `fav check fav/self/checker.fav` が checker.fav（Favnir 実装）経由で動作する ✓
- `fav check fav/self/checker.fav` が checker.fav 自身でチェック通過（完全ブートストラップ）✓
- 既存テストが全件通る（1106 passing）✓
- Rust checker.rs への依存は `--legacy-check` フォールバックのみ ✓

---

## 実装ノート（実際の実装）

- `checker_fav_runner.rs` は `middle/` 外（`src/` 直下）に配置 — lib.rs に backend が含まれないため
- `lower_stmts_and_tail` はスタックオーバーフロー対策で反復処理（rev().fold() パターン）
- `List.contains` が checker.fav で使われているが vm.rs に未登録 → 追加
- `unify_deep` / `apply_subst` に `is_fresh_var` チェック追加 — HM 推論が生成する "t0"-"tx" 型変数を認識
- `checker_fav_wire_self_check` は 64MB スタックスレッドで実行（VM の深い呼び出しチェーン対策）
- Result/Option タグは小文字 `"ok"/"err"/"some"/"none"`（vm.rs に合わせる）
- 多引数バリアント: `{_0, _1, _2, ...}` レコードペイロード
