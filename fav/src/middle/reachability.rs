use crate::ast::Effect;
use crate::middle::ir::{collect_calls_in_ir, IRGlobalKind, IRProgram};
use std::collections::{BTreeSet, HashMap, HashSet};

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

    let mut effects_required = BTreeSet::new();
    let mut emits = BTreeSet::new();
    for name in &included {
        let Some(fn_def) = fn_map.get(name) else {
            continue;
        };
        for effect in &fn_def.effects {
            match effect {
                Effect::Pure => {
                    effects_required.insert("Pure".to_string());
                }
                Effect::Io => {
                    effects_required.insert("Io".to_string());
                }
                Effect::Db => {
                    effects_required.insert("Db".to_string());
                }
                Effect::Network => {
                    effects_required.insert("Network".to_string());
                }
                Effect::File => {
                    effects_required.insert("File".to_string());
                }
                Effect::Unknown(name) => {
                    effects_required.insert(name.clone());
                }
                Effect::Trace => {
                    effects_required.insert("Trace".to_string());
                }
                Effect::Emit(name) => {
                    effects_required.insert(format!("Emit<{name}>"));
                    emits.insert(name.clone());
                }
                Effect::EmitUnion(names) => {
                    effects_required.insert(format!("Emit<{}>", names.join("|")));
                    for name in names {
                        emits.insert(name.clone());
                    }
                }
            }
        }
    }

    ReachabilityResult {
        included,
        excluded,
        effects_required: effects_required.into_iter().collect(),
        emits: emits.into_iter().collect(),
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
        let program = Parser::parse_str(
            r#"
stage Save: Int -> Int !Db = |x| { x }
public fn main() -> Int { Save(1) }
"#,
            "reachability_effects.fav",
        )
        .expect("parse");
        let ir = compile_program(&program);
        let result = reachability_analysis("main", &ir);
        assert!(result.effects_required.iter().any(|e| e == "Db"));
    }
}
