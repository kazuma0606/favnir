#![allow(dead_code)]

pub mod completion;
pub mod definition;
pub mod diagnostics;
pub mod doc_comment;
pub mod document_store;
pub mod hover;
pub mod protocol;

use std::io::{self, BufRead, Write};

use completion::handle_completion;
use definition::handle_definition;
use diagnostics::errors_to_diagnostics;
use document_store::DocumentStore;
use hover::handle_hover;
use protocol::{Position, RpcRequest, RpcResponse};

pub struct LspServer<W: Write> {
    store: DocumentStore,
    writer: W,
    shutdown_requested: bool,
}

impl<W: Write> LspServer<W> {
    pub fn new(writer: W) -> Self {
        Self {
            store: DocumentStore::new(),
            writer,
            shutdown_requested: false,
        }
    }

    pub fn handle(&mut self, request: RpcRequest) -> io::Result<bool> {
        match request.method.as_str() {
            "initialize" => {
                self.write_response(
                    request.id.unwrap_or(serde_json::Value::Null),
                    serde_json::json!({
                        "capabilities": {
                            "textDocumentSync": 1,
                            "hoverProvider": true,
                            "completionProvider": {
                                "triggerCharacters": ["."]
                            },
                            "definitionProvider": true
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
                let result = extract_hover_target(&request.params)
                    .and_then(|(uri, pos)| handle_definition(&self.store, &uri, pos))
                    .map(|location| {
                        serde_json::to_value(location).unwrap_or(serde_json::Value::Null)
                    })
                    .unwrap_or(serde_json::Value::Null);
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
        assert!(text.ends_with("{\"id\":1,\"jsonrpc\":\"2.0\",\"result\":{\"ok\":true}}"));
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
        assert!(text.contains("\"completionProvider\":{\"triggerCharacters\":[\".\"]}"));
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
        assert!(text.contains("\"code\":\"E000\""));
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
        assert!(text.contains("\"code\":\"E000\""));
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
