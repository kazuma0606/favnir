# v21.1.0 — DAP デバッガー タスク

## ステータス: DONE

---

## タスク一覧

### T1: `fav/src/dap/` — DAP モジュール新規作成

- [x] **事前確認（依存クレート）**: `grep -n "serde\|serde_json\|tokio\|tiny_http" fav/Cargo.toml` で `serde`・`serde_json` が既存依存であることを確認（新規クレート追加不要）
- [x] **事前確認**: `grep -n "mod dap" fav/src/lib.rs fav/src/main.rs` で dap モジュールが未存在なことを確認
- [x] `fav/src/dap/protocol.rs` を新規作成（plan.md T1-1 の型定義に従う）
  - [x] `DapMessage` / `DapRequest` / `DapResponse` / `DapEvent` struct（`serde::{Serialize, Deserialize}` derive）
  - [x] `DapBreakpoint` / `InitializeArguments` / `SetBreakpointsArguments` / `LaunchArguments` struct
- [x] `fav/src/dap/session.rs` を新規作成（plan.md T1-2 に従う）
  - [x] `DapSession::new()` / `next_seq()` / `set_breakpoints()` / `is_breakpoint()` / `stop_at(source, line, reason, stage, locals)` / `resume()`
  - [x] `DapSession` に `current_stage: Option<String>` / `event_queue: Vec<serde_json::Value>` フィールド追加
  - [x] `stop_at` 内で `stopped` イベント JSON を `event_queue` に積む
- [x] `fav/src/dap/adapter.rs` を新規作成（plan.md T1-3 に従う）
  - [x] `DapHook` enum（`StageEnter / StageExit / Output`）
  - [x] `DapAdapter::new()` / `on_hook()`
- [x] `fav/src/dap/server.rs` を新規作成（plan.md T1-4 に従う）
  - [x] `send_dap_message()` / `recv_dap_message()` ヘルパー（`R: BufRead + std::io::Read`、`read_exact` で body 読み取り）
  - [x] `run_dap_server(port: u16, session: Arc<Mutex<DapSession>>) -> Result<(), String>` — TcpListener + DAP メッセージループ（VM 側と session 共有）
  - [x] `handle_dap_session()` — `recv_dap_message` に委譲してメッセージ受信、response 送信後に `event_queue` を drain して `stopped` 等のイベントをプッシュ
  - [x] `handle_dap_request()` — 12 コマンド対応
- [x] `fav/src/dap/mod.rs` を新規作成（plan.md T1-5 に従う）
- [x] `fav/src/lib.rs` に `#[cfg(not(target_arch = "wasm32"))] pub mod dap;` を追加
- [x] `fav/src/main.rs` に `#[cfg(not(target_arch = "wasm32"))] mod dap;` を追加
- [x] `cargo check` でコンパイルエラー 0

---

### T2: `fav/src/backend/vm.rs` — debug フック挿入

- [x] **事前確認**: `grep -n "pub struct VM\|pub(crate) struct VM" fav/src/backend/vm.rs | head -3` で VM 定義を確認
- [x] `VM` struct に `#[cfg(not(target_arch = "wasm32"))] pub debug_mode: bool` を追加
- [x] `VM` struct に `#[cfg(not(target_arch = "wasm32"))] pub dap_adapter: Option<crate::dap::DapAdapter>` を追加
- [x] `VM::new_with_db_path()` の初期化に `debug_mode: false` / `dap_adapter: None` を追加
- [x] **事前確認**: `grep -n "run_stage\|run_trf\|call_stage\|OpCode::Call" fav/src/backend/vm.rs | head -10` で stage 実行ポイントを確認
- [x] stage 実行前に DAP フックを挿入（`debug_mode` フラグガード付き）
- [x] `collect_locals_for_dap(&self) -> Vec<(String, String, String)>` ヘルパーを実装
- [x] `cargo check` でコンパイルエラー 0

---

### T3: `fav/src/driver.rs` — `cmd_dap` / `cmd_run_debug` 追加

- [x] `cmd_dap(port: u16) -> Result<(), String>` を追加（plan.md T3-1 に従う）
- [x] `cmd_run_debug(path: &str, dap_port: u16) -> Result<(), String>` を追加（plan.md T3-2 に従う）
- [x] `fav/src/main.rs` の CLI ディスパッチに `"dap"` コマンドを追加（`--port` オプション対応）
- [x] `fav/src/main.rs` の `"run"` コマンドに `--debug` フラグ対応を追加
- [x] `cargo check` でコンパイルエラー 0

---

### T4: `fav/Cargo.toml` バージョン更新

- [x] `version = "21.0.0"` → `"21.1.0"` に変更
- [x] `v210000_tests` の `version_is_21_0_0` に `#[ignore]` を追加
  - `fav/src/driver.rs` の `v210000_tests` モジュール内 `version_is_21_0_0` 関数に追加
  - `#[test]` の直下に `#[ignore]` を挿入（`#[cfg]` は既存のモジュールレベルで設定済み）
- [x] `cargo build` でコンパイルエラー 0

---

### T5: `CHANGELOG.md` + `site/content/docs/tools/dap.mdx`

- [x] `CHANGELOG.md` の先頭に v21.1.0 エントリを追加（plan.md T5 の内容に従う）
  - [x] `### Added` — `fav dap` / `fav run --debug` / dap モジュール / 12 DAP コマンド / WASM ゼロコスト
- [x] `site/content/docs/tools/dap.mdx` を新規作成
  - [x] VS Code `launch.json` 設定例
  - [x] `fav dap` / `fav run --debug` の使い方
  - [x] サポートする DAP 操作一覧（ブレークポイント / ステップ / 変数インスペクション）
  - [x] WASM 非対応の注意書き

---

### T6: `fav/src/driver.rs` — `v211000_tests` 追加

- [x] `v210000_tests::version_is_21_0_0` に `#[ignore]` が付いていること（T4 で実施済み）
- [x] `v211000_tests` モジュールを追加（plan.md T6 の内容に従う）
  - [x] `version_is_21_1_0` — Cargo.toml に `"21.1.0"` が含まれる
  - [x] `dap_protocol_initialize_request_parses` — `initialize` JSON のパース
  - [x] `dap_protocol_response_serializes` — `DapResponse` の JSON シリアライズ
  - [x] `dap_session_breakpoint_set_and_hit` — ブレークポイント設定と is_breakpoint
  - [x] `dap_adapter_stopped_event_format` — StageEnter フック → stopped 状態
- [x] 各テストに `#[cfg(not(target_arch = "wasm32"))]` ガードを付与（モジュールレベル）
- [x] `cargo test v211000` — 5/5 PASS を確認

---

## テスト（v211000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_21_1_0` | Cargo.toml に `"21.1.0"` が含まれる |
| `dap_protocol_initialize_request_parses` | `initialize` リクエスト JSON がパースできる |
| `dap_protocol_response_serializes` | `DapResponse` が JSON にシリアライズできる |
| `dap_session_breakpoint_set_and_hit` | `set_breakpoints` + `is_breakpoint` が正しく動作する |
| `dap_adapter_stopped_event_format` | `StageEnter` フック → `stopped=true` / `current_line` / `stop_reason` |

---

## 完了条件チェックリスト

- [x] `fav dap` コマンドが TCP ポート 5678 でリッスンを開始する
- [x] `fav run --debug` が DAP 接続待機モードで起動する
- [x] `initialize` / `launch` / `setBreakpoints` / `configurationDone` / `threads` / `stackTrace` /
       `scopes` / `variables` / `next` / `stepIn` / `continue` / `disconnect` の 12 DAP コマンドに応答できる
- [x] `stopped` イベントがブレークポイントヒット時に正しいソース行情報とともに送信される
- [x] `variables` リクエストで現在の binding 名・型・値が返される
- [x] `--debug` なし実行でオーバーヘッドがゼロ（`debug_mode=false` ブランチが除去）
- [x] `fav/src/dap/` モジュール（protocol / session / adapter / server / mod.rs）が存在する
- [x] `lib.rs` / `main.rs` に `#[cfg(not(target_arch = "wasm32"))] mod dap` が追加されている
- [x] `site/content/docs/tools/dap.mdx` が存在する
- [x] `cargo test v211000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし（exit 0）
- [x] `fav/Cargo.toml` version が `21.1.0`
- [x] `CHANGELOG.md` に v21.1.0 エントリが追加されている

---

## 優先度

```
T1（fav/src/dap/ モジュール）  ← 最初（T6 テストが依存）
T2（vm.rs debug フック）        ← T1 完了後
T3（driver.rs cmd_dap 等）      ← T2 完了後
T4（Cargo.toml バージョン）     ← T1 と並列可
T5（CHANGELOG + MDX）           ← T3 完了後
T6（driver.rs テスト）          ← T1 完了後（T4 完了後に ignore 追加）
```

---

## 実装リスク と 対策

| リスク | 対策 |
|---|---|
| `TcpListener` + `BufReader` の借用競合（読み書きで stream を複数回借用） | `stream.try_clone()` でリーダー用クローンを作成。writer は元の stream を使用 |
| `read_line` が `\r\n` を正しく処理しない | `BufRead::read_line` は `\n` まで読む（`\r\n` の `\r` は含む）。trim で除去 |
| DAP メッセージの Content-Length パース失敗 | ヘッダーが完全に届くまで `read_line` でループ。空行で終了を検出 |
| VM の `frames` / `locals` アクセス方法が不明 | T2 開始前に `grep -n "frames\|locals\|local_slots" fav/src/backend/vm.rs` で確認 |
| `serde_json::Value` の `Serialize` / `Deserialize` が `DapRequest::arguments` と衝突 | `Option<serde_json::Value>` は serde が自動対応。手動 impl 不要 |
| WASM ビルドで `mod dap` が引き込まれる | lib.rs / main.rs の `mod dap` に `#[cfg(not(target_arch = "wasm32"))]` を付与（T1 で実施） |
| `collect_locals_for_dap` が VM の内部 frame 構造を参照できない | T2 前に VM の frame / locals 構造を grep で確認し、実際のフィールド名に合わせて実装 |
