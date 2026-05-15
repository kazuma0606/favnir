# Favnir v4.0.0 Implementation Plan

## Theme: 残件一括消化 — gRPC 完全実装 + pipe match + pattern guard + スタックトレース

---

## Phase 0: バージョン更新

`fav/Cargo.toml` の version を `"4.0.0"` に更新。
`fav/src/main.rs` のヘルプテキスト・バージョン文字列を更新。
追加 Cargo 依存なし（tonic/tokio/prost は v3.9.0 で導入済み）。

---

## Phase 1: gRPC serve_raw / serve_stream_raw — ハンドラ dispatch 実装

### 設計: チャネル型

```rust
// 各リクエストが自分の応答チャネルを持つ
type GrpcRequestMsg = (
    String,                              // handler_name (e.g. "handle_get_user")
    Vec<u8>,                             // proto bytes（framing 解除済み）
    std::sync::mpsc::SyncSender<Vec<u8>> // 応答用チャネル（proto bytes）
);
```

### `grpc_serve_impl` 関数（vm.rs）

`grpc_spawn_placeholder_server` を削除し、本実装の `grpc_serve_impl` に差し替える。

```rust
fn grpc_serve_impl(
    port: i64,
    service_name: String,
    streaming: bool,
    req_tx: std::sync::mpsc::SyncSender<GrpcRequestMsg>,
) -> Result<(), String>
```

1. 専用スレッドで `tokio::runtime::Builder::new_multi_thread().enable_all().build()` を起動
2. hyper / tonic で HTTP/2 サーバーをポートにバインド
3. リクエスト受信 → `decode_grpc_frame` → `proto_bytes`
4. URL パスから `/ServiceName/MethodName` を分解 → `pascal_to_snake(method)` → `handler_name`
5. `(res_tx, res_rx)` を生成（SyncSender を req_msg に含める）
6. `tokio::task::spawn_blocking(move || req_tx.send((handler_name, proto_bytes, res_tx)).unwrap())`
7. `res_rx.recv()` でブロッキング待機 → `encode_grpc_frame` → HTTP/2 レスポンス

### VM メインスレッドループ（`Grpc.serve_raw` アーム）

```rust
"Grpc.serve_raw" => {
    // ... port, service_name の取り出し ...

    let (req_tx, req_rx) = std::sync::mpsc::sync_channel::<GrpcRequestMsg>(0);
    grpc_serve_impl(port, service_name.clone(), false, req_tx)?;
    eprintln!("Listening on 0.0.0.0:{port} (gRPC / HTTP2)");

    // VM メインスレッドが無限ループでリクエストを処理
    loop {
        let (handler_name, proto_bytes, res_tx) = match req_rx.recv() {
            Ok(msg) => msg,
            Err(_) => break,  // サーバースレッドが終了したら終わり
        };
        let fn_idx = artifact.fn_idx_by_name(&handler_name)?;
        let payload = proto_bytes_to_string_map(&proto_bytes)?;
        let req_record = VMValue::Record(payload.into_iter().map(|(k, v)| (k, VMValue::Str(v))).collect());
        let result = self.invoke_function(artifact, fn_idx, vec![req_record])?;
        let resp_bytes = grpc_vm_value_to_proto_bytes(&result, type_metas)?;
        let _ = res_tx.send(resp_bytes);
    }
    Ok(VMValue::Unit)
}
```

### `grpc_vm_value_to_proto_bytes` ヘルパー

```rust
fn grpc_vm_value_to_proto_bytes(value: &VMValue, type_metas: &TypeMetas) -> Result<Vec<u8>, String> {
    match value {
        VMValue::Variant(tag, Some(payload)) if tag == "ok" => {
            if let VMValue::Record(map) = payload.as_ref() {
                Ok(string_map_to_proto_bytes(&schema_record_to_string_map(map)))
            } else {
                Err("serve_raw ok payload must be Map<String,String>".to_string())
            }
        }
        VMValue::Variant(tag, Some(payload)) if tag == "err" => {
            // gRPC error status をレスポンスに埋め込む（ステータスコードは grpc-status trailer で返す）
            Err(format!("grpc handler returned err"))
        }
        _ => Err(format!("serve_raw handler must return Result<Map, RpcError>"))
    }
}
```

### ストリーミング対応（`Grpc.serve_stream_raw`）

ハンドラが `VMValue::List(items)` を返す場合、各要素を連結した複数 gRPC フレームとして送信:

```rust
VMValue::Variant(tag, Some(inner)) if tag == "ok" => {
    if let VMValue::List(items) = inner.as_ref() {
        let mut body = Vec::new();
        for item in items {
            if let VMValue::Record(map) = item {
                let pb = string_map_to_proto_bytes(&schema_record_to_string_map(map));
                body.extend(encode_grpc_frame(&pb));
            }
        }
        Ok(body)
    } else { /* 通常の ok */ }
}
```

---

## Phase 2: gRPC call_raw — 実 RPC 送受信実装

### tonic 低レベル HTTP/2 クライアント

tonic の `Grpc` client API か、hyper の HTTP/2 クライアントを使う。

**推奨アプローチ（hyper 直接）**:

```rust
// hyper は tonic の依存として既にリンク済み
use hyper::{Client, Request, Body, Uri};
use hyper::client::HttpConnector;
use hyper::client::conn::Builder as ConnBuilder;

rt.block_on(async {
    // HTTP/2 接続を確立
    let uri: Uri = endpoint.parse()?;
    // ... HTTP/2 POST リクエストを送信
    // Content-Type: application/grpc
    // body: encode_grpc_frame(string_map_to_proto_bytes(payload))
    // レスポンスボディを全バイト読み取り
    // decode_grpc_frame(resp_bytes) → proto_bytes
    // proto_bytes_to_string_map(proto_bytes) → Map
})
```

**tonic API アプローチ（代替）**:

```rust
use tonic::transport::Channel;
use tonic::Request;

// prost::bytes::Bytes をそのままやりとりする raw codec
// tonic::codec::ProstCodec を流用
```

いずれのアプローチでも以下を守ること:
- `Content-Type: application/grpc` ヘッダー
- HTTP/2 必須（HTTP/1.1 不可）
- リクエストボディ: `encode_grpc_frame(proto_bytes)`
- レスポンスボディ: `decode_grpc_frame(resp_bytes)` で取り出す

### エラーハンドリング

```rust
// 接続失敗（既存テスト互換）
Err(connect_err) => Ok(err_vm(rpc_error_vm(14, connect_err.to_string())))

// gRPC status エラー（grpc-status trailer が 0 以外）
Err(status_err) => Ok(err_vm(rpc_error_vm(status_code as i64, status_message)))

// 正常
Ok(map) => Ok(ok_vm(VMValue::Record(map...)))
```

---

## Phase 3: gRPC call_stream_raw — バグ修正

v3.9.0 のバグ（`decode_all_grpc_frames(&frame)` でリクエストフレームを decode）を修正。

修正箇所（vm.rs ~6097行目）:

```rust
// Before（バグ）:
let rows = decode_all_grpc_frames(&frame)?  // ← frame はリクエスト

// After（修正）:
let resp_bytes = /* HTTP/2 レスポンスボディを読み取る */;
let rows = decode_all_grpc_frames(&resp_bytes)?  // ← レスポンス
```

実装は Phase 2 の `call_raw` と同じ HTTP/2 送受信コードを使い、
レスポンスボディを `decode_all_grpc_frames` に渡す。

---

## Phase 4: pipe match — パーサー実装

### lexer.rs

変更不要（`|>` と `match` は既存トークン）。

### ast.rs

```rust
// Expr::Pipe の variants を拡張するか、別ノードを追加
Expr::PipeMatch {
    lhs: Box<Expr>,
    arms: Vec<MatchArm>,
    span: Span,
}
```

### parser.rs

`parse_pipe_expr` 内で `|>` の次が `match` キーワードの場合に `PipeMatch` を生成:

```rust
// parse_pipe_expr:
if self.eat_pipe() {
    if self.peek_keyword("match") {
        self.bump(); // consume "match"
        let arms = self.parse_match_arms()?;
        lhs = Expr::PipeMatch { lhs: Box::new(lhs), arms, span };
    } else {
        let rhs = self.parse_call_expr()?;
        lhs = Expr::Pipe { lhs: Box::new(lhs), rhs: Box::new(rhs), span };
    }
}
```

### checker.rs

`PipeMatch` の型チェック:
1. `lhs` の型 `T` を推論
2. 仮の `bind __tmp: T <- lhs` を想定して match アームを検査（既存 `check_match_arms` を再利用）
3. 全アームの型が一致することを確認

### compiler.rs

`PipeMatch` → 中間 bind + match に脱糖してから既存 match コンパイルに委譲:

```rust
Expr::PipeMatch { lhs, arms, .. } => {
    // 1. lhs を評価してスタックに積む
    self.compile_expr(lhs)?;
    // 2. match_arms をそのまま使ってマッチコードを生成（既存 compile_match を使う）
    self.compile_match_on_stack(arms)?;
}
```

---

## Phase 5: pattern guard — パーサー実装

### ast.rs

`MatchArm` に `guard` フィールドを追加:

```rust
struct MatchArm {
    pattern: Pattern,
    guard:   Option<Expr>,  // ← 追加（None = ガードなし）
    body:    Expr,
    span:    Span,
}
```

### parser.rs

`parse_match_arm` で `=>` の前に `where` が来た場合にガード式を解析:

```rust
// parse_match_arm:
let pattern = self.parse_pattern()?;
let guard = if self.eat_keyword("where") {
    Some(self.parse_expr()?)
} else {
    None
};
self.expect_token(Token::FatArrow)?;
let body = self.parse_expr()?;
```

### checker.rs

`check_match_arm` でガード式を検査:

```rust
if let Some(guard_expr) = &arm.guard {
    let guard_ty = self.check_expr(guard_expr)?;
    if guard_ty != Type::Bool {
        self.type_error("E04xx", "match guard must be Bool", guard_expr.span);
    }
}
```

### compiler.rs

ガードありアームのコンパイル:

```rust
// ガードありアームのコード生成:
// 1. パターンマッチ（成功なら継続、失敗なら次アームへ jump）
// 2. ガード式を評価
// 3. Bool が false → 次アームの先頭へ jump（JumpIfFalse 命令を使う）
// 4. Bool が true → アーム本体を実行
```

---

## Phase 6: スタックトレース

### compiler.rs — デバッグ情報の埋め込み

`Artifact` に `debug_info: Vec<Option<DebugInfo>>` を追加:

```rust
struct DebugInfo {
    fn_name: String,
    file:    String,
    line:    u32,
}
```

各 `Call` opcode 生成時に対応する `DebugInfo` を記録する。

### vm.rs — CallFrame スタック

`VM` 構造体に `call_stack: Vec<CallFrame>` を追加:

```rust
struct CallFrame {
    fn_name: String,
    file:    String,
    line:    u32,
}
```

`invoke_function` 入口で `push`、出口で `pop`。

### エラー表示

```rust
fn format_stack_trace(frames: &[CallFrame]) -> String {
    let mut s = String::new();
    for frame in frames.iter().rev().take(50) {
        s.push_str(&format!("  at {} ({}:{})\n", frame.fn_name, frame.file, frame.line));
    }
    if frames.len() > 50 {
        s.push_str(&format!("  ... {} more frames\n", frames.len() - 50));
    }
    s
}
```

ランタイムエラー（`Err(msg)` が VM トップレベルに到達した時）に `format_stack_trace` を付加:

```
RuntimeError: {msg}
{stack_trace}
```

---

## Phase 7: テスト追加

### `vm_stdlib_tests.rs`

- `grpc_serve_call_raw_roundtrip` — ローカルポートにサーバーを起動してクライアントから呼び出す E2E テスト
  （注: テストのタイムアウトに注意。ポートは `0`（OS 割り当て）か専用テストポートを使う）
- `grpc_call_raw_bad_host_returns_err` — 既存テスト `grpc_call_raw_returns_err_on_bad_host` のリネームで動作確認
- `grpc_encode_decode_grpc_frame_roundtrip` — v3.9.0 で未追加だったテストを追加

### `driver.rs` 統合テスト

- `grpc_serve_raw_dispatches_handler` — serve_raw が実際にハンドラを呼び出すことを確認
- `pipe_match_basic` — `|> match { Ok ... Err ... }` の型チェックと実行
- `pipe_match_type_error` — アームの型不一致でエラー
- `pattern_guard_basic` — `where` 付きアームが条件付きでマッチする
- `pattern_guard_fallthrough` — ガード false で次アームにフォールスルー
- `stack_trace_displayed_on_runtime_error` — エラー時にスタックトレースが出力される

### `runes/grpc/grpc.test.fav`

- `grpc_serve_and_call_roundtrip` — `grpc.serve` + `grpc.call` の Favnir レベル E2E テスト

---

## Phase 8: examples + docs

- `fav/examples/grpc_e2e_demo/src/main.fav` — サーバーとクライアントが実際に通信するデモ
- `versions/v4.0.0/langspec.md` — pipe match / pattern guard / スタックトレースの仕様書
- `versions/v4.0.0/migration-guide.md` — v3.x → v4.0.0 の移行ガイド
- `versions/v4.0.0/progress.md` — 全フェーズ完了時に更新
- `memory/MEMORY.md` — v4.0.0 完了状態に更新

---

## 実装順序と依存関係

```
Phase 0: バージョン更新（独立）
Phase 1: serve_raw dispatch（tonic スレッド ↔ VM 同期が核心）
Phase 2: call_raw 実装（Phase 1 と独立）
Phase 3: call_stream_raw バグ修正（Phase 2 の HTTP/2 コードを流用）
Phase 4: pipe match（lexer/ast/parser/checker/compiler、独立）
Phase 5: pattern guard（AST MatchArm 拡張 → Phase 4 と密接）
  ※ Phase 4 と Phase 5 は同じ match/arm に触るため、合わせて実装するのが望ましい
Phase 6: スタックトレース（独立。compiler.rs + vm.rs）
Phase 7: テスト（Phase 1-6 完了後）
Phase 8: docs（最後）
```

---

## 実装上の最重要注意点

### serve_raw の VM スレッド同期

`invoke_function` は `&mut self`（VM への可変参照）を必要とする。
`self` は `!Send` なのでスレッド間を渡せない。
**必ず** VM メインスレッドが `req_rx.recv()` のループで処理を担当し、
tonic スレッドはリクエストデータを送るだけにすること。

```
❌ NG: tokio スレッド内で invoke_function を呼ぶ
✅ OK: VM メインスレッドが req_rx.recv() → invoke_function → res_tx.send()
```

### call_raw の tokio ランタイム

`std::thread::spawn` + `rt.block_on(...)` パターンを維持すること。
VM 実行コンテキストには tokio ランタイムがないため、`tokio::spawn` は使えない。

### pattern guard のフォールスルー

ガード不成立時は「次のアームの先頭」へ jump する。
「ブロック全体の外」に jump してはならない（他のアームを試す必要があるため）。
コード生成時にパッチアップ（backpatching）で jump 先を確定する。

### スタックトレースのパフォーマンス

`call_stack.push/pop` のオーバーヘッドは軽微だが、
スタックは最大 50 フレームに制限して無制限な再帰でもメモリを食わないようにする。
