# Favnir v4.0.0 Tasks

## Phase 0: バージョン更新

- [ ] `fav/Cargo.toml` の version を `"4.0.0"` に更新
- [ ] `fav/src/main.rs` のヘルプテキスト・バージョン文字列を `4.0.0` に更新

## Phase 1: `Grpc.serve_raw` / `Grpc.serve_stream_raw` — ハンドラ dispatch 実装

- [ ] `vm.rs`: `GrpcRequestMsg` 型を定義
  - `(String, Vec<u8>, std::sync::mpsc::SyncSender<Vec<u8>>)`
  - = (handler_name, proto_bytes, 応答用チャネル送信端)
- [ ] `vm.rs`: `grpc_serve_impl(port, service_name, streaming, req_tx)` 関数を実装
  - [ ] `std::thread::spawn` + `tokio::runtime::Builder::new_multi_thread()`
  - [ ] HTTP/2 サーバーをポートにバインド（tonic または hyper 直接）
  - [ ] リクエスト受信 → `decode_grpc_frame` → proto bytes
  - [ ] URL パスから method 名を抽出 → `pascal_to_snake` → `handle_<method>`
  - [ ] `(res_tx, res_rx)` を生成 → `req_tx.send((handler_name, proto_bytes, res_tx))`
  - [ ] `spawn_blocking(|| res_rx.recv())` でレスポンス待機
  - [ ] レスポンス bytes → `encode_grpc_frame` → HTTP/2 レスポンス返送
- [ ] `vm.rs`: `grpc_spawn_placeholder_server` を削除（`grpc_serve_impl` に置き換え）
- [ ] `vm.rs`: `grpc_vm_value_to_proto_bytes(value, type_metas)` ヘルパー実装
  - [ ] `Result::Ok(Map)` → `string_map_to_proto_bytes`
  - [ ] `Result::Err(RpcError)` → gRPC error status
- [ ] `vm.rs`: `Grpc.serve_raw` アームを VM メインスレッドの無限ループに変更
  - [ ] `req_rx.recv()` で待機
  - [ ] `artifact.fn_idx_by_name(handler_name)` でハンドラを解決
  - [ ] `proto_bytes_to_string_map(proto_bytes)` → `VMValue::Record`
  - [ ] `self.invoke_function(artifact, fn_idx, [req_record])`
  - [ ] 結果 → `grpc_vm_value_to_proto_bytes` → `res_tx.send(resp_bytes)`
  - [ ] ループ継続（serve_raw は返らない）
- [ ] `vm.rs`: `Grpc.serve_stream_raw` アームを実装
  - [ ] `Grpc.serve_raw` と同じ構造
  - [ ] ハンドラが `VMValue::List(items)` を返した場合、各要素を個別フレームとして連結
  - [ ] 連結した bytes を HTTP/2 レスポンスボディとして返す

## Phase 2: `Grpc.call_raw` — 実 RPC 送受信実装

- [ ] `vm.rs`: `Grpc.call_raw` の stub（code 12）を取り除き、実装を差し替え
  - [ ] `string_map_to_proto_bytes(payload)` → proto bytes
  - [ ] `encode_grpc_frame(proto_bytes)` → リクエストフレーム
  - [ ] HTTP/2 POST `<endpoint>/<method_path>` を送信
    - [ ] `Content-Type: application/grpc` ヘッダー
    - [ ] body: リクエストフレーム
  - [ ] HTTP/2 レスポンスボディを全バイト読み取り
  - [ ] `decode_grpc_frame(resp_bytes)` → proto bytes
  - [ ] `proto_bytes_to_string_map(proto_bytes)` → `VMValue::Record`
  - [ ] `ok_vm(VMValue::Record(map))` を返す
- [ ] エラー処理
  - [ ] 接続失敗 → `err_vm(rpc_error_vm(14, ...))` — 既存テスト `grpc_call_raw_returns_err_on_bad_host` と互換
  - [ ] gRPC status エラー → `err_vm(rpc_error_vm(status_code, message))`

## Phase 3: `Grpc.call_stream_raw` — バグ修正

- [ ] `vm.rs`: `decode_all_grpc_frames(&frame)` → `decode_all_grpc_frames(&resp_bytes)` に修正
  - [ ] Phase 2 の HTTP/2 レスポンス受信コードを流用
  - [ ] レスポンスボディを全バイト読み取る
  - [ ] `decode_all_grpc_frames(resp_bytes)` → フレームリスト
  - [ ] 各フレーム → `proto_bytes_to_string_map` → `VMValue::Record`
  - [ ] `VMValue::List(records)` を返す
- [ ] 接続失敗時の動作を維持（空リスト `VMValue::List(vec![])` を返す）

## Phase 4: `pipe match`（`|> match { ... }`）

- [ ] `ast.rs`: `Expr::PipeMatch { lhs: Box<Expr>, arms: Vec<MatchArm>, span: Span }` を追加
- [ ] `parser.rs`: `parse_pipe_expr` 内で `|>` 後に `match` が来た場合を分岐
  - [ ] `match` キーワードを消費 → `parse_match_arms()` で arms を解析
  - [ ] `Expr::PipeMatch { ... }` を生成
  - [ ] それ以外の場合は従来の `Expr::Pipe` と同じ処理
- [ ] `checker.rs`: `PipeMatch` の型チェックを追加
  - [ ] `lhs` の型を推論
  - [ ] 仮の束縛として arms を検査（既存 `check_match_arms` を再利用）
  - [ ] 全アームの戻り型が一致することを確認
- [ ] `compiler.rs`: `PipeMatch` のコンパイルを追加
  - [ ] `lhs` を評価してスタックに積む
  - [ ] 既存 `compile_match` (またはインライン match コード生成) に委譲
- [ ] `fmt.rs`: `PipeMatch` のフォーマット対応（あれば）

## Phase 5: `pattern guard`（`where` 句）

- [ ] `ast.rs`: `MatchArm` に `guard: Option<Expr>` フィールドを追加
  - [ ] 既存の `MatchArm` 構造体を更新
  - [ ] ガードなしは `guard: None`
- [ ] `parser.rs`: `parse_match_arm` で `=>` 前の `where <expr>` を解析
  - [ ] `where` キーワードを検出したらガード式を解析
  - [ ] `where` がない場合は `guard: None`
- [ ] `checker.rs`: `check_match_arm` でガード式の型を `Bool` と検査
  - [ ] ガード式のスコープでパターンバインド変数を使えるようにする
  - [ ] `Bool` 以外は型エラー（エラーコード `E04xx` 相当）
- [ ] `compiler.rs`: ガードありアームのコード生成
  - [ ] パターンマッチ失敗 → 次アームへ jump（既存）
  - [ ] パターンマッチ成功 → ガード式を評価
  - [ ] `JumpIfFalse` で次アームへ jump（ガード false の場合）
  - [ ] ガード true → アーム本体を実行
  - [ ] backpatching で jump 先アドレスを確定

## Phase 6: スタックトレース

- [ ] `ast.rs` または新規ファイル: `DebugInfo { fn_name: String, file: String, line: u32 }` を定義
- [ ] `compiler.rs` / `codegen.rs`: `Artifact` に `debug_info: Vec<Option<DebugInfo>>` を追加
  - [ ] 各 `Call` opcode 生成時に対応する `DebugInfo` を記録
  - [ ] ファイル名・行番号を span から取得（`artifact.source_file` 等）
- [ ] `vm.rs`: `CallFrame { fn_name: String, file: String, line: u32 }` を定義
- [ ] `vm.rs`: `VM` 構造体に `call_stack: Vec<CallFrame>` を追加
- [ ] `vm.rs`: `invoke_function` 入口で `call_stack.push(CallFrame { ... })` を追加
- [ ] `vm.rs`: `invoke_function` 出口（正常・エラー両方）で `call_stack.pop()` を追加
- [ ] `vm.rs`: `format_stack_trace(frames: &[CallFrame]) -> String` を実装
  - [ ] フレームを逆順（最新が先）に表示
  - [ ] 最大 50 フレーム（超過分は `... N more frames` で省略）
- [ ] `vm.rs` / `main.rs`: ランタイムエラー時に `format_stack_trace` をエラーメッセージに付加
- [ ] `driver.rs`: `fav test` 実行時にテスト失敗でスタックトレースを表示（IO キャプチャと共存）

## Phase 7: テスト追加

### `vm_stdlib_tests.rs`

- [ ] `grpc_encode_decode_grpc_frame_roundtrip` — v3.9.0 で未追加の framing テスト追加
- [ ] `grpc_call_raw_bad_host_returns_err` — code 14 を確認（Phase 2 実装後）

### `driver.rs` 統合テスト

- [ ] `grpc_serve_raw_dispatches_handler` — ハンドラ関数が実際に呼ばれることを確認
- [ ] `pipe_match_basic` — `result |> match { Ok(v) => v Err(_) => default }` が動く
- [ ] `pipe_match_type_error_on_arm_mismatch` — アームの型不一致で型エラー
- [ ] `pattern_guard_basic` — `where score >= 90` が成立するケース
- [ ] `pattern_guard_fallthrough` — ガード false で次アームへ
- [ ] `stack_trace_displayed_on_runtime_error` — エラー時に `at <fn> (<file>:<line>)` が出力される

### `runes/grpc/grpc.test.fav`

- [ ] `grpc_call_stream_bad_host_returns_empty` は既存（維持）
- [ ] `grpc_serve_and_call_roundtrip` — 別スレッドでサーバーを起動してクライアントから呼び出す
  （Favnir レベル E2E テスト; タイムアウト注意）

## Phase 8: examples + docs

- [ ] `fav/examples/grpc_e2e_demo/src/main.fav` 作成
- [ ] `versions/v4.0.0/langspec.md` 作成
- [ ] `versions/v4.0.0/migration-guide.md` 作成
- [ ] `versions/v4.0.0/progress.md` 全フェーズ完了時に更新
- [ ] `memory/MEMORY.md` を v4.0.0 完了状態に更新
