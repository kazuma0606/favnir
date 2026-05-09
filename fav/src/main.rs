mod ast;
mod toml;
mod lock;
mod value;
mod fmt;
mod lint;
mod frontend;
mod middle;
mod backend;
mod driver;
mod lsp;
mod std_states;

use std::process;
use driver::{cmd_run, cmd_build, cmd_exec, cmd_check, cmd_explain, cmd_explain_diff, cmd_test, cmd_fmt, cmd_lint, cmd_install, cmd_publish, cmd_bundle, cmd_graph, cmd_watch, cmd_bench, cmd_migrate};

// 笏笏 help text (4-6) 笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏

const HELP: &str = "\
fav - Favnir language toolchain v2.0.0

USAGE:
    fav <COMMAND> [OPTIONS] [FILE]

COMMANDS:
    run [--db <url>] [file]
                  Parse, type-check, and run a Favnir program.
                  If <file> is omitted, looks for fav.toml and runs src/main.fav.
    build [-o <file>] [--target <fvc|wasm>] [file]
                  Parse, type-check, and build a .fvc artifact or .wasm module.
                  If <file> is omitted, looks for fav.toml and builds src/main.fav.
    exec [--db <path>] [--info] <artifact>
                  Execute a .fvc artifact by running its `main` function.
                  With --info, print artifact metadata instead of executing.
    check [--no-warn] [file]
                  Parse and type-check (no execution).
                  With --no-warn, suppress warning output.
                  If <file> is omitted, checks all .fav files in the project.
    explain [--schema] [--format <text|json>] [--focus <all|fns|trfs|flws|types>] [file]
                  Show VIS / type / effect signatures of all top-level items.
                  With --schema, print SQL CREATE TABLE / CHECK output instead.
                  With --format json, emit structured explain JSON.
                  If <file> is omitted, explains all files in the project.
    explain diff [--format <text|json>] <from> <to>
                  Compare explain metadata across .fav / .json / .fvc inputs.
    bundle [-o <file>] [--entry <name>] [--manifest] [--explain] <file>
                  Build a reachability-trimmed .fvc artifact and optional manifest/explain outputs.
    graph [--format <text|mermaid>] [--focus <flw|fn>] [--entry <name>] [--depth <n>] <file>
                  Show the flow or function dependency graph in text or Mermaid form.
    test [--filter <pattern>] [--fail-fast] [--no-capture] [--coverage] [--coverage-report <dir>] [file]
                  Run test blocks in .fav / .test.fav / .spec.fav files.
                  With --coverage, print line coverage after tests complete.
                  With --coverage-report <dir>, write coverage report to <dir>/coverage.txt.
                  If <file> is omitted, runs all tests in the project.
    bench [--filter <pattern>] [--iters <n>] [file]
                  Run bench blocks in .fav / .bench.fav files.
                  --iters sets iteration count (default: 100).
                  If <file> is omitted, runs all bench blocks in the project.
    watch [--cmd <check|test|run>] [--dir <path>] [--debounce <ms>] [file]
                  Watch .fav files and re-run the selected command on change.
                  --dir can be specified multiple times to watch extra directories.
                  --debounce sets the debounce delay in milliseconds (default: 80).
    fmt [--check] [file]
                  Format a .fav file in-place (canonical style).
                  With --check, exit 1 if any file would change.
                  If <file> is omitted, formats all .fav files in the project.
    lint [--warn-only] [file]
                  Run static lint checks (L001-L004) on a .fav file.
                  With --warn-only, always exit 0 (warnings only).
                  If <file> is omitted, lints all .fav files in the project.
    lsp [--port <n>]
                  Run the Favnir Language Server scaffold.
    migrate [--in-place] [--dry-run] [--check] [--dir <path>] [file]
                  Migrate v1.x code to v2.0.0 syntax (trf→stage, flw→seq).
                  With --in-place, rewrite files directly.
                  With --dry-run, show changes without writing.
                  With --check, exit 1 if any file needs migration (CI use).
    install       Resolve [dependencies] from fav.toml and write fav.lock.
    publish       Validate project and prepare for local registry publishing.
    help          Show this help message

OPTIONS (run / exec):
    --db <path>   SQLite database path (default: :memory:)
                  e.g. --db myapp.db  or  --db :memory:
                  (exec: parsed and reserved; Db.* builtins coming in v0.7.0)

SINGLE-FILE EXAMPLES:
    fav run examples/hello.fav
    fav run --db myapp.db examples/users.fav
    fav build -o dist/app.fvc examples/hello.fav
    fav build --target wasm -o dist/hello.wasm examples/hello.fav
    fav exec dist/app.fvc
    fav exec --info dist/app.fvc
    fav check examples/pipeline.fav
    fav explain examples/users.fav
    fav explain examples/users.fav --format json --focus trfs
    fav bundle examples/hello.fav --manifest
    fav graph examples/abstract_flw_basic.fav --format mermaid
    fav graph examples/hello.fav --focus fn --entry main --depth 2

PROJECT EXAMPLES (requires fav.toml):
    fav run                 # runs src/main.fav
    fav check               # checks all src/**/*.fav
    fav watch --cmd check   # watches project .fav files and re-runs checks
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
    W001  WASM codegen: unsupported type
    W002  WASM codegen: unsupported expression
    W003  WASM codegen: main signature not supported (must be () -> Unit !Io)
    W004  --db cannot be used with .wasm artifacts
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
            let mut target: Option<&str> = None;
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
                    "--target" => {
                        target = Some(args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --target requires a value");
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
            cmd_build(file, out, target);
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
            let mut no_warn = false;
            let mut file: Option<&str> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--no-warn" => { no_warn = true; i += 1; }
                    other => { file = Some(other); i += 1; }
                }
            }
            cmd_check(file, no_warn);
        }

        Some("explain") => {
            if args.get(2).map(|s| s.as_str()) == Some("diff") {
                let mut format = String::from("text");
                let mut paths = Vec::new();
                let mut i = 3usize;
                while i < args.len() {
                    match args[i].as_str() {
                        "--format" => {
                            format = args.get(i + 1).unwrap_or_else(|| {
                                eprintln!("error: --format requires a value");
                                process::exit(1);
                            }).clone();
                            i += 2;
                        }
                        other => {
                            paths.push(other.to_string());
                            i += 1;
                        }
                    }
                }
                if paths.len() != 2 {
                    eprintln!("error: explain diff requires <from> and <to>");
                    process::exit(1);
                }
                cmd_explain_diff(&paths[0], &paths[1], &format);
                return;
            }
            let mut schema = false;
            let mut format = String::from("text");
            let mut focus = String::from("all");
            let mut file: Option<&str> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--schema" => { schema = true; i += 1; }
                    "--format" => {
                        format = args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --format requires a value");
                            process::exit(1);
                        }).clone();
                        i += 2;
                    }
                    "--focus" => {
                        focus = args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --focus requires a value");
                            process::exit(1);
                        }).clone();
                        i += 2;
                    }
                    other => { file = Some(other); i += 1; }
                }
            }
            cmd_explain(file, schema, &format, &focus);
        }

        Some("bundle") => {
            let mut out: Option<&str> = None;
            let mut entry = String::from("main");
            let mut manifest = false;
            let mut explain = false;
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
                    "--entry" => {
                        entry = args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --entry requires a name");
                            process::exit(1);
                        }).clone();
                        i += 2;
                    }
                    "--manifest" => { manifest = true; i += 1; }
                    "--explain" => { explain = true; i += 1; }
                    other => { file = Some(other); i += 1; }
                }
            }
            let file = file.unwrap_or_else(|| {
                eprintln!("error: bundle requires a source file");
                process::exit(1);
            });
            cmd_bundle(file, out, &entry, manifest, explain);
        }

        Some("graph") => {
            let mut format = String::from("text");
            let mut focus: Option<String> = None;
            let mut entry: Option<String> = None;
            let mut depth: Option<usize> = None;
            let mut file: Option<&str> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--format" => {
                        format = args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --format requires a value");
                            process::exit(1);
                        }).clone();
                        i += 2;
                    }
                    "--focus" => {
                        focus = Some(args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --focus requires a value");
                            process::exit(1);
                        }).clone());
                        i += 2;
                    }
                    "--entry" => {
                        entry = Some(args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --entry requires a value");
                            process::exit(1);
                        }).clone());
                        i += 2;
                    }
                    "--depth" => {
                        let raw = args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --depth requires a value");
                            process::exit(1);
                        });
                        depth = Some(raw.parse::<usize>().unwrap_or_else(|_| {
                            eprintln!("error: --depth must be an integer");
                            process::exit(1);
                        }));
                        i += 2;
                    }
                    other => { file = Some(other); i += 1; }
                }
            }
            let file = file.unwrap_or_else(|| {
                eprintln!("error: graph requires a source file");
                process::exit(1);
            });
            cmd_graph(file, &format, focus.as_deref(), entry.as_deref(), depth);
        }

        Some("test") => {
            let mut filter: Option<String> = None;
            let mut fail_fast = false;
            let mut no_capture = false;
            let mut coverage = false;
            let mut coverage_report_dir: Option<String> = None;
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
                    "--no-capture" => { no_capture = true; i += 1; }
                    "--coverage" => { coverage = true; i += 1; }
                    "--coverage-report" => {
                        coverage_report_dir = Some(args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --coverage-report requires a directory");
                            process::exit(1);
                        }).clone());
                        i += 2;
                    }
                    other => { file = Some(other.to_string()); i += 1; }
                }
            }
            cmd_test(file.as_deref(), filter.as_deref(), fail_fast, no_capture, coverage, coverage_report_dir.as_deref());
        }

        Some("bench") => {
            let mut filter: Option<String> = None;
            let mut iters: u64 = 100;
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
                    "--iters" => {
                        let raw = args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --iters requires a number");
                            process::exit(1);
                        });
                        iters = raw.parse::<u64>().unwrap_or_else(|_| {
                            eprintln!("error: --iters must be an integer");
                            process::exit(1);
                        });
                        i += 2;
                    }
                    other => { file = Some(other.to_string()); i += 1; }
                }
            }
            cmd_bench(file.as_deref(), filter.as_deref(), iters);
        }

        Some("watch") => {
            let mut cmd = String::from("check");
            let mut file: Option<String> = None;
            let mut dirs: Vec<String> = Vec::new();
            let mut debounce_ms: u64 = 80;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--cmd" => {
                        cmd = args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --cmd requires a value");
                            process::exit(1);
                        }).clone();
                        i += 2;
                    }
                    "--dir" => {
                        let d = args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --dir requires a path");
                            process::exit(1);
                        }).clone();
                        dirs.push(d);
                        i += 2;
                    }
                    "--debounce" => {
                        let raw = args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --debounce requires a value");
                            process::exit(1);
                        });
                        debounce_ms = raw.parse::<u64>().unwrap_or(80);
                        i += 2;
                    }
                    other => { file = Some(other.to_string()); i += 1; }
                }
            }
            let dir_refs: Vec<&str> = dirs.iter().map(|s| s.as_str()).collect();
            cmd_watch(file.as_deref(), &cmd, &dir_refs, debounce_ms);
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

        Some("migrate") => {
            let mut in_place = false;
            let mut dry_run = false;
            let mut check = false;
            let mut dir: Option<String> = None;
            let mut file: Option<String> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--in-place" => { in_place = true; i += 1; }
                    "--dry-run" => { dry_run = true; i += 1; }
                    "--check" => { check = true; i += 1; }
                    "--dir" => {
                        dir = Some(args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --dir requires a path");
                            process::exit(1);
                        }).clone());
                        i += 2;
                    }
                    other => { file = Some(other.to_string()); i += 1; }
                }
            }
            cmd_migrate(file.as_deref(), in_place, dry_run, check, dir.as_deref());
        }

        Some("install") => {
            cmd_install();
        }

        Some("publish") => {
            cmd_publish();
        }

        Some("lsp") => {
            let mut port: Option<u16> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--port" => {
                        let raw = args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --port requires a number");
                            process::exit(1);
                        });
                        port = Some(raw.parse::<u16>().unwrap_or_else(|_| {
                            eprintln!("error: --port must be a valid u16");
                            process::exit(1);
                        }));
                        i += 2;
                    }
                    other => {
                        eprintln!("error: unexpected lsp argument `{}`", other);
                        process::exit(1);
                    }
                }
            }
            lsp::run_lsp_server(port);
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
