use crate::frontend::lexer::Span;
use crate::lsp::protocol::{Diagnostic, Position, Range};
use crate::middle::checker::TypeError;

pub fn errors_to_diagnostics(errors: &[TypeError], source: &str) -> Vec<Diagnostic> {
    errors
        .iter()
        .map(|err| Diagnostic {
            range: span_to_range(&err.span, source, err.code),
            severity: 1,
            code: err.code.to_string(),
            message: err.message.clone(),
        })
        .collect()
}

fn span_to_range(span: &Span, source: &str, code: &str) -> Range {
    let line_index = span.line.saturating_sub(1) as usize;
    let line_text = source.lines().nth(line_index).unwrap_or("");
    let line_len = line_text.chars().count() as u32;

    let start_line = span.line.saturating_sub(1);
    let start_char = span.col.saturating_sub(1).min(line_len);

    let end_char = if code == "E000" {
        start_char.saturating_add(1).min(line_len)
    } else {
        line_len.max(start_char.saturating_add(1))
    };

    Range {
        start: Position {
            line: start_line,
            character: start_char,
        },
        end: Position {
            line: start_line,
            character: end_char,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::errors_to_diagnostics;
    use crate::frontend::lexer::Span;
    use crate::middle::checker::TypeError;

    #[test]
    fn converts_checker_error_to_zero_origin_diagnostic() {
        let errors = vec![TypeError::new(
            "E018",
            "type mismatch",
            Span::new("test.fav", 0, 0, 2, 5),
        )];
        let diags = errors_to_diagnostics(&errors, "fn main() -> Int {\n    true\n}");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, 1);
        assert_eq!(diags[0].code, "E018");
        assert_eq!(diags[0].range.start.line, 1);
        assert_eq!(diags[0].range.start.character, 4);
    }

    #[test]
    fn parse_error_e000_gets_single_char_range() {
        let errors = vec![TypeError::new(
            "E000",
            "parse error",
            Span::new("test.fav", 0, 0, 1, 4),
        )];
        let diags = errors_to_diagnostics(&errors, "fn x");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].range.start.line, 0);
        assert_eq!(diags[0].range.start.character, 3);
        assert_eq!(diags[0].range.end.character, 4);
    }
}
