//! v19.2.0: AOT compilation via Cranelift
//!
//! Compiles a Favnir IRProgram to a native binary.
//! Scope: Int/Bool literals, basic arithmetic/comparison, If, Block, Local variables.
//! Complex types (List, Stream, Closure) are not supported in v19.2.0.

use cranelift_codegen::ir::{condcodes::IntCC, types, AbiParam, InstBuilder};
use cranelift_codegen::settings::Configurable;
use cranelift_codegen::{settings, Context};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{default_libcall_names, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use crate::ast::{BinOp, Lit};
use crate::middle::ir::{IRExpr, IRProgram, IRStmt};
use std::collections::HashMap;

pub struct CraneliftBackend;

impl CraneliftBackend {
    /// Compile an IRProgram to a native binary at `out_path`.
    /// Requires a C compiler (`cc`) to link the generated object file.
    pub fn compile_to_binary(ir: &IRProgram, out_path: &str) -> Result<(), String> {
        let obj_bytes = Self::lower_to_object(ir)?;
        let wrapper_src = Self::c_wrapper_src();
        Self::link_binary(&obj_bytes, &wrapper_src, out_path)
    }

    /// Lower IRProgram → Cranelift object bytes (.o).
    fn lower_to_object(ir: &IRProgram) -> Result<Vec<u8>, String> {
        let mut flag_builder = settings::builder();
        flag_builder
            .set("use_colocated_libcalls", "false")
            .unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        flag_builder.set("opt_level", "none").unwrap();
        let flags = settings::Flags::new(flag_builder);

        let isa_builder = cranelift_native::builder()
            .map_err(|e| format!("cranelift_native::builder error: {e}"))?;
        let isa = isa_builder
            .finish(flags)
            .map_err(|e| format!("ISA finish error: {e}"))?;

        let obj_builder =
            ObjectBuilder::new(isa, "favnir_aot", default_libcall_names())
                .map_err(|e| format!("ObjectBuilder error: {e}"))?;
        let mut module = ObjectModule::new(obj_builder);

        // Find and lower `main` function
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
        Ok(())
    }
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
