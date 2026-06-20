use serde::{Deserialize, Serialize};

/// DAP リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapRequest {
    pub seq: u64,
    #[serde(rename = "type")]
    pub kind: String, // "request"
    pub command: String,
    pub arguments: Option<serde_json::Value>,
}

/// DAP レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapResponse {
    pub seq: u64,
    #[serde(rename = "type")]
    pub kind: String, // "response"
    pub request_seq: u64,
    pub success: bool,
    pub command: String,
    pub body: Option<serde_json::Value>,
    pub message: Option<String>,
}

/// ブレークポイント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapBreakpoint {
    pub source: String,
    pub line: u32,
    pub verified: bool,
}
