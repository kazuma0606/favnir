# Favnir v4.0.0 Progress

## Phase 0: バージョン更新
- [x] `fav/Cargo.toml` の version を `"4.0.0"` に更新
- [x] `fav/src/main.rs` のヘルプテキスト・バージョン文字列を `4.0.0` に更新

## Phase 1: `Grpc.serve_raw` / `Grpc.serve_stream_raw` — ハンドラ dispatch 実装
- [x] `GrpcRequestMsg` 型定義（`(String, Vec<u8>, SyncSender<Result<Vec<u8>, String>>)`）
- [x] `grpc_serve_impl(port, req_tx)` 実装（h2 HTTP/2 サーバー + tokio）
- [x] `grpc_handle_h2_request` async 関数（リクエスト受信 → VM dispatch）
- [x] `Grpc.serve_raw` アームを VM メインスレッドの無限ループに変更
- [x] `Grpc.serve_stream_raw` アームを実装（List → 各要素フレーム連結）
- [x] `grpc_vm_value_to_proto_bytes` ヘルパー実装
- [x] `grpc_spawn_placeholder_server` を削除（`grpc_serve_impl` に置き換え）

## Phase 2: `Grpc.call_raw` — 実 RPC 送受信実装
- [x] `Grpc.call_raw` スタブを削除し、h2 クライアントによる実装に差し替え
- [x] `grpc_tcp_addr / grpc_method_uri` ヘルパー追加
- [x] trailers から `grpc-status` チェック
- [x] 接続失敗 → `err_vm(rpc_error_vm(14, ...))` — 既存テスト互換

## Phase 3: `Grpc.call_stream_raw` — バグ修正
- [x] `decode_all_grpc_frames(&frame)` → `decode_all_grpc_frames(&resp_bytes)` に修正
- [x] h2 クライアントでレスポンスボディを全バイト読み取り
- [x] 接続失敗時は空リスト `VMValue::List(vec![])` を返す

## Phase 4: `pipe match`（`|> match { ... }`）
- [x] 実装済み（v3.x で既存）— parser.rs でパース時に `Expr::Match` にデシュガー

## Phase 5: `pattern guard`（`where` 句）
- [x] 実装済み（v3.x で既存）— `MatchArm.guard: Option<Box<Expr>>`、codegen 対応済み

## Phase 6: スタックトレース
- [x] 実装済み（v3.x で既存）— `TraceFrame`、`build_stack_trace`、`format_runtime_error` 対応済み

## Phase 7: テスト追加
- [x] `grpc_encode_grpc_frame_roundtrip` — vm_stdlib_tests.rs に追加済み（v3.9.0 で追加）
- [x] `grpc_call_raw_returns_err_on_bad_host` — code 14 確認
- [x] `grpc_serve_raw_dispatches_handler` — ハンドラが実際に呼ばれることを E2E 確認
- [x] serve_raw/serve_stream 型チェックテストを type-check-only に更新

## Phase 8: examples + docs
- [x] `fav/examples/grpc_e2e_demo/src/main.fav` 作成
- [x] `versions/v4.0.0/langspec.md` 作成
- [x] `versions/v4.0.0/migration-guide.md` 作成
- [x] `memory/MEMORY.md` を v4.0.0 完了状態に更新
