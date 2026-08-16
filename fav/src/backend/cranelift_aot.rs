//! v19.2.0: AOT compilation via Cranelift
//!
//! Compiles a Favnir IRProgram to a native binary.
//! Scope: Int/Bool literals, basic arithmetic/comparison, If, Block, Local variables.
//! Complex types (List, Stream, Closure) are not supported in v19.2.0.

use cranelift_codegen::ir::{condcodes::IntCC, types, AbiParam, InstBuilder};
use cranelift_codegen::settings::Configurable;
use cranelift_codegen::{isa, settings, Context};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{default_libcall_names, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use crate::ast::{BinOp, Lit};
use crate::middle::ir::{IRExpr, IRProgram, IRStmt};
use std::collections::HashMap;

/// v62.4.0: AOT インライン化分析結果。
pub struct AotStats {
    pub inlined: Vec<String>,
    pub dispatched: Vec<String>,
}

pub struct CraneliftBackend;

impl CraneliftBackend {
    /// Compile an IRProgram to a native binary at `out_path`.
    /// Requires a C compiler (`cc`) to link the generated object file.
    pub fn compile_to_binary(ir: &IRProgram, out_path: &str) -> Result<(), String> {
        let obj_bytes = Self::lower_to_object(ir)?;
        let wrapper_src = Self::c_wrapper_src();
        Self::link_binary(&obj_bytes, &wrapper_src, out_path)
    }

    /// Lower IRProgram → Cranelift object bytes (.o) using host ISA.
    /// v62.3.0 以降は `lower_to_object_with_target(ir, None)` に委譲。
    fn lower_to_object(ir: &IRProgram) -> Result<Vec<u8>, String> {
        Self::lower_to_object_with_target(ir, None)
    }

    /// Lower a single IRFnDef to a Cranelift function exported under `export_name`.
    fn lower_fn_def(
        module: &mut ObjectModule,
        fn_def: &crate::middle::ir::IRFnDef,
        export_name: &str,
    ) -> Result<(), String> {
        let mut sig = module.make_signature();
        for _ in 0..fn_def.param_count {
            sig.params.push(AbiParam::new(types::I64));
        }
        sig.returns.push(AbiParam::new(types::I64));

        let fn_id = module
            .declare_function(export_name, Linkage::Export, &sig)
            .map_err(|e| format!("declare_function error: {e}"))?;

        let mut ctx: Context = module.make_context();
        ctx.func.signature = sig;

        let mut fn_builder_ctx = FunctionBuilderContext::new();
        {
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fn_builder_ctx);

            let block0 = builder.create_block();
            builder.append_block_params_for_function_params(block0);
            builder.switch_to_block(block0);
            builder.seal_block(block0);

            // Declare all local variable slots
            let mut locals: HashMap<u16, Variable> = HashMap::new();
            for i in 0..fn_def.local_count as u16 {
                let var = Variable::from_u32(i as u32);
                builder.declare_var(var, types::I64);
                locals.insert(i, var);
            }

            let result = lower_expr(&mut builder, &fn_def.body, &locals)?;
            builder.ins().return_(&[result]);
            builder.finalize();
        }

        module
            .define_function(fn_id, &mut ctx)
            .map_err(|e| format!("define_function error: {e}"))?;
        module.clear_context(&mut ctx);
        Ok(())
    }

    /// C main() wrapper that calls fav_main() and prints the i64 result.
    fn c_wrapper_src() -> String {
        "#include <stdio.h>\n\
         extern long long fav_main();\n\
         int main(void) {\n\
             long long result = fav_main();\n\
             printf(\"%lld\\n\", result);\n\
             return 0;\n\
         }\n"
            .to_string()
    }

    /// Link: object bytes + C wrapper → native binary at out_path via `cc`.
    fn link_binary(obj_bytes: &[u8], wrapper_src: &str, out_path: &str) -> Result<(), String> {
        use std::fs;
        let tmp_dir =
            tempfile::tempdir().map_err(|e| format!("tempdir error: {e}"))?;

        let obj_path = tmp_dir.path().join("fav_out.o");
        let wrapper_path = tmp_dir.path().join("fav_wrapper.c");

        fs::write(&obj_path, obj_bytes).map_err(|e| format!("write .o error: {e}"))?;
        fs::write(&wrapper_path, wrapper_src)
            .map_err(|e| format!("write .c error: {e}"))?;

        let output = std::process::Command::new("cc")
            .arg(wrapper_path.to_str().unwrap())
            .arg(obj_path.to_str().unwrap())
            .arg("-o")
            .arg(out_path)
            .output()
            .map_err(|e| format!("cc exec error: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("linker error:\n{stderr}"));
        }
        // v71.6.0: strip でデバッグシンボルを除去してバイナリサイズを削減。
        // strip が存在しない環境（Windows 等）では Result を無視して続行する。
        let _ = std::process::Command::new("strip").arg(out_path).output();
        Ok(())
    }

    /// v62.1.0: テスト・driver.rs から呼び出すための pub(crate) ラッパー。
    /// `lower_to_object` の結果をそのまま返す。
    pub(crate) fn lower_to_object_pub(ir: &IRProgram) -> Result<Vec<u8>, String> {
        Self::lower_to_object(ir)
    }

    /// Cranelift の aarch64 ISA 内部登録名。
    const CRANELIFT_AARCH64_NAME: &'static str = "aarch64";

    /// v62.3.0: target triple を指定して object bytes を生成する。
    /// - `None` / `"x86_64-unknown-linux-gnu"` → `cranelift_native::builder()`（ホスト ISA）
    /// - `"aarch64-unknown-linux-gnu"` → cranelift aarch64 ISA
    /// - その他 → `Err("unsupported target triple: ...")`
    fn lower_to_object_with_target(
        ir: &IRProgram,
        target: Option<&str>,
    ) -> Result<Vec<u8>, String> {
        let mut flag_builder = settings::builder();
        flag_builder
            .set("use_colocated_libcalls", "false")
            .map_err(|e| format!("flag set error: {e}"))?;
        flag_builder
            .set("is_pic", "false")
            .map_err(|e| format!("flag set error: {e}"))?;
        flag_builder
            .set("opt_level", "none")
            .map_err(|e| format!("flag set error: {e}"))?;
        let flags = settings::Flags::new(flag_builder);

        let isa: std::sync::Arc<dyn cranelift_codegen::isa::TargetIsa> = match target {
            None | Some("x86_64-unknown-linux-gnu") => {
                let isa_builder = cranelift_native::builder()
                    .map_err(|e| format!("cranelift_native::builder error: {e}"))?;
                isa_builder
                    .finish(flags)
                    .map_err(|e| format!("ISA finish error: {e}"))?
            }
            Some("aarch64-unknown-linux-gnu") => {
                let isa_builder = isa::lookup_by_name(Self::CRANELIFT_AARCH64_NAME)
                    .map_err(|e| format!("aarch64 ISA lookup error: {e:?}"))?;
                isa_builder
                    .finish(flags)
                    .map_err(|e| format!("ISA finish error: {e}"))?
            }
            Some(t) => return Err(format!("unsupported target triple: {t}")),
        };

        let obj_builder =
            ObjectBuilder::new(isa, "favnir_aot", default_libcall_names())
                .map_err(|e| format!("ObjectBuilder error: {e}"))?;
        let mut module = ObjectModule::new(obj_builder);

        let main_fn = ir
            .fns
            .iter()
            .find(|f| f.name == "main")
            .ok_or_else(|| "no `fn main` found in IRProgram".to_string())?;

        Self::lower_fn_def(&mut module, main_fn, "fav_main")?;

        let product = module.finish();
        product
            .emit()
            .map_err(|e| format!("object emit error: {e}"))
    }

    /// v62.3.0: `lower_to_object_with_target` の pub(crate) ラッパー。
    pub(crate) fn lower_to_object_with_target_pub(
        ir: &IRProgram,
        target: Option<&str>,
    ) -> Result<Vec<u8>, String> {
        Self::lower_to_object_with_target(ir, target)
    }

    /// v62.2.0: `compile_to_binary` の pub(crate) ラッパー。
    /// `fav build --link` の driver.rs テストエントリポイント。
    pub(crate) fn compile_to_binary_pub(ir: &IRProgram, out_path: &str) -> Result<(), String> {
        Self::compile_to_binary(ir, out_path)
    }

    /// v71.6.0: アーキテクチャ文字列を Cranelift target triple に変換する。
    /// サポート対象: `"arm64"` / `"aarch64"` のみ（→ `"aarch64-unknown-linux-gnu"`）。
    /// 未知のアーキテクチャは `None` を返す。呼び出し側が警告を出すこと。
    pub(crate) fn arch_to_triple(arch: &str) -> Option<&'static str> {
        match arch {
            "arm64" | "aarch64" => Some("aarch64-unknown-linux-gnu"),
            _ => None,
        }
    }

    /// v71.6.0: arch 指定付き compile_to_binary。
    /// - `arch = None` → ホスト ISA（既存の `compile_to_binary` と同等）
    /// - `arch = Some("arm64")` | `Some("aarch64")` → `aarch64-unknown-linux-gnu`
    /// - `arch = Some(unknown)` → stderr に警告を出しホスト ISA にフォールバック
    pub(crate) fn compile_to_binary_for_arch(
        ir: &IRProgram,
        out_path: &str,
        arch: Option<&str>,
    ) -> Result<(), String> {
        let triple = arch.and_then(|a| {
            let t = Self::arch_to_triple(a);
            if t.is_none() {
                eprintln!(
                    "warning: unsupported --arch `{a}`; supported: arm64, aarch64. \
                     Falling back to host ISA."
                );
            }
            t
        });
        let obj_bytes = Self::lower_to_object_with_target(ir, triple)?;
        let wrapper_src = Self::c_wrapper_src();
        Self::link_binary(&obj_bytes, &wrapper_src, out_path)
    }

    /// v62.4.0: 各関数の AOT 純粋性を分析し、インライン候補とディスパッチ対象に分類する。
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
}

/// v62.4.0: AOT コンパイル可能な IR のみからなる式かを判定する。
/// `Lit::Str` は `lower_lit` が非サポートのため false。その他 Lit / Local / BinOp / If / Block は pure。
/// Global / TrfRef / Call 等は false。
fn is_aot_pure(expr: &IRExpr) -> bool {
    match expr {
        IRExpr::Lit(Lit::Str(_), _) => false,
        IRExpr::Lit(_, _) => true,
        IRExpr::Local(_, _) => true,
        IRExpr::BinOp(_, lhs, rhs, _) => is_aot_pure(lhs) && is_aot_pure(rhs),
        IRExpr::If(cond, then_e, else_e, _) => {
            is_aot_pure(cond) && is_aot_pure(then_e) && is_aot_pure(else_e)
        }
        IRExpr::Block(stmts, final_expr, _) => {
            stmts.iter().all(|s| match s {
                IRStmt::Bind(_, e) | IRStmt::LegacyBind(_, e) | IRStmt::Expr(e) => is_aot_pure(e),
                IRStmt::TrackLine(_) => true, // 行番号マーカー — 副作用なし
                IRStmt::RefinementAssert { expr, .. } => is_aot_pure(expr),
                _ => false,
            }) && is_aot_pure(final_expr)
        }
        _ => false,
    }
}

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
        // v62.8.0 code-review fix: recurse into all compound expression variants
        IRExpr::Call(f, args, _) => {
            contains_aot_unsupported(f) || args.iter().any(contains_aot_unsupported)
        }
        IRExpr::CallTrfLocal { arg, .. } => contains_aot_unsupported(arg),
        IRExpr::Match(scrutinee, arms, _) => {
            contains_aot_unsupported(scrutinee)
                || arms.iter().any(|arm| {
                    arm.guard
                        .as_ref()
                        .map_or(false, contains_aot_unsupported)
                        || contains_aot_unsupported(&arm.body)
                })
        }
        IRExpr::Closure(_, captures, _) => captures.iter().any(contains_aot_unsupported),
        IRExpr::Collect(inner, _) => contains_aot_unsupported(inner),
        IRExpr::FieldAccess(obj, _, _) => contains_aot_unsupported(obj),
        IRExpr::RecordConstruct(fields, _) => {
            fields.iter().any(|(_, v)| contains_aot_unsupported(v))
        }
        IRExpr::RecordSpread(base, updates, _) => {
            contains_aot_unsupported(base)
                || updates.iter().any(|(_, v)| contains_aot_unsupported(v))
        }
        IRExpr::Par { input, .. } => contains_aot_unsupported(input),
        IRExpr::AssertSchema { arg, .. } => contains_aot_unsupported(arg),
        // Leaf variants: Lit / Local / Global / TrfRef
        IRExpr::Lit(_, _) | IRExpr::Local(_, _) | IRExpr::Global(_, _) | IRExpr::TrfRef(_, _) => {
            false
        }
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

/// Recursively lower an IRExpr to a Cranelift Value (i64).
fn lower_expr(
    builder: &mut FunctionBuilder<'_>,
    expr: &IRExpr,
    locals: &HashMap<u16, Variable>,
) -> Result<cranelift_codegen::ir::Value, String> {
    match expr {
        IRExpr::Lit(lit, _) => lower_lit(builder, lit),

        IRExpr::Local(slot, _) => {
            let var = locals
                .get(slot)
                .ok_or_else(|| format!("undefined local slot {slot}"))?;
            Ok(builder.use_var(*var))
        }

        IRExpr::BinOp(op, lhs, rhs, _) => {
            let l = lower_expr(builder, lhs, locals)?;
            let r = lower_expr(builder, rhs, locals)?;
            let result = match op {
                BinOp::Add => builder.ins().iadd(l, r),
                BinOp::Sub => builder.ins().isub(l, r),
                BinOp::Mul => builder.ins().imul(l, r),
                BinOp::Div => builder.ins().sdiv(l, r),
                BinOp::Eq => {
                    let cmp = builder.ins().icmp(IntCC::Equal, l, r);
                    builder.ins().uextend(types::I64, cmp)
                }
                BinOp::NotEq => {
                    let cmp = builder.ins().icmp(IntCC::NotEqual, l, r);
                    builder.ins().uextend(types::I64, cmp)
                }
                BinOp::Lt => {
                    let cmp = builder.ins().icmp(IntCC::SignedLessThan, l, r);
                    builder.ins().uextend(types::I64, cmp)
                }
                BinOp::Gt => {
                    let cmp = builder.ins().icmp(IntCC::SignedGreaterThan, l, r);
                    builder.ins().uextend(types::I64, cmp)
                }
                BinOp::LtEq => {
                    let cmp = builder.ins().icmp(IntCC::SignedLessThanOrEqual, l, r);
                    builder.ins().uextend(types::I64, cmp)
                }
                BinOp::GtEq => {
                    let cmp = builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, l, r);
                    builder.ins().uextend(types::I64, cmp)
                }
                BinOp::And => builder.ins().band(l, r),
                BinOp::Or => builder.ins().bor(l, r),
                other => {
                    return Err(format!(
                        "unsupported BinOp {:?} in AOT v19.2.0",
                        other
                    ))
                }
            };
            Ok(result)
        }

        IRExpr::If(cond, then_e, else_e, _) => {
            let cond_val = lower_expr(builder, cond, locals)?;

            let then_block = builder.create_block();
            let else_block = builder.create_block();
            let merge_block = builder.create_block();
            builder.append_block_param(merge_block, types::I64);

            builder
                .ins()
                .brif(cond_val, then_block, &[], else_block, &[]);

            builder.switch_to_block(then_block);
            builder.seal_block(then_block);
            let then_val = lower_expr(builder, then_e, locals)?;
            builder.ins().jump(merge_block, &[then_val]);

            builder.switch_to_block(else_block);
            builder.seal_block(else_block);
            let else_val = lower_expr(builder, else_e, locals)?;
            builder.ins().jump(merge_block, &[else_val]);

            builder.switch_to_block(merge_block);
            builder.seal_block(merge_block);
            Ok(builder.block_params(merge_block)[0])
        }

        IRExpr::Block(stmts, final_expr, _) => {
            for stmt in stmts {
                match stmt {
                    IRStmt::Bind(slot, expr) | IRStmt::LegacyBind(slot, expr) => {
                        let val = lower_expr(builder, expr, locals)?;
                        let var = locals
                            .get(slot)
                            .ok_or_else(|| format!("undefined local slot {slot}"))?;
                        builder.def_var(*var, val);
                    }
                    IRStmt::Expr(e) => {
                        lower_expr(builder, e, locals)?;
                    }
                    _ => {
                        // Chain, SeqChain, Yield, TrackLine, RefinementAssert
                        // not supported in AOT v19.2.0
                    }
                }
            }
            lower_expr(builder, final_expr, locals)
        }

        other => Err(format!(
            "unsupported IRExpr in AOT v19.2.0: {:?}",
            other.ty()
        )),
    }
}

fn lower_lit(
    builder: &mut FunctionBuilder<'_>,
    lit: &Lit,
) -> Result<cranelift_codegen::ir::Value, String> {
    match lit {
        Lit::Int(n) => Ok(builder.ins().iconst(types::I64, *n)),
        Lit::Bool(b) => Ok(builder.ins().iconst(types::I64, *b as i64)),
        Lit::Float(f) => Ok(builder.ins().iconst(types::I64, f64::to_bits(*f) as i64)),
        Lit::Unit => Ok(builder.ins().iconst(types::I64, 0)),
        Lit::Str(_) => Err("string literals not supported in AOT v19.2.0".to_string()),
    }
}
