# v21.1.0 実装計画 — DAP デバッガー

## 実装順序

```
T1（fav/src/dap/ モジュール新規作成）  ← 最初（他と独立）
T2（fav/src/backend/vm.rs — debug フック挿入）  ← T1 完了後
T3（fav/src/driver.rs — cmd_dap / cmd_run_debug）  ← T2 完了後
T4（fav/Cargo.toml バージョン更新）    ← T1 と並列可
T5（CHANGELOG.md + site/ MDX）          ← T3 完了後
T6（fav/src/driver.rs — v211000_tests） ← T1 完了後
```

**Rust コードへの変更は T1〜T4 と T6。**
T5 はドキュメントのみ。

---

## T1: `fav/src/dap/` — DAP モジュール新規作成

### T1-1: `fav/src/dap/protocol.rs`

DAP JSON-RPC のプロトコル型定義。

```rust
use serde::{Deserialize, Serialize};

/// DAP メッセージの共通ヘッダー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapMessage {
    pub seq:  u64,
    #[serde(rename = "type")]
    pub kind: String,  // "request" | "response" | "event"
}

/// DAP リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapRequest {
    pub seq:      u64,
    #[serde(rename = "type")]
    pub kind:     String,  // "request"
    pub command:  String,
    pub arguments: Option<serde_json::Value>,
}

/// DAP レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapResponse {
    pub seq:         u64,
    #[serde(rename = "type")]
    pub kind:        String,  // "response"
    pub request_seq: u64,
    pub success:     bool,
    pub command:     String,
    pub body:        Option<serde_json::Value>,
    pub message:     Option<String>,
}

/// DAP イベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapEvent {
    pub seq:   u64,
    #[serde(rename = "type")]
    pub kind:  String,  // "event"
    pub event: String,
    pub body:  Option<serde_json::Value>,
}

/// ブレークポイント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapBreakpoint {
    pub source: String,
    pub line:   u32,
    pub verified: bool,
}

/// `initialize` リクエストの arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeArguments {
    #[serde(rename = "adapterID")]
    pub adapter_id: String,
    #[serde(rename = "linesStartAt1")]
    pub lines_start_at1: Option<bool>,
}

/// `setBreakpoints` リクエストの arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetBreakpointsArguments {
    pub source: DapSource,
    pub breakpoints: Option<Vec<SourceBreakpoint>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapSource {
    pub name: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceBreakpoint {
    pub line: u32,
    pub condition: Option<String>,
}

/// `launch` リクエストの arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchArguments {
    pub program: String,
    #[serde(rename = "noDebug")]
    pub no_debug: Option<bool>,
}
```

### T1-2: `fav/src/dap/session.rs`

デバッグセッション管理。

```rust
use std::collections::HashMap;
use super::protocol::DapBreakpoint;

#[derive(Debug, Default)]
pub struct DapSession {
    pub seq:            u64,
    pub breakpoints:    HashMap<String, Vec<DapBreakpoint>>,  // source path → breakpoints
    pub stopped:        bool,
    pub stop_reason:    Option<String>,
    pub current_line:   Option<u32>,
    pub current_source: Option<String>,
    pub current_stage:  Option<String>,  // stackTrace.name に使うステージ名
    pub locals:         Vec<(String, String, String)>,  // (name, type, value)
    pub event_queue:    Vec<serde_json::Value>,  // VM → クライアントへのプッシュイベント
}

impl DapSession {
    pub fn new() -> Self { Self::default() }

    pub fn next_seq(&mut self) -> u64 {
        self.seq += 1;
        self.seq
    }

    pub fn set_breakpoints(&mut self, source: &str, lines: Vec<u32>) {
        self.breakpoints.insert(
            source.to_string(),
            lines.iter().map(|&line| DapBreakpoint {
                source: source.to_string(),
                line,
                verified: true,
            }).collect(),
        );
    }

    pub fn is_breakpoint(&self, source: &str, line: u32) -> bool {
        self.breakpoints.get(source)
            .map(|bps| bps.iter().any(|bp| bp.line == line))
            .unwrap_or(false)
    }

    pub fn stop_at(&mut self, source: &str, line: u32, reason: &str,
                   stage: &str, locals: Vec<(String, String, String)>) {
        self.stopped = true;
        self.stop_reason = Some(reason.to_string());
        self.current_source = Some(source.to_string());
        self.current_line = Some(line);
        self.current_stage = Some(stage.to_string());
        self.locals = locals;
        // `stopped` イベントをキューに積む（サーバーループが drain して送信）
        let seq = self.next_seq();
        self.event_queue.push(serde_json::json!({
            "seq": seq, "type": "event", "event": "stopped",
            "body": {
                "reason": reason,
                "threadId": 1,
                "source": { "path": source },
                "line": line,
            }
        }));
    }

    pub fn resume(&mut self) {
        self.stopped = false;
        self.stop_reason = None;
    }
}
```

### T1-3: `fav/src/dap/adapter.rs`

VM フック → DAP イベント変換。

```rust
use std::sync::{Arc, Mutex};
use super::session::DapSession;

/// VM から DAP セッションに送るフック
#[derive(Debug, Clone)]
pub enum DapHook {
    StageEnter {
        name:   String,
        source: String,
        line:   u32,
        locals: Vec<(String, String, String)>,  // (name, type, value)
    },
    StageExit {
        name:   String,
        result: String,  // vmvalue_repr の結果
    },
    Output(String),
}

/// VM と DAP サーバーを繋ぐアダプター
#[derive(Clone)]
pub struct DapAdapter {
    pub session: Arc<Mutex<DapSession>>,
    pub step_mode: bool,  // true = ステップモード（各 stage で停止）
}

impl DapAdapter {
    pub fn new() -> Self {
        DapAdapter {
            session: Arc::new(Mutex::new(DapSession::new())),
            step_mode: false,
        }
    }

    /// VM から呼ばれるフック処理
    pub fn on_hook(&self, hook: DapHook) {
        let mut sess = self.session.lock().unwrap_or_else(|e| e.into_inner());
        match hook {
            DapHook::StageEnter { name, source, line, locals } => {
                let is_bp = sess.is_breakpoint(&source, line);
                let reason = if is_bp { "breakpoint" } else { "step" };
                if is_bp || self.step_mode {
                    sess.stop_at(&source, line, reason, &name, locals);
                }
            }
            DapHook::StageExit { .. } => {}
            DapHook::Output(_msg) => {}
        }
    }
}
```

### T1-4: `fav/src/dap/server.rs`

TCP ソケット + DAP メッセージループ（フェーズ1: シングルスレッド、`tiny_http` は HTTP 用のため TCP 直接利用）。

```rust
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use serde_json::Value;
use super::protocol::*;
use super::session::DapSession;

/// DAP メッセージを Content-Length ヘッダー付きで送信
pub fn send_dap_message<W: Write>(writer: &mut W, body: &Value) -> std::io::Result<()> {
    let json = serde_json::to_string(body)?;
    let content = format!("Content-Length: {}\r\n\r\n{}", json.len(), json);
    writer.write_all(content.as_bytes())?;
    writer.flush()
}

/// Content-Length ヘッダーを読んで DAP メッセージを受信
pub fn recv_dap_message<R: BufRead + std::io::Read>(reader: &mut R) -> Option<Value> {
    let mut header = String::new();
    let mut content_length: usize = 0;
    loop {
        header.clear();
        if reader.read_line(&mut header).ok()? == 0 { return None; }
        if header == "\r\n" { break; }
        if header.starts_with("Content-Length:") {
            let len_str = header["Content-Length:".len()..].trim();
            content_length = len_str.parse().ok()?;
        }
    }
    let mut buf = vec![0u8; content_length];
    reader.read_exact(&mut buf).ok()?;
    serde_json::from_str(&String::from_utf8(buf).ok()?).ok()
}

/// DAP サーバーのメインループ（接続1本処理）
/// session は Arc<Mutex<DapSession>> で VM 側と共有する（HIGH-3 修正）
pub fn run_dap_server(
    port: u16,
    session: std::sync::Arc<std::sync::Mutex<DapSession>>,
) -> Result<(), String> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr)
        .map_err(|e| format!("DAP: bind failed on {addr}: {e}"))?;
    eprintln!("[fav DAP] Listening on {addr} ...");

    let (stream, peer) = listener.accept()
        .map_err(|e| format!("DAP: accept failed: {e}"))?;
    eprintln!("[fav DAP] Connected from {peer}");

    let mut reader = BufReader::new(stream.try_clone()
        .map_err(|e| format!("DAP: clone stream: {e}"))?);
    let mut writer = stream;

    handle_dap_session(session, &mut reader, &mut writer)
}

fn handle_dap_session<R: BufRead + std::io::Read, W: Write>(
    session: std::sync::Arc<std::sync::Mutex<DapSession>>,
    reader:  &mut R,
    writer:  &mut W,
) -> Result<(), String> {
    loop {
        // recv_dap_message に委譲（Content-Length ヘッダーループ + read_exact）
        let msg = match recv_dap_message(reader) {
            Some(v) => v,
            None => break,  // EOF or parse error
        };
        let req: DapRequest = match serde_json::from_value(msg) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let (resp, pending_events) = {
            let mut sess = session.lock().unwrap_or_else(|e| e.into_inner());
            let resp = handle_dap_request(&mut sess, &req);
            let events = std::mem::take(&mut sess.event_queue);  // pending `stopped` 等を drain
            (resp, events)
        };
        let resp_val = serde_json::to_value(&resp)
            .map_err(|e| format!("DAP: serialize response: {e}"))?;
        send_dap_message(writer, &resp_val).map_err(|e| e.to_string())?;

        // VM からキューに積まれたイベント（`stopped` 等）を送信
        for event in pending_events {
            send_dap_message(writer, &event).map_err(|e| e.to_string())?;
        }

        if req.command == "disconnect" { break; }
    }
    Ok(())
}

fn handle_dap_request(session: &mut DapSession, req: &DapRequest) -> DapResponse {
    let seq = session.next_seq();
    let body = match req.command.as_str() {
        "initialize" => {
            Some(serde_json::json!({
                "supportsConfigurationDoneRequest": true,
                "supportsConditionalBreakpoints": false,
                "supportsStepBack": false,
            }))
        }
        "launch" => Some(serde_json::json!({})),
        "configurationDone" => Some(serde_json::json!({})),
        "threads" => Some(serde_json::json!({
            "threads": [{ "id": 1, "name": "main" }]
        })),
        "stackTrace" => Some(serde_json::json!({
            "stackFrames": [{
                "id": 1,
                "name": session.current_stage.as_deref().unwrap_or("main"),  // ステージ名
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
            let vars: Vec<_> = session.locals.iter().map(|(name, ty, val)| {
                serde_json::json!({ "name": name, "type": ty, "value": val, "variablesReference": 0 })
            }).collect();
            Some(serde_json::json!({ "variables": vars }))
        }
        "setBreakpoints" => {
            if let Some(args) = &req.arguments {
                let path = args.get("source")
                    .and_then(|s| s.get("path"))
                    .and_then(|p| p.as_str())
                    .unwrap_or("");
                let lines: Vec<u32> = args.get("breakpoints")
                    .and_then(|b| b.as_array())
                    .map(|arr| arr.iter()
                        .filter_map(|bp| bp.get("line").and_then(|l| l.as_u64()).map(|l| l as u32))
                        .collect())
                    .unwrap_or_default();
                session.set_breakpoints(path, lines.clone());
                let bps: Vec<_> = lines.iter().map(|&l| serde_json::json!({ "verified": true, "line": l })).collect();
                Some(serde_json::json!({ "breakpoints": bps }))
            } else {
                Some(serde_json::json!({ "breakpoints": [] }))
            }
        }
        "next" | "stepIn" => { session.resume(); Some(serde_json::json!({})) }
        "continue" => { session.resume(); Some(serde_json::json!({ "allThreadsContinued": true })) }
        "disconnect" => Some(serde_json::json!({})),
        _ => None,
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
```

### T1-5: `fav/src/dap/mod.rs`

```rust
pub mod protocol;
pub mod session;
pub mod adapter;
pub mod server;

pub use server::run_dap_server;
pub use adapter::{DapAdapter, DapHook};
pub use session::DapSession;
```

### T1-6: `fav/src/lib.rs` + `fav/src/main.rs` — `mod dap` 追加

```rust
// lib.rs と main.rs 両方に追加
#[cfg(not(target_arch = "wasm32"))]
pub mod dap;
```

---

## T2: `fav/src/backend/vm.rs` — debug フック挿入

### 2-1. VM struct に `debug_mode` + `dap_adapter` フィールド追加

```rust
// VM struct に追加
#[cfg(not(target_arch = "wasm32"))]
pub debug_mode: bool,
#[cfg(not(target_arch = "wasm32"))]
pub dap_adapter: Option<crate::dap::DapAdapter>,
```

`VM::new_with_db_path()` の初期化:
```rust
#[cfg(not(target_arch = "wasm32"))]
debug_mode: false,
#[cfg(not(target_arch = "wasm32"))]
dap_adapter: None,
```

### 2-2. stage 実行前後にフック挿入

stage 実行前（`run_stage` 関数内、または stage 境界の dispatch 直前）:

```rust
#[cfg(not(target_arch = "wasm32"))]
if self.debug_mode {
    if let Some(adapter) = &self.dap_adapter {
        let locals = self.collect_locals_for_dap();
        adapter.on_hook(crate::dap::DapHook::StageEnter {
            name:   stage_name.to_string(),
            source: source_path.to_string(),
            line:   current_line,
            locals,
        });
    }
}
```

### 2-3. `collect_locals_for_dap` ヘルパー

> 注意: `CallFrame` は `fn_idx / ip / base / n_locals / line` のみ（`locals` フィールドなし）。
> `FvcFunction` にも変数名マッピングはない（スタックベース VM のため）。
> ローカル変数は `stack[frame.base .. frame.base + frame.n_locals]` に格納されており、
> 名前は `local_0`, `local_1`, ... の連番で提供する。

```rust
#[cfg(not(target_arch = "wasm32"))]
fn collect_locals_for_dap(&self) -> Vec<(String, String, String)> {
    let Some(frame) = self.frames.last() else { return vec![]; };
    let end = (frame.base + frame.n_locals).min(self.stack.len());
    self.stack[frame.base..end]
        .iter()
        .enumerate()
        .map(|(i, nanval)| {
            let val: VMValue = nanval.clone().to_vmvalue();  // clone() 必須（to_vmvalue は値渡し）
            (
                format!("local_{}", i),
                vmvalue_type_name(&val).to_string(),
                vmvalue_repr(&val),
            )
        })
        .collect()
}
```

---

## T3: `fav/src/driver.rs` — `cmd_dap` / `cmd_run_debug`

### 3-1. `cmd_dap(port: u16)`

```rust
#[cfg(not(target_arch = "wasm32"))]
pub fn cmd_dap(port: u16) -> Result<(), String> {
    use std::sync::{Arc, Mutex};
    let session = Arc::new(Mutex::new(crate::dap::session::DapSession::new()));
    crate::dap::run_dap_server(port, session)
}
```

### 3-2. `cmd_run_debug(path: &str, dap_port: u16)`

```rust
#[cfg(not(target_arch = "wasm32"))]
pub fn cmd_run_debug(path: &str, dap_port: u16) -> Result<(), String> {
    // 1. ソースをコンパイルして FvcArtifact を生成
    //    （実際の呼び出しは既存 cmd_run のコンパイルパスを参照）
    //    NOTE: VM::new_with_db_path(artifact: &FvcArtifact, db_path: Option<String>)
    //    実装時は compile_source_to_artifact(path) → artifact を生成してから渡す
    let artifact = compile_path_to_artifact(path)?;  // driver.rs の既存コンパイルヘルパーを使用

    // 2. DapAdapter を作成（Arc<Mutex<DapSession>> を内包）
    let adapter = crate::dap::DapAdapter::new();

    // 3. VM を debug_mode で起動
    let mut vm = Vm::new_with_db_path(&artifact, None);
    vm.debug_mode = true;
    vm.dap_adapter = Some(adapter.clone());

    // 4. DAP サーバーを別スレッドで起動（adapter.session を共有）
    let shared_session = adapter.session.clone();  // Arc clone — 同じ DapSession を共有
    std::thread::spawn(move || {
        let _ = crate::dap::run_dap_server(dap_port, shared_session);
    });

    // 5. パイプライン実行（VM の run / execute_artifact を呼ぶ）
    //    実装時は既存 cmd_run の実行パターンに従う
    vm.run().map_err(|e| e.to_string())
}
```

### 3-3. `main.rs` への CLI 追加

```rust
// fav dap [--port N]
"dap" => {
    let port = args.iter()
        .position(|a| a == "--port")
        .and_then(|i| args.get(i + 1))
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(5678);
    cmd_dap(port).map_err(|e| eprintln!("{e}")).ok();
}
// fav run --debug
"run" if args.contains(&"--debug".to_string()) => {
    let port = args.iter()
        .position(|a| a == "--dap-port")
        .and_then(|i| args.get(i + 1))
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(5678);
    let file = /* ... */;
    cmd_run_debug(file, port).map_err(|e| eprintln!("{e}")).ok();
}
```

---

## T4: `fav/Cargo.toml` バージョン更新

`version = "21.0.0"` → `"21.1.0"`

`v210000_tests::version_is_21_0_0` に `#[ignore]` を追加。

---

## T5: `CHANGELOG.md` + `site/content/docs/tools/dap.mdx`

### CHANGELOG エントリ

```markdown
## [v21.1.0] — 2026-06-20 — DAP デバッガー

### Added
- `fav dap [--port 5678]` — DAP サーバー起動コマンド
- `fav run --debug [--dap-port N]` — デバッグモード実行
- `fav/src/dap/` モジュール（protocol / session / adapter / server）
- DAP サポートリクエスト: initialize / launch / setBreakpoints / configurationDone /
  threads / stackTrace / scopes / variables / next / stepIn / continue / disconnect
- `VM::debug_mode` / `VM::dap_adapter` フィールド（`--debug` なしはゼロコスト）
- VS Code `launch.json` 設定例（`site/content/docs/tools/dap.mdx`）
```

### site/content/docs/tools/dap.mdx の骨格

```mdx
# DAP デバッガー

VS Code / Neovim / Emacs から Favnir パイプラインをステップ実行する。

## セットアップ（VS Code）

1. `launch.json` を設定:
   ...

## 使い方

```bash
fav dap          # DAP サーバーを起動（ポート 5678）
fav run --debug pipeline.fav   # デバッグモード実行
```

## サポートする操作

| 操作 | ショートカット |
|---|---|
| ブレークポイント設定 | F9 |
| 実行 | F5 |
| ステップオーバー | F10 |
| ステップイン | F11 |

## WASM サポート

DAP サーバーは TCP ソケットを使用するため **WASM ビルドでは利用不可**。
`fav dap` / `fav run --debug` はネイティブバイナリ専用。
```

---

## T6: `fav/src/driver.rs` — `v211000_tests` 追加

```rust
// ── v211000_tests (v21.1.0) — DAP デバッガー ─────────────────────────────────
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod v211000_tests {
    #[test]
    fn version_is_21_1_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("21.1.0"), "Cargo.toml should have version 21.1.0");
    }

    #[test]
    fn dap_protocol_initialize_request_parses() {
        use crate::dap::protocol::DapRequest;
        let json = r#"{"seq":1,"type":"request","command":"initialize","arguments":{"adapterID":"favnir"}}"#;
        let req: DapRequest = serde_json::from_str(json).expect("should parse");
        assert_eq!(req.command, "initialize");
        assert_eq!(req.seq, 1);
    }

    #[test]
    fn dap_protocol_response_serializes() {
        use crate::dap::protocol::DapResponse;
        let resp = DapResponse {
            seq:         1,
            kind:        "response".to_string(),
            request_seq: 1,
            success:     true,
            command:     "initialize".to_string(),
            body:        Some(serde_json::json!({"supportsConfigurationDoneRequest": true})),
            message:     None,
        };
        let json = serde_json::to_string(&resp).expect("should serialize");
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"command\":\"initialize\""));
    }

    #[test]
    fn dap_session_breakpoint_set_and_hit() {
        use crate::dap::session::DapSession;
        let mut sess = DapSession::new();
        sess.set_breakpoints("src/pipeline.fav", vec![10, 20]);
        assert!(sess.is_breakpoint("src/pipeline.fav", 10));
        assert!(sess.is_breakpoint("src/pipeline.fav", 20));
        assert!(!sess.is_breakpoint("src/pipeline.fav", 15));
    }

    #[test]
    fn dap_adapter_stopped_event_format() {
        use crate::dap::adapter::{DapAdapter, DapHook};
        let adapter = DapAdapter::new();
        // ブレークポイントを事前設定
        {
            let mut sess = adapter.session.lock().unwrap();
            sess.set_breakpoints("pipeline.fav", vec![5]);
        }
        // StageEnter フックを発火
        adapter.on_hook(DapHook::StageEnter {
            name:   "Transform".to_string(),
            source: "pipeline.fav".to_string(),
            line:   5,
            locals: vec![("x".to_string(), "Int".to_string(), "42".to_string())],
        });
        let mut sess = adapter.session.lock().unwrap();
        assert!(sess.stopped, "should be stopped at breakpoint");
        assert_eq!(sess.current_line, Some(5));
        assert_eq!(sess.stop_reason.as_deref(), Some("breakpoint"));
        // stopped イベントが event_queue に積まれ、JSON フォーマットが正しいことを確認
        assert!(!sess.event_queue.is_empty(), "event_queue should have stopped event");
        let event = &sess.event_queue[0];
        assert_eq!(event["type"].as_str(), Some("event"));
        assert_eq!(event["event"].as_str(), Some("stopped"));
        assert_eq!(event["body"]["line"].as_u64(), Some(5));
        assert_eq!(event["body"]["source"]["path"].as_str(), Some("pipeline.fav"));
        let _ = std::mem::take(&mut sess.event_queue);  // drain
    }
}
```
