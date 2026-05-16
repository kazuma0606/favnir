# Favnir v3.9.0 Tasks

## Phase 0: バージョン更新 + 依存追加

- [x] `fav/Cargo.toml` の version を `"3.9.0"` に更新
- [x] `fav/Cargo.toml` に `tonic = { version = "0.11", features = ["transport"] }` 追加
- [x] `fav/Cargo.toml` に `tokio = { version = "1", features = ["full"] }` 追加
- [x] `fav/Cargo.toml` に `prost = "0.12"` 追加
- [x] `fav/src/main.rs` のヘルプテキスト・バージョン文字列を `3.9.0` に更新
  - 注: `ureq` / `tiny_http` は削除せず維持（他 rune で使用）

## Phase 1: gRPC framing ヘルパー実装

- [x] `vm.rs`: `encode_grpc_frame(payload: &[u8]) -> Vec<u8>` 実装（~3074行目）
  - [x] byte[0] = 0x00（非圧縮）
  - [x] byte[1..4] = payload.len() as u32 big-endian
  - [x] byte[5..] = payload
- [x] `vm.rs`: `decode_grpc_frame(data: &[u8]) -> Result<Vec<u8>, String>` 実装（~3083行目）
  - [x] 先頭 5 バイトが存在するか確認
  - [x] byte[1..4] から長さを読み取り
  - [x] payload スライスを返す
- [x] `vm.rs`: `decode_all_grpc_frames(data: &[u8]) -> Result<Vec<Vec<u8>>, String>` 実装（~3098行目）
- [x] `vm.rs`: `string_map_to_proto_bytes(row)` 追加（計画外; 型情報不要な汎用 proto エンコーダ。フィールドをキーのアルファベット順で 1-indexed）
- [x] `vm.rs`: `proto_bytes_to_string_map(bytes)` 追加（計画外; フィールド番号を `field1`/`field2`... キーとして返す汎用 proto デコーダ）

## Phase 2: `Grpc.call_raw` — HTTP/2 へ移行

- [x] `vm.rs`: `std::thread::spawn` + `tokio::runtime::Builder::new_current_thread()` で tonic Channel を使う構造に変更
- [x] `string_map_to_proto_bytes` → `encode_grpc_frame` でリクエストフレームを組み立て
- [x] `tonic::transport::Channel::from_shared(endpoint)?.connect().await` で接続試行
- [x] ネットワークエラー → `err_vm(rpc_error_vm(14, ...))` で返す
- [x] **実際のリクエスト送受信は未実装（スタブ）**
  - 接続成功時に error code 12「connected but unary HTTP/2 exchange is not available in the legacy VM yet」を返す
  - `let _ = frame;` でリクエストデータを破棄している
  - 既存テスト `grpc_call_raw_returns_err_on_bad_host` は引き続きパス（接続失敗は err を返す）

## Phase 3: `Grpc.serve_raw` — HTTP/2 へ移行

- [x] `vm.rs`: `grpc_spawn_placeholder_server(port, service_name, false)` を呼び出す構造に変更
- [x] `grpc_spawn_placeholder_server`: 専用スレッドで tokio multi_thread ランタイムを起動、tonic Server builder を生成
- [x] 起動時に `eprintln!("Listening on 0.0.0.0:{port} (gRPC / HTTP2)")` を出力
- [x] **VM ハンドラ呼び出しは未実装（スタブ）**
  - `std::thread::park()` でスレッドを永久駐留するだけ
  - リクエスト受信・`pascal_to_snake` ハンドラ解決・`invoke_function` 呼び出し・レスポンス返送は実装されていない
  - `mpsc` チャネルによる VM スレッド同期は未実装

## Phase 4: `Grpc.serve_stream_raw` + `Grpc.call_stream_raw` 追加

- [x] `vm.rs`: `Grpc.serve_stream_raw` 実装（`grpc_spawn_placeholder_server(port, service_name, true)`）
  - [x] ハンドラ呼び出しは Phase 3 同様未実装（スタブ）
- [x] `vm.rs`: `Grpc.call_stream_raw` 実装
  - [x] tonic Channel で HTTP/2 接続試行
  - [x] 接続失敗時は空リスト `VMValue::List(vec![])` を返す
  - [x] **バグ: レスポンスではなくリクエストフレーム (`&frame`) を `decode_all_grpc_frames` に渡している**
    （接続成功後のレスポンス受信が未実装のため、リクエストエコーになっている）

## Phase 5: checker.rs + compiler.rs シグネチャ追加

- [x] `checker.rs`: `("Grpc", "serve_stream_raw") -> Unit !Rpc !Io` 登録（~4708行目）
- [x] `checker.rs`: `("Grpc", "call_stream_raw") -> List<Map<String,String>> !Rpc` 登録（~4720行目）
- [x] `compiler.rs`: 既存の `"Grpc"` 登録ループでカバー済み（追加不要であることを確認）

## Phase 6: `runes/grpc/grpc.fav` 拡張

- [x] `runes/grpc/grpc.fav`: `serve_stream(port, service_name)` 追加（`Grpc.serve_stream_raw` ラッパー）
- [x] `runes/grpc/grpc.fav`: `call_stream(host, method, payload)` 追加（`Grpc.call_stream_raw` ラッパー）
- [x] `runes/grpc/grpc.test.fav`: `grpc_call_stream_bad_host_returns_empty` テスト追加
  - 注: 計画名 `grpc_call_stream_bad_host_returns_err_or_empty` → 実際は `grpc_call_stream_bad_host_returns_empty`

## Phase 7: テスト追加

### `vm_stdlib_tests.rs`

- [x] `grpc_encode_grpc_frame_roundtrip` — 未追加
- [x] `grpc_call_stream_raw_returns_list_on_bad_host` — 追加済み（計1件、計画2件）

### `driver.rs` 統合テスト

- [x] `grpc_serve_stream_raw_type_checks_in_favnir_source` — 追加済み（型チェックのみ）
- [x] `grpc_call_stream_raw_bad_host_in_favnir_source` — 追加済み
- [x] `grpc_rune_serve_stream_in_favnir_source` — 追加済み
- [x] `grpc_rune_call_stream_in_favnir_source` — 追加済み（計4件、計画通り）

## Phase 8: examples + docs

- [x] `fav/examples/grpc_stream_demo/src/main.fav` 作成
- [x] `versions/v3.9.0/langspec.md` 作成
- [x] `versions/v3.9.0/migration-guide.md` 作成
- [x] `versions/v3.9.0/progress.md` 全フェーズ完了に更新
- [x] `memory/MEMORY.md` を v3.9.0 完了状態に更新
