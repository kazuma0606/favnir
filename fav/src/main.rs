mod lexer;
mod ast;
mod parser;
mod checker;
mod eval;

use std::process;
use parser::Parser;
use checker::Checker;
use eval::Interpreter;

// ── help text (6-6) ───────────────────────────────────────────────────────────

const HELP: &str = "\
fav - Favnir language interpreter v0.1.0

USAGE:
    fav <COMMAND> <FILE>

COMMANDS:
    run <file>    Parse, type-check, and run a Favnir program
    check <file>  Parse and type-check a Favnir program (no execution)
    help          Show this help message

EXAMPLES:
    fav run examples/hello.fav
    fav check examples/pipeline.fav

ERROR CODES:
    E001  Type mismatch
    E002  Undefined identifier
    E003  Pipeline / flw connection error
    E004  Effect violation
    E005  Arity mismatch
    E006  Pattern match error
";

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("help") | Some("--help") | Some("-h") => {
            print!("{}", HELP);
        }

        Some("run") => {
            let path = args.get(2).unwrap_or_else(|| {
                eprintln!("error: `fav run` requires a file path");
                eprintln!("usage: fav run <file>");
                process::exit(1);
            });
            cmd_run(path);
        }

        Some("check") => {
            let path = args.get(2).unwrap_or_else(|| {
                eprintln!("error: `fav check` requires a file path");
                eprintln!("usage: fav check <file>");
                process::exit(1);
            });
            cmd_check(path);
        }

        Some(cmd) => {
            eprintln!("error: unknown command `{}`", cmd);
            eprintln!("run `fav help` for usage");
            process::exit(1);
        }

        None => {
            print!("{}", HELP);
        }
    }
}

// ── file loading (6-3) ────────────────────────────────────────────────────────

fn load_file(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("error: cannot read `{}`: {}", path, e);
        process::exit(1);
    })
}

// ── fav run (6-1) ─────────────────────────────────────────────────────────────

fn cmd_run(path: &str) {
    let source = load_file(path);

    // Parse
    let program = Parser::parse_str(&source, path).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    // Type-check
    let errors = Checker::check_program(&program);
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("{}", e);
        }
        process::exit(1);
    }

    // Interpret
    if let Err(e) = Interpreter::run(&program) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

// ── fav check (6-2) ───────────────────────────────────────────────────────────

fn cmd_check(path: &str) {
    let source = load_file(path);

    // Parse
    let program = Parser::parse_str(&source, path).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    // Type-check (6-4: errors include file/line/col)
    let errors = Checker::check_program(&program);
    if errors.is_empty() {
        println!("{}: no errors found", path);
    } else {
        for e in &errors {
            eprintln!("{}", e);
        }
        process::exit(1);
    }
}
