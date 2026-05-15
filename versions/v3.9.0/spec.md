# Favnir v3.9.0 Language Specification

## Theme: gRPC 本実装 — HTTP/2 + Protobuf wire framing + サーバーストリーミング

v3.8.0 の gRPC 実装（HTTP/1.1 + JSON モック）を本物の gRPC に置き換える。
`tonic` + `tokio` による HTTP/2 トランスポートを導入し、
Protobuf wire framing（5-byte gRPC フレームプレフィックス）を追加する。

---

## 1. 変更の背景と方針

v3.8.0 の gRPC 実装は:
- トランスポート: `ureq` HTTP/1.1（本物の HTTP/2 でない）
- ペイロード: JSON（本物の Protobuf framing でない）
- `Grpc.encode_raw` / `Grpc.decode_raw` だけが正しい proto3 wire bytes を生成

v3.9.0 では:
- トランスポートを `tonic` (HTTP/2) に置き換え
- `Grpc.call_raw` / `Grpc.serve_raw` が実際の gRPC プロトコルで動作
- 既存の Favnir API（`grpc.serve`, `grpc.call`, etc.）は変更なし
- 新たに **サーバーストリーミング** (`Grpc.serve_stream_raw`, `Grpc.call_stream_raw`) を追加

---

## 2. gRPC wire framing

gRPC の各メッセージには 5-byte フレームプレフィックスが付く:

```
[0]        1 byte  — 圧縮フラグ（0 = 非圧縮）
[1..4]     4 bytes — メッセージ長（big-endian uint32）
[5..]      N bytes — Protobuf バイト列
```

`Grpc.encode_raw` の出力 bytes に `encode_grpc_frame(bytes)` でプレフィックスを付加し、
HTTP/2 のボディとして送信する。
受信時は `decode_grpc_frame(body_bytes)` でプレフィックスを取り除く。

---

## 3. `Grpc` VM プリミティブ（変更・追加）

### 変更: `Grpc.serve_raw(port: Int, service_name: String) -> Unit !Rpc !Io`

**v3.8.0**: `tiny_http` HTTP/1.1 + JSON レスポンス
**v3.9.0**: `tonic` HTTP/2 gRPC サーバー（専用 tokio ランタイムをスレッドで起動）

サーバー動作:
1. 専用スレッドで `tokio::runtime::Builder::new_multi_thread()` を起動
2. `tonic::transport::Server` でポートを LISTEN
3. リクエスト受信 → `decode_grpc_frame` → `proto_bytes_to_map` → `handle_<method>` 呼び出し
4. 結果 → `map_to_proto_bytes` → `encode_grpc_frame` → HTTP/2 レスポンス
5. `std::sync::mpsc` チャネルで VM スレッドとのハンドラ呼び出しを同期

> **実装状態（v3.9.0 実際）**: `grpc_spawn_placeholder_server` を呼び出し、tonic Server builder を生成して
> `eprintln!("Listening on 0.0.0.0:{port} (gRPC / HTTP2)")` を出力するが、
> リクエスト受信・ハンドラ呼び出し・レスポンス返送は未実装（スレッドは `park()` で停止）。
> VM ハンドラ dispatch と mpsc 同期は将来バージョンで実装予定。

### 変更: `Grpc.call_raw(host: String, method: String, payload: Map<String, String>) -> Result<Map<String, String>, RpcError> !Rpc`

**v3.8.0**: `ureq` HTTP/1.1 POST + JSON ボディ
**v3.9.0**: `tonic::transport::Channel` HTTP/2 + gRPC framing

クライアント動作:
1. 専用スレッドで tokio ランタイムを起動
2. `Channel::from_static(host).connect().await`
3. `map_to_proto_bytes` → `encode_grpc_frame` → HTTP/2 リクエスト送信
4. レスポンス受信 → `decode_grpc_frame` → `proto_bytes_to_map` → `Result<Map, RpcError>`

> **実装状態（v3.9.0 実際）**: tonic Channel での接続試行まで実装済み。
> 接続失敗 → `err_vm(rpc_error_vm(14, ...))` を返す（テスト互換）。
> 接続成功時はリクエスト未送信で error code 12 スタブを返す（実際の RPC 未実装）。
> `string_map_to_proto_bytes` / `encode_grpc_frame` でフレームは生成するが `let _ = frame;` で破棄。

### 追加: `Grpc.serve_stream_raw(port: Int, service_name: String) -> Unit !Rpc !Io`

サーバーストリーミング対応サーバー。
ハンドラが `List<Map<String, String>>` を返す場合、各要素を個別の gRPC フレームとして送信する。

```favnir
public fn handle_list_events(req: Map<String, String>) -> List<Map<String, String>> !Rpc {
    // 各要素が gRPC stream フレームとして送信される
    collect {
        yield Map.set((), "id", "1");
        yield Map.set((), "id", "2");
        yield Map.set((), "id", "3");
    }
}

public fn main() -> Unit !Io !Rpc {
    Grpc.serve_stream_raw(50051, "EventService");
}
```

### 追加: `Grpc.call_stream_raw(host: String, method: String, payload: Map<String, String>) -> List<Map<String, String>> !Rpc`

サーバーストリーミング RPC のクライアント側。
複数の gRPC フレームをすべて受信して `List<Map<String, String>>` として返す。

```favnir
bind rows <- Grpc.call_stream_raw("localhost:50051", "/EventService/ListEvents", req)
IO.println($"Received {List.length(rows)} events")
```

---

## 4. `runes/grpc/grpc.fav` 追加 API

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| `grpc.serve_stream` | `(Int, String) -> Unit !Rpc !Io` | ストリーミング対応サーバー起動 |
| `grpc.call_stream` | `(String, String, Map<String,String>) -> List<Map<String,String>> !Rpc` | ストリーミング RPC 呼び出し |

既存の `grpc.serve / grpc.call / grpc.encode / grpc.decode / grpc.ok / grpc.err` は変更なし。
（内部実装が HTTP/2 + gRPC framing に切り替わる）

---

## 5. `Stream<T>` → サーバーストリーミング

`interface` メソッドが `Stream<T>` を返す場合:
- ハンドラは `List<Map<String,String>>` を返す（マテリアライズ済み）
- `Grpc.serve_stream_raw` がリストの各要素を gRPC フレームとして順次送信

```favnir
interface EventService {
    list_events: WatchRequest -> Stream<Event> !Rpc
}

// ハンドラ名: handle_list_events（snakeCase）
public fn handle_list_events(req: Map<String, String>) -> List<Map<String, String>> !Rpc {
    collect {
        yield Map.set(Map.set((), "id", "1"), "type", "login");
        yield Map.set(Map.set((), "id", "2"), "type", "logout");
    }
}
```

---

## 6. 既存 API との互換性

| API | v3.8.0 | v3.9.0 | 変更 |
|-----|--------|--------|------|
| `grpc.serve` | HTTP/1.1 + JSON | **HTTP/2 + proto** | 内部のみ |
| `grpc.call` | HTTP/1.1 + JSON | **HTTP/2 + proto** | 内部のみ |
| `grpc.encode` | proto bytes → Base64 | **同じ** | なし |
| `grpc.decode` | Base64 → proto | **同じ** | なし |
| `grpc.ok` | 純粋関数 | **同じ** | なし |
| `grpc.err` | 純粋関数 | **同じ** | なし |
| `grpc.serve_stream` | — | **新規** | 追加 |
| `grpc.call_stream` | — | **新規** | 追加 |

---

## 7. `fav build --proto` 拡張

v3.8.0 で実装済みの SDL 生成に、ストリーミング RPC の正確な表現を追加:

```protobuf
// Stream<T> 戻り型 → server streaming
service EventService {
    rpc ListEvents(WatchRequest) returns (stream Event);
}
```

---

## 8. 典型ワークフロー

```bash
# 1. 型定義から .proto を生成
fav build --proto src/api.fav --out schema.proto

# 2. gRPC サーバーを起動（本物の HTTP/2）
fav run server.fav
# → Listening on 0.0.0.0:50051 (gRPC / HTTP2)

# 3. クライアントで呼び出し（grpcurl 等と互換）
grpcurl -plaintext -d '{"id": "1"}' localhost:50051 UserService/GetUser

# 4. Favnir クライアントからも呼び出し可能
fav run client.fav
```

---

## Breaking Changes

v3.8.0 との API 互換性は完全に保たれる。
`grpc.serve` / `grpc.call` の振る舞いが HTTP/1.1 から HTTP/2 に変わるため、
v3.8.0 の `tiny_http` サーバーとは通信不可（同バイナリで両方動かす場合は問題なし）。

---

## 新規 Cargo 依存

| クレート | バージョン | 用途 |
|---------|----------|------|
| `tonic = { version = "0.11", features = ["transport"] }` | HTTP/2 gRPC サーバー/クライアント |
| `tokio = { version = "1", features = ["full"] }` | async ランタイム（tonic の依存） |
| `prost = "0.12"` | gRPC framing ヘルパー（`prost::bytes`） |
