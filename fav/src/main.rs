mod ast;
mod backend;
mod docs_server;
mod driver;
mod error_catalog;
mod fmt;
mod frontend;
mod lint;
mod lock;
mod lsp;
mod middle;
mod std_states;
mod toml;
mod value;

use driver::{
    cmd_bench, cmd_build, cmd_bundle, cmd_check, cmd_check_with_sample, cmd_checkpoint_list,
    cmd_checkpoint_reset, cmd_checkpoint_set, cmd_checkpoint_show, cmd_docs, cmd_exec, cmd_explain,
    cmd_explain_compiler, cmd_explain_diff, cmd_explain_error, cmd_explain_error_list, cmd_fmt,
    cmd_graph, cmd_infer, cmd_infer_proto, cmd_install, cmd_lint, cmd_migrate, cmd_new,
    cmd_publish, cmd_run, cmd_test, cmd_watch,
};
use std::process;

// 笏笏 help text (4-6) 笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏

const HELP: &str = "\
fav - Favnir language toolchain v4.0.0

USAGE:
    fav <COMMAND> [OPTIONS] [FILE]

COMMANDS:
    run [--db <url>] [file]
                  Parse, type-check, and run a Favnir program.
                  If <file> is omitted, looks for fav.toml and runs src/main.fav.
    build [-o <file>] [--target <fvc|wasm>] [--graphql] [--proto] [file]
                  Parse, type-check, and build a .fvc artifact or .wasm module.
                  With --graphql, generate GraphQL SDL from type/interface declarations.
                  With --proto, generate Protobuf schema from type/interface declarations.
                  If <file> is omitted, looks for fav.toml and builds src/main.fav.
    exec [--db <path>] [--info] <artifact>
                  Execute a .fvc artifact by running its `main` function.
                  With --info, print artifact metadata instead of executing.
    check [--no-warn] [--dir <path>] [--sample <N>] [file]
                  Parse and type-check (no execution).
                  With --no-warn, suppress warning output.
                  With --dir, check all .fav files under the given directory.
                  With --sample N, generate N synthetic rows for the first type and
                  verify the pipeline runs without errors (requires a file argument).
                  If <file> is omitted, checks all .fav files in the project.
    explain [--schema] [--format <text|json>] [--focus <all|fns|stages|seqs|types>] [file]
                  Show VIS / type / effect signatures of all top-level items.
                  With --schema, print SQL CREATE TABLE / CHECK output instead.
                  With --format json, emit structured explain JSON.
                  If <file> is omitted, explains all files in the project.
    docs [file] [--port <n>] [--no-open]
                  Start a local docs server backed by explain JSON and stdlib metadata.
                  If <file> is omitted, serves stdlib-only docs.
    checkpoint <list|show|reset|set> [args]
                  Manage incremental processing checkpoints.
    explain compiler
                  Show the 5-step Favnir compilation pipeline summary.
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
    new <name> [--template <script|pipeline|lib>]
                  Create a new project scaffold (default template: script).
    watch [--cmd <check|test|run>] [--dir <path>] [--debounce <ms>] [file]
                  Watch .fav files and re-run the selected command on change.
                  --dir can be specified multiple times to watch extra directories.
                  --debounce sets the debounce delay in milliseconds (default: 80).
    fmt [--check] [file]
                  Format a .fav file in-place (canonical style).
                  With --check, exit 1 if any file would change.
                  If <file> is omitted, formats all .fav files in the project.
    infer [csv_path] [table_name] [--db <conn_str>] [--out <path>] [--name <TypeName>] [--proto <file>]
                  Infer Favnir type definitions from a CSV file or DB table schema.
                  With --proto, infer Favnir definitions from a .proto file.
                  Without --db, infers from csv_path (positional arg).
                  With --db, infers from the given DB connection string.
                  Optionally specify a table name to infer only that table.
                  --out writes output to a file; --out <dir>/ writes one file per table.
                  --name sets the type name for CSV inference (default: Row).
    lint [--warn-only] [file]
                  Run static lint checks (L001-L008) on a .fav file.
                  With --warn-only, always exit 0 (warnings only).
                  If <file> is omitted, lints all .fav files in the project.
    lsp [--port <n>]
                  Run the Favnir Language Server scaffold.
    migrate [--in-place] [--dry-run] [--check] [--dir <path>] [file]
                  Migrate v1.x code to v2.0.0 syntax (trf→stage, flw→seq).
                  With --in-place, rewrite files directly.
                  With --dry-run, show changes without writing.
                  With --check, exit 1 if any file needs migration (CI use).
    explain-error <code>
                  Show details for a specific error code (e.g. E0213).
    explain-error --list
                  List all known error codes with titles.
    install       Resolve [dependencies] from fav.toml and write fav.lock.
    publish       Validate project and prepare for local registry publishing.
    help          Show this help message

OPTIONS (run / exec):
    --db <path>   SQLite database path (default: :memory:)
                  e.g. --db myapp.db  or  --db :memory:
                  (exec: parsed and reserved; Db.* builtins coming in v0.7.0)

SINGLE-FILE EXAMPLES:
    fav run examples/basic/hello.fav
    fav run --db myapp.db examples/basic/users.fav
    fav build -o dist/app.fvc examples/basic/hello.fav
    fav build --target wasm -o dist/hello.wasm examples/wasm/math_wasm.fav
    fav exec dist/app.fvc
    fav exec --info dist/app.fvc
    fav check examples/pipeline/pipeline.fav
    fav explain examples/basic/users.fav
    fav explain examples/basic/users.fav --format json --focus stages
    fav docs examples/basic/users.fav --port 7777 --no-open
    fav bundle examples/basic/hello.fav --manifest
    fav graph examples/pipeline/abstract_seq_basic.fav --format mermaid
    fav graph examples/basic/hello.fav --focus fn --entry main --depth 2

PROJECT EXAMPLES (requires fav.toml):
    fav run                 # runs src/main.fav
    fav check               # checks all src/**/*.fav
    fav check --dir src     # checks all .fav files under src/
    fav watch --cmd check   # watches project .fav files and re-runs checks
    fav explain             # explains all src/**/*.fav

ERROR CODES (v3.4.0 — E0xxx body):
    E01xx  Syntax / structure errors
    E02xx  Type errors (mismatch, undefined, arity)
    E03xx  Effect errors (undeclared, propagation)
    E05xx  Module errors (import, namespace)
    E09xx  Deprecated keywords (trf→stage, flw→seq, cap→interface)
    Use `fav explain-error <code>` for details on a specific error.
    W001  WASM codegen: unsupported type
    W002  WASM codegen: unsupported expression
    W003  WASM codegen: main signature not supported (must be () -> Unit !Io)
    W004  --db cannot be used with .wasm artifacts
";

fn main() {
    // Spawn a thread with a larger stack (64 MB) to support deep recursion in
    // the Favnir VM and the type-checker (especially for self-hosted code).
    let builder = std::thread::Builder::new().stack_size(64 * 1024 * 1024);
    let handler = builder
        .spawn(|| main_impl())
        .expect("failed to spawn main thread");
    match handler.join() {
        Ok(()) => {}
        Err(_) => std::process::exit(1),
    }
}

fn main_impl() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        print_welcome();
        return;
    }

    match args.get(1).map(|s| s.as_str()) {
        Some("help") | Some("--help") | Some("-h") => {
            print!("{}", HELP);
        }

        Some("--version") | Some("-V") => {
            println!("favnir {}", env!("CARGO_PKG_VERSION"));
        }

        Some("run") => {
            // Parse --db flag
            let mut db_path: Option<String> = None;
            let mut file_idx = 2usize;
            if args.get(2).map(|s| s.as_str()) == Some("--db") {
                db_path = Some(
                    args.get(3)
                        .unwrap_or_else(|| {
                            eprintln!("error: --db requires a path argument");
                            process::exit(1);
                        })
                        .clone(),
                );
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
                    "--graphql" => {
                        target = Some("graphql");
                        i += 1;
                    }
                    "--proto" => {
                        target = Some("proto");
                        i += 1;
                    }
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
                    "--info" => {
                        show_info = true;
                        i += 1;
                    }
                    "--db" => {
                        db_path = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --db requires a path argument");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    other => {
                        artifact = Some(other);
                        i += 1;
                    }
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
            let mut dir: Option<&str> = None;
            let mut sample: Option<usize> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--no-warn" => {
                        no_warn = true;
                        i += 1;
                    }
                    "--dir" => {
                        dir = Some(args.get(i + 1).map(|s| s.as_str()).unwrap_or_else(|| {
                            eprintln!("error: --dir requires a value");
                            process::exit(1);
                        }));
                        i += 2;
                    }
                    "--sample" => {
                        let n_str = args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --sample requires a number");
                            process::exit(1);
                        });
                        sample = Some(n_str.parse().unwrap_or_else(|_| {
                            eprintln!("error: --sample value must be a positive integer");
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
            if let Some(n) = sample {
                if let Some(f) = file {
                    cmd_check_with_sample(f, n);
                } else {
                    eprintln!("error: --sample requires a file argument");
                    process::exit(1);
                }
            } else if let Some(dir) = dir {
                driver::cmd_check_dir(dir);
            } else {
                cmd_check(file, no_warn);
            }
        }

        Some("explain") => {
            if args.get(2).map(|s| s.as_str()) == Some("compiler") {
                cmd_explain_compiler();
                return;
            }
            if args.get(2).map(|s| s.as_str()) == Some("diff") {
                let mut format = String::from("text");
                let mut paths = Vec::new();
                let mut i = 3usize;
                while i < args.len() {
                    match args[i].as_str() {
                        "--format" => {
                            format = args
                                .get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --format requires a value");
                                    process::exit(1);
                                })
                                .clone();
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
                    "--schema" => {
                        schema = true;
                        i += 1;
                    }
                    "--format" => {
                        format = args
                            .get(i + 1)
                            .unwrap_or_else(|| {
                                eprintln!("error: --format requires a value");
                                process::exit(1);
                            })
                            .clone();
                        i += 2;
                    }
                    "--focus" => {
                        focus = args
                            .get(i + 1)
                            .unwrap_or_else(|| {
                                eprintln!("error: --focus requires a value");
                                process::exit(1);
                            })
                            .clone();
                        i += 2;
                    }
                    other => {
                        file = Some(other);
                        i += 1;
                    }
                }
            }
            cmd_explain(file, schema, &format, &focus);
        }

        Some("docs") => {
            let mut port = 7777u16;
            let mut no_open = false;
            let mut file: Option<&str> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--port" => {
                        let raw = args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --port requires a number");
                            process::exit(1);
                        });
                        port = raw.parse::<u16>().unwrap_or_else(|_| {
                            eprintln!("error: --port must be a valid u16");
                            process::exit(1);
                        });
                        i += 2;
                    }
                    "--no-open" => {
                        no_open = true;
                        i += 1;
                    }
                    other => {
                        file = Some(other);
                        i += 1;
                    }
                }
            }
            cmd_docs(file, port, no_open);
        }

        Some("checkpoint") => match args.get(2).map(|s| s.as_str()) {
            Some("list") => cmd_checkpoint_list(),
            Some("show") => {
                let name = args.get(3).map(|s| s.as_str()).unwrap_or_else(|| {
                    eprintln!("error: checkpoint show requires <name>");
                    process::exit(1);
                });
                cmd_checkpoint_show(name);
            }
            Some("reset") => {
                let name = args.get(3).map(|s| s.as_str()).unwrap_or_else(|| {
                    eprintln!("error: checkpoint reset requires <name>");
                    process::exit(1);
                });
                cmd_checkpoint_reset(name);
            }
            Some("set") => {
                let name = args.get(3).map(|s| s.as_str()).unwrap_or_else(|| {
                    eprintln!("error: checkpoint set requires <name> <value>");
                    process::exit(1);
                });
                let value = args.get(4).map(|s| s.as_str()).unwrap_or_else(|| {
                    eprintln!("error: checkpoint set requires <name> <value>");
                    process::exit(1);
                });
                cmd_checkpoint_set(name, value);
            }
            _ => {
                eprintln!("error: checkpoint requires one of list|show|reset|set");
                process::exit(1);
            }
        },

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
                        entry = args
                            .get(i + 1)
                            .unwrap_or_else(|| {
                                eprintln!("error: --entry requires a name");
                                process::exit(1);
                            })
                            .clone();
                        i += 2;
                    }
                    "--manifest" => {
                        manifest = true;
                        i += 1;
                    }
                    "--explain" => {
                        explain = true;
                        i += 1;
                    }
                    other => {
                        file = Some(other);
                        i += 1;
                    }
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
                        format = args
                            .get(i + 1)
                            .unwrap_or_else(|| {
                                eprintln!("error: --format requires a value");
                                process::exit(1);
                            })
                            .clone();
                        i += 2;
                    }
                    "--focus" => {
                        focus = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --focus requires a value");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    "--entry" => {
                        entry = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --entry requires a value");
                                    process::exit(1);
                                })
                                .clone(),
                        );
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
                    other => {
                        file = Some(other);
                        i += 1;
                    }
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
                        filter = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --filter requires a pattern argument");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    "--fail-fast" => {
                        fail_fast = true;
                        i += 1;
                    }
                    "--no-capture" => {
                        no_capture = true;
                        i += 1;
                    }
                    "--coverage" => {
                        coverage = true;
                        i += 1;
                    }
                    "--coverage-report" => {
                        coverage_report_dir = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --coverage-report requires a directory");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    other => {
                        file = Some(other.to_string());
                        i += 1;
                    }
                }
            }
            cmd_test(
                file.as_deref(),
                filter.as_deref(),
                fail_fast,
                no_capture,
                coverage,
                coverage_report_dir.as_deref(),
            );
        }

        Some("bench") => {
            let mut filter: Option<String> = None;
            let mut iters: u64 = 100;
            let mut file: Option<String> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--filter" => {
                        filter = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --filter requires a pattern argument");
                                    process::exit(1);
                                })
                                .clone(),
                        );
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
                    other => {
                        file = Some(other.to_string());
                        i += 1;
                    }
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
                        cmd = args
                            .get(i + 1)
                            .unwrap_or_else(|| {
                                eprintln!("error: --cmd requires a value");
                                process::exit(1);
                            })
                            .clone();
                        i += 2;
                    }
                    "--dir" => {
                        let d = args
                            .get(i + 1)
                            .unwrap_or_else(|| {
                                eprintln!("error: --dir requires a path");
                                process::exit(1);
                            })
                            .clone();
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
                    other => {
                        file = Some(other.to_string());
                        i += 1;
                    }
                }
            }
            let dir_refs: Vec<&str> = dirs.iter().map(|s| s.as_str()).collect();
            cmd_watch(file.as_deref(), &cmd, &dir_refs, debounce_ms);
        }

        Some("new") => {
            let name = args.get(2).unwrap_or_else(|| {
                eprintln!("error: new requires a project name");
                process::exit(1);
            });
            let mut template = "script";
            let mut i = 3usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--template" => {
                        template = args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --template requires a value");
                            process::exit(1);
                        });
                        i += 2;
                    }
                    other => {
                        eprintln!("error: unexpected argument to new: {}", other);
                        process::exit(1);
                    }
                }
            }
            cmd_new(name, template);
        }

        Some("fmt") => {
            let mut check = false;
            let mut file: Option<String> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--check" => {
                        check = true;
                        i += 1;
                    }
                    other => {
                        file = Some(other.to_string());
                        i += 1;
                    }
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
                    "--warn-only" => {
                        warn_only = true;
                        i += 1;
                    }
                    other => {
                        file = Some(other.to_string());
                        i += 1;
                    }
                }
            }
            cmd_lint(file.as_deref(), warn_only);
        }

        Some("infer") => {
            let mut proto_path: Option<String> = None;
            let mut csv_path: Option<String> = None;
            let mut table_name: Option<String> = None;
            let mut db_conn: Option<String> = None;
            let mut out_path: Option<String> = None;
            let mut type_name: Option<String> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--proto" => {
                        proto_path = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --proto requires a file path");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    "--db" => {
                        db_conn = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --db requires a connection string");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    "--out" => {
                        out_path = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --out requires a path");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    "--name" => {
                        type_name = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --name requires a type name");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    other => {
                        // First positional: csv_path (or table if --db already set)
                        if db_conn.is_some() && csv_path.is_none() {
                            table_name = Some(other.to_string());
                        } else {
                            csv_path = Some(other.to_string());
                        }
                        i += 1;
                    }
                }
            }
            if let Some(proto_path) = proto_path {
                cmd_infer_proto(&proto_path, out_path.as_deref());
                return;
            }
            cmd_infer(
                csv_path.as_deref(),
                db_conn.as_deref(),
                table_name.as_deref(),
                out_path.as_deref(),
                type_name.as_deref(),
            );
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
                    "--in-place" => {
                        in_place = true;
                        i += 1;
                    }
                    "--dry-run" => {
                        dry_run = true;
                        i += 1;
                    }
                    "--check" => {
                        check = true;
                        i += 1;
                    }
                    "--dir" => {
                        dir = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --dir requires a path");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    other => {
                        file = Some(other.to_string());
                        i += 1;
                    }
                }
            }
            cmd_migrate(file.as_deref(), in_place, dry_run, check, dir.as_deref());
        }

        Some("explain-error") => {
            let mut list = false;
            let mut code: Option<&str> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--list" => {
                        list = true;
                        i += 1;
                    }
                    other => {
                        code = Some(other);
                        i += 1;
                    }
                }
            }
            if list {
                cmd_explain_error_list();
            } else if let Some(c) = code {
                cmd_explain_error(c);
            } else {
                eprintln!("error: explain-error requires a code (e.g. E0213) or --list");
                process::exit(1);
            }
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

        None => print_welcome(),
    }
}

fn print_welcome() {
    use supports_color::{Stream, on};
    let no_color = std::env::var_os("NO_COLOR").is_some();
    if !no_color && on(Stream::Stdout).is_some() {
        let image_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .map(|p| p.join("versions").join("favnir.png"));
        if let Some(path) = image_path {
            if path.exists() {
                let _ = viuer::print_from_file(
                    &path,
                    &viuer::Config {
                        transparent: true,
                        ..Default::default()
                    },
                );
                println!();
            }
        }
        println!("Favnir v3.9.0 - The pipeline-first language");
        println!();
    }
    print!("{}", HELP);
}
