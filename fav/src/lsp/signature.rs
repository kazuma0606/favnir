use crate::lsp::completion::{BUILTIN_FNS, BuiltinFn};
use crate::lsp::hover::position_to_char_offset;
use crate::lsp::protocol::Position;
use crate::middle::checker::LspSymbol;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SignatureHelp {
    pub signatures: Vec<SignatureInformation>,
    #[serde(rename = "activeSignature")]
    pub active_signature: u32,
    #[serde(rename = "activeParameter")]
    pub active_parameter: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignatureInformation {
    pub label: String,
    pub parameters: Vec<ParameterInformation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParameterInformation {
    pub label: String,
}

/// Return signature help for the function call at `pos` in `src`.
/// `symbols` is the list of user-defined symbols from the checked document.
pub fn get_signature_help(
    src: &str,
    pos: Position,
    symbols: &[LspSymbol],
) -> Option<SignatureHelp> {
    let offset = position_to_char_offset(src, pos)?;
    let (open_paren_byte, active_parameter) = find_call_site(src, offset)?;
    let (ns, fname) = function_name_before(src, open_paren_byte)?;

    // Look up in builtin function table first
    if let Some(builtin) = find_builtin(&ns, &fname) {
        return Some(builtin_signature_help(builtin, active_parameter));
    }

    // Fall back to user-defined symbols (no namespace)
    if ns.is_empty() {
        if let Some(sym) = symbols.iter().find(|s| s.name == fname) {
            return Some(SignatureHelp {
                signatures: vec![SignatureInformation {
                    label: format!("{}{}", sym.name, sym.detail),
                    parameters: vec![],
                }],
                active_signature: 0,
                active_parameter: 0,
            });
        }
    }

    None
}

fn builtin_signature_help(builtin: &'static BuiltinFn, active_parameter: u32) -> SignatureHelp {
    let label = format!(
        "{}.{}{}",
        builtin.namespace, builtin.name, builtin.signature
    );
    let parameters = builtin
        .params
        .iter()
        .map(|p| ParameterInformation {
            label: p.to_string(),
        })
        .collect();
    let max_param = builtin.params.len().saturating_sub(1) as u32;
    SignatureHelp {
        signatures: vec![SignatureInformation { label, parameters }],
        active_signature: 0,
        active_parameter: active_parameter.min(max_param),
    }
}

fn find_builtin(ns: &str, name: &str) -> Option<&'static BuiltinFn> {
    BUILTIN_FNS.iter().find(|f| f.namespace == ns && f.name == name)
}

/// Scan backwards from `offset` to find the innermost open `(`.
/// Returns `(byte_offset_of_paren, number_of_commas_at_depth_0)`.
fn find_call_site(src: &str, offset: usize) -> Option<(usize, u32)> {
    let before = &src[..offset.min(src.len())];
    let indexed: Vec<(usize, char)> = before.char_indices().collect();
    let mut depth = 0i32;
    let mut commas: u32 = 0;

    for &(byte_idx, ch) in indexed.iter().rev() {
        match ch {
            ')' | ']' | '}' => depth += 1,
            '(' | '[' | '{' => {
                if depth == 0 {
                    if ch == '(' {
                        return Some((byte_idx, commas));
                    } else {
                        return None; // found '[' or '{' at depth 0 — not a fn call
                    }
                }
                depth -= 1;
            }
            ',' if depth == 0 => commas += 1,
            _ => {}
        }
    }
    None
}

/// Extract `(namespace, function_name)` from the text just before `paren_byte`.
/// Examples:
///   `List.map(`   → `("List", "map")`
///   `double(`     → `("", "double")`
fn function_name_before(src: &str, paren_byte: usize) -> Option<(String, String)> {
    let before = src.get(..paren_byte)?;

    // Scan backwards for the function name (identifier chars)
    let name: String = before
        .chars()
        .rev()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    if name.is_empty() {
        return None;
    }

    // For ASCII identifiers, name.len() == byte length
    let name_byte_len = name.len();
    let name_start = paren_byte.checked_sub(name_byte_len)?;

    // Check if there is a '.' right before the name
    if name_start > 0 && src.as_bytes().get(name_start - 1) == Some(&b'.') {
        let before_dot = src.get(..name_start - 1)?;
        let ns: String = before_dot
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        Some((ns, name))
    } else {
        Some(("".to_string(), name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_call_site_finds_innermost_paren() {
        // "List.map(" — only one open paren, no commas
        let src = "List.map(";
        let (byte, commas) = find_call_site(src, src.len()).expect("call site");
        assert_eq!(commas, 0);
        assert_eq!(src.as_bytes()[byte], b'(');
    }

    #[test]
    fn find_call_site_counts_commas() {
        // "List.map(xs," — one comma at depth 0
        let src = "List.map(xs,";
        let (_, commas) = find_call_site(src, src.len()).expect("call site");
        assert_eq!(commas, 1);
    }

    #[test]
    fn function_name_before_simple() {
        let src = "double(";
        let (ns, name) = function_name_before(src, src.len() - 1).expect("name");
        assert_eq!(ns, "");
        assert_eq!(name, "double");
    }

    #[test]
    fn function_name_before_with_namespace() {
        let src = "List.map(";
        let (ns, name) = function_name_before(src, src.len() - 1).expect("name");
        assert_eq!(ns, "List");
        assert_eq!(name, "map");
    }

    #[test]
    fn get_signature_help_builtin_first_param() {
        let src = "List.map(";
        let help = get_signature_help(
            src,
            Position { line: 0, character: src.len() as u32 },
            &[],
        )
        .expect("signature help");
        assert_eq!(help.active_parameter, 0);
        assert!(help.signatures[0].label.contains("map"));
    }

    #[test]
    fn get_signature_help_builtin_second_param() {
        let src = "List.map(xs,";
        let help = get_signature_help(
            src,
            Position { line: 0, character: src.len() as u32 },
            &[],
        )
        .expect("signature help");
        assert_eq!(help.active_parameter, 1);
    }

    #[test]
    fn get_signature_help_string_split() {
        let src = "String.split(";
        let help = get_signature_help(
            src,
            Position { line: 0, character: src.len() as u32 },
            &[],
        )
        .expect("signature help");
        assert_eq!(help.active_parameter, 0);
        assert!(help.signatures[0].label.contains("split"));
    }
}
