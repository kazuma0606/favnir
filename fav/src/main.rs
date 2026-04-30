mod ast;
mod toml;
mod value;
mod fmt;
mod lint;
mod frontend;
mod middle;
mod backend;
mod driver;

use std::process;
use driver::{cmd_run, cmd_build, cmd_exec, cmd_check, cmd_explain, cmd_test, cmd_fmt, cmd_lint};

// 笏笏 help text (4-6) 笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏

const HELP: &str = "\
fav - Favnir language toolchain v0.8.0

USAGE:
    fav <COMMAND> [OPTIONS] [FILE]

COMMANDS:
    run [--db <url>] [file]
                  Parse, type-check, and run a Favnir program.
                  If <file> is omitted, looks for fav.toml and runs src/main.fav.
    build [-o <file>] [file]
                  Parse, type-check, and build a .fvc artifact.
                  If <file> is omitted, looks for fav.toml and builds src/main.fav.
    exec [--db <path>] [--info] <artifact>
                  Execute a .fvc artifact by running its `main` function.
                  With --info, print artifact metadata instead of executing.
    check [file]  Parse and type-check (no execution).
                  If <file> is omitted, checks all .fav files in the project.
    explain [file]
                  Show VIS / type / effect signatures of all top-level items.
                  If <file> is omitted, explains all files in the project.
    test [--filter <pattern>] [--fail-fast] [file]
                  Run test blocks in .fav / .test.fav / .spec.fav files.
                  If <file> is omitted, runs all tests in the project.
    fmt [--check] [file]
                  Format a .fav file in-place (canonical style).
                  With --check, exit 1 if any file would change.
                  If <file> is omitted, formats all .fav files in the project.
    lint [--warn-only] [file]
                  Run static lint checks (L001-L004) on a .fav file.
                  With --warn-only, always exit 0 (warnings only).
                  If <file> is omitted, lints all .fav files in the project.
    help          Show this help message

OPTIONS (run / exec):
    --db <path>   SQLite database path (default: :memory:)
                  e.g. --db myapp.db  or  --db :memory:
                  (exec: parsed and reserved; Db.* builtins coming in v0.7.0)

SINGLE-FILE EXAMPLES:
    fav run examples/hello.fav
    fav run --db myapp.db examples/users.fav
    fav build -o dist/app.fvc examples/hello.fav
    fav exec dist/app.fvc
    fav exec --info dist/app.fvc
    fav check examples/pipeline.fav
    fav explain examples/users.fav

PROJECT EXAMPLES (requires fav.toml):
    fav run                 # runs src/main.fav
    fav check               # checks all src/**/*.fav
    fav explain             # explains all src/**/*.fav

ERROR CODES:
    E001  Type mismatch
    E002  Undefined identifier
    E003  Pipeline / flw connection error
    E004  Effect violation
    E005  Arity mismatch
    E006  Pattern match error
    E007  Db effect missing
    E008  Network effect missing
    E009  Emit effect missing
    E012  Circular import
    E013  Module or symbol not found
    E014  Symbol not public (visibility violation)
    E015  Private symbol referenced from another file
    E016  Internal symbol referenced from outside rune
";

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("help") | Some("--help") | Some("-h") => {
            print!("{}", HELP);
        }

        Some("run") => {
            // Parse --db flag
            let mut db_path: Option<String> = None;
            let mut file_idx = 2usize;
            if args.get(2).map(|s| s.as_str()) == Some("--db") {
                db_path = Some(args.get(3).unwrap_or_else(|| {
                    eprintln!("error: --db requires a path argument");
                    process::exit(1);
                }).clone());
                file_idx = 4;
            }
            let file = args.get(file_idx).map(|s| s.as_str());
            cmd_run(file, db_path.as_deref());
        }

        Some("build") => {
            let mut out: Option<&str> = None;
            let mut file: Option<&str> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "-o" => {
                        out = Some(args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: -o requires a file path");
                            process::exit(1);
                        }));
                        i += 2;
                    }
                    other => {
                        file = Some(other);
                        i += 1;
                    }
                }
            }
            cmd_build(file, out);
        }

        Some("exec") => {
            let mut show_info = false;
            let mut db_path: Option<String> = None;
            let mut artifact: Option<&str> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--info" => { show_info = true; i += 1; }
                    "--db" => {
                        db_path = Some(args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --db requires a path argument");
                            process::exit(1);
                        }).clone());
                        i += 2;
                    }
                    other => { artifact = Some(other); i += 1; }
                }
            }
            let artifact = artifact.unwrap_or_else(|| {
                eprintln!("error: exec requires an artifact path");
                process::exit(1);
            });
            cmd_exec(artifact, show_info, db_path.as_deref());
        }

        Some("check") => {
            let file = args.get(2).map(|s| s.as_str());
            cmd_check(file);
        }

        Some("explain") => {
            let file = args.get(2).map(|s| s.as_str());
            cmd_explain(file);
        }

        Some("test") => {
            let mut filter: Option<String> = None;
            let mut fail_fast = false;
            let mut file: Option<String> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--filter" => {
                        filter = Some(args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --filter requires a pattern argument");
                            process::exit(1);
                        }).clone());
                        i += 2;
                    }
                    "--fail-fast" => { fail_fast = true; i += 1; }
                    other => { file = Some(other.to_string()); i += 1; }
                }
            }
            cmd_test(file.as_deref(), filter.as_deref(), fail_fast);
        }

        Some("fmt") => {
            let mut check = false;
            let mut file: Option<String> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--check" => { check = true; i += 1; }
                    other => { file = Some(other.to_string()); i += 1; }
                }
            }
            cmd_fmt(file.as_deref(), check);
        }

        Some("lint") => {
            let mut warn_only = false;
            let mut file: Option<String> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--warn-only" => { warn_only = true; i += 1; }
                    other => { file = Some(other.to_string()); i += 1; }
                }
            }
            cmd_lint(file.as_deref(), warn_only);
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

