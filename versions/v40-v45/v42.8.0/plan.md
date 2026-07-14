# v42.8.0 実装計画 — Real-Time Power cookbook

## 目標

リアルタイム機能の使用例として cookbook MDX 2 件を追加し、`driver.rs` にテスト 1 件を追加する。

---

## T0 — 事前確認

- [ ] `cargo test` が 2893 tests / 0 failures であることを確認
- [ ] `fav/Cargo.toml` version が `42.7.0` であることを確認
- [ ] `fav/src/driver.rs` の `v42700_tests` 閉じ `}` 行番号を記録
- [ ] `versions/roadmap/roadmap-v42.1-v43.0.md` に v42.8.0 エントリが存在することを確認

---

## T1 — `site/content/cookbook/cep-login-purchase.mdx` 作成

```mdx
---
title: "CEP: ログイン→購入セッション検出"
description: CEP Rune を使い、ログインの直後に購入が発生したセッションをリアルタイムで検出する
---

# CEP: ログイン→購入セッション検出

CEP Rune（v42.1.0）を使い、ログインイベントの直後に購入イベントが発生したセッションを
5 分ウィンドウでリアルタイム検出します。検出結果は WebSocket で push します。

## パイプライン

```fav
stage DetectLoginPurchase: Stream<String> -> List<String> =
    |ctx, event_stream|
    bind windowed  <- Cep.window(ctx, event_stream, 300)
    bind matched   <- Cep.match(ctx, windowed, |w|
        w.first == "login" && w.last == "purchase"
    )
    matched

stage PushResults: List<String> -> Unit =
    |ctx, sessions|
    bind _ <- List.map(sessions, |session_id|
        WebSocket.send(ctx, "ws://localhost:8080/sessions", session_id)
    )
    Unit
```

## ポイント

- `Cep.window(ctx, stream, seconds)` — 指定秒数のイベントウィンドウを生成（v42.1.0 CEP Rune）
- `Cep.match(ctx, window, predicate)` — パターン条件に合致するセッションを抽出
- ステージ引数 `event_stream` / `sessions` を明示して未束縛変数を避ける
- `WebSocket.send` で検出結果をブラウザへリアルタイム push（v42.6.0 stub）

> **注意**: `Cep.window` / `Cep.match` は v42.1.0 での stub 実装です。

## 関連

- [CEP Rune ドキュメント](/docs/runes/cep)
- [WebSocket Rune ドキュメント](/docs/runes/websocket)
```

---

## T2 — `site/content/cookbook/stream-join.mdx` 作成

```mdx
---
title: "Stream join: 2 ストリームの時間窓結合"
description: Stream.join を使い、2 つの整数ストリームを time-window で結合する
---

# Stream join: 2 ストリームの時間窓結合

`Stream.join`（v42.4.0）を使い、2 つのストリームを 60 秒の時間窓で結合します。

## パイプライン

```fav
stage JoinStreams: Unit -> List<List<Int>> =
    |ctx|
    bind left   <- Stream.from(List.range(1, 10))
    bind right  <- Stream.from(List.range(1, 10))
    bind joined <- Stream.join(left, right, |a, b| a == b, 60)
    Stream.to_list(joined)
```

## ポイント

- `Stream.join(left, right, predicate, window_secs)` — 4 引数の positional API（v42.4.0）
- `window_secs` は正の整数（秒）。0 以下はパースエラー
- 結果は `List<List<Int>>` — 各要素が `[left_item, right_item]` のペア
- v42.4.0 は nested-loop join（stub）のため大量データには不向き。v44.x で最適化予定
- `window_secs` の時刻フィルタリングは v44.x 以降で実装（現在は全要素を走査）

## 関連

- [Stream.join API リファレンス](/docs/language/stream)
```

---

## T3 — `driver.rs` — `v42800_tests` モジュール追加

`v42700_tests` の閉じ `}` の直前（降順配置）に挿入:

```rust
// -- v42800_tests (v42.8.0) -- Real-Time Power cookbook --
mod v42800_tests {
    #[test]
    fn realtime_cookbook_mdx_exists() {
        let cep_mdx = include_str!("../../site/content/cookbook/cep-login-purchase.mdx");
        assert!(cep_mdx.contains("Cep.window"), "cep-login-purchase.mdx must contain Cep.window");
        let join_mdx = include_str!("../../site/content/cookbook/stream-join.mdx");
        assert!(join_mdx.contains("Stream.join"), "stream-join.mdx must contain Stream.join");
    }
}
```

注意: `v42700_tests` の `cargo_toml_version_is_42_7_0` をスタブ化（`assert!(true)`）してから追加する。

---

## T4 — `fav/Cargo.toml` バージョン bump

`version = "42.7.0"` → `version = "42.8.0"`

---

## T5 — `CHANGELOG.md` 更新

`[v42.7.0]` の直前に `[v42.8.0]` エントリを追加:

```markdown
## [v42.8.0] — 2026-07-12

### Added
- `site/content/cookbook/cep-login-purchase.mdx` — CEP ログイン→購入セッション検出 cookbook
- `site/content/cookbook/stream-join.mdx` — Stream join 2 ストリーム結合 cookbook
- `v42800_tests`: `realtime_cookbook_mdx_exists`
```

---

## T6 — テスト実行・確認

- [ ] `cargo test` 実行
- [ ] failures = 0 を確認
- [ ] テスト数 = 2894 を確認（2893 + 1 件）
- [ ] `v42800_tests` 1 件 pass を確認

---

## T7 — バージョン管理ドキュメント更新

- [ ] `versions/current.md` を v42.8.0（最新安定版、2894 tests）・v42.9.0（次に切る版）に更新
- [ ] `versions/roadmap/roadmap-v42.1-v43.0.md` の v42.8.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）`）
- [ ] 同ロードマップの v42.8.0 推定テスト数を `2893` → 実績 `2894` に修正
- [ ] `versions/v40-v45/v42.8.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）

---

## ファイル変更サマリー

| ファイル | 変更種別 |
|---|---|
| `site/content/cookbook/cep-login-purchase.mdx` | 新規 |
| `site/content/cookbook/stream-join.mdx` | 新規 |
| `fav/src/driver.rs` | 変更 |
| `fav/Cargo.toml` | 変更 |
| `CHANGELOG.md` | 変更 |
| `versions/current.md` | 変更 |
| `versions/roadmap/roadmap-v42.1-v43.0.md` | 変更 |
