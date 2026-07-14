# v42.9.0 実装計画 — v43.0 前調整・安定化

## 目標

コードフリーズ。`real-time-power.mdx` ドキュメントを追加し、meta テスト 2 件を追加する。

---

## T0 — 事前確認

- [ ] `cargo test` が 2894 tests / 0 failures であることを確認
- [ ] `fav/Cargo.toml` version が `42.8.0` であることを確認
- [ ] `fav/src/driver.rs` の `v42800_tests` 閉じ `}` 行番号を記録
- [ ] `versions/roadmap/roadmap-v42.1-v43.0.md` に v42.9.0 エントリが存在することを確認

---

## T1 — `site/content/docs/real-time-power.mdx` 作成

```mdx
---
title: "Real-Time Power — Favnir v42.x リアルタイム機能概要"
description: "v42.x で整備したリアルタイムパイプライン機能（CEP / Stream join / back-pressure / WebSocket / fav monitor）の概要"
---

# Real-Time Power — Favnir v42.x リアルタイム機能概要

Favnir v42.x では、リアルタイムデータパイプラインを構築するための機能群を整備しました。

## CEP — Complex Event Processing（v42.1.0）

イベントストリームからパターンを検出します。

```fav
bind windowed <- Cep.window(ctx, event_stream, 300)
bind matched  <- Cep.match(ctx, windowed, |w| w.first == "login" && w.last == "purchase")
```

詳細: [CEP cookbook](/cookbook/cep-login-purchase)

## Stream join — 時間窓 2 ストリーム結合（v42.4.0）

2 つのストリームを時間窓で結合します。

```fav
bind joined <- Stream.join(left, right, |a, b| a == b, 60)
```

> **注意（stub）**: join キーの型安全チェックはイベント型システム整備後の v43.x 以降に延期。現バージョンは `Stream<Unknown>` 型推論のみ。

詳細: [Stream join cookbook](/cookbook/stream-join)

## Back-pressure — `#[max_inflight(n)]`（v42.5.0）

stage の最大同時実行数を宣言的に指定します（parser + AST のみ。runtime は v44.x で実装）。

```fav
#[max_inflight(10)]
stage ProcessEvents: Stream<Event> -> Stream<Result> = ...
```

## WebSocket Rune — リアルタイム push sink（v42.6.0）

処理結果をブラウザやクライアントへリアルタイムに push します（v42.6.0 は stub）。

```fav
bind _ <- WebSocket.send(ctx, "ws://localhost:8080/events", message)
```

詳細: [WebSocket Rune ドキュメント](/docs/runes/websocket)

## fav monitor — パイプライン監視（v42.7.0）

実行中パイプラインのメトリクスをターミナルに表示します（v42.7.0 は stub。実測は v43.x）。

```sh
fav monitor
```
```

---

## T2 — `driver.rs` — `v42900_tests` モジュール追加

`v42800_tests` の閉じ `}` の直前（降順配置）に挿入:

```rust
// -- v42900_tests (v42.9.0) -- v43.0 前調整・安定化 --
mod v42900_tests {
    #[test]
    fn cargo_toml_version_is_42_9_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("42.9.0"), "Cargo.toml must contain 42.9.0");
    }

    #[test]
    fn real_time_power_docs_exists() {
        let mdx = include_str!("../../site/content/docs/real-time-power.mdx");
        assert!(mdx.contains("Stream.join"), "real-time-power.mdx must contain Stream.join");
        assert!(mdx.contains("Cep.window"), "real-time-power.mdx must contain Cep.window");
        assert!(mdx.contains("WebSocket"), "real-time-power.mdx must contain WebSocket");
    }
}
```

注意: `v42800_tests` の `realtime_cookbook_mdx_exists` に NOTE コメントはないが、`cargo_toml_version` テストを持つ `v42900_tests` の version assert は次バージョン（v43.0.0）で必ずスタブ化すること。

---

## T3 — `fav/Cargo.toml` バージョン bump

`version = "42.8.0"` → `version = "42.9.0"`

---

## T4 — `CHANGELOG.md` 更新

`[v42.8.0]` の直前に `[v42.9.0]` エントリを追加:

```markdown
## [v42.9.0] — 2026-07-12

### Added
- `site/content/docs/real-time-power.mdx` — v42.x リアルタイム機能概要ドキュメント（CEP / Stream join / back-pressure / WebSocket / fav monitor）
- `v42900_tests`: `cargo_toml_version_is_42_9_0` / `real_time_power_docs_exists`

### Notes
- コードフリーズ（新規機能追加なし）
- v43.0.0 マイルストーン宣言は次バージョンで実施
```

---

## T5 — テスト実行・確認

- [ ] `cargo test` 実行
- [ ] failures = 0 を確認
- [ ] テスト数 = 2896 を確認（2894 + 2 件）
- [ ] `v42900_tests` 2 件 pass を確認

---

## T6 — バージョン管理ドキュメント更新

- [ ] `versions/current.md` を v42.9.0（最新安定版、2896 tests）・v43.0.0（次に切る版）に更新
- [ ] `versions/roadmap/roadmap-v42.1-v43.0.md` の v42.9.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）`）
- [ ] 同ロードマップの v42.9.0 推定テスト数を `2895` → 実績 `2896` に修正
- [ ] `versions/v40-v45/v42.9.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）

---

## ファイル変更サマリー

| ファイル | 変更種別 |
|---|---|
| `site/content/docs/real-time-power.mdx` | 新規 |
| `fav/src/driver.rs` | 変更 |
| `fav/Cargo.toml` | 変更 |
| `CHANGELOG.md` | 変更 |
| `versions/current.md` | 変更 |
| `versions/roadmap/roadmap-v42.1-v43.0.md` | 変更 |
