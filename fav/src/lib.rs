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

use middle::checker::Checker;
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
