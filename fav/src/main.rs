mod lexer;
mod ast;
mod parser;
mod checker;
mod eval;
mod toml;
mod resolver;

use std::path::{Path, PathBuf};
use std::process;
use std::sync::{Arc, Mutex};
use parser::Parser;
use checker::Checker;
use eval::Interpreter;
use toml::FavToml;
use resolver::Resolver;

// ── help text (4-6) ───────────────────────────────────────────────────────────

const HELP: &str = "\
fav - Favnir language interpreter v0.3.0

USAGE:
    fav <COMMAND> [OPTIONS] [FILE]

COMMANDS:
    run [--db <url>] [file]
                  Parse, type-check, and run a Favnir program.
                  If <file> is omitted, looks for fav.toml and runs src/main.fav.
    check [file]  Parse and type-check (no execution).
                  If <file> is omitted, checks all .fav files in the project.
    explain [file]
                  Show VIS / type / effect signatures of all top-level items.
                  If <file> is omitted, explains all files in the project.
    help          Show this help message

OPTIONS (run):
    --db <url>    SQLite database path (default: :memory:)
                  e.g. --db myapp.db  or  --db :memory:

SINGLE-FILE EXAMPLES:
    fav run examples/hello.fav
    fav run --db myapp.db examples/users.fav
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

        Some("check") => {
            let file = args.get(2).map(|s| s.as_str());
            cmd_check(file);
        }

        Some("explain") => {
            let file = args.get(2).map(|s| s.as_str());
            cmd_explain(file);
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

// ── file loading ──────────────────────────────────────────────────────────────

fn load_file(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("error: cannot read `{}`: {}", path, e);
        process::exit(1);
    })
}

// ── module loading ────────────────────────────────────────────────────────────

/// Load a program and all its transitive imports, returning a merged list of
/// items (dependencies first). Used so the evaluator can see all definitions.
fn load_all_items(
    entry_path: &str,
    toml: Option<&FavToml>,
    root: Option<&Path>,
) -> Vec<ast::Item> {
    use std::collections::HashSet;

    let mut visited: HashSet<String> = HashSet::new();
    let mut all_items: Vec<ast::Item> = Vec::new();

    fn load_rec(
        path: &str,
        toml: Option<&FavToml>,
        root: Option<&Path>,
        visited: &mut HashSet<String>,
        all_items: &mut Vec<ast::Item>,
    ) {
        if visited.contains(path) { return; }
        visited.insert(path.to_string());

        let source = std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("error: cannot read `{}`: {}", path, e);
            process::exit(1);
        });
        let program = Parser::parse_str(&source, path).unwrap_or_else(|e| {
            eprintln!("{}", e);
            process::exit(1);
        });

        // Recurse into imports first (dependency order)
        if let (Some(toml), Some(root)) = (toml, root) {
            let src_dir = toml.src_dir(root);
            for use_path in &program.uses {
                if use_path.len() < 2 { continue; }
                let mod_path = use_path[..use_path.len()-1].join(".");
                let rel: PathBuf = mod_path.split('.').collect();
                let dep_file = src_dir.join(rel).with_extension("fav");
                let dep_str = dep_file.to_string_lossy().to_string();
                load_rec(&dep_str, Some(toml), Some(root), visited, all_items);
            }
        }

        // Add this file's items (excluding namespace/use declarations)
        for item in program.items {
            match &item {
                ast::Item::NamespaceDecl(..) | ast::Item::UseDecl(..) => {}
                _ => all_items.push(item),
            }
        }
    }

    load_rec(entry_path, toml, root, &mut visited, &mut all_items);
    all_items
}

// ── project helpers ───────────────────────────────────────────────────────────

/// Find project root and entry point. Returns (entry_file, Option<(toml, root)>).
fn find_entry(file: Option<&str>) -> (String, Option<(FavToml, PathBuf)>) {
    if let Some(f) = file {
        return (f.to_string(), None);
    }
    // Project mode: look for fav.toml
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let root = FavToml::find_root(&cwd).unwrap_or_else(|| {
        eprintln!("error: no fav.toml found. Provide a file path or run from a project directory.");
        process::exit(1);
    });
    let toml = FavToml::load(&root).unwrap_or_else(|| {
        eprintln!("error: could not read fav.toml");
        process::exit(1);
    });
    let src_dir = toml.src_dir(&root);
    // Try src/main.fav then main.fav
    let entry = [src_dir.join("main.fav"), root.join("main.fav")]
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| {
            eprintln!("error: no main.fav found in `{}`", src_dir.display());
            process::exit(1);
        });
    (entry.to_string_lossy().to_string(), Some((toml, root)))
}

/// Collect all .fav files under a directory recursively.
fn collect_fav_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(collect_fav_files(&path));
            } else if path.extension().and_then(|e| e.to_str()) == Some("fav") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn make_resolver(toml: Option<FavToml>, root: Option<PathBuf>) -> Arc<Mutex<Resolver>> {
    Arc::new(Mutex::new(Resolver::new(toml, root)))
}

// ── fav run ───────────────────────────────────────────────────────────────────

fn cmd_run(file: Option<&str>, db_url: Option<&str>) {
    let (path, proj) = find_entry(file);
    let source = load_file(&path);

    // Parse entry file
    let program = Parser::parse_str(&source, &path).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    // Type-check (with resolver in project mode)
    let errors = if let Some((ref toml, ref root)) = proj {
        let r = make_resolver(Some(toml.clone()), Some(root.clone()));
        let mut checker = Checker::new_with_resolver(r, PathBuf::from(&path));
        checker.check_with_self(&program)
    } else {
        Checker::check_program(&program)
    };
    if !errors.is_empty() {
        for e in &errors { eprintln!("{}", e); }
        process::exit(1);
    }

    // Build a merged program for evaluation (all imports + entry file)
    let run_program = if let Some((ref toml, ref root)) = proj {
        let items = load_all_items(&path, Some(toml), Some(root));
        ast::Program { namespace: None, uses: vec![], items }
    } else {
        program
    };

    // Open Db connection
    let effective_url = db_url.unwrap_or(":memory:");
    let conn = rusqlite::Connection::open(effective_url).unwrap_or_else(|e| {
        eprintln!("error: cannot open database `{}`: {}", effective_url, e);
        process::exit(1);
    });

    // Interpret
    if let Err(e) = Interpreter::run_with_db(&run_program, conn) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

// ── fav check ─────────────────────────────────────────────────────────────────

fn cmd_check(file: Option<&str>) {
    if let Some(path) = file {
        // Single-file mode
        let source = load_file(path);
        let program = Parser::parse_str(&source, path).unwrap_or_else(|e| {
            eprintln!("{}", e);
            process::exit(1);
        });
        let errors = Checker::check_program(&program);
        if errors.is_empty() {
            println!("{}: no errors found", path);
        } else {
            for e in &errors { eprintln!("{}", e); }
            process::exit(1);
        }
    } else {
        // Project mode
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let root = FavToml::find_root(&cwd).unwrap_or_else(|| {
            eprintln!("error: no fav.toml found");
            process::exit(1);
        });
        let toml = FavToml::load(&root).unwrap_or_else(|| {
            eprintln!("error: could not read fav.toml");
            process::exit(1);
        });
        let src_dir = toml.src_dir(&root);
        let files = collect_fav_files(&src_dir);
        if files.is_empty() {
            println!("no .fav files found in `{}`", src_dir.display());
            return;
        }
        let resolver = make_resolver(Some(toml), Some(root));
        let mut total_errors = 0;
        for fav_file in &files {
            let path_str = fav_file.to_string_lossy().to_string();
            let source = load_file(&path_str);
            let program = match Parser::parse_str(&source, &path_str) {
                Ok(p)  => p,
                Err(e) => { eprintln!("{}", e); total_errors += 1; continue; }
            };
            let mut checker = Checker::new_with_resolver(resolver.clone(), fav_file.clone());
            let errors = checker.check_with_self(&program);
            if errors.is_empty() {
                println!("{}: ok", path_str);
            } else {
                for e in &errors { eprintln!("{}", e); }
                total_errors += errors.len();
            }
        }
        if total_errors > 0 {
            process::exit(1);
        }
    }
}

// ── fav explain ───────────────────────────────────────────────────────────────

fn cmd_explain(file: Option<&str>) {
    let paths: Vec<String> = if let Some(f) = file {
        vec![f.to_string()]
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let root = FavToml::find_root(&cwd).unwrap_or_else(|| {
            eprintln!("error: no fav.toml found");
            process::exit(1);
        });
        let toml = FavToml::load(&root).unwrap_or_else(|| {
            eprintln!("error: could not read fav.toml");
            process::exit(1);
        });
        collect_fav_files(&toml.src_dir(&root))
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect()
    };

    for path in &paths {
        let source = load_file(path);
        let program = Parser::parse_str(&source, path).unwrap_or_else(|e| {
            eprintln!("{}", e);
            process::exit(1);
        });
        let errors = Checker::check_program(&program);
        if !errors.is_empty() {
            eprintln!("warning: {} type error(s) in `{}` — output may be incomplete", errors.len(), path);
        }
        if paths.len() > 1 {
            println!("\n=== {} ===", path);
        }
        ExplainPrinter::new().print(&program);
    }
}

struct ExplainPrinter;

impl ExplainPrinter {
    fn new() -> Self { ExplainPrinter }

    fn print(&self, program: &ast::Program) {
        use ast::*;
        let col_vis   = 10usize;
        let col_name  = 26usize;
        let col_type  = 36usize;

        println!("{:<col_vis$} {:<col_name$} {:<col_type$} {}", "VIS", "NAME", "TYPE", "EFFECTS");
        println!("{}", "-".repeat(col_vis + col_name + col_type + 26));

        for item in &program.items {
            match item {
                Item::TypeDef(td) => {
                    let kind = match &td.body {
                        TypeBody::Record(_)  => "record",
                        TypeBody::Sum(_)     => "sum",
                    };
                    let vis = format_visibility(&td.visibility);
                    println!("{:<col_vis$} {:<col_name$} type ({:<col_type$} -",
                        vis, td.name,
                        format!("{})", kind),
                    );
                }
                Item::FnDef(fd) => {
                    let params: Vec<String> = fd.params.iter()
                        .map(|p| format_type_expr(&p.ty))
                        .collect();
                    let sig = format!("({}) -> {}", params.join(", "), format_type_expr(&fd.return_ty));
                    let effs = format_effects(&fd.effects);
                    let vis = format_visibility(&fd.visibility);
                    println!("{:<col_vis$} {:<col_name$} {:<col_type$} {}",
                        vis, format!("fn {}", fd.name), sig, effs);
                }
                Item::TrfDef(td) => {
                    let sig = format!("{} -> {}",
                        format_type_expr(&td.input_ty),
                        format_type_expr(&td.output_ty));
                    let effs = format_effects(&td.effects);
                    let vis = format_visibility(&td.visibility);
                    println!("{:<col_vis$} {:<col_name$} {:<col_type$} {}",
                        vis, format!("trf {}", td.name), sig, effs);
                }
                Item::FlwDef(fd) => {
                    println!("{:<col_vis$} {:<col_name$} {:<col_type$} -",
                        "", format!("flw {}", fd.name), fd.steps.join(" |> "));
                }
                Item::CapDef(cd) => {
                    println!("{:<col_vis$} {:<col_name$} {:<col_type$} -",
                        format_visibility(&cd.visibility),
                        format!("cap {}", cd.name),
                        format!("<{}>", cd.type_params.join(", ")));
                }
                Item::ImplDef(id) => {
                    let args: Vec<String> = id.type_args.iter()
                        .map(|a| format_type_expr(a))
                        .collect();
                    println!("{:<col_vis$} {:<col_name$} {:<col_type$} -",
                        "", format!("impl {}", id.cap_name), format!("<{}>", args.join(", ")));
                }
                Item::NamespaceDecl(..) | Item::UseDecl(..) => {}
            }
        }
    }
}

fn format_visibility(vis: &Option<ast::Visibility>) -> &'static str {
    match vis {
        Some(ast::Visibility::Public)   => "public",
        Some(ast::Visibility::Internal) => "internal",
        Some(ast::Visibility::Private)  => "private",
        None                            => "",
    }
}

fn format_type_expr(te: &ast::TypeExpr) -> String {
    use ast::TypeExpr::*;
    match te {
        Named(name, args, _) if args.is_empty() => name.clone(),
        Named(name, args, _) => {
            let s: Vec<_> = args.iter().map(format_type_expr).collect();
            format!("{}<{}>", name, s.join(", "))
        }
        Optional(inner, _) => format!("{}?", format_type_expr(inner)),
        Fallible(inner, _) => format!("{}!", format_type_expr(inner)),
        Arrow(a, b, _) => format!("{} -> {}", format_type_expr(a), format_type_expr(b)),
    }
}

fn format_effects(effects: &[ast::Effect]) -> String {
    use ast::Effect::*;
    if effects.is_empty() {
        return "Pure".into();
    }
    effects.iter().map(|e| match e {
        Pure           => "!Pure".into(),
        Io             => "!Io".into(),
        Db             => "!Db".into(),
        Network        => "!Network".into(),
        Emit(ev)       => format!("!Emit<{}>", ev),
        EmitUnion(evs) => format!("!Emit<{}>", evs.join("|")),
        Trace          => "!Trace".into(),
    }).collect::<Vec<_>>().join(" ")
}
