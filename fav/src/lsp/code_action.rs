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
    // v46.5.0: E0102 did-you-mean + E0101 arg count quickfix
    actions.extend(check_did_you_mean_fix(doc, uri, range));
    actions.extend(check_arg_count_fix(uri, doc, range));
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

// ── CA-4: didYouMeanFix (E0102 — 未定義変数) ────────────────────────────────

fn check_did_you_mean_fix(
    doc: &crate::lsp::document_store::CheckedDoc,
    uri: &str,
    range: Range,
) -> Vec<CodeAction> {
    let line_idx = range.start.line;
    let mut actions = Vec::new();
    for err in &doc.errors {
        if err.code != "E0102" {
            continue;
        }
        if err.span.line == 0 || err.span.col == 0 || err.span.line - 1 != line_idx {
            continue;
        }
        for hint in &err.hints {
            if let Some(name) = parse_did_you_mean(hint) {
                // span.col is 1-based → convert to 0-based character offset
                let start_col = err.span.col - 1;
                // span.end - span.start = byte length of the identifier.
                // Favnir identifiers are ASCII-only (alphanumeric + '_'),
                // so byte count == char count == LSP character width.
                let len = (err.span.end.saturating_sub(err.span.start)) as u32;
                let edit_range = Range {
                    start: Position { line: line_idx, character: start_col },
                    end:   Position { line: line_idx, character: start_col + len },
                };
                let mut changes = HashMap::new();
                changes.insert(
                    uri.to_string(),
                    vec![TextEdit { range: edit_range, new_text: name.to_string() }],
                );
                actions.push(CodeAction {
                    title: format!("Did you mean `{}`?", name),
                    kind: Some("quickfix".to_string()),
                    edit: Some(WorkspaceEdit { changes }),
                });
            }
        }
    }
    actions
}

/// "did you mean `foo`?" → Some("foo")
fn parse_did_you_mean(hint: &str) -> Option<&str> {
    let start = hint.find('`')? + 1;
    let end = hint[start..].find('`')? + start;
    Some(&hint[start..end])
}

// ── CA-5: argCountFix (E0101 — 引数数不一致) ────────────────────────────────

fn check_arg_count_fix(
    // _uri: reserved for future TextEdit support (e.g. auto-fill missing arguments)
    _uri: &str,
    doc: &crate::lsp::document_store::CheckedDoc,
    range: Range,
) -> Vec<CodeAction> {
    let line_idx = range.start.line;
    doc.errors
        .iter()
        .filter(|e| {
            e.code == "E0101"
                && e.message.contains("argument(s)")
                && e.span.line > 0
                && e.span.line - 1 == line_idx
        })
        .map(|e| CodeAction {
            title: format!("Fix: {}", e.message),
            kind: Some("quickfix".to_string()),
            edit: None,
        })
        .collect()
}
