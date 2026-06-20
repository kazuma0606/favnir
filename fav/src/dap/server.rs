use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Condvar, Mutex};
use serde_json::Value;
use super::protocol::*;
use super::session::DapSession;

/// DAP メッセージを Content-Length ヘッダー付きで送信
pub fn send_dap_message<W: Write>(writer: &mut W, body: &Value) -> std::io::Result<()> {
    let json = serde_json::to_string(body)?;
    // json.len() は UTF-8 バイト数（DAP 仕様も Content-Length をバイト数で定義）
    let content = format!("Content-Length: {}\r\n\r\n{}", json.len(), json);
    writer.write_all(content.as_bytes())?;
    writer.flush()
}

/// Content-Length ヘッダーを読んで DAP メッセージを受信
pub fn recv_dap_message<R: BufRead + std::io::Read>(reader: &mut R) -> Option<Value> {
    let mut header = String::new();
    let mut content_length: Option<usize> = None; // None = Content-Length ヘッダー未受信
    loop {
        header.clear();
        if reader.read_line(&mut header).ok()? == 0 {
            return None;
        }
        if header == "\r\n" {
            break;
        }
        if header.starts_with("Content-Length:") {
            let len_str = header["Content-Length:".len()..].trim();
            content_length = len_str.parse().ok();
        }
    }
    // Content-Length ヘッダーがなければ受信失敗
    let length = content_length?;
    let mut buf = vec![0u8; length];
    reader.read_exact(&mut buf).ok()?;
    serde_json::from_str(&String::from_utf8(buf).ok()?).ok()
}

/// DAP サーバーのメインループ（接続1本処理）
/// - session: Arc<Mutex<DapSession>> — VM 側と共有（ブレークポイント状態を読み書き）
/// - vm_block: Arc<(Mutex<bool>, Condvar)> — next/continue 時に VM スレッドを再開する
/// - ready_tx: Option<SyncSender<()>> — accept 準備完了を VM スレッドに通知する（起動順序の同期）
pub fn run_dap_server(
    port: u16,
    session: Arc<Mutex<DapSession>>,
    vm_block: Arc<(Mutex<bool>, Condvar)>,
    ready_tx: Option<std::sync::mpsc::SyncSender<()>>,
) -> Result<(), String> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr)
        .map_err(|e| format!("DAP: bind failed on {addr}: {e}"))?;
    eprintln!("[fav DAP] Listening on {addr} ...");

    // accept 前に VM スレッドへ「サーバー準備完了」を通知（HIGH-3 競合回避）
    if let Some(tx) = ready_tx {
        let _ = tx.send(());
    }

    let (stream, peer) = listener
        .accept()
        .map_err(|e| format!("DAP: accept failed: {e}"))?;
    eprintln!("[fav DAP] Connected from {peer}");

    let mut reader = BufReader::new(
        stream
            .try_clone()
            .map_err(|e| format!("DAP: clone stream: {e}"))?,
    );
    let mut writer = stream;

    handle_dap_session(session, vm_block, &mut reader, &mut writer)
}

fn handle_dap_session<R: BufRead + std::io::Read, W: Write>(
    session: Arc<Mutex<DapSession>>,
    vm_block: Arc<(Mutex<bool>, Condvar)>,
    reader: &mut R,
    writer: &mut W,
) -> Result<(), String> {
    loop {
        // recv_dap_message に委譲（Content-Length ヘッダーループ + read_exact）
        let msg = match recv_dap_message(reader) {
            Some(v) => v,
            None => break,
        };
        let req: DapRequest = match serde_json::from_value(msg) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let (resp, pending_events) = {
            let mut sess = session.lock().unwrap_or_else(|e| e.into_inner());
            let resp = handle_dap_request(&mut sess, &req);
            let events = std::mem::take(&mut sess.event_queue);
            (resp, events)
        };

        let resp_val = serde_json::to_value(&resp)
            .map_err(|e| format!("DAP: serialize response: {e}"))?;
        send_dap_message(writer, &resp_val).map_err(|e| e.to_string())?;

        // VM からキューに積まれたイベント（`stopped` 等）を送信
        for event in pending_events {
            send_dap_message(writer, &event).map_err(|e| e.to_string())?;
        }

        // next/continue/stepIn/disconnect — VM スレッドのブロックを解除（HIGH-1）
        if matches!(req.command.as_str(), "next" | "stepIn" | "continue" | "disconnect") {
            let (lock, cvar) = &*vm_block;
            let mut blocked = lock.lock().unwrap_or_else(|e| e.into_inner());
            *blocked = false;
            cvar.notify_one();
        }

        if req.command == "disconnect" {
            break;
        }
    }
    Ok(())
}

fn handle_dap_request(session: &mut DapSession, req: &DapRequest) -> DapResponse {
    let seq = session.next_seq();
    let body = match req.command.as_str() {
        "initialize" => Some(serde_json::json!({
            "supportsConfigurationDoneRequest": true,
            "supportsConditionalBreakpoints": false,
            "supportsStepBack": false,
        })),
        "launch" => Some(serde_json::json!({})),
        "configurationDone" => Some(serde_json::json!({})),
        "threads" => Some(serde_json::json!({
            "threads": [{ "id": 1, "name": "main" }]
        })),
        "stackTrace" => Some(serde_json::json!({
            "stackFrames": [{
                "id": 1,
                "name": session.current_stage.as_deref().unwrap_or("main"),
                "line": session.current_line.unwrap_or(0),
                "column": 0,
                "source": {
                    "path": session.current_source.as_deref().unwrap_or("")
                }
            }],
            "totalFrames": 1
        })),
        "scopes" => Some(serde_json::json!({
            "scopes": [{ "name": "Locals", "variablesReference": 1, "expensive": false }]
        })),
        "variables" => {
            let vars: Vec<_> = session
                .locals
                .iter()
                .map(|(name, ty, val)| {
                    serde_json::json!({
                        "name": name,
                        "type": ty,
                        "value": val,
                        "variablesReference": 0
                    })
                })
                .collect();
            Some(serde_json::json!({ "variables": vars }))
        }
        "setBreakpoints" => {
            if let Some(args) = &req.arguments {
                let path = args
                    .get("source")
                    .and_then(|s| s.get("path"))
                    .and_then(|p| p.as_str())
                    .unwrap_or("");
                let lines: Vec<u32> = args
                    .get("breakpoints")
                    .and_then(|b| b.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|bp| {
                                bp.get("line").and_then(|l| l.as_u64()).map(|l| l as u32)
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                session.set_breakpoints(path, lines.clone());
                let bps: Vec<_> = lines
                    .iter()
                    .map(|&l| serde_json::json!({ "verified": true, "line": l }))
                    .collect();
                Some(serde_json::json!({ "breakpoints": bps }))
            } else {
                Some(serde_json::json!({ "breakpoints": [] }))
            }
        }
        "next" | "stepIn" => {
            session.resume();
            Some(serde_json::json!({}))
        }
        "continue" => {
            session.resume();
            Some(serde_json::json!({ "allThreadsContinued": true }))
        }
        "disconnect" => Some(serde_json::json!({})),
        _ => {
            return DapResponse {
                seq,
                kind: "response".to_string(),
                request_seq: req.seq,
                success: false,
                command: req.command.clone(),
                body: None,
                message: Some(format!("unsupported command: {}", req.command)),
            };
        }
    };
    DapResponse {
        seq,
        kind: "response".to_string(),
        request_seq: req.seq,
        success: true,
        command: req.command.clone(),
        body,
        message: None,
    }
}
