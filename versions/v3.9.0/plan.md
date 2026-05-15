# Favnir v3.9.0 Implementation Plan

## Theme: gRPC 本実装 — HTTP/2 + Protobuf wire framing + サーバーストリーミング

---

## Phase 0: バージョン更新 + 依存追加

`fav/Cargo.toml` の version を `"3.9.0"` に更新し、tonic/tokio/prost を追加する。

```toml
[dependencies]
tonic = { version = "0.11", features = ["transport"] }
tokio = { version = "1", features = ["full"] }
prost = "0.12"
```

`main.rs` のバージョン文字列・ヘルプテキストを `3.9.0` に更新。

---

## Phase 1: gRPC framing ヘルパー実装

`vm.rs` に gRPC 5-byte フレームプレフィックスのエンコード/デコードを追加する。
既存の varint 実装（`encode_varint` / `decode_varint`）はそのまま再利用。

```rust
fn encode_grpc_frame(payload: &[u8]) -> Vec<u8>
fn decode_grpc_frame(data: &[u8]) -> Result<Vec<u8>, String>
```

- `encode_grpc_frame`: `[0x00, len_be_4bytes..., payload...]`
- `decode_grpc_frame`: 先頭 5 バイトを検証し payload を返す

---

## Phase 2: `Grpc.call_raw` — HTTP/2 へ移行

現行の `ureq::post` による HTTP/1.1 + JSON 実装を `tonic::transport::Channel` + gRPC framing に置き換える。

実装方針:
1. `std::thread::spawn` で専用スレッドを起動
2. `tokio::runtime::Builder::new_current_thread().build()` で tokio ランタイム生成
3. `runtime.block_on(async { ... })` で以下を実行:
   - `Channel::from_shared(host)?.connect().await`
   - `map_to_proto_bytes` → `encode_grpc_frame` → HTTP/2 リクエスト送信
   - レスポンス受信 → `decode_grpc_frame` → `proto_bytes_to_map`
4. `thread::JoinHandle` を `join()` して結果を取得
5. ネットワークエラー → `RpcError { code: 14, message: ... }` に変換

---

## Phase 3: `Grpc.serve_raw` — HTTP/2 へ移行

現行の `tiny_http` によるシングルリクエスト処理を `tonic` の generic gRPC サーバーに置き換える。

実装方針:
1. `std::thread::spawn` で専用スレッドを起動
2. `tokio::runtime::Builder::new_multi_thread().build()` で tokio ランタイム生成
3. tonic の `Routes` + `tower::ServiceFn` によるジェネリックハンドラ登録
4. リクエスト受信フロー:
   - HTTP/2 ボディ読み取り → `decode_grpc_frame` → `proto_bytes_to_map`
   - ハンドラ名 = `handle_<method_name_snake_case>` (既存 `pascal_to_snake` 再利用)
   - VM ハンドラ呼び出し (`std::sync::mpsc` で VM スレッドと同期)
   - 結果 → `map_to_proto_bytes` → `encode_grpc_frame` → HTTP/2 レスポンス
5. `IO.println` でポート・プロトコルを出力: `"Listening on 0.0.0.0:{port} (gRPC / HTTP2)"`

VM スレッドとの同期:
```
tokio thread                VM thread
    |                           |
    | --- (method, args) -----> |
    | <-- result --------------- |
```
`std::sync::mpsc::channel` で `(String, Vec<VMValue>)` → `VMValue` を受け渡し。

---

## Phase 4: `Grpc.serve_stream_raw` + `Grpc.call_stream_raw` 追加

### `Grpc.serve_stream_raw`

`Grpc.serve_raw` の実装をベースに、ハンドラが `VMValue::List` を返す場合に
各要素を個別の gRPC フレームとしてストリーム送信するよう拡張。

- ハンドラ戻り型が `VMValue::List(items)` → 各 item を `map_to_proto_bytes` → `encode_grpc_frame` → チャンクとして送信
- `Result::Ok(VMValue::List(...))` も同様に処理

### `Grpc.call_stream_raw`

`Grpc.call_raw` の実装をベースに、複数 gRPC フレームを全受信して `VMValue::List` として返す。

- レスポンスボディをバイト列として読み取り
- `decode_grpc_frame` を繰り返し呼び出し（オフセット管理）
- 各フレーム → `proto_bytes_to_map` → `VMValue::Record`
- 全 Record を `VMValue::List` に詰めて返す

---

## Phase 5: checker.rs + compiler.rs シグネチャ追加

`checker.rs`:
- `Grpc.serve_stream_raw(port: Int, service_name: String) -> Unit !Rpc !Io`
- `Grpc.call_stream_raw(host: String, method: String, payload: Map<String,String>) -> List<Map<String,String>> !Rpc`

`compiler.rs`:
- 既存の `"Grpc"` 登録ループで自動カバーされるため追加不要
- ただし新しいメソッド名 `serve_stream_raw` / `call_stream_raw` が認識されているか確認

---

## Phase 6: `runes/grpc/grpc.fav` 拡張

既存の 6 関数に `serve_stream` / `call_stream` を追加:

```favnir
public fn serve_stream(port: Int, service_name: String) -> Unit !Io !Rpc {
    Grpc.serve_stream_raw(port, service_name)
}

public fn call_stream(host: String, method: String, payload: Map<String, String>) -> List<Map<String, String>> !Rpc {
    Grpc.call_stream_raw(host, method, payload)
}
```

`runes/grpc/grpc.test.fav` に追加テスト:
- `grpc_call_stream_bad_host_is_empty_list_or_err`

---

## Phase 7: テスト追加

### `vm_stdlib_tests.rs`

- `grpc_encode_grpc_frame_roundtrip` — encode → decode が元のバイト列に一致
- `grpc_call_stream_raw_returns_list_on_bad_host` — エラー時は空リストまたは RpcError

### `driver.rs` 統合テスト

- `grpc_serve_stream_raw_in_favnir_source` — serve_stream_raw の型チェックが通る
- `grpc_call_stream_raw_bad_host_in_favnir_source` — call_stream_raw の型チェックが通る
- `grpc_rune_serve_stream_in_favnir_source` — `grpc.serve_stream` ラッパーが通る
- `grpc_rune_call_stream_in_favnir_source` — `grpc.call_stream` ラッパーが通る

---

## Phase 8: examples + docs

- `fav/examples/grpc_stream_demo/src/main.fav` — サーバーストリーミングデモ
- `versions/v3.9.0/langspec.md` — v3.9.0 向け langspec
- `versions/v3.9.0/migration-guide.md` — v3.8.0 → v3.9.0 移行ガイド
- `versions/v3.9.0/progress.md` — 全フェーズ完了時に更新

---

## 実装上の注意点

### tonic との統合

tonic は tokio 上で動作するため、VM のメインスレッドから直接呼べない。
必ず専用スレッドで `tokio::runtime::Builder` を使って起動する。

```rust
std::thread::spawn(move || {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        // tonic client/server コード
    })
}).join().unwrap()
```

### VM スレッドとの同期（serve_raw）

`Grpc.serve_raw` は VM スレッドをブロックしながらリクエストを処理する必要がある。
tokio スレッドから VM ハンドラを呼び出す際は `std::sync::mpsc` を使用:

```rust
let (req_tx, req_rx) = mpsc::channel::<(String, Vec<VMValue>)>();
let (res_tx, res_rx) = mpsc::channel::<VMValue>();
// tokio スレッド: req_tx.send(...) → res_rx.recv()
// VM スレッド: req_rx.recv() → run_handler → res_tx.send(...)
```

### 既存テストへの影響

`tiny_http` / `ureq` の依存を削除する場合、既存の `grpc_call_raw_returns_err_on_bad_host`
等のテストが失敗しないよう注意。エラー型・エラーコードが変わらないことを確認する。
