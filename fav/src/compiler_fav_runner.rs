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

fn get_compiler_fav_artifact() -> Arc<FvcArtifact> {
    COMPILER_FAV_ARTIFACT
        .get_or_init(|| {
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            let path = std::path::Path::new(manifest_dir)
                .join("self")
                .join("compiler.fav");
            let src = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!(
                    "compiler_fav_runner: cannot read {}: {}",
                    path.display(),
                    e
                )
            });
            let prog = Parser::parse_str(&src, "compiler.fav")
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
