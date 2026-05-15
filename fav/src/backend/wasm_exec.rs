#![allow(dead_code)]

use std::str;

use wasmtime::{Caller, Engine, Extern, Linker, Memory, Module, Store};

fn get_memory(caller: &mut Caller<'_, ()>) -> Result<Memory, String> {
    match caller.get_export("memory") {
        Some(Extern::Memory(memory)) => Ok(memory),
        _ => Err("missing exported memory".into()),
    }
}

fn read_utf8(caller: &mut Caller<'_, ()>, ptr: i32, len: i32) -> Result<String, String> {
    if ptr < 0 || len < 0 {
        return Err("negative string pointer/length".into());
    }
    let memory = get_memory(caller)?;
    let start = ptr as usize;
    let len = len as usize;
    let mut buf = vec![0u8; len];
    memory
        .read(caller, start, &mut buf)
        .map_err(|e| format!("memory read failed: {e}"))?;
    str::from_utf8(&buf)
        .map(|s| s.to_string())
        .map_err(|e| format!("utf8 decode failed: {e}"))
}

pub fn register_host_functions(linker: &mut Linker<()>) -> Result<(), String> {
    linker
        .func_wrap(
            "fav_host",
            "io_println",
            |mut caller: Caller<'_, ()>, ptr: i32, len: i32| -> Result<(), wasmtime::Error> {
                let text = read_utf8(&mut caller, ptr, len).map_err(wasmtime::Error::msg)?;
                println!("{text}");
                Ok(())
            },
        )
        .map_err(|e| format!("failed to register io_println: {e}"))?;
    linker
        .func_wrap(
            "fav_host",
            "io_print",
            |mut caller: Caller<'_, ()>, ptr: i32, len: i32| -> Result<(), wasmtime::Error> {
                let text = read_utf8(&mut caller, ptr, len).map_err(wasmtime::Error::msg)?;
                print!("{text}");
                Ok(())
            },
        )
        .map_err(|e| format!("failed to register io_print: {e}"))?;
    linker
        .func_wrap("fav_host", "io_println_int", |n: i64| {
            println!("{n}");
        })
        .map_err(|e| format!("failed to register io_println_int: {e}"))?;
    linker
        .func_wrap("fav_host", "io_println_float", |f: f64| {
            println!("{f}");
        })
        .map_err(|e| format!("failed to register io_println_float: {e}"))?;
    linker
        .func_wrap("fav_host", "io_println_bool", |b: i32| {
            println!("{}", if b != 0 { "true" } else { "false" });
        })
        .map_err(|e| format!("failed to register io_println_bool: {e}"))?;
    Ok(())
}

pub fn wasm_exec_main(bytes: &[u8]) -> Result<(), String> {
    let engine = Engine::default();
    let module =
        Module::new(&engine, bytes).map_err(|e| format!("error: invalid wasm module: {e}"))?;
    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new(&engine);
    register_host_functions(&mut linker)?;
    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(|e| format!("error: failed to instantiate wasm module: {e}"))?;
    let main = instance
        .get_typed_func::<(), ()>(&mut store, "main")
        .map_err(|e| format!("error: missing or invalid `main` export: {e}"))?;
    main.call(&mut store, ())
        .map_err(|e| format!("error: wasm main failed: {e}"))?;
    Ok(())
}

pub fn wasm_exec_info(bytes: &[u8]) -> String {
    let size = bytes.len();
    let engine = Engine::default();
    match Module::new(&engine, bytes) {
        Ok(module) => {
            let exports = module.exports().count();
            let imports = module.imports().count();
            let has_memory = module.exports().any(|e| e.name() == "memory");
            format!(
                "artifact: .wasm\nformat: WebAssembly binary\nsize: {size} bytes\nstatus: valid\nimports: {imports}\nexports: {exports}\nmemory: {}\n",
                if has_memory { "exported" } else { "none" }
            )
        }
        Err(err) => format!(
            "artifact: .wasm\nformat: WebAssembly binary\nsize: {size} bytes\nstatus: invalid\nerror: {err}\n"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{register_host_functions, wasm_exec_info, wasm_exec_main};
    use crate::ast::{Effect, Lit};
    use crate::backend::wasm_codegen::wasm_codegen_program;
    use crate::middle::checker::Type;
    use crate::middle::ir::{IRExpr, IRFnDef, IRGlobal, IRGlobalKind, IRProgram};
    use wasmtime::{Engine, Linker, Module};

    fn hello_ir() -> IRProgram {
        IRProgram {
            globals: vec![
                IRGlobal {
                    name: "IO".into(),
                    kind: IRGlobalKind::Builtin,
                },
                IRGlobal {
                    name: "main".into(),
                    kind: IRGlobalKind::Fn(0),
                },
            ],
            fns: vec![IRFnDef {
                name: "main".into(),
                param_count: 0,
                param_tys: vec![],
                local_count: 0,
                effects: vec![Effect::Io],
                return_ty: Type::Unit,
                body: IRExpr::Call(
                    Box::new(IRExpr::FieldAccess(
                        Box::new(IRExpr::Global(0, Type::Unknown)),
                        "println".into(),
                        Type::Unknown,
                    )),
                    vec![IRExpr::Lit(Lit::Str("Hello, Favnir!".into()), Type::String)],
                    Type::Unit,
                ),
            }],
            type_metas: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn register_host_functions_registers_all_symbols() {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        register_host_functions(&mut linker).unwrap();
    }

    #[test]
    fn wasm_exec_main_runs_generated_module() {
        let bytes = wasm_codegen_program(&hello_ir()).unwrap();
        wasm_exec_main(&bytes).unwrap();
    }

    #[test]
    fn wasm_exec_info_reports_imports_exports_and_memory() {
        let bytes = wasm_codegen_program(&hello_ir()).unwrap();
        let info = wasm_exec_info(&bytes);
        assert!(info.contains("artifact: .wasm"));
        assert!(info.contains("status: valid"));
        assert!(info.contains("imports: 1"));
        assert!(info.contains("memory: exported"));

        let engine = Engine::default();
        Module::new(&engine, &bytes).unwrap();
    }
}
