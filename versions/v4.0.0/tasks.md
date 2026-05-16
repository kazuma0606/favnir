# Favnir v4.0.0 Tasks

## Phase 0: バージョン更新

- [x] `fav/Cargo.toml` の version を `"4.0.0"` に更新
- [x] `fav/src/main.rs` のヘルプテキスト・バージョン文字列を `4.0.0` に更新

## Phase 1: `Grpc.serve_raw` / `Grpc.serve_stream_raw` — ハンドラ dispatch 実装

- [x] `vm.rs`: `GrpcRequestMsg` 型を定義
  - `(String, Vec<u8>, std::sync::mpsc::SyncSender<Vec<u8>>)`
  - = (handler_name, proto_bytes, 応答用チャネル送信端)
- [x] `vm.rs`: `grpc_serve_impl(port, service_name, streaming, req_tx)` 関数を実装
  - [x] `std::thread::spawn` + `tokio::runtime::Builder::new_multi_thread()`
  - [x] HTTP/2 サーバーをポートにバインド（tonic または hyper 直接）
  - [x] リクエスト受信 → `decode_grpc_frame` → proto bytes
  - [x] URL パスから method 名を抽出 → `pascal_to_snake` → `handle_<method>`
  - [x] `(res_tx, res_rx)` を生成 → `req_tx.send((handler_name, proto_bytes, res_tx))`
  - [x] `spawn_blocking(|| res_rx.recv())` でレスポンス待機
  - [x] レスポンス bytes → `encode_grpc_frame` → HTTP/2 レスポンス返送
- [x] `vm.rs`: `grpc_spawn_placeholder_server` を削除（`grpc_serve_impl` に置き換え）
- [x] `vm.rs`: `grpc_vm_value_to_proto_bytes(value, type_metas)` ヘルパー実装
  - [x] `Result::Ok(Map)` → `string_map_to_proto_bytes`
  - [x] `Result::Err(RpcError)` → gRPC error status
- [x] `vm.rs`: `Grpc.serve_raw` アームを VM メインスレッドの無限ループに変更
  - [x] `req_rx.recv()` で待機
  - [x] `artifact.fn_idx_by_name(handler_name)` でハンドラを解決
  - [x] `proto_bytes_to_string_map(proto_bytes)` → `VMValue::Record`
  - [x] `self.invoke_function(artifact, fn_idx, [req_record])`
  - [x] 結果 → `grpc_vm_value_to_proto_bytes` → `res_tx.send(resp_bytes)`
  - [x] ループ継続（serve_raw は返らない）
- [x] `vm.rs`: `Grpc.serve_stream_raw` アームを実装
  - [x] `Grpc.serve_raw` と同じ構造
  - [x] ハンドラが `VMValue::List(items)` を返した場合、各要素を個別フレームとして連結
  - [x] 連結した bytes を HTTP/2 レスポンスボディとして返す

## Phase 2: `Grpc.call_raw` — 実 RPC 送受信実装

- [x] `vm.rs`: `Grpc.call_raw` の stub（code 12）を取り除き、実装を差し替え
  - [x] `string_map_to_proto_bytes(payload)` → proto bytes
  - [x] `encode_grpc_frame(proto_bytes)` → リクエストフレーム
  - [x] HTTP/2 POST `<endpoint>/<method_path>` を送信
    - [x] `Content-Type: application/grpc` ヘッダー
    - [x] body: リクエストフレーム
  - [x] HTTP/2 レスポンスボディを全バイト読み取り
  - [x] `decode_grpc_frame(resp_bytes)` → proto bytes
  - [x] `proto_bytes_to_string_map(proto_bytes)` → `VMValue::Record`
  - [x] `ok_vm(VMValue::Record(map))` を返す
- [x] エラー処理
  - [x] 接続失敗 → `err_vm(rpc_error_vm(14, ...))` — 既存テスト `grpc_call_raw_returns_err_on_bad_host` と互換
  - [x] gRPC status エラー → `err_vm(rpc_error_vm(status_code, message))`

## Phase 3: `Grpc.call_stream_raw` — バグ修正

- [x] `vm.rs`: `decode_all_grpc_frames(&frame)` → `decode_all_grpc_frames(&resp_bytes)` に修正
  - [x] Phase 2 の HTTP/2 レスポンス受信コードを流用
  - [x] レスポンスボディを全バイト読み取る
  - [x] `decode_all_grpc_frames(resp_bytes)` → フレームリスト
  - [x] 各フレーム → `proto_bytes_to_string_map` → `VMValue::Record`
  - [x] `VMValue::List(records)` を返す
- [x] 接続失敗時の動作を維持（空リスト `VMValue::List(vec![])` を返す）

## Phase 4: `pipe match`（`|> match { ... }`）

- [x] `ast.rs`: `Expr::PipeMatch { lhs: Box<Expr>, arms: Vec<MatchArm>, span: Span }` を追加
- [x] `parser.rs`: `parse_pipe_expr` 内で `|>` 後に `match` が来た場合を分岐
  - [x] `match` キーワードを消費 → `parse_match_arms()` で arms を解析
  - [x] `Expr::PipeMatch { ... }` を生成
  - [x] それ以外の場合は従来の `Expr::Pipe` と同じ処理
- [x] `checker.rs`: `PipeMatch` の型チェックを追加
  - [x] `lhs` の型を推論
  - [x] 仮の束縛として arms を検査（既存 `check_match_arms` を再利用）
  - [x] 全アームの戻り型が一致することを確認
- [x] `compiler.rs`: `PipeMatch` のコンパイルを追加
  - [x] `lhs` を評価してスタックに積む
  - [x] 既存 `compile_match` (またはインライン match コード生成) に委譲
- [x] `fmt.rs`: `PipeMatch` のフォーマット対応（あれば）

## Phase 5: `pattern guard`（`where` 句）

- [x] `ast.rs`: `MatchArm` に `guard: Option<Expr>` フィールドを追加
  - [x] 既存の `MatchArm` 構造体を更新
  - [x] ガードなしは `guard: None`
- [x] `parser.rs`: `parse_match_arm` で `=>` 前の `where <expr>` を解析
  - [x] `where` キーワードを検出したらガード式を解析
  - [x] `where` がない場合は `guard: None`
- [x] `checker.rs`: `check_match_arm` でガード式の型を `Bool` と検査
  - [x] ガード式のスコープでパターンバインド変数を使えるようにする
  - [x] `Bool` 以外は型エラー（エラーコード `E04xx` 相当）
- [x] `compiler.rs`: ガードありアームのコード生成
  - [x] パターンマッチ失敗 → 次アームへ jump（既存）
  - [x] パターンマッチ成功 → ガード式を評価
  - [x] `JumpIfFalse` で次アームへ jump（ガード false の場合）
  - [x] ガード true → アーム本体を実行
  - [x] backpatching で jump 先アドレスを確定

## Phase 6: スタックトレース

- [x] `ast.rs` または新規ファイル: `DebugInfo { fn_name: String, file: String, line: u32 }` を定義
- [x] `compiler.rs` / `codegen.rs`: `Artifact` に `debug_info: Vec<Option<DebugInfo>>` を追加
  - [x] 各 `Call` opcode 生成時に対応する `DebugInfo` を記録
  - [x] ファイル名・行番号を span から取得（`artifact.source_file` 等）
- [x] `vm.rs`: `CallFrame { fn_name: String, file: String, line: u32 }` を定義
- [x] `vm.rs`: `VM` 構造体に `call_stack: Vec<CallFrame>` を追加
- [x] `vm.rs`: `invoke_function` 入口で `call_stack.push(CallFrame { ... })` を追加
- [x] `vm.rs`: `invoke_function` 出口（正常・エラー両方）で `call_stack.pop()` を追加
- [x] `vm.rs`: `format_stack_trace(frames: &[CallFrame]) -> String` を実装
  - [x] フレームを逆順（最新が先）に表示
  - [x] 最大 50 フレーム（超過分は `... N more frames` で省略）
- [x] `vm.rs` / `main.rs`: ランタイムエラー時に `format_stack_trace` をエラーメッセージに付加
- [x] `driver.rs`: `fav test` 実行時にテスト失敗でスタックトレースを表示（IO キャプチャと共存）

## Phase 7: テスト追加

### `vm_stdlib_tests.rs`

- [x] `grpc_encode_decode_grpc_frame_roundtrip` — v3.9.0 で未追加の framing テスト追加
- [x] `grpc_call_raw_bad_host_returns_err` — code 14 を確認（Phase 2 実装後）

### `driver.rs` 統合テスト

- [x] `grpc_serve_raw_dispatches_handler` — ハンドラ関数が実際に呼ばれることを確認
- [x] `pipe_match_basic` — `result |> match { Ok(v) => v Err(_) => default }` が動く
- [x] `pipe_match_type_error_on_arm_mismatch` — アームの型不一致で型エラー
- [x] `pattern_guard_basic` — `where score >= 90` が成立するケース
- [x] `pattern_guard_fallthrough` — ガード false で次アームへ
- [x] `stack_trace_displayed_on_runtime_error` — エラー時に `at <fn> (<file>:<line>)` が出力される

### `runes/grpc/grpc.test.fav`

- [x] `grpc_call_stream_bad_host_returns_empty` は既存（維持）
- [x] `grpc_serve_and_call_roundtrip` — 別スレッドでサーバーを起動してクライアントから呼び出す
  （Favnir レベル E2E テスト; タイムアウト注意）

## Phase 8: examples + docs

- [x] `fav/examples/grpc_e2e_demo/src/main.fav` 作成
- [x] `versions/v4.0.0/langspec.md` 作成
- [x] `versions/v4.0.0/migration-guide.md` 作成
- [x] `versions/v4.0.0/progress.md` 全フェーズ完了時に更新
- [x] `memory/MEMORY.md` を v4.0.0 完了状態に更新
