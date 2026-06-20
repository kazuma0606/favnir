// fav_core — WASM-safe public API
// Exposes parse + type-check without any native dependencies.

// Pre-existing clippy lints suppressed at crate level.
// These were present before CI lint checking was added (v6.9.0).
// Each lint is tracked here so future contributors can address them incrementally.
#![allow(dead_code)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_match)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::double_ended_iterator_last)]
#![allow(clippy::empty_line_after_doc_comments)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::iter_cloned_collect)]
#![allow(clippy::len_zero)]
#![allow(clippy::let_and_return)]
#![allow(clippy::manual_repeat_n)]
#![allow(clippy::manual_split_once)]
#![allow(clippy::manual_strip)]
#![allow(clippy::missing_const_for_thread_local)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::new_without_default)]
#![allow(clippy::print_literal)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::redundant_guards)]
#![allow(clippy::redundant_locals)]
#![allow(clippy::single_match)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(clippy::unnecessary_lazy_evaluations)]
#![allow(clippy::unnecessary_map_or)]
#![allow(clippy::unnecessary_sort_by)]
#![allow(clippy::unnecessary_to_owned)]
#![allow(clippy::useless_asref)]
#![allow(clippy::useless_conversion)]
#![allow(clippy::while_let_loop)]
#![allow(clippy::explicit_counter_loop)]
#![allow(clippy::write_literal)]

pub mod ast;
pub mod error_catalog;
pub mod frontend;
pub mod emit_python;
pub mod lineage;
pub mod middle;
#[cfg(not(target_arch = "wasm32"))]
pub mod backend;
#[cfg(not(target_arch = "wasm32"))]
pub mod checker_fav_runner;
#[cfg(not(target_arch = "wasm32"))]
pub mod compiler_fav_runner;
#[cfg(not(target_arch = "wasm32"))]
pub mod registry;
#[cfg(not(target_arch = "wasm32"))]
pub mod stdlib_fav_runner;
pub mod schemas;
pub mod std_states;
pub mod toml;
pub mod value;
#[cfg(not(target_arch = "wasm32"))]
pub mod incremental;
#[cfg(not(target_arch = "wasm32"))]
pub mod parallel;
#[cfg(not(target_arch = "wasm32"))]
pub mod profiler;
#[cfg(not(target_arch = "wasm32"))]
pub mod pushdown;
#[cfg(not(target_arch = "wasm32"))]
pub mod arena;
#[cfg(not(target_arch = "wasm32"))]
pub mod dap;
#[cfg(not(target_arch = "wasm32"))]
pub mod coverage;

// wasm_codegen uses only wasm-encoder (pure Rust), available on all targets.
// `#[allow]` needed because backend/mod.rs also declares the same file.
#[allow(clippy::duplicate_mod)]
#[path = "backend/wasm_codegen.rs"]
pub mod wasm_codegen;

use frontend::parser::Parser;
use middle::checker::Checker;
use middle::compiler::compile_program;

/// A single diagnostic produced by the type checker.
#[derive(serde::Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub line: u32,
    pub col: u32,
}

/// Parse and type-check Favnir source code.
/// Returns a list of diagnostics (empty = no errors).
pub fn check_source(source: &str) -> Vec<Diagnostic> {
    let result = Parser::parse_str(source, "<playground>");
    let items = match result {
        Ok(items) => items,
        Err(e) => {
            return vec![Diagnostic {
                code: "E0500".to_string(),
                message: e.message.clone(),
                line: e.span.line,
                col: e.span.col,
            }];
        }
    };

    let (errors, _warnings) = Checker::check_program(&items);

    errors
        .iter()
        .map(|e| Diagnostic {
            code: e.code.to_string(),
            message: e.message.clone(),
            line: e.span.line,
            col: e.span.col,
        })
        .collect()
}

/// Parse, type-check, and compile Favnir source to WASM bytes.
/// Returns Ok(bytes) on success, Err(diagnostic) on failure.
pub fn compile_source_to_wasm(source: &str) -> Result<Vec<u8>, Diagnostic> {
    let items = Parser::parse_str(source, "<playground>").map_err(|e| Diagnostic {
        code: "E0500".to_string(),
        message: e.message.clone(),
        line: e.span.line,
        col: e.span.col,
    })?;

    let (errors, _warnings) = Checker::check_program(&items);
    if let Some(e) = errors.first() {
        return Err(Diagnostic {
            code: e.code.to_string(),
            message: e.message.clone(),
            line: e.span.line,
            col: e.span.col,
        });
    }

    let ir = compile_program(&items);
    wasm_codegen::wasm_codegen_program(&ir).map_err(|e| Diagnostic {
        code: e.code().to_string(),
        message: e.to_string(),
        line: 0,
        col: 0,
    })
}

/// Execute a `.fvc` bytecode artifact file.
/// Mirrors `fav exec <file>` without the `--info` / `.wasm` paths.
/// Called by the `rvm` standalone binary.
#[cfg(not(target_arch = "wasm32"))]
pub fn exec_fvc_file(path: &str, db_url: Option<&str>) {
    use std::process;

    let mut file = std::fs::File::open(path).unwrap_or_else(|e| {
        eprintln!("error: cannot open `{}`: {}", path, e);
        process::exit(1);
    });
    let artifact = backend::artifact::FvcArtifact::read_from(&mut file).unwrap_or_else(|e| {
        eprintln!("error: cannot read artifact `{}`: {}", path, e);
        process::exit(1);
    });
    let main_idx = artifact.fn_idx_by_name("main").unwrap_or_else(|| {
        eprintln!("error: artifact does not contain a `main` function");
        process::exit(1);
    });
    backend::vm::VM::run_with_emits_db_path_and_source_file(
        &artifact,
        main_idx,
        vec![],
        db_url,
        None,
    )
    .unwrap_or_else(|e| {
        eprintln!("runtime error: {}", e.message);
        process::exit(1);
    });
}
