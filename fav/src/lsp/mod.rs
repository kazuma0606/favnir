#![allow(dead_code)]

pub mod code_action;
pub mod completion;
pub mod definition;
pub mod diagnostics;
pub mod doc_comment;
pub mod document_store;
pub mod hover;
pub mod protocol;
pub mod references;
pub mod rename;
pub mod signature;

use std::io::{self, BufRead, Write};

use code_action::handle_code_action;
use completion::handle_completion;
use definition::{handle_definition, handle_rune_definition};
use diagnostics::errors_to_diagnostics;
use document_store::DocumentStore;
use hover::handle_hover;
use protocol::{Position, Range, RpcRequest, RpcResponse};
use references::handle_references;
use rename::handle_rename;
use signature::get_signature_help;

pub struct LspServer<W: Write> {
    store: DocumentStore,
    writer: W,
    shutdown_requested: bool,
    workspace_root: Option<String>,
}

impl<W: Write> LspServer<W> {
    pub fn new(writer: W) -> Self {
        Self {
            store: DocumentStore::new(),
            writer,
            workspace_root: None,
            shutdown_requested: false,
        }
    }

    pub fn handle(&mut self, request: RpcRequest) -> io::Result<bool> {
        match request.method.as_str() {
            "initialize" => {
                self.workspace_root = request
                    .params
                    .get("rootUri")
                    .and_then(|v| v.as_str())
                    .map(|s| {
                        // Convert file:///path to filesystem path
                        let s = s.trim_start_matches("file:///");
                        #[cfg(windows)]
                        {
                            s.replace('/', "\\")
                        }
                        #[cfg(not(windows))]
                        {
                            format!("/{}", s)
                        }
                    });
                self.write_response(
                    request.id.unwrap_or(serde_json::Value::Null),
                    serde_json::json!({
                        "capabilities": {
                            "textDocumentSync": 1,
                            "hoverProvider": true,
                            "completionProvider": {
                                "triggerCharacters": [".", "\""]
                            },
                            "definitionProvider": true,
                            "signatureHelpProvider": {
                                "triggerCharacters": ["(", ","]
                            },
                            "codeActionProvider": true,
                            "renameProvider": true,
                            "referencesProvider": true
                        }
                    }),
                )?;
                Ok(false)
            }
            "initialized" => Ok(false),
            "textDocument/didOpen" => {
                if let Some((uri, text)) = extract_open_doc(&request.params) {
                    self.store.open_or_change(uri.clone(), text);
                    self.publish_diagnostics(&uri)?;
                }
                Ok(false)
            }
            "textDocument/didChange" => {
                if let Some((uri, text)) = extract_changed_doc(&request.params) {
                    self.store.open_or_change(uri.clone(), text);
                    self.publish_diagnostics(&uri)?;
                }
                Ok(false)
            }
            "textDocument/hover" => {
                let result = extract_hover_target(&request.params)
                    .and_then(|(uri, pos)| handle_hover(&self.store, &uri, pos))
                    .map(|hover| serde_json::to_value(hover).unwrap_or(serde_json::Value::Null))
                    .unwrap_or(serde_json::Value::Null);
                self.write_response(request.id.unwrap_or(serde_json::Value::Null), result)?;
                Ok(false)
            }
            "textDocument/completion" => {
                let result = extract_completion_target(&request.params)
                    .map(|(uri, pos, trigger)| handle_completion(&self.store, &uri, pos, trigger))
                    .and_then(|items| serde_json::to_value(items).ok())
                    .unwrap_or_else(|| serde_json::json!([]));
                self.write_response(request.id.unwrap_or(serde_json::Value::Null), result)?;
                Ok(false)
            }
            "textDocument/definition" => {
                let workspace_root = self.workspace_root.clone();
                let result = extract_hover_target(&request.params)
                    .and_then(|(uri, pos)| {
                        handle_definition(&self.store, &uri, pos).or_else(|| {
                            let doc = self.store.get(&uri)?;
                            let offset =
                                crate::lsp::hover::position_to_char_offset(&doc.source, pos)?;
                            workspace_root
                                .as_deref()
                                .and_then(|root| handle_rune_definition(&doc.source, offset, root))
                        })
                    })
                    .map(|location| {
                        serde_json::to_value(location).unwrap_or(serde_json::Value::Null)
                    })
                    .unwrap_or(serde_json::Value::Null);
                self.write_response(request.id.unwrap_or(serde_json::Value::Null), result)?;
                Ok(false)
            }
            "textDocument/signatureHelp" => {
                let result = extract_hover_target(&request.params)
                    .and_then(|(uri, pos)| {
                        let doc = self.store.get(&uri)?;
                        get_signature_help(&doc.source, pos, &doc.symbols)
                    })
                    .and_then(|help| serde_json::to_value(help).ok())
                    .unwrap_or(serde_json::Value::Null);
                self.write_response(request.id.unwrap_or(serde_json::Value::Null), result)?;
                Ok(false)
            }
            "textDocument/codeAction" => {
                let result = extract_code_action_params(&request.params)
                    .map(|(uri, range)| handle_code_action(&self.store, &uri, range))
                    .and_then(|actions| serde_json::to_value(actions).ok())
                    .unwrap_or_else(|| serde_json::json!([]));
                self.write_response(request.id.unwrap_or(serde_json::Value::Null), result)?;
                Ok(false)
            }
            "textDocument/rename" => {
                let result = extract_rename_params(&request.params)
                    .and_then(|(uri, pos, new_name)| {
                        handle_rename(&self.store, &uri, pos, &new_name)
                    })
                    .and_then(|edit| serde_json::to_value(edit).ok())
                    .unwrap_or(serde_json::Value::Null);
                self.write_response(request.id.unwrap_or(serde_json::Value::Null), result)?;
                Ok(false)
            }
            "textDocument/references" => {
                let result = extract_hover_target(&request.params)
                    .map(|(uri, pos)| handle_references(&self.store, &uri, pos))
                    .and_then(|locs| serde_json::to_value(locs).ok())
                    .unwrap_or_else(|| serde_json::json!([]));
                self.write_response(request.id.unwrap_or(serde_json::Value::Null), result)?;
                Ok(false)
            }
            "shutdown" => {
                self.shutdown_requested = true;
                self.write_response(
                    request.id.unwrap_or(serde_json::Value::Null),
                    serde_json::Value::Null,
                )?;
                Ok(false)
            }
            "exit" => Ok(true),
            _ => {
                if request.id.is_some() {
                    self.write_response(
                        request.id.unwrap_or(serde_json::Value::Null),
                        serde_json::Value::Null,
                    )?;
                }
                Ok(false)
            }
        }
    }

    fn write_response(
        &mut self,
        id: serde_json::Value,
        result: serde_json::Value,
    ) -> io::Result<()> {
        write_json_message(
            &mut self.writer,
            &serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            }),
        )
    }

    fn publish_diagnostics(&mut self, uri: &str) -> io::Result<()> {
        if let Some(doc) = self.store.get(uri) {
            let diagnostics = errors_to_diagnostics(&doc.errors, &doc.source);
            write_json_message(
                &mut self.writer,
                &serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/publishDiagnostics",
                    "params": {
                        "uri": uri,
                        "diagnostics": diagnostics
                    }
                }),
            )?;
        }
        Ok(())
    }
}

pub fn run_lsp_server(port: Option<u16>) {
    if let Some(port) = port {
        let mut stderr = io::stderr().lock();
        let _ = writeln!(
            stderr,
            "fav lsp TCP mode is not implemented yet (requested TCP port {port})"
        );
        return;
    }

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = io::BufReader::new(stdin.lock());
    let mut writer = stdout.lock();
    let _ = run_lsp_loop(&mut reader, &mut writer);
}

pub fn read_message(reader: &mut impl BufRead) -> Option<RpcRequest> {
    let mut content_length: Option<usize> = None;
    let mut line = String::new();

    loop {
        line.clear();
        let bytes = reader.read_line(&mut line).ok()?;
        if bytes == 0 {
            return None;
        }

        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }

        if let Some(rest) = trimmed.strip_prefix("Content-Length:") {
            let parsed = rest.trim().parse::<usize>().ok()?;
            content_length = Some(parsed);
        }
    }

    let len = content_length?;
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body).ok()?;
    serde_json::from_slice::<RpcRequest>(&body).ok()
}

pub fn write_message(writer: &mut impl Write, response: &RpcResponse) -> io::Result<()> {
    let body = serde_json::to_value(response)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    write_json_message(writer, &body)
}

fn write_json_message(writer: &mut impl Write, value: &serde_json::Value) -> io::Result<()> {
    let body =
        serde_json::to_vec(value).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}

fn extract_open_doc(params: &serde_json::Value) -> Option<(String, String)> {
    let doc = params.get("textDocument")?;
    Some((
        doc.get("uri")?.as_str()?.to_string(),
        doc.get("text")?.as_str()?.to_string(),
    ))
}

fn extract_changed_doc(params: &serde_json::Value) -> Option<(String, String)> {
    let uri = params
        .get("textDocument")?
        .get("uri")?
        .as_str()?
        .to_string();
    let text = params
        .get("contentChanges")?
        .as_array()?
        .last()?
        .get("text")?
        .as_str()?
        .to_string();
    Some((uri, text))
}

fn extract_hover_target(params: &serde_json::Value) -> Option<(String, Position)> {
    let uri = params
        .get("textDocument")?
        .get("uri")?
        .as_str()?
        .to_string();
    let pos = serde_json::from_value::<Position>(params.get("position")?.clone()).ok()?;
    Some((uri, pos))
}

fn extract_completion_target(
    params: &serde_json::Value,
) -> Option<(String, Position, Option<String>)> {
    let (uri, pos) = extract_hover_target(params)?;
    let trigger = params
        .get("context")
        .and_then(|context| context.get("triggerCharacter"))
        .and_then(|value| value.as_str())
        .map(|value| value.to_string());
    Some((uri, pos, trigger))
}

fn extract_code_action_params(params: &serde_json::Value) -> Option<(String, Range)> {
    let uri = params.get("textDocument")?.get("uri")?.as_str()?.to_string();
    let range = serde_json::from_value::<Range>(params.get("range")?.clone()).ok()?;
    Some((uri, range))
}

fn extract_rename_params(params: &serde_json::Value) -> Option<(String, Position, String)> {
    let uri = params.get("textDocument")?.get("uri")?.as_str()?.to_string();
    let pos = serde_json::from_value::<Position>(params.get("position")?.clone()).ok()?;
    let new_name = params.get("newName")?.as_str()?.to_string();
    Some((uri, pos, new_name))
}

fn run_lsp_loop(reader: &mut impl BufRead, writer: &mut impl Write) -> io::Result<()> {
    let mut server = LspServer::new(writer);
    while let Some(request) = read_message(reader) {
        if server.handle(request)? {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{LspServer, read_message, run_lsp_loop, write_message};
    use crate::lsp::protocol::RpcResponse;

    #[test]
    fn read_message_parses_content_length_frame() {
        let body = br#"{"id":1,"method":"initialize","params":{"rootUri":null}}"#;
        let raw = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
        let mut bytes = raw;
        bytes.extend_from_slice(body);
        let mut reader = std::io::BufReader::new(&bytes[..]);
        let msg = read_message(&mut reader).expect("message");
        assert_eq!(msg.method, "initialize");
        assert_eq!(msg.id, Some(serde_json::json!(1)));
    }

    #[test]
    fn write_message_emits_content_length_frame() {
        let response = RpcResponse {
            jsonrpc: "2.0",
            id: serde_json::json!(1),
            result: serde_json::json!({ "ok": true }),
        };
        let mut out = Vec::new();
        write_message(&mut out, &response).expect("write");
        let text = String::from_utf8(out).expect("utf8");
        assert!(text.starts_with("Content-Length: "));
        assert!(text.contains("\r\n\r\n"));
        assert!(text.ends_with("{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"ok\":true}}"));
    }

    #[test]
    fn initialize_returns_capabilities() {
        let mut out = Vec::new();
        let mut server = LspServer::new(&mut out);
        server
            .handle(crate::lsp::protocol::RpcRequest {
                id: Some(serde_json::json!(1)),
                method: "initialize".to_string(),
                params: serde_json::json!({}),
            })
            .expect("handle");
        let text = String::from_utf8(out).expect("utf8");
        assert!(text.contains("\"hoverProvider\":true"));
        assert!(text.contains("\"definitionProvider\":true"));
        assert!(text.contains("\"completionProvider\":{\"triggerCharacters\":[\".\",\"\\\"\"]}"));
    }

    #[test]
    fn did_open_publishes_diagnostics() {
        let mut out = Vec::new();
        let mut server = LspServer::new(&mut out);
        server
            .handle(crate::lsp::protocol::RpcRequest {
                id: None,
                method: "textDocument/didOpen".to_string(),
                params: serde_json::json!({
                    "textDocument": {
                        "uri": "file:///broken.fav",
                        "text": "fn main("
                    }
                }),
            })
            .expect("handle");
        let text = String::from_utf8(out).expect("utf8");
        assert!(text.contains("textDocument/publishDiagnostics"));
        assert!(text.contains("\"code\":\"E0500\""));
    }

    #[test]
    fn run_lsp_loop_processes_initialize_and_exit() {
        let init = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        })
        .to_string();
        let exit = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "exit",
            "params": {}
        })
        .to_string();
        let input = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            init.len(),
            init,
            exit.len(),
            exit
        );
        let mut reader = std::io::BufReader::new(input.as_bytes());
        let mut out = Vec::new();
        run_lsp_loop(&mut reader, &mut out).expect("loop");
        let text = String::from_utf8(out).expect("utf8");
        assert!(text.contains("\"hoverProvider\":true"));
    }

    #[test]
    fn run_lsp_loop_processes_did_change_and_hover() {
        let open = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///main.fav",
                    "text": "fn main() -> Int { 42 }"
                }
            }
        })
        .to_string();
        let hover = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": "file:///main.fav" },
                "position": { "line": 0, "character": 19 }
            }
        })
        .to_string();
        let exit = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "exit",
            "params": {}
        })
        .to_string();
        let input = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            open.len(),
            open,
            hover.len(),
            hover,
            exit.len(),
            exit
        );
        let mut reader = std::io::BufReader::new(input.as_bytes());
        let mut out = Vec::new();
        run_lsp_loop(&mut reader, &mut out).expect("loop");
        let text = String::from_utf8(out).expect("utf8");
        assert!(text.contains("textDocument/publishDiagnostics"));
        assert!(text.contains("```favnir\\nInt\\n```"));
    }

    #[test]
    fn run_lsp_loop_recovers_from_parse_error_and_then_hovers() {
        let open_bad = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///main.fav",
                    "text": "fn main("
                }
            }
        })
        .to_string();
        let change_good = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": { "uri": "file:///main.fav" },
                "contentChanges": [
                    { "text": "fn main() -> Int { 42 }" }
                ]
            }
        })
        .to_string();
        let hover = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": "file:///main.fav" },
                "position": { "line": 0, "character": 19 }
            }
        })
        .to_string();
        let exit = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "exit",
            "params": {}
        })
        .to_string();
        let input = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            open_bad.len(),
            open_bad,
            change_good.len(),
            change_good,
            hover.len(),
            hover,
            exit.len(),
            exit
        );
        let mut reader = std::io::BufReader::new(input.as_bytes());
        let mut out = Vec::new();
        run_lsp_loop(&mut reader, &mut out).expect("loop");
        let text = String::from_utf8(out).expect("utf8");
        assert!(text.contains("\"code\":\"E0500\""));
        assert!(text.contains("\"diagnostics\":[]"));
        assert!(text.contains("```favnir\\nInt\\n```"));
    }

    #[test]
    fn completion_request_returns_items() {
        let mut out = Vec::new();
        let mut server = LspServer::new(&mut out);
        server
            .handle(crate::lsp::protocol::RpcRequest {
                id: None,
                method: "textDocument/didOpen".to_string(),
                params: serde_json::json!({
                    "textDocument": {
                        "uri": "file:///main.fav",
                        "text": "fn double(n: Int) -> Int = n * 2\nfn main() -> Int = do"
                    }
                }),
            })
            .expect("open");
        server
            .handle(crate::lsp::protocol::RpcRequest {
                id: Some(serde_json::json!(9)),
                method: "textDocument/completion".to_string(),
                params: serde_json::json!({
                    "textDocument": { "uri": "file:///main.fav" },
                    "position": { "line": 1, "character": 20 }
                }),
            })
            .expect("completion");
        let text = String::from_utf8(out).expect("utf8");
        assert!(text.contains("\"label\":\"double\""));
        assert!(text.contains("\"label\":\"match\""));
    }

    #[test]
    fn definition_request_returns_location() {
        let mut out = Vec::new();
        let mut server = LspServer::new(&mut out);
        server
            .handle(crate::lsp::protocol::RpcRequest {
                id: None,
                method: "textDocument/didOpen".to_string(),
                params: serde_json::json!({
                    "textDocument": {
                        "uri": "file:///main.fav",
                        "text": "fn double(n: Int) -> Int = n * 2\nfn main() -> Int = double(21)"
                    }
                }),
            })
            .expect("open");
        server
            .handle(crate::lsp::protocol::RpcRequest {
                id: Some(serde_json::json!(10)),
                method: "textDocument/definition".to_string(),
                params: serde_json::json!({
                    "textDocument": { "uri": "file:///main.fav" },
                    "position": { "line": 1, "character": 20 }
                }),
            })
            .expect("definition");
        let text = String::from_utf8(out).expect("utf8");
        assert!(text.contains("\"uri\":\"file:///main.fav\""));
        assert!(text.contains("\"line\":0"));
    }
}

// ── v9110_tests (v9.11.0) — LSP module completion + rune completion + signature help ──
#[cfg(test)]
mod v9110_tests {
    use super::LspServer;
    use crate::lsp::completion::{BUILTIN_FNS, BUILTIN_NAMESPACES, module_completions};
    use crate::lsp::signature::get_signature_help;
    use crate::lsp::protocol::Position;

    // F-1a: List.map / filter appear in module completions
    #[test]
    fn module_completion_list_contains_map_and_filter() {
        let items = module_completions("List");
        assert!(
            items.iter().any(|i| i.label == "map"),
            "expected 'map' in List completions"
        );
        assert!(
            items.iter().any(|i| i.label == "filter"),
            "expected 'filter' in List completions"
        );
    }

    // F-1b: String.split / trim appear in module completions
    #[test]
    fn module_completion_string_contains_split_and_trim() {
        let items = module_completions("String");
        assert!(
            items.iter().any(|i| i.label == "split"),
            "expected 'split' in String completions"
        );
        assert!(
            items.iter().any(|i| i.label == "trim"),
            "expected 'trim' in String completions"
        );
    }

    // F-1c: Rune completion returns all known rune names via LspServer
    #[test]
    fn rune_completion_returns_known_runes() {
        let mut out = Vec::new();
        let mut server = LspServer::new(&mut out);
        server
            .handle(crate::lsp::protocol::RpcRequest {
                id: None,
                method: "textDocument/didOpen".to_string(),
                params: serde_json::json!({
                    "textDocument": {
                        "uri": "file:///main.fav",
                        "text": "import rune \""
                    }
                }),
            })
            .expect("open");
        server
            .handle(crate::lsp::protocol::RpcRequest {
                id: Some(serde_json::json!(20)),
                method: "textDocument/completion".to_string(),
                params: serde_json::json!({
                    "textDocument": { "uri": "file:///main.fav" },
                    "position": { "line": 0, "character": 13 }
                }),
            })
            .expect("completion");
        let text = String::from_utf8(out).expect("utf8");
        assert!(text.contains("\"label\":\"http\""), "expected http rune in completion: {}", &text[..200.min(text.len())]);
        assert!(text.contains("\"label\":\"csv\""), "expected csv rune in completion");
        assert!(text.contains("\"label\":\"json\""), "expected json rune in completion");
    }

    // F-1d: Signature help for List.map( returns activeParameter 0
    #[test]
    fn signature_help_builtin_first_param() {
        let src = "List.map(";
        let help = get_signature_help(
            src,
            Position { line: 0, character: src.len() as u32 },
            &[],
        )
        .expect("expected signature help for List.map(");
        assert_eq!(help.active_parameter, 0, "first arg → activeParameter 0");
        assert!(
            help.signatures[0].label.contains("map"),
            "signature label should contain 'map'"
        );
    }

    // F-1e: Signature help for List.map(xs, → activeParameter 1
    #[test]
    fn signature_help_builtin_second_param() {
        let src = "List.map(xs,";
        let help = get_signature_help(
            src,
            Position { line: 0, character: src.len() as u32 },
            &[],
        )
        .expect("expected signature help for List.map(xs,");
        assert_eq!(help.active_parameter, 1, "after comma → activeParameter 1");
    }

    // Extra: signatureHelp capability registered in initialize response
    #[test]
    fn initialize_reports_signature_help_provider() {
        let mut out = Vec::new();
        let mut server = LspServer::new(&mut out);
        server
            .handle(crate::lsp::protocol::RpcRequest {
                id: Some(serde_json::json!(1)),
                method: "initialize".to_string(),
                params: serde_json::json!({}),
            })
            .expect("handle");
        let text = String::from_utf8(out).expect("utf8");
        assert!(
            text.contains("signatureHelpProvider"),
            "expected signatureHelpProvider in capabilities"
        );
    }

    // Extra: BUILTIN_NAMESPACES covers key namespaces
    #[test]
    fn builtin_namespaces_includes_list_and_string() {
        assert!(BUILTIN_NAMESPACES.contains(&"List"));
        assert!(BUILTIN_NAMESPACES.contains(&"String"));
        assert!(BUILTIN_NAMESPACES.contains(&"Map"));
    }

    // Extra: BUILTIN_FNS table is non-empty and has correct structure
    #[test]
    fn builtin_fns_table_has_entries() {
        assert!(!BUILTIN_FNS.is_empty());
        let list_map = BUILTIN_FNS.iter().find(|f| f.namespace == "List" && f.name == "map");
        assert!(list_map.is_some(), "expected List.map in BUILTIN_FNS");
        let map = list_map.unwrap();
        assert!(!map.params.is_empty(), "List.map should have params");
    }
}

// ── v9120_tests (v9.12.0) — LSP: Rune definition jump + workspace_root ──────
#[cfg(test)]
mod v9120_tests {
    use super::LspServer;
    use crate::lsp::definition::handle_rune_definition;
    use crate::lsp::completion::KNOWN_RUNES;

    // Rune definition jump: unknown namespace → None
    #[test]
    fn rune_definition_unknown_namespace_returns_none() {
        let result = handle_rune_definition("mylib.foo(", 9, "/workspace");
        assert!(result.is_none(), "unknown namespace should return None");
    }

    // Rune definition jump: non-rune pattern (no dot) → None
    #[test]
    fn rune_definition_no_dot_returns_none() {
        let result = handle_rune_definition("foo(", 3, "/workspace");
        assert!(result.is_none(), "no dot pattern should return None");
    }

    // KNOWN_RUNES includes http and csv
    #[test]
    fn known_runes_includes_http_and_csv() {
        assert!(KNOWN_RUNES.iter().any(|(n, _)| *n == "http"), "expected http in KNOWN_RUNES");
        assert!(KNOWN_RUNES.iter().any(|(n, _)| *n == "csv"), "expected csv in KNOWN_RUNES");
    }

    // workspace_root is stored after initialize with rootUri
    #[test]
    fn initialize_stores_workspace_root() {
        let mut out = Vec::new();
        let mut server = LspServer::new(&mut out);
        server
            .handle(crate::lsp::protocol::RpcRequest {
                id: Some(serde_json::json!(1)),
                method: "initialize".to_string(),
                params: serde_json::json!({ "rootUri": "file:///c/Users/test/project" }),
            })
            .expect("handle");
        assert!(server.workspace_root.is_some(), "workspace_root should be set after initialize");
    }

    // definition handler falls through to rune jump for known namespace
    #[test]
    fn definition_handler_attempts_rune_jump_for_known_ns() {
        let mut out = Vec::new();
        let mut server = LspServer::new(&mut out);
        // Open a file with http.get reference
        server
            .handle(crate::lsp::protocol::RpcRequest {
                id: None,
                method: "textDocument/didOpen".to_string(),
                params: serde_json::json!({
                    "textDocument": {
                        "uri": "file:///main.fav",
                        "text": "http.get("
                    }
                }),
            })
            .expect("open");
        // Request definition at end of "http.get" — should not panic even if file not found
        server
            .handle(crate::lsp::protocol::RpcRequest {
                id: Some(serde_json::json!(2)),
                method: "textDocument/definition".to_string(),
                params: serde_json::json!({
                    "textDocument": { "uri": "file:///main.fav" },
                    "position": { "line": 0, "character": 8 }
                }),
            })
            .expect("definition request should not panic");
    }
}
