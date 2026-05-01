use std::collections::HashMap;

use crate::frontend::parser::Parser;
use crate::frontend::lexer::Span;
use crate::middle::checker::{Checker, Type, TypeError};

#[derive(Debug, Default)]
pub struct CheckedDoc {
    pub source: String,
    pub errors: Vec<TypeError>,
    pub type_at: HashMap<Span, Type>,
}

#[derive(Debug, Default)]
pub struct DocumentStore {
    docs: HashMap<String, CheckedDoc>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open_or_change(&mut self, uri: impl Into<String>, source: String) {
        let uri = uri.into();
        let checked = match Parser::parse_str(&source, &uri) {
            Ok(program) => {
                let mut checker = Checker::new();
                let errors = checker.check_with_self(&program);
                CheckedDoc {
                    source,
                    errors,
                    type_at: checker.type_at,
                }
            }
            Err(err) => CheckedDoc {
                source,
                errors: vec![TypeError::new("E000", err.message, err.span)],
                type_at: HashMap::new(),
            },
        };
        self.docs.insert(
            uri,
            checked,
        );
    }

    pub fn get(&self, uri: &str) -> Option<&CheckedDoc> {
        self.docs.get(uri)
    }
}

#[cfg(test)]
mod tests {
    use super::DocumentStore;

    #[test]
    fn open_or_change_collects_checker_types() {
        let mut docs = DocumentStore::new();
        docs.open_or_change(
            "file:///main.fav",
            "fn main() -> Int { 42 }".to_string(),
        );
        let doc = docs.get("file:///main.fav").expect("checked doc");
        assert!(doc.errors.is_empty(), "unexpected errors: {:?}", doc.errors);
        assert!(!doc.type_at.is_empty(), "expected at least one recorded type");
    }

    #[test]
    fn open_or_change_reports_parse_error_as_e000() {
        let mut docs = DocumentStore::new();
        docs.open_or_change("file:///broken.fav", "fn main(".to_string());
        let doc = docs.get("file:///broken.fav").expect("checked doc");
        assert_eq!(doc.errors.len(), 1);
        assert_eq!(doc.errors[0].code, "E000");
        assert!(doc.type_at.is_empty());
    }
}
