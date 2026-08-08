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
    // v56.4.0: エフェクト推論 inlay hints（rune 呼び出しから推論）
    hints.extend(collect_effect_hints(&doc.source));
    // v61.2.0: as-pattern 束縛変数の型ヒント
    hints.extend(collect_as_pattern_hints(&doc.source, &doc.type_at));
    // v61.7.0: `_` 型プレースホルダーのヒント
    hints.extend(collect_hole_hints(&doc.source, &doc.type_at));
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

// v61.2.0: as-pattern 束縛変数の型ヒント ──────────────────────────────────────

/// as-pattern 束縛変数の型を inlay hint 表示。
/// テキストスキャン方式（collect_bind_hints / collect_stage_hints と同一方針）。
/// ` as <name>` 形式を行内で検索し、直後の識別子に型ヒントを付与する。
/// `use X as Y` の `as` にも誤検出するが、use 行には type_at エントリが存在しないため実害なし。
/// Known limitation: 1 行に複数の ` as ` が存在する場合は最初のもののみ処理される。
pub(crate) fn collect_as_pattern_hints(
    source: &str,
    type_at: &HashMap<Span, Type>,
) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    let mut byte_offset: usize = 0;
    for (line_idx, line) in source.lines().enumerate() {
        if let Some(as_pos) = line.find(" as ") {
            let rest = &line[as_pos + 4..];
            let trimmed = rest.trim_start();
            let trim_delta = rest.len() - trimmed.len();
            let name_end = trimmed
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(trimmed.len());
            if name_end > 0 {
                let name = &trimmed[..name_end];
                if name != "_" {
                    let prefix_len = as_pos + 4 + trim_delta;
                    let name_start = byte_offset + prefix_len;
                    let name_end_offset = name_start + name_end;
                    if let Some(ty) = find_type_at(type_at, name_start, name_end_offset) {
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
            }
        }
        byte_offset += line.len() + 1;
    }
    hints
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

// v56.4.0: エフェクト推論 inlay hints ─────────────────────────────────────────

/// Rune 名前空間プレフィックス → !Effect タグのマッピング。
/// 規約: Favnir の公式 Rune は小文字（`kafka.`）だが、ユーザー定義や
/// レガシーコードで大文字バリアントが使われる場合に備えて両方登録する。
static EFFECT_PREFIXES: &[(&str, &str)] = &[
    ("kafka.",     "!Kafka"),
    ("Kafka.",     "!Kafka"),
    ("snowflake.", "!Snowflake"),
    ("Snowflake.", "!Snowflake"),
    ("db.",        "!DB"),
    ("DB.",        "!DB"),
    ("Postgres.",  "!DB"),
    ("postgres.",  "!DB"),
    ("s3.",        "!S3"),
    ("S3.",        "!S3"),
    ("AWS.",       "!AWS"),
    ("http.",      "!Http"),
    ("HTTP.",      "!Http"),
    ("Http.",      "!Http"),
    ("slack.",     "!Slack"),
    ("Slack.",     "!Slack"),
    ("llm.",       "!Llm"),
    ("Llm.",       "!Llm"),
    ("bigquery.",  "!BigQuery"),
    ("BigQuery.",  "!BigQuery"),
    ("redis.",     "!Redis"),
    ("Redis.",     "!Redis"),
    ("dynamodb.",  "!DynamoDB"),
    ("DynamoDB.",  "!DynamoDB"),
    ("sqs.",       "!Sqs"),
    ("Sqs.",       "!Sqs"),
    ("kinesis.",   "!Kinesis"),
    ("Kinesis.",   "!Kinesis"),
];

/// `fn` 定義の本体テキストから呼ばれている rune 名前空間を分析し、
/// `!Effect` タグの Sorted リストを返す（重複なし）。
///
/// **Known limitation (text-scan approach):** コメントや文字列リテラル内の
/// `kafka.` 等にも反応するため、過近似（false positive）が生じる可能性がある。
/// 実コードで rune を呼ばない fn が「呼んでいる」と推論されることがあるが、
/// LSP inlay hint 目的では許容範囲とする（既存の `collect_bind_hints` 等と同方針）。
pub fn infer_effect_tags(body_text: &str) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    for (prefix, tag) in EFFECT_PREFIXES {
        if body_text.contains(prefix) {
            seen.insert(tag.to_string());
        }
    }
    seen.into_iter().collect()
}

/// v56.4.0: `fn` 定義ごとに推論エフェクトを LSP inlay hint として収集する。
/// エフェクトが 1 件以上ある場合のみ hint を生成する。
/// `/*!Kafka !Snowflake*/` 形式で fn 宣言行の末尾（`{` の直前）に表示。
///
/// **Known limitations (text-scan approach, same policy as `collect_stage_hints`):**
/// - コメント・文字列リテラル内の rune プレフィックスにも反応する（過近似）。
/// - `fn_span.line` はパーサーが設定した 1-based 行番号を使用する。
/// - CRLF ソース: `str::lines()` が `\r` を除去するため CRLF ファイルでは列がずれる可能性がある。
pub(crate) fn collect_effect_hints(source: &str) -> Vec<InlayHint> {
    use crate::ast::Item;
    use crate::frontend::parser::Parser;

    let program = match Parser::parse_str(source, "<lsp>") {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let lines: Vec<&str> = source.lines().collect();
    let mut hints = Vec::new();

    for item in &program.items {
        let Item::FnDef(fd) = item else { continue };
        let fn_span = &fd.span;
        // fn_span.line == 0 はパーサーのバグや未初期化を示す — スキップ
        if fn_span.line == 0 {
            continue;
        }
        let body_start = fd.body.span.start;
        let body_end   = fd.body.span.end;
        if body_start >= body_end || body_end > source.len() {
            continue;
        }
        // UTF-8 マルチバイト文字を含む場合に備えて get() でパニック回避
        let body_text = match source.get(body_start..body_end) {
            Some(t) => t,
            None => continue,
        };
        let effects = infer_effect_tags(body_text);
        if effects.is_empty() {
            continue;
        }
        // fn 宣言行（0-based）に hint を置く
        let line_idx = fn_span.line.saturating_sub(1) as usize;
        let line = match lines.get(line_idx) {
            Some(l) => l,
            None => continue,
        };
        // `{` の最後の位置（なければ行末）の直前に挿入。
        // rfind を使うことでパラメータ型内のレコード型 `{ ... }` との誤検出を避ける。
        let col = if let Some(brace) = line.rfind('{') {
            brace as u32
        } else {
            line.len() as u32
        };
        let label = format!("/*!{}*/", effects.join(" "));
        hints.push(InlayHint {
            position: Position { line: line_idx as u32, character: col },
            label,
            kind: 1,
        });
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

/// v61.7.0: `_` 型プレースホルダーの位置に inlay hint を追加。
/// 関数の戻り型または引数型に TypeExpr::Hole があれば、type_at から推論型を表示。
/// type_at に該当 span がない場合はヒントを出力しない（Unknown 型を表示しても有益でないため）。
pub(crate) fn collect_hole_hints(
    source: &str,
    type_at: &HashMap<Span, Type>,
) -> Vec<InlayHint> {
    use crate::ast::{Item, TypeExpr};
    use crate::frontend::parser::Parser;
    let mut hints = Vec::new();
    let prog = match Parser::parse_str(source, "<lsp>") {
        Ok(p) => p,
        Err(_) => return hints,
    };
    for item in &prog.items {
        if let Item::FnDef(fd) = item {
            // Return type hole
            if let Some(TypeExpr::Hole(span)) = &fd.return_ty {
                if let Some(ty) = find_type_at(type_at, span.start, span.end) {
                    if !matches!(ty, Type::Unknown) {
                        // Position at span.end (after `_`) for correct LSP insertion point
                        let line = source[..span.end].lines().count().saturating_sub(1);
                        let col = source[..span.end]
                            .rfind('\n')
                            .map(|p| span.end - p - 1)
                            .unwrap_or(span.end);
                        hints.push(InlayHint {
                            position: Position {
                                line: line as u32,
                                character: col as u32,
                            },
                            label: format!(": {}", ty.display()),
                            kind: 1,
                        });
                    }
                }
            }
            // Parameter type holes
            for param in &fd.params {
                if let TypeExpr::Hole(span) = &param.ty {
                    if let Some(ty) = find_type_at(type_at, span.start, span.end) {
                        if !matches!(ty, Type::Unknown) {
                            // Position at span.end (after `_`) for correct LSP insertion point
                            let line = source[..span.end].lines().count().saturating_sub(1);
                            let col = source[..span.end]
                                .rfind('\n')
                                .map(|p| span.end - p - 1)
                                .unwrap_or(span.end);
                            hints.push(InlayHint {
                                position: Position {
                                    line: line as u32,
                                    character: col as u32,
                                },
                                label: format!(": {}", ty.display()),
                                kind: 1,
                            });
                        }
                    }
                }
            }
        }
    }
    hints
}
