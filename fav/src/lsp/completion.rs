use crate::lsp::document_store::{CheckedDoc, DocumentStore};
use crate::lsp::hover::{position_to_char_offset, span_contains};
use crate::lsp::protocol::{CompletionItem, MarkupContent, Position, completion_kind};
use crate::middle::checker::{SymbolKind, Type};

const KEYWORDS: &[&str] = &[
    "fn",
    "type",
    "stage",
    "seq",
    "interface",
    "impl",
    "match",
    "if",
    "else",
    "bind",
    "chain",
    "collect",
    "yield",
    "public",
    "async",
    "for",
    "in",
    "where",
    "bench",
    "test",
];

pub fn handle_completion(
    store: &DocumentStore,
    uri: &str,
    pos: Position,
    trigger_char: Option<String>,
) -> Vec<CompletionItem> {
    let Some(doc) = store.get(uri) else {
        return Vec::new();
    };
    let Some(offset) = position_to_char_offset(&doc.source, pos) else {
        return Vec::new();
    };

    if trigger_char.as_deref() == Some(".") {
        return field_completions(doc, offset);
    }

    let mut items = Vec::new();
    items.extend(global_completions(doc));
    items.extend(keyword_completions());
    items.extend(snippet_completions());
    items
}

fn field_completions(doc: &CheckedDoc, offset: usize) -> Vec<CompletionItem> {
    let Some(dot_offset) = offset.checked_sub(1) else {
        return Vec::new();
    };
    let Some(ty) = doc
        .type_at
        .iter()
        .filter(|(span, _)| span_contains(span, dot_offset.saturating_sub(1)))
        .min_by_key(|(span, _)| span.end.saturating_sub(span.start))
        .map(|(_, ty)| ty)
    else {
        return Vec::new();
    };
    let Some(type_name) = named_record_type(ty) else {
        return Vec::new();
    };
    let Some(fields) = doc.record_fields.get(type_name) else {
        return Vec::new();
    };

    fields
        .iter()
        .map(|(name, ty)| CompletionItem {
            label: name.clone(),
            kind: completion_kind::FIELD,
            detail: Some(ty.display()),
            insert_text: None,
            insert_text_format: None,
            documentation: None,
        })
        .collect()
}

fn named_record_type(ty: &Type) -> Option<&str> {
    match ty {
        Type::Named(name, _) => Some(name.as_str()),
        _ => None,
    }
}

fn global_completions(doc: &CheckedDoc) -> Vec<CompletionItem> {
    doc.symbols
        .iter()
        .map(|symbol| CompletionItem {
            label: symbol.name.clone(),
            kind: match symbol.kind {
                SymbolKind::Function => completion_kind::FUNCTION,
                SymbolKind::Type | SymbolKind::Stage | SymbolKind::Seq | SymbolKind::Interface => {
                    completion_kind::CLASS
                }
            },
            detail: Some(symbol.detail.clone()),
            insert_text: None,
            insert_text_format: None,
            documentation: doc
                .doc_comments
                .get(&symbol.name)
                .map(|text| MarkupContent {
                    kind: "markdown".to_string(),
                    value: text.clone(),
                }),
        })
        .collect()
}

fn keyword_completions() -> Vec<CompletionItem> {
    KEYWORDS
        .iter()
        .map(|keyword| CompletionItem {
            label: (*keyword).to_string(),
            kind: completion_kind::KEYWORD,
            detail: None,
            insert_text: None,
            insert_text_format: None,
            documentation: None,
        })
        .collect()
}

fn snippet_completions() -> Vec<CompletionItem> {
    [
        (
            "fn",
            "fn ${1:name}(${2:param}: ${3:Type}) -> ${4:RetType} {\n    $0\n}",
        ),
        ("type", "type ${1:Name} = { ${2:field}: ${3:Type} }"),
        (
            "interface",
            "interface ${1:Name} {\n    ${2:method}: ${3:Type}\n}",
        ),
        ("match", "match ${1:expr} {\n    ${2:pattern} => $0\n}"),
    ]
    .into_iter()
    .map(|(label, insert_text)| CompletionItem {
        label: label.to_string(),
        kind: completion_kind::SNIPPET,
        detail: None,
        insert_text: Some(insert_text.to_string()),
        insert_text_format: Some(2),
        documentation: None,
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::handle_completion;
    use crate::lsp::document_store::DocumentStore;
    use crate::lsp::protocol::Position;

    #[test]
    fn completion_returns_field_items_on_dot_trigger() {
        let mut store = DocumentStore::new();
        store.open_or_change(
            "file:///main.fav",
            "type User = { name: String age: Int }\nfn get(user: User) -> String = user.name"
                .to_string(),
        );
        let items = handle_completion(
            &store,
            "file:///main.fav",
            Position {
                line: 1,
                character: 36,
            },
            Some(".".to_string()),
        );
        assert!(items.iter().any(|item| item.label == "name"));
        assert!(items.iter().any(|item| item.label == "age"));
    }

    #[test]
    fn completion_returns_global_fn_name() {
        let mut store = DocumentStore::new();
        store.open_or_change(
            "file:///main.fav",
            "fn double(n: Int) -> Int = n * 2\nfn main() -> Int = do".to_string(),
        );
        let items = handle_completion(
            &store,
            "file:///main.fav",
            Position {
                line: 1,
                character: 20,
            },
            None,
        );
        assert!(items.iter().any(|item| item.label == "double"));
    }

    #[test]
    fn completion_includes_keywords() {
        let mut store = DocumentStore::new();
        store.open_or_change("file:///main.fav", "fn main() -> Int = 1".to_string());
        let items = handle_completion(
            &store,
            "file:///main.fav",
            Position {
                line: 0,
                character: 10,
            },
            None,
        );
        assert!(items.iter().any(|item| item.label == "match"));
        assert!(items.iter().any(|item| item.label == "bind"));
    }

    #[test]
    fn completion_includes_snippets() {
        let mut store = DocumentStore::new();
        store.open_or_change("file:///main.fav", "fn main() -> Int = 1".to_string());
        let items = handle_completion(
            &store,
            "file:///main.fav",
            Position {
                line: 0,
                character: 10,
            },
            None,
        );
        assert!(
            items
                .iter()
                .any(|item| item.label == "fn" && item.insert_text_format == Some(2))
        );
    }
}
