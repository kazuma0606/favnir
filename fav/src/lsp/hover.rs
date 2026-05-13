use crate::frontend::lexer::Span;
use crate::lsp::document_store::DocumentStore;
use crate::lsp::protocol::{Hover, MarkupContent, Position};
use crate::middle::checker::Type;

pub fn handle_hover(store: &DocumentStore, uri: &str, pos: Position) -> Option<Hover> {
    let doc = store.get(uri)?;
    let offset = position_to_char_offset(&doc.source, pos)?;

    let (span, ty) = doc
        .type_at
        .iter()
        .filter(|(span, _)| span_contains(span, offset))
        .min_by_key(|(span, _)| span.end.saturating_sub(span.start))?;

    let type_block = format!("```favnir\n{}\n```", display_type(ty));
    let value = match doc_comment_for_span(doc, span, offset) {
        Some(text) if !text.is_empty() => format!("{}\n\n{}", text, type_block),
        _ => type_block,
    };

    Some(Hover {
        contents: MarkupContent {
            kind: "markdown".to_string(),
            value,
        },
    })
}

fn doc_comment_for_span(
    doc: &crate::lsp::document_store::CheckedDoc,
    span: &Span,
    offset: usize,
) -> Option<String> {
    if let Some(def_span) = doc.def_at.get(span) {
        if let Some(symbol) = doc.symbols.iter().find(|sym| sym.def_span == *def_span) {
            return doc.doc_comments.get(&symbol.name).cloned();
        }
    }

    doc.symbols
        .iter()
        .filter(|sym| span_contains(&sym.def_span, offset))
        .min_by_key(|sym| sym.def_span.end.saturating_sub(sym.def_span.start))
        .and_then(|sym| doc.doc_comments.get(&sym.name).cloned())
}

fn display_type(ty: &Type) -> String {
    ty.display()
}

pub(crate) fn span_contains(span: &Span, offset: usize) -> bool {
    if span.start == span.end {
        offset == span.start
    } else {
        offset >= span.start && offset < span.end
    }
}

pub(crate) fn position_to_char_offset(source: &str, pos: Position) -> Option<usize> {
    let mut line = 0u32;
    let mut character = 0u32;

    for (idx, ch) in source.char_indices() {
        if line == pos.line && character == pos.character {
            return Some(idx);
        }
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += 1;
        }
    }

    if line == pos.line && character == pos.character {
        Some(source.len())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::handle_hover;
    use crate::lsp::document_store::DocumentStore;
    use crate::lsp::protocol::Position;

    #[test]
    fn hover_returns_smallest_matching_type() {
        let mut store = DocumentStore::new();
        store.open_or_change("file:///main.fav", "fn main() -> Int { 42 }".to_string());
        let hover = handle_hover(
            &store,
            "file:///main.fav",
            Position {
                line: 0,
                character: 19,
            },
        )
        .expect("hover");
        assert_eq!(hover.contents.kind, "markdown");
        assert!(hover.contents.value.contains("Int"));
    }

    #[test]
    fn hover_returns_none_when_position_is_outside_any_span() {
        let mut store = DocumentStore::new();
        store.open_or_change("file:///main.fav", "fn main() -> Int { 42 }".to_string());
        let hover = handle_hover(
            &store,
            "file:///main.fav",
            Position {
                line: 0,
                character: 0,
            },
        );
        assert!(hover.is_none());
    }

    #[test]
    fn hover_includes_doc_comment_for_symbol_use() {
        let mut store = DocumentStore::new();
        store.open_or_change(
            "file:///main.fav",
            "// Returns double of n.\nfn double(n: Int) -> Int = n * 2\nfn main() -> Int = double(21)"
                .to_string(),
        );
        let hover = handle_hover(
            &store,
            "file:///main.fav",
            Position {
                line: 2,
                character: 20,
            },
        )
        .expect("hover");
        assert!(hover.contents.value.contains("Returns double of n."));
        assert!(hover.contents.value.contains("Int"));
    }
}
