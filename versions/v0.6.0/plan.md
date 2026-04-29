# Favnir v0.6.0 実装計画

更新日: 2026-04-29

---

## Phase 1: Typed IR (`src/ir.rs` + `src/compiler.rs`)

### IR 定義 (`src/ir.rs`)

v0.4.0 の `Type` をそのまま利用する型注釈付き中間表現。各ノードに型が付く。

```rust
// src/ir.rs
use crate::ast::{BinOp, Effect, Lit};
use crate::checker::Type;

pub struct IRProgram {
    pub globals: Vec<IRGlobal>,
    pub fns:     Vec<IRFnDef>,
}

pub struct IRGlobal {
    pub name: String,
    pub kind: IRGlobalKind,
}

pub enum IRGlobalKind {
    Fn(usize),      // fns テーブルのインデックス
    Builtin,        // 組み込み（実行時に解決）
    VariantCtor,    // ADT コンストラクタ
}

pub struct IRFnDef {
    pub name:        String,
    pub param_count: usize,
    pub local_count: usize,
    pub effects:     Vec<Effect>,
    pub return_ty:   Type,
    pub body:        IRExpr,
}

pub enum IRExpr {
    Lit(Lit, Type),
    Local(u16, Type),
    Global(u16, Type),
    Call(Box<IRExpr>, Vec<IRExpr>, Type),
    Block(Vec<IRStmt>, Box<IRExpr>, Type),
    If(Box<IRExpr>, Box<IRExpr>, Box<IRExpr>, Type),
    Match(Box<IRExpr>, Vec<IRArm>, Type),
    FieldAccess(Box<IRExpr>, String, Type),
    BinOp(BinOp, Box<IRExpr>, Box<IRExpr>, Type),
    Closure(Vec<u16>, Box<IRExpr>, Type),
    Collect(Box<IRExpr>, Type),
    Emit(Box<IRExpr>, Type),
    RecordConstruct(Vec<(String, IRExpr)>, Type),
}

impl IRExpr {
    pub fn ty(&self) -> &Type {
        match self {
            IRExpr::Lit(_, t)               => t,
            IRExpr::Local(_, t)             => t,
            IRExpr::Global(_, t)            => t,
            IRExpr::Call(_, _, t)           => t,
            IRExpr::Block(_, _, t)          => t,
            IRExpr::If(_, _, _, t)          => t,
            IRExpr::Match(_, _, t)          => t,
            IRExpr::FieldAccess(_, _, t)    => t,
            IRExpr::BinOp(_, _, _, t)       => t,
            IRExpr::Closure(_, _, t)        => t,
            IRExpr::Collect(_, t)           => t,
            IRExpr::Emit(_, t)              => t,
            IRExpr::RecordConstruct(_, t)   => t,
        }
    }
}

pub enum IRStmt {
    Bind(u16, IRExpr),
    Chain(u16, IRExpr),
    Yield(IRExpr),
    Expr(IRExpr),
}

pub struct IRArm {
    pub pattern: IRPattern,
    pub guard:   Option<IRExpr>,
    pub body:    IRExpr,
}

pub enum IRPattern {
    Wildcard,
    Lit(Lit),
    Bind(u16),
    Variant(String, Option<Box<IRPattern>>),
    Record(Vec<(String, IRPattern)>),
}
```

### IR コンパイラ (`src/compiler.rs`)

AST → IR の変換。`CompileCtx` が変数スロットのマップを管理する。

```rust
// src/compiler.rs
use std::collections::HashMap;
use crate::ast::*;
use crate::checker::Type;
use crate::ir::*;

pub struct CompileCtx {
    // ローカル変数名 → スロット番号
    locals:  Vec<HashMap<String, u16>>,
    // グローバルテーブル（関数名・組み込み名 → インデックス）
    globals: HashMap<String, u16>,
    // 次に割り当てるスロット番号
    next_slot: u16,
}

impl CompileCtx {
    pub fn push_scope(&mut self) { self.locals.push(HashMap::new()); }
    pub fn pop_scope(&mut self)  { self.locals.pop(); }

    pub fn define_local(&mut self, name: String) -> u16 {
        let slot = self.next_slot;
        self.next_slot += 1;
        self.locals.last_mut().unwrap().insert(name, slot);
        slot
    }

    pub fn resolve_local(&self, name: &str) -> Option<u16> {
        self.locals.iter().rev().find_map(|s| s.get(name).copied())
    }

    pub fn resolve_global(&self, name: &str) -> Option<u16> {
        self.globals.get(name).copied()
    }
}

pub fn compile_program(program: &Program) -> IRProgram { /* ... */ }
pub fn compile_fn_def(fd: &FnDef, ctx: &mut CompileCtx) -> IRFnDef { /* ... */ }
pub fn compile_expr(expr: &Expr, ctx: &mut CompileCtx) -> IRExpr { /* ... */ }
pub fn compile_stmt(stmt: &Stmt, ctx: &mut CompileCtx) -> IRStmt { /* ... */ }
pub fn compile_pattern(pat: &Pattern, ctx: &mut CompileCtx) -> IRPattern { /* ... */ }
```

主な処理の流れ:

1. **グローバルテーブルの構築**: 全 `FnDef`、組み込み関数、バリアントコンストラクタをグローバルに登録する。
2. **関数コンパイル**: 各 `FnDef` を `compile_fn_def` で変換する。パラメータをスロット 0, 1, 2... に割り当てる。
3. **式の変換**: `Expr::Ident(name)` → `resolve_local(name)` → `IRExpr::Local` / `resolve_global(name)` → `IRExpr::Global`。
4. **クロージャのキャプチャ解析**: クロージャが参照する外部スコープのスロットを `captures: Vec<u16>` として記録する。
5. **型の引き渡し**: 各 IR ノードに `Type::Unknown` を仮セットする（v0.6.0 では型チェックは checker が担い、IR は型情報を省略可能とする）。

---

## Phase 2: Bytecode + コードジェネレータ (`src/codegen.rs`)

### `Opcode` 列挙体

```rust
// src/codegen.rs
#[repr(u8)]
pub enum Opcode {
    Nop           = 0x00,
    Const         = 0x01,   // idx: u16
    ConstUnit     = 0x02,
    ConstTrue     = 0x03,
    ConstFalse    = 0x04,
    LoadLocal     = 0x10,   // slot: u16
    StoreLocal    = 0x11,   // slot: u16
    LoadGlobal    = 0x12,   // idx: u16
    Pop           = 0x20,
    Dup           = 0x21,
    Add           = 0x30,
    Sub           = 0x31,
    Mul           = 0x32,
    Div           = 0x33,
    Eq            = 0x34,
    Neq           = 0x35,
    Lt            = 0x36,
    Gt            = 0x37,
    Leq           = 0x38,
    Geq           = 0x39,
    Jump          = 0x40,   // offset: i16
    JumpIfFalse   = 0x41,   // offset: i16
    JumpIfNotVariant = 0x42,// name_idx: u16, offset: i16
    Call          = 0x50,   // n_args: u8
    Return        = 0x51,
    MakeClosure   = 0x52,   // fn_idx: u16, n_cap: u8
    BuildList     = 0x60,   // n: u16
    BuildRecord   = 0x61,   // n: u8, names_idx: u16
    BuildVariant  = 0x70,   // name_idx: u16, has_payload: u8
    GetVariantPayload = 0x71,
    GetField      = 0x72,   // name_idx: u16
    ChainCheck    = 0x80,   // escape_offset: i16
    CollectBegin  = 0x81,
    CollectEnd    = 0x82,
    YieldValue    = 0x83,
    EmitEvent     = 0x90,
    BuiltinCall   = 0x91,   // ns_idx: u16, method_idx: u16, n_args: u8
    MatchFail     = 0xFF,
}
```

### `Codegen` 構造体

```rust
pub struct Codegen {
    pub code:      Vec<u8>,
    pub constants: Vec<Constant>,
    pub str_table: Vec<String>,   // グローバル文字列テーブルへの参照
}

pub enum Constant {
    Int(i64),
    Float(f64),
    Str(String),
    Name(String),
}

impl Codegen {
    fn emit_u8(&mut self, b: u8)          { self.code.push(b); }
    fn emit_u16(&mut self, v: u16)        { self.code.extend_from_slice(&v.to_le_bytes()); }
    fn emit_i16(&mut self, v: i16)        { self.code.extend_from_slice(&v.to_le_bytes()); }
    fn emit_opcode(&mut self, op: Opcode) { self.emit_u8(op as u8); }

    fn const_idx(&mut self, c: Constant) -> u16 {
        let idx = self.constants.len() as u16;
        self.constants.push(c);
        idx
    }

    fn intern_str(&mut self, s: &str) -> u16 {
        if let Some(i) = self.str_table.iter().position(|x| x == s) {
            return i as u16;
        }
        let i = self.str_table.len() as u16;
        self.str_table.push(s.to_string());
        i
    }
}
```

### ジャンプパッチング

前向きジャンプ（`if` / `match` の分岐先未確定）はプレースホルダを書き込み、後からパッチする。

```rust
fn emit_jump(&mut self, op: Opcode) -> usize {
    self.emit_opcode(op);
    let pos = self.code.len();
    self.emit_i16(0);   // プレースホルダ
    pos
}

fn patch_jump(&mut self, pos: usize) {
    let offset = (self.code.len() as isize - pos as isize - 2) as i16;
    let bytes = offset.to_le_bytes();
    self.code[pos]     = bytes[0];
    self.code[pos + 1] = bytes[1];
}
```

### 主要な emit 関数

```
emit_expr(expr: &IRExpr, cg: &mut Codegen)
emit_stmt(stmt: &IRStmt, cg: &mut Codegen)
emit_match(scrutinee: &IRExpr, arms: &[IRArm], cg: &mut Codegen)
emit_pattern_test(pat: &IRPattern, fail_label: &mut Vec<usize>, cg: &mut Codegen)
```

### `match` のコンパイル

```
scrutinee を push
各アームに対して:
  DUP                           ; scrutinee のコピーを残す
  JUMP_IF_NOT_VARIANT "ok" pos  ; バリアントでなければ next_arm へ
  GET_VARIANT_PAYLOAD
  STORE_LOCAL slot_n
  [guard があれば guard をコンパイル]
  [guard が false なら next_arm へ: JUMP_IF_FALSE]
  POP                           ; scrutinee を捨てる
  [body をコンパイル]
  JUMP arm_end
next_arm:
  ...
MATCH_FAIL                      ; 全アーム失敗
arm_end:
```

### `chain` のコンパイル

```
[expr をコンパイル]
CHAIN_CHECK escape_offset       ; ok/some → payload push、err/none → escape_offset にジャンプ
STORE_LOCAL slot                ; payload をスロットに格納
...（関数末尾）
escape_label:                   ; CHAIN_CHECK のジャンプ先（関数末尾のRETURNにパッチ）
RETURN                          ; err/none の値を直接返す
```

---

## Phase 3: `.fvc` フォーマット (`src/artifact.rs`)

### `FvcWriter`

`.fvc` ファイルをバイト列として構築する。

```rust
pub struct FvcWriter {
    str_table: Vec<String>,
    globals:   Vec<FvcGlobal>,
    functions: Vec<FvcFunction>,
}

pub struct FvcGlobal {
    pub name_idx: u32,
    pub kind: u8,       // 0=Fn, 1=Builtin, 2=VariantCtor
    pub fn_idx: u32,
}

pub struct FvcFunction {
    pub name_idx:    u32,
    pub param_count: u8,
    pub local_count: u16,
    pub source_line: u32,
    pub return_ty_str_idx: u32,
    pub effect_str_idx:    u32,
    pub constants:   Vec<Constant>,
    pub code:        Vec<u8>,
}

impl FvcWriter {
    pub fn intern(&mut self, s: &str) -> u32 { /* ... */ }
    pub fn add_global(&mut self, g: FvcGlobal) { /* ... */ }
    pub fn add_function(&mut self, f: FvcFunction) { /* ... */ }
    pub fn write_to(&self, w: &mut impl Write) -> io::Result<()> { /* ... */ }
}
```

`write_to` の処理順:
1. Header (16 bytes) を書く
2. String table を書く
3. Types section（各関数の return_ty_str / effect_str）を書く
4. Globals table を書く
5. Functions section（定数プール + bytecode）を書く

### `FvcReader`

```rust
pub struct FvcArtifact {
    pub str_table: Vec<String>,
    pub globals:   Vec<FvcGlobal>,
    pub functions: Vec<FvcFunction>,
}

impl FvcArtifact {
    pub fn read_from(r: &mut impl Read) -> Result<Self, ArtifactError> { /* ... */ }
    pub fn fn_idx_by_name(&self, name: &str) -> Option<usize> { /* ... */ }
}
```

エラーケース:
- マジックバイト不一致 → E033
- バージョン非互換 → E034
- `main` 関数なし → E035（`fav exec` 時）

---

## Phase 4: VM (`src/vm.rs`)

### データ構造

```rust
pub struct VM {
    pub globals: Vec<Value>,
    stack:       Vec<Value>,
    frames:      Vec<CallFrame>,
}

struct CallFrame {
    fn_idx:   usize,
    ip:       usize,
    base:     usize,   // スタック上のローカル変数の開始位置
    n_locals: usize,   // このフレームのローカル変数数
}
```

ローカル変数はスタック上に直接配置する:

```
stack: [ param0 | param1 | local2 | local3 | ... | (作業領域) ]
              ↑ frame.base
```

`local[slot]` = `stack[frame.base + slot]`

### `VMSignal` と ChainEscape

```rust
enum VMSignal {
    Normal(Value),
    Escape(Value),
}
```

`CHAIN_CHECK` がスタックトップを検査:
- `Variant("ok", Some(v))` / `Variant("some", Some(v))` → `v` をプッシュして継続
- `Variant("err", _)` / `Variant("none", _)` → `VMSignal::Escape(val)` として現フレームから即 RETURN
- `CALL` を処理する際: 呼び出し先が `VMSignal::Escape(v)` を返したら、そのまま `Ok(VMSignal::Escape(v))` として上位フレームに返す
- 関数呼び出し境界(`eval_apply` 相当)でのみ `Escape` → `Normal` に変換する

### 実行ループ

```rust
pub fn run(&mut self, artifact: &FvcArtifact, fn_idx: usize, args: Vec<Value>)
    -> Result<Value, VMError>
{
    self.frames.push(CallFrame { fn_idx, ip: 0, base: self.stack.len(), n_locals: ... });
    // args をスタックにプッシュ
    for arg in args { self.stack.push(arg); }
    // ローカル変数スロットを Unit で初期化（locals - params 個）

    loop {
        let frame = self.frames.last_mut().unwrap();
        let op = artifact.functions[frame.fn_idx].code[frame.ip];
        frame.ip += 1;

        match op {
            0x01 /* CONST */ => { /* 定数プールから push */ }
            0x10 /* LOAD_LOCAL */  => { /* stack[base + slot] を push */ }
            0x11 /* STORE_LOCAL */ => { /* スタックトップを stack[base + slot] に書く */ }
            0x50 /* CALL */        => { /* 新フレームを push、ループ継続 */ }
            0x51 /* RETURN */      => {
                let ret = self.stack.pop().unwrap();
                // フレームのローカル変数領域をスタックから除去
                self.stack.truncate(frame.base);
                self.frames.pop();
                if self.frames.is_empty() { return Ok(ret); }
                self.stack.push(ret);
            }
            0x80 /* CHAIN_CHECK */ => {
                let escape_offset = read_i16(...);
                match self.stack.last() {
                    Some(Value::Variant(tag, payload)) if tag == "ok" || tag == "some" => {
                        let v = payload.as_deref().cloned().unwrap_or(Value::Unit);
                        self.stack.pop();
                        self.stack.push(v);
                    }
                    _ => {
                        // frame.ip += escape_offset (RETURN へジャンプ)
                        frame.ip = (frame.ip as isize + escape_offset as isize) as usize;
                    }
                }
            }
            // ...
        }
    }
}
```

### 組み込み関数の呼び出し (`BUILTIN_CALL`)

eval.rs の `eval_builtin` に相当するロジックを `vm.rs` にポートする。
名前空間インデックス・メソッドインデックスを文字列テーブルから引いて分岐する。

### COLLECT_STACK の利用

`eval.rs` の `COLLECT_STACK` スレッドローカルを `pub` に変更して VM から直接使う（v0.6.0 の現実的な実装）。

```rust
// eval.rs で pub に変更
pub use crate::eval::{collect_push_frame, collect_yield, collect_pop_frame};
```

---

## Phase 5: CLI + 統合 (`src/main.rs`)

### `fav build`

```rust
"build" => {
    let file = args.next().ok_or("usage: fav build <file>")?;
    let out  = get_opt_flag(&args, "-o")
        .unwrap_or_else(|| file.replace(".fav", ".fvc"));

    let src     = std::fs::read_to_string(&file)?;
    let program = parse(&src)?;
    let _       = check(&program)?;        // 型エラーがあれば非 0 終了
    let ir      = compile_program(&program);
    let artifact = codegen_program(&ir);
    let mut f   = std::fs::File::create(&out)?;
    artifact.write_to(&mut f)?;
    eprintln!("built: {}", out);
}
```

### `fav exec`

```rust
"exec" => {
    let file = args.next().ok_or("usage: fav exec <file.fvc>")?;
    let info_only = args.any(|a| a == "--info");

    let mut f   = std::fs::File::open(&file).map_err(|_| E032)?;
    let artifact = FvcArtifact::read_from(&mut f)?;

    if info_only {
        print_artifact_info(&artifact);
        return;
    }

    let main_idx = artifact.fn_idx_by_name("main").ok_or(E035)?;
    let db_conn  = open_db(db_path);
    let mut vm   = VM::new(artifact.globals.clone(), db_conn);
    let result   = vm.run(&artifact, main_idx, vec![])?;
    // result を表示（fav run と同じフォーマット）
}
```

### `fav exec --info` の出力

```rust
fn print_artifact_info(artifact: &FvcArtifact) {
    println!("FVC v0.6  {} globals  {} functions",
             artifact.globals.len(), artifact.functions.len());
    println!();
    for f in &artifact.functions {
        let name      = &artifact.str_table[f.name_idx as usize];
        let return_ty = &artifact.str_table[f.return_ty_str_idx as usize];
        let effects   = &artifact.str_table[f.effect_str_idx as usize];
        // パラメータ型は v0.6.0 最小実装では "()" とする
        println!("{:<16}  () -> {}  {}", name, return_ty, effects);
    }
}
```

### `fav run` への変更

変更なし。tree-walking インタープリタをそのまま使う。

---

## Phase 6: テストと例

### 命令生成テスト

`codegen.rs` の単体テスト:

```rust
#[test]
fn test_codegen_lit_int() {
    // IRExpr::Lit(Lit::Int(42), Type::Int) → CONST idx; 定数プール[idx] = Int(42)
}

#[test]
fn test_codegen_if() {
    // IRExpr::If → JumpIfFalse + body + JUMP + else_body
}

#[test]
fn test_codegen_chain() {
    // IRStmt::Chain → [expr] CHAIN_CHECK offset STORE_LOCAL slot
}

#[test]
fn test_codegen_collect() {
    // IRExpr::Collect → COLLECT_BEGIN [body] COLLECT_END
}
```

### VM 実行テスト

```rust
#[test]
fn test_vm_lit() {
    // CONST 0; RETURN → Int(42)
}

#[test]
fn test_vm_add() {
    // CONST 0; CONST 1; ADD; RETURN → Int(7)
}

#[test]
fn test_vm_chain_ok() {
    // ok(42) → CHAIN_CHECK → 42
}

#[test]
fn test_vm_chain_escape_err() {
    // err("boom") → CHAIN_CHECK → escape → 関数から err("boom") が返る
}

#[test]
fn test_vm_collect_yield() {
    // COLLECT_BEGIN; CONST 1; YIELD_VALUE; CONST 2; YIELD_VALUE; COLLECT_END → List([1, 2])
}

#[test]
fn test_vm_match_guard() {
    // guard が false なら次のアームへ
}
```

### 統合テスト: `fav run` と `fav exec` の出力一致

全 examples に対して:
1. `fav run <file>` の標準出力を取得
2. `fav build <file>` → `fav exec <file.fvc>` の標準出力を取得
3. 両者が一致することを確認

---

## 設計メモ

### 型情報の省略

IR ノードの `Type` は v0.6.0 では `Type::Unknown` でも実行可能（VM は型を参照しない）。型チェックは checker が済ませているため、IR/VM で再度検証しない。将来のデバッグ情報改善フェーズで型を完全に持たせる予定。

### ChainEscape の境界

eval.rs では `RuntimeError.escape` を `?` で伝播させて `eval_apply` でキャッチする方式。VM では `CHAIN_CHECK` の `escape_offset` が直接 `RETURN` 命令にジャンプするため、eval.rs より単純。ChainEscape の「関数境界でのキャッチ」は `CALL` 命令の戻り値処理で行う（フレームポップ後、`Escape` シグナルが来たらそのフレームの `RETURN` として処理）。

### COLLECT_STACK の将来

v0.6.0 は `eval.rs` のスレッドローカルを流用する。将来的に `CallFrame` に collect フレームのネスト情報を持たせ、スレッドローカル依存を排除することで並行実行対応の布石とする。

### グローバルテーブルの構築順

グローバルテーブルは最初に全シグネチャを登録（`register_item_signatures` 相当）してからボディをコンパイルする。これにより相互再帰関数でも前方参照が解決できる。
