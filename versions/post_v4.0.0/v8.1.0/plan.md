# Favnir v8.1.0 実装計画

Date: 2026-05-28

---

## Phase A: `src/middle/ast_lower_checker.rs` 新規作成

Rust の `ast::Program` を checker.fav の VMValue ツリーに変換するモジュール。

### A-1: ファイル作成・モジュール登録

新規ファイル: `fav/src/middle/ast_lower_checker.rs`

`fav/src/middle/mod.rs`（または `lib.rs`）に追加:
```rust
pub mod ast_lower_checker;
```

### A-2: ヘルパー関数と VMValue ユーティリティ

```rust
use crate::backend::vm::VMValue;
use crate::backend::vm::FavList;
use std::collections::HashMap;

fn v0(tag: &str) -> VMValue {
    VMValue::Variant(tag.to_string(), None)
}

fn v1(tag: &str, payload: VMValue) -> VMValue {
    VMValue::Variant(tag.to_string(), Some(Box::new(payload)))
}

fn v2(tag: &str, a: VMValue, b: VMValue) -> VMValue {
    let mut map = HashMap::new();
    map.insert("_0".to_string(), a);
    map.insert("_1".to_string(), b);
    VMValue::Variant(tag.to_string(), Some(Box::new(VMValue::Record(map))))
}

fn v3(tag: &str, a: VMValue, b: VMValue, c: VMValue) -> VMValue {
    let mut map = HashMap::new();
    map.insert("_0".to_string(), a);
    map.insert("_1".to_string(), b);
    map.insert("_2".to_string(), c);
    VMValue::Variant(tag.to_string(), Some(Box::new(VMValue::Record(map))))
}

fn vm_str(s: &str) -> VMValue { VMValue::Str(s.to_string()) }
fn vm_bool(b: bool) -> VMValue { VMValue::Bool(b) }
fn vm_int(n: i64) -> VMValue { VMValue::Int(n) }

fn vm_list(items: Vec<VMValue>) -> VMValue {
    VMValue::List(FavList::new(items))
}

fn vm_option_none() -> VMValue { v0("None") }
fn vm_option_some(v: VMValue) -> VMValue { v1("Some", v) }

fn vm_record(fields: Vec<(&str, VMValue)>) -> VMValue {
    VMValue::Record(fields.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}
```

### A-3: Lit / Op / Pat の変換

```rust
pub fn lower_lit(lit: &crate::ast::Lit) -> VMValue {
    use crate::ast::Lit;
    match lit {
        Lit::Int(n)   => v1("LInt",   vm_int(*n)),
        Lit::Float(f) => v1("LFloat", VMValue::Float(*f)),
        Lit::Str(s)   => v1("LStr",   vm_str(s)),
        Lit::Bool(b)  => v1("LBool",  vm_bool(*b)),
        Lit::Unit     => v0("LUnit"),
    }
}

pub fn lower_binop(op: &crate::ast::BinOp) -> VMValue {
    use crate::ast::BinOp;
    let tag = match op {
        BinOp::Add  => "OpAdd",  BinOp::Sub => "OpSub",
        BinOp::Mul  => "OpMul",  BinOp::Div => "OpDiv",
        BinOp::Mod  => "OpMod",
        BinOp::Eq   => "OpEq",   BinOp::NotEq => "OpNeq",
        BinOp::Lt   => "OpLt",   BinOp::Gt    => "OpGt",
        BinOp::LtEq => "OpLtEq", BinOp::GtEq  => "OpGtEq",
        BinOp::And  => "OpAnd",  BinOp::Or    => "OpOr",
        _           => "OpAdd",  // fallback
    };
    v0(tag)
}

pub fn lower_pat(pat: &crate::ast::Pattern) -> VMValue {
    use crate::ast::Pattern;
    match pat {
        Pattern::Wildcard(_)           => v0("PWild"),
        Pattern::Bind(name, _)         => v1("PVar", vm_str(name)),
        Pattern::Lit(crate::ast::Lit::Int(n), _)   => v1("PInt",   vm_int(*n)),
        Pattern::Lit(crate::ast::Lit::Float(f), _) => v1("PFloat", VMValue::Float(*f)),
        Pattern::Lit(crate::ast::Lit::Str(s), _)   => v1("PStr",   vm_str(s)),
        Pattern::Lit(crate::ast::Lit::Bool(b), _)  => v1("PBool",  vm_bool(*b)),
        Pattern::Lit(crate::ast::Lit::Unit, _)     => v0("PUnit"),
        Pattern::Variant(name, None, _)    => v1("PVariant", vm_str(name)),
        Pattern::Variant(name, Some(p), _) => v2("PVariantP", vm_str(name), lower_pat(p)),
        _                               => v0("PWild"),  // Record パターン等はフォールバック
    }
}
```

### A-4: TypeExpr の変換

```rust
pub fn lower_te(te: &crate::ast::TypeExpr) -> VMValue {
    use crate::ast::TypeExpr;
    match te {
        TypeExpr::Named(name, args, _) => match (name.as_str(), args.as_slice()) {
            ("List",   [t])      => v1("TeList",   lower_te(t)),
            ("Option", [t])      => v1("TeOption", lower_te(t)),
            ("Result", [ok, er]) => v2("TeResult", lower_te(ok), lower_te(er)),
            ("Map",    [k, v])   => v2("TeMap",    lower_te(k),  lower_te(v)),
            ("Fn",     _)        => v0("TeFn"),  // 簡略化
            (name, [])           => v1("TeSimple", vm_str(name)),
            (name, _)            => v1("TeSimple", vm_str(name)),
        },
        TypeExpr::Optional(t, _)      => v1("TeOption", lower_te(t)),
        TypeExpr::Fallible(t, _)      => v2("TeResult", lower_te(t), v1("TeSimple", vm_str("String"))),
        TypeExpr::Arrow(a, b, _)      => v2("TeFn", lower_te(a), lower_te(b)),
        _                             => v1("TeSimple", vm_str("Unknown")),
    }
}
```

### A-5: Expr の変換

```rust
pub fn lower_expr(expr: &crate::ast::Expr) -> VMValue {
    use crate::ast::Expr;
    match expr {
        Expr::Lit(lit, _)                   => v1("ELit", lower_lit(lit)),
        Expr::Ident(name, _)                => v1("EVar", vm_str(name)),
        Expr::BinOp(op, l, r, _)            => v3("EBinOp", lower_binop(op), lower_expr(l), lower_expr(r)),
        Expr::FieldAccess(obj, field, _)    => v2("EAccess", lower_expr(obj), vm_str(field)),
        Expr::Closure(params, body, _)      => {
            let param = params.first().map(|s| s.as_str()).unwrap_or("_");
            v2("ELambda", vm_str(param), lower_expr(body))
        }
        Expr::RecordConstruct(name, fields, _) => {
            v2("ERecordLit", vm_str(name), lower_field_list(fields))
        }
        Expr::If(cond, then_, else_, _)     => {
            let else_e = else_.as_ref()
                .map(|b| lower_block(b))
                .unwrap_or_else(|| v1("ELit", v0("LUnit")));
            v3("EIf", lower_expr(cond), lower_block(then_), else_e)
        }
        Expr::Match(scrutinee, arms, _)     => v2("EMatch", lower_expr(scrutinee), lower_arms(arms)),
        Expr::Block(block)                  => lower_block(block),
        Expr::Apply(func, args, _)          => lower_apply(func, args),
        Expr::Pipeline(steps, span)         => lower_pipeline(steps, span),
        _                                   => v1("EVar", vm_str("_unsupported_")),
    }
}
```

### A-6: Block・Stmt の展開

```rust
fn lower_block(block: &crate::ast::Block) -> VMValue {
    lower_stmts_and_tail(&block.stmts, &block.tail)
}

fn lower_stmts_and_tail(stmts: &[crate::ast::Stmt], tail: &crate::ast::Expr) -> VMValue {
    use crate::ast::Stmt;
    match stmts.split_first() {
        None => lower_expr(tail),
        Some((Stmt::Bind(b), rest)) => {
            let name = extract_bind_name(&b.pattern);
            v3("EBind", vm_str(&name), lower_expr(&b.expr), lower_stmts_and_tail(rest, tail))
        }
        Some((Stmt::Expr(e), rest)) => {
            v2("EBlock", lower_expr(e), lower_stmts_and_tail(rest, tail))
        }
        Some((Stmt::Chain(c), rest)) => {
            v3("EBind", vm_str(&c.name), lower_expr(&c.expr), lower_stmts_and_tail(rest, tail))
        }
        Some((_, rest)) => lower_stmts_and_tail(rest, tail),
    }
}

fn extract_bind_name(pat: &crate::ast::Pattern) -> String {
    match pat {
        crate::ast::Pattern::Bind(name, _) => name.clone(),
        _ => "_".to_string(),
    }
}
```

### A-7: Apply / Pipeline の分解

```rust
fn lower_apply(func: &crate::ast::Expr, args: &[crate::ast::Expr]) -> VMValue {
    use crate::ast::Expr;
    let (ns, fname) = match func {
        Expr::Ident(name, _) => {
            if let Some(dot) = name.rfind('.') {
                (name[..dot].to_string(), name[dot+1..].to_string())
            } else {
                (String::new(), name.clone())
            }
        }
        Expr::FieldAccess(obj, field, _) => {
            let ns_name = match obj.as_ref() {
                Expr::Ident(n, _) => n.clone(),
                _ => String::new(),
            };
            (ns_name, field.clone())
        }
        _ => (String::new(), String::from("_call_")),
    };
    v3("ECall", vm_str(&ns), vm_str(&fname), lower_arg_list(args))
}

fn lower_arg_list(args: &[crate::ast::Expr]) -> VMValue {
    // 再帰的に末尾から EArgNil → EArgList(...) を構築
    args.iter().rev().fold(v0("EArgNil"), |acc, arg| {
        v2("EArgList", lower_expr(arg), acc)
    })
}
// 注意: fold は末尾から積み上げるため、先頭引数が EArgList の先頭になる

fn lower_pipeline(steps: &[crate::ast::Expr], _span: &crate::ast::Span) -> VMValue {
    // Pipeline: [f, g, h] → g(h の結果を f に適用) ... 実際には左結合
    // steps = [val, f1, f2] → f2(f1(val))
    // checker.fav の ECall として展開
    if steps.len() < 2 { return v1("EVar", vm_str("_pipeline_")); }
    let mut result = lower_expr(&steps[0]);
    for step in &steps[1..] {
        // step(result) として ECall に変換
        result = match step {
            crate::ast::Expr::Ident(name, _) => {
                let (ns, fname) = if let Some(d) = name.rfind('.') {
                    (name[..d].to_string(), name[d+1..].to_string())
                } else {
                    (String::new(), name.clone())
                };
                let arg_list = v2("EArgList", result, v0("EArgNil"));
                v3("ECall", vm_str(&ns), vm_str(&fname), arg_list)
            }
            _ => v2("EArgList", lower_expr(step), v2("EArgList", result, v0("EArgNil"))),
        };
    }
    result
}
```

### A-8: Match Arms / Record Fields の変換

```rust
fn lower_arms(arms: &[crate::ast::MatchArm]) -> VMValue {
    arms.iter().rev().fold(v0("EArmNil"), |rest, arm| {
        v3("EArm", lower_pat(&arm.pattern), lower_block(&arm.body), rest)
    })
}

fn lower_field_list(fields: &[(String, crate::ast::Expr)]) -> VMValue {
    fields.iter().rev().fold(v0("EFieldNil"), |rest, (name, expr)| {
        v3("EField", vm_str(name), lower_expr(expr), rest)
    })
}
```

### A-9: FnDef / TypeDef / TestDef / Item / Program

```rust
fn lower_param(p: &crate::ast::Param) -> VMValue {
    vm_record(vec![
        ("name", vm_str(&p.name)),
        ("ty",   lower_te(&p.ty)),
    ])
}

fn lower_fn_def(fd: &crate::ast::FnDef) -> VMValue {
    let is_public = vm_bool(matches!(fd.visibility, Some(crate::ast::Visibility::Public)));
    let effects = vm_list(fd.effects.iter().map(|e| vm_str(&effect_to_str(e))).collect());
    let params  = vm_list(fd.params.iter().map(lower_param).collect());
    let ret     = fd.return_ty.as_ref().map(lower_te).unwrap_or_else(|| v1("TeSimple", vm_str("Unit")));
    let body    = lower_block(&fd.body);
    vm_record(vec![
        ("is_public", is_public),
        ("name",      vm_str(&fd.name)),
        ("effects",   effects),
        ("params",    params),
        ("ret",       ret),
        ("body",      body),
    ])
}

fn effect_to_str(e: &crate::ast::Effect) -> String {
    use crate::ast::Effect;
    match e {
        Effect::Io | Effect::File => "IO".to_string(),
        Effect::Db | Effect::DbRead | Effect::DbWrite | Effect::DbAdmin => "Db".to_string(),
        Effect::Network | Effect::Rpc => "Network".to_string(),
        Effect::Emit(s) => s.clone(),
        Effect::EmitUnion(v) => v.join("|"),
        Effect::Unknown(s) => s.clone(),
        _ => "IO".to_string(),
    }
}

fn lower_type_def(td: &crate::ast::TypeDef) -> VMValue {
    use crate::ast::TypeBody;
    let (is_record, variants, fields) = match &td.body {
        TypeBody::Record(fs) => {
            let flds = vm_list(fs.iter().map(|f| vm_record(vec![
                ("name", vm_str(&f.name)),
                ("ty",   lower_te(&f.ty)),
            ])).collect());
            (true, vm_list(vec![]), flds)
        }
        TypeBody::Sum(vs) => {
            let vdefs = vm_list(vs.iter().map(|v| {
                let name = v.name().to_string();
                let payload = match v {
                    crate::ast::Variant::Tuple(_, ts, _) if !ts.is_empty() =>
                        vm_option_some(lower_te(&ts[0])),
                    _ => vm_option_none(),
                };
                vm_record(vec![("name", vm_str(&name)), ("payload", payload)])
            }).collect());
            (false, vdefs, vm_list(vec![]))
        }
        TypeBody::Alias(te) => (false, vm_list(vec![]), vm_list(vec![])),
    };
    vm_record(vec![
        ("name",        vm_str(&td.name)),
        ("is_record",   vm_bool(is_record)),
        ("type_params", vm_list(td.type_params.iter().map(|s| vm_str(s)).collect())),
        ("variants",    variants),
        ("fields",      fields),
    ])
}

fn lower_test_def(td: &crate::ast::TestDef) -> VMValue {
    vm_record(vec![
        ("name", vm_str(&td.description)),
        ("body", lower_block(&td.body)),
    ])
}

fn lower_item(item: &crate::ast::Item) -> Option<VMValue> {
    use crate::ast::Item;
    match item {
        Item::FnDef(fd)   => Some(v1("IFn",   lower_fn_def(fd))),
        Item::TypeDef(td)  => Some(v1("IType", lower_type_def(td))),
        Item::TestDef(td)  => Some(v1("ITest", lower_test_def(td))),
        _                  => None,  // TrfDef, FlwDef 等はスキップ
    }
}

pub fn lower_program(prog: &crate::ast::Program) -> VMValue {
    let items = vm_list(
        prog.items.iter().filter_map(lower_item).collect()
    );
    vm_record(vec![("items", items)])
}
```

---

## Phase B: `src/middle/checker_fav_runner.rs` 新規作成

```rust
use std::sync::{Arc, OnceLock};
use crate::backend::vm::{VM, VMValue, FvcArtifact};
use crate::frontend::parser::Parser;
use crate::middle::compiler::compile_program;
use crate::backend::codegen::codegen_program;
use crate::middle::checker::TypeError;
use crate::ast::Span;

static CHECKER_FAV_ARTIFACT: OnceLock<Arc<FvcArtifact>> = OnceLock::new();

fn get_checker_fav_artifact() -> Arc<FvcArtifact> {
    CHECKER_FAV_ARTIFACT.get_or_init(|| {
        // self/ ディレクトリは fav バイナリと同じ場所に存在することを期待
        // または CARGO_MANIFEST_DIR 経由で開発時に解決
        let checker_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("self").join("checker.fav");
        let src = std::fs::read_to_string(&checker_path)
            .expect("checker.fav not found");
        let prog = Parser::parse_str(&src, "checker.fav").expect("checker.fav parse error");
        let ir = compile_program(&prog);
        Arc::new(codegen_program(&ir))
    }).clone()
}

pub fn run_checker_fav(prog_vm: VMValue) -> Result<(), Vec<String>> {
    let artifact = get_checker_fav_artifact();
    let check_idx = artifact.fn_idx_by_name("check")
        .expect("checker.fav must export `check` function");

    let result = VM::run(&artifact, check_idx, vec![prog_vm])
        .map_err(|e| vec![format!("checker.fav VM error: {}", e)])?;

    match result {
        VMValue::Variant(ref tag, _) if tag == "Ok" => Ok(()),
        VMValue::Variant(ref tag, Some(ref payload)) if tag == "Err" => {
            let msg = extract_vm_str(payload);
            // 複数エラーは "\n" で結合されている可能性
            Err(msg.lines().filter(|l| !l.is_empty()).map(|l| l.to_string()).collect())
        }
        _ => Err(vec!["unexpected checker.fav result format".to_string()]),
    }
}

fn extract_vm_str(v: &VMValue) -> String {
    match v {
        VMValue::Str(s) => s.clone(),
        _ => format!("{:?}", v),
    }
}

pub fn msgs_to_type_errors(msgs: Vec<String>) -> Vec<TypeError> {
    msgs.into_iter().map(|msg| TypeError {
        message: msg,
        span: Span::default(),
    }).collect()
}
```

---

## Phase C: `src/driver.rs` の修正

### check_single_file の差し替え

```rust
fn check_single_file(path: &str) -> (String, Vec<TypeError>, Vec<FavWarning>) {
    let source = load_file(path);
    let program = Parser::parse_str(&source, path).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    let prog_vm = crate::middle::ast_lower_checker::lower_program(&program);
    match crate::middle::checker_fav_runner::run_checker_fav(prog_vm) {
        Ok(()) => (source, vec![], vec![]),
        Err(msgs) => {
            let errors = crate::middle::checker_fav_runner::msgs_to_type_errors(msgs);
            (source, errors, vec![])
        }
    }
}
```

`--legacy-check` フラグ対応（オプション）:
```rust
fn check_single_file(path: &str, legacy: bool) -> (String, Vec<TypeError>, Vec<FavWarning>) {
    if legacy { return check_single_file_legacy(path); }
    // ... checker.fav 経由 ...
}
```

---

## Phase D: `vm.rs` の Compiler.check_raw 更新

```rust
"Compiler.check_raw" => {
    let path = /* 既存の引数取り出し処理 */;
    let src = match std::fs::read_to_string(&path) {
        Err(e) => return Ok(err_vm(VMValue::Str(format!("cannot read {}: {}", path, e)))),
        Ok(s)  => s,
    };
    let program = match crate::frontend::parser::Parser::parse_str(&src, &path) {
        Err(e) => return Ok(err_vm(VMValue::Str(e.to_string()))),
        Ok(p)  => p,
    };
    let prog_vm = crate::middle::ast_lower_checker::lower_program(&program);
    match crate::middle::checker_fav_runner::run_checker_fav(prog_vm) {
        Ok(())    => Ok(ok_vm(VMValue::Str("compiled".to_string()))),
        Err(msgs) => Ok(err_vm(VMValue::Str(msgs.join("\n")))),
    }
}
```

---

## Phase E: driver.rs 統合テスト 3 件

```rust
// ── checker_v81_tests (v8.1.0) ────────────────────────────────────────────────
#[cfg(test)]
mod checker_v81_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::ast_lower_checker;
    use crate::middle::checker_fav_runner;

    #[test]
    fn checker_fav_wire_valid_fn() {
        // 型エラーなし: 正しい Favnir ソースが Ok を返す
        let src = r#"
fn add(x: Int, y: Int) -> Int { x + y }
public fn main() -> Int { add(1, 2) }
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        let prog_vm = ast_lower_checker::lower_program(&prog);
        assert!(checker_fav_runner::run_checker_fav(prog_vm).is_ok());
    }

    #[test]
    fn checker_fav_wire_generic_fn() {
        // ジェネリクス関数を含むソースが通る
        let src = r#"
fn first_elem(xs: List<A>) -> Option<A> { List.first(xs) }
public fn main() -> Option<Int> { first_elem(List.singleton(42)) }
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        let prog_vm = ast_lower_checker::lower_program(&prog);
        assert!(checker_fav_runner::run_checker_fav(prog_vm).is_ok(),
            "generic fn should pass checker.fav");
    }

    #[test]
    fn checker_fav_wire_self_check() {
        // checker.fav 自身が checker.fav でチェックを通る（完全ブートストラップ）
        let checker_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("self").join("checker.fav");
        let src = std::fs::read_to_string(&checker_path).expect("checker.fav");
        let prog = Parser::parse_str(&src, "checker.fav").expect("parse");
        let prog_vm = ast_lower_checker::lower_program(&prog);
        let result = checker_fav_runner::run_checker_fav(prog_vm);
        assert!(result.is_ok(), "checker.fav self-check failed: {:?}", result);
    }
}
```

---

## Phase F: `src/middle/mod.rs` への登録

```rust
pub mod ast_lower_checker;
pub mod checker_fav_runner;
```

---

## 実装上の注意点

### lower_arg_list の順序

`args.iter().rev().fold(v0("EArgNil"), |acc, arg| v2("EArgList", lower_expr(arg), acc))`

`rev()` してから fold することで、先頭引数が最終的に `EArgList` の先頭になる。
例: `[a, b, c]` → fold from c → b → a:
1. start: EArgNil
2. c: EArgList(c, EArgNil)
3. b: EArgList(b, EArgList(c, EArgNil))
4. a: EArgList(a, EArgList(b, EArgList(c, EArgNil)))

これは `infer_arg_tys` が `List.push` で先頭追加することと対応する。

### OnceLock と cargo test

`cargo test` は並列実行されるため `OnceLock` でのシングルトンキャッシュは適切。
ただし `CARGO_MANIFEST_DIR` は `cargo test` 実行時のみ有効。
本番配布時は checker.fav のパスを設定可能にする。

### checker.fav の check 関数シグネチャ確認

```
public fn check(prog: Program) -> Result<String, String>
```

VMValue として渡す `prog` は `vm_record([("items", vm_list([...]))])` の形式。
`check_single_file` のテスト前に `run_checker_inline` 相当のスモークテストを実行して確認する。

### fold の方向と EArm / EField

EArm チェーン、EField チェーンも `lower_arg_list` と同じく `rev().fold()` で構築することで
正しい順序（先頭アームが最初）を維持する。
