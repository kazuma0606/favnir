use crate::middle::ir::{IRGlobalKind, IRProgram, collect_calls_in_ir};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ReachabilityResult {
    pub included: HashSet<String>,
    pub excluded: HashSet<String>,
    pub effects_required: Vec<String>,
    pub emits: Vec<String>,
}

pub fn reachability_analysis(entry: &str, program: &IRProgram) -> ReachabilityResult {
    let fn_map: HashMap<String, &crate::middle::ir::IRFnDef> =
        program.fns.iter().map(|f| (f.name.clone(), f)).collect();
    let all_names: HashSet<String> = program
        .globals
        .iter()
        .filter_map(|g| match g.kind {
            IRGlobalKind::Fn(_) | IRGlobalKind::VariantCtor => Some(g.name.clone()),
            IRGlobalKind::Builtin => None,
        })
        .collect();

    let mut included = HashSet::new();
    let mut work = vec![entry.to_string()];

    while let Some(name) = work.pop() {
        if !included.insert(name.clone()) {
            continue;
        }
        let Some(fn_def) = fn_map.get(&name) else {
            continue;
        };
        for dep in collect_calls_in_ir(fn_def, &program.globals) {
            if all_names.contains(&dep) && !included.contains(&dep) {
                work.push(dep);
            }
        }
    }

    let excluded = all_names
        .iter()
        .filter(|name| !included.contains(*name))
        .cloned()
        .collect::<HashSet<_>>();

    ReachabilityResult {
        included,
        excluded,
        effects_required: vec![],
        emits: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::reachability_analysis;
    use crate::frontend::parser::Parser;
    use crate::middle::compiler::compile_program;

    #[test]
    fn test_reachability_simple() {
        let program = Parser::parse_str(
            r#"
fn b() -> Int { 1 }
fn a() -> Int { b() }
public fn main() -> Int { a() }
"#,
            "reachability_simple.fav",
        )
        .expect("parse");
        let ir = compile_program(&program);
        let result = reachability_analysis("main", &ir);
        assert!(result.included.contains("main"));
        assert!(result.included.contains("a"));
        assert!(result.included.contains("b"));
    }

    #[test]
    fn test_reachability_excluded() {
        let program = Parser::parse_str(
            r#"
fn helper() -> Int { 1 }
public fn main() -> Int { 0 }
"#,
            "reachability_excluded.fav",
        )
        .expect("parse");
        let ir = compile_program(&program);
        let result = reachability_analysis("main", &ir);
        assert!(result.included.contains("main"));
        assert!(result.excluded.contains("helper"));
    }

    #[test]
    fn test_reachability_effects_required() {
        // v34.8A: !Db annotation removed (E0374); pure stage has no effect requirements
        let program = Parser::parse_str(
            r#"
stage Save: Int -> Int = |x| { x }
public fn main() -> Int { Save(1) }
"#,
            "reachability_effects.fav",
        )
        .expect("parse");
        let ir = compile_program(&program);
        let result = reachability_analysis("main", &ir);
        // Pure stage with no annotation has no Db effect requirement
        assert!(
            !result.effects_required.iter().any(|e| e == "Db"),
            "expected no Db effect for pure stage, got: {:?}", result.effects_required
        );
    }
}
