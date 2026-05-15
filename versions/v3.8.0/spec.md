# Favnir v3.8.0 Language Specification

## Theme: `grpc` rune — gRPC サービス定義 + Protobuf 出力

Favnir の型定義から gRPC サービスを直接公開し、高速バイナリ通信とストリーミングを提供する。
`interface` + `Stream<T>` がそのまま Protobuf サービス定義にマッピングされる。

---

## 1. `!Rpc` エフェクト

gRPC の送受信を行う関数に付与するエフェクト。

```favnir
public fn main() -> Unit !Io !Rpc {
    Grpc.serve_raw(50051, "UserService");
}
```

---

## 2. 組み込み型

### `RpcError`

```favnir
type RpcError = {
    code:    Int     // gRPC ステータスコード（0=OK, 1=CANCELLED, 2=UNKNOWN, ...）
    message: String
}
```

### `RpcRequest`

```favnir
type RpcRequest = {
    method:  String              // RPC メソッド名
    payload: Map<String, String> // デシリアライズ済みリクエストフィールド
}
```

---

## 3. `Grpc` VM プリミティブ

### `Grpc.serve_raw(port: Int, service_name: String) -> Unit !Rpc !Io`

指定ポートで RPC サーバーを起動（ブロッキング）。
`service_name` に登録済み Favnir ハンドラ関数のプレフィックスを渡す。

内部実装: `tiny_http` によるシングルリクエスト受付 (HTTP/1.1)。
リクエストボディは JSON → `Map<String, String>` にデシリアライズ後、
`handle_<method_name_snake_case>` 命名規則のハンドラ関数を VM コール。
レスポンスは Base64 エンコード済み Protobuf wire format で返す。

> v3.8.0 は HTTP/1.1 + 独自 wire format による gRPC 互換実装。
> 本物の gRPC (HTTP/2 + TLS) 対応は将来バージョンで `tonic` 統合時に提供予定。

```favnir
public fn main() -> Unit !Io !Rpc {
    IO.println("Starting gRPC server on :50051...");
    Grpc.serve_raw(50051, "UserService");
}
```

### `Grpc.call_raw(host: String, method: String, payload: Map<String, String>) -> Result<Map<String, String>, RpcError> !Rpc`

gRPC ユニタリ呼び出し（クライアント側）。

```favnir
bind result <- Grpc.call_raw("localhost:50051", "/UserService/GetUser", row)
```

### `Grpc.encode_raw(type_name: String, row: Map<String, String>) -> String`

`Map<String, String>` を Protobuf バイト列（Base64）にエンコード。

### `Grpc.decode_raw(type_name: String, encoded: String) -> Map<String, String>`

Protobuf バイト列（Base64）を `Map<String, String>` にデコード。

---

## 4. `runes/grpc/grpc.fav` 公開 API

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| `grpc.serve` | `(Int, String) -> Unit !Rpc !Io` | gRPC サーバー起動 |
| `grpc.call` | `(String, String, Map<String,String>) -> Result<Map<String,String>, RpcError> !Rpc` | ユニタリ RPC 呼び出し |
| `grpc.encode` | `(String, Map<String,String>) -> String` | Protobuf エンコード |
| `grpc.decode` | `(String, String) -> Map<String,String>` | Protobuf デコード |
| `grpc.ok` | `Map<String,String> -> Result<Map<String,String>, RpcError>` | 成功レスポンス生成 |
| `grpc.err` | `(Int, String) -> Result<Map<String,String>, RpcError>` | エラーレスポンス生成 |

```favnir
import rune "grpc"

type GetUserRequest  = { id: Int }
type GetUserResponse = { id: Int name: String email: String }

public fn handle_get_user(req: Map<String, String>) -> Result<Map<String, String>, RpcError> !Rpc {
    bind id_str <- Option.unwrap_or(Map.get(req, "id"), "0")
    // ... lookup user ...
    grpc.ok(Map.set(Map.set(Map.set((), "id", id_str), "name", "Alice"), "email", "alice@example.com"))
}

public fn main() -> Unit !Io !Rpc {
    IO.println("gRPC server on :50051");
    grpc.serve(50051, "UserService");
}
```

---

## 5. `fav build --proto` — Protobuf スキーマ生成

`fav build` コマンドの `--proto` フラグ。AST を静的走査して `.proto` ファイルを生成する。

```bash
fav build --proto src/main.fav --out schema.proto
fav build --proto src/main.fav          # stdout に出力
```

### 型マッピング

| Favnir 型 | Protobuf 型 |
|----------|------------|
| `Int` | `int64` |
| `Float` | `double` |
| `String` | `string` |
| `Bool` | `bool` |
| `Option<T>` | `optional T` |
| `List<T>` | `repeated T` |
| `Result<T, E>` | `T`（エラーは gRPC status へ） |
| `Stream<T>` | `stream T`（サーバーストリーミング） |

### interface → service

```favnir
type GetUserRequest  = { id: Int }
type GetUserResponse = { id: Int name: String email: String }
type UserList        = { users: List<GetUserResponse> }

interface UserService {
    get_user:   GetUserRequest -> Result<GetUserResponse, RpcError>  !Rpc
    list_users: Unit           -> Stream<GetUserResponse>            !Rpc
}
```

生成される `schema.proto`:
```protobuf
syntax = "proto3";

message GetUserRequest {
    int64 id = 1;
}

message GetUserResponse {
    int64 id = 1;
    string name = 2;
    string email = 3;
}

message UserListResponse {
    repeated GetUserResponse users = 1;
}

service UserService {
    rpc GetUser(GetUserRequest) returns (GetUserResponse);
    rpc ListUsers(google.protobuf.Empty) returns (stream GetUserResponse);
}
```

---

## 6. `fav infer --proto` — .proto から Favnir 型定義を生成

既存の `.proto` ファイルから Favnir 型定義を自動生成する。
既存 gRPC サービスを Favnir で書き直す入口。

```bash
fav infer --proto users.proto --out schema/users.fav
fav infer --proto users.proto           # stdout に出力
```

### 変換ルール

| Protobuf 型 | Favnir 型 |
|-----------|---------|
| `int32` / `int64` | `Int` |
| `float` / `double` | `Float` |
| `string` | `String` |
| `bool` | `Bool` |
| `repeated T` | `List<T>` |
| `optional T` | `Option<T>` |
| `message` | `type` 定義 |
| `service` + `rpc` | `interface` 定義 |
| `stream T`（戻り型） | `Stream<T>` |

```protobuf
// 入力: users.proto
message User {
    int64 id = 1;
    string name = 2;
}
service UserService {
    rpc GetUser(GetUserRequest) returns (User);
}
```

```favnir
// 出力: users.fav（auto-generated by `fav infer --proto users.proto`）
type User = { id: Int name: String }

interface UserService {
    get_user: GetUserRequest -> Result<User, RpcError> !Rpc
}
```

---

## 7. `Stream<T>` → サーバーストリーミング RPC

`interface` のメソッドが `Stream<T>` を返す場合、自動的にサーバーストリーミング RPC にマッピングされる。

```favnir
interface EventService {
    watch_events: WatchRequest -> Stream<Event> !Rpc
}
```

Favnir 側のハンドラは `List<T>` を返すか、`Stream<T>` を返す（マテリアライズされてから送信）。

---

## 8. 典型ワークフロー

```bash
# 既存 .proto から型定義を生成
fav infer --proto users.proto --out schema/users.fav

# Favnir でサービスを実装
fav run server.fav

# .proto を再生成（クライアントコード生成ツールと連携）
fav build --proto src/main.fav --out schema.proto

# gRPC クライアントテスト
fav run client.fav
```

---

## Breaking Changes

v3.7.0 との破壊的変更なし。

---

## 新規 Cargo 依存

**なし** — v3.8.0 は既存の依存のみで実装。

| 既存クレート | 用途 |
|------------|------|
| `tiny_http = "0.12"` | RPC サーバー（HTTP/1.1 ベース） |
| `ureq = "2"` | RPC クライアント（HTTP/1.1 ベース） |
| `base64 = "0.22"` | Protobuf wire bytes の Base64 エンコード |

Protobuf wire format エンコード/デコードは `prost` を使わず純粋 Rust で実装
（varint エンコーディング、フィールドタグ、LEN wire type を手動実装）。

> 将来の v3.x で `tonic` + `tokio` を導入し、本物の HTTP/2 gRPC に移行予定。
