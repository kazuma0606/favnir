use crate::frontend::lexer::Span;
use crate::lsp::completion::KNOWN_RUNES;
use crate::lsp::document_store::DocumentStore;
use crate::lsp::hover::{position_to_char_offset, span_contains};
use crate::lsp::protocol::{Location, Position, Range};

/// Detect `<ns>.<fn>` pattern at `offset` and jump to the Rune function definition.
pub fn handle_rune_definition(src: &str, offset: usize, workspace_root: &str) -> Option<Location> {
    // Extract identifier before cursor (the function name)
    let before = &src[..offset.min(src.len())];
    // Find end of function name: scan back from cursor past alnum/_
    let fn_end = before.len();
    let fn_start = before
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_alphanumeric() || *c == '_')
        .last()
        .map(|(i, _)| i)
        .unwrap_or(fn_end);
    if fn_start == fn_end {
        return None;
    }
    let fn_name = &before[fn_start..fn_end];

    // Expect a dot before the function name
    let before_fn = &before[..fn_start];
    if !before_fn.ends_with('.') {
        return None;
    }

    // Extract namespace before the dot
    let before_dot = &before_fn[..before_fn.len() - 1];
    let ns_end = before_dot.len();
    let ns_start = before_dot
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_alphanumeric() || *c == '_')
        .last()
        .map(|(i, _)| i)
        .unwrap_or(ns_end);
    let ns = &before_dot[ns_start..ns_end];
    if ns.is_empty() {
        return None;
    }

    // Check that ns is a known rune
    let is_known = KNOWN_RUNES.iter().any(|(name, _)| *name == ns);
    if !is_known {
        return None;
    }

    // Build rune file path
    let rune_path = std::path::Path::new(workspace_root)
        .join("rune_modules")
        .join(ns)
        .join(format!("{}.fav", ns));

    let source = std::fs::read_to_string(&rune_path).ok()?;

    // Find line containing `fn <fn_name>` or `public fn <fn_name>`
    let target_fn = format!("fn {}", fn_name);
    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(&target_fn)
            || trimmed.starts_with(&format!("public {}", target_fn))
        {
            let uri = format!(
                "file:///{}",
                rune_path
                    .to_string_lossy()
                    .replace('\\', "/")
                    .trim_start_matches('/')
            );
            return Some(Location {
                uri,
                range: Range {
                    start: Position {
                        line: line_idx as u32,
                        character: 0,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: 0,
                    },
                },
            });
        }
    }
    None
}

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
