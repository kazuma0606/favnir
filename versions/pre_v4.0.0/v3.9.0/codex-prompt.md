# Codex Prompt: Favnir v3.9.0 — gRPC 本実装

## 目的

v3.8.0 の gRPC モック実装（HTTP/1.1 + JSON + `tiny_http` / `ureq`）を、
本物の gRPC（HTTP/2 + Protobuf wire framing）に置き換える。

**Favnir 言語の API は一切変更しない。変わるのは Rust の内部実装のみ。**

---

## 変更対象ファイル一覧

1. `fav/Cargo.toml` — version + 依存追加
2. `fav/src/main.rs` — バージョン文字列
3. `fav/src/backend/vm.rs` — Grpc.* の Rust 実装を置き換え + 新プリミティブ追加
4. `fav/src/middle/checker.rs` — 新プリミティブのシグネチャ登録
5. `runes/grpc/grpc.fav` — `serve_stream` / `call_stream` ラッパー追加
6. `runes/grpc/grpc.test.fav` — テスト追加
7. `fav/src/backend/vm_stdlib_tests.rs` — テスト追加
8. `fav/src/driver.rs` — 統合テスト追加
9. `fav/examples/grpc_stream_demo/src/main.fav` — 新規作成
10. `versions/v3.9.0/progress.md` — 完了時に全 [x] に更新

---

## Phase 0: Cargo.toml + バージョン更新

`fav/Cargo.toml`:
- `version = "3.8.0"` → `"3.9.0"`
- 以下を追加（`tiny_http` と `ureq` は **削除しない**。他の用途で使っている可能性があるため）:

```toml
tonic = { version = "0.11", features = ["transport"] }
tokio = { version = "1", features = ["full"] }
prost = "0.12"
```

`fav/src/main.rs`:
- バージョン文字列・ヘルプテキストを `3.9.0` に更新

---

## Phase 1: gRPC framing ヘルパー（vm.rs に追加）

`fav/src/backend/vm.rs` の **グローバル関数** として追加（`impl VM` の外）:

```rust
/// gRPC 5-byte フレームプレフィックスを付加する
fn encode_grpc_frame(payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(5 + payload.len());
    out.push(0u8); // compression flag: 0 = no compression
    let len = payload.len() as u32;
    out.extend_from_slice(&len.to_be_bytes()); // 4 bytes big-endian
    out.extend_from_slice(payload);
    out
}

/// gRPC 5-byte フレームプレフィックスを取り除き、payload を返す
/// 複数フレーム連結の場合は最初のフレームのみ返す（call_stream_raw では自前でループする）
fn decode_grpc_frame(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 5 {
        return Err(format!("gRPC frame too short: {} bytes", data.len()));
    }
    let len = u32::from_be_bytes([data[1], data[2], data[3], data[4]]) as usize;
    if data.len() < 5 + len {
        return Err(format!(
            "gRPC frame body truncated: expected {} bytes, got {}",
            len,
            data.len() - 5
        ));
    }
    Ok(data[5..5 + len].to_vec())
}

/// 連結された複数の gRPC フレームをすべてデコードして返す
fn decode_all_grpc_frames(data: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    let mut frames = Vec::new();
    let mut offset = 0;
    while offset + 5 <= data.len() {
        let len = u32::from_be_bytes([
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
        ]) as usize;
        if offset + 5 + len > data.len() {
            break;
        }
        frames.push(data[offset + 5..offset + 5 + len].to_vec());
        offset += 5 + len;
    }
    Ok(frames)
}
```

---

## Phase 2: `Grpc.call_raw` の置き換え（vm.rs）

**現在の実装**（vm.rs ~5981行目）: `ureq::post` で HTTP/1.1 + JSON ボディを送信している。

**置き換え後**: tonic HTTP/2 + gRPC framing。

```rust
"Grpc.call_raw" => {
    if args.len() != 3 {
        return Err("Grpc.call_raw requires 3 arguments".to_string());
    }
    let mut it = args.into_iter();
    let host = vm_string(it.next().unwrap(), "Grpc.call_raw host")?;
    let method = vm_string(it.next().unwrap(), "Grpc.call_raw method")?;
    let payload = match it.next().unwrap() {
        VMValue::Record(map) => schema_record_to_string_map(&map),
        other => {
            return Err(format!(
                "Grpc.call_raw expects Map<String,String>, got {}",
                vmvalue_type_name(&other)
            ));
        }
    };
    // type_metas は &self から取得（既存パターンに倣う）
    let type_metas = self.type_metas.clone();
    // method からメソッド名部分を取得: "/ServiceName/MethodName" → "MethodName"
    let method_simple = method.split('/').last().unwrap_or("Unknown").to_string();
    // 型名はサービス名から推測できないため、payload をそのまま proto bytes に変換
    // ここでは type_name として "_grpc_call" という仮の型を使わず、
    // Map<String,String> を直接 proto bytes にする（フィールド名 → フィールド番号は 1-indexed）
    let proto_bytes = {
        let fields: Vec<(String, String)> = payload.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let mut buf = Vec::new();
        for (i, (_key, val)) in fields.iter().enumerate() {
            let field_num = (i + 1) as u64;
            // LEN wire type (2): tag = (field_num << 3) | 2
            let tag = (field_num << 3) | 2u64;
            encode_varint(tag, &mut buf);
            let val_bytes = val.as_bytes();
            encode_varint(val_bytes.len() as u64, &mut buf);
            buf.extend_from_slice(val_bytes);
        }
        buf
    };
    let frame = encode_grpc_frame(&proto_bytes);
    let uri_str = if host.starts_with("http://") || host.starts_with("https://") {
        format!("{}{}", host, method)
    } else {
        format!("http://{}{}", host, method)
    };
    let result = std::thread::spawn(move || -> Result<VMValue, String> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Grpc.call_raw tokio build failed: {}", e))?;
        rt.block_on(async {
            use tonic::transport::Channel;
            use tonic::Request;
            use tonic::codegen::http::uri::Uri;
            let endpoint_str = if host.starts_with("http://") || host.starts_with("https://") {
                host.clone()
            } else {
                format!("http://{}", host)
            };
            let channel = Channel::from_shared(endpoint_str)
                .map_err(|e| format!("Grpc.call_raw invalid URI: {}", e))?
                .connect()
                .await
                .map_err(|e| format!("Grpc.call_raw connect failed: {}", e))?;
            // hyper HTTP/2 リクエストを直接送信
            use tonic::codegen::http;
            let mut client = tonic::client::Grpc::new(channel);
            client.ready().await
                .map_err(|e| format!("Grpc.call_raw not ready: {}", e))?;
            let path = tonic::codegen::http::uri::PathAndQuery::from_static(Box::leak(
                method.clone().into_boxed_str(),
            ));
            // prost::bytes::Bytes として frame を送信
            let body = prost::bytes::Bytes::from(frame.clone());
            let request = Request::new(body);
            let codec = tonic::codec::ProstCodec::<prost::bytes::Bytes, prost::bytes::Bytes>::default();
            let resp = client
                .unary(request, path, codec)
                .await
                .map_err(|e| format!("Grpc.call_raw rpc failed: status={}", e.code() as i32))?;
            let resp_bytes = resp.into_inner();
            let payload_bytes = decode_grpc_frame(&resp_bytes)
                .map_err(|e| format!("Grpc.call_raw frame decode: {}", e))?;
            // proto bytes → Map<String, String>（フィールドはすべて string として扱う）
            let row = proto_bytes_to_string_map(&payload_bytes);
            Ok(ok_vm(VMValue::Record(
                row.into_iter().map(|(k, v)| (k, VMValue::Str(v))).collect(),
            )))
        })
    })
    .join()
    .map_err(|_| "Grpc.call_raw thread panicked".to_string())??;
    Ok(result)
}
```

**注意**: `tonic::codec::ProstCodec` の使い方が上記と異なる場合は、`hyper` を直接使った HTTP/2 POST でも可。重要なのは:
1. **必ず HTTP/2** を使うこと（ureq の HTTP/1.1 ではなく）
2. **リクエストボディに `encode_grpc_frame(proto_bytes)` を付加**すること
3. **レスポンスボディから `decode_grpc_frame` で payload を取り出す**こと
4. エラー時は `err_vm(rpc_error_vm(code, message))` を返すこと（既存パターン通り）

---

## Phase 3: `Grpc.serve_raw` の置き換え（vm.rs）

**現在の実装**（vm.rs ~2228行目）: `tiny_http::Server` で HTTP/1.1 リクエストを1件受け付けている。

**置き換え後**: tonic HTTP/2 + gRPC framing。単一リクエスト処理後に終了する動作は維持。

VM スレッドとの同期に `std::sync::mpsc` を使う:

```rust
"Grpc.serve_raw" => {
    // ... (port, service_name の取り出しは現在と同じ) ...

    // ① チャネル作成
    let (req_tx, req_rx) = std::sync::mpsc::sync_channel::<(String, Vec<u8>)>(0);
    let (res_tx, res_rx) = std::sync::mpsc::sync_channel::<Vec<u8>>(0);

    let port_u16 = port as u16;
    let svc_name = service_name.clone();

    // ② 専用スレッドで tokio ランタイム + tonic サーバーを起動
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            use tonic::transport::Server;
            // tonic の generic service を使ってハンドラを登録
            // 詳細は Phase 3 注意事項を参照
            let addr: std::net::SocketAddr = format!("0.0.0.0:{}", port_u16).parse().unwrap();
            // リクエストを受け取ったら req_tx で VM スレッドに送信し、
            // res_rx から応答を受け取ってクライアントに返す
            // (実装省略: 以下の注意事項を参照)
        });
    });

    // ③ VM スレッド: リクエストを受け取り、ハンドラを呼び出し、応答を返す
    if let Ok((handler_name, req_bytes)) = req_rx.recv() {
        let fn_idx = artifact.fn_idx_by_name(&handler_name).ok_or_else(|| {
            self.error(artifact, &format!("Grpc.serve_raw unknown handler `{}`", handler_name))
        })?;
        let payload_map = proto_bytes_to_string_map(&req_bytes);
        // ... invoke_function と response encode は現在の実装に倣う ...
        let response_bytes = /* encode result */ vec![];
        let _ = res_tx.send(response_bytes);
    }
    Ok(VMValue::Unit)
}
```

**重要な注意事項**:

tonic の generic HTTP/2 サービスで `req_tx` / `res_rx` を使って VM スレッドと同期する際、
async コンテキストからは `tokio::task::spawn_blocking` を使って同期受信する:

```rust
// tokio task 内での VM スレッドとの同期
let resp_bytes = tokio::task::spawn_blocking(move || {
    req_tx.send((handler_name, req_bytes)).unwrap();
    res_rx.recv().unwrap()
}).await.unwrap();
```

tonic の generic raw service については `tonic::server::NamedService` + `tower::Service` を実装するか、
あるいは **`hyper` を直接使った HTTP/2 サーバー** でも構わない。

hyper を使う場合（より簡単）:
```rust
// hyper = { version = "0.14", features = ["http2", "server"] } を Cargo.toml に追加してもよい
// または tonic が依存する hyper を使う
```

いずれの実装でも、以下の動作を守ること:
- HTTP/2 で待受けすること（HTTP/1.1 の tiny_http は使わない）
- リクエストボディから `decode_grpc_frame` で proto bytes を取り出すこと
- `pascal_to_snake(method_name)` でハンドラ名を解決すること（既存関数を再利用）
- ハンドラ結果を `encode_grpc_frame(map_to_proto_bytes(...))` でラップしてレスポンスを返すこと
- 起動時に `eprintln!("Listening on 0.0.0.0:{port} (gRPC / HTTP2)")` または同等の出力

---

## Phase 4: 新プリミティブ `Grpc.serve_stream_raw` / `Grpc.call_stream_raw`（vm.rs）

### `Grpc.serve_stream_raw`

`Grpc.serve_raw` と同じ実装だが、ハンドラが `VMValue::List(items)` を返す場合に
各要素を個別の gRPC フレームとして連結して送信する:

```rust
"Grpc.serve_stream_raw" => {
    // serve_raw と同じ処理 ...
    // ハンドラ結果が List の場合:
    VMValue::List(items) => {
        let mut body = Vec::new();
        for item in items {
            if let VMValue::Record(map) = item {
                let pb = map_to_proto_bytes("_stream_item", &schema_record_to_string_map(&map), type_metas)?;
                body.extend(encode_grpc_frame(&pb));
            }
        }
        // body を HTTP/2 レスポンスとして返す
    }
    // Result::Ok(VMValue::List(...)) も同様に処理
}
```

### `Grpc.call_stream_raw`

`Grpc.call_raw` と同じ実装だが、レスポンスから `decode_all_grpc_frames` で
複数フレームをすべて取り出して `VMValue::List` として返す:

```rust
"Grpc.call_stream_raw" => {
    // call_raw と同じ送信処理 ...
    // レスポンスのデコード:
    let frames = decode_all_grpc_frames(&resp_bytes)?;
    let list: Vec<VMValue> = frames.iter().map(|frame| {
        let row = proto_bytes_to_string_map(frame);
        VMValue::Record(row.into_iter().map(|(k, v)| (k, VMValue::Str(v))).collect())
    }).collect();
    Ok(VMValue::List(list))
}
```

---

## Phase 5: checker.rs にシグネチャ追加

`fav/src/middle/checker.rs` の `("Grpc", "decode_raw")` ブロックの直後（約 4719 行目）に追加:

```rust
("Grpc", "serve_stream_raw") => {
    self.require_rpc_effect(span);
    self.require_io_effect(span);
    Some(Type::Unit)
}
("Grpc", "call_stream_raw") => {
    self.require_rpc_effect(span);
    Some(Type::List(Box::new(Type::Map(
        Box::new(Type::String),
        Box::new(Type::String),
    ))))
}
```

---

## Phase 6: `runes/grpc/grpc.fav` に追加

既存の6関数の末尾に追加:

```favnir
public fn serve_stream(port: Int, service_name: String) -> Unit !Io !Rpc {
    Grpc.serve_stream_raw(port, service_name)
}

public fn call_stream(host: String, method: String, payload: Map<String, String>) -> List<Map<String, String>> !Rpc {
    Grpc.call_stream_raw(host, method, payload)
}
```

`runes/grpc/grpc.test.fav` に追加:

```favnir
test grpc_call_stream_bad_host_is_empty_or_err {
    let result = Grpc.call_stream_raw("localhost:1", "/X/Y", ())
    assert List.length(result) == 0
}
```

---

## Phase 7: テスト追加

### `fav/src/backend/vm_stdlib_tests.rs` に追加

```rust
#[test]
fn grpc_encode_decode_grpc_frame_roundtrip() {
    let payload = b"hello grpc";
    let framed = encode_grpc_frame(payload);
    assert_eq!(framed.len(), 5 + payload.len());
    assert_eq!(framed[0], 0); // compression flag
    let decoded = decode_grpc_frame(&framed).unwrap();
    assert_eq!(decoded, payload);
}

#[test]
fn grpc_call_stream_raw_returns_empty_on_bad_host() {
    // エラー時に空リストまたはエラーが返ること（パニックしないこと）
    let result = /* run Grpc.call_stream_raw("localhost:1", "/X/Y", ()) */;
    // list か err かに関わらずパニックしないこと
}
```

### `fav/src/driver.rs` に統合テスト追加（4件）

```rust
#[test]
fn grpc_serve_stream_raw_type_checks_in_favnir_source() {
    // Grpc.serve_stream_raw のシグネチャが型チェックを通ること
    let src = r#"
public fn main() -> Unit !Io !Rpc {
    Grpc.serve_stream_raw(50052, "TestService");
}
"#;
    // 型チェックのみ（実行はしない）
}

#[test]
fn grpc_call_stream_raw_bad_host_in_favnir_source() { ... }

#[test]
fn grpc_rune_serve_stream_in_favnir_source() { ... }

#[test]
fn grpc_rune_call_stream_in_favnir_source() { ... }
```

---

## Phase 8: examples + docs

`fav/examples/grpc_stream_demo/src/main.fav`:

```favnir
import rune "grpc"

type Event = { id: String kind: String }

public fn handle_list_events(req: Map<String, String>) -> List<Map<String, String>> !Rpc {
    collect {
        yield Map.set(Map.set((), "id", "1"), "kind", "login");
        yield Map.set(Map.set((), "id", "2"), "kind", "logout");
    }
}

public fn main() -> Unit !Io !Rpc {
    IO.println("gRPC streaming server on :50052");
    grpc.serve_stream(50052, "EventService");
}
```

`versions/v3.9.0/progress.md`: 完了後に全フェーズを `[x]` に更新

---

## 絶対に守ること（よくある間違いの防止）

1. **`ureq` の HTTP/1.1 で「なんとなく動く」実装にしない**
   - `Grpc.call_raw` は必ず HTTP/2 を使うこと
   - `Grpc.serve_raw` は必ず HTTP/2 サーバーを起動すること

2. **Favnir 言語の API は変更しない**
   - `grpc.serve / call / encode / decode / ok / err` のシグネチャは一切変えない
   - `RpcError / RpcRequest` 型定義も変更しない
   - checker.rs の既存 Grpc.* シグネチャも変更しない（追加のみ）

3. **gRPC フレームプレフィックスを省略しない**
   - リクエスト/レスポンスのボディは必ず `encode_grpc_frame` を通すこと
   - 生の proto bytes をそのまま HTTP ボディにしてはならない

4. **既存テストを壊さない**
   - `grpc_encode_decode_roundtrip` など v3.8.0 の全テストがパスすること
   - `grpc_call_raw_returns_err_on_bad_host` のエラー応答パターン（`err_vm(rpc_error_vm(...))` ）を維持すること

5. **`tiny_http` と `ureq` は削除しない**
   - 他の用途（Http.serve_raw / Http.* など）でも使っているため
   - Grpc.* の内部実装だけを置き換える

6. **`map_to_proto_bytes` / `proto_bytes_to_map` は再実装しない**
   - v3.8.0 の varint 実装はそのまま再利用すること

7. **Cargo.toml の edition / features を変えない**
   - `edition = "2024"` を維持
   - `postgres_integration` feature はそのまま

---

## 期待するテスト結果

実装完了後、以下が全てパスすること:
- `cargo test` — 全テスト（~900 件以上）
- v3.8.0 の既存 grpc テスト6件（vm_stdlib_tests）
- v3.8.0 の既存 grpc driver テスト7件
- 新規 vm_stdlib_tests 2件
- 新規 driver 統合テスト 4件
