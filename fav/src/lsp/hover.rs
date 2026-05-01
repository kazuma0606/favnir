use crate::frontend::lexer::Span;
use crate::lsp::document_store::DocumentStore;
use crate::lsp::protocol::{Hover, MarkupContent, Position};
use crate::middle::checker::Type;

pub fn handle_hover(store: &DocumentStore, uri: &str, pos: Position) -> Option<Hover> {
    let doc = store.get(uri)?;
    let offset = position_to_char_offset(&doc.source, pos)?;

    let (_, ty) = doc
        .type_at
        .iter()
        .filter(|(span, _)| span_contains(span, offset))
        .min_by_key(|(span, _)| span.end.saturating_sub(span.start))?;

    Some(Hover {
        contents: MarkupContent {
            kind: "markdown".to_string(),
            value: format!("```favnir\n{}\n```", display_type(ty)),
        },
    })
}

fn display_type(ty: &Type) -> String {
    ty.display()
}

fn span_contains(span: &Span, offset: usize) -> bool {
    if span.start == span.end {
        offset == span.start
    } else {
        offset >= span.start && offset < span.end
    }
}

fn position_to_char_offset(source: &str, pos: Position) -> Option<usize> {
    let target_line = pos.line as usize;
    let target_char = pos.character as usize;

    let mut offset = 0usize;
    for (line_idx, line) in source.split('\n').enumerate() {
        if line_idx == target_line {
            let line_len = line.chars().count();
            if target_char > line_len {
                return None;
            }
            return Some(offset + target_char);
        }
        offset += line.chars().count() + 1;
    }

    None
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
}
