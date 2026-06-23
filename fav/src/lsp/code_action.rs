use std::collections::HashMap;

use crate::lsp::completion::KNOWN_RUNES;
use crate::lsp::document_store::DocumentStore;
use crate::lsp::protocol::{CodeAction, Position, Range, TextEdit, WorkspaceEdit};

pub fn handle_code_action(store: &DocumentStore, uri: &str, range: Range) -> Vec<CodeAction> {
    let doc = match store.get(uri) {
        Some(d) => d,
        None => return Vec::new(),
    };

    // get the text of the cursor line
    let line_idx = range.start.line as usize;
    let line_text = doc.source.lines().nth(line_idx).unwrap_or("").to_string();

    let mut actions = Vec::new();

    if let Some(a) = check_add_missing_import(doc, uri, &line_text) {
        actions.push(a);
    }
    if let Some(a) = check_convert_to_fstring(&line_text) {
        actions.push(a);
    }
    if let Some(a) = check_inline_binding(&line_text) {
        actions.push(a);
    }
    actions
}

// ── CA-1: addMissingImport ────────────────────────────────────────────────────

fn check_add_missing_import(
    doc: &crate::lsp::document_store::CheckedDoc,
    uri: &str,
    line_text: &str,
) -> Option<CodeAction> {
    // Extract NS from NS.method( pattern
    let ns = extract_namespace(line_text)?;

    // Must be a known rune namespace
    if !KNOWN_RUNES.iter().any(|(n, _)| *n == ns.as_str()) {
        return None;
    }

    // Check if already imported
    if let Some(program) = &doc.program {
        let already_imported = program
            .uses
            .iter()
            .any(|path| path.first().map(|s| s == &ns).unwrap_or(false));
        if already_imported {
            return None;
        }
    }

    // Build WorkspaceEdit: insert "use <ns>\n" at the very beginning
    let insert_text = format!("use {}\n", ns);
    let insert_range = Range {
        start: Position { line: 0, character: 0 },
        end:   Position { line: 0, character: 0 },
    };
    let mut changes = HashMap::new();
    changes.insert(
        uri.to_string(),
        vec![TextEdit { range: insert_range, new_text: insert_text }],
    );

    Some(CodeAction {
        title: format!("Add missing import: use {}", ns),
        kind: Some("quickfix".to_string()),
        edit: Some(WorkspaceEdit { changes }),
    })
}

/// Extract leading namespace from patterns like `http.get(`, `IO.println(`.
fn extract_namespace(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let dot_pos = trimmed.find('.')?;
    let ns: String = trimmed[..dot_pos]
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if ns.is_empty() { None } else { Some(ns) }
}

// ── CA-2: convertToFstring ───────────────────────────────────────────────────

fn check_convert_to_fstring(line_text: &str) -> Option<CodeAction> {
    if !line_text.contains("String.concat(") {
        return None;
    }
    Some(CodeAction {
        title: "Convert to f-string".to_string(),
        kind: Some("refactor.rewrite".to_string()),
        edit: None,
    })
}

// ── CA-3: inlineBinding ──────────────────────────────────────────────────────

fn check_inline_binding(line_text: &str) -> Option<CodeAction> {
    let trimmed = line_text.trim_start();
    if !trimmed.starts_with("bind ") {
        return None;
    }
    // Extract binding name: "bind <name> <-"
    let rest = &trimmed["bind ".len()..];
    let name: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() || name == "_" {
        return None;
    }
    Some(CodeAction {
        title: format!("Inline binding `{}`", name),
        kind: Some("refactor.inline".to_string()),
        edit: None,
    })
}
