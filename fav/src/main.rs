// Pre-existing lints — suppressed until addressed incrementally.
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
#![allow(clippy::fn_to_numeric_cast)]
#![allow(clippy::write_literal)]

mod ast;
mod backend;
mod checker_fav_runner;
mod compiler_fav_runner;
mod emit_python;
mod docs_server;
mod driver;
mod lineage;
mod stdlib_fav_runner;
mod error_catalog;
mod fmt;
mod frontend;
mod lint;
mod lock;
mod lsp;
mod mcp;
mod middle;
mod notebook;
mod registry;
mod rune_cmd;
mod schemas;
mod incremental;
mod parallel;
mod profiler;
#[cfg(not(target_arch = "wasm32"))]
mod pushdown;
#[cfg(not(target_arch = "wasm32"))]
mod otel;
#[cfg(not(target_arch = "wasm32"))]
mod arena;
#[cfg(not(target_arch = "wasm32"))]
mod dap;
#[cfg(not(target_arch = "wasm32"))]
mod coverage;
mod std_states;
mod toml;
mod value;

use driver::{
    cmd_bench, cmd_build, cmd_build_schema, cmd_bundle, cmd_check, cmd_check_with_sample,
    BenchOpts,
    cmd_checkpoint_list, cmd_checkpoint_reset, cmd_checkpoint_set, cmd_checkpoint_show,
    cmd_db_migrate, cmd_db_migrate_rollback, cmd_db_migrate_status, cmd_deploy, cmd_doc,
    cmd_doc_builtins, cmd_doc_site, cmd_doc_serve, cmd_docs,
    cmd_exec, cmd_explain, cmd_explain_code, cmd_explain_compiler, cmd_explain_diff, cmd_explain_error,
    cmd_explain_error_list, cmd_explain_error_list_json, cmd_explain_lineage, cmd_explain_sla, cmd_fmt, cmd_graph,
    cmd_infer, cmd_infer_delta, cmd_infer_iceberg, cmd_infer_postgres, cmd_infer_proto, cmd_infer_snowflake, cmd_install, cmd_lint, cmd_migrate, cmd_upgrade, cmd_new, cmd_new_list,
    cmd_profile, cmd_profile_compare, cmd_scaffold, cmd_transpile,
    cmd_publish, cmd_registry, cmd_repl, cmd_run, cmd_search, cmd_test, cmd_watch,
    cmd_add, cmd_update, cmd_remove, cmd_login, cmd_info,
    cmd_generate_api, cmd_api_serve,
};
use rune_cmd::cmd_rune;
use std::process;

// 笏笏 help text (4-6) 笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏

const HELP: &str = "\
fav - Favnir language toolchain v4.12.0

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
    explain --lineage [--format <text|json|mermaid|d2>] [file]
                  Static lineage analysis: show Sources / Sinks / Transformations / Pipelines.
                  Extracts DB table names from SQL string literals and DbRead/DbWrite lineage tags.
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
    test [--filter <pattern>] [--fail-fast] [--no-capture] [--coverage] [--html] [--lcov] [--coverage-report <dir>] [file]
                  Run test blocks in .fav / .test.fav / .spec.fav files.
                  With --coverage, print line coverage after tests complete.
                  With --coverage-report <dir>, write coverage report to <dir>/coverage.txt.
                  If <file> is omitted, runs all tests in the project.
    bench [--runs <n>] [--iters <n>] [--warmup <n>] [--filter <pattern>] [--json] [file]
                  Run bench blocks in .fav / .bench.fav files.
                  --runs / --iters sets iteration count (default: 100).
                  --warmup sets warmup iterations (default: 5).
                  --json outputs results in JSON format.
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
    mcp           Start MCP server (JSON-RPC over stdin/stdout, protocol 2024-11-05).
    notebook new <name>
                  Create a new notebook (<name>.fav.nb).
    notebook run [--no-cache] <file>
                  Execute all code cells and save outputs.
    notebook serve [--port <n>] [--no-open] <file>
                  Start interactive browser UI (default port 8888).
    notebook export [--out <path>] <file>
                  Export notebook to Markdown.
    notebook check <file>
                  Type-check all code cells without executing.
    migrate [--in-place] [--dry-run] [--check] [--dir <path>] [file]
                  Migrate v1.x code to v2.0.0 syntax (trf→stage, flw→seq).
                  With --in-place, rewrite files directly.
                  With --dry-run, show changes without writing.
                  With --check, exit 1 if any file needs migration (CI use).
    explain-error <code>
                  Show details for a specific error code (e.g. E0213).
    explain-error --list [--format <text|json>]
                  List all known error codes with titles.
                  With --format json, emit structured JSON (for site generation).
    deploy [--target <ecs|k8s|fly|aws-lambda>] [--env <name>] [--function <name>]
           [--region <r>] [--out-dir <path>] [--dry-run]
                  Deploy pipeline to a container or serverless platform.
                  Targets: ecs (AWS ECS Fargate), k8s (Kubernetes CronJob),
                           fly (Fly.io), aws-lambda (default, packages & uploads to S3/Lambda).
                  Generates Dockerfile + platform config in --out-dir (default .fav-deploy/).
    install [<name>] [--force]
                  Install [dependencies] from fav.toml into ./runes/ (Semver deps)
                  or write fav.lock (path/registry deps). Optionally install a single dep by name.
    publish [--name <n>] [--version <v>] [--dry-run] [--force]
                  Publish runes to the local registry (~/.fav/registry/).
    registry [list|search <q>|info <name>]
                  Manage the local Rune registry.
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

const SELF_HOST_STACK_SIZE: usize = 256 * 1024 * 1024;

fn main() {
    // Self-hosted compiler runs recurse deeply through the parser, checker,
    // and VM, so keep the main worker stack aligned with bootstrap tests.
    // In Lambda/constrained environments, use a smaller stack (8 MB) to avoid
    // EAGAIN when spawning threads with large virtual stacks.
    let stack_size = if std::env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
        8 * 1024 * 1024 // 8 MB for Lambda
    } else {
        SELF_HOST_STACK_SIZE
    };
    let builder = std::thread::Builder::new().stack_size(stack_size);
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

    // When invoked as `rune` (symlink / copy), treat all args as `fav rune <args>`.
    let exe_stem = args
        .first()
        .and_then(|p| std::path::Path::new(p).file_stem())
        .and_then(|s| s.to_str())
        .unwrap_or("fav");
    if exe_stem == "rune" {
        cmd_rune(&args[1..]);
        return;
    }

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

        // ── v19.7.0: fav compile ──────────────────────────────────────────────
        Some("compile") => {
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
            let src = file.unwrap_or_else(|| {
                eprintln!("error: compile requires a source file");
                process::exit(1);
            });
            driver::cmd_compile(src, out);
        }

        Some("run") => {
            // ── v21.1.0: fav run --debug <file> ──────────────────────────────
            #[cfg(not(target_arch = "wasm32"))]
            if args.iter().any(|a| a == "--debug") {
                let dap_port = args
                    .iter()
                    .position(|a| a == "--dap-port")
                    .and_then(|i| args.get(i + 1))
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(5678);
                let file = args
                    .iter()
                    .skip(2)
                    .find(|a| !a.starts_with("--"))
                    .map(|s| s.as_str())
                    .unwrap_or_else(|| {
                        eprintln!("error: fav run --debug requires a file path");
                        process::exit(1);
                    });
                driver::cmd_run_debug(file, dap_port);
                return;
            }
            // ── v19.7.0: fav run --precompiled <path> ────────────────────────
            if args.get(2).map(|s| s.as_str()) == Some("--precompiled") {
                let path = args.get(3).map(|s| s.as_str()).unwrap_or_else(|| {
                    eprintln!("error: --precompiled requires a .favc file path");
                    process::exit(1);
                });
                driver::cmd_run_precompiled(path);
                return;
            }
            // ─────────────────────────────────────────────────────────────────
            // ── v24.0.0: fav run --vm <path> --hex <hex> ─────────────────────
            if args.iter().any(|a| a == "--hex") && !args.iter().any(|a| a == "--vm") {
                eprintln!("error: --hex requires --vm <path>");
                process::exit(1);
            }
            if let Some(vm_pos) = args.iter().position(|a| a == "--vm") {
                let vm_path = args.get(vm_pos + 1).map(|s| s.as_str()).unwrap_or_else(|| {
                    eprintln!("error: --vm requires a path argument");
                    process::exit(1);
                });
                let vm_src = std::fs::read_to_string(vm_path).unwrap_or_else(|e| {
                    eprintln!("error: cannot read {}: {}", vm_path, e);
                    process::exit(1);
                });
                // ── v25.9.0: fav run --vm <path> --compile <src> ─────────────
                if let Some(compile_pos) = args.iter().position(|a| a == "--compile") {
                    let src_path = args.get(compile_pos + 1).map(|s| s.as_str()).unwrap_or_else(|| {
                        eprintln!("error: --compile requires a source path argument");
                        process::exit(1);
                    });
                    let src_text = std::fs::read_to_string(src_path).unwrap_or_else(|e| {
                        eprintln!("error: cannot read {}: {}", src_path, e);
                        process::exit(1);
                    });
                    let tokens = crate::frontend::lexer::Lexer::new(&src_text, src_path)
                        .tokenize()
                        .unwrap_or_else(|e| {
                            eprintln!("error: lex: {:?}", e);
                            process::exit(1);
                        });
                    let program = crate::frontend::parser::Parser::new(tokens)
                        .parse_program()
                        .unwrap_or_else(|e| {
                            eprintln!("error: parse: {:?}", e);
                            process::exit(1);
                        });
                    let artifact = driver::build_artifact_pub(&program);
                    let program_json = driver::build_vm_program_json(&artifact);
                    let result = driver::run_via_vm(&vm_src, &program_json);
                    println!("{}", result);
                    return;
                }
                // ── v24.0.0: fav run --vm <path> --hex <hex> ─────────────────
                let hex_pos = args.iter().position(|a| a == "--hex").unwrap_or_else(|| {
                    eprintln!("error: --vm requires --hex <bytecode_hex> or --compile <src>");
                    process::exit(1);
                });
                let bytecode_hex = args.get(hex_pos + 1).map(|s| s.as_str()).unwrap_or_else(|| {
                    eprintln!("error: --hex requires a hex string argument");
                    process::exit(1);
                });
                match driver::run_with_vm(&vm_src, bytecode_hex, &[]) {
                    Ok(v) => println!("{}", v.display()),
                    Err(e) => {
                        eprintln!("error: {}", e);
                        process::exit(1);
                    }
                }
                return;
            }
            // ─────────────────────────────────────────────────────────────────
            // Parse --db / --legacy / --self-host / --verbose / --trace flags
            let mut db_path: Option<String> = None;
            let mut legacy = false;
            let mut verbose = false;
            let mut trace = false;
            let mut no_tap = false;
            let mut legacy_value_repr = false;
            let mut explain_pushdown = false;
            let mut checkpoint_dir: Option<String> = None;
            let mut resume_dir: Option<String> = None;
            let mut file_idx = 2usize;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
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
                        file_idx = i;
                    }
                    "--legacy" => {
                        // [deprecated since v9.0.0] Force Rust pipeline (opt-out of Favnir default)
                        legacy = true;
                        i += 1;
                        file_idx = i;
                    }
                    "--self-host" => {
                        // Backward-compat alias: --self-host is now the default,
                        // so this flag is a no-op.
                        i += 1;
                        file_idx = i;
                    }
                    "--verbose" => {
                        verbose = true;
                        i += 1;
                        file_idx = i;
                    }
                    "--trace" => {
                        trace = true;
                        i += 1;
                        file_idx = i;
                    }
                    "--no-color" => {
                        // Plain text output (default; ANSI color is opt-in in future versions).
                        i += 1;
                        file_idx = i;
                    }
                    "--no-tap" => {
                        // Compile tap/inspect steps as identity (zero cost in production, v16.8.0)
                        no_tap = true;
                        i += 1;
                        file_idx = i;
                    }
                    "--legacy-value-repr" => {
                        // v20.3.0: フォールバック用フラグ（NaN-boxing 移行検証用）
                        legacy_value_repr = true;
                        i += 1;
                        file_idx = i;
                    }
                    "--explain-pushdown" => {
                        // v20.4.0: DuckDB pushdown 適用状況を stderr に出力
                        explain_pushdown = true;
                        i += 1;
                        file_idx = i;
                    }
                    "--checkpoint-dir" => {
                        // v22.1.0: checkpoint ファイルを保存するディレクトリ
                        checkpoint_dir = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --checkpoint-dir requires a directory path");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                        file_idx = i;
                    }
                    "--resume" => {
                        // v22.1.0: checkpoint ファイルを読み込んで該当 stage をスキップ
                        resume_dir = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --resume requires a directory path");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                        file_idx = i;
                    }
                    _ => break,
                }
            }
            let file = args.get(file_idx).map(|s| s.as_str());
            cmd_run(file, db_path.as_deref(), legacy, verbose, trace, no_tap, legacy_value_repr, explain_pushdown, checkpoint_dir.as_deref(), resume_dir.as_deref());
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
                    "--schema" => {
                        target = Some("schema");
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
            if matches!(target, Some("schema")) {
                let f = file.unwrap_or_else(|| {
                    eprintln!("error: build --schema requires a source file");
                    process::exit(1);
                });
                cmd_build_schema(f, out);
            } else {
                cmd_build(file, out, target);
            }
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
            let mut legacy_check = false;
            let mut json = false;
            let mut show_types = false;
            let mut show_effects = false;
            let mut refresh_schemas = false;
            let mut strict = false;
            let mut ambient = false;
            let mut report = false;
            let mut file: Option<&str> = None;
            let mut dir: Option<&str> = None;
            let mut sample: Option<usize> = None;
            let mut all_mode = false;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--no-warn" => {
                        no_warn = true;
                        i += 1;
                    }
                    "--legacy-check" => {
                        legacy_check = true;
                        i += 1;
                    }
                    "--json" => {
                        json = true;
                        i += 1;
                    }
                    "--show-types" => {
                        show_types = true;
                        i += 1;
                    }
                    "--show-effects" => {
                        show_effects = true;
                        i += 1;
                    }
                    "--refresh-schemas" => {
                        refresh_schemas = true;
                        i += 1;
                    }
                    "--strict" => {
                        strict = true;
                        i += 1;
                    }
                    "--ambient" => {
                        ambient = true;
                        i += 1;
                    }
                    "--report" => {
                        report = true;
                        i += 1;
                    }
                    "--no-color" => {
                        // Plain text output (default; ANSI color is opt-in in future versions).
                        // Also respected when NO_COLOR env var is set.
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
                    "--all" => {
                        all_mode = true;
                        i += 1;
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
            } else if all_mode {
                driver::cmd_check_all(json);
            } else {
                cmd_check(file, no_warn, legacy_check, json, show_types, strict, ambient, report, show_effects, refresh_schemas);
            }
        }

        Some("explain") => {
            if args.get(2).map(|s| s.as_str()) == Some("compiler") {
                cmd_explain_compiler();
                return;
            }
            if args.iter().any(|a| a == "--sla") {
                let file = args.iter().skip(2).find(|a| !a.starts_with('-')).map(|s| s.as_str());
                cmd_explain_sla(file);
                return;
            }
            if args.iter().any(|a| a == "--lineage") {
                let mut format = String::from("text");
                let mut file: Option<&str> = None;
                let mut i = 2usize;
                while i < args.len() {
                    match args[i].as_str() {
                        "--lineage" => { i += 1; }
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
                            file = Some(other);
                            i += 1;
                        }
                    }
                }
                cmd_explain_lineage(file, &format);
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
            // fav explain <code>  e.g. E0018 / W006
            if let Some(code) = args.get(2) {
                let c = code.as_str();
                let is_error_code = c.len() >= 2
                    && (c.starts_with('E') || c.starts_with('W'))
                    && c[1..].chars().all(|ch| ch.is_ascii_digit());
                if is_error_code {
                    cmd_explain_code(c);
                    return;
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

        Some("db") => match args.get(2).map(|s| s.as_str()) {
            Some("migrate") => {
                let db_url = args.get(3).map(|s| s.as_str()).unwrap_or_else(|| {
                    eprintln!("error: db migrate requires <db-url>");
                    process::exit(1);
                });
                let migrations_dir = args.get(4).map(|s| s.as_str()).unwrap_or("migrations");
                if args.iter().any(|a| a == "--status") {
                    cmd_db_migrate_status(db_url, migrations_dir);
                } else if args.iter().any(|a| a == "--rollback") {
                    cmd_db_migrate_rollback(db_url, migrations_dir);
                } else {
                    let dry_run = args.iter().any(|a| a == "--dry-run");
                    cmd_db_migrate(db_url, migrations_dir, dry_run);
                }
            }
            _ => {
                eprintln!("error: db requires 'migrate'");
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
            let mut coverage_html = false;
            let mut coverage_lcov = false;
            let mut coverage_report_dir: Option<String> = None;
            let mut update_snapshots = false;
            let mut file: Option<String> = None;
            let mut watch_mode = false;
            let mut watch_dirs: Vec<String> = Vec::new();
            let mut cases: Option<u64> = None;
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
                    "--html" => {
                        coverage_html = true;
                        i += 1;
                    }
                    "--lcov" => {
                        coverage_lcov = true;
                        i += 1;
                    }
                    "--update-snapshots" => {
                        update_snapshots = true;
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
                    "--cases" => {
                        let n: u64 = args
                            .get(i + 1)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or_else(|| {
                                eprintln!("error: --cases requires a positive integer");
                                process::exit(1);
                            });
                        cases = Some(n);
                        i += 2;
                    }
                    "--watch" => {
                        watch_mode = true;
                        i += 1;
                    }
                    other => {
                        file = Some(other.to_string());
                        i += 1;
                    }
                }
            }
            if let Some(n) = cases {
                // SAFETY: setting FORALL_CASES for this process only
                unsafe { std::env::set_var("FORALL_CASES", n.to_string()) };
            }
            if watch_mode {
                let file_for_watch: Option<&str> = if let Some(ref f) = file {
                    let path = std::path::Path::new(f);
                    if path.is_dir() {
                        watch_dirs.push(f.clone());
                        None
                    } else {
                        Some(f.as_str())
                    }
                } else {
                    None
                };
                let dir_refs: Vec<&str> = watch_dirs.iter().map(|s| s.as_str()).collect();
                cmd_watch(file_for_watch, "test", &dir_refs, 80);
                return;
            }
            if (coverage_html || coverage_lcov) && !coverage {
                eprintln!("error: --html/--lcov requires --coverage");
                std::process::exit(1);
            }
            cmd_test(
                file.as_deref(),
                filter.as_deref(),
                fail_fast,
                no_capture,
                coverage,
                coverage_report_dir.as_deref(),
                update_snapshots,
                coverage_html,
                coverage_lcov,
            );
        }

        Some("bench") => {
            // ── v24.3.0: --baseline flag → compare mode ───────────────────────
            if args.iter().any(|a| a == "--baseline") {
                let baseline_path = args.iter().position(|a| a == "--baseline")
                    .and_then(|i| args.get(i + 1))
                    .map(|s| s.as_str())
                    .unwrap_or_else(|| {
                        eprintln!("error: fav bench requires --baseline <path>");
                        process::exit(1);
                    });
                let current_path = args.iter().position(|a| a == "--current")
                    .and_then(|i| args.get(i + 1))
                    .map(|s| s.as_str())
                    .unwrap_or_else(|| {
                        eprintln!("error: fav bench requires --current <path>");
                        process::exit(1);
                    });
                let threshold: f64 = args.iter().position(|a| a == "--threshold")
                    .and_then(|i| args.get(i + 1))
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(5.0);
                let emit_md = args.iter().any(|a| a == "--emit-md");
                let baseline_json = std::fs::read_to_string(baseline_path).unwrap_or_else(|e| {
                    eprintln!("error: cannot read {baseline_path}: {e}");
                    process::exit(1);
                });
                let current_json = std::fs::read_to_string(current_path).unwrap_or_else(|e| {
                    eprintln!("error: cannot read {current_path}: {e}");
                    process::exit(1);
                });
                let (ok, report) = driver::cmd_bench_compare(&baseline_json, &current_json, threshold, emit_md);
                println!("{report}");
                if !ok {
                    process::exit(1);
                }
                return;
            }
            // ── original bench runner ─────────────────────────────────────────
            let mut opts = BenchOpts::default();
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--filter" => {
                        opts.filter = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --filter requires a pattern argument");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    "--runs" | "--iters" => {
                        let raw = args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: {} requires a number", args[i]);
                            process::exit(1);
                        });
                        opts.runs = raw.parse::<u64>().unwrap_or_else(|_| {
                            eprintln!("error: {} must be an integer", args[i]);
                            process::exit(1);
                        });
                        i += 2;
                    }
                    "--warmup" => {
                        let raw = args.get(i + 1).unwrap_or_else(|| {
                            eprintln!("error: --warmup requires a number");
                            process::exit(1);
                        });
                        opts.warmup = raw.parse::<u64>().unwrap_or_else(|_| {
                            eprintln!("error: --warmup must be an integer");
                            process::exit(1);
                        });
                        i += 2;
                    }
                    "--json" => {
                        opts.json = true;
                        i += 1;
                    }
                    other => {
                        opts.file = Some(other.to_string());
                        i += 1;
                    }
                }
            }
            cmd_bench(&opts);
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
            // --list フラグ: テンプレート一覧を表示して終了
            if args.get(2).map(|s| s.as_str()) == Some("--list") {
                cmd_new_list();
                return;
            }
            let name = args.get(2).unwrap_or_else(|| {
                eprintln!("error: new requires a project name");
                eprintln!("  hint: run 'fav new --list' to see available templates");
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

        Some("scaffold") => {
            let sub = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| {
                eprintln!("usage: fav scaffold <stage|seq|postgres-etl|rune> [Name] [flags]");
                process::exit(1);
            });
            let rest: Vec<String> = args[3..].to_vec();
            cmd_scaffold(sub, &rest);
        }

        Some("fmt") => {
            let mut check = false;
            let mut migrate = false;
            let mut file: Option<String> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--check" => {
                        check = true;
                        i += 1;
                    }
                    "--migrate" => {
                        migrate = true;
                        i += 1;
                    }
                    other => {
                        file = Some(other.to_string());
                        i += 1;
                    }
                }
            }
            cmd_fmt(file.as_deref(), check, migrate);
        }

        Some("lint") => {
            let mut warn_only = false;
            let mut deny_warnings = false;
            let mut file: Option<String> = None;
            let mut cli_allow: Vec<String> = Vec::new();
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--warn-only" => {
                        warn_only = true;
                        i += 1;
                    }
                    "--deny-warnings" => {
                        deny_warnings = true;
                        i += 1;
                    }
                    "--allow" => {
                        if i + 1 < args.len() {
                            cli_allow.push(args[i + 1].clone());
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    other => {
                        file = Some(other.to_string());
                        i += 1;
                    }
                }
            }
            cmd_lint(file.as_deref(), warn_only, deny_warnings, cli_allow);
        }

        Some("doc") => {
            // fav doc --builtins [--format json|markdown] [--out <file>]
            if args.iter().any(|a| a == "--builtins") {
                let format = args.windows(2)
                    .find(|w| w[0] == "--format")
                    .map(|w| w[1].as_str())
                    .unwrap_or("markdown");
                let out = args.windows(2)
                    .find(|w| w[0] == "--out")
                    .map(|w| w[1].as_str());
                cmd_doc_builtins(format, out);
                return;
            }
            // fav doc --serve [path] [--port N] [--no-open]
            if args.iter().any(|a| a == "--serve") {
                let port = args.windows(2)
                    .find(|w| w[0] == "--port")
                    .and_then(|w| w[1].parse::<u16>().ok())
                    .unwrap_or(8080);
                let no_open = args.iter().any(|a| a == "--no-open");
                let mut path = ".".to_string();
                let mut i = 2usize;
                while i < args.len() {
                    match args[i].as_str() {
                        "--serve" | "--no-open" => { i += 1; }
                        "--port" => { i += 2; }
                        other if !other.starts_with("--") => {
                            path = other.to_string();
                            break;
                        }
                        _ => { i += 1; }
                    }
                }
                cmd_doc_serve(&path, port, no_open);
                return;
            }
            let format = args.windows(2)
                .find(|w| w[0] == "--format")
                .map(|w| w[1].clone())
                .unwrap_or_else(|| "markdown".to_string());
            let mut path = ".".to_string();
            let mut out_dir = "docs".to_string();
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--format" => { i += 2; }
                    "--out" => {
                        out_dir = args
                            .get(i + 1)
                            .unwrap_or_else(|| {
                                eprintln!("error: --out requires a directory path");
                                process::exit(1);
                            })
                            .clone();
                        i += 2;
                    }
                    other if !other.starts_with("--") => {
                        path = other.to_string();
                        i += 1;
                    }
                    _ => { i += 1; }
                }
            }
            match format.as_str() {
                "site" => cmd_doc_site(&path, &out_dir),
                _ => cmd_doc(&path, &out_dir),
            }
        }

        Some("spec") => {
            // ── v24.1.0: fav spec [--format markdown|html] ───────────────
            let format = if let Some(pos) = args.iter().position(|a| a == "--format") {
                args.get(pos + 1).map(|s| s.as_str()).unwrap_or_else(|| {
                    eprintln!("error: --format requires markdown or html");
                    process::exit(1);
                })
            } else {
                "markdown"
            };
            println!("{}", driver::cmd_spec(format));
        }

        Some("transpile") => {
            let targs: Vec<String> = args[2..].to_vec();
            cmd_transpile(&targs);
        }

        Some("repl") => {
            cmd_repl();
        }

        Some("profile") => {
            let mut path = String::new();
            let mut format = "text".to_string();
            let mut runs: usize = 1;
            let mut stage_filter: Option<String> = None;
            let mut out: Option<String> = None;
            let mut compare: Option<String> = None;
            let mut i = 2usize;
            while i < args.len() {
                let arg = args[i].as_str();
                if let Some(v) = arg.strip_prefix("--format=") {
                    format = v.to_string(); i += 1;
                } else if arg == "--format" {
                    format = args.get(i + 1).cloned().unwrap_or_default(); i += 2;
                } else if let Some(v) = arg.strip_prefix("--runs=") {
                    runs = v.parse().unwrap_or(1); i += 1;
                } else if arg == "--runs" {
                    runs = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(1); i += 2;
                } else if let Some(v) = arg.strip_prefix("--stage=") {
                    stage_filter = Some(v.to_string()); i += 1;
                } else if arg == "--stage" {
                    stage_filter = args.get(i + 1).cloned(); i += 2;
                } else if let Some(v) = arg.strip_prefix("--out=") {
                    out = Some(v.to_string()); i += 1;
                } else if arg == "--out" {
                    out = args.get(i + 1).cloned(); i += 2;
                } else if let Some(v) = arg.strip_prefix("--compare=") {
                    compare = Some(v.to_string()); i += 1;
                } else if arg == "--compare" {
                    compare = args.get(i + 1).cloned(); i += 2;
                } else {
                    path = arg.to_string(); i += 1;
                }
            }
            if path.is_empty() {
                if compare.is_some() {
                    eprintln!("error: profile --compare requires a .fav file path");
                } else {
                    eprintln!("error: profile requires a .fav file");
                }
                process::exit(1);
            }
            if let Some(ref v) = compare {
                cmd_profile_compare(v, &path);
            } else {
                cmd_profile(&path, &format, runs, stage_filter.as_deref(), out.as_deref());
            }
        }

        Some("infer") => {
            let mut proto_path: Option<String> = None;
            let mut csv_path: Option<String> = None;
            let mut table_name: Option<String> = None;
            let mut db_conn: Option<String> = None;
            let mut out_path: Option<String> = None;
            let mut type_name: Option<String> = None;
            let mut from_source: Option<String> = None;
            let mut path_arg: Option<String> = None;
            let mut catalog_arg: Option<String> = None;
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
                    "--from" => {
                        from_source = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --from requires a source name");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    "--table" => {
                        table_name = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --table requires a table name");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    "--path" => {
                        path_arg = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --path requires a path");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    "--catalog" => {
                        catalog_arg = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --catalog requires a catalog URL");
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
            if from_source.as_deref() == Some("snowflake") {
                let table = table_name.as_deref().unwrap_or_else(|| {
                    eprintln!("error: --from snowflake requires --table <name>");
                    process::exit(1);
                });
                cmd_infer_snowflake(table, out_path.as_deref());
                return;
            }
            if from_source.as_deref() == Some("postgres") {
                let table = table_name.as_deref().unwrap_or_else(|| {
                    eprintln!("error: --from postgres requires --table <name>");
                    process::exit(1);
                });
                cmd_infer_postgres(table, out_path.as_deref());
                return;
            }
            if from_source.as_deref() == Some("delta") {
                let path = path_arg.as_deref().unwrap_or_else(|| {
                    eprintln!("error: --from delta requires --path <path>");
                    process::exit(1);
                });
                cmd_infer_delta(path, out_path.as_deref());
                return;
            }
            if from_source.as_deref() == Some("iceberg") {
                let catalog = catalog_arg.as_deref().unwrap_or_else(|| {
                    eprintln!("error: --from iceberg requires --catalog <url>");
                    process::exit(1);
                });
                let table = table_name.as_deref().unwrap_or_else(|| {
                    eprintln!("error: --from iceberg requires --table <name>");
                    process::exit(1);
                });
                cmd_infer_iceberg(catalog, table, out_path.as_deref());
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
            let mut from_effects = false;
            let mut dir: Option<String> = None;
            let mut file: Option<String> = None;
            let mut from_version: Option<String> = None;
            let mut to_version: Option<String> = None;
            let mut config_file: Option<String> = None;
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
                    "--from-effects" => {
                        from_effects = true;
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
                    "--from" => {
                        from_version = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --from requires a version argument");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    "--to" => {
                        to_version = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --to requires a version argument");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    "--config" => {
                        config_file = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --config requires a file path argument");
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
            cmd_migrate(
                file.as_deref(), in_place, dry_run, check, dir.as_deref(), from_effects,
                from_version.as_deref(), to_version.as_deref(), config_file.as_deref(),
            );
        }

        Some("upgrade") => {
            let arg_refs: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
            match cmd_upgrade(&arg_refs) {
                Ok(msg) => println!("{}", msg),
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            }
        }

        Some("explain-error") => {
            let mut list = false;
            let mut format = "text";
            let mut code: Option<&str> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--list" => {
                        list = true;
                        i += 1;
                    }
                    "--format" => {
                        if i + 1 < args.len() {
                            format = args[i + 1].as_str();
                            i += 2;
                        } else {
                            eprintln!("error: --format requires a value (text|json)");
                            process::exit(1);
                        }
                    }
                    other => {
                        code = Some(other);
                        i += 1;
                    }
                }
            }
            if list {
                if format == "json" {
                    cmd_explain_error_list_json();
                } else {
                    cmd_explain_error_list();
                }
            } else if let Some(c) = code {
                cmd_explain_error(c);
            } else {
                eprintln!("error: explain-error requires a code (e.g. E0213) or --list");
                process::exit(1);
            }
        }

        Some("install") => {
            let mut pkg_name: Option<String> = None;
            let mut force = false;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--force" => {
                        force = true;
                        i += 1;
                    }
                    other if !other.starts_with('-') => {
                        pkg_name = Some(other.to_string());
                        i += 1;
                    }
                    other => {
                        eprintln!("error: unexpected install argument `{}`", other);
                        process::exit(1);
                    }
                }
            }
            cmd_install(pkg_name.as_deref(), force);
        }

        Some("publish") => {
            let mut name_override: Option<String> = None;
            let mut version_override: Option<String> = None;
            let mut dry_run = false;
            let mut force = false;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--name" => {
                        name_override = args.get(i + 1).cloned();
                        i += 2;
                    }
                    "--version" => {
                        version_override = args.get(i + 1).cloned();
                        i += 2;
                    }
                    "--dry-run" => {
                        dry_run = true;
                        i += 1;
                    }
                    "--force" => {
                        force = true;
                        i += 1;
                    }
                    other => {
                        eprintln!("error: unexpected publish argument `{}`", other);
                        process::exit(1);
                    }
                }
            }
            cmd_publish(
                name_override.as_deref(),
                version_override.as_deref(),
                dry_run,
                force,
            );
        }

        Some("registry") => {
            let subcommand = args.get(2).map(|s| s.as_str());
            let sub_args: Vec<String> = args.iter().skip(3).cloned().collect();
            cmd_registry(subcommand, &sub_args);
        }

        Some("search") => {
            let query = args.get(2).map(|s| s.as_str()).unwrap_or("");
            cmd_search(query);
        }

        Some("info") => {
            let pkg_name = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| {
                eprintln!("error: `fav info` requires a package name");
                process::exit(1);
            });
            cmd_info(pkg_name);
        }

        Some("add") => {
            let mut dev = false;
            let mut pkg_arg: Option<String> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--dev" => { dev = true; i += 1; }
                    other => { pkg_arg = Some(other.to_string()); i += 1; }
                }
            }
            let pkg = pkg_arg.unwrap_or_else(|| {
                eprintln!("error: `fav add` requires a package name");
                process::exit(1);
            });
            cmd_add(&pkg, dev);
        }

        Some("update") => {
            let pkg_name = args.get(2).map(|s| s.as_str());
            cmd_update(pkg_name);
        }

        Some("remove") => {
            let pkg_name = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| {
                eprintln!("error: `fav remove` requires a package name");
                process::exit(1);
            });
            cmd_remove(pkg_name);
        }

        Some("login") => {
            cmd_login();
        }

        Some("rune") => {
            let sub_args: Vec<String> = args[2..].to_vec();
            cmd_rune(&sub_args);
        }

        Some("deploy") => {
            let mut env: Option<String> = None;
            let mut function_name: Option<String> = None;
            let mut region: Option<String> = None;
            let mut dry_run = false;
            let mut trigger_file: Option<String> = None;
            let mut target: Option<String>  = None;  // v22.8.0
            let mut out_dir: Option<String> = None;  // v22.8.0
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--env" => {
                        env = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --env requires a value");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    "--function" => {
                        function_name = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --function requires a value");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    "--region" => {
                        region = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --region requires a value");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    "--dry-run" => {
                        dry_run = true;
                        i += 1;
                    }
                    "--target" => {
                        target = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --target requires a value (ecs|k8s|fly|aws-lambda)");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    "--out-dir" => {
                        out_dir = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --out-dir requires a directory path");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    "--trigger" => {
                        trigger_file = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --trigger requires a file path");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    other => {
                        eprintln!("error: unexpected deploy argument `{}`", other);
                        process::exit(1);
                    }
                }
            }
            if let Some(ref tfile) = trigger_file {
                crate::driver::cmd_deploy_trigger(tfile, None);
            } else {
                cmd_deploy(
                    env.as_deref(),
                    function_name.as_deref(),
                    region.as_deref(),
                    dry_run,
                    target.as_deref(),
                    out_dir.as_deref(),
                );
            }
        }

        Some("orchestrate") => {
            match args.get(2).map(|s| s.as_str()) {
                Some("run") => {
                    let pipeline_name = args.get(3).unwrap_or_else(|| {
                        eprintln!("error: orchestrate run requires <PipelineName>");
                        process::exit(1);
                    });
                    let file = args.get(4).unwrap_or_else(|| {
                        eprintln!("error: orchestrate run requires <file>");
                        process::exit(1);
                    });
                    let dry_run = args.iter().any(|a| a == "--dry-run");
                    crate::driver::cmd_orchestrate_run(file, pipeline_name, dry_run);
                }
                Some("status") => {
                    let pipeline_name = args.get(3).unwrap_or_else(|| {
                        eprintln!("error: orchestrate status requires <PipelineName>");
                        process::exit(1);
                    });
                    crate::driver::cmd_orchestrate_status(pipeline_name);
                }
                Some("retry") => {
                    let step_name = args.get(3).unwrap_or_else(|| {
                        eprintln!("error: orchestrate retry requires <StepName>");
                        process::exit(1);
                    });
                    let pipeline_name = args.get(4).unwrap_or_else(|| {
                        eprintln!("error: orchestrate retry requires <PipelineName>");
                        process::exit(1);
                    });
                    let file = args.get(5).unwrap_or_else(|| {
                        eprintln!("error: orchestrate retry requires <file>");
                        process::exit(1);
                    });
                    crate::driver::cmd_orchestrate_retry(step_name, file, pipeline_name);
                }
                _ => {
                    eprintln!("usage: fav orchestrate run <PipelineName> <file> [--dry-run]");
                    eprintln!("       fav orchestrate status <PipelineName>");
                    eprintln!("       fav orchestrate retry <StepName> <PipelineName> <file>");
                    process::exit(1);
                }
            }
        }

        Some("mcp") => {
            mcp::run_mcp_server();
        }

        Some("notebook") => match args.get(2).map(|s| s.as_str()) {
            Some("new") => {
                let name = args.get(3).map(|s| s.as_str()).unwrap_or_else(|| {
                    eprintln!("error: notebook new requires <name>");
                    process::exit(1);
                });
                driver::cmd_notebook_new(name);
            }
            Some("run") => {
                let no_cache = args.iter().any(|a| a == "--no-cache");
                let file = args
                    .iter()
                    .skip(3)
                    .find(|a| !a.starts_with('-'))
                    .map(|s| s.as_str())
                    .unwrap_or_else(|| {
                        eprintln!("error: notebook run requires <file>");
                        process::exit(1);
                    });
                driver::cmd_notebook_run(file, no_cache);
            }
            Some("serve") => {
                let mut port = 8888u16;
                let mut no_open = false;
                let mut file: Option<&str> = None;
                let mut i = 3usize;
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
                let file = file.unwrap_or_else(|| {
                    eprintln!("error: notebook serve requires <file>");
                    process::exit(1);
                });
                driver::cmd_notebook_serve(file, port, no_open);
            }
            Some("export") => {
                let mut out: Option<&str> = None;
                let mut file: Option<&str> = None;
                let mut i = 3usize;
                while i < args.len() {
                    match args[i].as_str() {
                        "--out" => {
                            out = Some(args.get(i + 1).unwrap_or_else(|| {
                                eprintln!("error: --out requires a path");
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
                    eprintln!("error: notebook export requires <file>");
                    process::exit(1);
                });
                driver::cmd_notebook_export(file, out);
            }
            Some("check") => {
                let file = args.get(3).map(|s| s.as_str()).unwrap_or_else(|| {
                    eprintln!("error: notebook check requires <file>");
                    process::exit(1);
                });
                driver::cmd_notebook_check(file);
            }
            _ => {
                eprintln!("error: notebook requires new|run|serve|export|check");
                process::exit(1);
            }
        },

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

        Some("generate") => match args.get(2).map(|s| s.as_str()) {
            Some("api") => {
                let mut format = "openapi";
                let mut as_json = false;
                let mut out: Option<String> = None;
                let mut source: Option<String> = None;
                let mut i = 3usize;
                while i < args.len() {
                    match args[i].as_str() {
                        "--format" => {
                            format = match args.get(i + 1).map(|s| s.as_str()) {
                                Some("openapi") => "openapi",
                                Some("graphql") => "graphql",
                                other => {
                                    eprintln!("error: unknown format {:?}", other);
                                    process::exit(1);
                                }
                            };
                            i += 2;
                        }
                        "--json" => {
                            as_json = true;
                            i += 1;
                        }
                        "--out" | "-o" => {
                            out = Some(args.get(i + 1).cloned().unwrap_or_else(|| {
                                eprintln!("error: --out requires a file path");
                                process::exit(1);
                            }));
                            i += 2;
                        }
                        other => {
                            source = Some(other.to_string());
                            i += 1;
                        }
                    }
                }
                let src = source.unwrap_or_else(|| {
                    eprintln!("error: `fav generate api` requires a source file");
                    process::exit(1);
                });
                cmd_generate_api(&src, format, as_json, out.as_deref());
            }
            other => {
                eprintln!("error: unknown generate subcommand {:?}", other);
                process::exit(1);
            }
        }

        Some("api-serve") => {
            let mut port = 8080u16;
            let mut source: Option<String> = None;
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
                    other => {
                        source = Some(other.to_string());
                        i += 1;
                    }
                }
            }
            let src = source.unwrap_or_else(|| {
                eprintln!("error: `fav api-serve` requires a source file");
                process::exit(1);
            });
            cmd_api_serve(&src, port);
        }

        // ── v21.1.0: fav dap [--port N] ──────────────────────────────────────
        #[cfg(not(target_arch = "wasm32"))]
        Some("dap") => {
            let port = args
                .iter()
                .position(|a| a == "--port")
                .and_then(|i| args.get(i + 1))
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(5678);
            if let Err(e) = driver::cmd_dap(port) {
                eprintln!("error: {e}");
                process::exit(1);
            }
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
        println!("Favnir v5.0.0 - The pipeline-first language");
        println!();
    }
    print!("{}", HELP);
}
