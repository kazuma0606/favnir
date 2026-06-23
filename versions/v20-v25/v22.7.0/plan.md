# v22.7.0 実装計画 — OpenTelemetry 統合

## 実装順序

```
T1（otel.rs）          ← 最初（新規モジュール。VM/driver の依存元）
T2（vm.rs）            ← T1 完了後（SeqStageEnter hoist + OTel span 追加）
T3（driver.rs）        ← T2 完了後（cmd_run に OTel 追加 + テスト）
T4（lib.rs / main.rs） ← T1 完了後（mod otel 宣言、T2 と並行可）
T5（Cargo + doc）      ← T3 完了後（バージョン更新・CHANGELOG・MDX・benchmarks）
```

---

## T1: `fav/src/otel.rs` — OTel モジュール新規作成

### 事前確認コマンド

```bash
# rand / ureq が native-only deps にあることを確認
grep -n "rand\|ureq" fav/Cargo.toml | head -10

# SeqStageEnter の行番号を確認（T2 の挿入基準）
grep -n "SeqStageEnter\|SeqStageCheck\|stage_name" fav/src/backend/vm.rs | head -15
```

### 1-1: ファイル全体を新規作成

```rust
//! v22.7.0 — OpenTelemetry span 収集 + OTLP/HTTP エクスポート
//!
//! 新規依存なし。rand（ID 生成）・ureq（HTTP POST）を使用。

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ── 型 ──────────────────────────────────────────────────────────────────────

pub type TraceId = String;  // hex 32 chars
pub type SpanId  = String;  // hex 16 chars

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
}

#[derive(Debug, Clone, PartialEq)]
pub enum OtelStatus { Ok, Error(String) }

// ── Thread-local ─────────────────────────────────────────────────────────────

thread_local! {
    static OTEL_SPANS:    RefCell<Vec<OtelSpan>>         = const { RefCell::new(Vec::new()) };
    static CURRENT_TRACE: RefCell<Option<TraceId>>       = const { RefCell::new(None) };
    static PARENT_STACK:  RefCell<Vec<SpanId>>           = const { RefCell::new(Vec::new()) };
    static OTEL_ENABLED:  RefCell<bool>                  = const { RefCell::new(false) };
    static PENDING_SPANS: RefCell<HashMap<SpanId, OtelSpan>> = const { RefCell::new(HashMap::new()) };
}

// ── ID 生成 ──────────────────────────────────────────────────────────────────

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

// ── 公開 API ─────────────────────────────────────────────────────────────────

/// OTel を有効化して新しいトレースを開始する（cmd_run の先頭で呼ぶ）。
pub fn otel_init() {
    OTEL_ENABLED.with(|e|  *e.borrow_mut() = true);
    CURRENT_TRACE.with(|t| *t.borrow_mut() = Some(gen_trace_id()));
    PARENT_STACK.with(|s|  s.borrow_mut().clear());
    OTEL_SPANS.with(|s|    s.borrow_mut().clear());
    PENDING_SPANS.with(|p| p.borrow_mut().clear());
}

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
    let trace_id = CURRENT_TRACE.with(|t| t.borrow().clone().unwrap_or_else(gen_trace_id));
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
    };
    PENDING_SPANS.with(|p| p.borrow_mut().insert(span_id.clone(), span));
    PARENT_STACK.with(|s| s.borrow_mut().push(span_id.clone()));
    span_id
}

/// span を終了して収集済みリスト（OTEL_SPANS）に移す。PARENT_STACK から pop する。
pub fn otel_span_end(
    span_id: &SpanId,
    input_items:  u64,
    output_items: u64,
    status: OtelStatus,
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

/// 収集済み spans を返す（テスト・エクスポートに使用）。
pub fn otel_collected_spans() -> Vec<OtelSpan> {
    OTEL_SPANS.with(|s| s.borrow().clone())
}

/// thread-local をすべてリセット（テスト間クリーンアップ・run 後のリセット）。
pub fn otel_reset() {
    OTEL_ENABLED.with(|e|  *e.borrow_mut() = false);
    CURRENT_TRACE.with(|t| *t.borrow_mut() = None);
    PARENT_STACK.with(|s|  s.borrow_mut().clear());
    OTEL_SPANS.with(|s|    s.borrow_mut().clear());
    PENDING_SPANS.with(|p| p.borrow_mut().clear());
}

// ── エクスポート ─────────────────────────────────────────────────────────────

/// OTLP/HTTP (JSON) として POST する。
/// 失敗は stderr に警告するのみ（fav run を止めない）。
pub fn otel_export_http(endpoint: &str) {
    let spans = otel_collected_spans();
    if spans.is_empty() { return; }
    let body = build_otlp_json(&spans);
    let url  = format!("{}/v1/traces", endpoint.trim_end_matches('/'));
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
    }
}

// ── OTLP JSON 生成 ────────────────────────────────────────────────────────────

fn build_otlp_json(spans: &[OtelSpan]) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let span_jsons: Vec<String> = spans.iter().map(|s| {
        let parent      = s.parent_span_id.as_deref().unwrap_or("");
        let status_code = match &s.status { OtelStatus::Ok => 1, OtelStatus::Error(_) => 2 };
        let status_str  = match &s.status { OtelStatus::Ok => "ok", OtelStatus::Error(_) => "error" };
        let attrs = format!(
            r#"[{{"key":"favnir.stage.name","value":{{"stringValue":"{}"}}}},{{"key":"favnir.stage.input_items","value":{{"intValue":"{}"}}}},{{"key":"favnir.stage.output_items","value":{{"intValue":"{}"}}}},{{"key":"favnir.stage.status","value":{{"stringValue":"{}"}}}}]"#,
            escape_json_str(&s.name), s.input_items, s.output_items, status_str,
        );
        format!(
            r#"{{"traceId":"{}","spanId":"{}","parentSpanId":"{}","name":"{}","kind":2,"startTimeUnixNano":"{}","endTimeUnixNano":"{}","attributes":{},"status":{{"code":{}}}}}"#,
            s.trace_id, s.span_id, parent, escape_json_str(&s.name),
            s.start_unix_ns, s.end_unix_ns, attrs, status_code,
        )
    }).collect();

    format!(
        r#"{{"resourceSpans":[{{"resource":{{"attributes":[{{"key":"service.name","value":{{"stringValue":"favnir"}}}},{{"key":"service.version","value":{{"stringValue":"{}"}}}}]}},"scopeSpans":[{{"scope":{{"name":"fav","version":"{}"}},"spans":[{}]}}]}}]}}"#,
        version, version, span_jsons.join(","),
    )
}

fn escape_json_str(s: &str) -> String {
    s.chars().flat_map(|c| match c {
        '"'  => vec!['\\', '"'],
        '\\' => vec!['\\', '\\'],
        '\n' => vec!['\\', 'n'],
        '\r' => vec!['\\', 'r'],
        '\t' => vec!['\\', 't'],
        c    => vec![c],
    }).collect()
}
```

---

## T2: `fav/src/backend/vm.rs` — stage 実行に OTel span を追加

### 事前確認コマンド

```bash
# SeqStageEnter / SeqStageCheck の正確な行番号と stage_name のスコープを確認
grep -n "SeqStageEnter\|SeqStageCheck\|stage_name\s*=\|verbose_level.*0\|trace_emit.*enter\|trace_emit.*exit" fav/src/backend/vm.rs | head -20

# VM struct フィールドを確認（current_otel_span_id の挿入位置）
grep -n "pub struct VM\|trace_lines\|pub " fav/src/backend/vm.rs | head -20
```

### 2-1: `VM` struct に `current_otel_span_id` フィールドを追加

`pub trace_lines: Vec<String>` の直後に追加:

```rust
/// v22.7.0: 現在実行中 stage の OTel span ID。SeqStageEnter で設定、SeqStageCheck で消費。
pub current_otel_span_id: Option<crate::otel::SpanId>,
```

### 2-2: `VM` struct literal（初期化箇所）に `None` を追加

```bash
grep -n "trace_lines: Vec::new\|trace_lines:" fav/src/backend/vm.rs | head -5
```

で `trace_lines` 初期化行を確認し、その直後に追加:

```rust
current_otel_span_id: None,
```

### 2-3: `SeqStageEnter` の `stage_name` を verbose ブロックの外に hoist

**変更前**:
```rust
x if x == Opcode::SeqStageEnter as u8 => {
    let name_idx = Self::read_u16(function, frame)? as usize;
    if Self::verbose_level() > 0 {
        let stage_name = artifact
            .str_table
            .get(name_idx)
            .map(|s| s.as_str())
            .unwrap_or("?");
        trace_emit(&mut vm.trace_lines, format!("[TRACE] stage {}: enter", stage_name));
    }
}
```

**変更後**（`stage_name` を if ブロック外に hoist し、OTel コードを追加）:

```rust
x if x == Opcode::SeqStageEnter as u8 => {
    let name_idx = Self::read_u16(function, frame)? as usize;
    // v22.7.0: stage_name を hoist（OTel + verbose 両方で使う）
    let stage_name = artifact
        .str_table
        .get(name_idx)
        .map(|s| s.as_str())
        .unwrap_or("?");
    if Self::verbose_level() > 0 {
        trace_emit(&mut vm.trace_lines, format!("[TRACE] stage {}: enter", stage_name));
    }
    // v22.7.0: OTel span 開始
    if crate::otel::otel_is_enabled() {
        let parent  = crate::otel::otel_current_parent();
        let span_id = crate::otel::otel_span_start(
            &format!("stage:{}", stage_name),
            parent.as_ref(),
        );
        vm.current_otel_span_id = Some(span_id);
    }
}
```

### 2-4: `SeqStageCheck` の Ok / Err 終了時に OTel span 終了を追加

`SeqStageCheck` で `stage_name` はすでに verbose ブロック外に定義されている（変更不要）。

**Ok パス**: `trace_emit(&mut vm.trace_lines, format!("[TRACE] stage {}: exit Ok({})", ...))` の直後:

```rust
// v22.7.0: OTel span 終了（Ok）
if let Some(ref sid) = vm.current_otel_span_id.take() {
    let out_items = otel_value_items(&unwrapped.to_vmvalue());
    crate::otel::otel_span_end(sid, 0, out_items, crate::otel::OtelStatus::Ok);
}
```

**Err パス**: `trace_emit(&mut vm.trace_lines, format!("[TRACE] stage {}: exit Err({})", ...))` の直後:

```rust
// v22.7.0: OTel span 終了（Err）
if let Some(ref sid) = vm.current_otel_span_id.take() {
    crate::otel::otel_span_end(
        sid, 0, 0,
        crate::otel::OtelStatus::Error(inner_msg.clone()),
    );
}
```

> **注意**: `unwrapped` / `inner_msg` は `SeqStageCheck` ブランチの実際の変数名。
> `grep -n "unwrapped\|inner_msg" fav/src/backend/vm.rs | grep -v "//\|test"` で確認してから使うこと。

### 2-5: `otel_value_items` ヘルパーを追加

`fn trace_emit` の直前（vm.rs 末尾付近）に追加:

```rust
/// v22.7.0: VMValue の要素数を返す（OTel output_items 用）。
fn otel_value_items(v: &VMValue) -> u64 {
    match v {
        VMValue::List(items, ..) => items.len() as u64,
        VMValue::Str(s)          => s.len() as u64,
        _                        => 1,
    }
}
```

> **注意**: `VMValue::List` の実際のバリアント形式を
> `grep -n "enum VMValue\|List(" fav/src/backend/vm.rs | head -10` で確認してから使うこと。

---

## T3: `fav/src/driver.rs` — `cmd_run` に OTel 追加 + `v227000_tests`

### 事前確認コマンド

```bash
# cmd_run の set_verbose_level と関数末尾を確認
grep -n "set_verbose_level\|cmd_run_self_hosted\|fn cmd_run\b" fav/src/driver.rs | head -10

# 既存テストで build_artifact / exec_artifact_main がどう呼ばれているかを確認
grep -n "build_artifact\|exec_artifact_main" fav/src/driver.rs | grep -v "fn " | head -10
```

### 3-1: `cmd_run` の `set_verbose_level(verbose_level)` 直後に OTel init を追加

```rust
// v22.7.0: OTel 初期化（trace フラグが立っているとき）
if trace {
    crate::otel::otel_init();
}
```

### 3-2: `cmd_run` の末尾（`cmd_run_self_hosted` の直前）に OTel export を追加

`cmd_run` 本体の閉じ `}` の直前に追加する。`cmd_run` が `cmd_run_self_hosted` の上で終わるので
「`// ── fav run --self-host` コメントの直前」が目印。

```rust
// v22.7.0: OTel エクスポート（run 完了後）
if trace {
    match std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        Ok(ep) => crate::otel::otel_export_http(&ep),
        Err(_) => crate::otel::otel_export_stdout(),
    }
    crate::otel::otel_reset();
}
```

### 3-3: `v226000_tests::version_is_22_6_0` に `#[ignore]` を追加

```bash
grep -n "fn version_is_22_6_0" fav/src/driver.rs
```

で行番号を確認し、`#[test]` の前に `#[ignore]` を追加。

### 3-4: `v227000_tests` モジュールを追加（5 件）

`v226000_extra_tests` ブロックの直後に追加。テスト内では `driver.rs` 内の private fn
（`build_artifact` / `exec_artifact_main`）を `super::` 経由で呼び出せる。

```rust
// ── v227000_tests (v22.7.0) — OpenTelemetry 統合 ─────────────────────────────
#[cfg(test)]
mod v227000_tests {
    use super::*;

    fn parse_and_run_with_otel(src: &str) {
        crate::otel::otel_init();
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize().expect("lex failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse failed");
        let artifact = build_artifact(&prog);
        // exec_artifact_main は private だが同一ファイルのテストからアクセス可
        let _ = exec_artifact_main(&artifact, None);
        // spans は thread-local OTEL_SPANS に収集済み
    }

    #[test]
    fn version_is_22_7_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"22.7.0\""), "Cargo.toml should have version 22.7.0");
    }

    #[test]
    fn otel_spans_collected_after_run() {
        let src = r#"
stage Double: Int -> Int = |n| { n }
seq TestPipe = Double
"#;
        parse_and_run_with_otel(src);
        let spans = crate::otel::otel_collected_spans();
        assert!(!spans.is_empty(), "expected at least 1 OTel span, got 0");
        crate::otel::otel_reset();
    }

    #[test]
    fn otel_span_name_includes_stage_name() {
        let src = r#"
stage MyStage: Int -> Int = |n| { n }
seq TestPipe = MyStage
"#;
        parse_and_run_with_otel(src);
        let spans = crate::otel::otel_collected_spans();
        assert!(
            spans.iter().any(|s| s.name.contains("MyStage")),
            "expected span with name containing 'MyStage', got: {:?}",
            spans.iter().map(|s| &s.name).collect::<Vec<_>>(),
        );
        crate::otel::otel_reset();
    }

    #[test]
    fn otel_export_stdout_does_not_panic() {
        crate::otel::otel_init();
        let sid = crate::otel::otel_span_start("stage:Test", None);
        crate::otel::otel_span_end(&sid, 1, 1, crate::otel::OtelStatus::Ok);
        // [OTEL] を stderr に出力（パニックしないことを検証）
        crate::otel::otel_export_stdout();
        crate::otel::otel_reset();
    }

    #[test]
    fn changelog_has_v22_7_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v22.7.0]"), "CHANGELOG should have v22.7.0 entry");
    }
}
```

> **注意**:
> - `exec_artifact_main` の引数は `(artifact: &FvcArtifact, db_path: Option<&str>)` — `None` を渡す。
> - `build_artifact` は `(program: &ast::Program) -> FvcArtifact` — driver.rs 内 private fn。
> - テストが `seq TestPipe = Double` を正しく実行するには `Double` stage が `Int -> Int` で
>   整合している必要がある（`main` fn として codegen されているか確認）。
> - 各テストで `otel_reset()` を呼ぶことを忘れずに（テスト間の thread-local 干渉を防ぐ）。

---

## T4: `fav/src/lib.rs` / `fav/src/main.rs` — `mod otel` 宣言追加

### 事前確認コマンド

```bash
# lib.rs の native-only mod 宣言を確認
grep -n "cfg(not.*wasm32\|^pub mod\|^mod " fav/src/lib.rs | head -20

# main.rs の mod 宣言を確認（main.rs は native-only なので cfg なしでも可だが一貫性のため付ける）
grep -n "cfg(not.*wasm32\|^mod\|^pub mod" fav/src/main.rs | head -20
```

### 4-1: `lib.rs` に追加

`mod pushdown;` / `mod arena;` / `mod parallel;` などの `#[cfg(not(target_arch = "wasm32"))]` ブロックの近くに追加:

```rust
#[cfg(not(target_arch = "wasm32"))]
pub mod otel;
```

### 4-2: `main.rs` に追加

`lib.rs` と同じ native-only mod のブロック近くに追加（`#[cfg]` ガード付き）:

```rust
#[cfg(not(target_arch = "wasm32"))]
mod otel;
```

---

## T5: バージョン更新・CHANGELOG・MDX・benchmarks

### 5-1: `fav/Cargo.toml`

```toml
version = "22.7.0"
```

### 5-2: `CHANGELOG.md` の先頭に v22.7.0 エントリを追加

```markdown
## [v22.7.0] — 2026-06-21 — OpenTelemetry 統合

### 追加
- `fav/src/otel.rs` — OTel span 収集・OTLP/HTTP エクスポート・stderr フォールバック（新規依存なし）
- `fav run --trace` に OTel span 収集を統合
  - `SeqStageEnter` 時に `stage:<name>` span を開始
  - `SeqStageCheck` 終了（Ok / Err）時に span を完了
- `OTEL_EXPORTER_OTLP_ENDPOINT` 環境変数でエクスポート先を制御
  - 設定あり: OTLP/HTTP POST（Jaeger / Grafana Tempo 対応）
  - 設定なし: `[OTEL]` サマリーを stderr に出力（後方互換）

### テスト
- `v227000_tests` 5 件追加
```

### 5-3: `benchmarks/v22.7.0.json` を新規作成

```json
{
  "version": "22.7.0",
  "timestamp": "2026-06-21T00:00:00Z",
  "_note": "OpenTelemetry integration milestone snapshot.",
  "metrics": {
    "test_count": 1879,
    "otel_span_types": 1,
    "otlp_protocol": "http-json"
  },
  "otel_features": {
    "stage_spans":          { "achieved": true, "version": "v22.7.0" },
    "otlp_http_export":     { "achieved": true, "version": "v22.7.0" },
    "stderr_fallback":      { "achieved": true, "version": "v22.7.0" },
    "env_var_endpoint":     { "achieved": true, "version": "v22.7.0" }
  }
}
```

### 5-4: `site/content/docs/cli/otel.mdx` を新規作成

以下の内容を含む MDX ドキュメント:

- `fav run --trace` の基本的な使い方
- `OTEL_EXPORTER_OTLP_ENDPOINT` の設定方法
- Jaeger All-in-One の起動例（`docker run` コマンド）
- Span 構造（`stage:<name>` 形式の span）
- Span Attributes 一覧表
- `[OTEL]` stderr 出力例（エンドポイント未設定時）
- スコープ外（OTLP/gRPC、traceparent、root span）への言及
