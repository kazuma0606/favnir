# Favnir v8.1.0 Tasks

Date: 2026-05-28
Theme: fav check 配線 — checker.fav を fav check パイプラインに接続

---

## Phase A: ast_lower_checker.rs（Rust AST → VMValue 変換）

- [ ] A-1: `src/middle/ast_lower_checker.rs` 新規作成
  — `v0/v1/v2/v3/vm_str/vm_bool/vm_int/vm_list/vm_record` ヘルパー関数定義
- [ ] A-2: `lower_lit(lit: &ast::Lit) -> VMValue`
  — LInt / LFloat / LStr / LBool / LUnit
- [ ] A-3: `lower_binop(op: &ast::BinOp) -> VMValue`
  — OpAdd / OpSub / ... / OpOr の 13 変換
- [ ] A-4: `lower_pat(pat: &ast::Pattern) -> VMValue`
  — PWild / PVar / PInt / PFloat / PStr / PBool / PUnit / PVariant / PVariantP
- [ ] A-5: `lower_te(te: &ast::TypeExpr) -> VMValue`
  — TeSimple / TeList / TeOption / TeResult / TeMap / TeFn + フォールバック
- [ ] A-6: `lower_arg_list(args: &[ast::Expr]) -> VMValue`
  — rev().fold() で EArgNil → EArgList チェーンを構築（先頭引数が先頭）
- [ ] A-7: `lower_arms(arms: &[ast::MatchArm]) -> VMValue`
  — EArmNil から rev().fold() で EArm チェーンを構築
- [ ] A-8: `lower_field_list(fields: &[(String, ast::Expr)]) -> VMValue`
  — EFieldNil から rev().fold() で EField チェーンを構築
- [ ] A-9: `lower_block(block: &ast::Block) -> VMValue` および `lower_stmts_and_tail`
  — Stmt::Bind → EBind / Stmt::Expr → EBlock / Stmt::Chain → EBind / 末尾 → lower_expr
- [ ] A-10: `lower_expr(expr: &ast::Expr) -> VMValue`
  — ELit / EVar / EBinOp / EAccess / ELambda / ERecordLit / EIf / EMatch / EBlock / ECall + フォールバック
- [ ] A-11: `lower_apply(func: &ast::Expr, args: &[ast::Expr]) -> VMValue`
  — Ident/FieldAccess を ECall(ns, fname, arglist) に分解
- [ ] A-12: `lower_pipeline(steps: &[ast::Expr], ...) -> VMValue`
  — Pipeline を ECall チェーンに展開
- [ ] A-13: `lower_param(p: &ast::Param) -> VMValue`
  — `{name: String, ty: TypeExpr}` VMValue レコード
- [ ] A-14: `lower_fn_def(fd: &ast::FnDef) -> VMValue`
  — is_public / name / effects / params / ret / body を VMValue レコードに変換
  — `effect_to_str(e: &ast::Effect) -> String` ヘルパー
- [ ] A-15: `lower_type_def(td: &ast::TypeDef) -> VMValue`
  — Record body → is_record=true, fields, variants=[]
  — Sum body → is_record=false, variants（VariantDef のリスト）, fields=[]
- [ ] A-16: `lower_test_def(td: &ast::TestDef) -> VMValue`
  — `{name, body}` レコード
- [ ] A-17: `lower_item(item: &ast::Item) -> Option<VMValue>`
  — IFn / IType / ITest; TrfDef / FlwDef 等は None を返す
- [ ] A-18: `pub fn lower_program(prog: &ast::Program) -> VMValue`
  — `{items: List<Item>}` レコード
- [ ] A-19: `src/middle/mod.rs` に `pub mod ast_lower_checker;` を追加
- [ ] A-20: `cargo build` で型エラーなく通ることを確認

---

## Phase B: checker_fav_runner.rs（checker.fav ローダー＋ランナー）

- [ ] B-1: `src/middle/checker_fav_runner.rs` 新規作成
- [ ] B-2: `OnceLock<Arc<FvcArtifact>>` でアーティファクトキャッシュを実装
  — `CARGO_MANIFEST_DIR/self/checker.fav` をロード → parse → compile → codegen
- [ ] B-3: `pub fn run_checker_fav(prog_vm: VMValue) -> Result<(), Vec<String>>`
  — アーティファクトの `check` 関数を VM 実行
  — `Ok(VMValue::Variant("Ok", _))` → `Ok(())`
  — `Err(VMValue::Variant("Err", Some(msg)))` → `Err(lines)`
- [ ] B-4: `pub fn msgs_to_type_errors(msgs: Vec<String>) -> Vec<TypeError>`
  — `"E0xxx: message"` → `TypeError { message, span: Span::default() }`
- [ ] B-5: `src/middle/mod.rs` に `pub mod checker_fav_runner;` を追加
- [ ] B-6: スモークテスト — `checker.fav` がロード・パース・コンパイル・実行できることを確認
  （`cargo test checker_fav_wire_self_check` 単体で先に実行して確認）

---

## Phase C: driver.rs — check_single_file の差し替え

- [ ] C-1: 既存 `check_single_file` を `check_single_file_legacy` にリネーム（fallback 用）
- [ ] C-2: 新しい `check_single_file(path, legacy: bool)` を実装
  — `legacy=false` の場合: `ast_lower_checker::lower_program` → `checker_fav_runner::run_checker_fav`
  — `legacy=true` の場合: `check_single_file_legacy` に委譲
- [ ] C-3: `cmd_check` に `--legacy-check` フラグを追加（`no_warn` と同様の処理）
  — `main.rs` の引数パースに `legacy_check` フラグを追加
- [ ] C-4: `cmd_check_dir` / `cmd_check_with_sample` にも `legacy_check` を伝播

---

## Phase D: vm.rs — Compiler.check_raw の更新

- [ ] D-1: `"Compiler.check_raw"` の型チェック部分を `checker_fav_runner::run_checker_fav` に差し替え
  — パース部分（Rust parser）は既存のまま
  — `ast_lower_checker::lower_program` でローした VMValue を渡す

---

## Phase E: driver.rs 統合テスト（3 件）

- [ ] E-1: `checker_fav_wire_valid_fn` — 型エラーなし Favnir ソースが `Ok` を返す
- [ ] E-2: `checker_fav_wire_generic_fn` — ジェネリクス関数を含むソースが通る
- [ ] E-3: `checker_fav_wire_self_check` — `checker.fav` 自身が checker.fav でチェック通過（完全ブートストラップ）

---

## Phase F: 最終確認・ドキュメント

- [ ] F-1: `fav check fav/self/checker.fav` — checker.fav 経由で no errors
- [ ] F-2: `cargo test` — 1106+ tests passing（+3 新規）
- [ ] F-3: `site/content/docs/language/self-host-checker.mdx` に v8.1.0 セクション追記
  — 「型チェッカーのセルフホスト完成」と fav check 配線の概要を記載
- [ ] F-4: このファイルを完了状態に更新
- [ ] F-5: commit

---

## 完了条件

- `fav check fav/self/checker.fav` が checker.fav（Favnir 実装）経由で動作する
- `fav check fav/self/checker.fav` が checker.fav 自身でチェック通過（完全ブートストラップ）
- 既存テストが全件通る（1106+ passing）
- Rust checker.rs への依存は `--legacy-check` フォールバックのみ

---

## 実装ノート（既知の課題）

- `lower_arg_list` / `lower_arms` / `lower_field_list` はすべて `rev().fold()` で構築する
  （先頭要素が chain の先頭になるよう）
- `OnceLock` のキャッシュは `cargo test` 並列実行に対して安全
- `Span::default()` が `TypeError` に使えない場合、`Span { start: 0, end: 0, line: 0, col: 0 }` 等の空スパンを使う
- `FavWarning` は v8.1.0 では常に空スライスを返す（警告は v8.2.0 以降で checker.fav 側に追加）
- Pipeline の展開が複雑な場合は `EVar("_unsupported_")` にフォールバックして先に進む
  （偽陰性になるが crash よりまし）
- `TestDef.description` フィールド名は `ast::TestDef` の実際のフィールド名を確認すること
