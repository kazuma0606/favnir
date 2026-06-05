/// compiler.fav ローダー＋ランナー (v8.3.0)
///
/// `compiler.fav` を Rust パイプラインで一度だけコンパイルし OnceLock にキャッシュ。
/// `compile_bytes(path)` で任意の .fav ファイルを Favnir 実装でコンパイルし、
/// FVC バイトコード (`Vec<u8>`) を返す。
///
/// 使用側:
///   let bytes = compiler_fav_runner::compile_file_to_bytes("hello.fav")?;
///   let artifact = FvcArtifact::from_bytes(&bytes)?;
///   VM::run(&artifact, fn_idx, vec![]);
use std::sync::{Arc, OnceLock};

use crate::backend::artifact::FvcArtifact;
use crate::backend::codegen::codegen_program;
use crate::backend::vm::{VM, VMError};
use crate::frontend::parser::Parser;
use crate::middle::compiler::compile_program;
use crate::value::Value;

// ── artifact cache ────────────────────────────────────────────────────────────

static COMPILER_FAV_ARTIFACT: OnceLock<Arc<FvcArtifact>> = OnceLock::new();

static COMPILER_FAV_SRC: &str = include_str!("../self/compiler.fav");

fn get_compiler_fav_artifact() -> Arc<FvcArtifact> {
    COMPILER_FAV_ARTIFACT
        .get_or_init(|| {
            let prog = Parser::parse_str(COMPILER_FAV_SRC, "compiler.fav")
                .expect("compiler_fav_runner: compiler.fav parse error");
            let ir = compile_program(&prog);
            Arc::new(codegen_program(&ir))
        })
        .clone()
}

// ── public API ────────────────────────────────────────────────────────────────

/// `compiler.fav` の `compile_bytes(path)` を呼び出して FVC バイトコードを返す。
///
/// * `Ok(bytes)` — 成功。`FvcArtifact::from_bytes(&bytes)` で復元可能。
/// * `Err(msg)`  — 字句/構文/コンパイルエラー。
pub fn compile_file_to_bytes(path: &str) -> Result<Vec<u8>, String> {
    let artifact = get_compiler_fav_artifact();
    let fn_idx = artifact
        .fn_idx_by_name("compile_bytes")
        .ok_or_else(|| "compiler_fav_runner: compile_bytes not found in compiler.fav".to_string())?;

    let result = VM::run(&artifact, fn_idx, vec![Value::Str(path.to_string())])
        .map_err(|e: VMError| format!("compiler.fav VM error: {}", e.message))?;

    match result {
        Value::Variant(ref tag, Some(ref payload)) if tag == "ok" => {
            let ints = match payload.as_ref() {
                Value::List(items) => items,
                _ => return Err("compiler_fav_runner: compile_bytes returned non-list Ok payload".to_string()),
            };
            let bytes: Result<Vec<u8>, String> = ints
                .iter()
                .map(|v| match v {
                    Value::Int(n) => {
                        if *n >= 0 && *n <= 255 {
                            Ok(*n as u8)
                        } else {
                            Err(format!("compiler_fav_runner: byte value {} out of range", n))
                        }
                    }
                    _ => Err("compiler_fav_runner: non-Int in byte list".to_string()),
                })
                .collect();
            bytes
        }
        Value::Variant(ref tag, ref payload) if tag == "err" => {
            let msg = match payload {
                Some(p) => match p.as_ref() {
                    Value::Str(s) => s.clone(),
                    _ => format!("{:?}", p),
                },
                None => "unknown compile error".to_string(),
            };
            Err(msg)
        }
        _ => Err("compiler_fav_runner: unexpected result from compile_bytes".to_string()),
    }
}

// ── rune-import-aware compilation ─────────────────────────────────────────────

/// Recursively collect source texts for `path` and all its rune dependencies
/// (standalone mode: `rune_modules/<name>/` relative to the source file's dir).
///
/// Files are added in dependency-first order (rune deps before the importing file).
/// `import rune "..."` lines and `namespace ...` lines are stripped so the
/// concatenated result can be parsed as a single flat program.
fn collect_merged_sources(
    path: &str,
    visited: &mut std::collections::HashSet<String>,
    out: &mut Vec<String>,
) -> Result<(), String> {
    let canon = std::path::Path::new(path)
        .canonicalize()
        .map_err(|e| format!("cannot resolve path `{}`: {}", path, e))?;
    let canon_str = canon.to_string_lossy().to_string();
    if visited.contains(&canon_str) {
        return Ok(());
    }
    visited.insert(canon_str);

    let src = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read `{}`: {}", path, e))?;

    let program = Parser::parse_str(&src, path).map_err(|e| e.to_string())?;

    let source_dir = std::path::Path::new(path)
        .parent()
        .unwrap_or(std::path::Path::new("."));

    // Recurse into rune dependencies first (dependency order)
    for item in &program.items {
        if let crate::ast::Item::ImportDecl {
            path: rune_name,
            is_rune: true,
            ..
        } = item
        {
            let rune_dir = source_dir.join("rune_modules").join(rune_name.as_str());
            if rune_dir.is_dir() {
                let entry = crate::toml::rune_entry_file(&rune_dir, rune_name);
                let entry_str = entry.to_string_lossy().to_string();
                collect_merged_sources(&entry_str, visited, out)?;
            }
        }
    }

    // Strip `import rune` and `namespace` lines before appending
    let stripped: String = src
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.starts_with("import rune") && !t.starts_with("namespace ")
        })
        .collect::<Vec<_>>()
        .join("\n");
    out.push(stripped);
    Ok(())
}

/// Compile a pre-merged source string via `compiler.fav`'s `compile_bytes_from_src`.
/// Returns raw FVC bytecode.
pub fn compile_src_str_to_bytes(merged: &str) -> Result<Vec<u8>, String> {
    let artifact = get_compiler_fav_artifact();
    let fn_idx = artifact
        .fn_idx_by_name("compile_bytes_from_src")
        .ok_or_else(|| {
            "compiler_fav_runner: compile_bytes_from_src not found in compiler.fav".to_string()
        })?;

    let result = VM::run(&artifact, fn_idx, vec![Value::Str(merged.to_string())])
        .map_err(|e: VMError| format!("compiler.fav VM error: {}", e.message))?;

    match result {
        Value::Variant(ref tag, Some(ref payload)) if tag == "ok" => {
            let ints = match payload.as_ref() {
                Value::List(items) => items,
                _ => {
                    return Err("compiler_fav_runner: compile_bytes_from_src returned non-list Ok payload".to_string())
                }
            };
            let bytes: Result<Vec<u8>, String> = ints
                .iter()
                .map(|v| match v {
                    Value::Int(n) => {
                        if *n >= 0 && *n <= 255 {
                            Ok(*n as u8)
                        } else {
                            Err(format!("compiler_fav_runner: byte value {} out of range", n))
                        }
                    }
                    _ => Err("compiler_fav_runner: non-Int in byte list".to_string()),
                })
                .collect();
            bytes
        }
        Value::Variant(ref tag, ref payload) if tag == "err" => {
            let msg = match payload {
                Some(p) => match p.as_ref() {
                    Value::Str(s) => s.clone(),
                    _ => format!("{:?}", p),
                },
                None => "unknown compile error".to_string(),
            };
            Err(msg)
        }
        _ => Err(
            "compiler_fav_runner: unexpected result from compile_bytes_from_src".to_string(),
        ),
    }
}

/// Rune-import-aware variant of `compile_file_to_bytes`.
///
/// Collects all rune dependency sources, merges them into one flat source string,
/// then calls `compile_src_str_to_bytes` to compile the result.
///
/// * `Ok(bytes)` — success. `FvcArtifact::from_bytes(&bytes)` で復元可能。
/// * `Err(msg)`  — file I/O or compile error.
pub fn compile_file_to_bytes_rune(path: &str) -> Result<Vec<u8>, String> {
    let mut visited = std::collections::HashSet::new();
    let mut sources: Vec<String> = Vec::new();
    collect_merged_sources(path, &mut visited, &mut sources)?;
    let merged = sources.join("\n");
    compile_src_str_to_bytes(&merged)
}

/// Call `compiler.fav`'s `fmt_source(src)` and return the formatted string.
///
/// * `Ok(formatted)` — success.
/// * `Err(msg)`      — lex/parse error in the source.
pub fn fmt_source_str(src: &str) -> Result<String, String> {
    let artifact = get_compiler_fav_artifact();
    let fn_idx = artifact
        .fn_idx_by_name("fmt_source")
        .ok_or_else(|| "compiler_fav_runner: fmt_source not found in compiler.fav".to_string())?;

    let result = VM::run(&artifact, fn_idx, vec![Value::Str(src.to_string())])
        .map_err(|e: VMError| format!("compiler.fav VM error: {}", e.message))?;

    match result {
        Value::Variant(ref tag, Some(ref payload)) if tag == "ok" => match payload.as_ref() {
            Value::Str(s) => Ok(s.clone()),
            _ => Err("compiler_fav_runner: fmt_source returned non-string Ok payload".to_string()),
        },
        Value::Variant(ref tag, ref payload) if tag == "err" => {
            let msg = match payload {
                Some(p) => match p.as_ref() {
                    Value::Str(s) => s.clone(),
                    _ => format!("{:?}", p),
                },
                None => "unknown fmt error".to_string(),
            };
            Err(msg)
        }
        _ => Err("compiler_fav_runner: unexpected result from fmt_source".to_string()),
    }
}

/// Call `compiler.fav`'s `lint_source(src)` and return the warning lines string.
///
/// * `Ok(warnings)` — success; empty string means no warnings.
/// * `Err(msg)`     — lex/parse error in the source.
pub fn lint_source_str(src: &str) -> Result<String, String> {
    let artifact = get_compiler_fav_artifact();
    let fn_idx = artifact
        .fn_idx_by_name("lint_source")
        .ok_or_else(|| "compiler_fav_runner: lint_source not found in compiler.fav".to_string())?;

    let result = VM::run(&artifact, fn_idx, vec![Value::Str(src.to_string())])
        .map_err(|e: VMError| format!("compiler.fav VM error: {}", e.message))?;

    match result {
        Value::Variant(ref tag, Some(ref payload)) if tag == "ok" => match payload.as_ref() {
            Value::Str(s) => Ok(s.clone()),
            _ => Err("compiler_fav_runner: lint_source returned non-string Ok payload".to_string()),
        },
        Value::Variant(ref tag, ref payload) if tag == "err" => {
            let msg = match payload {
                Some(p) => match p.as_ref() {
                    Value::Str(s) => s.clone(),
                    _ => format!("{:?}", p),
                },
                None => "unknown lint error".to_string(),
            };
            Err(msg)
        }
        _ => Err("compiler_fav_runner: unexpected result from lint_source".to_string()),
    }
}

/// Call `compiler.fav`'s `compile_source_profiled(src)` and return instrumented FVC bytecode.
///
/// * `Ok(bytes)` — success; bytecode has stage calls wrapped with `Env.profile_timed_raw`.
/// * `Err(msg)`  — lex/parse/compile error.
pub fn compile_profiled_str(src: &str) -> Result<Vec<u8>, String> {
    let artifact = get_compiler_fav_artifact();
    let fn_idx = artifact
        .fn_idx_by_name("compile_source_profiled")
        .ok_or_else(|| "compiler_fav_runner: compile_source_profiled not found in compiler.fav".to_string())?;

    let result = VM::run(&artifact, fn_idx, vec![Value::Str(src.to_string())])
        .map_err(|e: VMError| format!("compiler.fav VM error: {}", e.message))?;

    match result {
        Value::Variant(ref tag, Some(ref payload)) if tag == "ok" => {
            let ints = match payload.as_ref() {
                Value::List(items) => items,
                _ => return Err("compiler_fav_runner: compile_source_profiled returned non-list Ok payload".to_string()),
            };
            let bytes: Result<Vec<u8>, String> = ints
                .iter()
                .map(|v| match v {
                    Value::Int(n) => {
                        if *n >= 0 && *n <= 255 {
                            Ok(*n as u8)
                        } else {
                            Err(format!("compiler_fav_runner: byte value {} out of range", n))
                        }
                    }
                    _ => Err("compiler_fav_runner: non-Int in byte list".to_string()),
                })
                .collect();
            bytes
        }
        Value::Variant(ref tag, ref payload) if tag == "err" => {
            let msg = match payload {
                Some(p) => match p.as_ref() {
                    Value::Str(s) => s.clone(),
                    _ => format!("{:?}", p),
                },
                None => "unknown compile error".to_string(),
            };
            Err(msg)
        }
        _ => Err("compiler_fav_runner: unexpected result from compile_source_profiled".to_string()),
    }
}

/// Call `compiler.fav`'s `doc_source(src)` and return the Markdown string.
///
/// * `Ok(markdown)` — success.
/// * `Err(msg)`     — lex/parse error in the source.
pub fn doc_source_str(src: &str) -> Result<String, String> {
    let artifact = get_compiler_fav_artifact();
    let fn_idx = artifact
        .fn_idx_by_name("doc_source")
        .ok_or_else(|| "compiler_fav_runner: doc_source not found in compiler.fav".to_string())?;

    let result = VM::run(&artifact, fn_idx, vec![Value::Str(src.to_string())])
        .map_err(|e: VMError| format!("compiler.fav VM error: {}", e.message))?;

    match result {
        Value::Variant(ref tag, Some(ref payload)) if tag == "ok" => match payload.as_ref() {
            Value::Str(s) => Ok(s.clone()),
            _ => Err("compiler_fav_runner: doc_source returned non-string Ok payload".to_string()),
        },
        Value::Variant(ref tag, ref payload) if tag == "err" => {
            let msg = match payload {
                Some(p) => match p.as_ref() {
                    Value::Str(s) => s.clone(),
                    _ => format!("{:?}", p),
                },
                None => "unknown doc error".to_string(),
            };
            Err(msg)
        }
        _ => Err("compiler_fav_runner: unexpected result from doc_source".to_string()),
    }
}

// ── project-mode compilation ───────────────────────────────────────────────────

/// Recursively collect source texts for a fav.toml project file and all its
/// dependencies (both local modules and rune imports).
///
/// * `import "name"` → `src/<name>.fav` (resolved via toml.src_dir)
/// * `import rune "name"` → delegated to `collect_merged_sources`
///
/// `import` and `namespace` lines are stripped so the concatenated result
/// can be parsed as a single flat program.
fn collect_project_sources(
    path: &str,
    root: &std::path::Path,
    toml: &crate::toml::FavToml,
    visited: &mut std::collections::HashSet<String>,
    out: &mut Vec<String>,
) -> Result<(), String> {
    let canon = std::path::Path::new(path)
        .canonicalize()
        .map_err(|e| format!("cannot resolve path `{}`: {}", path, e))?;
    let canon_str = canon.to_string_lossy().to_string();
    if visited.contains(&canon_str) {
        return Ok(());
    }
    visited.insert(canon_str);

    let src = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read `{}`: {}", path, e))?;

    let program = Parser::parse_str(&src, path).map_err(|e| e.to_string())?;

    // Recurse into dependencies first (dependency order)
    for item in &program.items {
        match item {
            crate::ast::Item::ImportDecl { path: name, is_rune: false, .. } => {
                // Local module: import "name" → src/<name>.fav
                let dep = toml.src_dir(root).join(format!("{}.fav", name));
                let dep_str = dep.to_string_lossy().to_string();
                collect_project_sources(&dep_str, root, toml, visited, out)?;
            }
            crate::ast::Item::ImportDecl { path: name, is_rune: true, .. } => {
                // Rune module: import rune "name" → rune_modules/<name>/
                let rune_dir = root.join("rune_modules").join(name.as_str());
                if rune_dir.is_dir() {
                    let entry = crate::toml::rune_entry_file(&rune_dir, name);
                    let entry_str = entry.to_string_lossy().to_string();
                    collect_merged_sources(&entry_str, visited, out)?;
                }
            }
            _ => {}
        }
    }

    // Strip import and namespace lines before appending
    let stripped: String = src
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.starts_with("import ") && !t.starts_with("namespace ")
        })
        .collect::<Vec<_>>()
        .join("\n");
    out.push(stripped);
    Ok(())
}

/// Collect and merge all project sources into a single source string.
///
/// Useful when callers need the merged source before compilation (e.g., for type-checking).
pub fn collect_project_merged(
    entry: &str,
    root: &std::path::Path,
    toml: &crate::toml::FavToml,
) -> Result<String, String> {
    let mut visited = std::collections::HashSet::new();
    let mut sources: Vec<String> = Vec::new();
    collect_project_sources(entry, root, toml, &mut visited, &mut sources)?;
    Ok(sources.join("\n"))
}

/// Compile a fav.toml project to FVC bytecode.
///
/// Recursively collects all sources starting from `entry`, merges them into
/// a single flat program, then compiles via `compiler.fav`.
///
/// * `Ok(bytes)` — success.
/// * `Err(msg)`  — file I/O or compile error.
pub fn compile_project_to_bytes(
    entry: &str,
    root: &std::path::Path,
    toml: &crate::toml::FavToml,
) -> Result<Vec<u8>, String> {
    let merged = collect_project_merged(entry, root, toml)?;
    compile_src_str_to_bytes(&merged)
}
