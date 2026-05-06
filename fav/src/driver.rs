use crate::ast;
use crate::backend;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::{Arc, Mutex};
use crate::backend::artifact::FvcArtifact;
use crate::backend::codegen::{codegen_program, Opcode};
use crate::middle::compiler::compile_program;
use crate::backend::wasm_codegen::wasm_codegen_program;
use crate::backend::wasm_exec::{wasm_exec_info, wasm_exec_main};
use crate::frontend::parser::Parser;
use crate::middle::checker::Checker;
use crate::value::Value;
use crate::toml::FavToml;
use crate::middle::resolver::Resolver;
use crate::backend::vm::VM;

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
    (source, errors, checker.warnings)
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

    let errors = if let Some((ref toml, ref root)) = proj {
        let r = make_resolver(Some(toml.clone()), Some(root.clone()));
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
        ast::Program { namespace: None, uses: vec![], items }
    } else {
        program
    };

    (merged, path)
}

// ── fav run ───────────────────────────────────────────────────────────────────

pub fn cmd_run(file: Option<&str>, db_url: Option<&str>) {
    let (run_program, _) = load_and_check_program(file);
    let artifact = build_artifact(&run_program);

    exec_artifact_main(&artifact, db_url).unwrap_or_else(|message| {
        eprintln!("{message}");
        process::exit(1);
    });
}

pub fn cmd_build(file: Option<&str>, out: Option<&str>, target: Option<&str>) {
    let (program, path) = load_and_check_program(file);
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
            if errors.is_empty() {
                println!("{}: ok", path_str);
            } else {
                for e in &errors { eprintln!("{}", format_diagnostic(&source, e)); }
                total_errors += errors.len();
            }
            if !no_warn {
                for w in &checker.warnings {
                    eprintln!("{}", format_warning(&source, w));
                }
                total_warnings += checker.warnings.len();
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

pub fn cmd_test(file: Option<&str>, filter: Option<&str>, fail_fast: bool) {
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
    let mut tests_to_run: Vec<(String, String, ast::Program)> = Vec::new();
    for (path, prog) in programs {
        for item in &prog.items {
            if let ast::Item::TestDef(td) = item {
                if let Some(f) = filter {
                    if !td.name.contains(f) { continue; }
                }
                tests_to_run.push((path.clone(), td.name.clone(), prog.clone()));
            }
        }
    }

    let total = tests_to_run.len();
    if total == 0 {
        println!("no tests found");
        return;
    }

    println!("running {} test{}", total, if total == 1 { "" } else { "s" });
    println!();

    let mut passed = 0usize;
    let mut failures: Vec<(String, String, String)> = Vec::new();

    for (path, test_name, prog) in &tests_to_run {
        let fn_name = format!("$test:{}", test_name);
        let artifact = build_artifact(prog);
        let fn_idx = match artifact.fn_idx_by_name(&fn_name) {
            Some(i) => i,
            None => {
                println!("test {:?} ... FAILED", test_name);
                failures.push((path.clone(), test_name.clone(), "test function not found in artifact".into()));
                if fail_fast { break; }
                continue;
            }
        };
        match VM::run(&artifact, fn_idx, vec![]) {
            Ok(_) => {
                println!("test {:?} ... ok", test_name);
                passed += 1;
            }
            Err(e) => {
                println!("test {:?} ... FAILED", test_name);
                failures.push((path.clone(), test_name.clone(), e.message.clone()));
                if fail_fast { break; }
            }
        }
    }

    println!();

    if !failures.is_empty() {
        println!("---- failures ----");
        for (path, name, err) in &failures {
            println!();
            println!("test {:?}  ({})", name, path);
            println!("  {}", err);
        }
        println!();
    }

    let failed = failures.len();
    println!("test result: {}. {} passed; {} failed",
        if failed == 0 { "ok" } else { "FAILED" },
        passed,
        failed,
    );

    if failed > 0 {
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        artifact_info_string,
        build_artifact, build_wasm_artifact, exec_artifact_main, exec_artifact_main_with_emits,
        exec_wasm_bytes, load_and_check_program, read_artifact_from_path, read_wasm_from_path,
        render_warnings, check_single_file, write_artifact_to_path, write_wasm_to_path,
    };
    use crate::frontend::parser::Parser;
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
trf Double: Int -> Int = |x| { x + x }

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
trf Inc: Int -> Int = |x| { x + 1 }
flw Bump = Inc |> Inc

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
        assert!(info.contains("g#0 main [fn#0] => fn#0 main @L"));
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
trf TraceOnce: String -> String !Trace = |x| {
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
trf Double: Int -> Int = |x| { x + x }

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
trf TraceOnce: String -> String !Trace = |x| {
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

pub fn cmd_explain(file: Option<&str>) {
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
        ExplainPrinter::new().print(&program, ir.as_ref());
    }
}

struct ExplainPrinter;

impl ExplainPrinter {
    fn new() -> Self { ExplainPrinter }

    fn print(&self, program: &ast::Program, ir: Option<&crate::middle::ir::IRProgram>) {
        use ast::*;
        use crate::middle::ir::collect_deps;

        let col_vis   = 10usize;
        let col_name  = 26usize;
        let col_type  = 36usize;
        let col_eff   = 20usize;

        println!("{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {}",
            "VIS", "NAME", "TYPE", "EFFECTS", "DEPS");
        println!("{}", "-".repeat(col_vis + col_name + col_type + col_eff + 32));

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
                    };
                    let vis = format_visibility(&td.visibility);
                    println!("{:<col_vis$} {:<col_name$} type ({:<col_type$} {:<col_eff$} -",
                        vis, td.name,
                        format!("{})", kind),
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
                    println!("{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {}",
                        vis, format!("fn {}", fd.name), sig, effs, deps);
                }
                Item::TrfDef(td) => {
                    let sig = format!("{} -> {}",
                        format_type_expr(&td.input_ty),
                        format_type_expr(&td.output_ty));
                    let effs = format_effects(&td.effects);
                    let vis = format_visibility(&td.visibility);
                    let deps = deps_map.get(&td.name).map(|s| s.as_str()).unwrap_or("-");
                    println!("{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {}",
                        vis, format!("trf {}", td.name), sig, effs, deps);
                }
                Item::FlwDef(fd) => {
                    println!("{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} -",
                        "", format!("flw {}", fd.name), fd.steps.join(" |> "), "-");
                }
                Item::InterfaceDecl(id) => {
                    let kind = if let Some(sup) = &id.super_interface {
                        format!("interface : {}", sup)
                    } else {
                        "interface".to_string()
                    };
                    println!("{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} -",
                        format_visibility(&id.visibility),
                        format!("interface {}", id.name),
                        kind,
                        "-");
                }
                Item::InterfaceImplDecl(id) => {
                    println!("{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} -",
                        "",
                        format!("impl {}", id.interface_names.join(", ")),
                        id.type_name.clone(),
                        "-");
                }
                Item::CapDef(cd) => {
                    println!("{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} -",
                        format_visibility(&cd.visibility),
                        format!("cap {}", cd.name),
                        format!("<{}>", cd.type_params.join(", ")),
                        "-");
                }
                Item::ImplDef(id) => {
                    let args: Vec<String> = id.type_args.iter()
                        .map(|a| format_type_expr(a))
                        .collect();
                    println!("{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} -",
                        "", format!("impl {}", id.cap_name), format!("<{}>", args.join(", ")), "-");
                }
                Item::TestDef(td) => {
                    let deps = deps_map.get(&format!("$test:{}", td.name))
                        .map(|s| s.as_str()).unwrap_or("-");
                    println!("{:<col_vis$} {:<col_name$} {:<col_type$} {:<col_eff$} {}",
                        "", format!("test {:?}", td.name), "() -> Unit", "-", deps);
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
        File           => "!File".into(),
        Emit(ev)       => format!("!Emit<{}>", ev),
        EmitUnion(evs) => format!("!Emit<{}>", evs.join("|")),
        Trace          => "!Trace".into(),
    }).collect::<Vec<_>>().join(" ")
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
