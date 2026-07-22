//! v22.7.0 — OpenTelemetry span 収集 + OTLP/HTTP エクスポート
//!
//! 新規依存なし。rand（ID 生成）・ureq（HTTP POST）・std::time（タイムスタンプ）を使用。

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ── 型定義 ──────────────────────────────────────────────────────────────────

pub type TraceId = String; // hex 32 chars（128-bit random）
pub type SpanId  = String; // hex 16 chars（64-bit random）

#[derive(Debug, Clone)]
pub struct OtelSpan {
    pub trace_id:       TraceId,
    pub span_id:        SpanId,
    pub parent_span_id: Option<SpanId>,
    pub name:           String,
    pub start_unix_ns:  u128,
    pub end_unix_ns:    u128,
    pub input_items:    u64,
    pub output_items:   u64,
    pub status:         OtelStatus,
    pub attrs:          Vec<(String, String)>,  // v52.7.0: schema / lineage 属性
}

#[derive(Debug, Clone, PartialEq)]
pub enum OtelStatus {
    Ok,
    Error(String),
}

// ── Thread-local ──────────────────────────────────────────────────────────────

thread_local! {
    /// 完成した spans（end 済み）
    static OTEL_SPANS: RefCell<Vec<OtelSpan>> = const { RefCell::new(Vec::new()) };
    /// 現在のトレース ID
    static CURRENT_TRACE: RefCell<Option<TraceId>> = const { RefCell::new(None) };
    /// 親 span スタック（ネスト対応）
    static PARENT_STACK: RefCell<Vec<SpanId>> = const { RefCell::new(Vec::new()) };
    /// OTel 有効フラグ
    static OTEL_ENABLED: RefCell<bool> = const { RefCell::new(false) };
    /// 進行中 spans（span_id → OtelSpan。end 時に OTEL_SPANS へ移動）
    // HashMap::new() は const fn でないため const {} ブロックを使えない（他の 4 変数と異なる）
    static PENDING_SPANS: RefCell<HashMap<SpanId, OtelSpan>> = RefCell::new(HashMap::new());
}

// ── ID 生成 ───────────────────────────────────────────────────────────────────

fn gen_trace_id() -> TraceId {
    let bytes: [u8; 16] = rand::random();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn gen_span_id() -> SpanId {
    let bytes: [u8; 8] = rand::random();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn now_unix_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

// ── 公開 API ──────────────────────────────────────────────────────────────────

/// OTel を有効化して新しいトレースを開始する（cmd_run の先頭で呼ぶ）。
pub fn otel_init() {
    OTEL_ENABLED.with(|e|  *e.borrow_mut() = true);
    CURRENT_TRACE.with(|t| *t.borrow_mut() = Some(gen_trace_id()));
    PARENT_STACK.with(|s|  s.borrow_mut().clear());
    OTEL_SPANS.with(|s|    s.borrow_mut().clear());
    PENDING_SPANS.with(|p| p.borrow_mut().clear());
}

/// OTel が有効かどうかを返す。
pub fn otel_is_enabled() -> bool {
    OTEL_ENABLED.with(|e| *e.borrow())
}

/// 現在の親 span ID（PARENT_STACK の先頭）を返す。
pub fn otel_current_parent() -> Option<SpanId> {
    PARENT_STACK.with(|s| s.borrow().last().cloned())
}

/// 新しい span を開始し、span_id を返す。PARENT_STACK に push する。
pub fn otel_span_start(name: &str, parent_id: Option<&SpanId>) -> SpanId {
    let span_id  = gen_span_id();
    let trace_id = CURRENT_TRACE.with(|t| {
        t.borrow().clone().unwrap_or_else(gen_trace_id)
    });
    let span = OtelSpan {
        trace_id,
        span_id:        span_id.clone(),
        parent_span_id: parent_id.cloned(),
        name:           name.to_string(),
        start_unix_ns:  now_unix_ns(),
        end_unix_ns:    0,
        input_items:    0,
        output_items:   0,
        status:         OtelStatus::Ok,
        attrs:          Vec::new(),
    };
    PENDING_SPANS.with(|p| p.borrow_mut().insert(span_id.clone(), span));
    PARENT_STACK.with(|s| s.borrow_mut().push(span_id.clone()));
    span_id
}

/// span を終了して収集済みリスト（OTEL_SPANS）に移す。PARENT_STACK から pop する。
pub fn otel_span_end(
    span_id:     &SpanId,
    input_items:  u64,
    output_items: u64,
    status:       OtelStatus,
) {
    PENDING_SPANS.with(|p| {
        if let Some(mut span) = p.borrow_mut().remove(span_id) {
            span.end_unix_ns  = now_unix_ns();
            span.input_items  = input_items;
            span.output_items = output_items;
            span.status       = status;
            OTEL_SPANS.with(|s| s.borrow_mut().push(span));
        }
    });
    PARENT_STACK.with(|s| {
        let mut stack = s.borrow_mut();
        if stack.last().map(|id| id == span_id).unwrap_or(false) {
            stack.pop();
        }
    });
}

/// v52.7.0: 現在実行中 span に文字列属性を追加する。OTel 有効時のみ呼ぶこと。
pub fn otel_add_attr(key: &str, val: &str) {
    let span_id = PARENT_STACK.with(|s| s.borrow().last().cloned());
    if let Some(sid) = span_id {
        PENDING_SPANS.with(|p| {
            if let Some(span) = p.borrow_mut().get_mut(&sid) {
                span.attrs.push((key.to_string(), val.to_string()));
            }
        });
    }
}

/// v52.7.0: 完了済み span リストの最後のエントリに属性を後付けする。
/// lineage.downstream の遡及追加に使用。
/// `expected_stage_name` と一致しない span には何も書き込まない（誤パッチ防止）。
pub fn otel_patch_attr_on_last(expected_stage_name: &str, key: &str, val: &str) {
    OTEL_SPANS.with(|s| {
        if let Some(span) = s.borrow_mut().last_mut() {
            if span.name == format!("stage:{}", expected_stage_name) {
                span.attrs.push((key.to_string(), val.to_string()));
            }
        }
    });
}

/// 収集済み spans を返す（テスト・エクスポートに使用）。
pub fn otel_collected_spans() -> Vec<OtelSpan> {
    OTEL_SPANS.with(|s| s.borrow().clone())
}

/// OTel が有効な場合のみエクスポートしてリセットする。
/// process::exit の直前など、通常の関数末尾を通らないパスで呼ぶ。
pub fn otel_flush_if_enabled() {
    if !otel_is_enabled() {
        return;
    }
    match std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        Ok(ep) => otel_export_http(&ep),
        Err(_) => otel_export_stdout(),
    }
    otel_reset();
}

/// thread-local をすべてリセット（テスト間クリーンアップ・run 後のリセット）。
pub fn otel_reset() {
    OTEL_ENABLED.with(|e|  *e.borrow_mut() = false);
    CURRENT_TRACE.with(|t| *t.borrow_mut() = None);
    PARENT_STACK.with(|s|  s.borrow_mut().clear());
    OTEL_SPANS.with(|s|    s.borrow_mut().clear());
    PENDING_SPANS.with(|p| p.borrow_mut().clear());
}

// ── エクスポート ──────────────────────────────────────────────────────────────

/// OTLP/HTTP (JSON) として POST する。
/// 失敗は stderr に警告するのみ（fav run を止めない）。
pub fn otel_export_http(endpoint: &str) {
    let spans = otel_collected_spans();
    if spans.is_empty() {
        return;
    }
    let body = build_otlp_json(&spans);
    let url   = format!("{}/v1/traces", endpoint.trim_end_matches('/'));
    match ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_string(&body)
    {
        Ok(_) => {}
        Err(ureq::Error::Status(code, _)) => {
            eprintln!("[OTEL] warn: OTLP endpoint returned HTTP {}", code);
        }
        Err(e) => {
            eprintln!("[OTEL] warn: failed to export spans: {}", e);
        }
    }
}

/// [OTEL] を stderr に出力する（エンドポイント未設定時のフォールバック）。
pub fn otel_export_stdout() {
    let spans = otel_collected_spans();
    if spans.is_empty() {
        eprintln!("[OTEL] no spans collected");
        return;
    }
    let trace_id = spans.first().map(|s| s.trace_id.as_str()).unwrap_or("?");
    eprintln!("[OTEL] trace_id={}", trace_id);
    for span in &spans {
        let dur_ms = span.end_unix_ns.saturating_sub(span.start_unix_ns) / 1_000_000;
        let status = match &span.status {
            OtelStatus::Ok       => "ok".to_string(),
            OtelStatus::Error(e) => format!("error({})", e),
        };
        eprintln!(
            "[OTEL] span {:<35} dur={:>6}ms  status={}  in={} out={}",
            span.name, dur_ms, status, span.input_items, span.output_items,
        );
        // v52.7.0: extra attrs（schema / lineage）
        for (k, v) in &span.attrs {
            eprintln!("       {:<30} = {}", k, v);
        }
    }
}

// ── OTLP JSON 生成 ────────────────────────────────────────────────────────────

fn build_otlp_json(spans: &[OtelSpan]) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let span_jsons: Vec<String> = spans.iter().map(|s| {
        let parent      = s.parent_span_id.as_deref().unwrap_or("");
        let status_code = match &s.status { OtelStatus::Ok => 1, OtelStatus::Error(_) => 2 };
        let status_str  = match &s.status { OtelStatus::Ok => "ok", OtelStatus::Error(_) => "error" };
        // v52.7.0: extra attrs（schema.name / schema.fields / lineage.upstream / lineage.downstream）
        let extra_attrs: String = s.attrs.iter().map(|(k, v)| {
            format!(
                r#"{{"key":"{}","value":{{"stringValue":"{}"}}}}"#,
                escape_json_str(k), escape_json_str(v)
            )
        }).collect::<Vec<_>>().join(",");
        let extra_part = if extra_attrs.is_empty() {
            String::new()
        } else {
            format!(",{}", extra_attrs)
        };
        let attrs = format!(
            r#"[{{"key":"favnir.stage.name","value":{{"stringValue":"{}"}}}},{{"key":"favnir.stage.input_items","value":{{"intValue":"{}"}}}},{{"key":"favnir.stage.output_items","value":{{"intValue":"{}"}}}},{{"key":"favnir.stage.status","value":{{"stringValue":"{}"}}}}{}]"#,
            escape_json_str(&s.name), s.input_items, s.output_items, status_str, extra_part,
        );
        format!(
            r#"{{"traceId":"{}","spanId":"{}","parentSpanId":"{}","name":"{}","kind":2,"startTimeUnixNano":"{}","endTimeUnixNano":"{}","attributes":{},"status":{{"code":{}}}}}"#,
            s.trace_id,
            s.span_id,
            parent,
            escape_json_str(&s.name),
            s.start_unix_ns,
            s.end_unix_ns,
            attrs,
            status_code,
        )
    }).collect();

    format!(
        r#"{{"resourceSpans":[{{"resource":{{"attributes":[{{"key":"service.name","value":{{"stringValue":"favnir"}}}},{{"key":"service.version","value":{{"stringValue":"{}"}}}}]}},"scopeSpans":[{{"scope":{{"name":"fav","version":"{}"}},"spans":[{}]}}]}}]}}"#,
        version,
        version,
        span_jsons.join(","),
    )
}

fn escape_json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"'  => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                // RFC 8259 §7: control characters U+0000–U+001F must be \uXXXX-escaped
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}
