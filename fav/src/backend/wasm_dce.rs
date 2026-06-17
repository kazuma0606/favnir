// v19.6.0 — Dead Code Elimination for WASM output.
// Traverses IRProgram from `main` (BFS) and removes unreachable functions.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::middle::ir::{IRExpr, IRGlobal, IRGlobalKind, IRProgram, IRStmt};

/// Dead code elimination report.
#[derive(Debug, Clone)]
pub struct DceReport {
    pub removed: usize,
    pub remaining: usize,
}

/// Collect fn indices reachable from `entry` via BFS over the call graph.
/// Returns a `HashSet<usize>` of IR fn indices (indices into `ir.fns`).
pub fn collect_reachable_fns(ir: &IRProgram, entry: &str) -> HashSet<usize> {
    // Build global_idx → fn_idx map for Fn globals.
    let global_to_fn: HashMap<usize, usize> = ir
        .globals
        .iter()
        .enumerate()
        .filter_map(|(g_idx, g)| {
            if let IRGlobalKind::Fn(fn_idx) = g.kind {
                Some((g_idx, fn_idx))
            } else {
                None
            }
        })
        .collect();

    // Find the entry function's fn_idx.
    let entry_fn_idx = ir.globals.iter().find_map(|g| {
        if g.name == entry {
            if let IRGlobalKind::Fn(fn_idx) = g.kind {
                Some(fn_idx)
            } else {
                None
            }
        } else {
            None
        }
    });

    let entry_fn_idx = match entry_fn_idx {
        Some(idx) => idx,
        None => return HashSet::new(),
    };

    let mut visited: HashSet<usize> = HashSet::new();
    let mut queue: VecDeque<usize> = VecDeque::new();
    queue.push_back(entry_fn_idx);

    while let Some(fn_idx) = queue.pop_front() {
        if !visited.insert(fn_idx) {
            continue;
        }
        if let Some(fn_def) = ir.fns.get(fn_idx) {
            collect_expr_fns(
                &fn_def.body,
                &ir.globals,
                &global_to_fn,
                &mut queue,
            );
        }
    }

    visited
}

fn collect_expr_fns(
    expr: &IRExpr,
    globals: &[IRGlobal],
    global_to_fn: &HashMap<usize, usize>,
    queue: &mut VecDeque<usize>,
) {
    match expr {
        IRExpr::Global(idx, _) | IRExpr::TrfRef(idx, _) => {
            let g_idx = *idx as usize;
            if let Some(&fn_idx) = global_to_fn.get(&g_idx) {
                queue.push_back(fn_idx);
            }
        }
        IRExpr::Closure(global_idx, captures, _) => {
            let g_idx = *global_idx as usize;
            if let Some(&fn_idx) = global_to_fn.get(&g_idx) {
                queue.push_back(fn_idx);
            }
            for cap in captures {
                collect_expr_fns(cap, globals, global_to_fn, queue);
            }
        }
        IRExpr::Call(func, args, _) => {
            collect_expr_fns(func, globals, global_to_fn, queue);
            for arg in args {
                collect_expr_fns(arg, globals, global_to_fn, queue);
            }
        }
        IRExpr::FieldAccess(obj, _, _) => {
            collect_expr_fns(obj, globals, global_to_fn, queue);
        }
        IRExpr::Block(stmts, tail, _) => {
            for stmt in stmts {
                collect_stmt_fns(stmt, globals, global_to_fn, queue);
            }
            collect_expr_fns(tail, globals, global_to_fn, queue);
        }
        IRExpr::If(cond, then_, else_, _) => {
            collect_expr_fns(cond, globals, global_to_fn, queue);
            collect_expr_fns(then_, globals, global_to_fn, queue);
            collect_expr_fns(else_, globals, global_to_fn, queue);
        }
        IRExpr::Match(scrutinee, arms, _) => {
            collect_expr_fns(scrutinee, globals, global_to_fn, queue);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_expr_fns(guard, globals, global_to_fn, queue);
                }
                collect_expr_fns(&arm.body, globals, global_to_fn, queue);
            }
        }
        IRExpr::BinOp(_, lhs, rhs, _) => {
            collect_expr_fns(lhs, globals, global_to_fn, queue);
            collect_expr_fns(rhs, globals, global_to_fn, queue);
        }
        IRExpr::CallTrfLocal { arg, .. } => {
            collect_expr_fns(arg, globals, global_to_fn, queue);
        }
        IRExpr::Collect(inner, _) | IRExpr::Emit(inner, _) => {
            collect_expr_fns(inner, globals, global_to_fn, queue);
        }
        IRExpr::RecordConstruct(fields, _) => {
            for (_, val) in fields {
                collect_expr_fns(val, globals, global_to_fn, queue);
            }
        }
        IRExpr::RecordSpread(base, fields, _) => {
            collect_expr_fns(base, globals, global_to_fn, queue);
            for (_, val) in fields {
                collect_expr_fns(val, globals, global_to_fn, queue);
            }
        }
        IRExpr::Lit(_, _) | IRExpr::Local(_, _) => {}
    }
}

fn collect_stmt_fns(
    stmt: &IRStmt,
    globals: &[IRGlobal],
    global_to_fn: &HashMap<usize, usize>,
    queue: &mut VecDeque<usize>,
) {
    match stmt {
        IRStmt::Bind(_, expr)
        | IRStmt::LegacyBind(_, expr)
        | IRStmt::Chain(_, expr)
        | IRStmt::Yield(expr)
        | IRStmt::Expr(expr) => collect_expr_fns(expr, globals, global_to_fn, queue),
        IRStmt::SeqChain { expr, .. } => {
            collect_expr_fns(expr, globals, global_to_fn, queue)
        }
        IRStmt::TrackLine(_) => {}
        IRStmt::RefinementAssert { expr, .. } => {
            collect_expr_fns(expr, globals, global_to_fn, queue)
        }
    }
}

/// Apply DCE: remove unreachable functions from `ir.fns`.
///
/// Strategy: keep all `ir.globals` entries at their original indices
/// (so `IRExpr::Global(idx)` references remain valid), but:
/// - Remap `IRGlobalKind::Fn(idx)` for reachable fns to new fn indices
/// - Mark unreachable Fn globals as `IRGlobalKind::Builtin` (tombstone)
///
/// This avoids re-indexing all `IRExpr::Global` references in function bodies.
pub fn apply_dce(ir: &mut IRProgram, reachable: &HashSet<usize>) -> DceReport {
    let original_count = ir.fns.len();

    // Build old_fn_idx → new_fn_idx remap for reachable fns.
    let mut fn_remap: HashMap<usize, usize> = HashMap::new();
    let mut new_fns = Vec::new();
    for (old_idx, fn_def) in ir.fns.iter().enumerate() {
        if reachable.contains(&old_idx) {
            fn_remap.insert(old_idx, new_fns.len());
            new_fns.push(fn_def.clone());
        }
    }
    ir.fns = new_fns;

    // Update ir.globals: remap reachable Fn globals, tombstone unreachable ones.
    for g in ir.globals.iter_mut() {
        if let IRGlobalKind::Fn(ref mut idx) = g.kind {
            if let Some(&new_idx) = fn_remap.get(idx) {
                *idx = new_idx;
            } else {
                // Mark as tombstone — global index stays the same, fn is gone.
                g.kind = IRGlobalKind::Builtin;
            }
        }
    }

    DceReport {
        removed: original_count - ir.fns.len(),
        remaining: ir.fns.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Lit;
    use crate::middle::checker::Type;
    use crate::middle::ir::{IRExpr, IRFnDef, IRGlobal, IRGlobalKind, IRProgram};

    fn simple_ir(with_dead: bool) -> IRProgram {
        let mut globals = vec![
            IRGlobal { name: "main".into(), kind: IRGlobalKind::Fn(0) },
        ];
        let mut fns = vec![IRFnDef {
            name: "main".into(),
            param_count: 0,
            param_tys: vec![],
            local_count: 0,
            effects: vec![],
            return_ty: Type::Int,
            body: IRExpr::Lit(Lit::Int(1), Type::Int),
        }];
        if with_dead {
            globals.push(IRGlobal { name: "dead_fn".into(), kind: IRGlobalKind::Fn(1) });
            fns.push(IRFnDef {
                name: "dead_fn".into(),
                param_count: 0,
                param_tys: vec![],
                local_count: 0,
                effects: vec![],
                return_ty: Type::Int,
                body: IRExpr::Lit(Lit::Int(99), Type::Int),
            });
        }
        IRProgram { globals, fns, type_metas: Default::default() }
    }

    #[test]
    fn dce_removes_dead_fn() {
        let mut ir = simple_ir(true);
        assert_eq!(ir.fns.len(), 2);
        let reachable = collect_reachable_fns(&ir, "main");
        let report = apply_dce(&mut ir, &reachable);
        assert_eq!(ir.fns.len(), 1, "dead_fn should be removed");
        assert_eq!(report.removed, 1);
        assert_eq!(report.remaining, 1);
    }

    #[test]
    fn dce_keeps_all_if_all_reachable() {
        let mut ir = simple_ir(false);
        let reachable = collect_reachable_fns(&ir, "main");
        let report = apply_dce(&mut ir, &reachable);
        assert_eq!(ir.fns.len(), 1);
        assert_eq!(report.removed, 0);
    }

    #[test]
    fn dce_tombstones_dead_global() {
        let mut ir = simple_ir(true);
        let reachable = collect_reachable_fns(&ir, "main");
        apply_dce(&mut ir, &reachable);
        // dead_fn global should be tombstoned to Builtin
        let dead_g = ir.globals.iter().find(|g| g.name == "dead_fn").unwrap();
        assert!(
            matches!(dead_g.kind, IRGlobalKind::Builtin),
            "dead fn global should be tombstoned"
        );
    }
}
