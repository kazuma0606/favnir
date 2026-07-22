use crate::frontend::lexer::Span;
use crate::lsp::document_store::DocumentStore;
use crate::lsp::protocol::{Hover, MarkupContent, Position};
use crate::middle::checker::Type;

pub fn handle_hover(store: &DocumentStore, uri: &str, pos: Position) -> Option<Hover> {
    let doc = store.get(uri)?;
    let offset = position_to_char_offset(&doc.source, pos)?;

    // v50.6.0: try builtin / rune method hover first.
    // Trade-off: static-table hover takes priority over type_at, meaning call-site-specific
    // inferred types (e.g. a generic List.map instantiation) are not shown. The static
    // signature with doc text is considered more useful for discoverability than the
    // inferred type alone. Type-precise hover can be added in a future version by merging
    // the doc text into the type_at path instead of replacing it.
    if let Some(content) = builtin_hover_at(&doc.source, offset)
        .or_else(|| rune_hover_at(&doc.source, offset))
    {
        return Some(Hover {
            contents: MarkupContent {
                kind: "markdown".to_string(),
                value: content,
            },
        });
    }

    let (span, ty) = doc
        .type_at
        .iter()
        .filter(|(span, _)| span_contains(span, offset))
        .min_by_key(|(span, _)| span.end.saturating_sub(span.start))?;

    let type_block = format!("```favnir\n{}\n```", display_type(ty));
    let base_value = match doc_comment_for_span(doc, span, offset) {
        Some(text) if !text.is_empty() => format!("{}\n\n{}", text, type_block),
        _ => type_block,
    };

    // v53.1.0: append lineage info for stage names
    let token_name = doc.source.get(span.start..span.end).unwrap_or("").trim();
    let value = match lineage_block_for_stage(doc, token_name) {
        Some(lineage_block) => format!("{}\n\n{}", base_value, lineage_block),
        None => base_value,
    };

    Some(Hover {
        contents: MarkupContent {
            kind: "markdown".to_string(),
            value,
        },
    })
}

// v53.1.0: lineage block for stage hover ────────────────────────────────────

fn lineage_block_for_stage(
    doc: &crate::lsp::document_store::CheckedDoc,
    stage_name: &str,
) -> Option<String> {
    if stage_name.is_empty() {
        return None;
    }
    // Only show lineage for names that appear in transformations (i.e. stages).
    // Single find — reuse the entry for schema lookup below.
    let trans_entry = doc
        .lineage
        .transformations
        .iter()
        .find(|e| e.name == stage_name)?;

    let mut lines = Vec::new();

    // upstream / downstream from seq pipelines.
    // First matching pipeline wins; stages shared across multiple pipelines
    // show only the first view (intentional — future versions may expand).
    for pipeline in &doc.lineage.pipelines {
        if let Some(idx) = pipeline.steps.iter().position(|s| s == stage_name) {
            if idx > 0 {
                lines.push(format!("  upstream:   {}", pipeline.steps[idx - 1]));
            }
            if idx + 1 < pipeline.steps.len() {
                lines.push(format!("  downstream: {}", pipeline.steps[idx + 1]));
            }
            break;
        }
    }

    // schema from transformations (reuse entry found above)
    if let Some(ref schema) = trans_entry.schema {
        lines.push(format!("  schema:     {}", schema));
    }

    if lines.is_empty() {
        None
    } else {
        Some(format!("```\n{}\n```", lines.join("\n")))
    }
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

// v50.6.0: Rune method hover ───────────────────────────────────────────────

/// Static metadata for Rune exported functions.
/// v50.6.0: populated from known rune signatures (kafka×2, csv×2 — 4 of 50+ runes).
/// Other runes fall through to the `type_at` hover path; inconsistency is a known TODO.
/// Future: populate dynamically from rune.toml `[[exports]]` sections.
///
/// `pub(crate)` for future use by `completion.rs` / `references.rs`.
pub(crate) struct RuneFn {
    pub rune: &'static str,
    pub name: &'static str,
    pub signature: &'static str, // e.g. "(topic: String) -> Stream<RawMessage>"
    pub effect: &'static str,    // e.g. "!Kafka"
    pub doc: &'static str,
}

/// `pub(crate)` for future use by `completion.rs` / `references.rs`.
pub(crate) const RUNE_FNS: &[RuneFn] = &[
    RuneFn {
        rune: "kafka",
        name: "consume",
        signature: "(topic: String) -> Stream<RawMessage>",
        effect: "!Kafka",
        doc: "Consumes messages from the given Kafka topic.",
    },
    RuneFn {
        rune: "kafka",
        name: "produce",
        signature: "(topic: String, msg: RawMessage) -> Unit",
        effect: "!Kafka",
        doc: "Produces a message to the given Kafka topic.",
    },
    RuneFn {
        rune: "csv",
        name: "read",
        signature: "(path: String) -> List<Row>",
        effect: "!IO",
        doc: "Reads a CSV file and returns rows as a list.",
    },
    RuneFn {
        rune: "csv",
        name: "write",
        signature: "(path: String, rows: List<Row>) -> Unit",
        effect: "!IO",
        doc: "Writes rows to a CSV file.",
    },
];

/// Extract (namespace, method) from source at a byte offset.
/// `offset` must be a byte index (as returned by `position_to_char_offset`,
/// which itself returns `char_indices` first-byte positions — safe for ASCII
/// and for multibyte chars because only first-byte offsets are produced).
///
/// Returns None when:
/// - `offset` is on the dot separator itself (`start == end`)
/// - there is no preceding `namespace.` pattern
/// - `offset == source.len()` (end-of-file): safely returns None in almost all cases
///
/// `offset` may point to any byte within the method name and will work correctly.
///
/// For chained access (`a.b.method`), only the **immediate** left-hand namespace is
/// returned — e.g. offset on `method` yields `("b", "method")`, not `("a", "method")`.
fn word_and_ns_at(source: &str, offset: usize) -> Option<(&str, &str)> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    // find method end
    let mut end = offset;
    while end < len && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
    }
    // find method start
    let mut start = offset;
    while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
        start -= 1;
    }
    if start == end {
        return None;
    }
    // require preceding dot
    if start == 0 || bytes[start - 1] != b'.' {
        return None;
    }
    let dot_pos = start - 1;
    // find namespace start
    let mut ns_start = dot_pos;
    while ns_start > 0 && (bytes[ns_start - 1].is_ascii_alphanumeric() || bytes[ns_start - 1] == b'_') {
        ns_start -= 1;
    }
    if ns_start == dot_pos {
        return None;
    }
    Some((&source[ns_start..dot_pos], &source[start..end]))
}

/// v50.6.0: Text-scan hover for builtin Namespace.method.
/// Reuses `BUILTIN_FNS` from completion.rs. Builtin namespaces are PascalCase.
/// Note: `entry.params` is intentionally unused here.
/// Signature help (existing completion.rs feature) handles parameter-level detail.
pub(crate) fn builtin_hover_at(source: &str, offset: usize) -> Option<String> {
    let (ns, method) = word_and_ns_at(source, offset)?;
    // Builtin namespaces are PascalCase (uppercase first letter).
    // unwrap_or(true): empty namespace → treat as "is lowercase" → skip (conservative).
    if ns.chars().next().map(|c| c.is_lowercase()).unwrap_or(true) {
        return None;
    }
    use crate::lsp::completion::BUILTIN_FNS;
    let entry = BUILTIN_FNS.iter().find(|f| f.namespace == ns && f.name == method)?;
    Some(format!("```favnir\nfn {}.{}{}\n```", ns, method, entry.signature))
}

/// v50.6.0: Text-scan hover for rune.method.
/// Uses static RUNE_FNS table. Rune names are lowercase (vs PascalCase builtins).
pub(crate) fn rune_hover_at(source: &str, offset: usize) -> Option<String> {
    let (rune, method) = word_and_ns_at(source, offset)?;
    // Rune names are lowercase (vs PascalCase builtins).
    // unwrap_or(true): empty rune name → treat as "is uppercase" → skip (conservative).
    if rune.chars().next().map(|c| c.is_uppercase()).unwrap_or(true) {
        return None;
    }
    let entry = RUNE_FNS.iter().find(|f| f.rune == rune && f.name == method)?;
    // Effect is placed outside the fenced code block as a Markdown annotation,
    // since `fn sig  !Effect` is not valid Favnir syntax and would mislead users.
    let sig = format!("fn {}{}", method, entry.signature);
    let value = if entry.doc.is_empty() {
        format!("```favnir\n{}\n```\n\n**Effect:** `{}`", sig, entry.effect)
    } else {
        format!("```favnir\n{}\n```\n\n**Effect:** `{}`\n\n{}", sig, entry.effect, entry.doc)
    };
    Some(value)
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
    use super::{handle_hover, lineage_block_for_stage};
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

    // v53.1.0: lineage display path tests (lineage_block_for_stage is private;
    // tested here directly since tests share the same module scope)
    #[test]
    fn lineage_block_shows_upstream_for_stage() {
        use crate::lsp::document_store::DocumentStore;
        let mut store = DocumentStore::new();
        store.open_or_change(
            "file:///t.fav",
            "stage A: Int -> Int = |n| { n }\nstage B: Int -> Int = |n| { n }\nseq pipe = A |> B"
                .to_string(),
        );
        let doc = store.get("file:///t.fav").unwrap();
        let block = lineage_block_for_stage(doc, "B");
        assert!(block.is_some(), "B should have lineage info");
        let text = block.unwrap();
        assert!(
            text.contains("upstream"),
            "lineage block for B should contain 'upstream', got: {text}"
        );
        assert!(
            text.contains('A'),
            "upstream of B should be A, got: {text}"
        );
    }

    #[test]
    fn lineage_block_shows_downstream_for_stage() {
        use crate::lsp::document_store::DocumentStore;
        let mut store = DocumentStore::new();
        store.open_or_change(
            "file:///t.fav",
            "stage A: Int -> Int = |n| { n }\nstage B: Int -> Int = |n| { n }\nseq pipe = A |> B"
                .to_string(),
        );
        let doc = store.get("file:///t.fav").unwrap();
        let block = lineage_block_for_stage(doc, "A");
        assert!(block.is_some(), "A should have lineage info");
        let text = block.unwrap();
        assert!(
            text.contains("downstream"),
            "lineage block for A should contain 'downstream', got: {text}"
        );
        assert!(
            text.contains('B'),
            "downstream of A should be B, got: {text}"
        );
    }

    #[test]
    fn lineage_block_returns_none_for_non_stage() {
        use crate::lsp::document_store::DocumentStore;
        let mut store = DocumentStore::new();
        store.open_or_change(
            "file:///t.fav",
            "fn helper(n: Int) -> Int = n".to_string(),
        );
        let doc = store.get("file:///t.fav").unwrap();
        // "helper" is a fn, not a stage — no lineage entry unless lineage_analysis tracks fns
        // In any case, it has no pipeline membership, so lineage_block would be None
        let block = lineage_block_for_stage(doc, "nonexistent_stage");
        assert!(block.is_none(), "unknown name should return None");
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
