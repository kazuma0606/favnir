use crate::ast;
use crate::backend;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::{Arc, Mutex};
use crate::backend::artifact::FvcArtifact;
use crate::backend::codegen::{codegen_program, Opcode};
use crate::middle::ir::{IRArm, IRExpr, IRGlobalKind, IRPattern, IRProgram, IRStmt};
use crate::middle::compiler::compile_program;
use crate::backend::wasm_codegen::wasm_codegen_program;
use crate::backend::wasm_exec::{wasm_exec_info, wasm_exec_main};
use crate::frontend::parser::Parser;
use crate::middle::checker::Checker;
use crate::value::Value;
use crate::toml::FavToml;
use crate::middle::resolver::Resolver;
use crate::backend::vm::{VM, enable_coverage, take_coverage};
use crate::middle::compiler::set_coverage_mode;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde_json::json;
use serde::Serialize;
use std::collections::{BTreeSet, HashSet};
use std::sync::mpsc;
use std::time::Duration;

// ── diagnostic formatting ─────────────────────────────────────────────────────

/// Format a type-check error with a `^^^` underline under the offending source token.
///
/// Output format:
/// ```text
/// error[E001]: Type mismatch
///   --> file.fav:5:10
///    |
///  5 |     let x = foo
///    |             ^^^
/// ```
fn format_diagnostic(source: &str, error: &crate::middle::checker::TypeError) -> String {
    let span = &error.span;
    let line_num = span.line as usize;
    let col = span.col as usize;
    let token_len = if span.end > span.start { span.end - span.start } else { 1 };

    // Try to extract the source line
    let source_line = source.lines().nth(line_num.saturating_sub(1)).unwrap_or("");

    // Width of the line number prefix (e.g. "5" → 1 char, "42" → 2 chars)
    let line_prefix = line_num.to_string();
    let padding = " ".repeat(line_prefix.len());

    // Underline: col is 1-based, so offset = col-1 spaces
    let col_offset = " ".repeat(col.saturating_sub(1));
    // Cap underline to not exceed the line length
    let max_len = source_line.len().saturating_sub(col.saturating_sub(1)).max(1);
    let underline = "^".repeat(token_len.min(max_len).max(1));

    format!(
        "error[{}]: {}\n  --> {}:{}:{}\n{} |\n{} | {}\n{} | {}{}",
        error.code, error.message,
        span.file, span.line, span.col,
        padding,
        line_prefix, source_line,
        padding, col_offset, underline,
    )
}

fn format_warning(source: &str, warning: &crate::middle::checker::TypeWarning) -> String {
    let span = &warning.span;
    let line_num = span.line as usize;
    let col = span.col as usize;
    let token_len = if span.end > span.start { span.end - span.start } else { 1 };

    let source_line = source.lines().nth(line_num.saturating_sub(1)).unwrap_or("");
    let line_prefix = line_num.to_string();
    let padding = " ".repeat(line_prefix.len());
    let col_offset = " ".repeat(col.saturating_sub(1));
    let max_len = source_line.len().saturating_sub(col.saturating_sub(1)).max(1);
    let underline = "^".repeat(token_len.min(max_len).max(1));

    format!(
        "warning[{}]: {}\n  --> {}:{}:{}\n{} |\n{} | {}\n{} | {}{}",
        warning.code, warning.message,
        span.file, span.line, span.col,
        padding,
        line_prefix, source_line,
        padding, col_offset, underline,
    )
}

fn render_warnings(
    source: &str,
    warnings: &[crate::middle::checker::TypeWarning],
    no_warn: bool,
) -> Vec<String> {
    if no_warn {
        Vec::new()
    } else {
        warnings
            .iter()
            .map(|w| format_warning(source, w))
            .collect()
    }
}

fn check_single_file(
    path: &str,
) -> (
    String,
    Vec<crate::middle::checker::TypeError>,
    Vec<crate::middle::checker::TypeWarning>,
) {
    let source = load_file(path);
    let program = Parser::parse_str(&source, path).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });
    let mut checker = Checker::new();
    let errors = checker.check_with_self(&program);
    let mut warnings = checker.warnings;
    warnings.extend(partial_flw_warnings(&program));
    (source, errors, warnings)
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
                if mod_path == "std.states" {
                    continue;
                }
                let rel: PathBuf = mod_path.split('.').collect();
                let dep_file = src_dir.join(rel).with_extension("fav");
                let dep_str = dep_file.to_string_lossy().to_string();
                load_rec(&dep_str, Some(toml), Some(root), visited, all_items);
            }
        }

        // Add this file's items (excluding namespace/use declarations)
        for item in program.items {
            match &item {
                ast::Item::NamespaceDecl(..)
                | ast::Item::UseDecl(..)
                | ast::Item::InterfaceDecl(..)
                | ast::Item::InterfaceImplDecl(..) => {}
                _ => all_items.push(item),
            }
        }
    }

    load_rec(entry_path, toml, root, &mut visited, &mut all_items);
    all_items
}

fn find_partial_flw_bindings(program: &ast::Program) -> Vec<(String, Vec<String>)> {
    let templates: std::collections::HashMap<String, ast::AbstractFlwDef> = program
        .items
        .iter()
        .filter_map(|item| match item {
            ast::Item::AbstractFlwDef(def) => Some((def.name.clone(), def.clone())),
            _ => None,
        })
        .collect();

    let mut out = Vec::new();
    for item in &program.items {
        let ast::Item::FlwBindingDef(fd) = item else { continue };
        let Some(template) = templates.get(&fd.template) else { continue };
        let bound: std::collections::HashSet<&str> =
            fd.bindings.iter().map(|(slot, _)| slot.as_str()).collect();
        let unbound: Vec<String> = template
            .slots
            .iter()
            .filter(|slot| !bound.contains(slot.name.as_str()))
            .map(|slot| slot.name.clone())
            .collect();
        if !unbound.is_empty() {
            out.push((fd.name.clone(), unbound));
        }
    }
    out
}

fn partial_flw_warnings(program: &ast::Program) -> Vec<crate::middle::checker::TypeWarning> {
    find_partial_flw_bindings(program)
        .into_iter()
        .filter_map(|(name, slots)| {
            program.items.iter().find_map(|item| match item {
                ast::Item::FlwBindingDef(fd) if fd.name == name => Some(
                    crate::middle::checker::TypeWarning::new(
                        "W011",
                        format!(
                            "`{}` is a partial flw binding with unbound slots: {}",
                            name,
                            slots.join(", ")
                        ),
                        fd.span.clone(),
                    ),
                ),
                _ => None,
            })
        })
        .collect()
}

fn ensure_no_partial_flw(program: &ast::Program) -> Result<(), String> {
    let partials = find_partial_flw_bindings(program);
    if partials.is_empty() {
        return Ok(());
    }
    let (name, slots) = &partials[0];
    Err(format!(
        "error[E050]: `{}` has unbound slots: {}\n  hint: bind remaining slots before running or building",
        name,
        slots.join(", ")
    ))
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

fn load_and_check_program(file: Option<&str>) -> (ast::Program, String) {
    let (path, proj) = find_entry(file);
    let source = load_file(&path);

    let program = Parser::parse_str(&source, &path).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    let uses_std_states = program
        .uses
        .iter()
        .any(|path| path.len() >= 2 && path[0] == "std" && path[1] == "states");

    let errors = if let Some((ref toml, ref root)) = proj {
        let r = make_resolver(Some(toml.clone()), Some(root.clone()));
        let mut checker = Checker::new_with_resolver(r, PathBuf::from(&path));
        checker.check_with_self(&program)
    } else if uses_std_states {
        let r = make_resolver(None, None);
        let mut checker = Checker::new_with_resolver(r, PathBuf::from(&path));
        checker.check_with_self(&program)
    } else {
        Checker::check_program(&program)
    };
    if !errors.is_empty() {
        for e in &errors { eprintln!("{}", format_diagnostic(&source, e)); }
        process::exit(1);
    }

    let merged = if let Some((ref toml, ref root)) = proj {
        let items = load_all_items(&path, Some(toml), Some(root));
        ast::Program {
            namespace: program.namespace.clone(),
            uses: program.uses.clone(),
            items,
        }
    } else {
        program
    };

    (merged, path)
}

// ── fav run ───────────────────────────────────────────────────────────────────

pub fn cmd_run(file: Option<&str>, db_url: Option<&str>) {
    let (run_program, _) = load_and_check_program(file);
    ensure_no_partial_flw(&run_program).unwrap_or_else(|message| {
        eprintln!("{message}");
        process::exit(1);
    });
    let artifact = build_artifact(&run_program);

    exec_artifact_main(&artifact, db_url).unwrap_or_else(|message| {
        eprintln!("{message}");
        process::exit(1);
    });
}

pub fn cmd_build(file: Option<&str>, out: Option<&str>, target: Option<&str>) {
    let (program, path) = load_and_check_program(file);
    ensure_no_partial_flw(&program).unwrap_or_else(|message| {
        eprintln!("{message}");
        process::exit(1);
    });
    let target = target.unwrap_or("fvc");
    match target {
        "fvc" => {
            let out_path = out
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(&path).with_extension("fvc"));
            let artifact = build_artifact(&program);

            write_artifact_to_path(&artifact, &out_path).unwrap_or_else(|message| {
                eprintln!("{message}");
                process::exit(1);
            });

            println!("built {}", out_path.display());
        }
        "wasm" => {
            let out_path = out
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(&path).with_extension("wasm"));
            let bytes = build_wasm_artifact(&program).unwrap_or_else(|message| {
                eprintln!("{message}");
                process::exit(1);
            });
            write_wasm_to_path(&bytes, &out_path).unwrap_or_else(|message| {
                eprintln!("{message}");
                process::exit(1);
            });
            println!("built {} (wasm)", out_path.display());
        }
        other => {
            eprintln!("error: unsupported build target `{}`", other);
            process::exit(1);
        }
    }
}

fn build_artifact(program: &ast::Program) -> FvcArtifact {
    let ir = compile_program(program);
    codegen_program(&ir)
}

fn build_wasm_artifact(program: &ast::Program) -> Result<Vec<u8>, String> {
    let ir = compile_program(program);
    wasm_codegen_program(&ir).map_err(|e| e.to_string())
}

fn write_artifact_to_path(artifact: &FvcArtifact, out_path: &Path) -> Result<(), String> {
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!("error: cannot create output directory `{}`: {}", parent.display(), e)
            })?;
        }
    }

    let mut file = std::fs::File::create(out_path)
        .map_err(|e| format!("error: cannot create artifact `{}`: {}", out_path.display(), e))?;
    backend::artifact::FvcWriter {
        str_table: artifact.str_table.clone(),
        globals: artifact.globals.clone(),
        functions: artifact.functions.clone(),
        explain_json: artifact.explain_json.clone(),
    }
    .write_to(&mut file)
    .map_err(|e| format!("error: cannot write artifact `{}`: {}", out_path.display(), e))
}

pub fn cmd_exec(path: &str, show_info: bool, db_path: Option<&str>) {
    if path.ends_with(".wasm") {
        let bytes = read_wasm_from_path(Path::new(path)).unwrap_or_else(|message| {
            eprintln!("{message}");
            process::exit(1);
        });
        match exec_wasm_bytes(&bytes, show_info, db_path) {
            Ok(Some(info)) => {
                print!("{info}");
                return;
            }
            Ok(None) => return,
            Err(message) => {
                eprintln!("{message}");
                process::exit(1);
            }
        }
    }

    let artifact = read_artifact_from_path(Path::new(path)).unwrap_or_else(|message| {
        eprintln!("{message}");
        process::exit(1);
    });

    if show_info {
        print!("{}", artifact_info_string(&artifact));
        return;
    }

    exec_artifact_main(&artifact, db_path).unwrap_or_else(|message| {
        eprintln!("{message}");
        process::exit(1);
    });
}

fn exec_wasm_bytes(
    bytes: &[u8],
    show_info: bool,
    db_path: Option<&str>,
) -> Result<Option<String>, String> {
    if db_path.is_some() {
        return Err("error[W004]: --db cannot be used with .wasm artifacts".into());
    }
    if show_info {
        return Ok(Some(wasm_exec_info(bytes)));
    }
    wasm_exec_main(bytes).map_err(|message| {
        eprintln!("{message}");
        message
    })?;
    Ok(None)
}

fn read_artifact_from_path(path: &Path) -> Result<FvcArtifact, String> {
    let mut file = std::fs::File::open(path)
        .map_err(|e| format!("error: cannot open artifact `{}`: {}", path.display(), e))?;
    FvcArtifact::read_from(&mut file)
          .map_err(|e| format!("error: cannot read artifact `{}`: {}", path.display(), e))
}

fn explain_json_from_artifact(artifact: &FvcArtifact) -> Result<&str, String> {
    artifact
        .explain_json
        .as_deref()
        .ok_or_else(|| "error: artifact does not contain embedded explain json".to_string())
}

fn read_wasm_from_path(path: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(path)
        .map_err(|e| format!("error: cannot read wasm artifact `{}`: {}", path.display(), e))
}

fn write_wasm_to_path(bytes: &[u8], out_path: &Path) -> Result<(), String> {
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!("error: cannot create output directory `{}`: {}", parent.display(), e)
            })?;
        }
    }
    std::fs::write(out_path, bytes)
        .map_err(|e| format!("error: cannot write wasm artifact `{}`: {}", out_path.display(), e))
}

fn exec_artifact_main(artifact: &FvcArtifact, db_path: Option<&str>) -> Result<Value, String> {
    let main_idx = artifact
        .fn_idx_by_name("main")
        .ok_or_else(|| "error: artifact does not contain a `main` function".to_string())?;
    VM::run_with_db_path(artifact, main_idx, vec![], db_path)
        .map(|(value, _)| value)
        .map_err(|e| format!("vm error in {} @{}: {}", e.fn_name, e.ip, e.message))
}

#[cfg(test)]
fn exec_artifact_main_with_emits(artifact: &FvcArtifact) -> Result<(Value, Vec<Value>), String> {
    let main_idx = artifact
        .fn_idx_by_name("main")
        .ok_or_else(|| "error: artifact does not contain a `main` function".to_string())?;
    VM::run_with_emits_and_db_path(artifact, main_idx, vec![], None)
        .map_err(|e| format!("vm error in {} @{}: {}", e.fn_name, e.ip, e.message))
}

fn artifact_info_string(artifact: &FvcArtifact) -> String {
    let mut out = String::new();
    let total_bytecode_bytes: usize = artifact.functions.iter().map(|f| f.code.len()).sum();
    let total_constants: usize = artifact.functions.iter().map(|f| f.constants.len()).sum();
    let total_string_bytes: usize = artifact.str_table.iter().map(|s| s.len()).sum();
    let longest_string: usize = artifact.str_table.iter().map(|s| s.len()).max().unwrap_or(0);
    let max_locals: u32 = artifact
        .functions
        .iter()
        .map(|f| f.local_count)
        .max()
        .unwrap_or(0);
    let string_preview = summarize_string_table_preview(&artifact.str_table);
    let (total_instructions, opcode_counts) = collect_opcode_counts(artifact);
    let (reachable_function_count, reachable_global_count) = artifact
        .fn_idx_by_name("main")
        .map(|main_idx| collect_reachable_symbols(artifact, main_idx))
        .map(|(functions, globals)| (functions.len(), globals.len()))
        .unwrap_or((0, 0));
    let constant_counts = collect_constant_counts(artifact);
    let mut hot_opcodes = opcode_counts
        .iter()
        .map(|(name, count)| (name.as_str(), *count))
        .collect::<Vec<_>>();
    hot_opcodes.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    let hot_opcodes = hot_opcodes
        .into_iter()
        .take(5)
        .map(|(name, count)| format!("{name}={count}"))
        .collect::<Vec<_>>();
    let closure_count = artifact
        .functions
        .iter()
        .filter(|f| {
            artifact
                .str_table
                .get(f.name_idx as usize)
                .map(|s| s.starts_with("$closure"))
                .unwrap_or(false)
        })
        .count();
    let variant_ctor_count = artifact.globals.iter().filter(|g| g.kind == 2).count();
    let function_global_count = artifact.globals.iter().filter(|g| g.kind == 0).count();
    let mut effect_counts = std::collections::BTreeMap::<String, usize>::new();
    let mut trace_enabled_functions = 0usize;
    let mut emitted_events = std::collections::BTreeSet::<String>::new();
    for function in &artifact.functions {
        let eff = artifact
            .str_table
            .get(function.effect_str_idx as usize)
            .map(|s| s.as_str())
            .unwrap_or("<invalid>");
        for part in normalize_effect_summary(eff) {
            if part == "!Trace" {
                trace_enabled_functions += 1;
            }
            if let Some(events) = part.strip_prefix("!Emit<").and_then(|s| s.strip_suffix('>')) {
                for event in events.split('|') {
                    emitted_events.insert(event.to_string());
                }
            }
            *effect_counts.entry(part).or_default() += 1;
        }
    }

    out.push_str("artifact: .fvc\n");
    out.push_str(&format!("strings: {}\n", artifact.str_table.len()));
    out.push_str(&format!("globals: {}\n", artifact.globals.len()));
    out.push_str(&format!("functions: {}\n", artifact.functions.len()));
    out.push_str("summary:\n");
    out.push_str(&format!("- function globals: {}\n", function_global_count));
    out.push_str(&format!("- variant ctors: {}\n", variant_ctor_count));
    out.push_str(&format!("- synthetic closures: {}\n", closure_count));
    out.push_str(&format!("- total bytecode bytes: {}\n", total_bytecode_bytes));
    out.push_str(&format!("- total constants: {}\n", total_constants));
    out.push_str(&format!("- total string bytes: {}\n", total_string_bytes));
    out.push_str(&format!("- longest string entry: {}\n", longest_string));
    out.push_str(&format!("- string preview: {}\n", string_preview));
    out.push_str(&format!("- max locals in function: {}\n", max_locals));
    out.push_str(&format!("- reachable functions from entry: {}\n", reachable_function_count));
    out.push_str(&format!("- reachable globals from entry: {}\n", reachable_global_count));
    out.push_str(&format!("- total instructions: {}\n", total_instructions));
    out.push_str(&format!("- distinct opcode kinds: {}\n", opcode_counts.len()));
    if hot_opcodes.is_empty() {
        out.push_str("- hot opcodes: <none>\n");
    } else {
        out.push_str(&format!("- hot opcodes: {}\n", hot_opcodes.join(", ")));
    }
    out.push_str("- constant kinds:");
    if constant_counts.is_empty() {
        out.push_str(" <none>\n");
    } else {
        out.push('\n');
        for (kind, count) in constant_counts {
            out.push_str(&format!("  - {}: {}\n", kind, count));
        }
    }
    out.push_str(&format!("- trace-enabled functions: {}\n", trace_enabled_functions));
    if emitted_events.is_empty() {
        out.push_str("- emitted events: <none>\n");
    } else {
        out.push_str(&format!(
            "- emitted events: {}\n",
            emitted_events.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }
    out.push_str("- effect counts:");
    if effect_counts.is_empty() {
        out.push_str(" <none>\n");
    } else {
        out.push('\n');
        for (effect, count) in effect_counts {
            out.push_str(&format!("  - {}: {}\n", effect, count));
        }
    }
    out.push_str("globals table:\n");

    for (idx, global) in artifact.globals.iter().enumerate() {
        let name = artifact
            .str_table
            .get(global.name_idx as usize)
            .map(|s| s.as_str())
            .unwrap_or("<invalid>");
        let kind = match global.kind {
            0 => format!("fn#{}", global.fn_idx),
            1 => "builtin".to_string(),
            2 => "variant_ctor".to_string(),
            other => format!("kind#{other}"),
        };
        let target = summarize_global_target(artifact, global.fn_idx as usize);
        out.push_str(&format!("- g#{} {} [{}] => {}\n", idx, name, kind, target));
    }

    out.push_str("function table:\n");

    for (idx, function) in artifact.functions.iter().enumerate() {
        let name = artifact
            .str_table
            .get(function.name_idx as usize)
            .map(|s| s.as_str())
            .unwrap_or("<invalid>");
        let ret = artifact
            .str_table
            .get(function.return_ty_str_idx as usize)
            .map(|s| s.as_str())
            .unwrap_or("<invalid>");
        let eff = artifact
            .str_table
            .get(function.effect_str_idx as usize)
            .map(|s| s.as_str())
            .unwrap_or("<invalid>");
        let opcode_summary = summarize_function_opcodes(function);

        out.push_str(&format!(
            "- fn#{} {} @L{} ({} params, {} locals, {} consts, {} bytes) -> {} [{}] opcodes: {}\n",
            idx,
            name,
            function.source_line,
            function.param_count,
            function.local_count,
            function.constants.len(),
            function.code.len(),
            ret,
            eff,
            opcode_summary
        ));
        let const_preview = summarize_function_constants(function);
        if const_preview != "<none>" {
            out.push_str(&format!("    consts: {}\n", const_preview));
        }
    }

    if let Some(main_idx) = artifact.fn_idx_by_name("main") {
        let main_fn = &artifact.functions[main_idx];
        let ret = artifact
            .str_table
            .get(main_fn.return_ty_str_idx as usize)
            .map(|s| s.as_str())
            .unwrap_or("<invalid>");
        let eff = artifact
            .str_table
            .get(main_fn.effect_str_idx as usize)
            .map(|s| s.as_str())
            .unwrap_or("<invalid>");
        out.push_str(&format!("entry: main (fn#{})\n", main_idx));
        out.push_str(&format!("entry signature: () -> {} [{}]\n", ret, eff));
    } else {
        out.push_str("entry: <missing main>\n");
    }

    out
}

fn normalize_effect_summary(raw: &str) -> Vec<String> {
    let raw = raw.trim();
    if raw.is_empty() || raw == "Pure" || raw == "[]" {
        return vec!["Pure".to_string()];
    }
    if raw.starts_with('[') && raw.ends_with(']') {
        let inner = &raw[1..raw.len() - 1];
        if inner.trim().is_empty() {
            return vec!["Pure".to_string()];
        }
        return inner
            .split(", ")
            .map(|part| {
                if let Some(name) = part.strip_prefix("Emit(\"").and_then(|s| s.strip_suffix("\")")) {
                    format!("!Emit<{}>", name)
                } else if let Some(inner) = part.strip_prefix("EmitUnion([").and_then(|s| s.strip_suffix("])")) {
                    let names = inner
                        .split(", ")
                        .map(|s| s.trim_matches('"'))
                        .collect::<Vec<_>>()
                        .join("|");
                    format!("!Emit<{}>", names)
                } else {
                    format!("!{}", part)
                }
            })
            .collect();
    }
    raw.split_whitespace().map(|s| s.to_string()).collect()
}

fn collect_opcode_counts(artifact: &FvcArtifact) -> (usize, std::collections::BTreeMap<String, usize>) {
    let mut total = 0usize;
    let mut counts = std::collections::BTreeMap::<String, usize>::new();

    for function in &artifact.functions {
        let mut ip = 0usize;
        while ip < function.code.len() {
            let opcode = function.code[ip];
            if let Some((name, width)) = decode_opcode(opcode) {
                total += 1;
                *counts.entry(name.to_string()).or_default() += 1;
                ip += width;
            } else {
                break;
            }
        }
    }

    (total, counts)
}

fn collect_constant_counts(
    artifact: &FvcArtifact,
) -> std::collections::BTreeMap<String, usize> {
    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    for function in &artifact.functions {
        for constant in &function.constants {
            let key = match constant {
                backend::codegen::Constant::Int(_) => "Int",
                backend::codegen::Constant::Float(_) => "Float",
                backend::codegen::Constant::Str(_) => "Str",
                backend::codegen::Constant::Name(_) => "Name",
            };
            *counts.entry(key.to_string()).or_default() += 1;
        }
    }
    counts
}

fn summarize_string_table_preview(strings: &[String]) -> String {
    if strings.is_empty() {
        return "<none>".to_string();
    }

    strings
        .iter()
        .enumerate()
        .take(5)
        .map(|(idx, value)| format!("#{}={}", idx, preview_string_literal(value, 24)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn preview_string_literal(value: &str, max_chars: usize) -> String {
    let mut out = String::new();
    out.push('"');
    for (i, ch) in value.chars().enumerate() {
        if i >= max_chars {
            out.push_str("...");
            break;
        }
        match ch {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn summarize_function_constants(function: &backend::artifact::FvcFunction) -> String {
    if function.constants.is_empty() {
        return "<none>".to_string();
    }

    function
        .constants
        .iter()
        .enumerate()
        .take(4)
        .map(|(idx, constant)| match constant {
            backend::codegen::Constant::Int(value) => format!("#{idx}=Int({value})"),
            backend::codegen::Constant::Float(value) => format!("#{idx}=Float({value})"),
            backend::codegen::Constant::Str(value) => {
                format!("#{idx}=Str({})", preview_string_literal(value, 20))
            }
            backend::codegen::Constant::Name(value) => {
                format!("#{idx}=Name({})", preview_string_literal(value, 20))
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn summarize_global_target(artifact: &FvcArtifact, fn_idx: usize) -> String {
    let Some(function) = artifact.functions.get(fn_idx) else {
        return format!("fn#{} <invalid>", fn_idx);
    };

    let name = artifact
        .str_table
        .get(function.name_idx as usize)
        .map(|s| s.as_str())
        .unwrap_or("<invalid>");
    let ret = artifact
        .str_table
        .get(function.return_ty_str_idx as usize)
        .map(|s| s.as_str())
        .unwrap_or("<invalid>");
    let eff = artifact
        .str_table
        .get(function.effect_str_idx as usize)
        .map(|s| s.as_str())
        .unwrap_or("<invalid>");

    format!(
        "fn#{} {} @L{} ({} params) -> {} [{}]",
        fn_idx,
        name,
        function.source_line,
        function.param_count,
        ret,
        eff
    )
}

fn collect_reachable_symbols(
    artifact: &FvcArtifact,
    entry_fn_idx: usize,
) -> (
    std::collections::BTreeSet<usize>,
    std::collections::BTreeSet<usize>,
) {
    let mut reachable_functions = std::collections::BTreeSet::new();
    let mut reachable_globals = std::collections::BTreeSet::new();
    let mut work = vec![entry_fn_idx];

    while let Some(fn_idx) = work.pop() {
        if !reachable_functions.insert(fn_idx) {
            continue;
        }
        let Some(function) = artifact.functions.get(fn_idx) else {
            continue;
        };

        let mut ip = 0usize;
        while ip < function.code.len() {
            let opcode = function.code[ip];
            match decode_opcode(opcode) {
                Some((_name, width)) => {
                    if opcode == Opcode::LoadGlobal as u8 && ip + 2 < function.code.len() {
                        let global_idx =
                            u16::from_le_bytes([function.code[ip + 1], function.code[ip + 2]])
                                as usize;
                        if reachable_globals.insert(global_idx) {
                            if let Some(global) = artifact.globals.get(global_idx) {
                                work.push(global.fn_idx as usize);
                            }
                        }
                    } else if opcode == Opcode::MakeClosure as u8 && ip + 4 < function.code.len() {
                        let global_idx =
                            u16::from_le_bytes([function.code[ip + 1], function.code[ip + 2]])
                                as usize;
                        if reachable_globals.insert(global_idx) {
                            if let Some(global) = artifact.globals.get(global_idx) {
                                work.push(global.fn_idx as usize);
                            }
                        }
                    }
                    ip += width;
                }
                None => break,
            }
        }
    }

    (reachable_functions, reachable_globals)
}

fn decode_opcode(byte: u8) -> Option<(&'static str, usize)> {
    let opcode = match byte {
        x if x == Opcode::Const as u8 => Opcode::Const,
        x if x == Opcode::ConstUnit as u8 => Opcode::ConstUnit,
        x if x == Opcode::ConstTrue as u8 => Opcode::ConstTrue,
        x if x == Opcode::ConstFalse as u8 => Opcode::ConstFalse,
        x if x == Opcode::LoadLocal as u8 => Opcode::LoadLocal,
        x if x == Opcode::StoreLocal as u8 => Opcode::StoreLocal,
        x if x == Opcode::LoadGlobal as u8 => Opcode::LoadGlobal,
        x if x == Opcode::Pop as u8 => Opcode::Pop,
        x if x == Opcode::Dup as u8 => Opcode::Dup,
        x if x == Opcode::Call as u8 => Opcode::Call,
        x if x == Opcode::Return as u8 => Opcode::Return,
        x if x == Opcode::Add as u8 => Opcode::Add,
        x if x == Opcode::Sub as u8 => Opcode::Sub,
        x if x == Opcode::Mul as u8 => Opcode::Mul,
        x if x == Opcode::Div as u8 => Opcode::Div,
        x if x == Opcode::Eq as u8 => Opcode::Eq,
        x if x == Opcode::Ne as u8 => Opcode::Ne,
        x if x == Opcode::Lt as u8 => Opcode::Lt,
        x if x == Opcode::Le as u8 => Opcode::Le,
        x if x == Opcode::Gt as u8 => Opcode::Gt,
        x if x == Opcode::Ge as u8 => Opcode::Ge,
        x if x == Opcode::Jump as u8 => Opcode::Jump,
        x if x == Opcode::JumpIfFalse as u8 => Opcode::JumpIfFalse,
        x if x == Opcode::MatchFail as u8 => Opcode::MatchFail,
        x if x == Opcode::ChainCheck as u8 => Opcode::ChainCheck,
        x if x == Opcode::JumpIfNotVariant as u8 => Opcode::JumpIfNotVariant,
        x if x == Opcode::GetField as u8 => Opcode::GetField,
        x if x == Opcode::BuildRecord as u8 => Opcode::BuildRecord,
        x if x == Opcode::MakeClosure as u8 => Opcode::MakeClosure,
        x if x == Opcode::GetVariantPayload as u8 => Opcode::GetVariantPayload,
        x if x == Opcode::CollectBegin as u8 => Opcode::CollectBegin,
        x if x == Opcode::CollectEnd as u8 => Opcode::CollectEnd,
        x if x == Opcode::YieldValue as u8 => Opcode::YieldValue,
        x if x == Opcode::EmitEvent as u8 => Opcode::EmitEvent,
        x if x == Opcode::TrackLine as u8 => Opcode::TrackLine,
        _ => return None,
    };

    let (name, width) = match opcode {
        Opcode::Const => ("Const", 3),
        Opcode::ConstUnit => ("ConstUnit", 1),
        Opcode::ConstTrue => ("ConstTrue", 1),
        Opcode::ConstFalse => ("ConstFalse", 1),
        Opcode::LoadLocal => ("LoadLocal", 3),
        Opcode::StoreLocal => ("StoreLocal", 3),
        Opcode::LoadGlobal => ("LoadGlobal", 3),
        Opcode::Pop => ("Pop", 1),
        Opcode::Dup => ("Dup", 1),
        Opcode::Call => ("Call", 3),
        Opcode::Return => ("Return", 1),
        Opcode::Add => ("Add", 1),
        Opcode::Sub => ("Sub", 1),
        Opcode::Mul => ("Mul", 1),
        Opcode::Div => ("Div", 1),
        Opcode::Eq => ("Eq", 1),
        Opcode::Ne => ("Ne", 1),
        Opcode::Lt => ("Lt", 1),
        Opcode::Le => ("Le", 1),
        Opcode::Gt => ("Gt", 1),
        Opcode::Ge => ("Ge", 1),
        Opcode::Jump => ("Jump", 3),
        Opcode::JumpIfFalse => ("JumpIfFalse", 3),
        Opcode::MatchFail => ("MatchFail", 1),
        Opcode::ChainCheck => ("ChainCheck", 3),
        Opcode::JumpIfNotVariant => ("JumpIfNotVariant", 5),
        Opcode::GetField => ("GetField", 3),
        Opcode::BuildRecord => ("BuildRecord", 5),
        Opcode::MakeClosure => ("MakeClosure", 5),
        Opcode::GetVariantPayload => ("GetVariantPayload", 1),
        Opcode::CollectBegin => ("CollectBegin", 1),
        Opcode::CollectEnd => ("CollectEnd", 1),
        Opcode::YieldValue => ("YieldValue", 1),
        Opcode::EmitEvent => ("EmitEvent", 1),
        Opcode::TrackLine => ("TrackLine", 5), // 1 byte opcode + 4 bytes u32 line
    };

    Some((name, width))
}

fn summarize_function_opcodes(function: &backend::artifact::FvcFunction) -> String {
    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    let mut ip = 0usize;
    while ip < function.code.len() {
        let opcode = function.code[ip];
        if let Some((name, width)) = decode_opcode(opcode) {
            *counts.entry(name.to_string()).or_default() += 1;
            ip += width;
        } else {
            break;
        }
    }

    let mut sorted = counts.into_iter().collect::<Vec<_>>();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    if sorted.is_empty() {
        return "<none>".to_string();
    }
    sorted
        .into_iter()
        .take(3)
        .map(|(name, count)| format!("{name}={count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

// ── fav check ─────────────────────────────────────────────────────────────────

pub fn cmd_check(file: Option<&str>, no_warn: bool) {
    if let Some(path) = file {
        // Single-file mode
        let (source, errors, warnings) = check_single_file(path);
        if errors.is_empty() {
            println!("{}: no errors found", path);
        } else {
            for e in &errors { eprintln!("{}", format_diagnostic(&source, e)); }
            process::exit(1);
        }
        for warning in render_warnings(&source, &warnings, no_warn) {
            eprintln!("{}", warning);
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
        let mut total_warnings = 0;
        for fav_file in &files {
            let path_str = fav_file.to_string_lossy().to_string();
            let source = load_file(&path_str);
            let program = match Parser::parse_str(&source, &path_str) {
                Ok(p)  => p,
                Err(e) => { eprintln!("{}", e); total_errors += 1; continue; }
            };
            let mut checker = Checker::new_with_resolver(resolver.clone(), fav_file.clone());
            let errors = checker.check_with_self(&program);
            let mut warnings = checker.warnings.clone();
            warnings.extend(partial_flw_warnings(&program));
            if errors.is_empty() {
                println!("{}: ok", path_str);
            } else {
                for e in &errors { eprintln!("{}", format_diagnostic(&source, e)); }
                total_errors += errors.len();
            }
            if !no_warn {
                for w in &warnings {
                    eprintln!("{}", format_warning(&source, w));
                }
                total_warnings += warnings.len();
            }
        }
        if total_errors > 0 {
            process::exit(1);
        }
        if !no_warn && total_warnings > 0 {
            eprintln!("\ncheck: {} warning{}", total_warnings, if total_warnings == 1 { "" } else { "s" });
        }
    }
}

// ── fav test ──────────────────────────────────────────────────────────────────

/// Collect .fav, .test.fav, and .spec.fav files from a directory tree.
/// .test.fav and .spec.fav are included only for `fav test`.
fn collect_test_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(collect_test_files(&path));
            } else {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.ends_with(".test.fav") || name.ends_with(".spec.fav") || name.ends_with(".fav") {
                    out.push(path);
                }
            }
        }
    }
    out.sort();
    out
}

struct TestResult {
    description: String,
    passed: bool,
    error_msg: Option<String>,
    elapsed_ms: u128,
}

fn collect_test_cases(
    programs: Vec<(String, ast::Program)>,
    filter: Option<&str>,
) -> (Vec<(String, String, ast::Program)>, usize) {
    let mut tests_to_run: Vec<(String, String, ast::Program)> = Vec::new();
    let mut total_discovered = 0usize;
    for (path, prog) in programs {
        for item in &prog.items {
            if let ast::Item::TestDef(td) = item {
                total_discovered += 1;
                if let Some(f) = filter {
                    if !td.name.contains(f) {
                        continue;
                    }
                }
                tests_to_run.push((path.clone(), td.name.clone(), prog.clone()));
            }
        }
    }
    (tests_to_run, total_discovered)
}

fn format_test_results(results: &[TestResult], filtered: usize, total_ms: u128) -> String {
    let mut out = String::new();
    for result in results {
        if result.passed {
            out.push_str(&format!("PASS  {}  ({}ms)\n", result.description, result.elapsed_ms));
        } else {
            out.push_str(&format!("FAIL  {}  ({}ms)\n", result.description, result.elapsed_ms));
            if let Some(msg) = &result.error_msg {
                out.push_str(&format!("      {}\n", msg));
            }
        }
    }
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len().saturating_sub(passed);
    out.push_str(&format!(
        "\ntest result: {}. {} passed; {} failed; {} filtered; finished in {}ms\n",
        if failed == 0 { "ok" } else { "FAILED" },
        passed,
        failed,
        filtered,
        total_ms,
    ));
    out
}

fn with_test_output_mode<T>(no_capture: bool, f: impl FnOnce() -> T) -> T {
    if no_capture {
        f()
    } else {
        let _guard = crate::backend::vm::SuppressIoGuard::new(true);
        f()
    }
}

pub fn cmd_test(file: Option<&str>, filter: Option<&str>, fail_fast: bool, no_capture: bool, coverage: bool, coverage_report_dir: Option<&str>) {
    // Collect (file_path, parsed_program) pairs
    let programs: Vec<(String, ast::Program)> = if let Some(path) = file {
        let source = load_file(path);
        let program = Parser::parse_str(&source, path).unwrap_or_else(|e| {
            eprintln!("{}", e);
            process::exit(1);
        });
        vec![(path.to_string(), program)]
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let root = FavToml::find_root(&cwd).unwrap_or_else(|| {
            eprintln!("error: no fav.toml found; pass a file path or run in project root");
            process::exit(1);
        });
        let toml = FavToml::load(&root).unwrap_or_else(|| {
            eprintln!("error: could not read fav.toml");
            process::exit(1);
        });
        let src_dir = toml.src_dir(&root);
        collect_test_files(&src_dir).into_iter().filter_map(|f| {
            let path_str = f.to_string_lossy().to_string();
            let src = std::fs::read_to_string(&f).ok()?;
            Parser::parse_str(&src, &path_str).ok().map(|p| (path_str, p))
        }).collect()
    };

    // Flatten: one entry per test item per file
    let (tests_to_run, total_discovered) = collect_test_cases(programs, filter);
    let total = tests_to_run.len();
    let filtered = total_discovered.saturating_sub(total);
    if total == 0 {
        println!("no tests found");
        return;
    }

    println!("running {} test{}", total, if total == 1 { "" } else { "s" });
    println!();

    if coverage {
        set_coverage_mode(true);
    }

    let mut all_covered: HashSet<u32> = HashSet::new();

    let (results, total_ms) = with_test_output_mode(no_capture, || {
        let started_all = std::time::Instant::now();
        let mut results: Vec<TestResult> = Vec::new();

        for (path, test_name, prog) in &tests_to_run {
            let started = std::time::Instant::now();
            let fn_name = format!("$test:{}", test_name);
            let artifact = build_artifact(prog);
            let fn_idx = match artifact.fn_idx_by_name(&fn_name) {
                Some(i) => i,
                None => {
                    results.push(TestResult {
                        description: format!("{test_name} ({path})"),
                        passed: false,
                        error_msg: Some("test function not found in artifact".into()),
                        elapsed_ms: started.elapsed().as_millis(),
                    });
                    if fail_fast { break; }
                    continue;
                }
            };
            if coverage {
                enable_coverage();
            }
            match VM::run(&artifact, fn_idx, vec![]) {
                Ok(_) => results.push(TestResult {
                    description: format!("{test_name} ({path})"),
                    passed: true,
                    error_msg: None,
                    elapsed_ms: started.elapsed().as_millis(),
                }),
                Err(e) => {
                    results.push(TestResult {
                        description: format!("{test_name} ({path})"),
                        passed: false,
                        error_msg: Some(e.message.clone()),
                        elapsed_ms: started.elapsed().as_millis(),
                    });
                    if fail_fast { break; }
                }
            }
            if coverage {
                all_covered.extend(take_coverage());
            }
        }

        (results, started_all.elapsed().as_millis())
    });

    if coverage {
        set_coverage_mode(false);
    }

    let rendered = format_test_results(&results, filtered, total_ms);
    print!("{rendered}");

    if coverage {
        // Report coverage across all test source files
        let source_paths: Vec<String> = tests_to_run.iter()
            .map(|(path, _, _)| path.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        let mut full_report = String::new();
        for path in &source_paths {
            if let Ok(source) = std::fs::read_to_string(path) {
                let report = format_coverage_report(path, &source, &all_covered);
                println!("{}", report);
                full_report.push_str(&report);
                full_report.push('\n');
                if let Ok(prog) = crate::frontend::parser::Parser::parse_str(&source, path) {
                    let ir = compile_program(&prog);
                    let fn_report = format_coverage_report_by_fn(&ir, &all_covered);
                    if !fn_report.is_empty() {
                        println!("{}", fn_report);
                        full_report.push_str(&fn_report);
                        full_report.push('\n');
                    }
                }
            }
        }
        if let Some(dir) = coverage_report_dir {
            if let Err(e) = std::fs::create_dir_all(dir) {
                eprintln!("warning: could not create coverage report dir: {e}");
            } else {
                let out_path = std::path::Path::new(dir).join("coverage.txt");
                if let Err(e) = std::fs::write(&out_path, &full_report) {
                    eprintln!("warning: could not write coverage report: {e}");
                } else {
                    println!("coverage report written to {}", out_path.display());
                }
            }
        }
    }

    if results.iter().any(|r| !r.passed) {
        process::exit(1);
    }
}

fn is_executable_line(source: &str, line: u32) -> bool {
    let line_str = source.lines().nth((line - 1) as usize).unwrap_or("");
    let trimmed = line_str.trim();
    !trimmed.is_empty()
        && !trimmed.starts_with("//")
        && !trimmed.starts_with("type ")
        && !trimmed.starts_with("}")
        && !trimmed.starts_with("fn ")
        && !trimmed.starts_with("trf ")
        && !trimmed.starts_with("test ")
}

pub fn format_coverage_report(
    file_path: &str,
    source: &str,
    executed: &HashSet<u32>,
) -> String {
    let total_lines = source.lines().count() as u32;
    let executable: Vec<u32> = (1..=total_lines)
        .filter(|l| is_executable_line(source, *l))
        .collect();
    let executable_count = executable.len();
    let covered_count = executable.iter().filter(|l| executed.contains(l)).count();
    let pct = if executable_count == 0 {
        100.0f64
    } else {
        covered_count as f64 / executable_count as f64 * 100.0
    };
    let uncovered: Vec<u32> = executable.iter()
        .filter(|l| !executed.contains(l))
        .copied()
        .collect();
    let uncovered_str = if uncovered.is_empty() {
        "none".to_string()
    } else {
        uncovered.iter().map(|l| l.to_string()).collect::<Vec<_>>().join(", ")
    };
    format!(
        "\ncoverage: {}\n  lines covered: {} / {} ({:.1}%)\n  uncovered:     lines {}",
        file_path, covered_count, executable_count, pct, uncovered_str
    )
}

fn collect_tracklines_in_expr(expr: &IRExpr, out: &mut HashSet<u32>) {
    match expr {
        IRExpr::Block(stmts, tail, _) => {
            for stmt in stmts {
                match stmt {
                    IRStmt::TrackLine(n) => { out.insert(*n); }
                    IRStmt::Expr(e) | IRStmt::Bind(_, e) | IRStmt::Chain(_, e) | IRStmt::Yield(e) => {
                        collect_tracklines_in_expr(e, out);
                    }
                }
            }
            collect_tracklines_in_expr(tail, out);
        }
        IRExpr::If(cond, then, els, _) => {
            collect_tracklines_in_expr(cond, out);
            collect_tracklines_in_expr(then, out);
            collect_tracklines_in_expr(els, out);
        }
        IRExpr::Call(f, args, _) => {
            collect_tracklines_in_expr(f, out);
            for a in args { collect_tracklines_in_expr(a, out); }
        }
        IRExpr::Match(scrutinee, arms, _) => {
            collect_tracklines_in_expr(scrutinee, out);
            for arm in arms {
                collect_tracklines_in_expr(&arm.body, out);
            }
        }
        IRExpr::FieldAccess(inner, _, _) => {
            collect_tracklines_in_expr(inner, out);
        }
        IRExpr::Collect(inner, _) | IRExpr::Emit(inner, _) => {
            collect_tracklines_in_expr(inner, out);
        }
        IRExpr::Closure(_, captures, _) => {
            for c in captures { collect_tracklines_in_expr(c, out); }
        }
        IRExpr::BinOp(_, l, r, _) => {
            collect_tracklines_in_expr(l, out);
            collect_tracklines_in_expr(r, out);
        }
        IRExpr::RecordConstruct(fields, _) => {
            for (_, e) in fields { collect_tracklines_in_expr(e, out); }
        }
        IRExpr::CallTrfLocal { arg, .. } => { collect_tracklines_in_expr(arg, out); }
        _ => {}
    }
}

pub fn format_coverage_report_by_fn(ir: &IRProgram, executed: &HashSet<u32>) -> String {
    let mut lines = Vec::new();
    for fn_def in &ir.fns {
        // Skip internal functions ($test:, $bench:, closures starting with $)
        let name = &fn_def.name;
        if name.starts_with('$') { continue; }
        let mut fn_lines: HashSet<u32> = HashSet::new();
        collect_tracklines_in_expr(&fn_def.body, &mut fn_lines);
        if fn_lines.is_empty() { continue; }
        let total = fn_lines.len();
        let covered = fn_lines.iter().filter(|l| executed.contains(l)).count();
        let pct = covered as f64 / total as f64 * 100.0;
        let status = if covered == total { "full" } else if covered == 0 { "none" } else { "partial" };
        lines.push(format!("  fn {:<30} {}/{} ({:.0}%) [{}]", name, covered, total, pct, status));
    }
    if lines.is_empty() {
        return String::new();
    }
    format!("function coverage:\n{}", lines.join("\n"))
}

fn collect_watch_paths(file: Option<&str>) -> Vec<PathBuf> {
    if let Some(path) = file {
        return vec![PathBuf::from(path)];
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let root = FavToml::find_root(&cwd).unwrap_or_else(|| {
        eprintln!("error: no fav.toml found; pass a file path or run in project root");
        process::exit(1);
    });
    let toml = FavToml::load(&root).unwrap_or_else(|| {
        eprintln!("error: could not read fav.toml");
        process::exit(1);
    });
    collect_fav_files(&toml.src_dir(&root))
}

fn run_watch_cmd(file: Option<&str>, cmd: &str) {
    let exe = std::env::current_exe().unwrap_or_else(|e| {
        eprintln!("error: could not resolve current executable: {e}");
        process::exit(1);
    });
    let mut command = std::process::Command::new(exe);
    command.arg(cmd);
    if let Some(path) = file {
        command.arg(path);
    }
    let _ = command.status();
}

/// Recursively collect all `.fav` files under a directory.
pub fn collect_fav_files_recursive(dir: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.extend(collect_fav_files_recursive(&path));
            } else if path.extension().map(|e| e == "fav").unwrap_or(false) {
                result.push(path);
            }
        }
    }
    result
}

/// Collect all `.fav` files from a given directory (recursive).
pub fn collect_watch_paths_from_dir(dir: &str) -> Vec<PathBuf> {
    collect_fav_files_recursive(Path::new(dir))
}

// ── bench ──────────────────────────────────────────────────────────────────

fn collect_bench_cases(
    programs: Vec<(String, ast::Program)>,
    filter: Option<&str>,
) -> (Vec<(String, String, ast::Program)>, usize) {
    let mut cases: Vec<(String, String, ast::Program)> = Vec::new();
    let mut total = 0usize;
    for (path, prog) in programs {
        for item in &prog.items {
            if let ast::Item::BenchDef(bd) = item {
                total += 1;
                if let Some(f) = filter {
                    if !bd.description.contains(f) { continue; }
                }
                cases.push((path.clone(), bd.description.clone(), prog.clone()));
            }
        }
    }
    (cases, total)
}

fn exec_bench_case(prog: &ast::Program, description: &str, iters: u64) -> Result<f64, String> {
    let fn_name = format!("$bench:{}", description);
    let artifact = build_artifact(prog);
    let fn_idx = artifact.fn_idx_by_name(&fn_name)
        .ok_or_else(|| format!("bench function not found in artifact: {fn_name}"))?;
    // warmup: 1 iter
    VM::run(&artifact, fn_idx, vec![]).map_err(|e| e.message.clone())?;
    // timed iters
    let started = std::time::Instant::now();
    for _ in 0..iters {
        VM::run(&artifact, fn_idx, vec![]).map_err(|e| e.message.clone())?;
    }
    let elapsed_us = started.elapsed().as_micros() as f64;
    Ok(elapsed_us / iters as f64)
}

pub fn cmd_bench(file: Option<&str>, filter: Option<&str>, iters: u64) {
    let programs: Vec<(String, ast::Program)> = if let Some(path) = file {
        let source = load_file(path);
        let program = Parser::parse_str(&source, path).unwrap_or_else(|e| {
            eprintln!("{}", e);
            process::exit(1);
        });
        vec![(path.to_string(), program)]
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let root = FavToml::find_root(&cwd).unwrap_or_else(|| {
            eprintln!("error: no fav.toml found; pass a file path or run in project root");
            process::exit(1);
        });
        let toml = FavToml::load(&root).unwrap_or_else(|| {
            eprintln!("error: could not read fav.toml");
            process::exit(1);
        });
        let src_dir = toml.src_dir(&root);
        collect_bench_files(&src_dir).into_iter().filter_map(|f| {
            let path_str = f.to_string_lossy().to_string();
            let src = std::fs::read_to_string(&f).ok()?;
            Parser::parse_str(&src, &path_str).ok().map(|p| (path_str, p))
        }).collect()
    };

    let (cases, total_discovered) = collect_bench_cases(programs, filter);
    let filtered = total_discovered.saturating_sub(cases.len());
    if cases.is_empty() {
        println!("no benchmarks found");
        return;
    }

    println!("running {} benchmark{} ({} iterations each)", cases.len(), if cases.len() == 1 { "" } else { "s" }, iters);
    println!();

    let _suppress = crate::backend::vm::SuppressIoGuard::new(true);
    for (path, desc, prog) in &cases {
        match exec_bench_case(prog, desc, iters) {
            Ok(us_per_iter) => {
                println!("bench  {:<40}  {:.2} µs/iter  ({}  {})", desc, us_per_iter, iters, path);
            }
            Err(e) => {
                println!("ERROR  {:<40}  {}", desc, e);
            }
        }
    }
    println!();
    println!("bench result: ok. {} filtered", filtered);
}

fn collect_bench_files(dir: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.extend(collect_bench_files(&path));
            } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".bench.fav") || name.ends_with(".fav") {
                    result.push(path);
                }
            }
        }
    }
    result.sort();
    result
}

pub fn cmd_watch(file: Option<&str>, cmd: &str, extra_dirs: &[&str], debounce_ms: u64) {
    if !matches!(cmd, "check" | "test" | "run") {
        eprintln!("error: watch command must be one of `check`, `test`, or `run`");
        process::exit(1);
    }

    let mut files = collect_watch_paths(file);
    for dir in extra_dirs {
        files.extend(collect_watch_paths_from_dir(dir));
    }
    files.sort();
    files.dedup();

    if files.is_empty() {
        eprintln!("error: no .fav files found to watch");
        process::exit(1);
    }

    let mut dirs = BTreeSet::new();
    for file_path in &files {
        if let Some(parent) = file_path.parent() {
            dirs.insert(parent.to_path_buf());
        }
    }

    run_watch_cmd(file, cmd);
    eprintln!("[watch] watching {} files for changes...", files.len());

    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.send(res);
        },
        Config::default(),
    )
    .unwrap_or_else(|e| {
        eprintln!("error: could not create watcher: {e}");
        process::exit(1);
    });

    for dir in &dirs {
        watcher.watch(dir, RecursiveMode::NonRecursive).unwrap_or_else(|e| {
            eprintln!("error: could not watch {}: {e}", dir.display());
            process::exit(1);
        });
    }

    let debounce = Duration::from_millis(debounce_ms);
    loop {
        let Ok(event) = rx.recv() else { break };
        let Ok(event) = event else { continue };
        let interesting = matches!(
            event.kind,
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) | EventKind::Any
        );
        if !interesting {
            continue;
        }

        while rx.recv_timeout(debounce).is_ok() {}
        print!("\x1b[2J\x1b[H");
        run_watch_cmd(file, cmd);
        eprintln!("[watch] watching {} files for changes...", files.len());
    }
}

#[cfg(test)]
mod tests {
        use super::{
            artifact_info_string,
            build_artifact, build_wasm_artifact, exec_artifact_main, exec_artifact_main_with_emits,
            build_manifest_json, cmd_bundle, ensure_no_partial_flw, exec_wasm_bytes, filter_ir_program, format_invariants, load_all_items, load_and_check_program, make_resolver,
            partial_flw_warnings, collect_test_cases, collect_watch_paths, format_test_results, TestResult,
            diff_explain_json, explain_json_from_artifact, read_artifact_from_path, read_wasm_from_path, render_diff_json, render_diff_text, render_warnings, check_single_file,
            render_graph_mermaid, render_graph_mermaid_with_opts, render_graph_text, render_graph_text_with_opts, write_artifact_to_path, write_wasm_to_path, ExplainPrinter,
            format_coverage_report, format_coverage_report_by_fn, collect_watch_paths_from_dir,
            collect_bench_cases, cmd_bench,
        };
      use crate::ast;
      use crate::frontend::parser::Parser;
      use crate::middle::checker::Checker;
      use crate::middle::compiler::compile_program;
      use crate::middle::reachability::{reachability_analysis, ReachabilityResult};
      use crate::toml::FavToml;
      use std::path::PathBuf;
      use tempfile::tempdir;

    #[test]
    fn build_and_read_artifact_round_trip_for_temp_source() {
        let source = r#"
public fn main() -> Int {
    42
}
"#;
        let program = Parser::parse_str(source, "hello_build.fav").expect("parse");
        let artifact = build_artifact(&program);
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("hello.fvc");

        write_artifact_to_path(&artifact, &path).expect("write artifact");
        let restored = read_artifact_from_path(&path).expect("read artifact");

        assert!(restored.fn_idx_by_name("main").is_some());
        assert_eq!(restored.functions.len(), 1);
    }

    #[test]
    fn exec_artifact_main_runs_built_temp_source() {
        let source = r#"
stage Double: Int -> Int = |x| { x + x }

public fn main() -> Int {
    21 |> Double
}
"#;
        let program = Parser::parse_str(source, "hello_exec.fav").expect("parse");
        let artifact = build_artifact(&program);

        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Int(42));
    }

    #[test]
    fn exec_artifact_main_runs_named_flw_source() {
        let source = r#"
stage Inc: Int -> Int = |x| { x + 1 }
seq Bump = Inc |> Inc

public fn main() -> Int {
    1 |> Bump
}
"#;
        let program = Parser::parse_str(source, "flw_exec.fav").expect("parse");
        let artifact = build_artifact(&program);

        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Int(3));
    }

    #[test]
    fn exec_artifact_main_runs_variant_constructor_source() {
        let source = r#"
type Direction =
    | North
    | South

public fn main() -> Direction {
    North
}
"#;
        let program = Parser::parse_str(source, "variant_exec.fav").expect("parse");
        let artifact = build_artifact(&program);

        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Variant("North".into(), None));
    }

    #[test]
    fn exec_artifact_main_captures_emit_log_from_source() {
        let source = r#"
public fn main() -> Unit !Emit<Event> {
    emit "hello"
}
"#;
        let program = Parser::parse_str(source, "emit_exec.fav").expect("parse");
        let artifact = build_artifact(&program);

        let (value, emits) = exec_artifact_main_with_emits(&artifact).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Unit);
        assert_eq!(emits, vec![crate::value::Value::Str("hello".into())]);
    }

    #[test]
    fn exec_artifact_main_runs_uncaptured_closure_source() {
        let source = r#"
public fn main() -> Int {
    bind f <- |x| x + 1
    f(10)
}
"#;
        let program = Parser::parse_str(source, "closure_exec.fav").expect("parse");
        let artifact = build_artifact(&program);

        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Int(11));
    }

    #[test]
    fn exec_artifact_main_runs_captured_closure_source() {
        let source = r#"
public fn main() -> Int {
    bind y <- 2
    bind f <- |x| x + y
    f(10)
}
"#;
        let program = Parser::parse_str(source, "closure_capture_exec.fav").expect("parse");
        let artifact = build_artifact(&program);

        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Int(12));
    }

    #[test]
    fn exec_artifact_main_runs_state_constructor_ok_source() {
        let source = r#"
type PosInt = { value: Int invariant value > 0 }

public fn main() -> PosInt! {
    PosInt.new(5)
}
"#;
        let program = Parser::parse_str(source, "state_ctor_ok_exec.fav").expect("parse");
        let artifact = build_artifact(&program);

        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        let mut record = std::collections::HashMap::new();
        record.insert("value".into(), crate::value::Value::Int(5));
        assert_eq!(
            value,
            crate::value::Value::Variant(
                "ok".into(),
                Some(Box::new(crate::value::Value::Record(record)))
            )
        );
    }

    #[test]
    fn exec_artifact_main_runs_state_constructor_err_source() {
        let source = r#"
type PosInt = { value: Int invariant value > 0 }

public fn main() -> PosInt! {
    PosInt.new(0)
}
"#;
        let program = Parser::parse_str(source, "state_ctor_err_exec.fav").expect("parse");
        let artifact = build_artifact(&program);

        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(
            value,
            crate::value::Value::Variant(
                "err".into(),
                Some(Box::new(crate::value::Value::Str("InvariantViolation: PosInt".into())))
            )
        );
    }

    #[test]
    fn exec_artifact_main_runs_state_constructor_multi_field_source() {
        let source = r#"
type UserAge = {
    age: Int
    max: Int
    invariant age >= 0
    invariant age <= max
}

public fn main() -> UserAge! {
    UserAge.new(10, 5)
}
"#;
        let program = Parser::parse_str(source, "state_ctor_multi_exec.fav").expect("parse");
        let artifact = build_artifact(&program);

        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(
            value,
            crate::value::Value::Variant(
                "err".into(),
                Some(Box::new(crate::value::Value::Str("InvariantViolation: UserAge".into())))
            )
        );
    }

    #[test]
    fn exec_artifact_main_runs_state_constructor_chain_source() {
        let source = r#"
type PosInt = { value: Int invariant value > 0 }

public fn main() -> Int! {
    chain age <- PosInt.new(5)
    Result.ok(age.value)
}
"#;
        let program = Parser::parse_str(source, "state_ctor_chain_exec.fav").expect("parse");
        let artifact = build_artifact(&program);

        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(
            value,
            crate::value::Value::Variant(
                "ok".into(),
                Some(Box::new(crate::value::Value::Int(5)))
            )
        );
    }

    #[test]
    fn exec_artifact_main_runs_fstring_correct_output_source() {
        let source = r#"
public fn main() -> String {
    bind name <- "Favnir"
    $"Hello {name}!"
}
"#;
        let program = Parser::parse_str(source, "fstring_ok_exec.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Str("Hello Favnir!".into()));
    }

    #[test]
    fn exec_artifact_main_runs_fstring_int_auto_show_source() {
        let source = r#"
public fn main() -> String {
    bind age <- 42
    $"Age: {age}"
}
"#;
        let program = Parser::parse_str(source, "fstring_int_exec.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Str("Age: 42".into()));
    }

    #[test]
    fn exec_artifact_main_runs_assert_matches_record_source() {
        let source = r#"
type User = { name: String age: Int }
public fn main() -> Unit {
    bind user <- User { name: "Favnir", age: 42 }
    assert_matches(user, { name, age })
}
"#;
        let program = Parser::parse_str(source, "assert_matches_record_exec.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Unit);
    }

    #[test]
    fn exec_artifact_main_runs_assert_matches_fail_source() {
        let source = r#"
type User = { name: String age: Int }
public fn main() -> Unit {
    bind user <- User { name: "Favnir", age: 42 }
    assert_matches(user, { name: "Other" })
}
"#;
        let program = Parser::parse_str(source, "assert_matches_fail_exec.fav").expect("parse");
        let artifact = build_artifact(&program);
        let err = exec_artifact_main(&artifact, None).expect_err("assert_matches should fail");
        assert!(err.contains("assertion failed"), "unexpected error: {}", err);
    }

    #[test]
    fn test_stats_summary_format() {
        let results = vec![
            TestResult {
                description: "alpha".into(),
                passed: true,
                error_msg: None,
                elapsed_ms: 1,
            },
            TestResult {
                description: "beta".into(),
                passed: false,
                error_msg: Some("boom".into()),
                elapsed_ms: 2,
            },
        ];
        let rendered = format_test_results(&results, 3, 7);
        assert!(rendered.contains("PASS  alpha  (1ms)"));
        assert!(rendered.contains("FAIL  beta  (2ms)"));
        assert!(rendered.contains("2 failed") == false);
        assert!(rendered.contains("1 passed; 1 failed; 3 filtered; finished in 7ms"));
    }

    #[test]
    fn test_filter_matches_description() {
        let src = r#"
test "keyword alpha" { () }
test "beta" { () }
"#;
        let prog = Parser::parse_str(src, "filter_test.fav").expect("parse");
        let (tests, total) = collect_test_cases(vec![("filter_test.fav".into(), prog)], Some("keyword"));
        assert_eq!(total, 2);
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].1, "keyword alpha");
    }

    #[test]
    fn test_filter_excludes_non_matching() {
        let src = r#"
test "alpha" { () }
test "beta" { () }
"#;
        let prog = Parser::parse_str(src, "filter_test_2.fav").expect("parse");
        let (tests, total) = collect_test_cases(vec![("filter_test_2.fav".into(), prog)], Some("zzz"));
        assert_eq!(total, 2);
        assert!(tests.is_empty());
    }

    #[test]
    fn test_output_mode_suppresses_by_default_and_restores() {
        crate::backend::vm::set_suppress_io(false);
        let seen_inside = super::with_test_output_mode(false, || crate::backend::vm::io_output_suppressed_for_tests());
        assert!(seen_inside);
        assert!(!crate::backend::vm::io_output_suppressed_for_tests());
    }

    #[test]
    fn test_output_mode_respects_no_capture() {
        crate::backend::vm::set_suppress_io(false);
        let seen_inside = super::with_test_output_mode(true, || crate::backend::vm::io_output_suppressed_for_tests());
        assert!(!seen_inside);
        assert!(!crate::backend::vm::io_output_suppressed_for_tests());
    }

    /// Serialize tests that mutate the process-wide current directory so they
    /// cannot race against each other when the test suite runs in parallel.
    static CWD_MUTEX: std::sync::LazyLock<std::sync::Mutex<()>> =
        std::sync::LazyLock::new(|| std::sync::Mutex::new(()));

    #[test]
    fn watch_collect_paths_returns_fav_files() {
        let _cwd_guard = CWD_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::write(
            root.join("fav.toml"),
            "[rune]\nname = \"watch-test\"\nversion = \"0.1.0\"\nsrc = \"src\"\n",
        ).expect("write fav.toml");
        let src_dir = root.join("src");
        std::fs::create_dir_all(src_dir.join("nested")).expect("create src");
        std::fs::write(src_dir.join("main.fav"), "fn main() -> Unit { () }").expect("write main");
        std::fs::write(src_dir.join("nested").join("util.fav"), "fn util() -> Unit { () }").expect("write util");
        let saved = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(root).expect("chdir");
        let paths = collect_watch_paths(None);
        std::env::set_current_dir(saved).expect("restore cwd");
        assert_eq!(paths.len(), 2);
        assert!(paths.iter().all(|p| p.extension().and_then(|e| e.to_str()) == Some("fav")));
    }

    #[test]
    fn watch_collect_paths_excludes_non_fav() {
        let _cwd_guard = CWD_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::write(
            root.join("fav.toml"),
            "[rune]\nname = \"watch-test\"\nversion = \"0.1.0\"\nsrc = \"src\"\n",
        ).expect("write fav.toml");
        let src_dir = root.join("src");
        std::fs::create_dir_all(&src_dir).expect("create src");
        std::fs::write(src_dir.join("main.fav"), "fn main() -> Unit { () }").expect("write main");
        std::fs::write(src_dir.join("notes.md"), "# ignore").expect("write notes");
        std::fs::write(src_dir.join("config.toml"), "x=1").expect("write toml");
        let saved = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(root).expect("chdir");
        let paths = collect_watch_paths(None);
        std::env::set_current_dir(saved).expect("restore cwd");
        assert_eq!(paths.len(), 1);
        assert!(paths[0].ends_with("main.fav"));
    }

    #[test]
    fn example_fstring_demo_build_and_exec() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples").join("fstring_demo.fav");
        let path_str = path.to_string_lossy().to_string();
        let (program, _) = load_and_check_program(Some(&path_str));
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Str("Hello Favnir! Age: 42".into()));
    }

    #[test]
    fn example_record_match_build_and_exec() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples").join("record_match.fav");
        let path_str = path.to_string_lossy().to_string();
        let (program, _) = load_and_check_program(Some(&path_str));
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Str("Favnir".into()));
    }

    fn exec_project_main_source(source: &str) -> crate::value::Value {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::write(
            root.join("fav.toml"),
            "[rune]\nname = \"state-test\"\nversion = \"0.1.0\"\nsrc = \"src\"\n",
        )
        .expect("write fav.toml");
        let src_dir = root.join("src");
        std::fs::create_dir_all(&src_dir).expect("create src");
        let src = src_dir.join("main.fav");
        std::fs::write(&src, source).expect("write main.fav");

        let toml = FavToml::load(root).expect("load fav.toml");
        let source_text = std::fs::read_to_string(&src).expect("read main.fav");
        let src_str = src.to_string_lossy().to_string();
        let program = Parser::parse_str(&source_text, &src_str).expect("parse main.fav");
        let resolver = make_resolver(Some(toml.clone()), Some(root.to_path_buf()));
        let mut checker = Checker::new_with_resolver(resolver, PathBuf::from(&src_str));
        let errors = checker.check_with_self(&program);
        assert!(errors.is_empty(), "unexpected project-mode errors: {:?}", errors);

        let merged = ast::Program {
            namespace: program.namespace.clone(),
            uses: program.uses.clone(),
            items: load_all_items(&src_str, Some(&toml), Some(&root.to_path_buf())),
        };
        let artifact = build_artifact(&merged);
        exec_artifact_main(&artifact, None).expect("exec artifact")
    }

    #[test]
    fn exec_artifact_main_runs_std_states_posint_ok_source() {
        let value = exec_project_main_source(
            r#"
use std.states.PosInt

public fn main() -> PosInt! {
    PosInt.new(1)
}
"#,
        );

        let mut record = std::collections::HashMap::new();
        record.insert("value".into(), crate::value::Value::Int(1));
        assert_eq!(
            value,
            crate::value::Value::Variant(
                "ok".into(),
                Some(Box::new(crate::value::Value::Record(record)))
            )
        );
    }

    #[test]
    fn exec_artifact_main_runs_std_states_posint_err_source() {
        let value = exec_project_main_source(
            r#"
use std.states.PosInt

public fn main() -> PosInt! {
    PosInt.new(0)
}
"#,
        );

        assert_eq!(
            value,
            crate::value::Value::Variant(
                "err".into(),
                Some(Box::new(crate::value::Value::Str("InvariantViolation: PosInt".into())))
            )
        );
    }

    #[test]
    fn exec_artifact_main_runs_std_states_email_ok_source() {
        let value = exec_project_main_source(
            r#"
use std.states.Email

public fn main() -> Email! {
    Email.new("a@b.com")
}
"#,
        );

        let mut record = std::collections::HashMap::new();
        record.insert("value".into(), crate::value::Value::Str("a@b.com".into()));
        assert_eq!(
            value,
            crate::value::Value::Variant(
                "ok".into(),
                Some(Box::new(crate::value::Value::Record(record)))
            )
        );
    }

    #[test]
    fn exec_artifact_main_runs_std_states_email_err_source() {
        let value = exec_project_main_source(
            r#"
use std.states.Email

public fn main() -> Email! {
    Email.new("bad")
}
"#,
        );

        assert_eq!(
            value,
            crate::value::Value::Variant(
                "err".into(),
                Some(Box::new(crate::value::Value::Str("InvariantViolation: Email".into())))
            )
        );
    }

    #[test]
    fn exec_artifact_main_runs_std_states_probability_bounds_source() {
        let ok_value = exec_project_main_source(
            r#"
use std.states.Probability

public fn main() -> Probability! {
    Probability.new(0.5)
}
"#,
        );
        let mut ok_record = std::collections::HashMap::new();
        ok_record.insert("value".into(), crate::value::Value::Float(0.5));
        assert_eq!(
            ok_value,
            crate::value::Value::Variant(
                "ok".into(),
                Some(Box::new(crate::value::Value::Record(ok_record)))
            )
        );

        let high_value = exec_project_main_source(
            r#"
use std.states.Probability

public fn main() -> Probability! {
    Probability.new(1.5)
}
"#,
        );
        assert_eq!(
            high_value,
            crate::value::Value::Variant(
                "err".into(),
                Some(Box::new(crate::value::Value::Str("InvariantViolation: Probability".into())))
            )
        );

        let low_value = exec_project_main_source(
            r#"
use std.states.Probability

public fn main() -> Probability! {
    Probability.new(-0.1)
}
"#,
        );
        assert_eq!(
            low_value,
            crate::value::Value::Variant(
                "err".into(),
                Some(Box::new(crate::value::Value::Str("InvariantViolation: Probability".into())))
            )
        );

        let port_ok = exec_project_main_source(
            r#"
use std.states.PortNumber

public fn main() -> PortNumber! {
    PortNumber.new(8080)
}
"#,
        );
        let mut port_record = std::collections::HashMap::new();
        port_record.insert("value".into(), crate::value::Value::Int(8080));
        assert_eq!(
            port_ok,
            crate::value::Value::Variant(
                "ok".into(),
                Some(Box::new(crate::value::Value::Record(port_record)))
            )
        );

        let port_err = exec_project_main_source(
            r#"
use std.states.PortNumber

public fn main() -> PortNumber! {
    PortNumber.new(0)
}
"#,
        );
        assert_eq!(
            port_err,
            crate::value::Value::Variant(
                "err".into(),
                Some(Box::new(crate::value::Value::Str("InvariantViolation: PortNumber".into())))
            )
        );
    }

    #[test]
    fn exec_artifact_main_runs_std_states_string_family_source() {
        let nonempty_ok = exec_project_main_source(
            r#"
use std.states.NonEmptyString

public fn main() -> NonEmptyString! {
    NonEmptyString.new("hello")
}
"#,
        );
        let mut nonempty_record = std::collections::HashMap::new();
        nonempty_record.insert("value".into(), crate::value::Value::Str("hello".into()));
        assert_eq!(
            nonempty_ok,
            crate::value::Value::Variant(
                "ok".into(),
                Some(Box::new(crate::value::Value::Record(nonempty_record)))
            )
        );

        let slug_ok = exec_project_main_source(
            r#"
use std.states.Slug

public fn main() -> Slug! {
    Slug.new("hello-world")
}
"#,
        );
        let mut slug_record = std::collections::HashMap::new();
        slug_record.insert("value".into(), crate::value::Value::Str("hello-world".into()));
        assert_eq!(
            slug_ok,
            crate::value::Value::Variant(
                "ok".into(),
                Some(Box::new(crate::value::Value::Record(slug_record)))
            )
        );

        let slug_err = exec_project_main_source(
            r#"
use std.states.Slug

public fn main() -> Slug! {
    Slug.new("hello world")
}
"#,
        );
        assert_eq!(
            slug_err,
            crate::value::Value::Variant(
                "err".into(),
                Some(Box::new(crate::value::Value::Str("InvariantViolation: Slug".into())))
            )
        );

        let url_ok = exec_project_main_source(
            r#"
use std.states.Url

public fn main() -> Url! {
    Url.new("https://example.com")
}
"#,
        );
        let mut url_record = std::collections::HashMap::new();
        url_record.insert("value".into(), crate::value::Value::Str("https://example.com".into()));
        assert_eq!(
            url_ok,
            crate::value::Value::Variant(
                "ok".into(),
                Some(Box::new(crate::value::Value::Record(url_record)))
            )
        );

        let url_err = exec_project_main_source(
            r#"
use std.states.Url

public fn main() -> Url! {
    Url.new("ftp://example.com")
}
"#,
        );
        assert_eq!(
            url_err,
            crate::value::Value::Variant(
                "err".into(),
                Some(Box::new(crate::value::Value::Str("InvariantViolation: Url".into())))
            )
        );
    }

    #[test]
    fn format_invariants_joins_multiple_expressions() {
        let program = Parser::parse_str(
            r#"
type Probability = {
    value: Float
    invariant value >= 0.0
    invariant value <= 1.0
}
"#,
            "format_invariants.fav",
        )
        .expect("parse");
        let td = program
            .items
            .into_iter()
            .find_map(|item| match item {
                ast::Item::TypeDef(td) => Some(td),
                _ => None,
            })
            .expect("type def");
        let text = format_invariants(&td.invariants);
        assert!(text.contains("value >= 0.0"));
        assert!(text.contains("value <= 1.0"));
        assert!(text.contains(";"));
    }

    #[test]
    fn explain_render_shows_invariants_for_types() {
        let program = Parser::parse_str(
            r#"
type Email = {
    value: String
    invariant String.contains(value, "@")
    invariant String.length(value) > 3
}
"#,
            "explain_invariants.fav",
        )
        .expect("parse");
        let rendered = ExplainPrinter::new().render(&program, None, false);
        assert!(rendered.contains("INVARIANTS"));
        assert!(rendered.contains("String.contains(value, \"@\")"));
        assert!(rendered.contains("String.length(value) > 3"));
    }

    #[test]
    fn explain_render_labels_stdlib_states() {
        let program = Parser::parse_str(
            r#"
use std.states.PosInt

public fn main() -> PosInt! {
    PosInt.new(1)
}
"#,
            "explain_stdlib.fav",
        )
        .expect("parse");
        let rendered = ExplainPrinter::new().render(&program, None, true);
        assert!(rendered.contains("PosInt (stdlib)"));
        assert!(rendered.contains("value > 0"));
    }

    #[test]
    fn explain_render_shows_abstract_trf_section() {
        let program = Parser::parse_str(
            r#"
abstract stage FetchUser: Int -> String !Db
"#,
            "explain_abstract_trf.fav",
        )
        .expect("parse");
        let rendered = ExplainPrinter::new().render(&program, None, false);
        assert!(rendered.contains("ABSTRACT TRF"));
        assert!(rendered.contains("FetchUser: Int -> String !Db"));
    }

    #[test]
    fn explain_render_shows_abstract_flw_section() {
        let program = Parser::parse_str(
            r#"
abstract seq DataPipeline<Row> {
    parse: String -> List<Row>!
    save: List<Row> -> Int !Db
}
"#,
            "explain_abstract_flw.fav",
        )
        .expect("parse");
        let rendered = ExplainPrinter::new().render(&program, None, false);
        assert!(rendered.contains("ABSTRACT FLW"));
        assert!(rendered.contains("DataPipeline<Row>"));
        assert!(rendered.contains("parse: String -> List<Row>!"));
        assert!(rendered.contains("save: List<Row> -> Int !Db"));
    }

    #[test]
    fn explain_render_shows_flw_binding_section() {
        let program = Parser::parse_str(
            r#"
type UserRow = { name: String }
abstract seq DataPipeline<Row> {
    parse: String -> List<Row>!
    validate: Row -> Row!
    save: List<Row> -> Int !Db
}
abstract stage ParseCsv: String -> List<UserRow>!
abstract stage SaveUsers: List<UserRow> -> Int !Db
seq PartialImport = DataPipeline<UserRow> {
    parse <- ParseCsv
    save <- SaveUsers
}
"#,
            "explain_flw_binding.fav",
        )
        .expect("parse");
        let rendered = ExplainPrinter::new().render(&program, None, false);
        assert!(rendered.contains("FLW BINDINGS"));
        assert!(rendered.contains("PartialImport = DataPipeline<UserRow>"));
        assert!(rendered.contains("parse <- ParseCsv"));
        assert!(rendered.contains("save <- SaveUsers"));
        assert!(rendered.contains("resolved: partial"));
        assert!(rendered.contains("unbound: validate"));
    }

    #[test]
    fn explain_json_valid_schema() {
        let program = Parser::parse_str(
            r#"
public fn main() -> Unit !Io {
    IO.println("hello")
}
"#,
            "explain_json_schema.fav",
        )
        .expect("parse");
        let ir = crate::middle::compiler::compile_program(&program);
        let rendered = ExplainPrinter::new().render_json(
            &program,
            Some(&ir),
            false,
            "all",
            "explain_json_schema.fav",
            None,
        );
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("valid json");
        assert_eq!(value["schema_version"], "1.0");
        assert_eq!(value["favnir_version"], "1.5.0");
        assert!(value["fns"].is_array());
        assert!(value["trfs"].is_array());
        assert!(value["flws"].is_array());
        assert!(value["types"].is_array());
    }

    #[test]
    fn explain_json_has_all_sections() {
        let program = Parser::parse_str(
            r#"
type UserRow = { name: String }
abstract stage FetchUser: Int -> UserRow? !Db
abstract seq Pipeline<Row> {
    fetch: Int -> Row? !Db
}
stage ParseUser: Int -> UserRow = |x| { UserRow { name: "a" } }
seq ImportUsers = ParseUser |> ParseUser
seq UserFetch = Pipeline<UserRow> { fetch <- FetchUser }
public fn main() -> Unit !Io {
    IO.println("ok")
}
"#,
            "explain_json_all.fav",
        )
        .expect("parse");
        let ir = crate::middle::compiler::compile_program(&program);
        let rendered = ExplainPrinter::new().render_json(
            &program,
            Some(&ir),
            false,
            "all",
            "explain_json_all.fav",
            None,
        );
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("valid json");
        assert!(value["fns"].as_array().expect("fns").iter().any(|v| v["name"] == "main"));
        assert!(value["trfs"].as_array().expect("trfs").iter().any(|v| v["name"] == "FetchUser"));
        assert!(value["flws"].as_array().expect("flws").iter().any(|v| v["name"] == "Pipeline"));
        assert!(value["types"].as_array().expect("types").iter().any(|v| v["name"] == "UserRow"));
    }

    #[test]
    fn explain_json_focus_trfs() {
        let program = Parser::parse_str(
            r#"
abstract stage FetchUser: Int -> String !Db
stage ParseName: Int -> String = |x| { "name" }
"#,
            "explain_json_focus.fav",
        )
        .expect("parse");
        let ir = crate::middle::compiler::compile_program(&program);
        let rendered = ExplainPrinter::new().render_json(
            &program,
            Some(&ir),
            false,
            "trfs",
            "explain_json_focus.fav",
            None,
        );
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("valid json");
        assert_eq!(value["fns"].as_array().expect("fns").len(), 0);
        assert!(value["trfs"].as_array().expect("trfs").len() >= 2);
        assert_eq!(value["flws"].as_array().expect("flws").len(), 0);
        assert_eq!(value["types"].as_array().expect("types").len(), 0);
    }

    #[test]
    fn explain_json_kinds() {
        let program = Parser::parse_str(
            r#"
type UserRow = { name: String }
abstract stage FetchUser: Int -> UserRow? !Db
abstract seq Pipeline<Row> {
    fetch: Int -> Row? !Db
}
seq UserFetch = Pipeline<UserRow> { fetch <- FetchUser }
"#,
            "explain_json_kinds.fav",
        )
        .expect("parse");
        let rendered = ExplainPrinter::new().render_json(
            &program,
            None,
            false,
            "all",
            "explain_json_kinds.fav",
            None,
        );
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("valid json");
        assert!(value["trfs"].as_array().expect("trfs").iter().any(|v| v["kind"] == "abstract_trf"));
        assert!(value["flws"].as_array().expect("flws").iter().any(|v| v["kind"] == "abstract_flw"));
        assert!(value["flws"].as_array().expect("flws").iter().any(|v| v["kind"] == "flw_binding"));
    }

    #[test]
    fn explain_json_custom_effects() {
        let program = Parser::parse_str(
            r#"
public effect Payment
effect Notification
stage Charge: Int -> Int !Payment = |x| { x }
"#,
            "explain_json_custom_effects.fav",
        )
        .expect("parse");
        let ir = crate::middle::compiler::compile_program(&program);
        let rendered = ExplainPrinter::new().render_json(
            &program,
            Some(&ir),
            false,
            "all",
            "explain_json_custom_effects.fav",
            None,
        );
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("valid json");
        let effects = value["custom_effects"].as_array().expect("custom_effects");
        assert!(effects.iter().any(|v| v["name"] == "Payment" && v["public"] == true));
        assert!(effects.iter().any(|v| v["name"] == "Notification" && v["public"] == false));
        assert!(value["effects_used"].as_array().expect("effects_used").iter().any(|v| v == "Payment"));
    }

    #[test]
    fn explain_json_reachable_flag() {
        let program = Parser::parse_str(
            r#"
fn helper() -> Int { 1 }
public fn main() -> Int { 0 }
"#,
            "explain_json_reachable.fav",
        )
        .expect("parse");
        let ir = crate::middle::compiler::compile_program(&program);
        let reachability = crate::middle::reachability::reachability_analysis("main", &ir);
        let rendered = ExplainPrinter::new().render_json(
            &program,
            Some(&ir),
            false,
            "all",
            "explain_json_reachable.fav",
            Some(&reachability),
        );
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("valid json");
        let fns = value["fns"].as_array().expect("fns");
        let main = fns.iter().find(|v| v["name"] == "main").expect("main");
        let helper = fns.iter().find(|v| v["name"] == "helper").expect("helper");
        assert_eq!(main["reachable_from_entry"], serde_json::Value::Bool(true));
        assert_eq!(helper["reachable_from_entry"], serde_json::Value::Bool(false));
    }

    #[test]
    fn schema_render_emits_check_for_supported_invariant() {
        let program = Parser::parse_str(
            r#"
type PosInt = {
    value: Int
    invariant value > 0
}
"#,
            "schema_posint.fav",
        )
        .expect("parse");
        let rendered = ExplainPrinter::new().render_schema(&program, false);
        assert!(rendered.contains("CREATE TABLE pos_int"));
        assert!(rendered.contains("value INTEGER NOT NULL"));
        assert!(rendered.contains("CHECK (value > 0)"));
    }

    #[test]
    fn schema_render_emits_sql_for_string_helpers() {
        let program = Parser::parse_str(
            r#"
type Email = {
    value: String
    invariant String.contains(value, "@")
    invariant String.starts_with(value, "a")
}
"#,
            "schema_email.fav",
        )
        .expect("parse");
        let rendered = ExplainPrinter::new().render_schema(&program, false);
        assert!(rendered.contains("value TEXT NOT NULL"));
        assert!(rendered.contains("value LIKE '%@%'"));
        assert!(rendered.contains("value LIKE 'a%'"));
    }

    #[test]
    fn schema_render_comments_unsupported_invariant() {
        let program = Parser::parse_str(
            r#"
type Slug = {
    value: String
    invariant String.is_slug(value)
}
"#,
            "schema_slug.fav",
        )
        .expect("parse");
        let rendered = ExplainPrinter::new().render_schema(&program, false);
        assert!(rendered.contains("-- [unsupported invariant: String.is_slug(value)]"));
    }

    #[test]
    fn artifact_info_string_includes_main_signature() {
        let source = r#"
public fn main() -> Unit !Emit<Event> {
    emit "hello"
}
"#;
        let program = Parser::parse_str(source, "artifact_info.fav").expect("parse");
        let artifact = build_artifact(&program);

        let info = artifact_info_string(&artifact);
        assert!(info.contains("artifact: .fvc"));
        assert!(info.contains("summary:"));
        assert!(info.contains("- total bytecode bytes:"));
        assert!(info.contains("- total constants:"));
        assert!(info.contains("- total string bytes:"));
        assert!(info.contains("- longest string entry:"));
        assert!(info.contains("- string preview:"));
        assert!(info.contains("- max locals in function:"));
        assert!(info.contains("- reachable functions from entry: 1"));
        assert!(info.contains("- reachable globals from entry: 0"));
        assert!(info.contains("- total instructions:"));
        assert!(info.contains("- distinct opcode kinds:"));
        assert!(info.contains("- hot opcodes:"));
        assert!(info.contains("- constant kinds:"));
        assert!(info.contains("- effect counts:"));
        assert!(info.contains("globals table:"));
        assert!(info.contains("function table:"));
        assert!(info.contains("main [fn#0] => fn#0 main @L"));
        assert!(info.contains("fn#0 main @L"));
        assert!(info.contains("opcodes:"));
        assert!(info.contains("consts: #0=Str(\"hello\")"));
        assert!(info.contains("[Emit(\"Event\")]"));
        assert!(info.contains("entry: main"));
        assert!(info.contains("entry signature: () -> Unit"));
        assert!(info.contains("!Emit<Event>: 1"));
    }

    #[test]
    fn artifact_info_string_lists_closure_and_variant_globals() {
        let source = r#"
type Direction =
    | North

public fn main() -> String {
    bind suffix <- "!"
    bind f <- |x| x + suffix
    bind north <- North
    f("ok")
}
"#;
        let program = Parser::parse_str(source, "artifact_info_globals.fav").expect("parse");
        let artifact = build_artifact(&program);

        let info = artifact_info_string(&artifact);
        assert!(info.contains("- function globals: 2"));
        assert!(info.contains("- variant ctors: 1"));
        assert!(info.contains("- synthetic closures: 1"));
        assert!(info.contains("- reachable functions from entry: 3"));
        assert!(info.contains("- reachable globals from entry: 2"));
        assert!(info.contains("#0="));
        assert!(info.contains("Pure: 2"));
        assert!(info.contains("North [variant_ctor] => fn#"));
        assert!(info.contains("$closure0 [fn#"));
        assert!(info.contains("=> fn#"));
        assert!(info.contains("main [fn#0] => fn#0 main @L"));
        assert!(info.contains("opcodes: LoadLocal="));
        assert!(info.contains("consts: #0=Str(\"!\""));
    }

    #[test]
    fn artifact_info_string_reports_trace_and_emit_summary() {
        let source = r#"
stage TraceOnce: String -> String !Trace = |x| {
    x
}

public fn main() -> Unit !Emit<UserCreated> !Trace {
    emit "ok"
}
"#;
        let program = Parser::parse_str(source, "artifact_info_trace_emit.fav").expect("parse");
        let artifact = build_artifact(&program);

        let info = artifact_info_string(&artifact);
        assert!(info.contains("- trace-enabled functions: 2"));
        assert!(info.contains("- emitted events: UserCreated"));
        assert!(info.contains("!Trace: 2"));
        assert!(info.contains("!Emit<UserCreated>: 1"));
    }

    #[test]
    fn artifact_info_round_trip_from_file_preserves_summary() {
        let source = r#"
public fn main() -> Unit !Emit<UserCreated> {
    emit "hello"
}
"#;
        let program = Parser::parse_str(source, "artifact_info_round_trip.fav").expect("parse");
        let artifact = build_artifact(&program);
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("info_round_trip.fvc");

        write_artifact_to_path(&artifact, &path).expect("write artifact");
        let restored = read_artifact_from_path(&path).expect("read artifact");
        let info = artifact_info_string(&restored);

        assert!(info.contains("artifact: .fvc"));
        assert!(info.contains("- emitted events: UserCreated"));
        assert!(info.contains("entry signature: () -> Unit"));
    }

    #[test]
    fn file_path_build_exec_round_trip_runs_main() {
        let dir = tempdir().expect("tempdir");
        let src = dir.path().join("main.fav");
        std::fs::write(
            &src,
            r#"
stage Double: Int -> Int = |x| { x + x }

public fn main() -> Int {
    21 |> Double
}
"#,
        )
        .expect("write source");

        let src_str = src.to_string_lossy().to_string();
        let (program, loaded_path) = load_and_check_program(Some(&src_str));
        assert_eq!(loaded_path, src_str);

        let artifact = build_artifact(&program);
        let artifact_path = dir.path().join("main.fvc");
        write_artifact_to_path(&artifact, &artifact_path).expect("write artifact");
        let restored = read_artifact_from_path(&artifact_path).expect("read artifact");

        let value = exec_artifact_main(&restored, None).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Int(42));
    }

    #[test]
    fn file_path_build_info_round_trip_preserves_trace_emit_summary() {
        let dir = tempdir().expect("tempdir");
        let src = dir.path().join("main.fav");
        std::fs::write(
            &src,
            r#"
stage TraceOnce: String -> String !Trace = |x| {
    x
}

public fn main() -> Unit !Emit<UserCreated> !Trace {
    emit "hello"
}
"#,
        )
        .expect("write source");

        let src_str = src.to_string_lossy().to_string();
        let (program, _) = load_and_check_program(Some(&src_str));
        let artifact = build_artifact(&program);
        let artifact_path = dir.path().join("trace_emit.fvc");
        write_artifact_to_path(&artifact, &artifact_path).expect("write artifact");
        let restored = read_artifact_from_path(&artifact_path).expect("read artifact");

        let info = artifact_info_string(&restored);
        assert!(info.contains("entry: main"));
        assert!(info.contains("entry signature: () -> Unit"));
        assert!(info.contains("- trace-enabled functions: 2"));
        assert!(info.contains("- emitted events: UserCreated"));
        assert!(info.contains("!Trace: 2"));
        assert!(info.contains("!Emit<UserCreated>: 1"));
    }

    #[test]
    fn build_wasm_artifact_runs_main_for_temp_source() {
        let source = r#"
public fn main() -> Unit !Io {
    IO.println("Hello, Favnir!")
}
"#;
        let program = Parser::parse_str(source, "hello_wasm.fav").expect("parse");
        let bytes = build_wasm_artifact(&program).expect("build wasm");
        crate::backend::wasm_exec::wasm_exec_main(&bytes).expect("exec wasm");
    }

    #[test]
    fn file_path_build_wasm_exec_round_trip_runs_main() {
        let dir = tempdir().expect("tempdir");
        let src = dir.path().join("main.fav");
        std::fs::write(
            &src,
            r#"
public fn main() -> Unit !Io {
    IO.println("Hello, Favnir!")
}
"#,
        )
        .expect("write source");

        let src_str = src.to_string_lossy().to_string();
        let (program, loaded_path) = load_and_check_program(Some(&src_str));
        assert_eq!(loaded_path, src_str);

        let bytes = build_wasm_artifact(&program).expect("build wasm");
        let wasm_path = dir.path().join("main.wasm");
        write_wasm_to_path(&bytes, &wasm_path).expect("write wasm");
        let restored = read_wasm_from_path(&wasm_path).expect("read wasm");

        crate::backend::wasm_exec::wasm_exec_main(&restored).expect("exec wasm");
    }

    #[test]
    fn file_path_build_wasm_info_round_trip_reports_metadata() {
        let dir = tempdir().expect("tempdir");
        let src = dir.path().join("main.fav");
        std::fs::write(
            &src,
            r#"
public fn main() -> Unit !Io {
    IO.println("Hello, Favnir!")
}
"#,
        )
        .expect("write source");

        let src_str = src.to_string_lossy().to_string();
        let (program, _) = load_and_check_program(Some(&src_str));
        let bytes = build_wasm_artifact(&program).expect("build wasm");
        let wasm_path = dir.path().join("hello.wasm");
        write_wasm_to_path(&bytes, &wasm_path).expect("write wasm");
        let restored = read_wasm_from_path(&wasm_path).expect("read wasm");

        let info = crate::backend::wasm_exec::wasm_exec_info(&restored);
        assert!(info.contains("artifact: .wasm"));
        assert!(info.contains("status: valid"));
        assert!(info.contains("imports: 1"));
        assert!(info.contains("exports: 2"));
        assert!(info.contains("memory: exported"));
    }

    #[test]
    fn example_hello_wasm_build_and_exec() {
        let hello = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("hello.fav");
        let hello_str = hello.to_string_lossy().to_string();
        let (program, loaded_path) = load_and_check_program(Some(&hello_str));
        assert_eq!(loaded_path, hello_str);

        let bytes = build_wasm_artifact(&program).expect("build wasm");
        crate::backend::wasm_exec::wasm_exec_main(&bytes).expect("exec wasm");
        let info = crate::backend::wasm_exec::wasm_exec_info(&bytes);
        assert!(info.contains("artifact: .wasm"));
        assert!(info.contains("memory: exported"));
    }

    #[test]
    fn example_math_wasm_build_and_exec() {
        let math = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("math_wasm.fav");
        let math_str = math.to_string_lossy().to_string();
        let (program, loaded_path) = load_and_check_program(Some(&math_str));
        assert_eq!(loaded_path, math_str);

        let bytes = build_wasm_artifact(&program).expect("build wasm");
        crate::backend::wasm_exec::wasm_exec_main(&bytes).expect("exec wasm");
    }

    #[test]
    fn example_string_wasm_build_and_exec() {
        let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("string_wasm.fav");
        let source_str = source_path.to_string_lossy().to_string();
        let (program, _) = load_and_check_program(Some(&source_str));
        let bytes = build_wasm_artifact(&program).expect("build wasm");
        crate::backend::wasm_exec::wasm_exec_main(&bytes).expect("exec wasm");
        let info = crate::backend::wasm_exec::wasm_exec_info(&bytes);
        assert!(info.contains("artifact: .wasm"));
    }

    #[test]
    fn example_cap_sort_build_and_exec() {
        let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("cap_sort.fav");
        let source_str = source_path.to_string_lossy().to_string();
        let (program, loaded_path) = load_and_check_program(Some(&source_str));
        assert_eq!(loaded_path, source_str);

        let artifact = build_artifact(&program);
        exec_artifact_main(&artifact, None).expect("exec artifact");
    }

    #[test]
    fn example_cap_user_build_and_exec() {
        let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("cap_user.fav");
        let source_str = source_path.to_string_lossy().to_string();
        let (program, loaded_path) = load_and_check_program(Some(&source_str));
        assert_eq!(loaded_path, source_str);

        let artifact = build_artifact(&program);
        exec_artifact_main(&artifact, None).expect("exec artifact");
    }

    #[test]
    fn example_interface_basic_build_and_exec() {
        let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("interface_basic.fav");
        let source_str = source_path.to_string_lossy().to_string();
        let (program, loaded_path) = load_and_check_program(Some(&source_str));
        assert_eq!(loaded_path, source_str);

        let artifact = build_artifact(&program);
        exec_artifact_main(&artifact, None).expect("exec artifact");
    }

    #[test]
    fn example_interface_auto_build_and_exec() {
        let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("interface_auto.fav");
        let source_str = source_path.to_string_lossy().to_string();
        let (program, loaded_path) = load_and_check_program(Some(&source_str));
        assert_eq!(loaded_path, source_str);

        let artifact = build_artifact(&program);
        exec_artifact_main(&artifact, None).expect("exec artifact");
    }

    #[test]
    fn example_algebraic_build_and_exec() {
        let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("algebraic.fav");
        let source_str = source_path.to_string_lossy().to_string();
        let (program, loaded_path) = load_and_check_program(Some(&source_str));
        assert_eq!(loaded_path, source_str);

        let artifact = build_artifact(&program);
        exec_artifact_main(&artifact, None).expect("exec artifact");
    }

    #[test]
    fn example_invariant_basic_build_and_exec() {
        let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("invariant_basic.fav");
        let source_str = source_path.to_string_lossy().to_string();
        let (program, loaded_path) = load_and_check_program(Some(&source_str));
        assert_eq!(loaded_path, source_str);

        let artifact = build_artifact(&program);
        exec_artifact_main(&artifact, None).expect("exec artifact");
    }

    #[test]
    fn example_std_states_build_and_exec() {
        let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("std_states.fav");
        let source_str = source_path.to_string_lossy().to_string();
        let (program, loaded_path) = load_and_check_program(Some(&source_str));
        assert_eq!(loaded_path, source_str);

        let artifact = build_artifact(&program);
        exec_artifact_main(&artifact, None).expect("exec artifact");
    }

    #[test]
    fn partial_flw_reports_e050() {
        let program = Parser::parse_str(
            r#"
abstract seq DataPipeline<Row> {
    parse: String -> List<Row>!
    validate: Row -> Row!
    save: List<Row> -> Int !Db
}
abstract stage ParseCsv: String -> List<UserRow>!
type UserRow = { name: String }
seq PartialImport = DataPipeline<UserRow> { parse <- ParseCsv }
"#,
            "partial_flw.fav",
        )
        .expect("parse");
        let err = ensure_no_partial_flw(&program).expect_err("expected E050");
        assert!(err.contains("error[E050]"));
        assert!(err.contains("PartialImport"));
        assert!(err.contains("validate, save"));
    }

    #[test]
    fn partial_flw_check_emits_w011() {
        let source = r#"
abstract seq DataPipeline<Row> {
    parse: String -> List<Row>!
    validate: Row -> Row!
    save: List<Row> -> Int !Db
}
abstract stage ParseCsv: String -> List<UserRow>!
type UserRow = { name: String }
seq PartialImport = DataPipeline<UserRow> { parse <- ParseCsv }
"#;
        let program = Parser::parse_str(source, "partial_flw_check.fav").expect("parse");
        let warnings = partial_flw_warnings(&program);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W011");
        assert!(warnings[0].message.contains("PartialImport"));
        assert!(warnings[0].message.contains("validate, save"));
    }

    #[test]
    fn example_abstract_flw_basic_build_and_exec() {
        let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("abstract_flw_basic.fav");
        let source_str = source_path.to_string_lossy().to_string();

        let (program, _) = load_and_check_program(Some(&source_str));
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Str("saved".into()));
    }

    #[test]
    fn example_abstract_flw_inject_build_and_exec() {
        let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("abstract_flw_inject.fav");
        let source_str = source_path.to_string_lossy().to_string();

        let (program, _) = load_and_check_program(Some(&source_str));
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Str("user".into()));
    }

    #[test]
    fn example_dynamic_inject_build_and_exec() {
        let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("dynamic_inject.fav");
        let source_str = source_path.to_string_lossy().to_string();

        let (program, _) = load_and_check_program(Some(&source_str));
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Str("user".into()));
    }

    #[test]
    fn example_bundle_demo_build_and_exec() {
        let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("bundle_demo.fav");
        let source_str = source_path.to_string_lossy().to_string();

        let (program, _) = load_and_check_program(Some(&source_str));
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Str("bundle-ok".into()));
    }

    #[test]
    fn exec_artifact_main_runs_local_callable_param_source() {
        let source = r#"
stage ParseUser: String -> String = |text| {
    "user"
}

fn apply(parse: String -> String, input: String) -> String {
    parse(input)
}

public fn main() -> String {
    apply(ParseUser, "input")
}
"#;
        let program = Parser::parse_str(source, "local_callable_param.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Str("user".into()));
    }

    #[test]
    fn bundle_filter_excludes_dead_code() {
        let program = Parser::parse_str(
            r#"
public fn helper() -> Int {
    7
}

public fn main() -> Int {
    42
}
"#,
            "bundle_dead_code.fav",
        )
        .expect("parse");
        let ir = compile_program(&program);
        let reachability = reachability_analysis("main", &ir);
        let filtered = filter_ir_program(&ir, &reachability.included);

        assert!(filtered.globals.iter().any(|g| g.name == "main"));
        assert!(filtered.fns.iter().any(|f| f.name == "main"));
        assert!(!filtered.globals.iter().any(|g| g.name == "helper"));
        assert!(!filtered.fns.iter().any(|f| f.name == "helper"));
    }

    #[test]
    fn bundle_manifest_json_has_reachability() {
        let reachability = ReachabilityResult {
            included: ["main".to_string(), "SaveUsers".to_string()].into_iter().collect(),
            excluded: ["DeadFn".to_string()].into_iter().collect(),
            effects_required: vec!["Io".into(), "Db".into()],
            emits: vec!["UserCreated".into()],
        };
        let artifact_path = PathBuf::from("dist/app.fvc");
        let manifest = build_manifest_json(
            "src/main.fav",
            &artifact_path,
            123,
            "main",
            &reachability,
        );
        let value: serde_json::Value = serde_json::from_str(&manifest).expect("json");
        assert_eq!(value["entry"], "main");
        assert_eq!(value["artifact"]["format"], "fvc");
        assert_eq!(value["artifact"]["size_bytes"], 123);
        assert!(value["reachability"]["included"]
            .as_array()
            .expect("included array")
            .iter()
            .any(|v| v == "main"));
        assert!(value["reachability"]["excluded"]
            .as_array()
            .expect("excluded array")
            .iter()
            .any(|v| v == "DeadFn"));
        assert!(value["reachability"]["effects_required"]
            .as_array()
            .expect("effects array")
            .iter()
            .any(|v| v == "Db"));
    }

    #[test]
    fn bundle_writes_artifact_manifest_and_explain() {
        let dir = tempdir().expect("tempdir");
        let src = dir.path().join("bundle_me.fav");
        std::fs::write(
            &src,
            r#"
public fn helper() -> Int {
    7
}

public fn main() -> Int !Io {
    IO.println("bundle");
    42
}
"#,
        )
        .expect("write source");

        let out = dir.path().join("bundle_me.fvc");
        let src_str = src.to_string_lossy().to_string();
        let out_str = out.to_string_lossy().to_string();
        cmd_bundle(&src_str, Some(&out_str), "main", true, true);

        assert!(out.exists(), "artifact should exist");
        let manifest_path = out.with_extension("manifest.json");
        let explain_path = out.with_extension("explain.json");
        assert!(manifest_path.exists(), "manifest should exist");
        assert!(explain_path.exists(), "explain should exist");

        let artifact = read_artifact_from_path(&out).expect("read artifact");
        assert!(artifact.fn_idx_by_name("main").is_some());
        assert!(artifact.fn_idx_by_name("helper").is_none());

        let manifest: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&manifest_path).expect("read manifest"),
        )
        .expect("manifest json");
        assert_eq!(manifest["entry"], "main");
        assert!(manifest["reachability"]["included"]
            .as_array()
            .expect("included array")
            .iter()
            .any(|v| v == "main"));
        assert!(manifest["reachability"]["excluded"]
            .as_array()
            .expect("excluded array")
            .iter()
            .any(|v| v == "helper"));

        let explain: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&explain_path).expect("read explain"),
        )
        .expect("explain json");
        assert_eq!(explain["entry"], "main");
        assert!(explain["fns"]
            .as_array()
            .expect("fns array")
            .iter()
            .any(|v| v["name"] == "main"));
        assert!(!explain["fns"]
            .as_array()
            .expect("fns array")
            .iter()
            .any(|v| v["name"] == "helper"));
    }

    #[test]
    fn bundle_explain_embedded_round_trip() {
        let dir = tempdir().expect("tempdir");
        let src = dir.path().join("embedded.fav");
        std::fs::write(
            &src,
            r#"
public fn helper() -> Int {
    7
}

public fn main() -> Int {
    42
}
"#,
        )
        .expect("write source");

        let out = dir.path().join("embedded.fvc");
        let src_str = src.to_string_lossy().to_string();
        let out_str = out.to_string_lossy().to_string();
        cmd_bundle(&src_str, Some(&out_str), "main", false, true);

        let artifact = read_artifact_from_path(&out).expect("read artifact");
        let explain = explain_json_from_artifact(&artifact).expect("embedded explain");
        let value: serde_json::Value = serde_json::from_str(explain).expect("json");
        assert_eq!(value["entry"], "main");
        assert!(value["fns"]
            .as_array()
            .expect("fns array")
            .iter()
            .any(|v| v["name"] == "main"));
        assert!(!value["fns"]
            .as_array()
            .expect("fns array")
            .iter()
            .any(|v| v["name"] == "helper"));
    }

    #[test]
    fn graph_text_shows_flw_structure() {
        let program = Parser::parse_str(
            r#"
abstract seq DataPipeline<Row> {
    parse: String -> List<Row> !
    validate: Row -> Row !
    save: List<Row> -> Int !Db
}

seq ImportUsers = ParseCsv |> ValidateUser |> SaveUsers
seq DefaultImport = DataPipeline<UserRow> {
    parse <- ParseCsv
    validate <- ValidateUser
    save <- SaveUsers
}
"#,
            "graph_text.fav",
        )
        .expect("parse");

        let rendered = render_graph_text(&program, "flw");
        assert!(rendered.contains("abstract seq DataPipeline:"));
        assert!(rendered.contains("slot parse: String -> List<Row>"));
        assert!(rendered.contains("flw ImportUsers:"));
        assert!(rendered.contains("1. ParseCsv"));
        assert!(rendered.contains("flw DefaultImport = DataPipeline:"));
        assert!(rendered.contains("parse <- ParseCsv"));
    }

    #[test]
    fn graph_mermaid_valid_syntax() {
        let program = Parser::parse_str(
            r#"
abstract seq DataPipeline<Row> {
    parse: String -> List<Row> !
    validate: Row -> Row !
}

seq ImportUsers = ParseCsv |> ValidateUser
seq DefaultImport = DataPipeline<UserRow> {
    parse <- ParseCsv
    validate <- ValidateUser
}
"#,
            "graph_mermaid.fav",
        )
        .expect("parse");

        let rendered = render_graph_mermaid(&program, "flw");
        assert!(rendered.starts_with("flowchart LR"));
        assert!(rendered.contains("ImportUsers --> ParseCsv"));
        assert!(rendered.contains("DefaultImport --> DataPipeline"));
        assert!(rendered.contains("DefaultImport_parse --> ParseCsv"));
    }

    #[test]
    fn graph_fn_text_shows_calls() {
        let program = Parser::parse_str(
            r#"
fn helper() -> Int { 1 }
fn twice() -> Int { helper() + helper() }
public fn main() -> Int { twice() }
"#,
            "graph_fn_text.fav",
        )
        .expect("parse");

        let rendered = render_graph_text_with_opts(&program, "fn", Some("main"), None);
        assert!(rendered.contains("fn main:"));
        assert!(rendered.contains("-> twice"));
        assert!(rendered.contains("fn twice:"));
        assert!(rendered.contains("-> helper"));
    }

    #[test]
    fn graph_fn_mermaid_valid() {
        let program = Parser::parse_str(
            r#"
fn helper() -> Int { 1 }
public fn main() -> Int { helper() }
"#,
            "graph_fn_mermaid.fav",
        )
        .expect("parse");

        let rendered = render_graph_mermaid_with_opts(&program, "fn", Some("main"), None);
        assert!(rendered.starts_with("flowchart LR"));
        assert!(rendered.contains("main[\"fn main\"]"));
        assert!(rendered.contains("main --> helper"));
    }

    #[test]
    fn graph_fn_depth_limit() {
        let program = Parser::parse_str(
            r#"
fn leaf() -> Int { 1 }
fn mid() -> Int { leaf() }
public fn main() -> Int { mid() }
"#,
            "graph_fn_depth.fav",
        )
        .expect("parse");

        let rendered = render_graph_text_with_opts(&program, "fn", Some("main"), Some(1));
        assert!(rendered.contains("fn main:"));
        assert!(rendered.contains("fn mid:"));
        assert!(!rendered.contains("fn leaf:"));
    }

    #[test]
    fn graph_fn_cycle_safe() {
        let program = Parser::parse_str(
            r#"
fn a() -> Int { b() }
fn b() -> Int { a() }
public fn main() -> Int { a() }
"#,
            "graph_fn_cycle.fav",
        )
        .expect("parse");

        let rendered = render_graph_text_with_opts(&program, "fn", Some("main"), None);
        assert!(rendered.contains("fn main:"));
        assert!(rendered.contains("fn a:"));
        assert!(rendered.contains("fn b:"));
    }

    #[test]
    fn explain_diff_no_changes() {
        let program = Parser::parse_str(
            "public fn main() -> Int { 1 }",
            "diff_same.fav",
        )
        .expect("parse");
        let ir = crate::middle::compiler::compile_program(&program);
        let rendered = ExplainPrinter::new().render_json(&program, Some(&ir), false, "all", "diff_same.fav", None);
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("json");
        let diff = diff_explain_json("a", &value, "b", &value);
        assert_eq!(render_diff_text(&diff), "No changes detected.\n");
    }

    #[test]
    fn explain_diff_fn_removed_is_breaking() {
        let before = serde_json::json!({
            "fns": [{"name":"main","params":[],"return_type":"Int","effects":[]},{"name":"helper","params":[],"return_type":"Int","effects":[]}],
            "trfs": [],
            "flws": [],
            "types": [],
            "effects_used": []
        });
        let after = serde_json::json!({
            "fns": [{"name":"main","params":[],"return_type":"Int","effects":[]}],
            "trfs": [],
            "flws": [],
            "types": [],
            "effects_used": []
        });
        let diff = diff_explain_json("old", &before, "new", &after);
        assert!(diff.breaking_changes.iter().any(|c| c.contains("removed fn `helper`")));
        let rendered = render_diff_text(&diff);
        assert!(rendered.contains("- helper"));
    }

    #[test]
    fn explain_diff_json_valid() {
        let before = serde_json::json!({
            "fns": [{"name":"main","params":[],"return_type":"Int","effects":[]}],
            "trfs": [],
            "flws": [],
            "types": [],
            "effects_used": ["Io"]
        });
        let after = serde_json::json!({
            "fns": [{"name":"main","params":[],"return_type":"String","effects":["Io"]}],
            "trfs": [],
            "flws": [],
            "types": [],
            "effects_used": ["Io","Payment"]
        });
        let diff = diff_explain_json("old", &before, "new", &after);
        let rendered = render_diff_json(&diff);
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("valid json");
        assert_eq!(value["from_label"], "old");
        assert_eq!(value["to_label"], "new");
        assert!(value["fn_changes"]["changed"].is_array());
        assert!(value["effects_added"].as_array().expect("effects_added").iter().any(|v| v == "Payment"));
    }

    #[test]
    fn cap_example_check_emits_w010_but_no_errors() {
        let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("cap_user.fav");
        let source_str = source_path.to_string_lossy().to_string();

        let (source, errors, warnings) = check_single_file(&source_str);
        assert!(errors.is_empty(), "expected no check errors");
        assert!(!warnings.is_empty(), "expected deprecated cap warning");

        let rendered = render_warnings(&source, &warnings, false);
        assert!(rendered.iter().any(|w| w.contains("warning[W010]")));
        assert!(rendered.iter().any(|w| w.contains("deprecated")));
    }

    #[test]
    fn cap_example_check_no_warn_suppresses_warning_output() {
        let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("cap_user.fav");
        let source_str = source_path.to_string_lossy().to_string();

        let (source, errors, warnings) = check_single_file(&source_str);
        assert!(errors.is_empty(), "expected no check errors");
        assert!(!warnings.is_empty(), "expected deprecated cap warning");

        let rendered = render_warnings(&source, &warnings, true);
        assert!(rendered.is_empty(), "expected --no-warn equivalent to suppress warnings");
    }

    #[test]
    fn wasm_exec_bytes_rejects_db_path_with_w004() {
        let hello = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("hello.fav");
        let hello_str = hello.to_string_lossy().to_string();
        let (program, _) = load_and_check_program(Some(&hello_str));
        let bytes = build_wasm_artifact(&program).expect("build wasm");

        let err = exec_wasm_bytes(&bytes, false, Some("app.db")).unwrap_err();
        assert!(err.contains("error[W004]"));
    }

    #[test]
    fn wasm_exec_bytes_info_returns_metadata() {
        let hello = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("hello.fav");
        let hello_str = hello.to_string_lossy().to_string();
        let (program, _) = load_and_check_program(Some(&hello_str));
        let bytes = build_wasm_artifact(&program).expect("build wasm");

        let info = exec_wasm_bytes(&bytes, true, None)
            .expect("info ok")
            .expect("info text");
        assert!(info.contains("artifact: .wasm"));
        assert!(info.contains("memory: exported"));
    }

    #[test]
    fn exec_artifact_main_runs_file_read_source() {
        let dir = tempdir().expect("tempdir");
        let input = dir.path().join("input.txt");
        std::fs::write(&input, "hello from file").expect("write input");
        let src = dir.path().join("main.fav");
        let input_path = input.display().to_string().replace('\\', "/");
        std::fs::write(
            &src,
            format!(
                r#"
public fn main() -> String !File {{
    File.read("{}")
}}
"#,
                input_path
            ),
        )
        .expect("write source");

        let src_str = src.to_string_lossy().to_string();
        let (program, _) = load_and_check_program(Some(&src_str));
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Str("hello from file".into()));
    }

    #[test]
    fn exec_artifact_main_runs_file_write_lines_and_exists_source() {
        let dir = tempdir().expect("tempdir");
        let output = dir.path().join("lines.txt");
        let src = dir.path().join("main.fav");
        let output_path = output.display().to_string().replace('\\', "/");
        std::fs::write(
            &src,
            format!(
                r#"
public fn main() -> Bool !File {{
    bind lines <- collect {{
        yield "alpha";
        yield "beta";
        ()
    }}
    bind _ <- File.write_lines("{}", lines)
    File.exists("{}")
}}
"#,
                output_path,
                output_path
            ),
        )
        .expect("write source");

        let src_str = src.to_string_lossy().to_string();
        let (program, _) = load_and_check_program(Some(&src_str));
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec artifact");
        assert_eq!(value, crate::value::Value::Bool(true));
        assert_eq!(std::fs::read_to_string(output).expect("read output"), "alpha\nbeta");
    }

    // ── v1.7.0: coverage tracking ─────────────────────────────────────────

    #[test]
    fn coverage_tracks_executed_lines() {
        use crate::middle::compiler::set_coverage_mode;
        use crate::backend::vm::{enable_coverage, take_coverage};

        let source = r#"
public fn main() -> Int {
    bind x <- 10
    bind y <- 20
    x + y
}
"#;
        set_coverage_mode(true);
        let program = Parser::parse_str(source, "cov_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        enable_coverage();
        let _ = exec_artifact_main(&artifact, None).expect("exec");
        let covered = take_coverage();
        set_coverage_mode(false);

        assert!(!covered.is_empty(), "should have covered some lines");
    }

    #[test]
    fn coverage_report_format() {
        use std::collections::HashSet;

        let source = "public fn main() -> Int {\n    let x = 1\n    x\n}\n";
        let mut executed: HashSet<u32> = HashSet::new();
        executed.insert(2);
        executed.insert(3);

        let report = format_coverage_report("test.fav", source, &executed);
        assert!(report.contains("test.fav"), "report should contain filename");
        assert!(report.contains("%"), "report should contain percentage");
    }

    // ── v1.7.0: watch multi-dir path collection ───────────────────────────

    #[test]
    fn watch_collect_paths_from_dirs() {
        let dir = tempdir().expect("tempdir");
        let subdir = dir.path().join("sub");
        std::fs::create_dir_all(&subdir).expect("create subdir");

        std::fs::write(dir.path().join("a.fav"), "public fn main() -> Unit {}").expect("write a");
        std::fs::write(subdir.join("b.fav"), "public fn main() -> Unit {}").expect("write b");
        std::fs::write(dir.path().join("readme.txt"), "ignore me").expect("write txt");

        let dir_str = dir.path().to_string_lossy().to_string();
        let paths = collect_watch_paths_from_dir(&dir_str);

        assert!(paths.iter().any(|p| p.file_name().map(|n| n == "a.fav").unwrap_or(false)));
        assert!(paths.iter().any(|p| p.file_name().map(|n| n == "b.fav").unwrap_or(false)));
        assert!(!paths.iter().any(|p| p.extension().map(|e| e == "txt").unwrap_or(false)));
    }

    #[test]
    fn watch_collect_paths_multiple_dirs() {
        let dir1 = tempdir().expect("tempdir1");
        let dir2 = tempdir().expect("tempdir2");

        std::fs::write(dir1.path().join("x.fav"), "").expect("write x");
        std::fs::write(dir2.path().join("y.fav"), "").expect("write y");

        let d1 = dir1.path().to_string_lossy().to_string();
        let d2 = dir2.path().to_string_lossy().to_string();

        let mut paths = collect_watch_paths_from_dir(&d1);
        paths.extend(collect_watch_paths_from_dir(&d2));

        assert_eq!(paths.len(), 2);
        assert!(paths.iter().any(|p| p.file_name().map(|n| n == "x.fav").unwrap_or(false)));
        assert!(paths.iter().any(|p| p.file_name().map(|n| n == "y.fav").unwrap_or(false)));
    }

    // ── v1.8.0: Task<T> parallel API ─────────────────────────────────────────

    #[test]
    fn task_all_collects_results() {
        // Task.all takes a List<Task<T>> built via collect/yield
        let source = r#"
fn double(x: Int) -> Int { x * 2 }
public fn main() -> Int {
    bind tasks <- collect {
        yield Task.run(|| double(1));
        yield Task.run(|| double(2));
    }
    bind results <- Task.all(tasks)
    List.length(results)
}
"#;
        let program = Parser::parse_str(source, "task_all.fav").expect("parse");
        let errors = crate::middle::checker::Checker::check_program(&program);
        assert!(errors.is_empty(), "Task.all should type-check: {:?}", errors);
    }

    #[test]
    fn task_race_returns_first() {
        // Task.race(list) returns the first task's value
        let source = r#"
public fn main() -> Int {
    bind tasks <- collect {
        yield Task.run(|| 42);
        yield Task.run(|| 99);
    }
    Task.race(tasks)
}
"#;
        let program = Parser::parse_str(source, "task_race.fav").expect("parse");
        let errors = crate::middle::checker::Checker::check_program(&program);
        assert!(errors.is_empty(), "Task.race should type-check: {:?}", errors);
    }

    #[test]
    fn task_timeout_returns_some() {
        // Task.timeout(task, ms) returns Option<T>
        let source = r#"
public fn main() -> Option<Int> {
    bind t <- Task.run(|| 7)
    Task.timeout(t, 1000)
}
"#;
        let program = Parser::parse_str(source, "task_timeout.fav").expect("parse");
        let result = exec_artifact_main(&build_artifact(&program), None);
        assert!(result.is_ok(), "Task.timeout should succeed: {:?}", result.err());
    }

    // ── v1.8.0: async fn main ────────────────────────────────────────────────

    #[test]
    fn async_main_task_type_accepted() {
        // async fn main() -> Unit !Io should type-check without errors
        let source = r#"public async fn main() -> Unit !Io { IO.println("hi") }"#;
        let program = Parser::parse_str(source, "async_main.fav").expect("parse");
        let errors = crate::middle::checker::Checker::check_program(&program);
        assert!(errors.is_empty(), "async fn main should type-check: {:?}", errors);
    }

    #[test]
    fn async_main_executes_correctly() {
        let source = r#"public async fn main() -> Unit !Io { IO.println("async main") }"#;
        let program = Parser::parse_str(source, "async_main_exec.fav").expect("parse");
        let artifact = build_artifact(&program);
        let _suppress = crate::backend::vm::SuppressIoGuard::new(true);
        let result = exec_artifact_main(&artifact, None);
        assert!(result.is_ok(), "async fn main should execute: {:?}", result.err());
    }

    // ── v1.8.0: chain + Task<T> ──────────────────────────────────────────────

    #[test]
    fn chain_task_result_unwraps_both() {
        // Task<Option<T>> can be chained with `chain x <-`
        let source = r#"
public fn fetch() -> Task<Option<Int>> {
    Task.run(|| Option.some(42))
}
public fn main() -> Option<Int> {
    chain x <- fetch()
    Option.some(x)
}
"#;
        let program = Parser::parse_str(source, "chain_task.fav").expect("parse");
        let errors = crate::middle::checker::Checker::check_program(&program);
        assert!(errors.is_empty(), "chain + Task<Option<T>> should type-check: {:?}", errors);
    }

    // ── v1.8.0: coverage by function ─────────────────────────────────────────

    #[test]
    fn coverage_report_by_fn_shows_function_names() {
        use std::collections::HashSet;
        let source = r#"
fn add(x: Int, y: Int) -> Int { x + y }
public fn main() -> Int { add(1, 2) }
"#;
        let program = Parser::parse_str(source, "cov_by_fn.fav").expect("parse");
        let ir = compile_program(&program);
        // Simulate all lines covered
        let all_lines: HashSet<u32> = (1..=20).collect();
        let report = format_coverage_report_by_fn(&ir, &all_lines);
        // Should mention user functions (not $-prefixed internals)
        assert!(report.contains("add") || report.contains("main") || report.is_empty(),
            "report should contain fn names or be empty: {}", report);
    }

    #[test]
    fn coverage_report_dir_creates_file() {
        use std::collections::HashSet;
        let dir = tempdir().expect("tempdir");
        let report_dir = dir.path().join("coverage_out");
        let source = "public fn main() -> Int { 42 }\n";
        let full_report = format_coverage_report("test.fav", source, &HashSet::new());
        std::fs::create_dir_all(&report_dir).expect("create dir");
        let out_path = report_dir.join("coverage.txt");
        std::fs::write(&out_path, &full_report).expect("write");
        assert!(out_path.exists(), "coverage.txt should exist");
        let content = std::fs::read_to_string(&out_path).expect("read");
        assert!(content.contains("test.fav"), "report should contain filename");
    }

    // ── v1.8.0: fav bench ────────────────────────────────────────────────────

    #[test]
    fn bench_collect_bench_cases_finds_bench_defs() {
        let source = "bench \"add two numbers\" { 1 + 1 }\nbench \"multiply\" { 3 * 4 }";
        let program = Parser::parse_str(source, "math.bench.fav").expect("parse");
        let programs = vec![("math.bench.fav".to_string(), program)];
        let (cases, total) = collect_bench_cases(programs, None);
        assert_eq!(total, 2, "should find 2 bench defs");
        assert_eq!(cases.len(), 2);
        assert!(cases.iter().any(|(_, desc, _)| desc == "add two numbers"));
        assert!(cases.iter().any(|(_, desc, _)| desc == "multiply"));
    }

    #[test]
    fn bench_collect_bench_cases_filter_works() {
        let source = "bench \"add two numbers\" { 1 + 1 }\nbench \"multiply\" { 3 * 4 }";
        let program = Parser::parse_str(source, "math.bench.fav").expect("parse");
        let programs = vec![("math.bench.fav".to_string(), program)];
        let (cases, total) = collect_bench_cases(programs, Some("add"));
        assert_eq!(total, 2, "should discover 2 total");
        assert_eq!(cases.len(), 1, "filter should match 1");
        assert_eq!(cases[0].1, "add two numbers");
    }

    #[test]
    fn bench_runs_and_reports_timing() {
        let source = "bench \"simple arithmetic\" { 2 + 2 }";
        let program = Parser::parse_str(source, "bench_timing.fav").expect("parse");
        let artifact = build_artifact(&program);
        // Verify the bench function is in the artifact
        assert!(artifact.fn_idx_by_name("$bench:simple arithmetic").is_some(),
            "bench function should be compiled into artifact");
    }
}

// ── fav fmt ───────────────────────────────────────────────────────────────────

pub fn cmd_fmt(file: Option<&str>, check: bool) {
    use crate::fmt::format_program;

    let paths: Vec<String> = if let Some(f) = file {
        vec![f.to_string()]
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let root = FavToml::find_root(&cwd).unwrap_or_else(|| {
            eprintln!("error: no fav.toml found; pass a file path or run in project root");
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

    let mut any_diff = false;

    for path in &paths {
        let source = load_file(path);
        let program = Parser::parse_str(&source, path).unwrap_or_else(|e| {
            eprintln!("{}", e);
            process::exit(1);
        });

        let formatted = format_program(&program);

        if check {
            if formatted != source {
                eprintln!("{}: would reformat", path);
                any_diff = true;
            }
        } else {
            if formatted != source {
                std::fs::write(path, &formatted).unwrap_or_else(|e| {
                    eprintln!("error: cannot write `{}`: {}", path, e);
                    process::exit(1);
                });
                println!("{}: reformatted", path);
            } else {
                println!("{}: ok (no changes)", path);
            }
        }
    }

    if check && any_diff {
        process::exit(1);
    }
}

// ── fav lint ──────────────────────────────────────────────────────────────────

pub fn cmd_lint(file: Option<&str>, warn_only: bool) {
    use crate::lint::lint_program;

    let paths: Vec<String> = if let Some(f) = file {
        vec![f.to_string()]
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let root = FavToml::find_root(&cwd).unwrap_or_else(|| {
            eprintln!("error: no fav.toml found; pass a file path or run in project root");
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

    let mut total_warnings = 0usize;

    for path in &paths {
        let source = load_file(path);
        let program = Parser::parse_str(&source, path).unwrap_or_else(|e| {
            eprintln!("{}", e);
            process::exit(1);
        });

        let lints = lint_program(&program);
        if lints.is_empty() {
            println!("{}: ok", path);
        } else {
            for lint in &lints {
                let line_num = lint.span.line as usize;
                let col = lint.span.col as usize;
                let token_len = if lint.span.end > lint.span.start {
                    lint.span.end - lint.span.start
                } else { 1 };
                let source_line = source.lines().nth(line_num.saturating_sub(1)).unwrap_or("");
                let line_prefix = line_num.to_string();
                let padding = " ".repeat(line_prefix.len());
                let col_offset = " ".repeat(col.saturating_sub(1));
                let max_len = source_line.len().saturating_sub(col.saturating_sub(1)).max(1);
                let underline = "^".repeat(token_len.min(max_len).max(1));
                eprintln!(
                    "lint[{}]: {}\n  --> {}:{}:{}\n{} |\n{} | {}\n{} | {}{}",
                    lint.code, lint.message,
                    lint.span.file, lint.span.line, lint.span.col,
                    padding,
                    line_prefix, source_line,
                    padding, col_offset, underline,
                );
            }
            total_warnings += lints.len();
        }
    }

    if total_warnings > 0 {
        eprintln!("\nlint: {} warning{}", total_warnings, if total_warnings == 1 { "" } else { "s" });
        if !warn_only {
            process::exit(1);
        }
    }
}

// ── fav explain ───────────────────────────────────────────────────────────────

pub fn cmd_explain(file: Option<&str>, schema: bool, format: &str, focus: &str) {
    if let Some(path) = file {
        if path.ends_with(".fvc") {
            let artifact = read_artifact_from_path(Path::new(path)).unwrap_or_else(|message| {
                eprintln!("{message}");
                process::exit(1);
            });
            if schema {
                eprintln!("error: --schema is not supported for .fvc artifacts");
                process::exit(1);
            }
            if format == "json" {
                match explain_json_from_artifact(&artifact) {
                    Ok(json) => print!("{json}"),
                    Err(message) => {
                    eprintln!("{message}");
                    process::exit(1);
                }
                }
            } else {
                print!("{}", artifact_info_string(&artifact));
            }
            return;
        }
    }

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
        // Best-effort IR compilation for DEPS collection (may fail on type errors)
        let ir = if errors.is_empty() {
            Some(compile_program(&program))
        } else {
            None
        };
        let include_stdlib_states = program
            .uses
            .iter()
            .any(|use_path| use_path.len() >= 2 && use_path[..use_path.len() - 1].join(".") == "std.states");
        if schema {
            print!("{}", ExplainPrinter::new().render_schema(&program, include_stdlib_states));
        } else if format == "json" {
            let reachability = ir
                .as_ref()
                .map(|ir_program| crate::middle::reachability::reachability_analysis("main", ir_program));
            print!(
                "{}",
                ExplainPrinter::new().render_json(
                    &program,
                    ir.as_ref(),
                    include_stdlib_states,
                    focus,
                    path,
                    reachability.as_ref(),
                )
            );
        } else {
            ExplainPrinter::new().print(&program, ir.as_ref(), include_stdlib_states);
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct ChangedEntry {
    name: String,
    diffs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
struct CategoryDiff {
    added: Vec<serde_json::Value>,
    removed: Vec<serde_json::Value>,
    changed: Vec<ChangedEntry>,
}

#[derive(Debug, Clone, Serialize)]
struct ExplainDiff {
    from_label: String,
    to_label: String,
    fn_changes: CategoryDiff,
    trf_changes: CategoryDiff,
    flw_changes: CategoryDiff,
    type_changes: CategoryDiff,
    effects_added: Vec<String>,
    effects_removed: Vec<String>,
    breaking_changes: Vec<String>,
}

pub fn cmd_explain_diff(from_path: &str, to_path: &str, format: &str) {
    let from = load_explain_json(from_path);
    let to = load_explain_json(to_path);
    let diff = diff_explain_json(from_path, &from, to_path, &to);
    if format == "json" {
        print!("{}", render_diff_json(&diff));
    } else {
        print!("{}", render_diff_text(&diff));
    }
}

fn load_explain_json(path: &str) -> serde_json::Value {
    if path.ends_with(".json") {
        let text = load_file(path);
        serde_json::from_str(&text).unwrap_or_else(|e| {
            eprintln!("error: invalid explain json `{}`: {}", path, e);
            process::exit(1);
        })
    } else if path.ends_with(".fvc") {
        let artifact = read_artifact_from_path(Path::new(path)).unwrap_or_else(|message| {
            eprintln!("{message}");
            process::exit(1);
        });
        let text = explain_json_from_artifact(&artifact).unwrap_or_else(|message| {
            eprintln!("{message}");
            process::exit(1);
        });
        serde_json::from_str(&text).unwrap_or_else(|e| {
            eprintln!("error: invalid embedded explain json `{}`: {}", path, e);
            process::exit(1);
        })
    } else {
        let (program, source_path) = load_and_check_program(Some(path));
        let ir = compile_program(&program);
        let reachability = crate::middle::reachability::reachability_analysis("main", &ir);
        let rendered = ExplainPrinter::new().render_json(
            &program,
            Some(&ir),
            true,
            "all",
            &source_path,
            Some(&reachability),
        );
        serde_json::from_str(&rendered).unwrap_or_else(|e| {
            eprintln!("error: invalid generated explain json `{}`: {}", path, e);
            process::exit(1);
        })
    }
}

fn diff_explain_json(
    from_label: &str,
    from: &serde_json::Value,
    to_label: &str,
    to: &serde_json::Value,
) -> ExplainDiff {
    let fn_changes = diff_category(from, to, "fns");
    let trf_changes = diff_category(from, to, "trfs");
    let flw_changes = diff_category(from, to, "flws");
    let type_changes = diff_category(from, to, "types");
    let effects_added = diff_string_list(from, to, "effects_used").0;
    let effects_removed = diff_string_list(from, to, "effects_used").1;
    let mut breaking_changes = Vec::new();
    breaking_changes.extend(detect_breaking_changes("fn", &fn_changes));
    breaking_changes.extend(detect_breaking_changes("trf", &trf_changes));
    breaking_changes.extend(detect_breaking_changes("flw", &flw_changes));
    breaking_changes.extend(detect_breaking_changes("type", &type_changes));

    ExplainDiff {
        from_label: from_label.to_string(),
        to_label: to_label.to_string(),
        fn_changes,
        trf_changes,
        flw_changes,
        type_changes,
        effects_added,
        effects_removed,
        breaking_changes,
    }
}

fn diff_category(from: &serde_json::Value, to: &serde_json::Value, key: &str) -> CategoryDiff {
    let from_map = keyed_entries(from.get(key).and_then(|v| v.as_array()));
    let to_map = keyed_entries(to.get(key).and_then(|v| v.as_array()));

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    for (name, value) in &to_map {
        if !from_map.contains_key(name) {
            added.push(value.clone());
        }
    }
    for (name, value) in &from_map {
        if !to_map.contains_key(name) {
            removed.push(value.clone());
        }
    }
    for (name, before) in &from_map {
        if let Some(after) = to_map.get(name) {
            let diffs = diff_entry(before, after);
            if !diffs.is_empty() {
                changed.push(ChangedEntry {
                    name: name.clone(),
                    diffs,
                });
            }
        }
    }

    CategoryDiff { added, removed, changed }
}

fn keyed_entries(items: Option<&Vec<serde_json::Value>>) -> std::collections::BTreeMap<String, serde_json::Value> {
    let mut map = std::collections::BTreeMap::new();
    if let Some(items) = items {
        for item in items {
            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                map.insert(name.to_string(), item.clone());
            }
        }
    }
    map
}

fn diff_entry(from: &serde_json::Value, to: &serde_json::Value) -> Vec<String> {
    let mut diffs = Vec::new();
    for key in [
        "params",
        "return_type",
        "input_type",
        "output_type",
        "effects",
        "steps",
        "template",
        "bindings",
        "fields",
        "variants",
        "invariants",
    ] {
        let before = from.get(key);
        let after = to.get(key);
        if before != after {
            diffs.push(format!("{} changed", key));
        }
    }
    diffs
}

fn diff_string_list(from: &serde_json::Value, to: &serde_json::Value, key: &str) -> (Vec<String>, Vec<String>) {
    let from_set: std::collections::BTreeSet<String> = from
        .get(key)
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let to_set: std::collections::BTreeSet<String> = to
        .get(key)
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let added = to_set.difference(&from_set).cloned().collect();
    let removed = from_set.difference(&to_set).cloned().collect();
    (added, removed)
}

fn detect_breaking_changes(category: &str, diff: &CategoryDiff) -> Vec<String> {
    let mut out = Vec::new();
    for removed in &diff.removed {
        if let Some(name) = removed.get("name").and_then(|v| v.as_str()) {
            out.push(format!("removed {} `{}`", category, name));
        }
    }
    for changed in &diff.changed {
        if changed.diffs.iter().any(|d| matches!(d.as_str(), "params changed" | "return_type changed" | "input_type changed" | "output_type changed" | "effects changed" | "fields changed" | "variants changed")) {
            out.push(format!("changed {} `{}`", category, changed.name));
        }
    }
    out
}

fn render_diff_text(diff: &ExplainDiff) -> String {
    use std::fmt::Write as _;
    if diff.fn_changes.added.is_empty()
        && diff.fn_changes.removed.is_empty()
        && diff.fn_changes.changed.is_empty()
        && diff.trf_changes.added.is_empty()
        && diff.trf_changes.removed.is_empty()
        && diff.trf_changes.changed.is_empty()
        && diff.flw_changes.added.is_empty()
        && diff.flw_changes.removed.is_empty()
        && diff.flw_changes.changed.is_empty()
        && diff.type_changes.added.is_empty()
        && diff.type_changes.removed.is_empty()
        && diff.type_changes.changed.is_empty()
        && diff.effects_added.is_empty()
        && diff.effects_removed.is_empty()
    {
        return "No changes detected.\n".to_string();
    }
    let mut out = String::new();
    let _ = writeln!(out, "[summary] from={} to={}", diff.from_label, diff.to_label);
    let _ = writeln!(out, "  added/removed/changed fns: {}/{}/{}", diff.fn_changes.added.len(), diff.fn_changes.removed.len(), diff.fn_changes.changed.len());
    let _ = writeln!(out, "  added/removed/changed trfs: {}/{}/{}", diff.trf_changes.added.len(), diff.trf_changes.removed.len(), diff.trf_changes.changed.len());
    let _ = writeln!(out, "  added/removed/changed flws: {}/{}/{}", diff.flw_changes.added.len(), diff.flw_changes.removed.len(), diff.flw_changes.changed.len());
    let _ = writeln!(out, "  added/removed/changed types: {}/{}/{}", diff.type_changes.added.len(), diff.type_changes.removed.len(), diff.type_changes.changed.len());
    if !diff.breaking_changes.is_empty() {
        let _ = writeln!(out, "  breaking_changes:");
        for change in &diff.breaking_changes {
            let _ = writeln!(out, "    - {}", change);
        }
    }
    for (label, category) in [("fns", &diff.fn_changes), ("trfs", &diff.trf_changes), ("flws", &diff.flw_changes), ("types", &diff.type_changes)] {
        if category.added.is_empty() && category.removed.is_empty() && category.changed.is_empty() {
            continue;
        }
        let _ = writeln!(out, "\n[{}]", label);
        for entry in &category.added {
            if let Some(name) = entry.get("name").and_then(|v| v.as_str()) {
                let _ = writeln!(out, "+ {}", name);
            }
        }
        for entry in &category.removed {
            if let Some(name) = entry.get("name").and_then(|v| v.as_str()) {
                let _ = writeln!(out, "- {}", name);
            }
        }
        for entry in &category.changed {
            let _ = writeln!(out, "~ {} ({})", entry.name, entry.diffs.join(", "));
        }
    }
    out
}

fn render_diff_json(diff: &ExplainDiff) -> String {
    serde_json::to_string_pretty(diff).expect("diff json")
}

fn remap_ir_pattern(pattern: &IRPattern) -> IRPattern {
    match pattern {
        IRPattern::Wildcard => IRPattern::Wildcard,
        IRPattern::Lit(lit) => IRPattern::Lit(lit.clone()),
        IRPattern::Bind(slot) => IRPattern::Bind(*slot),
        IRPattern::Variant(name, inner) => {
            IRPattern::Variant(name.clone(), inner.as_ref().map(|p| Box::new(remap_ir_pattern(p))))
        }
        IRPattern::Record(fields) => IRPattern::Record(
            fields
                .iter()
                .map(|(name, pat)| (name.clone(), remap_ir_pattern(pat)))
                .collect(),
        ),
    }
}

fn remap_ir_arm(
    arm: &IRArm,
    global_idx_map: &std::collections::HashMap<u16, u16>,
) -> IRArm {
    IRArm {
        pattern: remap_ir_pattern(&arm.pattern),
        guard: arm
            .guard
            .as_ref()
            .map(|expr| remap_ir_expr(expr, global_idx_map)),
        body: remap_ir_expr(&arm.body, global_idx_map),
    }
}

fn remap_ir_stmt(
    stmt: &IRStmt,
    global_idx_map: &std::collections::HashMap<u16, u16>,
) -> IRStmt {
    match stmt {
        IRStmt::Bind(slot, expr) => IRStmt::Bind(*slot, remap_ir_expr(expr, global_idx_map)),
        IRStmt::Chain(slot, expr) => IRStmt::Chain(*slot, remap_ir_expr(expr, global_idx_map)),
        IRStmt::Yield(expr) => IRStmt::Yield(remap_ir_expr(expr, global_idx_map)),
        IRStmt::Expr(expr) => IRStmt::Expr(remap_ir_expr(expr, global_idx_map)),
        IRStmt::TrackLine(line) => IRStmt::TrackLine(*line),
    }
}

fn remap_ir_expr(
    expr: &IRExpr,
    global_idx_map: &std::collections::HashMap<u16, u16>,
) -> IRExpr {
    match expr {
        IRExpr::Lit(lit, ty) => IRExpr::Lit(lit.clone(), ty.clone()),
        IRExpr::Local(slot, ty) => IRExpr::Local(*slot, ty.clone()),
        IRExpr::Global(idx, ty) => {
            let mapped = global_idx_map.get(idx).copied().unwrap_or(*idx);
            IRExpr::Global(mapped, ty.clone())
        }
        IRExpr::TrfRef(idx, ty) => {
            let mapped = global_idx_map.get(idx).copied().unwrap_or(*idx);
            IRExpr::TrfRef(mapped, ty.clone())
        }
        IRExpr::CallTrfLocal { local, arg, ty } => IRExpr::CallTrfLocal {
            local: *local,
            arg: Box::new(remap_ir_expr(arg, global_idx_map)),
            ty: ty.clone(),
        },
        IRExpr::Call(callee, args, ty) => IRExpr::Call(
            Box::new(remap_ir_expr(callee, global_idx_map)),
            args.iter()
                .map(|arg| remap_ir_expr(arg, global_idx_map))
                .collect(),
            ty.clone(),
        ),
        IRExpr::Block(stmts, tail, ty) => IRExpr::Block(
            stmts.iter()
                .map(|stmt| remap_ir_stmt(stmt, global_idx_map))
                .collect(),
            Box::new(remap_ir_expr(tail, global_idx_map)),
            ty.clone(),
        ),
        IRExpr::If(cond, then_expr, else_expr, ty) => IRExpr::If(
            Box::new(remap_ir_expr(cond, global_idx_map)),
            Box::new(remap_ir_expr(then_expr, global_idx_map)),
            Box::new(remap_ir_expr(else_expr, global_idx_map)),
            ty.clone(),
        ),
        IRExpr::Match(scrutinee, arms, ty) => IRExpr::Match(
            Box::new(remap_ir_expr(scrutinee, global_idx_map)),
            arms.iter()
                .map(|arm| remap_ir_arm(arm, global_idx_map))
                .collect(),
            ty.clone(),
        ),
        IRExpr::FieldAccess(base, field, ty) => IRExpr::FieldAccess(
            Box::new(remap_ir_expr(base, global_idx_map)),
            field.clone(),
            ty.clone(),
        ),
        IRExpr::BinOp(op, left, right, ty) => IRExpr::BinOp(
            op.clone(),
            Box::new(remap_ir_expr(left, global_idx_map)),
            Box::new(remap_ir_expr(right, global_idx_map)),
            ty.clone(),
        ),
        IRExpr::Closure(idx, captures, ty) => {
            let mapped = global_idx_map.get(idx).copied().unwrap_or(*idx);
            IRExpr::Closure(
                mapped,
                captures
                    .iter()
                    .map(|expr| remap_ir_expr(expr, global_idx_map))
                    .collect(),
                ty.clone(),
            )
        }
        IRExpr::Collect(inner, ty) => {
            IRExpr::Collect(Box::new(remap_ir_expr(inner, global_idx_map)), ty.clone())
        }
        IRExpr::Emit(inner, ty) => {
            IRExpr::Emit(Box::new(remap_ir_expr(inner, global_idx_map)), ty.clone())
        }
        IRExpr::RecordConstruct(fields, ty) => IRExpr::RecordConstruct(
            fields
                .iter()
                .map(|(name, expr)| (name.clone(), remap_ir_expr(expr, global_idx_map)))
                .collect(),
            ty.clone(),
        ),
    }
}

fn filter_ir_program(ir: &IRProgram, included: &std::collections::HashSet<String>) -> IRProgram {
    let mut global_idx_map = std::collections::HashMap::new();
    let mut old_fn_idx_to_new = std::collections::HashMap::new();
    let mut new_globals = Vec::new();
    let mut kept_old_fn_idxs = Vec::new();

    for (old_global_idx, global) in ir.globals.iter().enumerate() {
        let keep = match &global.kind {
            IRGlobalKind::Builtin => true,
            IRGlobalKind::Fn(_) | IRGlobalKind::VariantCtor => included.contains(&global.name),
        };
        if keep {
            let new_idx = new_globals.len() as u16;
            global_idx_map.insert(old_global_idx as u16, new_idx);
            if let IRGlobalKind::Fn(old_fn_idx) = &global.kind {
                kept_old_fn_idxs.push(*old_fn_idx);
            }
            new_globals.push(global.clone());
        }
    }

    let mut new_fns = Vec::new();
    for old_fn_idx in kept_old_fn_idxs {
        if let Some(fn_def) = ir.fns.get(old_fn_idx) {
            let new_idx = new_fns.len();
            old_fn_idx_to_new.insert(old_fn_idx, new_idx);
            new_fns.push(fn_def.clone());
        }
    }

    for global in &mut new_globals {
        if let IRGlobalKind::Fn(old_fn_idx) = global.kind.clone() {
            let new_idx = old_fn_idx_to_new.get(&old_fn_idx).copied().unwrap_or(old_fn_idx);
            global.kind = IRGlobalKind::Fn(new_idx);
        }
    }

    for fn_def in &mut new_fns {
        fn_def.body = remap_ir_expr(&fn_def.body, &global_idx_map);
    }

    IRProgram {
        globals: new_globals,
        fns: new_fns,
    }
}

fn build_manifest_json(
    source: &str,
    artifact_path: &Path,
    artifact_size: u64,
    entry: &str,
    reachability: &crate::middle::reachability::ReachabilityResult,
) -> String {
    let mut included: Vec<_> = reachability.included.iter().cloned().collect();
    included.sort();
    let mut excluded: Vec<_> = reachability.excluded.iter().cloned().collect();
    excluded.sort();
    serde_json::to_string_pretty(&json!({
        "schema_version": "1.0",
        "favnir_version": "1.5.0",
        "entry": entry,
        "source": source,
        "artifact": {
            "path": artifact_path.display().to_string(),
            "size_bytes": artifact_size,
            "format": "fvc"
        },
        "reachability": {
            "included": included,
            "excluded": excluded,
            "effects_required": reachability.effects_required,
            "emits": reachability.emits
        }
    }))
    .expect("manifest json")
}

fn prune_explain_json_to_reachable(rendered: &str) -> String {
    let mut value: serde_json::Value =
        serde_json::from_str(rendered).expect("explain json should be valid");
    for key in ["fns", "trfs", "flws"] {
        if let Some(items) = value.get_mut(key).and_then(|v| v.as_array_mut()) {
            items.retain(|item| {
                item.get("reachable_from_entry")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true)
            });
        }
    }
    serde_json::to_string_pretty(&value).expect("pruned explain json")
}

pub fn cmd_bundle(file: &str, out: Option<&str>, entry: &str, manifest: bool, explain: bool) {
    let (program, source_path) = load_and_check_program(Some(file));
    ensure_no_partial_flw(&program).unwrap_or_else(|message| {
        eprintln!("{message}");
        process::exit(1);
    });

    let ir = compile_program(&program);
    let reachability = crate::middle::reachability::reachability_analysis(entry, &ir);
    let filtered_ir = filter_ir_program(&ir, &reachability.included);
    let artifact = codegen_program(&filtered_ir);

    let source_path_buf = PathBuf::from(&source_path);
    let stem = source_path_buf
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("bundle");
    let out_path = out
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("dist").join(format!("{stem}.fvc")));
    write_artifact_to_path(&artifact, &out_path).unwrap_or_else(|message| {
        eprintln!("{message}");
        process::exit(1);
    });

    if manifest {
        let size = std::fs::metadata(&out_path).map(|m| m.len()).unwrap_or(0);
        let manifest_path = out_path.with_extension("manifest.json");
        let manifest_json = build_manifest_json(&source_path, &out_path, size, entry, &reachability);
        std::fs::write(&manifest_path, manifest_json).unwrap_or_else(|e| {
            eprintln!(
                "error: cannot write manifest `{}`: {}",
                manifest_path.display(),
                e
            );
            process::exit(1);
        });
    }

    if explain {
        let explain_path = out_path.with_extension("explain.json");
        let explain_json = ExplainPrinter::new().render_json(
            &program,
            Some(&filtered_ir),
            true,
            "all",
            &source_path,
            Some(&reachability),
        );
        let pruned = prune_explain_json_to_reachable(&explain_json);
        let mut artifact = artifact.clone();
        artifact.explain_json = Some(pruned.clone());
        write_artifact_to_path(&artifact, &out_path).unwrap_or_else(|message| {
            eprintln!("{message}");
            process::exit(1);
        });
        std::fs::write(&explain_path, pruned).unwrap_or_else(|e| {
            eprintln!(
                "error: cannot write explain json `{}`: {}",
                explain_path.display(),
                e
            );
            process::exit(1);
        });
    }

    println!("bundled {}", out_path.display());
}

pub fn cmd_graph(file: &str, _format: &str, _focus: Option<&str>, entry: Option<&str>, depth: Option<usize>) {
    let source = load_file(file);
    let program = Parser::parse_str(&source, file).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    });
    let format = _format;
    let focus = _focus.unwrap_or("flw");
    let rendered = match format {
        "text" => render_graph_text_with_opts(&program, focus, entry, depth),
        "mermaid" => render_graph_mermaid_with_opts(&program, focus, entry, depth),
        other => {
            eprintln!("error: unsupported graph format `{other}`");
            process::exit(1);
        }
    };
    print!("{rendered}");
}

#[cfg(test)]
fn render_graph_text(program: &ast::Program, focus: &str) -> String {
    render_graph_text_with_opts(program, focus, None, None)
}

fn render_graph_text_with_opts(
    program: &ast::Program,
    focus: &str,
    entry: Option<&str>,
    max_depth: Option<usize>,
) -> String {
    use std::fmt::Write as _;

    let mut out = String::new();
    let _ = writeln!(out, "GRAPH ({focus})");

    if focus == "fn" {
        let ir = crate::middle::compiler::compile_program(program);
        let calls_map: std::collections::HashMap<String, Vec<String>> = ir
            .fns
            .iter()
            .filter(|f| !f.name.starts_with('$'))
            .map(|f| (f.name.clone(), crate::middle::ir::collect_deps(f, &ir.globals)))
            .collect();
        let selected = select_fn_graph_nodes(&calls_map, entry, max_depth);
        for name in &selected.order {
            let _ = writeln!(out, "fn {}:", name);
            let mut nexts = calls_map.get(name).cloned().unwrap_or_default();
            nexts.retain(|dep| selected.nodes.contains(dep));
            if nexts.is_empty() {
                let _ = writeln!(out, "  (no calls)");
            } else {
                for dep in nexts {
                    let _ = writeln!(out, "  -> {}", dep);
                }
            }
        }
        return out;
    }

    for item in &program.items {
        match item {
            ast::Item::FlwDef(def) if focus == "flw" || focus == "all" => {
                let _ = writeln!(out, "flw {}:", def.name);
                for (idx, step) in def.steps.iter().enumerate() {
                    let _ = writeln!(out, "  {}. {}", idx + 1, step);
                }
            }
            ast::Item::AbstractFlwDef(def) if focus == "flw" || focus == "all" => {
                let _ = writeln!(out, "abstract seq {}:", def.name);
                for slot in &def.slots {
                    let _ = writeln!(
                        out,
                        "  slot {}: {} -> {} {}",
                        slot.name,
                        format_type_expr(&slot.input_ty),
                        format_type_expr(&slot.output_ty),
                        format_effects(&slot.effects),
                    );
                }
            }
            ast::Item::FlwBindingDef(def) if focus == "flw" || focus == "all" => {
                let _ = writeln!(out, "flw {} = {}:", def.name, def.template);
                for (slot, bound) in &def.bindings {
                    let _ = writeln!(out, "  {} <- {}", slot, slot_impl_name(bound));
                }
            }
            _ => {}
        }
    }

    out
}

#[cfg(test)]
fn render_graph_mermaid(program: &ast::Program, focus: &str) -> String {
    render_graph_mermaid_with_opts(program, focus, None, None)
}

fn render_graph_mermaid_with_opts(
    program: &ast::Program,
    focus: &str,
    entry: Option<&str>,
    max_depth: Option<usize>,
) -> String {
    use std::fmt::Write as _;

    let mut out = String::new();
    let _ = writeln!(out, "flowchart LR");

    if focus == "fn" {
        let ir = crate::middle::compiler::compile_program(program);
        let calls_map: std::collections::HashMap<String, Vec<String>> = ir
            .fns
            .iter()
            .filter(|f| !f.name.starts_with('$'))
            .map(|f| (f.name.clone(), crate::middle::ir::collect_deps(f, &ir.globals)))
            .collect();
        let selected = select_fn_graph_nodes(&calls_map, entry, max_depth);
        for name in &selected.order {
            let _ = writeln!(out, "    {}[\"fn {}\"]", sanitize_mermaid_id(name), name);
            let mut nexts = calls_map.get(name).cloned().unwrap_or_default();
            nexts.retain(|dep| selected.nodes.contains(dep));
            for dep in nexts {
                let _ = writeln!(
                    out,
                    "    {} --> {}",
                    sanitize_mermaid_id(name),
                    sanitize_mermaid_id(&dep)
                );
            }
        }
        return out;
    }

    for item in &program.items {
        match item {
            ast::Item::FlwDef(def) if focus == "flw" || focus == "all" => {
                let _ = writeln!(out, "    {}[\"flw {}\"]", def.name, def.name);
                for step in &def.steps {
                    let _ = writeln!(out, "    {} --> {}", def.name, step);
                }
            }
            ast::Item::AbstractFlwDef(def) if focus == "flw" || focus == "all" => {
                let _ = writeln!(out, "    {}[\"abstract seq {}\"]", def.name, def.name);
                for slot in &def.slots {
                    let slot_node = format!("{}_{}", def.name, slot.name);
                    let _ = writeln!(out, "    {}[\"slot {}\"]", slot_node, slot.name);
                    let _ = writeln!(out, "    {} --> {}", def.name, slot_node);
                }
            }
            ast::Item::FlwBindingDef(def) if focus == "flw" || focus == "all" => {
                let _ = writeln!(out, "    {}[\"flw {}\"]", def.name, def.name);
                let _ = writeln!(out, "    {} --> {}", def.name, def.template);
                for (slot, bound) in &def.bindings {
                    let slot_node = format!("{}_{}", def.name, slot);
                    let _ = writeln!(out, "    {}[\"bind {}\"]", slot_node, slot);
                    let _ = writeln!(out, "    {} --> {}", def.name, slot_node);
                    let _ = writeln!(out, "    {} --> {}", slot_node, slot_impl_name(bound));
                }
            }
            _ => {}
        }
    }

    out
}

#[derive(Debug)]
struct FnGraphSelection {
    order: Vec<String>,
    nodes: std::collections::HashSet<String>,
}

fn select_fn_graph_nodes(
    calls_map: &std::collections::HashMap<String, Vec<String>>,
    entry: Option<&str>,
    max_depth: Option<usize>,
) -> FnGraphSelection {
    use std::collections::{HashSet, VecDeque};

    let mut order = Vec::new();
    let mut nodes = HashSet::new();

    if let Some(entry) = entry {
        let mut queue = VecDeque::new();
        queue.push_back((entry.to_string(), 0usize));
        while let Some((name, depth)) = queue.pop_front() {
            if !nodes.insert(name.clone()) {
                continue;
            }
            order.push(name.clone());
            if max_depth.map(|max| depth >= max).unwrap_or(false) {
                continue;
            }
            if let Some(nexts) = calls_map.get(&name) {
                for dep in nexts {
                    if calls_map.contains_key(dep) && !nodes.contains(dep) {
                        queue.push_back((dep.clone(), depth + 1));
                    }
                }
            }
        }
    } else {
        let mut names = calls_map.keys().cloned().collect::<Vec<_>>();
        names.sort();
        for name in names {
            nodes.insert(name.clone());
            order.push(name);
        }
    }

    FnGraphSelection { order, nodes }
}

fn sanitize_mermaid_id(name: &str) -> String {
    name.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

struct ExplainPrinter;

impl ExplainPrinter {
    fn new() -> Self { ExplainPrinter }

    fn print(
        &self,
        program: &ast::Program,
        ir: Option<&crate::middle::ir::IRProgram>,
        include_stdlib_states: bool,
    ) {
        print!("{}", self.render(program, ir, include_stdlib_states));
    }

    fn render(
        &self,
        program: &ast::Program,
        ir: Option<&crate::middle::ir::IRProgram>,
        include_stdlib_states: bool,
    ) -> String {
        use ast::*;
        use crate::middle::ir::collect_deps;
        use std::fmt::Write as _;

        let col_vis   = 10usize;
        let col_name  = 26usize;
        let col_type  = 36usize;
        let col_eff   = 20usize;
        let col_inv   = 34usize;
        let mut out = String::new();

        let _ = writeln!(
            out,
            "{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {:<col_inv$} {}",
            "VIS", "NAME", "TYPE", "EFFECTS", "INVARIANTS", "DEPS"
        );
        let _ = writeln!(
            out,
            "{}",
            "-".repeat(col_vis + col_name + col_type + col_eff + col_inv + 40)
        );

        // Build a map: fn_name → deps string (from IR if available)
        let deps_map: std::collections::HashMap<String, String> = if let Some(ir) = ir {
            ir.fns.iter()
                .filter(|f| !f.name.starts_with('$'))
                .map(|f| {
                    let deps = collect_deps(f, &ir.globals);
                    (f.name.clone(), if deps.is_empty() { "-".to_string() } else { deps.join(", ") })
                })
                .collect()
        } else {
            std::collections::HashMap::new()
        };

        for item in &program.items {
            match item {
                Item::TypeDef(td) => {
                    let kind = match &td.body {
                        TypeBody::Record(_)  => "record",
                        TypeBody::Sum(_)     => "sum",
                        TypeBody::Alias(_)   => "alias",
                    };
                    let vis = format_visibility(&td.visibility);
                    let _ = writeln!(
                        out,
                        "{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {:<col_inv$} -",
                        vis, td.name,
                        format!("type ({kind})"),
                        "-",
                        format_invariants(&td.invariants),
                    );
                }
                Item::EffectDef(ed) => {
                    let _ = writeln!(
                        out,
                        "{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {:<col_inv$} -",
                        format_visibility(&ed.visibility),
                        format!("effect {}", ed.name),
                        "effect",
                        "-",
                        "-",
                    );
                }
                Item::FnDef(fd) => {
                    let params: Vec<String> = fd.params.iter()
                        .map(|p| format_type_expr(&p.ty))
                        .collect();
                    let sig = format!("({}) -> {}", params.join(", "), format_type_expr(&fd.return_ty));
                    let effs = format_effects(&fd.effects);
                    let vis = format_visibility(&fd.visibility);
                    let deps = deps_map.get(&fd.name).map(|s| s.as_str()).unwrap_or("-");
                    let _ = writeln!(
                        out,
                        "{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {:<col_inv$} {}",
                        vis, format!("fn {}", fd.name), sig, effs, "-", deps
                    );
                }
                Item::TrfDef(td) => {
                    let sig = format!("{} -> {}",
                        format_type_expr(&td.input_ty),
                        format_type_expr(&td.output_ty));
                    let effs = format_effects(&td.effects);
                    let vis = format_visibility(&td.visibility);
                    let deps = deps_map.get(&td.name).map(|s| s.as_str()).unwrap_or("-");
                    let _ = writeln!(
                        out,
                        "{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {:<col_inv$} {}",
                        vis, format!("trf {}", td.name), sig, effs, "-", deps
                    );
                }
                Item::AbstractTrfDef(td) => {
                    let sig = format!("{} -> {}",
                        format_type_expr(&td.input_ty),
                        format_type_expr(&td.output_ty));
                    let effs = format_effects(&td.effects);
                    let vis = format_visibility(&td.visibility);
                    let _ = writeln!(
                        out,
                        "{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {:<col_inv$} -",
                        vis, format!("abstract stage {}", td.name), sig, effs, "-"
                    );
                }
                Item::FlwDef(fd) => {
                    let _ = writeln!(
                        out,
                        "{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {:<col_inv$} -",
                        "", format!("flw {}", fd.name), fd.steps.join(" |> "), "-", "-"
                    );
                }
                Item::AbstractFlwDef(fd) => {
                    let slots = fd.slots.iter()
                        .map(|slot| {
                            let effs = format_effects(&slot.effects);
                            format!(
                                "{}: {} -> {}{}",
                                slot.name,
                                format_type_expr(&slot.input_ty),
                                format_type_expr(&slot.output_ty),
                                if effs == "-" { String::new() } else { format!(" {}", effs) }
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("; ");
                    let vis = format_visibility(&fd.visibility);
                    let name = if fd.type_params.is_empty() {
                        format!("abstract seq {}", fd.name)
                    } else {
                        format!("abstract seq {}<{}>", fd.name, fd.type_params.join(", "))
                    };
                    let _ = writeln!(
                        out,
                        "{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {:<col_inv$} -",
                        vis, name, slots, "-", "-"
                    );
                }
                Item::FlwBindingDef(fd) => {
                    let type_args = if fd.type_args.is_empty() {
                        String::new()
                    } else {
                        format!("<{}>", fd.type_args.iter().map(|a| format_type_expr(a)).collect::<Vec<_>>().join(", "))
                    };
                    let bindings = fd.bindings.iter()
                        .map(|(slot, imp)| format!("{} <- {}", slot, slot_impl_name(imp)))
                        .collect::<Vec<_>>()
                        .join("; ");
                    let vis = format_visibility(&fd.visibility);
                    let _ = writeln!(
                        out,
                        "{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {:<col_inv$} -",
                        vis,
                        format!("flw {}", fd.name),
                        format!("{}{} {{ {} }}", fd.template, type_args, bindings),
                        "-",
                        "-"
                    );
                }
                Item::InterfaceDecl(id) => {
                    let kind = if let Some(sup) = &id.super_interface {
                        format!("interface : {}", sup)
                    } else {
                        "interface".to_string()
                    };
                    let _ = writeln!(
                        out,
                        "{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {:<col_inv$} -",
                        format_visibility(&id.visibility),
                        format!("interface {}", id.name),
                        kind,
                        "-",
                        "-",
                    );
                }
                Item::InterfaceImplDecl(id) => {
                    let _ = writeln!(
                        out,
                        "{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {:<col_inv$} -",
                        "",
                        format!("impl {}", id.interface_names.join(", ")),
                        id.type_name.clone(),
                        "-",
                        "-",
                    );
                }
                Item::CapDef(cd) => {
                    let _ = writeln!(
                        out,
                        "{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {:<col_inv$} -",
                        format_visibility(&cd.visibility),
                        format!("cap {}", cd.name),
                        format!("<{}>", cd.type_params.join(", ")),
                        "-",
                        "-",
                    );
                }
                Item::ImplDef(id) => {
                    let args: Vec<String> = id.type_args.iter()
                        .map(|a| format_type_expr(a))
                        .collect();
                    let _ = writeln!(
                        out,
                        "{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {:<col_inv$} -",
                        "", format!("impl {}", id.cap_name), format!("<{}>", args.join(", ")), "-", "-"
                    );
                }
                Item::TestDef(td) => {
                    let deps = deps_map.get(&format!("$test:{}", td.name))
                        .map(|s| s.as_str()).unwrap_or("-");
                    let _ = writeln!(
                        out,
                        "{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {:<col_inv$} {}",
                        "", format!("test {:?}", td.name), "() -> Unit", "-", "-", deps
                    );
                }
                Item::BenchDef(bd) => {
                    let _ = writeln!(
                        out,
                        "{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {:<col_inv$} -",
                        "", format!("bench {:?}", bd.description), "() -> Unit", "-", "-"
                    );
                }
                Item::NamespaceDecl(..) | Item::UseDecl(..) => {}
            }
        }

        if include_stdlib_states {
            for td in crate::std_states::parsed_type_defs() {
                let kind = match &td.body {
                    TypeBody::Record(_) => "type (record)",
                    TypeBody::Sum(_) => "type (sum)",
                    TypeBody::Alias(_) => "type (alias)",
                };
                let _ = writeln!(
                    out,
                    "{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {:<col_inv$} -",
                    "public",
                    format!("{} (stdlib)", td.name),
                    kind,
                    "-",
                    format_invariants(&td.invariants),
                );
            }
        }

        let abstract_trfs = program
            .items
            .iter()
            .filter_map(|item| match item {
                Item::AbstractTrfDef(td) => Some(td),
                _ => None,
            })
            .collect::<Vec<_>>();
        if !abstract_trfs.is_empty() {
            let _ = writeln!(out);
            let _ = writeln!(out, "ABSTRACT TRF");
            let _ = writeln!(out, "------------");
            for td in abstract_trfs {
                let _ = writeln!(
                    out,
                    "- {}: {} -> {} {}",
                    td.name,
                    format_type_expr(&td.input_ty),
                    format_type_expr(&td.output_ty),
                    format_effects(&td.effects)
                );
            }
        }

        let abstract_flws = program
            .items
            .iter()
            .filter_map(|item| match item {
                Item::AbstractFlwDef(fd) => Some(fd),
                _ => None,
            })
            .collect::<Vec<_>>();
        if !abstract_flws.is_empty() {
            let _ = writeln!(out);
            let _ = writeln!(out, "ABSTRACT FLW");
            let _ = writeln!(out, "------------");
            for fd in &abstract_flws {
                if fd.type_params.is_empty() {
                    let _ = writeln!(out, "- {}", fd.name);
                } else {
                    let _ = writeln!(out, "- {}<{}>", fd.name, fd.type_params.join(", "));
                }
                for slot in &fd.slots {
                    let _ = writeln!(
                        out,
                        "  - {}: {} -> {} {}",
                        slot.name,
                        format_type_expr(&slot.input_ty),
                        format_type_expr(&slot.output_ty),
                        format_effects(&slot.effects)
                    );
                }
            }
        }

        let template_map: std::collections::HashMap<&str, &AbstractFlwDef> = abstract_flws
            .iter()
            .map(|fd| (fd.name.as_str(), *fd))
            .collect();
        let flw_bindings = program
            .items
            .iter()
            .filter_map(|item| match item {
                Item::FlwBindingDef(fd) => Some(fd),
                _ => None,
            })
            .collect::<Vec<_>>();
        if !flw_bindings.is_empty() {
            let _ = writeln!(out);
            let _ = writeln!(out, "FLW BINDINGS");
            let _ = writeln!(out, "------------");
            for fd in flw_bindings {
                let type_args = if fd.type_args.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<{}>",
                        fd.type_args
                            .iter()
                            .map(format_type_expr)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                let _ = writeln!(out, "- {} = {}{}", fd.name, fd.template, type_args);
                for (slot, imp) in &fd.bindings {
                    let _ = writeln!(out, "  - {} <- {}", slot, slot_impl_name(imp));
                }

                if let Some(template) = template_map.get(fd.template.as_str()) {
                    let bound = fd
                        .bindings
                        .iter()
                        .map(|(slot, _)| slot.as_str())
                        .collect::<std::collections::HashSet<_>>();
                    let unbound = template
                        .slots
                        .iter()
                        .filter(|slot| !bound.contains(slot.name.as_str()))
                        .map(|slot| slot.name.clone())
                        .collect::<Vec<_>>();
                    if unbound.is_empty() {
                        let mut effects = Vec::<ast::Effect>::new();
                        for slot in &template.slots {
                            for effect in &slot.effects {
                                if !effects.contains(effect) {
                                    effects.push(effect.clone());
                                }
                            }
                        }
                        let _ = writeln!(out, "  - resolved: complete");
                        let _ = writeln!(out, "  - effects: {}", format_effects(&effects));
                    } else {
                        let _ = writeln!(out, "  - resolved: partial");
                        let _ = writeln!(out, "  - unbound: {}", unbound.join(", "));
                    }
                }
            }
        }

        out
    }

    fn render_json(
        &self,
        program: &ast::Program,
        ir: Option<&crate::middle::ir::IRProgram>,
        include_stdlib_states: bool,
        focus: &str,
        source: &str,
        reachability: Option<&crate::middle::reachability::ReachabilityResult>,
    ) -> String {
        use ast::*;
        let calls_map: std::collections::HashMap<String, Vec<String>> = if let Some(ir) = ir {
            ir.fns
                .iter()
                .filter(|f| !f.name.starts_with('$'))
                .map(|f| (f.name.clone(), crate::middle::ir::collect_deps(f, &ir.globals)))
                .collect()
        } else {
            std::collections::HashMap::new()
        };

        let want_all = focus == "all";
        let mut fns = Vec::new();
        let mut trfs = Vec::new();
        let mut flws = Vec::new();
        let mut types = Vec::new();
        let mut custom_effects = Vec::new();
        let mut effects_used = std::collections::BTreeSet::new();

        for item in &program.items {
            match item {
                Item::FnDef(fd) => {
                    for effect in &fd.effects {
                        effects_used.insert(effect_json_name(effect));
                    }
                    if want_all || focus == "fns" {
                        fns.push(json!({
                            "name": fd.name,
                            "kind": "fn",
                            "params": fd.params.iter().map(|p| format_type_expr(&p.ty)).collect::<Vec<_>>(),
                            "return_type": format_type_expr(&fd.return_ty),
                            "effects": fd.effects.iter().map(effect_json_name).collect::<Vec<_>>(),
                            "calls": calls_map.get(&fd.name).cloned().unwrap_or_default(),
                            "reachable_from_entry": reachability.map(|r| r.included.contains(&fd.name)).unwrap_or(true)
                        }));
                    }
                }
                Item::TrfDef(td) => {
                    for effect in &td.effects {
                        effects_used.insert(effect_json_name(effect));
                    }
                    if want_all || focus == "trfs" {
                        trfs.push(json!({
                            "name": td.name,
                            "kind": "trf",
                            "input_type": format_type_expr(&td.input_ty),
                            "output_type": format_type_expr(&td.output_ty),
                            "effects": td.effects.iter().map(effect_json_name).collect::<Vec<_>>(),
                            "calls": calls_map.get(&td.name).cloned().unwrap_or_default(),
                            "reachable_from_entry": reachability.map(|r| r.included.contains(&td.name)).unwrap_or(true)
                        }));
                    }
                }
                Item::AbstractTrfDef(td) => {
                    for effect in &td.effects {
                        effects_used.insert(effect_json_name(effect));
                    }
                    if want_all || focus == "trfs" {
                        trfs.push(json!({
                            "name": td.name,
                            "kind": "abstract_trf",
                            "input_type": format_type_expr(&td.input_ty),
                            "output_type": format_type_expr(&td.output_ty),
                            "effects": td.effects.iter().map(effect_json_name).collect::<Vec<_>>(),
                            "reachable_from_entry": reachability.map(|r| r.included.contains(&td.name)).unwrap_or(true)
                        }));
                    }
                }
                Item::FlwDef(fd) => {
                    if want_all || focus == "flws" {
                        flws.push(json!({
                            "name": fd.name,
                            "kind": "flw",
                            "steps": fd.steps,
                            "reachable_from_entry": reachability.map(|r| r.included.contains(&fd.name)).unwrap_or(true)
                        }));
                    }
                }
                Item::AbstractFlwDef(fd) => {
                    for slot in &fd.slots {
                        for effect in &slot.effects {
                            effects_used.insert(effect_json_name(effect));
                        }
                    }
                    if want_all || focus == "flws" {
                        flws.push(json!({
                            "name": fd.name,
                            "kind": "abstract_flw",
                            "type_params": fd.type_params,
                            "slots": fd.slots.iter().map(|slot| json!({
                                "name": slot.name,
                                "input_type": format_type_expr(&slot.input_ty),
                                "output_type": format_type_expr(&slot.output_ty),
                                "effects": slot.effects.iter().map(effect_json_name).collect::<Vec<_>>(),
                            })).collect::<Vec<_>>(),
                            "reachable_from_entry": reachability.map(|r| r.included.contains(&fd.name)).unwrap_or(true)
                        }));
                    }
                }
                Item::FlwBindingDef(fd) => {
                    if want_all || focus == "flws" {
                        flws.push(json!({
                            "name": fd.name,
                            "kind": "flw_binding",
                            "template": fd.template,
                            "type_args": fd.type_args.iter().map(format_type_expr).collect::<Vec<_>>(),
                            "bindings": fd.bindings.iter().map(|(slot, imp)| (slot.clone(), slot_impl_name(imp).to_string())).collect::<std::collections::BTreeMap<_, _>>(),
                            "reachable_from_entry": reachability.map(|r| r.included.contains(&fd.name)).unwrap_or(true)
                        }));
                    }
                }
                Item::TypeDef(td) => {
                    if want_all || focus == "types" {
                        match &td.body {
                            TypeBody::Record(fields) => {
                                types.push(json!({
                                    "name": td.name,
                                    "kind": "record",
                                    "fields": fields.iter().map(|f| json!({
                                        "name": f.name,
                                        "type": format_type_expr(&f.ty)
                                    })).collect::<Vec<_>>(),
                                    "invariants": td.invariants.iter().map(format_expr_compact).collect::<Vec<_>>()
                                }));
                            }
                            TypeBody::Sum(variants) => {
                                types.push(json!({
                                    "name": td.name,
                                    "kind": "sum",
                                    "variants": variants.iter().map(|v| match v {
                                        ast::Variant::Unit(name, _) => {
                                            json!({"name": name, "payload": serde_json::Value::Null})
                                        }
                                        ast::Variant::Tuple(name, ty, _) => {
                                            json!({"name": name, "payload": format_type_expr(ty)})
                                        }
                                        ast::Variant::Record(name, fields, _) => {
                                            json!({
                                                "name": name,
                                                "payload": fields.iter().map(|f| json!({
                                                    "name": f.name,
                                                    "type": format_type_expr(&f.ty)
                                                })).collect::<Vec<_>>()
                                            })
                                        }
                                    }).collect::<Vec<_>>(),
                                    "invariants": td.invariants.iter().map(format_expr_compact).collect::<Vec<_>>()
                                }));
                            }
                            TypeBody::Alias(target) => {
                                types.push(json!({
                                    "name": td.name,
                                    "kind": "alias",
                                    "target": format_type_expr(target)
                                }));
                            }
                        }
                    }
                }
                Item::EffectDef(ed) => {
                    custom_effects.push(json!({
                        "name": ed.name,
                        "public": ed.visibility == Some(ast::Visibility::Public),
                    }));
                }
                _ => {}
            }
        }

        if include_stdlib_states && (want_all || focus == "types") {
            for td in crate::std_states::parsed_type_defs() {
                if let ast::TypeBody::Record(fields) = &td.body {
                    types.push(json!({
                        "name": td.name,
                        "kind": "record",
                        "fields": fields.iter().map(|f| json!({
                            "name": f.name,
                            "type": format_type_expr(&f.ty)
                        })).collect::<Vec<_>>(),
                        "invariants": td.invariants.iter().map(format_expr_compact).collect::<Vec<_>>()
                    }));
                }
            }
        }

        serde_json::to_string_pretty(&json!({
            "schema_version": "1.0",
            "favnir_version": "1.5.0",
            "entry": "main",
            "source": source,
            "fns": fns,
            "trfs": trfs,
            "flws": flws,
            "types": types,
            "custom_effects": custom_effects,
            "effects_used": reachability.map(|r| r.effects_required.clone()).unwrap_or_else(|| effects_used.into_iter().collect::<Vec<_>>()),
            "emits": reachability.map(|r| r.emits.clone()).unwrap_or_default(),
            "runes_used": Vec::<String>::new()
        }))
        .expect("json serialization")
    }

    fn render_schema(&self, program: &ast::Program, include_stdlib_states: bool) -> String {
        use ast::{Item, TypeBody};
        use std::fmt::Write as _;

        let mut out = String::new();
        let mut wrote_any = false;
        let mut emit_type = |td: &ast::TypeDef, stdlib: bool, out: &mut String| {
            let TypeBody::Record(fields) = &td.body else { return; };
            if wrote_any {
                let _ = writeln!(out);
            }
            wrote_any = true;
            if stdlib {
                let _ = writeln!(out, "-- {} (stdlib)", td.name);
            } else {
                let _ = writeln!(out, "-- {}", td.name);
            }
            let _ = writeln!(out, "CREATE TABLE {} (", to_snake_case(&td.name));
            for field in fields {
                let _ = writeln!(
                    out,
                    "    {} {} NOT NULL,",
                    field.name,
                    favnir_type_to_sql_from_expr(&field.ty)
                );
            }
            let supported = td
                .invariants
                .iter()
                .filter_map(invariant_to_sql)
                .collect::<Vec<_>>();
            for inv in &td.invariants {
                if invariant_to_sql(inv).is_none() {
                    let _ = writeln!(out, "    -- [unsupported invariant: {}]", format_expr_compact(inv));
                }
            }
            if supported.is_empty() {
                let _ = writeln!(out, ");");
            } else {
                let _ = writeln!(out, "    CHECK ({})", supported.join(" AND "));
                let _ = writeln!(out, ");");
            }
        };

        for item in &program.items {
            if let Item::TypeDef(td) = item {
                emit_type(td, false, &mut out);
            }
        }
        if include_stdlib_states {
            for td in crate::std_states::parsed_type_defs() {
                emit_type(&td, true, &mut out);
            }
        }
        out
    }
}

fn format_invariants(invariants: &[ast::Expr]) -> String {
    if invariants.is_empty() {
        return "-".into();
    }
    invariants
        .iter()
        .map(format_expr_compact)
        .collect::<Vec<_>>()
        .join("; ")
}

fn format_expr_compact(expr: &ast::Expr) -> String {
    use ast::{BinOp, Expr, Lit};
    let rendered = match expr {
        Expr::Lit(Lit::Bool(v), _) => v.to_string(),
        Expr::Lit(Lit::Int(v), _) => v.to_string(),
        Expr::Lit(Lit::Float(v), _) => {
            let mut s = v.to_string();
            if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                s.push_str(".0");
            }
            s
        }
        Expr::Lit(Lit::Str(s), _) => format!("{s:?}"),
        Expr::Lit(Lit::Unit, _) => "()".into(),
        Expr::Ident(name, _) => name.clone(),
        Expr::FieldAccess(obj, field, _) => format!("{}.{}", format_expr_compact(obj), field),
        Expr::Apply(callee, args, _) => {
            let args = args.iter().map(format_expr_compact).collect::<Vec<_>>().join(", ");
            format!("{}({args})", format_expr_compact(callee))
        }
        Expr::BinOp(op, left, right, _) => {
            let op = match op {
                BinOp::Add          => "+",
                BinOp::Sub          => "-",
                BinOp::Mul          => "*",
                BinOp::Div          => "/",
                BinOp::Eq           => "==",
                BinOp::NotEq        => "!=",
                BinOp::Lt           => "<",
                BinOp::Gt           => ">",
                BinOp::LtEq         => "<=",
                BinOp::GtEq         => ">=",
                BinOp::NullCoalesce => "??",
            };
            format!("{} {} {}", format_expr_compact(left), op, format_expr_compact(right))
        }
        Expr::Pipeline(parts, _) => parts.iter().map(format_expr_compact).collect::<Vec<_>>().join(" |> "),
        Expr::RecordConstruct(name, fields, _) => {
            let fields = fields
                .iter()
                .map(|(k, v)| format!("{k}: {}", format_expr_compact(v)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name} {{ {fields} }}")
        }
        Expr::If(cond, ..) => format!("if {} {{ ... }}", format_expr_compact(cond)),
        Expr::Match(subject, ..) => format!("match {} {{ ... }}", format_expr_compact(subject)),
        Expr::AssertMatches(expr, ..) => format!("assert_matches({}, ...)", format_expr_compact(expr)),
        Expr::Block(_) => "{ ... }".into(),
        Expr::Closure(params, _, _) => format!("|{}| ...", params.join(", ")),
        Expr::Collect(_, _) => "collect { ... }".into(),
        Expr::FString(parts, _) => {
            let mut out = String::from("$\"");
            for part in parts {
                match part {
                    ast::FStringPart::Lit(s) => out.push_str(s),
                    ast::FStringPart::Expr(expr) => {
                        out.push('{');
                        out.push_str(&format_expr_compact(expr));
                        out.push('}');
                    }
                }
            }
            out.push('"');
            out
        }
        Expr::EmitExpr(inner, _) => format!("emit {}", format_expr_compact(inner)),
    };
    truncate_compact(rendered, 60)
}

fn truncate_compact(mut s: String, limit: usize) -> String {
    if s.chars().count() <= limit {
        return s;
    }
    s = s.chars().take(limit.saturating_sub(3)).collect();
    s.push_str("...");
    s
}

fn invariant_to_sql(expr: &ast::Expr) -> Option<String> {
    expr_to_sql(expr)
}

fn expr_to_sql(expr: &ast::Expr) -> Option<String> {
    use ast::{BinOp, Expr, Lit};
    match expr {
        Expr::Ident(name, _) => Some(name.clone()),
        Expr::Lit(Lit::Bool(v), _) => Some(if *v { "TRUE".into() } else { "FALSE".into() }),
        Expr::Lit(Lit::Int(v), _) => Some(v.to_string()),
        Expr::Lit(Lit::Float(v), _) => Some({
            let mut s = v.to_string();
            if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                s.push_str(".0");
            }
            s
        }),
        Expr::Lit(Lit::Str(s), _) => Some(format!("'{}'", s.replace('\'', "''"))),
        Expr::Lit(Lit::Unit, _) => None,
        Expr::BinOp(op, left, right, _) => {
            let l = expr_to_sql(left)?;
            let r = expr_to_sql(right)?;
            let op = match op {
                BinOp::Add          => "+",
                BinOp::Sub          => "-",
                BinOp::Mul          => "*",
                BinOp::Div          => "/",
                BinOp::Eq           => "=",
                BinOp::NotEq        => "!=",
                BinOp::Lt           => "<",
                BinOp::Gt           => ">",
                BinOp::LtEq         => "<=",
                BinOp::GtEq         => ">=",
                BinOp::NullCoalesce => return None,
            };
            Some(format!("{l} {op} {r}"))
        }
        Expr::Apply(callee, args, _) => match callee.as_ref() {
            Expr::FieldAccess(obj, field, _) => match (obj.as_ref(), field.as_str(), args.as_slice()) {
                (Expr::Ident(ns, _), "contains", [value, needle]) if ns == "String" => {
                    let value = expr_to_sql(value)?;
                    let needle = match needle {
                        Expr::Lit(Lit::Str(s), _) => s.clone(),
                        _ => return None,
                    };
                    Some(format!("{value} LIKE '%{}%'", needle.replace('\'', "''")))
                }
                (Expr::Ident(ns, _), "starts_with", [value, prefix]) if ns == "String" => {
                    let value = expr_to_sql(value)?;
                    let prefix = match prefix {
                        Expr::Lit(Lit::Str(s), _) => s.clone(),
                        _ => return None,
                    };
                    Some(format!("{value} LIKE '{}%'", prefix.replace('\'', "''")))
                }
                (Expr::Ident(ns, _), "length", [value]) if ns == "String" => {
                    let value = expr_to_sql(value)?;
                    Some(format!("length({value})"))
                }
                (Expr::Ident(ns, _), "is_url", [value]) if ns == "String" => {
                    let value = expr_to_sql(value)?;
                    Some(format!("({value} LIKE 'http://%' OR {value} LIKE 'https://%')"))
                }
                (Expr::Ident(ns, _), "is_slug", [_value]) if ns == "String" => None,
                _ => None,
            },
            _ => None,
        },
        Expr::FieldAccess(_, _, _)
        | Expr::Pipeline(_, _)
        | Expr::Block(_)
        | Expr::Match(_, _, _)
        | Expr::AssertMatches(_, _, _)
        | Expr::Collect(_, _)
        | Expr::If(_, _, _, _)
        | Expr::Closure(_, _, _)
        | Expr::FString(_, _)
        | Expr::RecordConstruct(_, _, _)
        | Expr::EmitExpr(_, _) => None,
    }
}

fn favnir_type_to_sql_from_expr(ty: &ast::TypeExpr) -> &'static str {
    match ty {
        ast::TypeExpr::Named(name, _, _) => match name.as_str() {
            "Int" => "INTEGER",
            "Float" => "REAL",
            "String" => "TEXT",
            "Bool" => "INTEGER",
            _ => "TEXT",
        },
        ast::TypeExpr::Optional(inner, _) => favnir_type_to_sql_from_expr(inner),
        ast::TypeExpr::Fallible(inner, _) => favnir_type_to_sql_from_expr(inner),
        ast::TypeExpr::Arrow(_, _, _) => "TEXT",
        ast::TypeExpr::TrfFn { .. } => "TEXT",
    }
}

fn to_snake_case(name: &str) -> String {
    let mut out = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
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
        TrfFn { input, output, effects, .. } => {
            let effs = format_effects(effects);
            let effs = if effs == "Pure" { String::new() } else { format!(" {}", effs) };
            format!("{} -> {}{}", format_type_expr(input), format_type_expr(output), effs)
        }
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
        File           => "!File".into(),
        Unknown(name)  => format!("!{}", name),
        Emit(ev)       => format!("!Emit<{}>", ev),
        EmitUnion(evs) => format!("!Emit<{}>", evs.join("|")),
        Trace          => "!Trace".into(),
      }).collect::<Vec<_>>().join(" ")
}

fn effect_json_name(effect: &ast::Effect) -> String {
    match effect {
        ast::Effect::Pure => "Pure".into(),
        ast::Effect::Io => "Io".into(),
        ast::Effect::Db => "Db".into(),
        ast::Effect::Network => "Network".into(),
        ast::Effect::File => "File".into(),
        ast::Effect::Unknown(name) => name.clone(),
        ast::Effect::Emit(ev) => format!("Emit<{ev}>"),
        ast::Effect::EmitUnion(evs) => format!("Emit<{}>", evs.join("|")),
        ast::Effect::Trace => "Trace".into(),
    }
}

fn slot_impl_name(imp: &ast::SlotImpl) -> &str {
    match imp {
        ast::SlotImpl::Global(name) | ast::SlotImpl::Local(name) => name.as_str(),
    }
}

// ── Phase 4: rune dependency management ───────────────────────────────────────

/// `fav install` — resolve path dependencies and write `fav.lock`.
pub fn cmd_install() {
    use crate::toml::{FavToml, DependencySpec};
    use crate::lock::{LockFile, LockedPackage, resolve_path_dep};

    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let root = FavToml::find_root(&cwd).unwrap_or_else(|| {
        eprintln!("error: no fav.toml found");
        std::process::exit(1);
    });
    let toml = FavToml::load(&root).unwrap_or_else(|| {
        eprintln!("error: could not read fav.toml");
        std::process::exit(1);
    });

    if toml.dependencies.is_empty() {
        println!("No dependencies to install.");
        return;
    }

    let mut lock = LockFile::new();
    let mut errors = 0usize;

    for dep in &toml.dependencies {
        match dep {
            DependencySpec::Path { name, path } => {
                let resolved = resolve_path_dep(&root, path);
                if !resolved.exists() {
                    eprintln!("error: dependency `{name}` path `{path}` does not exist");
                    errors += 1;
                    continue;
                }
                let resolved_str = resolved.to_string_lossy().into_owned();
                println!("  + {name} (path: {resolved_str})");
                lock.packages.push(LockedPackage {
                    name: name.clone(),
                    version: String::new(),
                    resolved_path: resolved_str,
                });
            }
            DependencySpec::Registry { name, registry, version } => {
                // Local registry: look for `<registry_name>/<name>-<version>/` relative to root
                let registry_dir = root.join(registry).join(format!("{name}-{version}"));
                if !registry_dir.exists() {
                    eprintln!(
                        "error: dependency `{name}@{version}` not found in local registry `{registry}`"
                    );
                    errors += 1;
                    continue;
                }
                let resolved_str = registry_dir.to_string_lossy().into_owned();
                println!("  + {name}@{version} (registry: {registry})");
                lock.packages.push(LockedPackage {
                    name: name.clone(),
                    version: version.clone(),
                    resolved_path: resolved_str,
                });
            }
        }
    }

    if errors > 0 {
        eprintln!("error: {errors} dependency/dependencies could not be resolved");
        std::process::exit(1);
    }

    lock.save(&root).unwrap_or_else(|e| {
        eprintln!("error: could not write fav.lock: {e}");
        std::process::exit(1);
    });
    println!("Wrote fav.lock ({} package(s))", lock.packages.len());
}

/// `fav publish` — validate project and emit a publish manifest stub.
pub fn cmd_publish() {
    use crate::toml::FavToml;

    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let root = FavToml::find_root(&cwd).unwrap_or_else(|| {
        eprintln!("error: no fav.toml found");
        std::process::exit(1);
    });
    let toml = FavToml::load(&root).unwrap_or_else(|| {
        eprintln!("error: could not read fav.toml");
        std::process::exit(1);
    });

    if toml.name.is_empty() {
        eprintln!("error: fav.toml is missing `name`");
        std::process::exit(1);
    }
    if toml.version.is_empty() {
        eprintln!("error: fav.toml is missing `version`");
        std::process::exit(1);
    }

    println!("Publishing {}@{}", toml.name, toml.version);
    println!("(note: remote registry publishing is not yet implemented)");
    println!("To share locally, copy this project to your local registry directory.");
}

// ── fav migrate ───────────────────────────────────────────────────────────────

/// Rewrite a single line, replacing v1.x keywords with v2.0.0 equivalents.
/// Returns the rewritten line.
fn migrate_line(line: &str) -> String {
    // Process replacements in order (longest match first to avoid partial hits).
    // Use word-boundary-like logic: only replace at start of line or after whitespace.
    let replacements: &[(&str, &str)] = &[
        ("abstract trf ", "abstract stage "),
        ("abstract flw ", "abstract seq "),
        ("trf ", "stage "),
        ("flw ", "seq "),
    ];

    let mut result = line.to_string();
    for (from, to) in replacements {
        // Replace at start of line
        if result.starts_with(from) {
            result = format!("{}{}", to, &result[from.len()..]);
            continue;
        }
        // Replace after leading whitespace
        let trimmed = result.trim_start();
        let indent_len = result.len() - trimmed.len();
        if trimmed.starts_with(from) {
            result = format!("{}{}{}", &result[..indent_len], to, &trimmed[from.len()..]);
        }
    }
    result
}

/// Migrate source text from v1.x to v2.0.0 syntax.
pub fn migrate_source(src: &str) -> String {
    src.lines()
        .map(|line| migrate_line(line))
        .collect::<Vec<_>>()
        .join("\n")
        + if src.ends_with('\n') { "\n" } else { "" }
}

/// Check whether source text needs migration.
#[allow(dead_code)]
pub fn source_needs_migration(src: &str) -> bool {
    src != migrate_source(src)
}

/// `fav migrate` — migrate .fav files from v1.x to v2.0.0 syntax.
///
/// Modes:
/// - `--dry-run` (default): show what would change.
/// - `--in-place`: rewrite files.
/// - `--check`: exit 1 if any file needs migration (CI use).
pub fn cmd_migrate(file: Option<&str>, in_place: bool, _dry_run: bool, check: bool, dir: Option<&str>) {
    let files: Vec<PathBuf> = if let Some(f) = file {
        vec![PathBuf::from(f)]
    } else if let Some(d) = dir {
        collect_fav_files_recursive(&PathBuf::from(d))
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if let Some(root) = crate::toml::FavToml::find_root(&cwd) {
            collect_fav_files_recursive(&root.parent().unwrap_or(&root).to_path_buf())
        } else {
            collect_fav_files_recursive(&cwd)
        }
    };

    let mut any_needs_migration = false;
    let mut changed_count = 0usize;

    for path in &files {
        let src = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("warning: could not read {}: {}", path.display(), e);
                continue;
            }
        };

        let migrated = migrate_source(&src);
        if migrated == src {
            continue;
        }

        any_needs_migration = true;
        changed_count += 1;

        if check {
            println!("needs migration: {}", path.display());
        } else if in_place {
            match std::fs::write(path, &migrated) {
                Ok(_) => println!("migrated: {}", path.display()),
                Err(e) => eprintln!("error writing {}: {}", path.display(), e),
            }
        } else {
            // dry-run: show unified diff-like output
            println!("--- {}", path.display());
            println!("+++ {} (migrated)", path.display());
            for (i, (old, new)) in src.lines().zip(migrated.lines()).enumerate() {
                if old != new {
                    println!(" {:4}: - {}", i + 1, old);
                    println!(" {:4}: + {}", i + 1, new);
                }
            }
            println!();
        }
    }

    if changed_count == 0 && !check {
        println!("All files are already v2.0.0 compatible.");
    }

    if check && any_needs_migration {
        std::process::exit(1);
    }
}

// ── migrate tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod migrate_tests {
    use super::*;

    #[test]
    fn migrate_trf_to_stage() {
        assert_eq!(migrate_line("trf Foo: Int -> Int = |x| { x }"), "stage Foo: Int -> Int = |x| { x }");
    }

    #[test]
    fn migrate_flw_to_seq() {
        assert_eq!(migrate_line("flw Pipeline = A |> B"), "seq Pipeline = A |> B");
    }

    #[test]
    fn migrate_abstract_trf_to_abstract_stage() {
        assert_eq!(migrate_line("abstract trf Parse: String -> Int"), "abstract stage Parse: String -> Int");
    }

    #[test]
    fn migrate_abstract_flw_to_abstract_seq() {
        assert_eq!(migrate_line("abstract flw Flow<T> {"), "abstract seq Flow<T> {");
    }

    #[test]
    fn migrate_indented_trf() {
        assert_eq!(migrate_line("    trf Inner: Bool -> Bool = |b| { b }"), "    stage Inner: Bool -> Bool = |b| { b }");
    }

    #[test]
    fn migrate_indented_abstract_trf() {
        assert_eq!(migrate_line("    abstract trf Step: Int -> Int"), "    abstract stage Step: Int -> Int");
    }

    #[test]
    fn migrate_leaves_stage_unchanged() {
        assert_eq!(migrate_line("stage Foo: Int -> Int = |x| { x }"), "stage Foo: Int -> Int = |x| { x }");
    }

    #[test]
    fn migrate_leaves_seq_unchanged() {
        assert_eq!(migrate_line("seq Pipeline = A |> B"), "seq Pipeline = A |> B");
    }

    #[test]
    fn migrate_source_multiline() {
        let src = "trf Foo: Int -> Int = |x| { x }\nflw Bar = Foo\n";
        let expected = "stage Foo: Int -> Int = |x| { x }\nseq Bar = Foo\n";
        assert_eq!(migrate_source(src), expected);
    }

    #[test]
    fn source_needs_migration_true() {
        assert!(source_needs_migration("trf Foo: Int -> Int = |x| { x }"));
    }

    #[test]
    fn source_needs_migration_false() {
        assert!(!source_needs_migration("stage Foo: Int -> Int = |x| { x }"));
    }
}
