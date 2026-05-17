// fav_core — WASM-safe public API
// Exposes parse + type-check without any native dependencies.

pub mod ast;
pub mod error_catalog;
pub mod frontend;
pub mod middle;
#[cfg(not(target_arch = "wasm32"))]
pub mod registry;
pub mod schemas;
pub mod std_states;
pub mod toml;
pub mod value;

// wasm_codegen uses only wasm-encoder (pure Rust), available on all targets.
#[path = "backend/wasm_codegen.rs"]
pub mod wasm_codegen;

use middle::checker::Checker;
use middle::compiler::compile_program;
use frontend::parser::Parser;

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
