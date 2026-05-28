/// Favnir self-hosted stdlib loader (v8.2.0)
///
/// Loads `self/stdlib/list_stdlib.fav` and `self/stdlib/string_stdlib.fav` as
/// compiled artifacts cached in OnceLock.  Functions in those files are
/// dispatched from `vm_call_builtin` in vm.rs before falling back to Rust
/// implementations, allowing new stdlib functions to be added in Favnir
/// without touching vm.rs.
use std::sync::{Arc, OnceLock};

use crate::backend::artifact::FvcArtifact;
use crate::backend::codegen::codegen_program;
use crate::backend::vm::{VM, VMError};
use crate::frontend::parser::Parser;
use crate::middle::compiler::compile_program;
use crate::value::Value;

// ── artifact caches ───────────────────────────────────────────────────────────

static LIST_STDLIB_ARTIFACT: OnceLock<Arc<FvcArtifact>> = OnceLock::new();
static STRING_STDLIB_ARTIFACT: OnceLock<Arc<FvcArtifact>> = OnceLock::new();

fn get_list_stdlib_artifact() -> Arc<FvcArtifact> {
    LIST_STDLIB_ARTIFACT
        .get_or_init(|| {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("self")
                .join("stdlib")
                .join("list_stdlib.fav");
            let src = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!("stdlib_fav_runner: cannot read {}: {}", path.display(), e)
            });
            let prog = Parser::parse_str(&src, "list_stdlib.fav")
                .expect("stdlib_fav_runner: list_stdlib.fav parse error");
            let ir = compile_program(&prog);
            Arc::new(codegen_program(&ir))
        })
        .clone()
}

fn get_string_stdlib_artifact() -> Arc<FvcArtifact> {
    STRING_STDLIB_ARTIFACT
        .get_or_init(|| {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("self")
                .join("stdlib")
                .join("string_stdlib.fav");
            let src = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!("stdlib_fav_runner: cannot read {}: {}", path.display(), e)
            });
            let prog = Parser::parse_str(&src, "string_stdlib.fav")
                .expect("stdlib_fav_runner: string_stdlib.fav parse error");
            let ir = compile_program(&prog);
            Arc::new(codegen_program(&ir))
        })
        .clone()
}

// ── public API ────────────────────────────────────────────────────────────────

/// Call a function from `list_stdlib.fav`.
/// Returns `Err` if the function is not found or execution fails.
pub fn call_list_stdlib(fname: &str, args: Vec<Value>) -> Result<Value, VMError> {
    let artifact = get_list_stdlib_artifact();
    let fn_idx = artifact.fn_idx_by_name(fname).ok_or_else(|| VMError {
        message: format!("list_stdlib: function not found: {}", fname),
        fn_name: fname.to_string(),
        ip: 0,
        stack_trace: vec![],
    })?;
    VM::run(&artifact, fn_idx, args)
}

/// Call a function from `string_stdlib.fav`.
/// Returns `Err` if the function is not found or execution fails.
pub fn call_string_stdlib(fname: &str, args: Vec<Value>) -> Result<Value, VMError> {
    let artifact = get_string_stdlib_artifact();
    let fn_idx = artifact.fn_idx_by_name(fname).ok_or_else(|| VMError {
        message: format!("string_stdlib: function not found: {}", fname),
        fn_name: fname.to_string(),
        ip: 0,
        stack_trace: vec![],
    })?;
    VM::run(&artifact, fn_idx, args)
}
