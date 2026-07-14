# v42.6.0 実装計画 — WebSocket Rune

## 目標

WebSocket push sink Rune を追加する。
`runes/websocket/` ディレクトリ作成、VM プリミティブ stub 追加、MDX ドキュメント作成、driver.rs テスト 2 件追加。

---

## T0 — 事前確認

- [ ] `cargo test` が 2889 tests / 0 failures であることを確認
- [ ] `fav/Cargo.toml` version が `42.5.0` であることを確認
- [ ] `fav/src/backend/vm.rs` の `Email.send_raw` ブロック行番号を記録
- [ ] `fav/src/driver.rs` の `v42500_tests` 閉じ `}` 行番号を記録
- [ ] `versions/roadmap/roadmap-v42.1-v43.0.md` に v42.6.0 エントリが存在することを確認

---

## T1 — `runes/websocket/rune.toml` 作成

```toml
[rune]
name = "websocket"
version = "42.6.0"
entry = "websocket.fav"
description = "WebSocket Rune — リアルタイム push sink（WebSocket.send / WebSocket.broadcast）"
```

---

## T2 — `runes/websocket/websocket.fav` 作成

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

---

## T3 — `vm.rs` — `WebSocket.send_raw` / `WebSocket.broadcast_raw` stub 追加

`Email.send_raw` ブロックの直後に追加:

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

---

## T4 — `site/content/docs/runes/websocket.mdx` 作成

他 Rune の MDX に倣い最小限の構成で作成:

```mdx
---
title: WebSocket Rune
description: WebSocket リアルタイム push sink（v42.6.0）
---

# WebSocket Rune

リアルタイム push sink として WebSocket へメッセージを送信します。

## 関数

### `WebSocket.send`

```favnir
fn send(ctx: AppCtx, url: String, message: String) -> Result<Unit, String>
```

単一メッセージを指定 URL の WebSocket エンドポイントへ送信します。

### `WebSocket.broadcast`

```favnir
fn broadcast(ctx: AppCtx, url: String, messages: List<String>) -> Result<Int, String>
```

複数メッセージを一斉送信します。返り値は送信成功数です。

## 使用例

```favnir
bind _ <- WebSocket.send(ctx, "ws://localhost:8080/events", "hello")
```

## 注意事項

- v42.6.0 では stub 実装です。実際の WebSocket 接続は v44.x 以降で実装されます。
- `!WebSocket` エフェクトは実接続実装時に追加予定です。
```

---

## T5 — `driver.rs` — `v42600_tests` モジュール追加

`v42500_tests` の閉じ `}` の直前（降順配置）に挿入:

```rust
// -- v42600_tests (v42.6.0) -- WebSocket Rune --
mod v42600_tests {
    #[test]
    fn cargo_toml_version_is_42_6_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(toml.contains("42.6.0"), "Cargo.toml must contain 42.6.0");
    }

    #[test]
    fn websocket_rune_fav_exists() {
        let fav_src = include_str!("../../runes/websocket/websocket.fav");
        assert!(fav_src.contains("WebSocket.send_raw"), "websocket.fav must contain WebSocket.send_raw");
        assert!(fav_src.contains("broadcast"), "websocket.fav must contain broadcast");
        let rune_toml = include_str!("../../runes/websocket/rune.toml");
        assert!(rune_toml.contains("websocket"), "rune.toml must contain websocket");
        let mdx = include_str!("../../site/content/docs/runes/websocket.mdx");
        assert!(mdx.contains("WebSocket"), "websocket.mdx must contain WebSocket");
    }
}
```

注意: `v42500_tests` の `cargo_toml_version_is_42_5_0` をスタブ化（`assert!(true)`）してから追加する。

---

## T6 — `fav/Cargo.toml` バージョン bump

`version = "42.5.0"` → `version = "42.6.0"`

---

## T7 — `CHANGELOG.md` 更新

`[v42.5.0]` の直前に `[v42.6.0]` エントリを追加:

```markdown
## [v42.6.0] — 2026-07-12

### Added
- `runes/websocket/` — WebSocket push sink Rune（`send` / `broadcast` 関数）
- VM プリミティブ `WebSocket.send_raw` / `WebSocket.broadcast_raw` stub 追加（実接続は v44.x 以降）
- `site/content/docs/runes/websocket.mdx` — WebSocket Rune ドキュメント

### Notes
- 実際の WebSocket 接続（TCP / TLS / WS handshake）は v44.x 以降で実装
- `!WebSocket` エフェクトは実接続実装時に合わせて追加予定
```

---

## T8 — テスト実行・確認

- [ ] `cargo test` 実行
- [ ] failures = 0 を確認
- [ ] テスト数 = 2891 を確認（2889 + 2 件）
- [ ] `v42600_tests` 2 件 pass を確認

---

## T9 — バージョン管理ドキュメント更新

- [ ] `versions/current.md` を v42.6.0（最新安定版、2891 tests）・v42.7.0（次に切る版）に更新
- [ ] `versions/roadmap/roadmap-v42.1-v43.0.md` の v42.6.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）`）、実績テスト数 2891 を記録
- [ ] `versions/v40-v45/v42.6.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）

---

## ファイル変更サマリー

| ファイル | 変更種別 |
|---|---|
| `runes/websocket/rune.toml` | 新規 |
| `runes/websocket/websocket.fav` | 新規 |
| `site/content/docs/runes/websocket.mdx` | 新規 |
| `fav/src/backend/vm.rs` | 変更 |
| `fav/src/driver.rs` | 変更 |
| `fav/Cargo.toml` | 変更 |
| `CHANGELOG.md` | 変更 |
| `versions/current.md` | 変更 |
| `versions/roadmap/roadmap-v42.1-v43.0.md` | 変更 |
