use std::collections::HashMap;

use crate::lsp::document_store::DocumentStore;
use crate::lsp::hover::position_to_char_offset;
use crate::lsp::protocol::{Position, TextEdit, WorkspaceEdit};
use crate::lsp::references::{collect_symbol_occurrences, span_to_range, word_at_offset};

const FAVNIR_KEYWORDS: &[&str] = &[
    "fn", "stage", "type", "bind", "chain", "seq", "pub",
    "public", "match", "if", "else", "true", "false",
    "use", "import", "test", "forall", "par", "abstract",
    "interface", "impl", "cap", "bench", "namespace",
];

pub fn handle_rename(
    store: &DocumentStore,
    uri: &str,
    pos: Position,
    new_name: &str,
) -> Option<WorkspaceEdit> {
    let doc = store.get(uri)?;
    let program = doc.program.as_ref()?;
    let offset = position_to_char_offset(&doc.source, pos)?;
    let name = word_at_offset(&doc.source, offset)?;

    if FAVNIR_KEYWORDS.contains(&name.as_str()) {
        return None;
    }

    let spans = collect_symbol_occurrences(program, &name);
    if spans.is_empty() {
        return None;
    }

    let edits: Vec<TextEdit> = spans
        .into_iter()
        .map(|span| TextEdit {
            range: span_to_range(&span),
            new_text: new_name.to_string(),
        })
        .collect();

    let mut changes = HashMap::new();
    changes.insert(uri.to_string(), edits);
    Some(WorkspaceEdit { changes })
}
