# v21.1.0 Spec — DAP デバッガー（Debug Adapter Protocol）

## 概要

VS Code / Neovim / Emacs から Favnir パイプラインをステップ実行できる
**DAP（Debug Adapter Protocol）サーバー** を実装する。

**テーマ**: Developer Tooling Complete シリーズ第1弾 — 「デバッグできない」摩擦の解消

---

## 動機

v21.0.0 で VM は限界まで速くなった。しかしパイプラインが期待通りに動かないとき、
現状の手段は `IO.println` デバッグだけである。

- どの stage でデータが壊れているか？
- binding の型と値は正しいか？
- 条件分岐のどちらに入っているか？

これらを VS Code のブレークポイント + 変数ウィンドウで確認できることが、
開発体験の次の大きな改善点である。

---

## 成果物一覧

| 成果物 | 役割 |
|---|---|
| `fav/src/dap/mod.rs` | DAP サーバーのエントリポイント |
| `fav/src/dap/protocol.rs` | DAP JSON-RPC プロトコル型定義（Request / Response / Event） |
| `fav/src/dap/server.rs` | TCP ソケット + DAP メッセージループ |
| `fav/src/dap/session.rs` | デバッグセッション管理（ブレークポイント / スレッド / スタック） |
| `fav/src/dap/adapter.rs` | VM ↔ DAP ブリッジ（VM 実行状態を DAP イベントに変換） |
| `fav/src/backend/vm.rs` | `--debug` フラグ対応（VM への DAP フック挿入） |
| `fav/src/driver.rs` | `cmd_dap()` / `cmd_run_debug()` 追加、`v211000_tests` |
| `site/content/docs/tools/dap.mdx` | DAP デバッガーの使い方（VS Code 設定例含む） |

---

## DAP 機能仕様

### サポートする DAP リクエスト（Phase 1）

| DAP リクエスト | 機能 | 優先度 |
|---|---|---|
| `initialize` | DAP ハンドシェイク、capabilities 返却 | 必須 |
| `launch` | `fav run --debug <file>` を起動 | 必須 |
| `setBreakpoints` | ソースファイル + 行番号でブレークポイント設定 | 必須 |
| `configurationDone` | 設定完了通知（実行開始） | 必須 |
| `threads` | スレッド一覧（Favnir は単一スレッド） | 必須 |
| `stackTrace` | 現在のコールスタック（stage 名 + 行番号） | 必須 |
| `scopes` | 変数スコープ（local bindings） | 必須 |
| `variables` | binding 名 / 型 / 値の一覧 | 必須 |
| `next` | ステップオーバー（次の stage へ） | 必須 |
| `stepIn` | ステップイン（stage 内の次の式へ） | 必須 |
| `continue` | 次のブレークポイントまで実行 | 必須 |
| `disconnect` | セッション終了 | 必須 |

### Phase 1 スコープ外（将来バージョン）

以下は Phase 1 の対象外。将来バージョンで検討:
- **条件付きブレークポイント**（ロードマップには v21.1 として記載されているが Phase 1 では見送り。`condition` フィールドは受信するが評価しない。v21.2 以降で対応予定）
- ウォッチ式（`evaluate` リクエスト）
- 複数スレッド対応（Favnir は現在シングルスレッド VM）
- マルチ接続 DAP セッション

### DAP イベント（VM → クライアント）

| DAP イベント | タイミング |
|---|---|
| `initialized` | `initialize` 応答後 |
| `stopped` | ブレークポイントヒット / ステップ完了 |
| `continued` | 実行再開 |
| `terminated` | パイプライン実行完了 |
| `output` | `IO.println` 出力（`category: "stdout"`） |

---

## CLI 仕様

```bash
# DAP サーバーを起動（ポート 5678、VS Code が接続）
fav dap [--port 5678]

# デバッグモードで実行（DAP 接続待機）
fav run --debug [--dap-port 5678] <file.fav>
```

---

## VS Code 設定例（`launch.json`）

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "favnir",
      "request": "launch",
      "name": "Debug Favnir",
      "program": "${workspaceFolder}/src/pipeline.fav",
      "debugServer": 5678
    }
  ]
}
```

---

## VM への計装（`--debug` フラグ）

`--debug` フラグが付いている場合、VM の各 stage 境界で `DapSession` にフックを送る。

```rust
// vm.rs の stage 実行前後にフック
if self.debug_mode {
    self.dap_hook(DapHook::StageEnter { name, line, locals });
    // ...stage 実行...
    self.dap_hook(DapHook::StageExit { result });
}
```

`debug_mode` が false の場合はブランチが最適化で除去される（ゼロコスト）。

---

## 変数インスペクション

binding の値は `vmvalue_repr` を使って文字列化する。型名は `vmvalue_type_name` を使用。

```
Locals (stage: Transform):
  row       Record { id: 42, name: "Alice", amount: 1500.0 }
  result    Record { id: 42, label: "premium" }
```

---

## テスト（v211000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_21_1_0` | Cargo.toml に `"21.1.0"` が含まれる |
| `dap_protocol_initialize_request_parses` | `initialize` リクエスト JSON が正しくパースできる |
| `dap_protocol_response_serializes` | DAP Response が正しく JSON にシリアライズできる |
| `dap_session_breakpoint_set_and_hit` | ブレークポイントを設定して `is_breakpoint` が true になる |
| `dap_adapter_stopped_event_format` | `stopped` イベントの JSON フォーマットが正しい |

---

## 完了条件

- [ ] `fav dap` コマンドが TCP ポート 5678 でリッスンを開始する
- [ ] `fav run --debug` が DAP 接続待機モードで起動する
- [ ] `initialize` / `launch` / `setBreakpoints` / `continue` / `next` の DAP リクエストに応答できる
- [ ] `stopped` イベントが正しいソース行情報とともに送信される
- [ ] `variables` リクエストで現在の binding 名・型・値が返される
- [ ] `--debug` なし実行でオーバーヘッドがゼロ（ブランチが最適化で除去）
- [ ] `site/content/docs/tools/dap.mdx` が存在する
- [ ] `cargo test v211000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
- [ ] `CHANGELOG.md` に v21.1.0 エントリが追加されている
- [ ] `fav/Cargo.toml` version が `21.1.0`
