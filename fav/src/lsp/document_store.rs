use std::collections::HashMap;

use crate::frontend::lexer::Span;
use crate::frontend::parser::Parser;
use crate::lsp::doc_comment::extract_doc_comments;
use crate::middle::checker::{Checker, LspSymbol, Type, TypeError};

#[derive(Debug, Default)]
pub struct CheckedDoc {
    pub source: String,
    pub errors: Vec<TypeError>,
    pub type_at: HashMap<Span, Type>,
    pub symbols: Vec<LspSymbol>,
    pub def_at: HashMap<Span, Span>,
    pub doc_comments: HashMap<String, String>,
    pub record_fields: HashMap<String, Vec<(String, Type)>>,
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
                let (errors, _) = checker.check_with_self(&program);
                let doc_comments = extract_doc_comments(&source);
                CheckedDoc {
                    source,
                    errors,
                    type_at: checker.type_at,
                    symbols: checker.symbol_index,
                    def_at: checker.def_at,
                    doc_comments,
                    record_fields: checker.record_fields,
                }
            }
            Err(err) => CheckedDoc {
                source,
                errors: vec![TypeError::new("E0500", err.message, err.span)],
                type_at: HashMap::new(),
                symbols: Vec::new(),
                def_at: HashMap::new(),
                doc_comments: HashMap::new(),
                record_fields: HashMap::new(),
            },
        };
        self.docs.insert(uri, checked);
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
        docs.open_or_change("file:///main.fav", "fn main() -> Int { 42 }".to_string());
        let doc = docs.get("file:///main.fav").expect("checked doc");
        assert!(doc.errors.is_empty(), "unexpected errors: {:?}", doc.errors);
        assert!(
            !doc.type_at.is_empty(),
            "expected at least one recorded type"
        );
        assert!(doc.symbols.iter().any(|sym| sym.name == "main"));
    }

    #[test]
    fn open_or_change_reports_parse_error_as_e000() {
        let mut docs = DocumentStore::new();
        docs.open_or_change("file:///broken.fav", "fn main(".to_string());
        let doc = docs.get("file:///broken.fav").expect("checked doc");
        assert_eq!(doc.errors.len(), 1);
        assert_eq!(doc.errors[0].code, "E0500");
        assert!(doc.type_at.is_empty());
        assert!(doc.symbols.is_empty());
    }
}
