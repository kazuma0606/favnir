/// checker.fav ローダー＋ランナー
///
/// `checker.fav` をコンパイル済みアーティファクトとしてキャッシュし、
/// `check(prog: Program) -> Result<String, String>` を VM 実行する。
use std::sync::{Arc, OnceLock};

use crate::backend::artifact::FvcArtifact;
use crate::backend::codegen::codegen_program;
use crate::backend::vm::{VM, VMError};
use crate::frontend::parser::Parser;
use crate::middle::checker::TypeError;
use crate::frontend::lexer::Span;
use crate::middle::compiler::compile_program;
use crate::value::Value;

// ── artifact cache ────────────────────────────────────────────────────────────

static CHECKER_FAV_ARTIFACT: OnceLock<Arc<FvcArtifact>> = OnceLock::new();

static CHECKER_FAV_SRC: &str = include_str!("../self/checker.fav");

fn get_checker_fav_artifact() -> Arc<FvcArtifact> {
    CHECKER_FAV_ARTIFACT
        .get_or_init(|| {
            let prog = Parser::parse_str(CHECKER_FAV_SRC, "checker.fav")
                .expect("checker_fav_runner: checker.fav parse error");
            let ir = compile_program(&prog);
            Arc::new(codegen_program(&ir))
        })
        .clone()
}

// ── public API ────────────────────────────────────────────────────────────────

/// checker.fav の `check` 関数を実行する。
///
/// * `Ok(())` — 型エラーなし
/// * `Err(msgs)` — `"E0xxx: message"` 形式の行リスト
pub fn run_checker_fav(prog_vm: Value) -> Result<(), Vec<String>> {
    let artifact = get_checker_fav_artifact();
    let check_idx = artifact
        .fn_idx_by_name("check")
        .expect("checker_fav_runner: checker.fav must export `check`");

    let result = VM::run(&artifact, check_idx, vec![prog_vm]).map_err(|e: VMError| {
        vec![format!("checker.fav VM error: {}", e.message)]
    })?;

    match result {
        Value::Variant(ref tag, _) if tag == "ok" => Ok(()),
        Value::Variant(ref tag, Some(ref payload)) if tag == "err" => {
            let text = match payload.as_ref() {
                Value::Str(s) => s.clone(),
                _ => format!("{:?}", payload),
            };
            let lines: Vec<String> = text
                .lines()
                .map(|l| l.to_string())
                .filter(|l| !l.is_empty())
                .collect();
            Err(if lines.is_empty() {
                vec!["type error (no message)".to_string()]
            } else {
                lines
            })
        }
        _ => Err(vec!["unexpected checker.fav result".to_string()]),
    }
}

/// checker.fav の `check` 関数を実行し、Ok ペイロード（警告文字列を含む）を返す。
///
/// * `Ok(payload)` — 型エラーなし; `"ok"` または `"ok\nW006: ..."` 形式
/// * `Err(msgs)` — 型エラーあり
pub fn run_checker_fav_full(prog_vm: Value) -> Result<String, Vec<String>> {
    let artifact = get_checker_fav_artifact();
    let check_idx = artifact
        .fn_idx_by_name("check")
        .expect("checker_fav_runner: checker.fav must export `check`");

    let result = VM::run(&artifact, check_idx, vec![prog_vm]).map_err(|e: VMError| {
        vec![format!("checker.fav VM error: {}", e.message)]
    })?;

    match result {
        Value::Variant(ref tag, Some(ref payload)) if tag == "ok" => {
            let text = match payload.as_ref() {
                Value::Str(s) => s.clone(),
                _ => String::new(),
            };
            Ok(text)
        }
        Value::Variant(ref tag, Some(ref payload)) if tag == "err" => {
            let text = match payload.as_ref() {
                Value::Str(s) => s.clone(),
                _ => format!("{:?}", payload),
            };
            let lines: Vec<String> = text
                .lines()
                .map(|l| l.to_string())
                .filter(|l| !l.is_empty())
                .collect();
            Err(if lines.is_empty() {
                vec!["type error (no message)".to_string()]
            } else {
                lines
            })
        }
        _ => Err(vec!["unexpected checker.fav result".to_string()]),
    }
}

/// `"E0xxx: message"` 行リスト → `Vec<TypeError>`
pub fn msgs_to_type_errors(msgs: Vec<String>) -> Vec<TypeError> {
    let empty_span = Span {
        file: String::new(),
        start: 0,
        end: 0,
        line: 0,
        col: 0,
    };
    msgs.into_iter()
        .map(|msg| {
            // code は "E0xxx" 部分だけ抽出して static str に leak する
            let code: &'static str = if msg.len() >= 5 && msg.starts_with('E') {
                let candidate = &msg[..5];
                Box::leak(candidate.to_string().into_boxed_str())
            } else {
                "E9999"
            };
            TypeError {
                code,
                message: msg,
                span: empty_span.clone(),
            }
        })
        .collect()
}
