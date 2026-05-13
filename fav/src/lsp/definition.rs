use crate::frontend::lexer::Span;
use crate::lsp::document_store::DocumentStore;
use crate::lsp::hover::{position_to_char_offset, span_contains};
use crate::lsp::protocol::{Location, Position, Range};

pub fn handle_definition(store: &DocumentStore, uri: &str, pos: Position) -> Option<Location> {
    let doc = store.get(uri)?;
    let offset = position_to_char_offset(&doc.source, pos)?;
    let (_, def_span) = doc
        .def_at
        .iter()
        .filter(|(usage_span, _)| span_contains(usage_span, offset))
        .min_by_key(|(usage_span, _)| usage_span.end.saturating_sub(usage_span.start))?;

    Some(Location {
        uri: uri.to_string(),
        range: span_to_range(&doc.source, def_span),
    })
}

fn span_to_range(source: &str, span: &Span) -> Range {
    Range {
        start: byte_offset_to_position(source, span.start),
        end: byte_offset_to_position(source, span.end),
    }
}

fn byte_offset_to_position(source: &str, byte_offset: usize) -> Position {
    let clamped = byte_offset.min(source.len());
    let mut line = 0u32;
    let mut character = 0u32;
    for (idx, ch) in source.char_indices() {
        if idx >= clamped {
            break;
        }
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += 1;
        }
    }
    Position { line, character }
}

#[cfg(test)]
mod tests {
    use super::handle_definition;
    use crate::lsp::document_store::DocumentStore;
    use crate::lsp::protocol::Position;

    #[test]
    fn definition_returns_location_for_global_fn() {
        let mut store = DocumentStore::new();
        store.open_or_change(
            "file:///main.fav",
            "fn divide(n: Int) -> Int = n / 2\nfn main() -> Int = divide(8)".to_string(),
        );
        let location = handle_definition(
            &store,
            "file:///main.fav",
            Position {
                line: 1,
                character: 20,
            },
        )
        .expect("location");
        assert_eq!(location.uri, "file:///main.fav");
        assert_eq!(location.range.start.line, 0);
    }

    #[test]
    fn definition_returns_none_for_unknown_position() {
        let mut store = DocumentStore::new();
        store.open_or_change("file:///main.fav", "fn main() -> Int { 42 }".to_string());
        let location = handle_definition(
            &store,
            "file:///main.fav",
            Position {
                line: 0,
                character: 0,
            },
        );
        assert!(location.is_none());
    }
}
