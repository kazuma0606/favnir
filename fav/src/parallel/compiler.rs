//! v19.4.0: 並列コンパイルオーケストレーター

use rayon::prelude::*;
use crate::middle::ir::{IRGlobal, IRGlobalKind, IRProgram};

/// `(file_name, source_code)` のリストを並列コンパイルして `IRProgram` を返す。
///
/// - `jobs = 0`: CPU コア数に自動設定
/// - `jobs > 0`: 指定スレッド数を使用
pub fn compile_parallel(
    sources: Vec<(String, String)>,
    jobs: usize,
) -> Result<IRProgram, String> {
    let num_threads = if jobs == 0 { rayon::current_num_threads() } else { jobs };

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .map_err(|e| format!("threadpool error: {e}"))?;

    pool.install(|| {
        // フェーズ 1: 並列 AST パース
        let parsed: Vec<(String, crate::ast::Program)> = sources
            .par_iter()
            .map(|(name, src)| {
                let prog = crate::frontend::parser::Parser::parse_str(src, name)
                    .map_err(|e| format!("parse error in {name}: {e}"))?;
                Ok((name.clone(), prog))
            })
            .collect::<Result<Vec<_>, String>>()?;

        // フェーズ 3: 並列 IR 生成（compile_program が型チェックを内包）
        let programs: Vec<IRProgram> = parsed
            .par_iter()
            .map(|(_, prog)| crate::middle::compiler::compile_program(prog))
            .collect();

        // フェーズ 4: IR マージ
        Ok(merge_ir_programs(programs))
    })
}

/// 複数の `IRProgram` を 1 つにマージする。
/// - `globals`: 重複名をスキップして連結（fn インデックスをオフセット付きで更新）
/// - `fns`: オフセット付きで連結
/// - `type_metas`: HashMap を merge
fn merge_ir_programs(programs: Vec<IRProgram>) -> IRProgram {
    use std::collections::HashMap;

    let mut globals: Vec<IRGlobal> = Vec::new();
    let mut fns = Vec::new();
    let mut type_metas = HashMap::new();

    for prog in programs {
        let fn_offset = fns.len();

        for mut global in prog.globals {
            // fn インデックスをオフセット付きで更新
            if let IRGlobalKind::Fn(idx) = &mut global.kind {
                *idx += fn_offset;
            }
            // 重複グローバル（同名）はスキップ
            if !globals.iter().any(|g: &IRGlobal| g.name == global.name) {
                globals.push(global);
            }
        }

        fns.extend(prog.fns);
        type_metas.extend(prog.type_metas);
    }

    IRProgram { globals, fns, type_metas }
}
