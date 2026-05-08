use std::collections::HashMap;

use crate::ast::{Item, TypeDef};
use crate::frontend::parser::Parser;
use crate::ast::Visibility;
use crate::middle::checker::Type;

const STD_STATES_SOURCE: &str = r#"
public type PosInt = { value: Int invariant value > 0 }
public type NonNegInt = { value: Int invariant value >= 0 }
public type Probability = {
    value: Float
    invariant value >= 0.0
    invariant value <= 1.0
}
public type PortNumber = {
    value: Int
    invariant value >= 1
    invariant value <= 65535
}
public type NonEmptyString = {
    value: String
    invariant String.length(value) > 0
}
public type Email = {
    value: String
    invariant String.contains(value, "@")
    invariant String.length(value) > 3
}
public type Url = {
    value: String
    invariant String.is_url(value)
}
public type Slug = {
    value: String
    invariant String.is_slug(value)
}
"#;

pub fn parsed_type_defs() -> Vec<TypeDef> {
    let program = Parser::parse_str(STD_STATES_SOURCE, "<std.states>")
        .expect("std.states source must parse");
    program
        .items
        .into_iter()
        .filter_map(|item| match item {
            Item::TypeDef(td) => Some(td),
            _ => None,
        })
        .collect()
}

pub fn export_scope() -> HashMap<String, (Type, Visibility)> {
    parsed_type_defs()
        .into_iter()
        .map(|td| (td.name.clone(), (Type::Named(td.name, vec![]), Visibility::Public)))
        .collect()
}
