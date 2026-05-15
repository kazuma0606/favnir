# Favnir v3.9.0 Tasks

## Phase 0: バージョン更新 + 依存追加

- [ ] `fav/Cargo.toml` の version を `"3.9.0"` に更新
- [ ] `fav/Cargo.toml` に `tonic = { version = "0.11", features = ["transport"] }` 追加
- [ ] `fav/Cargo.toml` に `tokio = { version = "1", features = ["full"] }` 追加
- [ ] `fav/Cargo.toml` に `prost = "0.12"` 追加
- [ ] `fav/src/main.rs` のヘルプテキスト・バージョン文字列を `3.9.0` に更新

## Phase 1: gRPC framing ヘルパー実装

- [ ] `vm.rs`: `encode_grpc_frame(payload: &[u8]) -> Vec<u8>` 実装
  - [ ] byte[0] = 0x00（非圧縮）
  - [ ] byte[1..4] = payload.len() as u32 big-endian
  - [ ] byte[5..] = payload
- [ ] `vm.rs`: `decode_grpc_frame(data: &[u8]) -> Result<Vec<u8>, String>` 実装
  - [ ] 先頭 5 バイトが存在するか確認
  - [ ] byte[1..4] から長さを読み取り
  - [ ] payload スライスを返す

## Phase 2: `Grpc.call_raw` — HTTP/2 へ移行

- [ ] `vm.rs`: `Grpc.call_raw` を tonic HTTP/2 + gRPC framing で再実装
  - [ ] `std::thread::spawn` + `tokio::runtime::Builder::new_current_thread()`
  - [ ] `map_to_proto_bytes` → `encode_grpc_frame` でリクエスト組み立て
  - [ ] `tonic::transport::Channel::from_shared(host)?.connect().await`
  - [ ] HTTP/2 リクエスト送信
  - [ ] レスポンス受信 → `decode_grpc_frame` → `proto_bytes_to_map`
  - [ ] ネットワークエラー → `RpcError { code: 14, message: ... }`
  - [ ] 既存テスト `grpc_call_raw_returns_err_on_bad_host` が引き続きパスすること

## Phase 3: `Grpc.serve_raw` — HTTP/2 へ移行

- [ ] `vm.rs`: `Grpc.serve_raw` を tonic HTTP/2 gRPC サーバーで再実装
  - [ ] `std::thread::spawn` + `tokio::runtime::Builder::new_multi_thread()`
  - [ ] `tonic::transport::Server` でポートを LISTEN
  - [ ] リクエスト受信 → `decode_grpc_frame` → `proto_bytes_to_map`
  - [ ] `pascal_to_snake(method_name)` でハンドラ名を解決（既存関数再利用）
  - [ ] `std::sync::mpsc` チャネルで VM スレッドとのハンドラ呼び出しを同期
  - [ ] ハンドラ結果 → `map_to_proto_bytes` → `encode_grpc_frame` → HTTP/2 レスポンス
  - [ ] 起動時に `"Listening on 0.0.0.0:{port} (gRPC / HTTP2)"` を出力

## Phase 4: `Grpc.serve_stream_raw` + `Grpc.call_stream_raw` 追加

- [ ] `vm.rs`: `Grpc.serve_stream_raw(port, service_name)` 実装
  - [ ] `Grpc.serve_raw` ベースにストリーミング対応を追加
  - [ ] ハンドラが `VMValue::List(items)` を返す場合、各要素を個別フレームとして送信
  - [ ] `Result::Ok(VMValue::List(...))` も同様に処理
- [ ] `vm.rs`: `Grpc.call_stream_raw(host, method, payload)` 実装
  - [ ] `Grpc.call_raw` ベースに複数フレーム受信対応を追加
  - [ ] レスポンスボディから `decode_grpc_frame` を繰り返し呼び出し
  - [ ] 各フレーム → `proto_bytes_to_map` → `VMValue::Record`
  - [ ] 全 Record を `VMValue::List` にして返す

## Phase 5: checker.rs + compiler.rs シグネチャ追加

- [ ] `checker.rs`: `Grpc.serve_stream_raw(port: Int, service_name: String) -> Unit !Rpc !Io` 登録
- [ ] `checker.rs`: `Grpc.call_stream_raw(host: String, method: String, payload: Map<String,String>) -> List<Map<String,String>> !Rpc` 登録
- [ ] `compiler.rs`: 新メソッド名が既存の `"Grpc"` 登録ループでカバーされていることを確認

## Phase 6: `runes/grpc/grpc.fav` 拡張

- [ ] `runes/grpc/grpc.fav`: `serve_stream(port, service_name)` 追加（`Grpc.serve_stream_raw` ラッパー）
- [ ] `runes/grpc/grpc.fav`: `call_stream(host, method, payload)` 追加（`Grpc.call_stream_raw` ラッパー）
- [ ] `runes/grpc/grpc.test.fav`: `grpc_call_stream_bad_host_returns_err_or_empty` テスト追加

## Phase 7: テスト追加

### `vm_stdlib_tests.rs`

- [ ] `grpc_encode_grpc_frame_roundtrip` — encode_grpc_frame → decode_grpc_frame が元のバイト列に一致
- [ ] `grpc_call_stream_raw_returns_list_on_bad_host` — エラー時は空リスト or RpcError

### `driver.rs` 統合テスト

- [ ] `grpc_serve_stream_raw_type_checks_in_favnir_source` — serve_stream_raw の型チェックが通る
- [ ] `grpc_call_stream_raw_bad_host_in_favnir_source` — call_stream_raw の型チェック + エラー動作
- [ ] `grpc_rune_serve_stream_in_favnir_source` — `grpc.serve_stream` ラッパーが型チェック通る
- [ ] `grpc_rune_call_stream_in_favnir_source` — `grpc.call_stream` ラッパーが型チェック通る

## Phase 8: examples + docs

- [ ] `fav/examples/grpc_stream_demo/src/main.fav` 作成（サーバーストリーミングデモ）
- [ ] `versions/v3.9.0/langspec.md` 作成
- [ ] `versions/v3.9.0/migration-guide.md` 作成
- [ ] `versions/v3.9.0/progress.md` 全フェーズ完了に更新
- [ ] `memory/MEMORY.md` を v3.9.0 完了状態に更新
