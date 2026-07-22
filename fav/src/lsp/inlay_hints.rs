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
    let mut hints = collect_bind_hints(&doc.source, &doc.type_at);
    // v46.4.0: パイプラインステージの型ヒントも結合
    hints.extend(collect_stage_hints(&doc.source, &doc.type_at));
    // v50.4.0: fn 戻り型ヒント（明示的 `->` なし定義）
    hints.extend(collect_fn_return_hints(&doc.source, &doc.type_at));
    // v50.5.0: pipeline stage IO type hints (Arrow / Trf types only)
    hints.extend(collect_pipeline_type_hints(&doc.source, &doc.type_at));
    hints
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

// v46.4.0: パイプラインステージ型ヒント ──────────────────────────────────────

/// `stage <Name>` 行を走査し、ステージ名の型を行末にヒント表示する。
/// `collect_bind_hints` と対称な実装。テキストスキャン方式のため、
/// コメント・文字列内の `stage ` に誤検出する可能性があるが v46.4.0 スコープ外として許容する。
/// v50.5.0: `Type::Arrow` / `Type::Trf` の stage は `collect_pipeline_type_hints` が担当するため
/// 重複ヒントを避けるためにここではスキップする。
pub(crate) fn collect_stage_hints(
    source: &str,
    type_at: &HashMap<Span, Type>,
) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    let mut byte_offset: usize = 0;
    for (line_idx, line) in source.lines().enumerate() {
        if let Some(rest) = find_stage_prefix(line) {
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
            let name_end_offset = name_start + name_end;
            if let Some(ty) = find_type_at(type_at, name_start, name_end_offset) {
                // v50.5.0: Arrow/Trf are handled by collect_pipeline_type_hints
                if matches!(ty, Type::Arrow(_, _) | Type::Trf(_, _)) {
                    byte_offset += line.len() + 1;
                    continue;
                }
                let col = (prefix_len + name_end) as u32;
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

fn find_stage_prefix(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    trimmed.strip_prefix("stage ").map(|r| r.trim_start())
}

/// v50.4.0: Collect inlay hints for fn return types (no explicit `->` annotation).
///
/// **Known limitations (text-scan approach, same policy as `collect_stage_hints`):**
/// - `fn f(cb: Fn() -> Int)` — lines containing `->` anywhere are skipped, so
///   functions whose *parameter types* include `->` will not show a hint.
/// - Multi-line fn definitions (parameters split across lines) are not supported;
///   only single-line definitions are matched.
/// - CRLF sources: `str::lines()` strips `\r`, so `byte_offset += line.len() + 1`
///   under-counts by 1 per line in CRLF files. Shared limitation with
///   `collect_bind_hints` / `collect_stage_hints`.
///
/// Note: `find_type_at` is module-private, so this function must live in inlay_hints.rs.
pub(crate) fn collect_fn_return_hints(
    source: &str,
    type_at: &HashMap<Span, Type>,
) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    let mut byte_offset: usize = 0;
    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        // Only process single-line `fn` definitions without an explicit return type.
        // Known limitation: lines with `->` anywhere (e.g. in param types) are skipped.
        if !trimmed.starts_with("fn ") || line.contains("->") {
            byte_offset += line.len() + 1;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("fn ") {
            let name_end = rest
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            if name_end == 0 {
                byte_offset += line.len() + 1;
                continue;
            }
            let indent_len = line.len() - trimmed.len();
            let name_start = byte_offset + indent_len + 3; // 3 = "fn ".len()
            let name_end_offset = name_start + name_end;
            // Place hint at the closing `)` of the parameter list
            if let Some(paren_pos) = line.rfind(')') {
                if let Some(ty) = find_type_at(type_at, name_start, name_end_offset) {
                    hints.push(InlayHint {
                        position: Position {
                            line: line_idx as u32,
                            character: paren_pos as u32 + 1,
                        },
                        label: format!(" -> {}", ty.display()),
                        kind: 1,
                    });
                }
            }
        }
        byte_offset += line.len() + 1;
    }
    hints
}

/// v50.5.0: Collect inlay hints for pipeline stage IO types.
/// Only emits hints for stages whose type is `Type::Arrow` or `Type::Trf`
/// (i.e. has a concrete In -> Out structure).
/// `Type::AbstractTrf` and `Type::LinearFn` are intentionally excluded:
/// they represent abstract/polymorphic types whose IO display is less meaningful
/// in pipeline stage context. `collect_stage_hints` handles all remaining scalar/named types.
/// Shares the same text-scan approach and known limitations as `collect_stage_hints`.
pub(crate) fn collect_pipeline_type_hints(
    source: &str,
    type_at: &HashMap<Span, Type>,
) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    let mut byte_offset: usize = 0;
    for (line_idx, line) in source.lines().enumerate() {
        if let Some(rest) = find_stage_prefix(line) {
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
            let name_end_offset = name_start + name_end;
            if let Some(ty) = find_type_at(type_at, name_start, name_end_offset) {
                let label = match ty {
                    Type::Arrow(_, _) | Type::Trf(_, _) => {
                        Some(format!(": {}", ty.display()))
                    }
                    _ => None,
                };
                if let Some(label) = label {
                    let col = (prefix_len + name_end) as u32;
                    hints.push(InlayHint {
                        position: Position {
                            line: line_idx as u32,
                            character: col,
                        },
                        label,
                        kind: 1,
                    });
                }
            }
        }
        byte_offset += line.len() + 1;
    }
    hints
}

/// Matches by byte-offset overlap only; `Span.file` is intentionally ignored.
/// In single-file mode this is safe. In multi-file/project mode a Span from
/// a different file could theoretically match — accepted limitation.
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
