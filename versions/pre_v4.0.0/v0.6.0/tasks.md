# Favnir v0.6.0 タスク一覧

更新日: 2026-04-29 (完了)

タスクが完了したら `[ ]` を `[x]` に変える。

---

## Phase 1: Typed IR

### IR 定義 (`src/ir.rs`)

- [x] 1-1: `src/ir.rs` を新規作成する
- [x] 1-2: `IRGlobalKind` 列挙体を定義する（`Fn(usize)` / `Builtin` / `VariantCtor`）
- [x] 1-3: `IRGlobal` 構造体を定義する（`name: String`, `kind: IRGlobalKind`）
- [x] 1-4: `IRProgram` 構造体を定義する（`globals: Vec<IRGlobal>`, `fns: Vec<IRFnDef>`）
- [x] 1-5: `IRFnDef` 構造体を定義する（`name`, `param_count`, `local_count`, `effects`, `return_ty`, `body`）
- [x] 1-6: `IRExpr` 列挙体を定義する（13 バリアント: `Lit` / `Local` / `Global` / `Call` / `Block` / `If` / `Match` / `FieldAccess` / `BinOp` / `Closure` / `Collect` / `Emit` / `RecordConstruct`）
  - 実装: `Closure` のシグネチャが `(u16, Vec<IRExpr>, Type)`（グローバルインデックス + キャプチャ式リスト）と plan.md のシグネチャと異なるが、機能的に等価
- [x] 1-7: `IRExpr::ty(&self) -> &Type` を実装する（全バリアントのアーム）
- [x] 1-8: `IRStmt` 列挙体を定義する（`Bind(u16, IRExpr)` / `Chain(u16, IRExpr)` / `Yield(IRExpr)` / `Expr(IRExpr)`）
- [x] 1-9: `IRPattern` 列挙体を定義する（`Wildcard` / `Lit` / `Bind(u16)` / `Variant` / `Record`）
- [x] 1-10: `IRArm` 構造体を定義する（`pattern: IRPattern`, `guard: Option<IRExpr>`, `body: IRExpr`）

### IR コンパイラ (`src/compiler.rs`)

- [x] 1-11: `src/compiler.rs` を新規作成する
- [x] 1-12: `CompileCtx` 構造体を定義する（`locals: Vec<HashMap<String, u16>>`, `globals: HashMap<String, u16>`, `next_slot: u16`）
- [x] 1-13: `CompileCtx::push_scope` / `pop_scope` を実装する
- [x] 1-14: `CompileCtx::define_local(name) -> u16` を実装する
- [x] 1-15: `CompileCtx::resolve_local(name) -> Option<u16>` を実装する（スコープチェーンを内側から探索）
- [x] 1-16: `CompileCtx::resolve_global(name) -> Option<u16>` を実装する
- [x] 1-17: `compile_program(program: &Program) -> IRProgram` を実装する
  - 全 `FnDef`・`TrfDef`・`FlwDef`、バリアントコンストラクタをグローバルテーブルに登録する
  - 各関数を `compile_fn_def` で変換する
- [x] 1-18: `compile_fn_def(fd: &FnDef, ctx: &mut CompileCtx) -> IRFnDef` を実装する
  - パラメータをスロット 0, 1, ... に割り当てる
  - body を `compile_block` で変換する
- [x] 1-19: `compile_expr(expr: &Expr, ctx: &mut CompileCtx) -> IRExpr` を実装する
  - `Expr::Lit` / `Ident` / `Apply` / `Block` / `If` / `Match` / `Pipeline` / `FieldAccess` / `BinOp` / `Closure` / `Collect` / `EmitExpr` / `RecordConstruct` を全て変換する
  - クロージャのキャプチャ解析（自由変数の検出）を含む
- [x] 1-20: `compile_stmt(stmt: &Stmt, ctx: &mut CompileCtx) -> IRStmt` を実装する
- [x] 1-21: `compile_pattern(pat: &Pattern, ctx: &mut CompileCtx) -> IRPattern` を実装する
- [x] 1-22: `src/main.rs` に `mod ir; mod compiler;` を追加する
- [x] 1-23: `cargo build` が通ることを確認する

---

## Phase 2: Bytecode + コードジェネレータ

### `Opcode` と `Codegen` 構造体

- [x] 2-1: `src/codegen.rs` を新規作成する
- [x] 2-2: `Opcode` 列挙体を定義する（34 命令、`#[repr(u8)]`）
  - 注: spec の hex 値と一部異なるが命令セットの機能は同等
  - `NOP` / `BUILTIN_CALL` は未実装（将来追加）
- [x] 2-3: `Constant` 列挙体を定義する（`Int(i64)` / `Float(f64)` / `Str(String)` / `Name(String)`）
- [x] 2-4: `Codegen` 構造体を定義する（`code: Vec<u8>`, `constants: Vec<Constant>`, `str_table: Vec<String>`）
- [x] 2-5: `Codegen::emit_u8` / `emit_u16` / `emit_i16` / `emit_opcode` を実装する
- [x] 2-6: `Codegen::const_idx(c: Constant) -> u16` を実装する
- [x] 2-7: `Codegen::intern_str(s: &str) -> u16` を実装する（重複登録なし）

### ジャンプパッチング

- [x] 2-8: `Codegen::emit_jump(op: Opcode) -> usize` を実装する（プレースホルダ付きで書き込み）
- [x] 2-9: `Codegen::patch_jump(pos: usize)` を実装する（現在位置から相対オフセットを計算して書き戻す）

### 式・文のコンパイル

- [x] 2-10: `emit_expr(expr: &IRExpr, cg: &mut Codegen)` を実装する（全 IRExpr バリアントを網羅）
- [x] 2-11: `emit_stmt(stmt: &IRStmt, cg: &mut Codegen)` を実装する
  - `IRStmt::Chain` → [expr] `CHAIN_CHECK offset` `STORE_LOCAL slot`（offset は chain 後の STORE_LOCAL をスキップして escape 先に飛ぶ）
- [x] 2-12: `emit_match(scrutinee: &IRExpr, arms: &[IRArm], cg: &mut Codegen)` を実装する
- [x] 2-13: `emit_pattern_test(pat: &IRPattern, fail_jumps: &mut Vec<usize>, cg: &mut Codegen)` を実装する
- [x] 2-14: `codegen_program(ir: &IRProgram) -> FvcArtifact` を実装する
  - 文字列テーブルのリマップ（`remap_string_operands`）を含む
- [x] 2-15: `src/main.rs` に `mod codegen;` を追加する
- [x] 2-16: `cargo build` が通ることを確認する

---

## Phase 3: `.fvc` フォーマット

### `FvcWriter` (`src/artifact.rs`)

- [x] 3-1: `src/artifact.rs` を新規作成する
- [x] 3-2: `FvcGlobal` 構造体を定義する（`name_idx: u32`, `kind: u8`, `fn_idx: u32`）
- [x] 3-3: `FvcFunction` 構造体を定義する（`name_idx`, `param_count`, `local_count`, `source_line`, `return_ty_str_idx`, `effect_str_idx`, `constants`, `code`）
- [x] 3-4: `FvcWriter` 構造体を定義する（`str_table`, `globals`, `functions`）
- [x] 3-5: `FvcWriter::intern(s: &str) -> u32` を実装する（重複登録なし）
- [x] 3-6: `FvcWriter::add_global` / `add_function` を実装する
- [x] 3-7: `FvcWriter::write_to(w: &mut impl Write) -> io::Result<()>` を実装する
  - Header（magic `FVC\x01`, version `0x06`, counts）→ String table → Types section → Globals → Functions の順に書く

### 定数プールのシリアライズ

- [x] 3-8: 定数プールのシリアライズ / デシリアライズを実装する
  - `Int` → tag `0x01` + i64 LE / `Float` → `0x02` + f64 LE / `Str` → `0x03` + u32 len + bytes / `Name` → `0x04` + u16 len + bytes

### `FvcReader` / `FvcArtifact`

- [x] 3-9: `ArtifactError` 列挙体を定義する（`BadMagic` / `BadVersion` / `BadSectionLayout` / `IoError` / `Utf8Error`）
- [x] 3-10: `FvcArtifact` 構造体を定義する（`str_table`, `globals`, `functions`）
- [x] 3-11: `FvcArtifact::read_from(r: &mut impl Read) -> Result<Self, ArtifactError>` を実装する
  - マジックバイト・バージョン検証、Types section の整合性検証を含む
- [x] 3-12: `FvcArtifact::fn_idx_by_name(name: &str) -> Option<usize>` を実装する
- [x] 3-13: `src/main.rs` に `mod artifact;` を追加する
- [x] 3-14: `cargo build` が通ることを確認する

---

## Phase 4: VM

### `VMError` と `VMSignal`

- [x] 4-1: `src/vm.rs` を新規作成する
- [x] 4-2: `VMError` 構造体を定義する（`message: String`, `fn_name: String`, `ip: usize`）
- [ ] 4-3: `VMSignal` 列挙体を定義する（`Normal(Value)` / `Escape(Value)`）
  - 実装: chain escape は `CHAIN_CHECK` + IP オフセットジャンプで VM ネイティブ実装（`VMSignal` 不要）
  - eval.rs の `RuntimeError.escape` アプローチも不使用。IP ジャンプ方式の方がシンプルなため当タスクは不要と判断

### `VM` と `CallFrame`

- [x] 4-4: `CallFrame` 構造体を定義する（`fn_idx: usize`, `ip: usize`, `base: usize`, `n_locals: usize`）
- [x] 4-5: `VM` 構造体を定義する（`globals: Vec<VMValue>`, `stack: Vec<VMValue>`, `frames: Vec<CallFrame>`）
  - 実装: VM は内部型 `VMValue` を持ち、インタフェースで `eval::Value` と相互変換する
- [x] 4-6: `VM::new(artifact: &FvcArtifact) -> VM` を実装する（グローバルテーブルの初期化）

### 実行ループ

- [x] 4-7: `VM::run(artifact: &FvcArtifact, fn_idx: usize, args: Vec<Value>) -> Result<Value, VMError>` の骨格を実装する
  - 初期フレームのプッシュ、args のスタック配置、ローカル変数領域の確保を含む
- [x] 4-8: 定数命令を実装する（`CONST` / `CONST_UNIT` / `CONST_TRUE` / `CONST_FALSE`）
- [x] 4-9: ローカル変数命令を実装する（`LOAD_LOCAL` / `STORE_LOCAL`）
  - `stack[frame.base + slot]` でアドレス計算
- [x] 4-10: グローバル命令を実装する（`LOAD_GLOBAL`）
- [x] 4-11: スタック操作命令を実装する（`POP` / `DUP`）
- [x] 4-12: 算術命令を実装する（`ADD` / `SUB` / `MUL` / `DIV`）
- [x] 4-13: 比較命令を実装する（`EQ` / `NEQ` / `LT` / `GT` / `LEQ` / `GEQ`）
- [x] 4-14: ジャンプ命令を実装する（`JUMP` / `JUMP_IF_FALSE` / `JUMP_IF_NOT_VARIANT`）
- [x] 4-15: `CALL` 命令を実装する
  - `CompiledFn` / `Closure` / `VariantCtor` の 3 ケースに対応
- [x] 4-16: `RETURN` 命令を実装する
  - `stack.truncate(frame.base)` でフレームのローカル変数領域を除去
- [x] 4-17: `MAKE_CLOSURE` 命令を実装する（グローバルインデックス + キャプチャをポップして `Closure` を作成）
- [ ] 4-18: `BUILD_LIST` 命令を実装する
  - 現状: Favnir にリストリテラル構文がないため不要。`collect { yield ... }` で代替
- [ ] 4-19: `BUILD_VARIANT` 命令を実装する
  - 現状: VariantCtor + `CALL` で代替実装済み（機能的に等価）
  - `GET_VARIANT_PAYLOAD` は実装済み ✓
- [x] 4-20: `GET_FIELD` 命令を実装する（`Value::Record` からフィールドを取り出す）
- [x] 4-21: `CHAIN_CHECK` 命令を実装する
  - `ok(v)` / `some(v)` → payload をプッシュして継続
  - `err(e)` / `none` → escape 値をプッシュして `frame.ip += offset`（RETURN 命令に直接ジャンプ）
- [x] 4-22: `COLLECT_BEGIN` / `COLLECT_END` / `YIELD_VALUE` 命令を実装する
  - 実装: `VM` に `collect_frames: Vec<Vec<VMValue>>` フィールドを持つ VM ネイティブ実装（eval.rs の COLLECT_STACK は不使用）
- [x] 4-23: `EMIT_EVENT` 命令を実装する（`VM.emit_log` フィールドに追加、Unit をプッシュ）
- [x] 4-24: `BUILTIN_CALL` 命令を実装する
  - 実装: `VMValue::Builtin(String)` を追加; LOAD_GLOBAL が kind=1 のグローバルを `Builtin(name)` に初期化
  - GET_FIELD で `Builtin("IO")` → `Builtin("IO.println")` と連結; CALL で `vm_call_builtin` へディスパッチ
  - 対応: `IO.println/print`, `Debug.show`, `Result.ok/err`, `Option.some/none`,
    `Int.show.show`, `Int.ord.compare`, `Int.eq.equals`, `Bool.show.show`,
    `String.*`, `List.*`（高階関数除く）, `Map.*`, `Trace.*`, `Emit.log`
  - chain escape バグ修正: `Codegen::chain_escapes` で脱出先を RETURN まで遅延パッチ
  - Db.* / Http.* は v0.7.0 で対応予定
- [x] 4-25: `MATCH_FAIL` 命令を実装する（`VMError` を返す）
- [x] 4-26: `src/main.rs` に `mod vm;` を追加する
- [x] 4-27: `cargo build` が通ることを確認する

---

## Phase 5: CLI + 統合

### `fav build`

- [x] 5-1: `main.rs` の引数パースに `"build"` サブコマンドを追加する
- [x] 5-2: `fav build <file>` の処理を実装する
  - parse → check → `compile_program` → `codegen_program` → `write_artifact_to_path`
  - 型チェックエラーがあれば非 0 終了
- [x] 5-3: `-o <out>` オプションを実装する（省略時は `.fav` → `.fvc`）
- [x] 5-4: ビルド成功時に `"built <path>"` を stdout に出力する

### `fav exec`

- [x] 5-5: `main.rs` の引数パースに `"exec"` サブコマンドを追加する
- [x] 5-6: `fav exec <file.fvc>` の処理を実装する
  - `FvcArtifact::read_from` → `fn_idx_by_name("main")` → `VM::run` → 結果表示
- [x] 5-7: `--db <path>` オプションを実装する（省略時は `:memory:`）
  - 実装: `exec` サブコマンドの引数パースに `--db <path>` を追加; `cmd_exec(_db_path)` として受け取り済み
  - Db.* ビルトインは v0.7.0 で対応予定（現在はパース・保持のみ）
- [x] 5-8: `--info` オプションを実装する
  - 実装: `artifact_info_string` — spec より詳細な情報を出力
    - `artifact: .fvc` ヘッダ、summary、globals table、function table（opcodes / consts 付き）、entry 情報を表示
- [x] 5-9: エラーメッセージを実装する
  - ファイルが見つからない（E032 相当）/ マジック不一致（E033）/ バージョン非互換（E034）/ main なし（E035）
- [x] 5-10: `fav help` の出力に `build` / `exec` コマンドの説明を追加する

---

## Phase 6: テストと例

### コードジェネレータ単体テスト

- [x] 6-1: `test_codegen_lit_int` — `IRExpr::Lit(Int(42))` → `CONST idx`（`emit_expr_for_call_writes_callee_args_and_call_count` で網羅）
- [x] 6-2: `test_codegen_lit_unit` — `IRExpr::Lit(Unit)` → `CONST_UNIT`（`codegen_program_emits_artifact_sections` で網羅）
- [x] 6-3: `test_codegen_load_local` — `LOAD_LOCAL` / `STORE_LOCAL`（`emit_stmt_bind_stores_into_local_slot` で網羅）
- [x] 6-4: `test_codegen_binop_add` — `ADD` 命令生成（`vm_calls_global_function_and_returns_result` で E2E 確認）
- [x] 6-5: `test_codegen_if` — `JUMP_IF_FALSE` + body + `JUMP` + else（`emit_jump_and_patch_jump_write_forward_offset` + E2E テストで確認）
- [x] 6-6: `test_codegen_chain` — `IRStmt::Chain` → [expr] `CHAIN_CHECK offset` `STORE_LOCAL slot`（`vm_chain_check_*` テストで確認）
- [x] 6-7: `test_codegen_collect` — `COLLECT_BEGIN` [body] `COLLECT_END`（`vm_collect_yield_builds_list` で確認）
- [x] 6-8: `test_codegen_match_wildcard` — ワイルドカードアーム（`vm_jump_if_not_variant_skips_on_mismatch` 等で確認）
- [x] 6-9: `test_codegen_match_variant` — バリアントパターン → `JUMP_IF_NOT_VARIANT` + `GET_VARIANT_PAYLOAD`（`codegen_program_remaps_variant_string_operands_into_artifact_table`）

### VM 単体テスト

- [x] 6-10: `test_vm_const_int` — `vm_runs_const_return_program` ✓
- [x] 6-11: `test_vm_add` — `vm_calls_global_function_and_returns_result` ✓
- [x] 6-12: `test_vm_load_store_local` — `vm_handles_load_and_store_local` ✓
- [x] 6-13: `test_vm_jump_if_false` — `vm_jump_if_not_variant_skips_on_mismatch` でジャンプ動作を確認 ✓
- [x] 6-14: `test_vm_call` — `vm_calls_global_function_and_returns_result` ✓
- [x] 6-15: `test_vm_chain_ok` — `vm_chain_check_unwraps_ok_payload` ✓
- [x] 6-16: `test_vm_chain_escape_err` — `vm_chain_check_escapes_err_variant` ✓
- [x] 6-17: `test_vm_chain_escape_none` — `vm_chain_check_escapes_none_variant` ✓（本版で追加）
- [x] 6-18: `test_vm_collect_yield` — `vm_collect_yield_builds_list` ✓
- [x] 6-19: `test_vm_collect_empty` — `vm_collect_empty_yields_empty_list` ✓（本版で追加）
- [x] 6-20: `test_vm_match_variant_guard_true` — `vm_match_guard_true_selects_matching_arm` ✓（本版で追加）
- [x] 6-21: `test_vm_match_variant_guard_false` — `vm_match_guard_false_falls_through_to_next_arm` ✓（本版で追加）
- [x] 6-22: `test_vm_match_fail` — `vm_match_fail_returns_error` ✓

### `fav build` / `fav exec` の統合テスト

- [x] 6-23: `test_build_hello` — `build_and_read_artifact_round_trip_for_temp_source` ✓
- [x] 6-24: `test_exec_hello` — `exec_artifact_main_runs_built_temp_source` + `file_path_build_exec_round_trip_runs_main` ✓
- [x] 6-25: `test_run_exec_match_hello` — `file_path_build_exec_round_trip_runs_main`（fav → fvc → exec の全工程）✓
- [x] 6-26: `test_run_exec_match_chain` — chain.fav の VM 統合テスト
  - 実装: `vm_integration_chain_process_and_lookup` — `process("42")` → `ok(42)`, `lookup(0)` → `none`
- [x] 6-27: `test_run_exec_match_collect` — collect.fav の VM 統合テスト
  - 実装: `vm_integration_collect_small_nums` — `small_nums()` → `[1, 2, 3]`
- [x] 6-28: `test_run_exec_match_pipe_match` — pipe_match.fav の VM 統合テスト
  - 実装: `vm_integration_pipe_match_classify` — `classify(42)` → `"positive"`
- [x] 6-29: `test_run_exec_match_generics` — generics.fav の VM 統合テスト
  - 実装: `vm_integration_generics_main` — `main()` が正常終了する
- [x] 6-30: `test_run_exec_match_cap_sort` — cap_sort.fav の VM 統合テスト
  - 実装: `vm_integration_cap_sort_min_by` — `min_by(3, 7)` → `3`

### デグレ確認

- [x] 6-31: 既存の全テストが引き続き通ることを確認する（`cargo test`）
  - v0.6.0 最終: 259 テスト全パス（v0.5.0 の 202 テスト + 57 件追加）

---

## ドキュメント

- [x] 7-1: `README.md` に `fav build` / `fav exec` / `fav exec --info` の使い方を追記する
  - 追加セクション「bytecode コンパイルと VM 実行 (v0.6.0)」: build/exec/--info 使い方、E032-E035、ビルドサイクル例
  - CLI 一覧と examples テーブルも更新
- [x] 7-2: `versions/roadmap.md` の v0.6.0 完了日を記録する
  - 完了: 2026-04-29
