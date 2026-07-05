use serde::Serialize;
use std::collections::HashMap;

use crate::frontend::lexer::Span;
use crate::lsp::document_store::DocumentStore;
use crate::lsp::protocol::Position;
use crate::middle::checker::Type;

#[derive(Debug, Serialize)]
pub struct InlayHint {
    pub position: Position,
    pub label: String,
    pub kind: u32, // 1 = Type
}

pub fn handle_inlay_hints(store: &DocumentStore, uri: &str) -> Vec<InlayHint> {
    let doc = match store.get(uri) {
        Some(d) => d,
        None => return vec![],
    };
    collect_bind_hints(&doc.source, &doc.type_at)
}

pub(crate) fn collect_bind_hints(
    source: &str,
    type_at: &HashMap<Span, Type>,
) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    let mut byte_offset: usize = 0;
    for (line_idx, line) in source.lines().enumerate() {
        if let Some(rest) = find_bind_prefix(line) {
            let name_end = rest
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            if name_end == 0 {
                byte_offset += line.len() + 1;
                continue;
            }
            let name = &rest[..name_end];
            if name == "_" {
                byte_offset += line.len() + 1;
                continue;
            }
            let prefix_len = line.len() - rest.len();
            let name_start = byte_offset + prefix_len;
            let name_end_offset = name_start + name.len();
            if let Some(ty) = find_type_at(type_at, name_start, name_end_offset) {
                let col = (prefix_len + name.len()) as u32;
                hints.push(InlayHint {
                    position: Position {
                        line: line_idx as u32,
                        character: col,
                    },
                    label: format!(": {}", ty.display()),
                    kind: 1,
                });
            }
        }
        byte_offset += line.len() + 1;
    }
    hints
}

fn find_bind_prefix(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    trimmed.strip_prefix("bind ").map(|r| r.trim_start())
}

fn find_type_at(
    type_at: &HashMap<Span, Type>,
    name_start: usize,
    name_end: usize,
) -> Option<&Type> {
    type_at
        .iter()
        .filter(|(span, _)| span.start <= name_end && span.end >= name_start)
        .min_by_key(|(span, _)| span.end.saturating_sub(span.start))
        .map(|(_, ty)| ty)
}
