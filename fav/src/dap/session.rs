use std::collections::HashMap;
use super::protocol::DapBreakpoint;

#[derive(Debug, Default)]
pub struct DapSession {
    pub seq: u64,
    pub breakpoints: HashMap<String, Vec<DapBreakpoint>>, // source path → breakpoints
    pub stopped: bool,
    pub stop_reason: Option<String>,
    pub current_line: Option<u32>,
    pub current_source: Option<String>,
    pub current_stage: Option<String>, // stackTrace.name に使うステージ名
    pub locals: Vec<(String, String, String)>, // (name, type, value)
    pub event_queue: Vec<serde_json::Value>, // VM → クライアントへのプッシュイベント
}

impl DapSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_seq(&mut self) -> u64 {
        self.seq += 1;
        self.seq
    }

    pub fn set_breakpoints(&mut self, source: &str, lines: Vec<u32>) {
        self.breakpoints.insert(
            source.to_string(),
            lines
                .iter()
                .map(|&line| DapBreakpoint {
                    source: source.to_string(),
                    line,
                    verified: true,
                })
                .collect(),
        );
    }

    pub fn is_breakpoint(&self, source: &str, line: u32) -> bool {
        self.breakpoints
            .get(source)
            .map(|bps| bps.iter().any(|bp| bp.line == line))
            .unwrap_or(false)
    }

    pub fn stop_at(
        &mut self,
        source: &str,
        line: u32,
        reason: &str,
        stage: &str,
        locals: Vec<(String, String, String)>,
    ) {
        self.stopped = true;
        self.stop_reason = Some(reason.to_string());
        self.current_source = Some(source.to_string());
        self.current_line = Some(line);
        self.current_stage = Some(stage.to_string());
        self.locals = locals;
        // `stopped` イベントをキューに積む（サーバーループが drain して送信）
        let seq = self.next_seq();
        self.event_queue.push(serde_json::json!({
            "seq": seq,
            "type": "event",
            "event": "stopped",
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
