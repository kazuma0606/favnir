# v42.6.0 仕様書 — WebSocket Rune

## 概要

リアルタイム push sink として `runes/websocket/` を追加する。
`WebSocket.send(ctx, url, message)` と `WebSocket.broadcast(ctx, url, messages)` の 2 関数を提供する。
VM プリミティブ（`WebSocket.send_raw` / `WebSocket.broadcast_raw`）を stub 実装し、将来の実 WebSocket 接続実装への足がかりとする。

---

## 背景・動機

v42.1〜v42.5 で CEP・Stream join・back-pressure を整備した。
リアルタイムパイプラインの最終段として、処理結果をブラウザやクライアントへリアルタイムに push する WebSocket sink が必要。
v42.6.0 ではファイル構造・関数シグネチャ・VM プリミティブを整備し、実際の WebSocket 接続は v44.x 以降で実装する。

---

## 実装スコープ

### 1. `runes/websocket/rune.toml`

```toml
[rune]
name = "websocket"
version = "42.6.0"
entry = "websocket.fav"
description = "WebSocket Rune — リアルタイム push sink（WebSocket.send / WebSocket.broadcast）"
```

### 2. `runes/websocket/websocket.fav`

```favnir
// runes/websocket/websocket.fav — WebSocket push sink (v42.6.0)
//
// Usage:
//   bind _ <- WebSocket.send(ctx, "ws://host/path", message)
//   bind _ <- WebSocket.broadcast(ctx, "ws://host/path", messages)

// 単一メッセージを WebSocket で送信する。
// Returns Ok(()) on success, Err(message) on failure.
public fn send(ctx: AppCtx, url: String, message: String) -> Result<Unit, String> {
    WebSocket.send_raw(ctx, url, message)
}

// 複数メッセージを WebSocket で一斉送信する（broadcast）。
// Returns Ok(count) — 送信成功数（stub では messages の長さを返す）。
public fn broadcast(ctx: AppCtx, url: String, messages: List<String>) -> Result<Int, String> {
    WebSocket.broadcast_raw(ctx, url, messages)
}
```

`broadcast` は `WebSocket.broadcast_raw` を直接呼び出す。
stub では messages リストの長さを成功数として返す。実接続実装（v44.x）時に実際の送信成否を集計する予定。

### 3. `vm.rs` — `WebSocket.send_raw` / `WebSocket.broadcast_raw` プリミティブ追加

既存の `"Email.send_raw"` ブロックの直後に追加。どちらも stub（実際の接続は行わず `Ok(Unit)` / `Ok(Int(n))` を返す）:

```rust
// ── v42.6.0: WebSocket primitives ────────────────────────────────────────────
"WebSocket.send_raw" => {
    if args.len() != 3 {
        return Err(self.error(artifact, "WebSocket.send_raw requires 3 arguments: (ctx, url, message)"));
    }
    // stub: 実接続は v44.x 以降。ctx/url/message を消費して Ok(Unit) を返す。
    Ok(VMValue::Result(Ok(Box::new(VMValue::Unit))))
}
"WebSocket.broadcast_raw" => {
    if args.len() != 3 {
        return Err(self.error(artifact, "WebSocket.broadcast_raw requires 3 arguments: (ctx, url, messages)"));
    }
    // stub: messages リストの長さを成功数として返す。
    let mut it = args.into_iter();
    let _ctx = it.next().expect("ctx");
    let _url = it.next().expect("url");
    let msgs = it.next().expect("messages");
    let count = match msgs {
        VMValue::List(list) => list.len() as i64,
        _ => 0,
    };
    Ok(VMValue::Result(Ok(Box::new(VMValue::Int(count)))))
}
```

### 4. `driver.rs` — `v42600_tests` 追加（2 テスト）

```rust
// -- v42600_tests (v42.6.0) -- WebSocket Rune --
mod v42600_tests {
    fn cargo_toml_version_is_42_6_0()
    fn websocket_rune_fav_exists()    // rune.toml + websocket.fav + websocket.mdx の内容を確認
}
```

`websocket_rune_fav_exists`:
- `include_str!("../../runes/websocket/websocket.fav")` が `WebSocket.send_raw` を含む
- `include_str!("../../runes/websocket/websocket.fav")` が `broadcast` を含む
- `include_str!("../../runes/websocket/rune.toml")` が `"websocket"` を含む
- `include_str!("../../site/content/docs/runes/websocket.mdx")` が `"WebSocket"` を含む

### 5. `site/content/docs/runes/websocket.mdx`

他 Rune（email, kafka, snowflake 等）に倣い、関数シグネチャ・使用例・非スコープ事項（実接続は v44.x）を記載する簡易 MDX を作成する。

---

## テスト計画

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_42_6_0` | Cargo.toml に "42.6.0" が含まれる |
| `websocket_rune_fav_exists` | `websocket.fav` が `WebSocket.send_raw` と `broadcast` を含み、`rune.toml` が `websocket` を含み、`websocket.mdx` が `WebSocket` を含む |

**推定テスト数**: 2889 + 2 = **2891**
（v42.5.0 実績 2889 を基準。ロードマップ記載の 2890 は v42.5.0 でネガティブテストが 1 件追加されたことによる誤差）

---

## 影響範囲

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `runes/websocket/rune.toml` | 新規 | Rune メタデータ |
| `runes/websocket/websocket.fav` | 新規 | `send` / `broadcast` 関数（`WebSocket.send_raw` / `WebSocket.broadcast_raw` を呼び出す） |
| `site/content/docs/runes/websocket.mdx` | 新規 | WebSocket Rune ドキュメント（関数シグネチャ・使用例・非スコープ） |
| `fav/src/backend/vm.rs` | 変更 | `WebSocket.send_raw` / `WebSocket.broadcast_raw` stub プリミティブ追加 |
| `fav/src/driver.rs` | 変更 | `v42600_tests` 2 件追加 |
| `fav/Cargo.toml` | 変更 | version `42.5.0` → `42.6.0` |
| `CHANGELOG.md` | 変更 | `[v42.6.0]` エントリ追加 |
| `versions/current.md` | 変更 | 最新安定版 v42.6.0・次版 v42.7.0 に更新 |

---

## 非スコープ

- 実際の WebSocket 接続（TCP / TLS / WS handshake）— v44.x 以降
- `!WebSocket` エフェクト — checker.rs / Effect enum 追加（実接続実装時に合わせて追加）
- WebSocket クライアント認証（Bearer token 等）
- ping/pong / keep-alive 制御
- `runes/websocket/` の npm/cargo 依存ライブラリ追加（`tungstenite` 等）— v44.x 以降
- `broadcast` の送信成否個別集計（stub では messages 長を全成功として返す）— v44.x 以降
